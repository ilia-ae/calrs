//! The changes behind the write tools, as plain async functions returning
//! DTOs read back from the database.
//!
//! Every change holds `AgentCtx::write_lock` from its first read to its last,
//! touches only the acting user's personal event types (found by slug, then
//! addressed by id), and leaves one `change` audit event with the settings it
//! changed, before and after. Free text (titles, descriptions, locations,
//! names) appears there only as `(text)`.
//!
//! Two ways of writing:
//! - Creating an event type and the general edit go through the web form's
//!   own handlers (`event_types.rs`), fed the form a browser would post, so
//!   the same checks, defaults and side tables apply. An edit posts the whole
//!   stored state with the patch applied, after checking the version token.
//!   The handlers report little and swallow some errors, so the result is
//!   always read back from the database.
//! - Weekly hours, date overrides, on/off and the profile are exact
//!   statements on the rows involved.

use super::audit;
use super::dto::*;
use super::error::{db, ToolError};
use super::ops_read::{self, col, personal_event_type};
use super::sanitize::is_invisible;
use super::server::Session;
use super::snapshot;
use super::{AgentCtx, OWNER_TEXT_MAX};
use crate::auth::AuthUser;
use crate::booking_window::{today_in, MAX_DAYS_AHEAD};
use crate::models::User;
use crate::web::availability::{build_avail_schedule, event_type_followed_tz, get_host_tz};
use crate::web::event_types::{parse_bookable_range, validate_event_type_form, EventTypeForm};
use axum::body::Body;
use axum::extract::{Form, FromRequest, Path, State};
use axum::http::{header, HeaderMap, HeaderValue, Method, Request};
use axum::response::{IntoResponse, Response};
use chrono::{NaiveDate, NaiveTime};
use chrono_tz::Tz;
use serde_json::{json, Map, Value};
use sqlx::SqlitePool;

/// Longest title, as the web form allows.
const TITLE_MAX: usize = 255;
const SLUG_MAX: usize = 100;
const LOCATION_MAX: usize = 500;
const PATTERN_MAX: usize = 200;
const WINDOWS_PER_DAY: usize = 10;
const DAY_MINUTES: i64 = 24 * 60;
const YEAR_MINUTES: i64 = 365 * DAY_MINUTES;
const PENDING_EXPIRY_MAX_HOURS: i64 = 365 * 24;
const REMINDER_MAX_MINUTES: i64 = 30 * DAY_MINUTES;
const GUESTS_MAX: i64 = 100;
/// Most event types `set_profile` lists.
const FOLLOWING_MAX: i64 = 200;
/// What the web's new event type form starts with (`new_event_type_form`):
/// a reminder a day before and an hour's notice.
pub(crate) const NEW_FORM_REMINDER_MINUTES: i64 = 24 * 60;
pub(crate) const NEW_FORM_MIN_NOTICE_MIN: i64 = 60;
/// How free text appears in the audit: that it is there, not what it says.
const TEXT: &str = "(text)";

/// One `availability_rules` row: day of week (0 is Sunday), start, end.
pub(crate) type Rule = (i32, String, String);

/// Who is writing: the shared context, the session (for the audit) and the
/// acting admin.
pub(crate) struct Writer<'a> {
    pub(crate) ctx: &'a AgentCtx,
    pub(crate) session: &'a Session,
    pub(crate) me: &'a User,
}

/// A `Writer`'s parts, owned, for a change that runs in a task of its own
/// (`AgentServer::run_to_end`).
pub(crate) struct Actor {
    pub(crate) ctx: std::sync::Arc<AgentCtx>,
    pub(crate) session: std::sync::Arc<Session>,
    pub(crate) me: User,
}

impl Actor {
    pub(crate) fn writer(&self) -> Writer<'_> {
        Writer {
            ctx: &self.ctx,
            session: &self.session,
            me: &self.me,
        }
    }
}

impl Writer<'_> {
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.ctx.state.pool
    }

    /// The `change` audit event of one write.
    pub(crate) fn audit(
        &self,
        tool: &str,
        object: &str,
        before: Map<String, Value>,
        after: Map<String, Value>,
    ) {
        audit::change(
            self.session,
            tool,
            object,
            &Value::Object(before).to_string(),
            &Value::Object(after).to_string(),
        );
    }
}

// --- Settings, for the audit and the `changed` lists ---

/// One setting as the audit and the `changed` lists see it.
struct Setting {
    name: &'static str,
    value: Value,
    /// Free text, logged only as `TEXT`.
    text: bool,
}

/// What an audit event may say about `setting`.
fn audit_value(setting: &Setting) -> Value {
    if setting.text && !setting.value.is_null() {
        Value::from(TEXT)
    } else {
        setting.value.clone()
    }
}

/// Every setting, as audit values.
fn audit_map(settings: &[Setting]) -> Map<String, Value> {
    settings
        .iter()
        .map(|setting| (setting.name.to_string(), audit_value(setting)))
        .collect()
}

/// The names of the settings that differ between `before` and `after` (the
/// same settings in the same order), and their audit values on each side.
fn diff(
    before: &[Setting],
    after: &[Setting],
) -> (Vec<String>, Map<String, Value>, Map<String, Value>) {
    let mut changed = Vec::new();
    let mut old = Map::new();
    let mut new = Map::new();
    for (b, a) in before.iter().zip(after) {
        if b.value != a.value {
            changed.push(b.name.to_string());
            old.insert(b.name.to_string(), audit_value(b));
            new.insert(a.name.to_string(), audit_value(a));
        }
    }
    (changed, old, new)
}

// --- Checks on arguments ---

fn check_range(field: &str, value: i64, min: i64, max: i64) -> Result<i64, ToolError> {
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(ToolError::invalid(field, format!("must be {min} to {max}")))
    }
}

/// A slug exactly as the web form would store it: lowercase letters, digits
/// and dashes, with at least one letter or digit.
pub(crate) fn check_slug(field: &str, slug: &str) -> Result<String, ToolError> {
    let valid = !slug.is_empty()
        && slug.len() <= SLUG_MAX
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && slug.chars().any(|c| c.is_ascii_alphanumeric());
    if valid {
        Ok(slug.to_string())
    } else {
        Err(ToolError::invalid(
            field,
            format!("use 1 to {SLUG_MAX} lowercase letters, digits and dashes"),
        ))
    }
}

/// Owner text must show what it says: no characters that hide or reorder it.
fn check_visible(field: &str, text: &str) -> Result<(), ToolError> {
    if text.chars().any(is_invisible) {
        return Err(ToolError::invalid(
            field,
            "must not contain invisible formatting characters",
        ));
    }
    Ok(())
}

/// One line of owner text: trimmed, no control or invisible characters, at
/// most `max` characters. Empty becomes `None`.
pub(crate) fn check_line(field: &str, text: &str, max: usize) -> Result<Option<String>, ToolError> {
    let text = text.trim();
    if text.chars().any(char::is_control) {
        return Err(ToolError::invalid(field, "must be one line"));
    }
    check_visible(field, text)?;
    if text.chars().count() > max {
        return Err(ToolError::invalid(
            field,
            format!("at most {max} characters"),
        ));
    }
    Ok((!text.is_empty()).then(|| text.to_string()))
}

fn check_title(field: &str, title: &str) -> Result<String, ToolError> {
    check_line(field, title, TITLE_MAX)?
        .ok_or_else(|| ToolError::invalid(field, "must not be empty"))
}

/// A description: Markdown of at most `OWNER_TEXT_MAX` characters whose
/// links are http, https or mailto. Empty becomes `None`.
fn check_description(field: &str, text: &str) -> Result<Option<String>, ToolError> {
    let text = text.trim();
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(ToolError::invalid(
            field,
            "must not contain control characters",
        ));
    }
    check_visible(field, text)?;
    if text.chars().count() > OWNER_TEXT_MAX {
        return Err(ToolError::invalid(
            field,
            format!("at most {OWNER_TEXT_MAX} characters"),
        ));
    }
    check_markdown_links(field, text)?;
    Ok((!text.is_empty()).then(|| text.to_string()))
}

/// Whether `url` is an absolute http, https or mailto URL. Browsers ignore
/// whitespace and control characters inside a scheme, so those are dropped
/// before it is read.
fn allowed_url(url: &str) -> bool {
    let cleaned = url
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect::<String>()
        .to_ascii_lowercase();
    ["http://", "https://", "mailto:"]
        .iter()
        .any(|scheme| cleaned.len() > scheme.len() && cleaned.starts_with(scheme))
}

/// Every link and image in Markdown `text`, as the public page parses it
/// (`utils::render_inline_markdown`), must point to an http, https or mailto
/// URL. The page drops other links when it renders; here they are refused.
fn check_markdown_links(field: &str, text: &str) -> Result<(), ToolError> {
    use pulldown_cmark::{Event, LinkType, Options, Parser, Tag};
    let safe = Parser::new_ext(text, Options::ENABLE_STRIKETHROUGH).all(|event| match event {
        // `<name@example.com>`, rendered as a mailto link.
        Event::Start(Tag::Link {
            link_type: LinkType::Email,
            ..
        }) => true,
        Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
            allowed_url(&dest_url)
        }
        _ => true,
    });
    if safe {
        Ok(())
    } else {
        Err(ToolError::invalid(
            field,
            "links must be http, https or mailto URLs",
        ))
    }
}

/// The location value a location type needs: nothing for the types that
/// make a link per booking, else some text, and for `link` an http, https or
/// mailto URL.
fn check_location(field: &str, location_type: &str, value: Option<&str>) -> Result<(), ToolError> {
    if crate::web::meeting::is_auto_location(location_type) {
        return Ok(());
    }
    let Some(value) = value.filter(|v| !v.trim().is_empty()) else {
        return Err(ToolError::invalid(field, "required for this location_type"));
    };
    if location_type == "link" && (value.chars().any(char::is_whitespace) || !allowed_url(value)) {
        return Err(ToolError::invalid(
            field,
            "for location_type link, an http, https or mailto URL",
        ));
    }
    Ok(())
}

fn check_timezone(field: &str, name: &str) -> Result<Tz, ToolError> {
    name.trim()
        .parse::<Tz>()
        .map_err(|_| ToolError::invalid(field, "not an IANA timezone name such as Europe/Berlin"))
}

fn check_date(field: &str, date: &str) -> Result<NaiveDate, ToolError> {
    let date = date.trim();
    (date.len() == 10)
        .then(|| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
        .flatten()
        .ok_or_else(|| ToolError::invalid(field, "use YYYY-MM-DD"))
}

/// A time of day written exactly as `HH:MM`.
fn check_time(field: &str, time: &str) -> Result<NaiveTime, ToolError> {
    let time = time.trim();
    let shaped = time.len() == 5
        && time.bytes().enumerate().all(|(i, b)| {
            if i == 2 {
                b == b':'
            } else {
                b.is_ascii_digit()
            }
        });
    shaped
        .then(|| NaiveTime::parse_from_str(time, "%H:%M").ok())
        .flatten()
        .ok_or_else(|| ToolError::invalid(field, "use HH:MM, 00:00 to 23:59"))
}

fn hhmm(time: NaiveTime) -> String {
    time.format("%H:%M").to_string()
}

/// Weekly hours as `availability_rules` rows, sorted. Each day at most once,
/// at most `WINDOWS_PER_DAY` windows a day, each start before its end, no
/// two overlapping. A week with no window at all is refused: the web form
/// would quietly fill it with the owner's default hours.
pub(crate) fn weekly_rules(field: &str, days: &[DayHoursIn]) -> Result<Vec<Rule>, ToolError> {
    let mut seen = Vec::new();
    let mut rules = Vec::new();
    for (i, day) in days.iter().enumerate() {
        let at = format!("{field}[{i}]");
        if seen.contains(&day.day) {
            return Err(ToolError::invalid(
                &format!("{at}.day"),
                "each day may appear once",
            ));
        }
        seen.push(day.day);
        if day.windows.len() > WINDOWS_PER_DAY {
            return Err(ToolError::invalid(
                &format!("{at}.windows"),
                format!("at most {WINDOWS_PER_DAY} windows a day"),
            ));
        }
        let mut windows = Vec::with_capacity(day.windows.len());
        for (j, window) in day.windows.iter().enumerate() {
            let here = format!("{at}.windows[{j}]");
            let start = check_time(&format!("{here}.start"), &window.start)?;
            let end = check_time(&format!("{here}.end"), &window.end)?;
            if start >= end {
                return Err(ToolError::invalid(&here, "start must be before end"));
            }
            windows.push((start, end));
        }
        windows.sort();
        if windows.windows(2).any(|pair| pair[1].0 < pair[0].1) {
            return Err(ToolError::invalid(
                &format!("{at}.windows"),
                "windows must not overlap",
            ));
        }
        let day_of_week = day.day.day_of_week();
        rules.extend(
            windows
                .into_iter()
                .map(|(start, end)| (day_of_week, hhmm(start), hhmm(end))),
        );
    }
    if rules.is_empty() {
        return Err(ToolError::invalid(
            field,
            "no hours in the whole week; give at least one window",
        ));
    }
    rules.sort();
    Ok(rules)
}

/// Rules in the web form's `avail_schedule` format, as the edit page
/// writes it: `1:09:00-12:00,13:00-17:00;2:09:00-17:00`.
pub(crate) fn schedule_string(rules: &[Rule]) -> String {
    build_avail_schedule(rules)
}

/// Frequency limits in the web form's format: `2:day,5:week:m`, where `:m`
/// counts per team member.
fn limits_string(limits: &[(i64, String, bool)]) -> String {
    limits
        .iter()
        .map(|(count, period, per_member)| {
            if *per_member {
                format!("{count}:{period}:m")
            } else {
                format!("{count}:{period}")
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

// --- The web form ---

/// A POST of `fields` to a web form handler as the acting admin: the user
/// its extractor would give, a CSRF cookie whose token the form repeats (the
/// double-submit check, `verify_csrf_token`), and the form parsed the way
/// axum parses a browser's.
pub(crate) async fn web_form<T: serde::de::DeserializeOwned>(
    me: &User,
    fields: &[(&'static str, String)],
) -> Result<(AuthUser, HeaderMap, Form<T>), ToolError> {
    let token = crate::web::generate_csrf_token();
    let cookie = HeaderValue::from_str(&format!("{}={token}", crate::web::CSRF_COOKIE_NAME))
        .map_err(|e| ToolError::internal("build the form cookie", &e))?;
    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, cookie);
    let body = std::iter::once(("_csrf", token.as_str()))
        .chain(fields.iter().map(|(name, value)| (*name, value.as_str())))
        .map(|(name, value)| {
            format!(
                "{}={}",
                urlencoding::encode(name),
                urlencoding::encode(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&");
    let request = Request::builder()
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .map_err(|e| ToolError::internal("build the form request", &e))?;
    let form = Form::<T>::from_request(request, &())
        .await
        .map_err(|e| ToolError::internal("parse the form", &e))?;
    let user = AuthUser {
        user: me.clone(),
        impersonation: None,
        lang: "en",
    };
    Ok((user, headers, form))
}

/// A form handler answers a saved form with a redirect, and a refused one
/// with the form page again or an error page.
fn web_saved(response: &Response) -> bool {
    response.status().is_redirection()
}

/// The handler refused a form the checks here let through.
fn refused_by_form(response: &Response) -> ToolError {
    tracing::warn!(
        target: "calrs::agent",
        status = %response.status(),
        "the event type form refused an agent change"
    );
    ToolError::unavailable(
        "the web form refused the change and nothing was saved; compare the values with get_event_type",
    )
}

/// A web form check's Fluent key as a tool error: its English text, under
/// the field it is about.
fn form_error(key: &str) -> ToolError {
    let field = match key {
        "form-error-event-type-title-required"
        | "form-error-event-type-title-length"
        | "form-error-title-required" => "title",
        "form-error-event-type-duration" => "duration_min",
        "form-error-event-type-horizon" => "booking_horizon_days",
        "form-error-slug-charset"
        | "form-error-event-type-slug-taken"
        | "form-error-event-type-slug-taken-team" => "slug",
        "form-error-bookable-range-order" | "form-error-bookable-range-invalid" => "bookable_from",
        "form-error-location-required" => "location_value",
        _ => "settings",
    };
    ToolError::invalid(field, crate::i18n::translate("en", key, None))
}

/// The web form's own checks, which its handler would answer with the form
/// page again. Run first, so the model learns the reason; they also catch
/// stored values the web form no longer accepts.
pub(crate) async fn web_form_checks(
    pool: &SqlitePool,
    me: &User,
    et_id: Option<&str>,
    form: &EventTypeForm,
) -> Result<(), ToolError> {
    validate_event_type_form(form).map_err(form_error)?;
    parse_bookable_range(form).map_err(form_error)?;
    let location_type = form.location_type.as_deref().unwrap_or("link");
    let location_given = form
        .location_value
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    if !crate::web::meeting::is_auto_location(location_type) && !location_given {
        return Err(form_error("form-error-location-required"));
    }
    let meet_error = crate::web::booking::google_meet_location_error(
        pool,
        location_type,
        Some(&me.id),
        None,
        et_id,
    )
    .await;
    if meet_error.is_some() {
        return Err(ToolError::invalid(
            "location_type",
            "google_meet needs Google Calendar connected, with a calendar chosen for bookings",
        ));
    }
    Ok(())
}

/// Whether another event type of `account_id` has `slug`.
async fn slug_taken(
    pool: &SqlitePool,
    account_id: &str,
    slug: &str,
    except: Option<&str>,
) -> Result<bool, ToolError> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM event_types
                        WHERE account_id = ? AND slug = ? AND id IS NOT ?)",
    )
    .bind(account_id)
    .bind(slug)
    .bind(except)
    .fetch_one(pool)
    .await
    .map_err(db("check the slug"))
}

/// The version token of event type `et_id` now.
pub(super) async fn version_of(pool: &SqlitePool, et_id: &str) -> Result<String, ToolError> {
    snapshot::snapshot(pool, et_id)
        .await
        .map_err(db("snapshot event type"))?
        .map(|snapshot| snapshot::version_token(&snapshot))
        .ok_or_else(|| ToolError::not_found("event type"))
}

pub(super) fn stale_version() -> ToolError {
    ToolError::invalid(
        "version",
        "the event type changed since it was read; call get_event_type again and use its version",
    )
}

/// An edit that found the event type changed in another way while saving.
fn changed_meanwhile() -> ToolError {
    ToolError::unavailable(
        "the event type also changed in another way while this was saved (probably in the \
         web interface at the same moment), so the change may not have been applied as asked; \
         read it again with get_event_type and check it",
    )
}

// --- The edit form's state ---

/// Everything the event type edit page shows and posts back, as stored.
#[derive(Clone, Debug)]
pub(crate) struct EditState {
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) duration_min: i64,
    pub(crate) slot_interval_min: Option<i64>,
    pub(crate) buffer_before: Option<i64>,
    pub(crate) buffer_after: Option<i64>,
    pub(crate) min_notice_min: Option<i64>,
    pub(crate) requires_confirmation: bool,
    pub(crate) pending_blocks_slot: bool,
    pub(crate) pending_expiry_hours: Option<i64>,
    pub(crate) visibility: String,
    pub(crate) location_type: String,
    pub(crate) location_value: Option<String>,
    pub(crate) meeting_pattern_override: Option<String>,
    /// Its own timezone; `None` follows the owner's.
    pub(crate) timezone: Option<String>,
    pub(crate) booking_horizon_days: Option<i64>,
    pub(crate) bookable_from: Option<String>,
    pub(crate) bookable_until: Option<String>,
    pub(crate) reminder_minutes: Option<i64>,
    pub(crate) max_additional_guests: i64,
    pub(crate) first_slot_only: bool,
    pub(crate) default_calendar_view: String,
    pub(crate) week_start: Option<String>,
    pub(crate) cancel_notice_min: Option<i64>,
    pub(crate) reschedule_notice_min: Option<i64>,
    pub(crate) sms_phone_mode: String,
    pub(crate) scheduling_mode: String,
    pub(crate) resource_scheduling_mode: String,
    pub(crate) enabled: bool,
    pub(crate) weekly_hours: Vec<Rule>,
    pub(crate) calendar_ids: Vec<String>,
    pub(crate) resource_ids: Vec<String>,
    pub(crate) frequency_limits: Vec<(i64, String, bool)>,
}

impl EditState {
    pub(crate) async fn load(pool: &SqlitePool, et_id: &str) -> Result<Self, ToolError> {
        let row = sqlx::query(
            "SELECT slug, title, description, duration_min, slot_interval_min, buffer_before,
                    buffer_after, min_notice_min, requires_confirmation, pending_blocks_slot,
                    pending_expiry_hours, visibility, location_type, location_value,
                    meeting_pattern_override, timezone, booking_horizon_days, bookable_from,
                    bookable_until, reminder_minutes, max_additional_guests, first_slot_only,
                    default_calendar_view, week_start, cancel_notice_min, reschedule_notice_min,
                    sms_phone_mode, scheduling_mode, resource_scheduling_mode, enabled
             FROM event_types WHERE id = ?",
        )
        .bind(et_id)
        .fetch_optional(pool)
        .await
        .map_err(db("read event type"))?
        .ok_or_else(|| ToolError::not_found("event type"))?;
        let flag = |name: &str| col::<i64>(&row, name).map(|value| value != 0);

        let weekly_hours: Vec<Rule> = sqlx::query_as(
            "SELECT day_of_week, start_time, end_time FROM availability_rules
             WHERE event_type_id = ? ORDER BY day_of_week, start_time, end_time",
        )
        .bind(et_id)
        .fetch_all(pool)
        .await
        .map_err(db("read weekly hours"))?;
        let calendar_ids: Vec<String> = sqlx::query_scalar(
            "SELECT calendar_id FROM event_type_calendars
             WHERE event_type_id = ? ORDER BY calendar_id",
        )
        .bind(et_id)
        .fetch_all(pool)
        .await
        .map_err(db("read calendar choice"))?;
        let resource_ids: Vec<String> = sqlx::query_scalar(
            "SELECT resource_id FROM event_type_resources
             WHERE event_type_id = ? ORDER BY resource_id",
        )
        .bind(et_id)
        .fetch_all(pool)
        .await
        .map_err(db("read resources"))?;
        let frequency_limits: Vec<(i64, String, i64)> = sqlx::query_as(
            "SELECT max_bookings, period, per_member FROM booking_frequency_limits
             WHERE event_type_id = ? ORDER BY period, max_bookings, per_member",
        )
        .bind(et_id)
        .fetch_all(pool)
        .await
        .map_err(db("read frequency limits"))?;

        Ok(Self {
            slug: col(&row, "slug")?,
            title: col(&row, "title")?,
            description: col(&row, "description")?,
            duration_min: col(&row, "duration_min")?,
            slot_interval_min: col(&row, "slot_interval_min")?,
            buffer_before: col(&row, "buffer_before")?,
            buffer_after: col(&row, "buffer_after")?,
            min_notice_min: col(&row, "min_notice_min")?,
            requires_confirmation: flag("requires_confirmation")?,
            pending_blocks_slot: flag("pending_blocks_slot")?,
            pending_expiry_hours: col(&row, "pending_expiry_hours")?,
            visibility: col(&row, "visibility")?,
            location_type: col(&row, "location_type")?,
            location_value: col(&row, "location_value")?,
            meeting_pattern_override: col(&row, "meeting_pattern_override")?,
            timezone: col::<Option<String>>(&row, "timezone")?.filter(|tz| !tz.is_empty()),
            booking_horizon_days: col(&row, "booking_horizon_days")?,
            bookable_from: col(&row, "bookable_from")?,
            bookable_until: col(&row, "bookable_until")?,
            reminder_minutes: col(&row, "reminder_minutes")?,
            max_additional_guests: col(&row, "max_additional_guests")?,
            first_slot_only: flag("first_slot_only")?,
            default_calendar_view: col(&row, "default_calendar_view")?,
            week_start: col(&row, "week_start")?,
            cancel_notice_min: col(&row, "cancel_notice_min")?,
            reschedule_notice_min: col(&row, "reschedule_notice_min")?,
            sms_phone_mode: col(&row, "sms_phone_mode")?,
            scheduling_mode: col(&row, "scheduling_mode")?,
            resource_scheduling_mode: col(&row, "resource_scheduling_mode")?,
            enabled: flag("enabled")?,
            weekly_hours,
            calendar_ids,
            resource_ids,
            frequency_limits: frequency_limits
                .into_iter()
                .map(|(count, period, per_member)| (count, period, per_member != 0))
                .collect(),
        })
    }

    /// Every setting, in a fixed order, under its column or tool name.
    fn settings(&self) -> Vec<Setting> {
        let plain = |name: &'static str, value: Value| Setting {
            name,
            value,
            text: false,
        };
        let text = |name: &'static str, value: Option<&str>| Setting {
            name,
            value: json!(value),
            text: true,
        };
        vec![
            plain("slug", json!(self.slug)),
            text("title", Some(&self.title)),
            text("description", self.description.as_deref()),
            plain("duration_min", json!(self.duration_min)),
            plain("slot_interval_min", json!(self.slot_interval_min)),
            plain("buffer_before", json!(self.buffer_before)),
            plain("buffer_after", json!(self.buffer_after)),
            plain("min_notice_min", json!(self.min_notice_min)),
            plain("requires_confirmation", json!(self.requires_confirmation)),
            plain("pending_blocks_slot", json!(self.pending_blocks_slot)),
            plain("pending_expiry_hours", json!(self.pending_expiry_hours)),
            plain("visibility", json!(self.visibility)),
            plain("location_type", json!(self.location_type)),
            text("location_value", self.location_value.as_deref()),
            text(
                "meeting_pattern_override",
                self.meeting_pattern_override.as_deref(),
            ),
            plain("timezone", json!(self.timezone)),
            plain("booking_horizon_days", json!(self.booking_horizon_days)),
            plain("bookable_from", json!(self.bookable_from)),
            plain("bookable_until", json!(self.bookable_until)),
            plain("reminder_minutes", json!(self.reminder_minutes)),
            plain("max_additional_guests", json!(self.max_additional_guests)),
            plain("first_slot_only", json!(self.first_slot_only)),
            plain("default_calendar_view", json!(self.default_calendar_view)),
            plain("week_start", json!(self.week_start)),
            plain("cancel_notice_min", json!(self.cancel_notice_min)),
            plain("reschedule_notice_min", json!(self.reschedule_notice_min)),
            plain("sms_phone_mode", json!(self.sms_phone_mode)),
            plain("scheduling_mode", json!(self.scheduling_mode)),
            plain(
                "resource_scheduling_mode",
                json!(self.resource_scheduling_mode),
            ),
            plain("enabled", json!(self.enabled)),
            plain("weekly_hours", json!(schedule_string(&self.weekly_hours))),
            plain("calendar_ids", json!(self.calendar_ids)),
            plain("resource_ids", json!(self.resource_ids)),
            plain(
                "frequency_limits",
                json!(limits_string(&self.frequency_limits)),
            ),
        ]
    }

    /// The form the edit page posts for this state, field by field
    /// (`EventTypeForm`). A NULL timezone is sent as the timezone it follows,
    /// which keeps it NULL (`resolve_event_type_tz`). The pending section and
    /// the week start are sent only when they change: left out, the handler
    /// keeps what is stored.
    pub(crate) fn form_fields(
        &self,
        followed_tz: &str,
        send_pending: bool,
        send_week_start: bool,
    ) -> Vec<(&'static str, String)> {
        let number = |value: Option<i64>| value.map(|v| v.to_string()).unwrap_or_default();
        let text = |value: &Option<String>| value.clone().unwrap_or_default();
        let mut fields = vec![
            ("title", self.title.clone()),
            ("slug", self.slug.clone()),
            ("description", text(&self.description)),
            ("duration_min", self.duration_min.to_string()),
            ("slot_interval_min", number(self.slot_interval_min)),
            ("buffer_before", number(self.buffer_before)),
            ("buffer_after", number(self.buffer_after)),
            ("min_notice_min", number(self.min_notice_min)),
            ("sms_phone_mode", self.sms_phone_mode.clone()),
            ("visibility", self.visibility.clone()),
            ("location_type", self.location_type.clone()),
            ("location_value", text(&self.location_value)),
            (
                "meeting_pattern_override",
                text(&self.meeting_pattern_override),
            ),
            ("avail_schedule", schedule_string(&self.weekly_hours)),
            ("scheduling_mode", self.scheduling_mode.clone()),
            ("calendar_ids", self.calendar_ids.join(",")),
            ("resource_ids", self.resource_ids.join(",")),
            (
                "resource_scheduling_mode",
                self.resource_scheduling_mode.clone(),
            ),
            ("reminder_minutes", number(self.reminder_minutes)),
            (
                "max_additional_guests",
                self.max_additional_guests.to_string(),
            ),
            ("default_calendar_view", self.default_calendar_view.clone()),
            ("frequency_limits", limits_string(&self.frequency_limits)),
            (
                "timezone",
                self.timezone
                    .clone()
                    .unwrap_or_else(|| followed_tz.to_string()),
            ),
            ("cancel_notice_value", number(self.cancel_notice_min)),
            ("cancel_notice_unit", "minutes".to_string()),
            (
                "reschedule_notice_value",
                number(self.reschedule_notice_min),
            ),
            ("reschedule_notice_unit", "minutes".to_string()),
            ("booking_horizon_days", number(self.booking_horizon_days)),
            ("bookable_from", text(&self.bookable_from)),
            ("bookable_until", text(&self.bookable_until)),
        ];
        if self.requires_confirmation {
            fields.push(("requires_confirmation", "on".to_string()));
        }
        if self.first_slot_only {
            fields.push(("first_slot_only", "on".to_string()));
        }
        if send_pending {
            if self.pending_blocks_slot {
                fields.push(("pending_blocks_slot", "on".to_string()));
            }
            let expiry = self
                .pending_expiry_hours
                .map(|hours| hours.to_string())
                .unwrap_or_else(|| "never".to_string());
            fields.push(("pending_expiry_hours", expiry));
        }
        if send_week_start {
            fields.push(("week_start", text(&self.week_start)));
        }
        fields
    }

    /// Check `patch` and apply it. Returns the names of the settings it sets,
    /// changed or not.
    pub(crate) async fn apply(
        &mut self,
        pool: &SqlitePool,
        account_id: &str,
        et_id: &str,
        patch: EventTypePatch,
    ) -> Result<Vec<&'static str>, ToolError> {
        let mut set = Vec::new();
        if let Some(slug) = patch.slug {
            let slug = check_slug("patch.slug", &slug)?;
            if slug != self.slug && slug_taken(pool, account_id, &slug, Some(et_id)).await? {
                return Err(ToolError::invalid(
                    "patch.slug",
                    "already used by another event type",
                ));
            }
            self.slug = slug;
            set.push("slug");
        }
        if let Some(title) = patch.title {
            self.title = check_title("patch.title", &title)?;
            set.push("title");
        }
        if let Some(description) = patch.description {
            self.description = match description {
                Some(text) => check_description("patch.description", &text)?,
                None => None,
            };
            set.push("description");
        }
        if let Some(minutes) = patch.duration_min {
            self.duration_min = check_range("patch.duration_min", minutes, 1, DAY_MINUTES)?;
            set.push("duration_min");
        }
        if let Some(minutes) = patch.slot_interval_min {
            self.slot_interval_min = minutes
                .map(|m| check_range("patch.slot_interval_min", m, 1, DAY_MINUTES))
                .transpose()?;
            set.push("slot_interval_min");
        }
        if let Some(minutes) = patch.buffer_before {
            self.buffer_before = Some(check_range("patch.buffer_before", minutes, 0, DAY_MINUTES)?);
            set.push("buffer_before");
        }
        if let Some(minutes) = patch.buffer_after {
            self.buffer_after = Some(check_range("patch.buffer_after", minutes, 0, DAY_MINUTES)?);
            set.push("buffer_after");
        }
        if let Some(minutes) = patch.min_notice_min {
            self.min_notice_min = Some(check_range(
                "patch.min_notice_min",
                minutes,
                0,
                YEAR_MINUTES,
            )?);
            set.push("min_notice_min");
        }
        if let Some(on) = patch.requires_confirmation {
            self.requires_confirmation = on;
            set.push("requires_confirmation");
        }
        if let Some(on) = patch.pending_blocks_slot {
            self.pending_blocks_slot = on;
            set.push("pending_blocks_slot");
        }
        if let Some(hours) = patch.pending_expiry_hours {
            self.pending_expiry_hours = hours
                .map(|h| check_range("patch.pending_expiry_hours", h, 1, PENDING_EXPIRY_MAX_HOURS))
                .transpose()?;
            set.push("pending_expiry_hours");
        }
        if let Some(visibility) = patch.visibility {
            self.visibility = visibility.key().to_string();
            set.push("visibility");
        }
        if let Some(location_type) = patch.location_type {
            self.location_type = location_type.key().to_string();
            set.push("location_type");
        }
        if let Some(value) = patch.location_value {
            self.location_value = match value {
                Some(text) => check_line("patch.location_value", &text, LOCATION_MAX)?,
                None => None,
            };
            set.push("location_value");
        }
        if let Some(pattern) = patch.meeting_pattern_override {
            self.meeting_pattern_override = match pattern {
                Some(text) => check_line("patch.meeting_pattern_override", &text, PATTERN_MAX)?,
                None => None,
            };
            set.push("meeting_pattern_override");
        }
        if let Some(timezone) = patch.timezone {
            self.timezone = Some(
                check_timezone("patch.timezone", &timezone)?
                    .name()
                    .to_string(),
            );
            set.push("timezone");
        }
        if let Some(days) = patch.booking_horizon_days {
            self.booking_horizon_days = days
                .map(|d| check_range("patch.booking_horizon_days", d, 0, MAX_DAYS_AHEAD))
                .transpose()?;
            set.push("booking_horizon_days");
        }
        if let Some(date) = patch.bookable_from {
            self.bookable_from = date
                .map(|d| check_date("patch.bookable_from", &d).map(|d| d.to_string()))
                .transpose()?;
            set.push("bookable_from");
        }
        if let Some(date) = patch.bookable_until {
            self.bookable_until = date
                .map(|d| check_date("patch.bookable_until", &d).map(|d| d.to_string()))
                .transpose()?;
            set.push("bookable_until");
        }
        if let Some(minutes) = patch.reminder_minutes {
            self.reminder_minutes = minutes
                .map(|m| check_range("patch.reminder_minutes", m, 1, REMINDER_MAX_MINUTES))
                .transpose()?;
            set.push("reminder_minutes");
        }
        if let Some(guests) = patch.max_additional_guests {
            self.max_additional_guests =
                check_range("patch.max_additional_guests", guests, 0, GUESTS_MAX)?;
            set.push("max_additional_guests");
        }
        if let Some(on) = patch.first_slot_only {
            self.first_slot_only = on;
            set.push("first_slot_only");
        }
        if let Some(view) = patch.default_calendar_view {
            self.default_calendar_view = view.key().to_string();
            set.push("default_calendar_view");
        }
        if let Some(day) = patch.week_start {
            self.week_start = day.map(|d| week_start_key(d).to_string());
            set.push("week_start");
        }
        if let Some(minutes) = patch.cancel_notice_min {
            self.cancel_notice_min = minutes
                .map(|m| check_range("patch.cancel_notice_min", m, 1, YEAR_MINUTES))
                .transpose()?;
            set.push("cancel_notice_min");
        }
        if let Some(minutes) = patch.reschedule_notice_min {
            self.reschedule_notice_min = minutes
                .map(|m| check_range("patch.reschedule_notice_min", m, 1, YEAR_MINUTES))
                .transpose()?;
            set.push("reschedule_notice_min");
        }
        if let Some(mode) = patch.sms_phone_mode {
            self.sms_phone_mode = mode.key().to_string();
            set.push("sms_phone_mode");
        }

        // Settings that only make sense together, checked as they will be.
        if set.contains(&"location_type") || set.contains(&"location_value") {
            check_location(
                "patch.location_value",
                &self.location_type,
                self.location_value.as_deref(),
            )?;
        }
        if set.contains(&"bookable_from") || set.contains(&"bookable_until") {
            if let (Some(from), Some(until)) = (&self.bookable_from, &self.bookable_until) {
                if from > until {
                    return Err(ToolError::invalid(
                        "patch.bookable_until",
                        "must be on or after bookable_from",
                    ));
                }
            }
        }
        Ok(set)
    }
}

/// `event_types.week_start` for a day.
fn week_start_key(day: Day) -> &'static str {
    match day {
        Day::Mon => "mon",
        Day::Tue => "tue",
        Day::Wed => "wed",
        Day::Thu => "thu",
        Day::Fri => "fri",
        Day::Sat => "sat",
        Day::Sun => "sun",
    }
}

// --- create_event_type ---

pub(crate) async fn create_event_type(
    w: &Writer<'_>,
    args: CreateEventTypeArgs,
) -> Result<EventTypeDetail, ToolError> {
    let slug = check_slug("slug", &args.slug)?;
    let title = check_title("title", &args.title)?;
    let duration = check_range("duration_min", args.duration_min, 1, DAY_MINUTES)?;
    let description = match args.description {
        Some(text) => check_description("description", &text)?,
        None => None,
    };
    let location_type = args.location_type.unwrap_or(LocationType::Link).key();
    let location_value = match args.location_value {
        Some(text) => check_line("location_value", &text, LOCATION_MAX)?,
        None => None,
    };
    check_location("location_value", location_type, location_value.as_deref())?;
    let visibility = args.visibility.unwrap_or(Visibility::Public).key();
    let timezone = check_timezone("timezone", &args.timezone)?;
    let rules = weekly_rules("weekly_hours", &args.weekly_hours)?;

    let pool = w.pool();
    let _write = w.ctx.write_lock.lock().await;
    // The account the web form's handler uses.
    let account_id: String =
        sqlx::query_scalar("SELECT id FROM accounts WHERE user_id = ? LIMIT 1")
            .bind(&w.me.id)
            .fetch_optional(pool)
            .await
            .map_err(db("find the account"))?
            .ok_or_else(|| ToolError::unavailable("the acting user has no scheduling account"))?;
    if slug_taken(pool, &account_id, &slug, None).await? {
        return Err(ToolError::invalid(
            "slug",
            "already used by another event type",
        ));
    }

    let mut fields = vec![
        ("title", title),
        ("slug", slug.clone()),
        ("description", description.unwrap_or_default()),
        ("duration_min", duration.to_string()),
        ("location_type", location_type.to_string()),
        ("location_value", location_value.unwrap_or_default()),
        ("visibility", visibility.to_string()),
        ("timezone", timezone.name().to_string()),
        ("avail_schedule", schedule_string(&rules)),
        // The new event type form's defaults (`new_event_type_form`), posted
        // as a browser posts them instead of left to the handler, whose
        // fallback for a missing reminder is none rather than 24 hours.
        ("min_notice_min", NEW_FORM_MIN_NOTICE_MIN.to_string()),
        ("buffer_before", "0".to_string()),
        ("buffer_after", "0".to_string()),
        ("reminder_minutes", NEW_FORM_REMINDER_MINUTES.to_string()),
        ("max_additional_guests", "0".to_string()),
        ("default_calendar_view", "month".to_string()),
    ];
    if args.requires_confirmation == Some(true) {
        fields.push(("requires_confirmation", "on".to_string()));
    }
    let (user, headers, form) = web_form::<EventTypeForm>(w.me, &fields).await?;
    web_form_checks(pool, w.me, None, &form).await?;
    let response =
        crate::web::event_types::create_event_type(State(w.ctx.state.clone()), user, headers, form)
            .await
            .into_response();

    let et_id = match personal_event_type(pool, w.me, &slug).await {
        Ok(et_id) => et_id,
        Err(_) if !web_saved(&response) => return Err(refused_by_form(&response)),
        Err(error) => return Err(error),
    };
    let created = EditState::load(pool, &et_id).await?;
    w.audit(
        "create_event_type",
        &format!("slug={slug}"),
        Map::new(),
        audit_map(&created.settings()),
    );
    ops_read::get_event_type(pool, w.me, &slug).await
}

// --- update_event_type ---

/// Edit an event type the way its web form does: post every stored setting,
/// with the patch applied, to the form's handler. The version is checked
/// under the write lock first, and again just before the handler runs. The
/// web interface does not take that lock, so a web save landing during the
/// handler's own reads and writes (milliseconds) can still be overwritten.
/// When that shows, because a setting outside the patch changed or the slug
/// is not the one asked for (a web rename, after which the handler finds
/// nothing and answers as if it saved), the call fails instead of reporting
/// success. A web save of the settings the patch also sets is lost unseen.
pub(crate) async fn update_event_type(
    w: &Writer<'_>,
    args: UpdateEventTypeArgs,
) -> Result<EventTypeChanged, ToolError> {
    let pool = w.pool();
    let _write = w.ctx.write_lock.lock().await;
    let et_id = personal_event_type(pool, w.me, &args.slug).await?;
    if version_of(pool, &et_id).await? != args.version.trim() {
        return Err(stale_version());
    }
    let before = EditState::load(pool, &et_id).await?;
    if before.weekly_hours.is_empty() {
        // The form would give it the owner's default hours.
        return Err(ToolError::unavailable(
            "this event type has no weekly hours, and saving it the way the web form does \
             would give it the default ones; set its hours with set_weekly_hours first",
        ));
    }
    let account_id: String = sqlx::query_scalar("SELECT account_id FROM event_types WHERE id = ?")
        .bind(&et_id)
        .fetch_one(pool)
        .await
        .map_err(db("read event type"))?;
    let mut wanted = before.clone();
    let set = wanted.apply(pool, &account_id, &et_id, args.patch).await?;

    let followed_tz = event_type_followed_tz(pool, &et_id).await;
    let send_pending =
        set.contains(&"pending_blocks_slot") || set.contains(&"pending_expiry_hours");
    let fields = wanted.form_fields(&followed_tz, send_pending, set.contains(&"week_start"));
    let (user, headers, form) = web_form::<EventTypeForm>(w.me, &fields).await?;
    web_form_checks(pool, w.me, Some(&et_id), &form).await?;
    // Most of the time since the first check went into building the form:
    // a web save in it is caught here, before anything is overwritten.
    if version_of(pool, &et_id).await? != args.version.trim() {
        return Err(stale_version());
    }
    let response = crate::web::event_types::update_event_type(
        State(w.ctx.state.clone()),
        user,
        headers,
        Path(before.slug.clone()),
        form,
    )
    .await
    .into_response();

    // The audit says what changed, whoever changed it.
    let after = EditState::load(pool, &et_id).await?;
    let (changed, old, new) = diff(&before.settings(), &after.settings());
    w.audit(
        "update_event_type",
        &format!("slug={}", before.slug),
        old,
        new,
    );
    if !web_saved(&response) {
        return Err(refused_by_form(&response));
    }
    let unasked: Vec<&String> = changed
        .iter()
        .filter(|name| !set.contains(&name.as_str()))
        .collect();
    if !unasked.is_empty() || after.slug != wanted.slug {
        tracing::warn!(
            target: "calrs::agent",
            settings = ?unasked,
            slug_as_asked = after.slug == wanted.slug,
            "an event type changed in another way while the agent saved it"
        );
        return Err(changed_meanwhile());
    }
    Ok(EventTypeChanged {
        changed,
        event_type: ops_read::get_event_type(pool, w.me, &after.slug).await?,
    })
}

// --- set_weekly_hours ---

/// Replace the weekly hours in one transaction that also checks the
/// version, so a change made anywhere since it was read is refused.
pub(crate) async fn set_weekly_hours(
    w: &Writer<'_>,
    args: WeeklyHoursArgs,
) -> Result<EventTypeChanged, ToolError> {
    let rules = weekly_rules("days", &args.days)?;
    let timezone = check_timezone("timezone", &args.timezone)?
        .name()
        .to_string();

    let pool = w.pool();
    let _write = w.ctx.write_lock.lock().await;
    let et_id = personal_event_type(pool, w.me, &args.slug).await?;
    let followed_tz = event_type_followed_tz(pool, &et_id).await;

    let mut tx = pool.begin().await.map_err(db("start a transaction"))?;
    let current = snapshot::snapshot_on(&mut tx, &et_id)
        .await
        .map_err(db("snapshot event type"))?
        .ok_or_else(|| ToolError::not_found("event type"))?;
    if snapshot::version_token(&current) != args.version.trim() {
        return Err(stale_version());
    }
    let old_rules: Vec<Rule> = sqlx::query_as(
        "SELECT day_of_week, start_time, end_time FROM availability_rules
         WHERE event_type_id = ? ORDER BY day_of_week, start_time, end_time",
    )
    .bind(&et_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(db("read weekly hours"))?;
    let stored_tz: Option<String> =
        sqlx::query_scalar("SELECT NULLIF(timezone, '') FROM event_types WHERE id = ?")
            .bind(&et_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(db("read event type"))?;
    // A timezone the event type already has, its own or followed, is left
    // as it is, so one that follows the owner keeps following.
    let effective_tz = stored_tz.clone().unwrap_or(followed_tz);
    let new_tz = (timezone != effective_tz).then_some(timezone);

    let mut changed = Vec::new();
    if old_rules != rules {
        sqlx::query("DELETE FROM availability_rules WHERE event_type_id = ?")
            .bind(&et_id)
            .execute(&mut *tx)
            .await
            .map_err(db("replace weekly hours"))?;
        for (day, start, end) in &rules {
            sqlx::query(
                "INSERT INTO availability_rules (id, event_type_id, day_of_week, start_time, end_time)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&et_id)
            .bind(day)
            .bind(start)
            .bind(end)
            .execute(&mut *tx)
            .await
            .map_err(db("replace weekly hours"))?;
        }
        changed.push("weekly_hours".to_string());
    }
    if let Some(timezone) = &new_tz {
        sqlx::query("UPDATE event_types SET timezone = ? WHERE id = ?")
            .bind(timezone)
            .bind(&et_id)
            .execute(&mut *tx)
            .await
            .map_err(db("set the event type timezone"))?;
        changed.push("timezone".to_string());
    }
    tx.commit().await.map_err(db("save weekly hours"))?;

    let slug = slug_of(pool, &et_id).await?;
    let mut old = Map::new();
    let mut new = Map::new();
    if changed.iter().any(|c| c == "weekly_hours") {
        old.insert("weekly_hours".into(), json!(schedule_string(&old_rules)));
        new.insert("weekly_hours".into(), json!(schedule_string(&rules)));
    }
    if let Some(timezone) = &new_tz {
        old.insert("timezone".into(), json!(stored_tz));
        new.insert("timezone".into(), json!(timezone));
    }
    w.audit("set_weekly_hours", &format!("slug={slug}"), old, new);
    Ok(EventTypeChanged {
        changed,
        event_type: ops_read::get_event_type(pool, w.me, &slug).await?,
    })
}

// --- Date overrides ---

fn override_out(
    id: String,
    date: String,
    start: Option<String>,
    end: Option<String>,
    blocked: i64,
) -> OverrideOut {
    OverrideOut {
        id,
        date,
        kind: if blocked != 0 {
            OverrideKind::Blocked
        } else {
            OverrideKind::CustomHours
        },
        start,
        end,
    }
}

fn override_audit(found: &OverrideOut) -> Map<String, Value> {
    let kind = match found.kind {
        OverrideKind::Blocked => "blocked",
        OverrideKind::CustomHours => "custom_hours",
    };
    let mut map = Map::new();
    map.insert("override_id".into(), json!(found.id));
    map.insert("date".into(), json!(found.date));
    map.insert("kind".into(), json!(kind));
    map.insert("start".into(), json!(found.start));
    map.insert("end".into(), json!(found.end));
    map
}

/// A date override by id, if it belongs to event type `et_id`.
async fn find_override(
    pool: &SqlitePool,
    et_id: &str,
    id: &str,
) -> Result<Option<OverrideOut>, ToolError> {
    let row: Option<(String, String, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT id, date, start_time, end_time, is_blocked FROM availability_overrides
         WHERE id = ? AND event_type_id = ?",
    )
    .bind(id)
    .bind(et_id)
    .fetch_optional(pool)
    .await
    .map_err(db("read date override"))?;
    Ok(row.map(|(id, date, start, end, blocked)| override_out(id, date, start, end, blocked)))
}

/// Add a date override, addressed by the event type's id (the web handler
/// looks the slug up across all accounts for an admin).
pub(crate) async fn add_date_override(
    w: &Writer<'_>,
    args: AddOverrideArgs,
) -> Result<OverrideAdded, ToolError> {
    let date = check_date("date", &args.date)?;
    let (start, end) = match args.kind {
        OverrideKind::Blocked => {
            for (field, value) in [("start", &args.start), ("end", &args.end)] {
                if value.is_some() {
                    return Err(ToolError::invalid(field, "not used for a blocked day"));
                }
            }
            (None, None)
        }
        OverrideKind::CustomHours => {
            let required = |field: &str| ToolError::invalid(field, "required for custom_hours");
            let start = check_time(
                "start",
                args.start.as_deref().ok_or_else(|| required("start"))?,
            )?;
            let end = check_time("end", args.end.as_deref().ok_or_else(|| required("end"))?)?;
            if start >= end {
                return Err(ToolError::invalid("end", "must be after start"));
            }
            (Some(hhmm(start)), Some(hhmm(end)))
        }
    };

    let pool = w.pool();
    let _write = w.ctx.write_lock.lock().await;
    let et_id = personal_event_type(pool, w.me, &args.slug).await?;
    let today = today_in(get_host_tz(pool, &et_id).await);
    let last = today + chrono::Duration::days(MAX_DAYS_AHEAD);
    if date < today || date > last {
        return Err(ToolError::invalid(
            "date",
            format!("must be from {today} to {last}"),
        ));
    }
    let date = date.format("%Y-%m-%d").to_string();
    let blocked = i64::from(args.kind == OverrideKind::Blocked);
    let same: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM availability_overrides
                        WHERE event_type_id = ? AND date = ? AND is_blocked = ?
                          AND start_time IS ? AND end_time IS ?)",
    )
    .bind(&et_id)
    .bind(&date)
    .bind(blocked)
    .bind(&start)
    .bind(&end)
    .fetch_one(pool)
    .await
    .map_err(db("read date overrides"))?;
    if same {
        return Err(ToolError::invalid(
            "date",
            "the same override already exists on that date",
        ));
    }
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO availability_overrides
             (id, event_type_id, date, start_time, end_time, is_blocked)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&et_id)
    .bind(&date)
    .bind(&start)
    .bind(&end)
    .bind(blocked)
    .execute(pool)
    .await
    .map_err(db("add date override"))?;

    let added = find_override(pool, &et_id, &id)
        .await?
        .ok_or_else(|| ToolError::internal("add date override", &"row missing after insert"))?;
    let slug = slug_of(pool, &et_id).await?;
    w.audit(
        "add_date_override",
        &format!("slug={slug}"),
        Map::new(),
        override_audit(&added),
    );
    Ok(OverrideAdded {
        slug,
        date_override: added,
        version: version_of(pool, &et_id).await?,
    })
}

pub(crate) async fn remove_date_override(
    w: &Writer<'_>,
    args: RemoveOverrideArgs,
) -> Result<OverrideRemoved, ToolError> {
    let pool = w.pool();
    let _write = w.ctx.write_lock.lock().await;
    let et_id = personal_event_type(pool, w.me, &args.slug).await?;
    let id = args.override_id.trim();
    let not_found = || ToolError::not_found("date override");
    let removed = find_override(pool, &et_id, id)
        .await?
        .ok_or_else(not_found)?;
    let deleted =
        sqlx::query("DELETE FROM availability_overrides WHERE id = ? AND event_type_id = ?")
            .bind(id)
            .bind(&et_id)
            .execute(pool)
            .await
            .map_err(db("remove date override"))?
            .rows_affected();
    if deleted == 0 {
        return Err(not_found());
    }
    let slug = slug_of(pool, &et_id).await?;
    w.audit(
        "remove_date_override",
        &format!("slug={slug}"),
        override_audit(&removed),
        Map::new(),
    );
    Ok(OverrideRemoved {
        slug,
        removed,
        version: version_of(pool, &et_id).await?,
    })
}

/// The slug of event type `et_id`.
pub(super) async fn slug_of(pool: &SqlitePool, et_id: &str) -> Result<String, ToolError> {
    sqlx::query_scalar("SELECT slug FROM event_types WHERE id = ?")
        .bind(et_id)
        .fetch_optional(pool)
        .await
        .map_err(db("read event type"))?
        .ok_or_else(|| ToolError::not_found("event type"))
}

// --- enable_event_type / disable_event_type ---

async fn enabled_of(pool: &SqlitePool, et_id: &str) -> Result<bool, ToolError> {
    sqlx::query_scalar::<_, i64>("SELECT enabled FROM event_types WHERE id = ?")
        .bind(et_id)
        .fetch_one(pool)
        .await
        .map(|value| value != 0)
        .map_err(db("read event type"))
}

/// Switch an event type on or off. Unlike the web's toggle, the result does
/// not depend on the state it finds, so a repeated call changes nothing.
pub(crate) async fn set_enabled(
    w: &Writer<'_>,
    tool: &'static str,
    slug: &str,
    enabled: bool,
) -> Result<EnabledOut, ToolError> {
    let pool = w.pool();
    let _write = w.ctx.write_lock.lock().await;
    let et_id = personal_event_type(pool, w.me, slug).await?;
    let was = enabled_of(pool, &et_id).await?;
    if was != enabled {
        sqlx::query("UPDATE event_types SET enabled = ? WHERE id = ?")
            .bind(i64::from(enabled))
            .bind(&et_id)
            .execute(pool)
            .await
            .map_err(db("switch event type"))?;
    }
    let now = enabled_of(pool, &et_id).await?;
    let active_bookings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bookings
         WHERE event_type_id = ? AND status IN ('confirmed', 'pending')",
    )
    .bind(&et_id)
    .fetch_one(pool)
    .await
    .map_err(db("count bookings"))?;
    let slug = slug_of(pool, &et_id).await?;
    let state = |on: bool| {
        let mut map = Map::new();
        map.insert("enabled".into(), json!(on));
        map
    };
    w.audit(tool, &format!("slug={slug}"), state(was), state(now));
    Ok(EnabledOut {
        slug,
        enabled: now,
        changed: was != now,
        active_bookings,
        version: version_of(pool, &et_id).await?,
    })
}

// --- set_profile ---

/// Name, timezone and language of a user.
type Profile = (String, String, Option<String>);

async fn profile_of(pool: &SqlitePool, user_id: &str) -> Result<Profile, ToolError> {
    sqlx::query_as("SELECT name, timezone, language FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(db("read profile"))
}

/// A display name as `settings_save` accepts it (trimmed, 1 to 255 bytes),
/// on one line.
fn check_name(name: &str) -> Result<String, ToolError> {
    let name = check_line("name", name, TITLE_MAX)?
        .ok_or_else(|| ToolError::invalid("name", "must not be empty"))?;
    if name.len() > TITLE_MAX {
        return Err(ToolError::invalid(
            "name",
            format!("at most {TITLE_MAX} bytes"),
        ));
    }
    Ok(name)
}

/// A language code calrs ships, or `auto` (stored as NULL: follow the
/// browser), as `settings_save` stores them.
fn check_language(code: &str) -> Result<Option<String>, ToolError> {
    let code = code.trim();
    if code == "auto" {
        return Ok(None);
    }
    crate::i18n::supported_code(code)
        .map(|code| Some(code.to_string()))
        .ok_or_else(|| {
            let codes: Vec<&str> = crate::i18n::supported_with_labels()
                .map(|(code, _)| code)
                .collect();
            ToolError::invalid(
                "language",
                format!("use one of {} or auto", codes.join(", ")),
            )
        })
}

/// Name, timezone and language of the acting user: the narrow part of
/// `settings_save` (dashboard.rs), which also renames the linked account.
/// It does not touch `accounts.timezone`, and neither does this.
pub(crate) async fn set_profile(
    w: &Writer<'_>,
    args: ProfileArgs,
) -> Result<ProfileOut, ToolError> {
    if args.name.is_none() && args.timezone.is_none() && args.language.is_none() {
        return Err(ToolError::invalid(
            "arguments",
            "give at least one of name, timezone, language",
        ));
    }
    let name = args.name.as_deref().map(check_name).transpose()?;
    let timezone = args
        .timezone
        .as_deref()
        .map(|tz| check_timezone("timezone", tz).map(|tz| tz.name().to_string()))
        .transpose()?;
    let language = args.language.as_deref().map(check_language).transpose()?;

    let pool = w.pool();
    let _write = w.ctx.write_lock.lock().await;
    let before = profile_of(pool, &w.me.id).await?;
    let mut tx = pool.begin().await.map_err(db("start a transaction"))?;
    sqlx::query(
        "UPDATE users SET name = COALESCE(?, name), timezone = COALESCE(?, timezone),
                language = CASE WHEN ? THEN ? ELSE language END,
                updated_at = datetime('now')
         WHERE id = ?",
    )
    .bind(&name)
    .bind(&timezone)
    .bind(language.is_some())
    .bind(language.clone().flatten())
    .bind(&w.me.id)
    .execute(&mut *tx)
    .await
    .map_err(db("save profile"))?;
    if let Some(name) = &name {
        sqlx::query("UPDATE accounts SET name = ? WHERE user_id = ?")
            .bind(name)
            .bind(&w.me.id)
            .execute(&mut *tx)
            .await
            .map_err(db("rename the account"))?;
    }
    tx.commit().await.map_err(db("save profile"))?;
    let after = profile_of(pool, &w.me.id).await?;

    let settings = |(name, timezone, language): &Profile| {
        vec![
            Setting {
                name: "name",
                value: json!(name),
                text: true,
            },
            Setting {
                name: "timezone",
                value: json!(timezone),
                text: false,
            },
            Setting {
                name: "language",
                value: json!(language),
                text: false,
            },
        ]
    };
    let (changed, old, new) = diff(&settings(&before), &settings(&after));
    w.audit("set_profile", "profile", old, new);

    let event_types_following_timezone: Vec<String> = sqlx::query_scalar(
        "SELECT et.slug FROM event_types et JOIN accounts a ON a.id = et.account_id
         WHERE a.user_id = ? AND et.team_id IS NULL AND NULLIF(et.timezone, '') IS NULL
         ORDER BY et.created_at, et.id LIMIT ?",
    )
    .bind(&w.me.id)
    .bind(FOLLOWING_MAX)
    .fetch_all(pool)
    .await
    .map_err(db("list event types"))?;
    let team_event_types_following_timezone: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_types et JOIN accounts a ON a.id = et.account_id
         WHERE a.user_id = ? AND et.team_id IS NOT NULL AND NULLIF(et.timezone, '') IS NULL",
    )
    .bind(&w.me.id)
    .fetch_one(pool)
    .await
    .map_err(db("count team event types"))?;
    let (name, timezone, language) = after;
    Ok(ProfileOut {
        name,
        timezone,
        language,
        changed,
        event_types_following_timezone,
        team_event_types_following_timezone,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(day: Day, windows: &[(&str, &str)]) -> DayHoursIn {
        DayHoursIn {
            day,
            windows: windows
                .iter()
                .map(|(start, end)| WindowIn {
                    start: start.to_string(),
                    end: end.to_string(),
                })
                .collect(),
        }
    }

    fn rule(day: i32, start: &str, end: &str) -> Rule {
        (day, start.to_string(), end.to_string())
    }

    #[test]
    fn weekly_hours_are_checked_and_sorted() {
        let rules = weekly_rules(
            "days",
            &[
                day(Day::Sun, &[("10:00", "12:00")]),
                day(Day::Mon, &[("13:00", "17:30"), ("09:00", "12:00")]),
                day(Day::Tue, &[]),
            ],
        )
        .unwrap();
        assert_eq!(
            rules,
            vec![
                rule(0, "10:00", "12:00"),
                rule(1, "09:00", "12:00"),
                rule(1, "13:00", "17:30"),
            ]
        );
        assert_eq!(
            schedule_string(&rules),
            "0:10:00-12:00;1:09:00-12:00,13:00-17:30"
        );
        // Touching windows do not overlap.
        assert!(weekly_rules(
            "days",
            &[day(Day::Fri, &[("09:00", "12:00"), ("12:00", "13:00")])]
        )
        .is_ok());
    }

    #[test]
    fn weekly_hours_errors_name_the_field() {
        let error = |days: &[DayHoursIn]| weekly_rules("days", days).unwrap_err().to_string();
        assert!(error(&[]).starts_with("invalid days: no hours"));
        assert!(error(&[day(Day::Mon, &[])]).starts_with("invalid days: no hours"));
        assert!(error(&[day(Day::Mon, &[("9:00", "17:00")])])
            .starts_with("invalid days[0].windows[0].start: use HH:MM"));
        assert!(error(&[day(Day::Mon, &[("09:00", "24:00")])])
            .starts_with("invalid days[0].windows[0].end"));
        assert!(error(&[day(Day::Mon, &[("12:00", "09:00")])])
            .starts_with("invalid days[0].windows[0]: start must be before end"));
        assert!(
            error(&[day(Day::Mon, &[("09:00", "12:00"), ("11:00", "13:00")])])
                .starts_with("invalid days[0].windows: windows must not overlap")
        );
        assert!(error(&[
            day(Day::Mon, &[("09:00", "10:00")]),
            day(Day::Mon, &[("11:00", "12:00")])
        ])
        .starts_with("invalid days[1].day"));
        let many: Vec<(String, String)> = (0..11)
            .map(|h| (format!("{h:02}:00"), format!("{h:02}:30")))
            .collect();
        let many: Vec<(&str, &str)> = many.iter().map(|(s, e)| (s.as_str(), e.as_str())).collect();
        assert!(error(&[day(Day::Mon, &many)]).starts_with("invalid days[0].windows: at most 10"));
    }

    #[test]
    fn times_and_dates_are_strict() {
        assert_eq!(hhmm(check_time("t", " 07:05 ").unwrap()), "07:05");
        for bad in [
            "7:05", "07:5", "07-05", "24:00", "23:60", "0705", "07:05:00", "ab:cd",
        ] {
            assert!(check_time("t", bad).is_err(), "{bad}");
        }
        assert_eq!(
            check_date("d", "2030-02-28").unwrap().to_string(),
            "2030-02-28"
        );
        for bad in ["2030-2-28", "2030-02-30", "28.02.2030", "2030-02-28T00:00"] {
            assert!(check_date("d", bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn links_must_be_http_https_or_mailto() {
        for good in [
            "See [docs](https://example.com/docs) or [mail](mailto:a@example.com).",
            "Write to <owner@example.com> or <https://example.com>.",
            "Plain text: 10:00 sharp, note: bring a laptop.",
        ] {
            assert!(check_markdown_links("description", good).is_ok(), "{good}");
        }
        for bad in [
            "[click](javascript:alert(1))",
            "[click]( JavaScript:alert(1) )",
            "<javascript:alert(1)>",
            "![x](data:image/png;base64,AAAA)",
            "[relative](/u/someone)",
            "[ref][1]\n\n[1]: vbscript:x",
        ] {
            let error = check_markdown_links("description", bad).unwrap_err();
            assert!(
                error.to_string().starts_with("invalid description"),
                "{bad}"
            );
        }
    }

    #[test]
    fn locations_fit_their_type() {
        assert!(check_location("v", "link", Some("https://meet.example.com/x")).is_ok());
        assert!(check_location("v", "link", Some("mailto:a@example.com")).is_ok());
        assert!(check_location("v", "link", Some("javascript:alert(1)")).is_err());
        assert!(check_location("v", "link", Some("https://a.example.com/ x")).is_err());
        assert!(check_location("v", "link", None).is_err());
        assert!(check_location("v", "phone", Some("+33 1 23 45 67 89")).is_ok());
        assert!(check_location("v", "in_person", Some("  ")).is_err());
        assert!(check_location("v", "jitsi_auto", None).is_ok());
    }

    #[test]
    fn owner_text_stays_visible_and_bounded() {
        assert_eq!(
            check_line("title", "  Intro call ", TITLE_MAX).unwrap(),
            Some("Intro call".to_string())
        );
        assert_eq!(check_line("title", "   ", TITLE_MAX).unwrap(), None);
        assert!(check_line("title", "a\nb", TITLE_MAX).is_err());
        assert!(check_line("title", "Pay\u{202E}moc.evil", TITLE_MAX).is_err());
        assert!(check_line("title", &"x".repeat(256), TITLE_MAX).is_err());
        assert!(check_description("description", "Line one\nline two").is_ok());
        assert!(check_description("description", "hidden\u{200B}text").is_err());
        assert!(check_description("description", &"x".repeat(OWNER_TEXT_MAX + 1)).is_err());
        assert!(check_slug("slug", "intro-30").is_ok());
        for bad in [
            "",
            "Intro",
            "intro call",
            "---",
            "intro_30",
            &"a".repeat(101),
        ] {
            assert!(check_slug("slug", bad).is_err(), "{bad}");
        }
        assert_eq!(check_language("auto").unwrap(), None);
        assert_eq!(
            check_language("de-formal").unwrap().as_deref(),
            Some("de-formal")
        );
        assert!(check_language("xx")
            .unwrap_err()
            .to_string()
            .contains("en, "));
    }

    #[test]
    fn frequency_limits_use_the_form_format() {
        assert_eq!(
            limits_string(&[(2, "day".into(), false), (5, "week".into(), true)]),
            "2:day,5:week:m"
        );
        assert_eq!(limits_string(&[]), "");
    }

    #[test]
    fn audit_values_hide_free_text() {
        let before = [
            Setting {
                name: "title",
                value: json!("Secret plan"),
                text: true,
            },
            Setting {
                name: "duration_min",
                value: json!(30),
                text: false,
            },
            Setting {
                name: "description",
                value: Value::Null,
                text: true,
            },
        ];
        let after = [
            Setting {
                name: "title",
                value: json!("Other plan"),
                text: true,
            },
            Setting {
                name: "duration_min",
                value: json!(30),
                text: false,
            },
            Setting {
                name: "description",
                value: json!("Now set"),
                text: true,
            },
        ];
        let (changed, old, new) = diff(&before, &after);
        assert_eq!(changed, vec!["title", "description"]);
        assert_eq!(
            Value::Object(old),
            json!({"title": "(text)", "description": null})
        );
        assert_eq!(
            Value::Object(new),
            json!({"title": "(text)", "description": "(text)"})
        );
    }
}
