//! The queries behind the read tools, as plain async functions returning
//! DTOs, so they can be tested without MCP.
//!
//! Everything is scoped to the acting user's personal account (their
//! `accounts` row, event types with no team), as the dashboard is. Queries
//! select the columns a DTO needs and nothing else: no tokens, no source
//! URLs or credentials, no calendar hrefs, no event text, no guest notes,
//! phone numbers or full addresses.

use super::dto::*;
use super::error::{db, ToolError};
use super::sanitize::{mask_email, owner_text, untrusted_text};
use super::server::Session;
use super::snapshot;
use super::{
    AgentCtx, APPROVALS_PER_HOUR, BOOKING_READS_PER_HOUR, CALLS_PER_HOUR, CAL_NAME_MAX, CURSOR_MAX,
    ENDINGS_PER_HOUR, GUEST_NAME_MAX, MAX_CONNECTIONS, MAX_FRAME_BYTES, OWNER_TEXT_MAX,
    PAGE_DEFAULT, PAGE_MAX, SYNC_COOLDOWN_SECS, WRITES_PER_HOUR,
};
use crate::booking_window::{today_in, MAX_DAYS_AHEAD};
use crate::models::User;
use crate::web::availability::{
    booking_window_warnings, compute_slots, fetch_busy_times_for_user, get_booking_window,
    get_host_tz, BusySource,
};
use crate::web::AppState;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, Sqlite, SqlitePool};

/// Most free times `get_slots` returns.
const SLOTS_MAX: usize = 500;
/// Most date overrides `get_event_type` returns.
const OVERRIDES_MAX: usize = 200;

/// The personal bookings of the user bound to the `?`, with the aliases `b`,
/// `et`, `a` and `u` (the event type's owner).
pub(crate) const PERSONAL_BOOKINGS: &str = "FROM bookings b
     JOIN event_types et ON et.id = b.event_type_id
     JOIN accounts a ON a.id = et.account_id
     JOIN users u ON u.id = a.user_id
     WHERE a.user_id = ? AND et.team_id IS NULL";

/// A booking's start and end as `booking_time::local` reads them: version 0
/// rows are a wall clock in the event type's timezone even with a trailing Z.
pub(crate) const START: &str =
    "CASE b.time_version WHEN 1 THEN b.start_at ELSE rtrim(b.start_at, 'Z') END";
pub(crate) const END: &str =
    "CASE b.time_version WHEN 1 THEN b.end_at ELSE rtrim(b.end_at, 'Z') END";
/// The timezone version 0 rows are stored in (as `format.rs` reads it).
pub(crate) const STORED_TZ: &str = "COALESCE(NULLIF(et.timezone, ''), u.timezone)";

/// Offset and size of one page, from a tool's `cursor` and `limit`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PageReq {
    pub(crate) offset: u32,
    pub(crate) limit: u32,
}

impl PageReq {
    pub(crate) fn parse(cursor: Option<&str>, limit: Option<u32>) -> Result<Self, ToolError> {
        let limit = limit.unwrap_or(PAGE_DEFAULT);
        if !(1..=PAGE_MAX).contains(&limit) {
            return Err(ToolError::invalid(
                "limit",
                format!("must be 1 to {PAGE_MAX}"),
            ));
        }
        let offset = match cursor.map(str::trim).filter(|c| !c.is_empty()) {
            None => 0,
            Some(cursor) => match cursor.parse::<u32>() {
                Ok(offset) if offset <= CURSOR_MAX => offset,
                _ => {
                    return Err(ToolError::invalid(
                        "cursor",
                        "pass next_cursor from the previous page unchanged",
                    ))
                }
            },
        };
        Ok(Self { offset, limit })
    }

    /// Rows to fetch: one more than the page, to know whether another follows.
    fn fetch(&self) -> i64 {
        i64::from(self.limit) + 1
    }

    fn offset(&self) -> i64 {
        i64::from(self.offset)
    }

    fn page<T>(&self, mut items: Vec<T>) -> Page<T> {
        let limit = self.limit as usize;
        let more = items.len() > limit;
        items.truncate(limit);
        let next = self.offset + self.limit;
        Page {
            items,
            next_cursor: (more && next <= CURSOR_MAX).then(|| next.to_string()),
        }
    }
}

pub(crate) fn user_tz(me: &User) -> Tz {
    me.timezone.parse().unwrap_or(Tz::UTC)
}

/// The instant of a stored booking time. A version 1 value is an instant
/// already, so it stays right in a repeated hour. A version 0 value is a wall
/// clock in `stored_tz`, read the way `booking_time::local` reads it (the
/// earlier time when the clock repeats, as UTC when it skips).
pub(crate) fn booking_utc(value: &str, stored_tz: &str) -> Option<chrono::DateTime<Utc>> {
    if let Ok(instant) = chrono::DateTime::parse_from_rfc3339(value) {
        return Some(instant.with_timezone(&Utc));
    }
    let stored = stored_tz.parse::<Tz>().unwrap_or(Tz::UTC);
    crate::booking_time::naive(value).map(|wall| {
        stored
            .from_local_datetime(&wall)
            .earliest()
            .unwrap_or_else(|| stored.from_utc_datetime(&wall))
            .with_timezone(&Utc)
    })
}

/// A stored booking time as RFC 3339 in `target` (see `booking_utc`). Falls
/// back to the stored value if it cannot be read.
pub(crate) fn booking_instant(value: &str, stored_tz: &str, target: Tz) -> String {
    booking_utc(value, stored_tz)
        .map(|instant| instant.with_timezone(&target).fixed_offset().to_rfc3339())
        .unwrap_or_else(|| value.to_string())
}

/// Validate a slug argument without echoing it.
fn check_slug(slug: &str) -> Result<&str, ToolError> {
    let slug = slug.trim();
    if slug.is_empty() || slug.chars().count() > 100 {
        return Err(ToolError::invalid("slug", "must be 1 to 100 characters"));
    }
    Ok(slug)
}

/// The id of the acting user's personal event type `slug`.
pub(crate) async fn personal_event_type(
    pool: &SqlitePool,
    me: &User,
    slug: &str,
) -> Result<String, ToolError> {
    let slug = check_slug(slug)?;
    let id: Option<String> = sqlx::query_scalar(
        "SELECT et.id FROM event_types et
         JOIN accounts a ON a.id = et.account_id
         WHERE a.user_id = ? AND et.team_id IS NULL AND et.slug = ?
         ORDER BY et.created_at LIMIT 1",
    )
    .bind(&me.id)
    .bind(slug)
    .fetch_optional(pool)
    .await
    .map_err(db("find event type"))?;
    id.ok_or_else(|| ToolError::not_found("event type"))
}

// --- whoami ---

pub(crate) fn whoami(ctx: &AgentCtx, session: &Session, me: &User) -> WhoAmI {
    WhoAmI {
        server: "calrs",
        version: env!("CARGO_PKG_VERSION"),
        connection: ConnectionOut {
            role: session.role.to_string(),
            peer_uid: session.uid,
            session_id: session.id.clone(),
        },
        acting_user: ActingUser {
            name: me.name.clone(),
            username: me.username.clone(),
            email: me.email.clone(),
            timezone: me.timezone.clone(),
            language: me.language.clone(),
        },
        limits: LimitsOut {
            max_message_bytes: MAX_FRAME_BYTES,
            idle_timeout_secs: ctx.config.idle_timeout.as_secs(),
            max_session_secs: ctx.config.max_session.as_secs(),
            max_connections: MAX_CONNECTIONS,
            calls_per_hour: CALLS_PER_HOUR,
            writes_per_hour: WRITES_PER_HOUR,
            approvals_per_hour: APPROVALS_PER_HOUR,
            declines_and_cancellations_per_hour: ENDINGS_PER_HOUR,
            booking_reads_per_hour: BOOKING_READS_PER_HOUR,
            sync_cooldown_secs: SYNC_COOLDOWN_SECS,
        },
    }
}

// --- get_overview ---

pub(crate) async fn overview(pool: &SqlitePool, me: &User) -> Result<Overview, ToolError> {
    let (total, enabled): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(SUM(et.enabled != 0), 0)
         FROM event_types et JOIN accounts a ON a.id = et.account_id
         WHERE a.user_id = ? AND et.team_id IS NULL",
    )
    .bind(&me.id)
    .fetch_one(pool)
    .await
    .map_err(db("count event types"))?;

    let count = |condition: &str| format!("SELECT COUNT(*) {PERSONAL_BOOKINGS} AND {condition}");
    let pending: i64 = sqlx::query_scalar(&count("b.status = 'pending'"))
        .bind(&me.id)
        .fetch_one(pool)
        .await
        .map_err(db("count pending bookings"))?;
    let upcoming: i64 = sqlx::query_scalar(&count(
        "b.status = 'confirmed' AND datetime(b.start_at) >= datetime('now')",
    ))
    .bind(&me.id)
    .fetch_one(pool)
    .await
    .map_err(db("count upcoming bookings"))?;
    let writeback_failures: i64 = sqlx::query_scalar(&count(
        "b.status = 'confirmed' AND b.writeback_error IS NOT NULL \
         AND datetime(b.start_at) >= datetime('now')",
    ))
    .bind(&me.id)
    .fetch_one(pool)
    .await
    .map_err(db("count write-back failures"))?;
    let next: Option<(String, String)> = sqlx::query_as(&format!(
        "SELECT {START}, {STORED_TZ} {PERSONAL_BOOKINGS}
         AND b.status = 'confirmed' AND datetime(b.start_at) >= datetime('now')
         ORDER BY datetime(b.start_at), b.id LIMIT 1"
    ))
    .bind(&me.id)
    .fetch_optional(pool)
    .await
    .map_err(db("find the next booking"))?;

    let (sources, failing): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(SUM(cs.sync_failing_since IS NOT NULL), 0)
         FROM caldav_sources cs JOIN accounts a ON a.id = cs.account_id
         WHERE a.user_id = ?",
    )
    .bind(&me.id)
    .fetch_one(pool)
    .await
    .map_err(db("count sources"))?;

    let tz = user_tz(me);
    Ok(Overview {
        event_types: EventTypeCounts { total, enabled },
        bookings: BookingCounts {
            pending,
            upcoming_confirmed: upcoming,
            next_start: next.map(|(start, stored)| booking_instant(&start, &stored, tz)),
        },
        sources: SourceCounts {
            total: sources,
            failing,
        },
        writeback_failures,
    })
}

// --- get_settings ---

pub(crate) async fn settings(state: &AppState) -> Result<SettingsOut, ToolError> {
    let flags: Option<(i64, i64, i64, i64)> = sqlx::query_as(
        "SELECT COALESCE(registration_enabled, 0), COALESCE(mfa_required, 0),
                COALESCE(oidc_enabled, 0), COALESCE(german_formal, 0)
         FROM auth_config WHERE id = 'singleton'",
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(db("read auth settings"))?;
    let (registration, mfa, oidc, german_formal) = flags.unwrap_or_default();
    // As the admin page reads them: configured means an environment block or
    // a stored row; enabled means it is switched on and would be used. An
    // invalid environment block is configured but not enabled.
    let smtp = crate::email::load_smtp_status(&state.pool).await;
    let sms = crate::sms::load_status(&state.pool).await;
    let smtp_configured = crate::email::smtp_env_active() || matches!(smtp, Ok(Some(_)));
    let smtp_enabled = matches!(smtp, Ok(Some(ref status)) if status.enabled);
    let sms_configured = crate::sms::sms_env_active() || matches!(sms, Ok(Some(_)));
    let sms_enabled = matches!(sms, Ok(Some(ref status)) if status.enabled);
    Ok(SettingsOut {
        calrs_version: env!("CARGO_PKG_VERSION"),
        base_url: crate::settings::base_url(),
        registration_enabled: registration != 0,
        mfa_required: mfa != 0,
        oidc_enabled: oidc != 0,
        german_formal: german_formal != 0,
        default_week_start: crate::web::week_start::site(&state.pool).await.key(),
        smtp_configured,
        smtp_enabled,
        sms_configured,
        sms_enabled,
        captcha_enabled: state.captcha_config.read().await.is_some(),
    })
}

// --- list_event_types ---

pub(crate) async fn list_event_types(
    pool: &SqlitePool,
    me: &User,
    page: PageReq,
) -> Result<Page<EventTypeSummary>, ToolError> {
    let rows: Vec<(String, String, i64, i64, i64, String, String, i64)> = sqlx::query_as(
        "SELECT et.slug, et.title, et.enabled, et.duration_min, et.requires_confirmation,
                et.visibility, COALESCE(NULLIF(et.timezone, ''), u.timezone),
                (SELECT COUNT(*) FROM bookings b
                 WHERE b.event_type_id = et.id AND b.status IN ('confirmed', 'pending'))
         FROM event_types et
         JOIN accounts a ON a.id = et.account_id
         JOIN users u ON u.id = a.user_id
         WHERE a.user_id = ? AND et.team_id IS NULL
         ORDER BY et.created_at, et.id
         LIMIT ? OFFSET ?",
    )
    .bind(&me.id)
    .bind(page.fetch())
    .bind(page.offset())
    .fetch_all(pool)
    .await
    .map_err(db("list event types"))?;
    Ok(page.page(
        rows.into_iter()
            .map(
                |(slug, title, enabled, duration, confirm, visibility, timezone, active)| {
                    EventTypeSummary {
                        slug,
                        title: owner_text(&title, OWNER_TEXT_MAX).0,
                        enabled: enabled != 0,
                        duration_min: duration,
                        requires_confirmation: confirm != 0,
                        visibility,
                        timezone,
                        active_bookings: active,
                    }
                },
            )
            .collect(),
    ))
}

// --- get_event_type ---

pub(crate) fn col<'r, T>(row: &'r SqliteRow, name: &str) -> Result<T, ToolError>
where
    T: sqlx::Decode<'r, Sqlite> + sqlx::Type<Sqlite>,
{
    row.try_get(name)
        .map_err(|e| ToolError::internal("read event type", &e))
}

/// Day names in output order (Monday first) with their `day_of_week` (0 is
/// Sunday).
const WEEK: [(i64, &str); 7] = [
    (1, "mon"),
    (2, "tue"),
    (3, "wed"),
    (4, "thu"),
    (5, "fri"),
    (6, "sat"),
    (0, "sun"),
];

pub(crate) async fn get_event_type(
    pool: &SqlitePool,
    me: &User,
    slug: &str,
) -> Result<EventTypeDetail, ToolError> {
    let et_id = personal_event_type(pool, me, slug).await?;
    let version = snapshot::snapshot(pool, &et_id)
        .await
        .map_err(db("snapshot event type"))?
        .map(|snapshot| snapshot::version_token(&snapshot))
        .ok_or_else(|| ToolError::not_found("event type"))?;
    let row = sqlx::query(
        "SELECT slug, title, description, enabled, duration_min, slot_interval_min,
                buffer_before, buffer_after, min_notice_min, requires_confirmation,
                pending_blocks_slot, pending_expiry_hours, visibility, location_type,
                location_value, meeting_pattern_override, timezone, booking_horizon_days,
                bookable_from, bookable_until, reminder_minutes, max_additional_guests,
                first_slot_only, default_calendar_view, week_start, cancel_notice_min,
                reschedule_notice_min, sms_phone_mode
         FROM event_types WHERE id = ?",
    )
    .bind(&et_id)
    .fetch_one(pool)
    .await
    .map_err(db("read event type"))?;

    let rules: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT day_of_week, start_time, end_time FROM availability_rules
         WHERE event_type_id = ? ORDER BY day_of_week, start_time, end_time",
    )
    .bind(&et_id)
    .fetch_all(pool)
    .await
    .map_err(db("read weekly hours"))?;
    let weekly_hours = WEEK
        .iter()
        .map(|(number, day)| WeekdayHours {
            day,
            windows: rules
                .iter()
                .filter(|(d, _, _)| d == number)
                .map(|(_, start, end)| TimeWindow {
                    start: start.clone(),
                    end: end.clone(),
                })
                .collect(),
        })
        .collect();

    let mut overrides: Vec<(String, String, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT id, date, start_time, end_time, is_blocked FROM availability_overrides
             WHERE event_type_id = ? AND date >= date('now', '-1 day')
             ORDER BY date, start_time, id LIMIT ?",
    )
    .bind(&et_id)
    .bind(OVERRIDES_MAX as i64 + 1)
    .fetch_all(pool)
    .await
    .map_err(db("read date overrides"))?;
    let overrides_truncated = overrides.len() > OVERRIDES_MAX;
    overrides.truncate(OVERRIDES_MAX);

    let busy_calendars = busy_calendars(pool, me, &et_id).await?;
    let frequency_limits: Vec<(i64, String, i64)> = sqlx::query_as(
        "SELECT max_bookings, period, per_member FROM booking_frequency_limits
         WHERE event_type_id = ? ORDER BY period, max_bookings",
    )
    .bind(&et_id)
    .fetch_all(pool)
    .await
    .map_err(db("read frequency limits"))?;

    let window = get_booking_window(pool, &et_id).await;
    let warnings = booking_window_warnings(pool, &et_id, &window, "en")
        .await
        .iter()
        .filter_map(|warning| {
            let text = warning.get_attr("text").ok()?;
            text.as_str().map(str::to_string)
        })
        .collect();

    let description: Option<String> = col(&row, "description")?;
    let (description, description_truncated) = match description {
        Some(text) => {
            let (text, cut) = owner_text(&text, OWNER_TEXT_MAX);
            (Some(text), cut)
        }
        None => (None, false),
    };
    let title: String = col(&row, "title")?;
    let flag = |name: &str| col::<Option<i64>>(&row, name).map(|v| v.unwrap_or(0) != 0);
    let timezone_stored: Option<String> = col(&row, "timezone")?;
    Ok(EventTypeDetail {
        slug: col(&row, "slug")?,
        version,
        title: owner_text(&title, OWNER_TEXT_MAX).0,
        description,
        description_truncated,
        enabled: flag("enabled")?,
        duration_min: col(&row, "duration_min")?,
        slot_interval_min: col(&row, "slot_interval_min")?,
        buffer_before: col::<Option<i64>>(&row, "buffer_before")?.unwrap_or(0),
        buffer_after: col::<Option<i64>>(&row, "buffer_after")?.unwrap_or(0),
        min_notice_min: col::<Option<i64>>(&row, "min_notice_min")?.unwrap_or(0),
        requires_confirmation: flag("requires_confirmation")?,
        pending_blocks_slot: flag("pending_blocks_slot")?,
        pending_expiry_hours: col(&row, "pending_expiry_hours")?,
        visibility: col(&row, "visibility")?,
        location_type: col(&row, "location_type")?,
        location_value: col(&row, "location_value")?,
        meeting_pattern_override: col(&row, "meeting_pattern_override")?,
        timezone_stored: timezone_stored.filter(|tz| !tz.is_empty()),
        timezone_effective: get_host_tz(pool, &et_id).await.name().to_string(),
        booking_horizon_days: col(&row, "booking_horizon_days")?,
        bookable_from: col(&row, "bookable_from")?,
        bookable_until: col(&row, "bookable_until")?,
        reminder_minutes: col(&row, "reminder_minutes")?,
        max_additional_guests: col::<Option<i64>>(&row, "max_additional_guests")?.unwrap_or(0),
        first_slot_only: flag("first_slot_only")?,
        default_calendar_view: col(&row, "default_calendar_view")?,
        week_start: col(&row, "week_start")?,
        cancel_notice_min: col(&row, "cancel_notice_min")?,
        reschedule_notice_min: col(&row, "reschedule_notice_min")?,
        sms_phone_mode: col(&row, "sms_phone_mode")?,
        weekly_hours,
        overrides: overrides
            .into_iter()
            .map(|(id, date, start, end, blocked)| OverrideOut {
                id,
                date,
                kind: if blocked != 0 {
                    OverrideKind::Blocked
                } else {
                    OverrideKind::CustomHours
                },
                start,
                end,
            })
            .collect(),
        overrides_truncated,
        busy_calendars,
        frequency_limits: frequency_limits
            .into_iter()
            .map(|(max_bookings, period, per_member)| FrequencyLimitOut {
                max_bookings,
                period,
                per_member: per_member != 0,
            })
            .collect(),
        warnings,
    })
}

/// The calendars whose events block the slots of event type `et_id`, picked
/// the way `fetch_busy_times_for_user_opts` (availability.rs) picks them: the
/// owner's calendars that are synced and marked busy, all of them unless a
/// synced calendar was chosen for the event type, then only the chosen ones.
async fn busy_calendars(
    pool: &SqlitePool,
    me: &User,
    et_id: &str,
) -> Result<BusyCalendarsOut, ToolError> {
    let selected: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM event_type_calendars etc
                        JOIN calendars sc ON sc.id = etc.calendar_id
                        WHERE etc.event_type_id = ? AND sc.sync_enabled = 1)",
    )
    .bind(et_id)
    .fetch_one(pool)
    .await
    .map_err(db("read busy calendars"))?;
    let calendar_ids: Vec<String> = sqlx::query_scalar(
        "SELECT c.id FROM calendars c
         JOIN caldav_sources cs ON cs.id = c.source_id
         JOIN accounts a ON a.id = cs.account_id
         WHERE a.user_id = ? AND c.is_busy = 1 AND c.sync_enabled = 1
           AND (NOT EXISTS (SELECT 1 FROM event_type_calendars etc
                            JOIN calendars sc ON sc.id = etc.calendar_id
                            WHERE etc.event_type_id = ? AND sc.sync_enabled = 1)
                OR c.id IN (SELECT calendar_id FROM event_type_calendars
                            WHERE event_type_id = ?))
         ORDER BY c.id",
    )
    .bind(&me.id)
    .bind(et_id)
    .bind(et_id)
    .fetch_all(pool)
    .await
    .map_err(db("read busy calendars"))?;
    let chosen: Vec<String> = sqlx::query_scalar(
        "SELECT calendar_id FROM event_type_calendars WHERE event_type_id = ? ORDER BY calendar_id",
    )
    .bind(et_id)
    .fetch_all(pool)
    .await
    .map_err(db("read busy calendars"))?;
    let ignored_calendar_ids = chosen
        .into_iter()
        .filter(|id| !calendar_ids.contains(id))
        .collect();
    Ok(BusyCalendarsOut {
        selection: if selected {
            BusySelection::Selected
        } else {
            BusySelection::AllSynced
        },
        calendar_ids,
        ignored_calendar_ids,
    })
}

// --- get_slots ---

/// Free start times, computed the way the personal booking page does
/// (`public_pages.rs`, the `/u/{username}/{slug}` handler): sync stale
/// sources, then `compute_slots` over the host's busy times. Disabled event
/// types are computed too, as a preview.
pub(crate) async fn get_slots(
    state: &AppState,
    me: &User,
    args: SlotsArgs,
) -> Result<SlotsOut, ToolError> {
    let pool = &state.pool;
    let et_id = personal_event_type(pool, me, &args.slug).await?;
    let days = args.days.unwrap_or(7);
    if !(1..=31).contains(&days) {
        return Err(ToolError::invalid("days", "must be 1 to 31"));
    }
    let requested_tz = match args.timezone.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(name) => Some(name.parse::<Tz>().map_err(|_| {
            ToolError::invalid(
                "timezone",
                "not an IANA timezone name such as Europe/Berlin",
            )
        })?),
    };
    let requested_from = match args.from.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(date) => Some(
            NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .map_err(|_| ToolError::invalid("from", "use YYYY-MM-DD"))?,
        ),
    };

    let (duration, buffer_before, buffer_after, min_notice): (i32, i32, i32, i32) = sqlx::query_as(
        "SELECT duration_min, COALESCE(buffer_before, 0), COALESCE(buffer_after, 0),
                    COALESCE(min_notice_min, 0)
             FROM event_types WHERE id = ?",
    )
    .bind(&et_id)
    .fetch_one(pool)
    .await
    .map_err(db("read event type"))?;

    crate::commands::sync::sync_if_stale(pool, &state.secret_key, &me.id).await;

    let host_tz = get_host_tz(pool, &et_id).await;
    let window = get_booking_window(pool, &et_id).await;
    let today = today_in(host_tz);
    let from = requested_from.unwrap_or(today);
    let ahead = (from - today).num_days();
    if !(0..=MAX_DAYS_AHEAD).contains(&ahead) {
        let last = today + chrono::Duration::days(MAX_DAYS_AHEAD);
        return Err(ToolError::invalid(
            "from",
            format!("must be from {today} to {last}"),
        ));
    }
    let guest_tz = requested_tz.unwrap_or(host_tz);
    let start_offset = i32::try_from(ahead).unwrap_or(0);
    let days_ahead = i32::try_from(days).unwrap_or(7);

    let now_host = Utc::now().with_timezone(&host_tz).naive_local();
    let end_date = now_host.date() + chrono::Duration::days(i64::from(start_offset + days_ahead));
    let window_end = end_date.and_hms_opt(23, 59, 59).unwrap_or(now_host);
    let busy = BusySource::Individual(
        fetch_busy_times_for_user(pool, &me.id, now_host, window_end, host_tz, Some(&et_id)).await,
    );
    let slot_days = compute_slots(
        pool,
        &et_id,
        duration,
        buffer_before,
        buffer_after,
        min_notice,
        start_offset,
        days_ahead,
        window,
        host_tz,
        guest_tz,
        busy,
        None,
    )
    .await;

    let mut left = SLOTS_MAX;
    let mut truncated = false;
    let mut out = Vec::new();
    for day in slot_days {
        if left == 0 {
            truncated = true;
            break;
        }
        if day.slots.len() > left {
            truncated = true;
        }
        let times: Vec<String> = day.slots.into_iter().take(left).map(|s| s.start).collect();
        left -= times.len();
        out.push(SlotDayOut {
            date: day.date,
            times,
        });
    }
    Ok(SlotsOut {
        slug: check_slug(&args.slug)?.to_string(),
        timezone: guest_tz.name().to_string(),
        from: from.format("%Y-%m-%d").to_string(),
        days: out,
        truncated,
    })
}

// --- list_sources ---

pub(crate) async fn list_sources(
    pool: &SqlitePool,
    me: &User,
    page: PageReq,
) -> Result<Page<SourceOut>, ToolError> {
    let items = sources(pool, me, None, page.fetch(), page.offset()).await?;
    Ok(page.page(items))
}

/// One of the acting user's sources, as list_sources shows it.
pub(crate) async fn source(
    pool: &SqlitePool,
    me: &User,
    source_id: &str,
) -> Result<SourceOut, ToolError> {
    sources(pool, me, Some(source_id), 1, 0)
        .await?
        .pop()
        .ok_or_else(|| ToolError::not_found("calendar source"))
}

/// The acting user's sources (only `only` when given), `limit` from
/// `offset`.
async fn sources(
    pool: &SqlitePool,
    me: &User,
    only: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<SourceOut>, ToolError> {
    // The write calendar is matched to a calendar id here, in SQL, so the
    // href itself never leaves the database.
    let rows: Vec<(
        String,
        String,
        String,
        i64,
        Option<String>,
        Option<String>,
        i64,
        i64,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT cs.id, cs.name, cs.provider_type, cs.enabled, cs.last_synced,
                cs.sync_failing_since, cs.sync_failure_count,
                NULLIF(cs.write_calendar_href, '') IS NOT NULL,
                (SELECT c.id FROM calendars c
                 WHERE c.source_id = cs.id AND c.href = cs.write_calendar_href
                 ORDER BY c.id LIMIT 1)
         FROM caldav_sources cs
         JOIN accounts a ON a.id = cs.account_id
         WHERE a.user_id = ? AND (? IS NULL OR cs.id = ?)
         ORDER BY cs.created_at, cs.id
         LIMIT ? OFFSET ?",
    )
    .bind(&me.id)
    .bind(only)
    .bind(only)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(db("list sources"))?;

    let mut items = Vec::with_capacity(rows.len());
    for (id, name, provider, enabled, last_synced, failing_since, failures, writes, write_id) in
        rows
    {
        let calendars: Vec<(String, i64, Option<String>, i64)> = sqlx::query_as(
            "SELECT c.id, c.sync_enabled, c.display_name,
                    COALESCE(cs.write_calendar_href IS NOT NULL
                             AND c.href = cs.write_calendar_href, 0)
             FROM calendars c JOIN caldav_sources cs ON cs.id = c.source_id
             WHERE c.source_id = ?
             ORDER BY c.display_name, c.id",
        )
        .bind(&id)
        .fetch_all(pool)
        .await
        .map_err(db("list calendars"))?;
        let state = if failing_since.is_some() {
            SyncState::Failing
        } else if last_synced.is_none() {
            SyncState::NeverSynced
        } else {
            SyncState::Ok
        };
        let write_calendar = match (writes != 0, write_id) {
            (false, _) => WriteCalendarOut {
                state: WriteCalendarState::None,
                calendar_id: None,
            },
            (true, Some(calendar_id)) => WriteCalendarOut {
                state: WriteCalendarState::Calendar,
                calendar_id: Some(calendar_id),
            },
            (true, None) => WriteCalendarOut {
                state: WriteCalendarState::Unlisted,
                calendar_id: None,
            },
        };
        items.push(SourceOut {
            id,
            name: untrusted_text(&name, CAL_NAME_MAX),
            provider,
            enabled: enabled != 0,
            last_synced,
            sync: SyncOut {
                state,
                failing_since,
                failure_count: failures,
            },
            write_calendar,
            calendars: calendars
                .into_iter()
                .map(|(id, sync_enabled, display_name, is_write)| CalendarOut {
                    id,
                    sync_enabled: sync_enabled != 0,
                    is_write_target: is_write != 0,
                    calendar_supplied: CalendarSupplied {
                        display_name: display_name
                            .map(|name| untrusted_text(&name, CAL_NAME_MAX))
                            .filter(|name| !name.is_empty()),
                    },
                })
                .collect(),
        });
    }
    Ok(items)
}

// --- list_bookings ---

/// The columns of a `BookingSummary`, in the order of `SummaryRow`, over
/// `PERSONAL_BOOKINGS`.
pub(crate) fn summary_columns() -> String {
    format!(
        "b.id, et.slug, et.title, b.status, {START}, {END}, {STORED_TZ},
         b.guest_name, b.guest_email, b.guest_timezone,
         (SELECT COUNT(*) FROM booking_attendees ba WHERE ba.booking_id = b.id),
         COALESCE(trim(b.notes), '') != '',
         b.reschedule_by_host,
         b.writeback_error IS NOT NULL"
    )
}

/// One row of `summary_columns()`.
pub(crate) type SummaryRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    i64,
    i64,
    i64,
);

/// A booking as list_bookings shows it, times in `tz`.
pub(crate) fn summary_from(row: SummaryRow, tz: Tz) -> BookingSummary {
    let (
        id,
        slug,
        title,
        status,
        start,
        end,
        stored_tz,
        guest_name,
        guest_email,
        guest_tz,
        attendees,
        has_notes,
        reschedule,
        writeback_failed,
    ) = row;
    BookingSummary {
        id,
        event_type: EventTypeRef {
            slug,
            title: owner_text(&title, OWNER_TEXT_MAX).0,
        },
        status,
        start: booking_instant(&start, &stored_tz, tz),
        end: booking_instant(&end, &stored_tz, tz),
        additional_guests: attendees,
        has_notes: has_notes != 0,
        awaiting_reschedule: reschedule != 0,
        writeback_failed: writeback_failed != 0,
        guest_supplied: GuestSupplied {
            name: untrusted_text(&guest_name, GUEST_NAME_MAX),
            email_masked: mask_email(&guest_email),
            timezone: untrusted_text(&guest_tz, 64),
        },
    }
}

pub(crate) async fn list_bookings(
    pool: &SqlitePool,
    me: &User,
    filter: BookingFilter,
    page: PageReq,
) -> Result<Page<BookingSummary>, ToolError> {
    let (condition, order) = match filter {
        BookingFilter::Pending => ("b.status = 'pending'", "datetime(b.start_at), b.id"),
        BookingFilter::Upcoming => (
            "b.status = 'confirmed' AND datetime(b.start_at) >= datetime('now')",
            "datetime(b.start_at), b.id",
        ),
        BookingFilter::Past => (
            "datetime(b.start_at) < datetime('now') \
             AND datetime(b.start_at) >= datetime('now', '-90 days')",
            "datetime(b.start_at) DESC, b.id DESC",
        ),
    };
    let rows: Vec<SummaryRow> = sqlx::query_as(&format!(
        "SELECT {}
         {PERSONAL_BOOKINGS} AND {condition}
         ORDER BY {order}
         LIMIT ? OFFSET ?",
        summary_columns()
    ))
    .bind(&me.id)
    .bind(page.fetch())
    .bind(page.offset())
    .fetch_all(pool)
    .await
    .map_err(db("list bookings"))?;

    let tz = user_tz(me);
    Ok(page.page(rows.into_iter().map(|row| summary_from(row, tz)).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_requests_are_bounded() {
        assert_eq!(
            PageReq::parse(None, None).unwrap(),
            PageReq {
                offset: 0,
                limit: PAGE_DEFAULT
            }
        );
        assert_eq!(
            PageReq::parse(Some(" 40 "), Some(50)).unwrap(),
            PageReq {
                offset: 40,
                limit: 50
            }
        );
        assert_eq!(PageReq::parse(Some(""), Some(1)).unwrap().offset, 0);
        for (cursor, limit) in [
            (None, Some(0)),
            (None, Some(51)),
            (Some("-1"), None),
            (Some("abc"), None),
            (Some("10001"), None),
        ] {
            assert!(
                PageReq::parse(cursor, limit).is_err(),
                "{cursor:?} {limit:?}"
            );
        }
    }

    #[test]
    fn pages_know_whether_more_follow() {
        let req = PageReq {
            offset: 20,
            limit: 2,
        };
        let page = req.page(vec![1, 2, 3]);
        assert_eq!(page.items, vec![1, 2]);
        assert_eq!(page.next_cursor.as_deref(), Some("22"));
        let last = req.page(vec![1, 2]);
        assert_eq!(last.next_cursor, None);
        let at_cap = PageReq {
            offset: CURSOR_MAX,
            limit: 2,
        };
        assert_eq!(at_cap.page(vec![1, 2, 3]).next_cursor, None);
    }

    #[test]
    fn booking_times_get_an_offset() {
        let dubai: Tz = "Asia/Dubai".parse().unwrap();
        assert_eq!(
            booking_instant("2030-01-07T06:00:00Z", "UTC", dubai),
            "2030-01-07T10:00:00+04:00"
        );
        // Version 0: a wall clock in the stored timezone.
        assert_eq!(
            booking_instant("2030-01-07T10:00:00", "Asia/Dubai", dubai),
            "2030-01-07T10:00:00+04:00"
        );
        assert_eq!(
            booking_instant("2030-01-07T10:00:00", "Asia/Dubai", Tz::UTC),
            "2030-01-07T06:00:00+00:00"
        );
        assert_eq!(booking_instant("garbage", "UTC", Tz::UTC), "garbage");
    }

    #[test]
    fn booking_times_stay_right_when_the_clock_goes_back() {
        // 2030-10-27, Europe/Berlin: 03:00 CEST becomes 02:00 CET at 01:00Z,
        // so 02:30 local happens twice.
        let berlin: Tz = "Europe/Berlin".parse().unwrap();
        assert_eq!(
            booking_instant("2030-10-27T00:30:00Z", "UTC", berlin),
            "2030-10-27T02:30:00+02:00"
        );
        assert_eq!(
            booking_instant("2030-10-27T01:30:00Z", "UTC", berlin),
            "2030-10-27T02:30:00+01:00"
        );
        assert_eq!(
            booking_instant("2030-10-27T02:00:00Z", "UTC", berlin),
            "2030-10-27T03:00:00+01:00"
        );
        // A legacy wall clock in the repeated hour reads as the earlier one.
        assert_eq!(
            booking_instant("2030-10-27T02:30:00", "Europe/Berlin", Tz::UTC),
            "2030-10-27T00:30:00+00:00"
        );
    }
}
