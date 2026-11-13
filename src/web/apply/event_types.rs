//! ilia fork: `config apply`, the `event_types` section: the admin's
//! personal event types, keyed by slug (docs/apply-schema.md).
//!
//! An item is the MCP `update_event_type` edit, reused as it is: the stored
//! form state (`ops_write::EditState`, or the new event type form's defaults
//! for a slug that does not exist yet) with the item applied by
//! `EditState::apply`, which holds the MCP ranges and checks, then posted to
//! the web form's own handler (`create_event_type`, `update_event_type`) as
//! the whole form a browser would post. The form is posted only when a
//! setting it carries differs from the stored one.
//!
//! Four settings are not posted with that form and are written the way their
//! own pages write them:
//! - `enabled` through the list's toggle (`toggle_event_type`), only when it
//!   differs;
//! - `telegram_guest_mode` with `telegram::store::save_event_type_mode`, the
//!   statement of Settings → Telegram;
//! - `timezone`: the form keeps a NULL timezone NULL when it posts back the
//!   one it follows, and pins one otherwise, so it can neither clear a
//!   timezone nor pin the followed one. After the form, a timezone that is
//!   still not the asked one is set with one `UPDATE`, as MCP's
//!   `set_weekly_hours` does.
//! - `ton_price`: the form's price section, checked by its own check
//!   (`ton_pay::validate_price_form`) and turned into what it stores
//!   (`ton_pay::price_write`) from the fields a browser would post for it,
//!   then written by `ton_pay::write_price` on the run's connection: the
//!   form's own save runs in a transaction of its own (`pool.begin()`),
//!   which SQLite refuses inside the run's. A price switched on makes a type
//!   with no Telegram mode chosen ask for it (`required`), as on the form;
//!   `telegram_guest_mode` in the item is written after it.
//!
//! Two more keys (ilia fork) go their own way too:
//! - `busy_calendars` is the form's calendar choice (`calendar_ids`, posted
//!   with the form, stored in `event_type_calendars`), named by display name
//!   among the calendars of the admin's sources as the `sources` section
//!   names them; null is no choice (every synced calendar blocks).
//! - `overrides` are the date overrides of the type's overrides page
//!   (`availability_overrides`), from today on (in the type's timezone):
//!   written after the form with one `DELETE` of the rows from today on and
//!   an `INSERT` per entry, the columns `create_override` writes. Rows
//!   before today are never compared, written or exported, and an entry
//!   before today is left out, so an inventory still applies after a day
//!   off has passed.
//!
//! `weekly_hours: null` means the profile's default hours: the event type's
//! rules are a copy of `user_availability_rules` as they are once the
//! profile section has run (the web form copies them the same way when its
//! schedule is empty).
//!
//! Pruning (`none`, `disable`, `delete`) works on the admin's personal event
//! types the document leaves out, listed in `unmanaged.event_types` in every
//! case. `delete` posts the list's delete (`delete_event_type`, which also
//! removes the type's past bookings) and is refused, before anything is
//! written, for a type that still has pending or confirmed bookings.

use super::{
    answered, check_opt_text, check_text, check_timezone, form, ApplyError, CalendarView, Ctx,
    DateOverride, Day, DayHours, EventTypeItem, EventTypes, FrequencyLimit, GuestMode,
    LocationType, Period, PhoneMode, Prune, TonPrice, Visibility, Window,
};
use crate::booking_window::{today_in, MAX_DAYS_AHEAD};
use crate::web::agent::dto;
use crate::web::agent::error::ToolError;
use crate::web::agent::ops_write::{self, EditState, Rule};
use crate::web::event_types::EventTypeForm;
use crate::web::ton_pay::{
    self,
    store::{Price, Terms},
    PriceForm, PriceWrite,
};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use chrono::{NaiveDate, NaiveTime};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

/// The days of a week in the document's order.
const WEEK: [Day; 7] = [
    Day::Mon,
    Day::Tue,
    Day::Wed,
    Day::Thu,
    Day::Fri,
    Day::Sat,
    Day::Sun,
];

/// The periods of a frequency limit in the document's order.
const PERIODS: [Period; 4] = [Period::Day, Period::Week, Period::Month, Period::Year];

/// What `ensure_user_avail_seeded` gives a profile with no hours: Mon–Fri
/// 09:00–17:00.
const SEED_START: &str = "09:00";
const SEED_END: &str = "17:00";

/// The keys of an item after `slug`, in the document's order: the order
/// of a report's changes.
const KEYS: [&str; 33] = [
    "title",
    "description",
    "duration_min",
    "slot_interval_min",
    "buffer_before",
    "buffer_after",
    "min_notice_min",
    "requires_confirmation",
    "pending_blocks_slot",
    "pending_expiry_hours",
    "visibility",
    "location_type",
    "location_value",
    "meeting_pattern_override",
    "timezone",
    "weekly_hours",
    "booking_horizon_days",
    "bookable_from",
    "bookable_until",
    "reminder_minutes",
    "max_additional_guests",
    "first_slot_only",
    "default_calendar_view",
    "week_start",
    "cancel_notice_min",
    "reschedule_notice_min",
    "sms_phone_mode",
    "frequency_limits",
    "busy_calendars",
    "overrides",
    "ton_price",
    "telegram_guest_mode",
    "enabled",
];

/// Settings not carried by the event type form (see the module comment).
const OFF_FORM: [&str; 5] = [
    "timezone",
    "enabled",
    "overrides",
    "ton_price",
    "telegram_guest_mode",
];

/// What an item's `ton_price` is. The release guards of the role grep for
/// this string in the source and in the binary (as for `FORMAT`): keep it a
/// literal, and used at runtime (the refusal of a price with neither or
/// both amounts).
pub(crate) const TON_PRICE_SHAPE: &str = "ton_price: null or {ton|usd, window_min, enabled}";

/// ilia fork: the marker of a calrs whose `ton_price` also takes the
/// demand ladder. Kept a literal and used at runtime, like
/// [`TON_PRICE_SHAPE`] (which stays as it was, so the role's older check
/// still matches).
pub(crate) const TON_PRICE_LADDER_SHAPE: &str =
    "ton_price: null or {ton|usd, window_min, enabled} or {ladder: true, window_min, enabled}";

/// ilia fork: what an item's `busy_calendars` is. The role greps for this
/// literal in the binary before it sends the key (an older calrs refuses it
/// as an unknown key): keep it a literal, and used at runtime (the refusal
/// of `[]`).
pub(crate) const BUSY_CALENDARS_SHAPE: &str = "busy_calendars: null or a list of calendar names";

/// ilia fork: what an item's `overrides` is; the role's marker, like
/// [`BUSY_CALENDARS_SHAPE`] (used in the refusal of an entry of another
/// shape).
pub(crate) const OVERRIDES_SHAPE: &str =
    "overrides: [{date, blocked: true} | {date, start, end}], from today on";

fn item_path(slug: &str) -> String {
    format!("event_types[{slug}]")
}

fn at(slug: &str, key: &str) -> String {
    format!("event_types[{slug}].{key}")
}

/// An MCP check's error (`invalid <field>: <why>`, the field possibly as
/// `patch.<field>`) under the item's path.
fn tool_error(slug: &str, error: ToolError) -> ApplyError {
    let message = error.message();
    if let Some((field, why)) = message
        .strip_prefix("invalid ")
        .and_then(|rest| rest.split_once(": "))
    {
        let field = field.strip_prefix("patch.").unwrap_or(field);
        return ApplyError::invalid(at(slug, field), why);
    }
    ApplyError::refused(item_path(slug), message)
}

/// A failed read of the MCP machinery (`EditState::load`): the details are
/// already in the log.
fn read_error(error: ToolError) -> ApplyError {
    ApplyError::internal("read event type", error.message())
}

/// A keyword of the document as the MCP enum of the same spelling.
fn mcp<T: DeserializeOwned>(key: &str) -> Result<T, ApplyError> {
    serde_json::from_value(Value::from(key))
        .map_err(|e| ApplyError::internal("convert a keyword", e))
}

// --- Reading ---

/// The id of the admin's personal event type `slug`.
async fn personal_id(ctx: &Ctx, slug: &str) -> Result<Option<String>, ApplyError> {
    Ok(sqlx::query_scalar(
        "SELECT et.id FROM event_types et
         JOIN accounts a ON a.id = et.account_id
         WHERE a.user_id = ? AND et.team_id IS NULL AND et.slug = ?
         ORDER BY et.created_at LIMIT 1",
    )
    .bind(&ctx.me.id)
    .bind(slug)
    .fetch_optional(ctx.pool())
    .await?)
}

/// Every personal event type of the admin: id and slug, by slug.
async fn personal_types(ctx: &Ctx) -> Result<Vec<(String, String)>, ApplyError> {
    Ok(sqlx::query_as(
        "SELECT et.id, et.slug FROM event_types et
         JOIN accounts a ON a.id = et.account_id
         WHERE a.user_id = ? AND et.team_id IS NULL
         ORDER BY et.slug, et.created_at, et.id",
    )
    .bind(&ctx.me.id)
    .fetch_all(ctx.pool())
    .await?)
}

/// The profile's default hours (`user_availability_rules`), sorted as
/// `EditState` holds rules; the web's seed when there are none.
async fn profile_rules(ctx: &Ctx) -> Result<Vec<Rule>, ApplyError> {
    let rules: Vec<Rule> = sqlx::query_as(
        "SELECT day_of_week, start_time, end_time FROM user_availability_rules
         WHERE user_id = ? ORDER BY day_of_week, start_time, end_time",
    )
    .bind(&ctx.me.id)
    .fetch_all(ctx.pool())
    .await?;
    if rules.is_empty() {
        return Ok((1..=5)
            .map(|day| (day, SEED_START.to_string(), SEED_END.to_string()))
            .collect());
    }
    Ok(rules)
}

/// The stored price of `et_id` in the document's shape; `None` when free.
async fn stored_price(ctx: &Ctx, et_id: &str) -> Result<Option<TonPrice>, ApplyError> {
    let mut conn = ctx.pool().acquire().await?;
    let row = crate::web::agent::ops_ton::read_price(&mut conn, et_id)
        .await
        .map_err(read_error)?;
    Ok(row.map(|(terms, window_min, enabled)| price_item(terms, window_min, enabled != 0)))
}

/// A price as the document states it, the amount as the form shows it.
fn price_item(terms: Terms, window_min: Option<i64>, enabled: bool) -> TonPrice {
    let (ton, usd, ladder) = match terms {
        Terms::Fixed(price @ Price::Ton(_)) => (Some(price.amount()), None, None),
        Terms::Fixed(price @ Price::Usd(_)) => (None, Some(price.amount()), None),
        Terms::Ladder => (None, None, Some(true)),
    };
    TonPrice {
        ton,
        usd,
        ladder,
        window_min,
        enabled,
    }
}

/// The calendars of the admin's sources: id and name (the display name, its
/// address when it has none, as the `sources` section names them).
struct Calendars(Vec<(String, String)>);

impl Calendars {
    async fn load(ctx: &Ctx) -> Result<Self, ApplyError> {
        let rows: Vec<(String, Option<String>, String)> = sqlx::query_as(
            "SELECT c.id, c.display_name, c.href FROM calendars c
             JOIN caldav_sources cs ON cs.id = c.source_id
             JOIN accounts a ON a.id = cs.account_id
             WHERE a.user_id = ? ORDER BY c.id",
        )
        .bind(&ctx.me.id)
        .fetch_all(ctx.pool())
        .await?;
        Ok(Self(
            rows.into_iter()
                .map(|(id, display_name, href)| {
                    let name = display_name
                        .map(|n| n.trim().to_string())
                        .filter(|n| !n.is_empty())
                        .unwrap_or(href);
                    (id, name)
                })
                .collect(),
        ))
    }

    fn sharing(&self, name: &str) -> usize {
        self.0.iter().filter(|(_, n)| n == name).count()
    }

    /// Calendar `id` in a document: its name when no other calendar of the
    /// admin has it, else its id (which no document names, so it never
    /// equals a desired list).
    fn label(&self, id: &str) -> String {
        match self.0.iter().find(|(i, _)| i == id) {
            Some((_, name)) if self.sharing(name) == 1 => name.clone(),
            _ => id.to_string(),
        }
    }

    /// The calendar choice `ids` as `busy_calendars`: null for none, else
    /// the sorted labels.
    fn labels(&self, ids: &[String]) -> Option<Vec<String>> {
        if ids.is_empty() {
            return None;
        }
        let mut names: Vec<String> = ids.iter().map(|id| self.label(id)).collect();
        names.sort();
        Some(names)
    }

    /// Whether a document can name every calendar of `ids`.
    fn nameable(&self, ids: &[String]) -> bool {
        ids.iter().all(|id| {
            self.0
                .iter()
                .any(|(i, name)| i == id && self.sharing(name) == 1)
        })
    }

    /// The id of the one calendar named `name`, or why there is none.
    fn named(&self, path: &str, name: &str) -> Result<String, ApplyError> {
        let found: Vec<&String> = self
            .0
            .iter()
            .filter(|(_, n)| n == name)
            .map(|(id, _)| id)
            .collect();
        match found.as_slice() {
            [id] => Ok((*id).clone()),
            [] => {
                let mut names: Vec<String> = self.0.iter().map(|(_, n)| format!("{n:?}")).collect();
                names.sort();
                names.dedup();
                let known = if names.is_empty() {
                    "no source of this admin has listed a calendar yet: sync it once first"
                        .to_string()
                } else {
                    format!("the calendars are {}", names.join(", "))
                };
                Err(ApplyError::invalid(
                    path,
                    format!("no calendar named {name:?}; {known}"),
                ))
            }
            several => Err(ApplyError::invalid(
                path,
                format!(
                    "{} calendars of this admin's sources are named {name:?}: rename one in the \
                     calendar server, then sync the source",
                    several.len()
                ),
            )),
        }
    }
}

/// Today in the timezone an event type follows (its own, else the
/// profile's, else the server's): where the managed date overrides start.
async fn today_for(ctx: &Ctx, timezone: Option<&str>) -> NaiveDate {
    let tz = match timezone
        .unwrap_or(&ctx.me.timezone)
        .parse::<chrono_tz::Tz>()
    {
        Ok(tz) => tz,
        Err(_) => crate::web::availability::get_host_tz(ctx.pool(), "").await,
    };
    today_in(tz)
}

/// A stored override row in the document's shape.
fn override_item(
    date: String,
    start: Option<String>,
    end: Option<String>,
    blocked: bool,
) -> DateOverride {
    if blocked {
        DateOverride {
            date,
            blocked: Some(true),
            start: None,
            end: None,
        }
    } else {
        DateOverride {
            date,
            blocked: None,
            start,
            end,
        }
    }
}

/// The stored date overrides of `et_id` from `today` on, sorted.
async fn stored_overrides(
    ctx: &Ctx,
    et_id: &str,
    today: NaiveDate,
) -> Result<Vec<DateOverride>, ApplyError> {
    let rows: Vec<(String, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT date, start_time, end_time, is_blocked FROM availability_overrides
         WHERE event_type_id = ? AND date >= ?",
    )
    .bind(et_id)
    .bind(today.format("%Y-%m-%d").to_string())
    .fetch_all(ctx.pool())
    .await?;
    let mut out: Vec<DateOverride> = rows
        .into_iter()
        .map(|(date, start, end, blocked)| override_item(date, start, end, blocked != 0))
        .collect();
    out.sort();
    Ok(out)
}

/// What is stored for one event type beside nothing else: the form state,
/// the Telegram mode, the price and the date overrides from `today` on (or
/// the new form's start for a slug that does not exist yet).
#[derive(Clone)]
struct Current {
    state: EditState,
    mode: GuestMode,
    price: Option<TonPrice>,
    overrides: Vec<DateOverride>,
    today: NaiveDate,
}

impl Current {
    async fn load(ctx: &Ctx, et_id: &str) -> Result<Self, ApplyError> {
        let state = EditState::load(ctx.pool(), et_id)
            .await
            .map_err(read_error)?;
        let today = today_for(ctx, state.timezone.as_deref()).await;
        Ok(Self {
            mode: guest_mode(ctx, et_id).await,
            price: stored_price(ctx, et_id).await?,
            overrides: stored_overrides(ctx, et_id, today).await?,
            today,
            state,
        })
    }

    async fn new_type(ctx: &Ctx, slug: &str, profile: &[Rule]) -> Self {
        let state = new_form(slug, &ctx.me, profile);
        Self {
            today: today_for(ctx, state.timezone.as_deref()).await,
            state,
            mode: GuestMode::Off,
            price: None,
            overrides: Vec::new(),
        }
    }

    fn item(&self, calendars: &Calendars) -> EventTypeItem {
        item_of(
            &self.state,
            self.mode,
            &self.price,
            &self.overrides,
            calendars,
        )
    }
}

/// The stored guest Telegram mode (`off` when none is stored).
async fn guest_mode(ctx: &Ctx, et_id: &str) -> GuestMode {
    GuestMode::from_key(crate::telegram::store::event_type_mode(ctx.pool(), et_id).await)
        .unwrap_or(GuestMode::Off)
}

// --- The document's shape ---

fn days_of(rules: &[Rule]) -> Vec<DayHours> {
    WEEK.iter()
        .filter_map(|day| {
            let windows: Vec<Window> = rules
                .iter()
                .filter(|(dow, _, _)| *dow == day.day_of_week())
                .map(|(_, start, end)| Window {
                    start: start.clone(),
                    end: end.clone(),
                })
                .collect();
            (!windows.is_empty()).then_some(DayHours { day: *day, windows })
        })
        .collect()
}

fn period_rank(period: Period) -> usize {
    PERIODS.iter().position(|p| *p == period).unwrap_or(0)
}

fn limits_of(limits: &[(i64, String, bool)]) -> Vec<FrequencyLimit> {
    let mut out: Vec<FrequencyLimit> = limits
        .iter()
        .filter_map(|(count, period, _)| {
            Period::from_key(period).map(|period| FrequencyLimit {
                count: *count,
                period,
            })
        })
        .collect();
    out.sort_by_key(|l| (period_rank(l.period), l.count));
    out
}

/// A form state, its Telegram mode, its price and its date overrides as a
/// document item, every key given.
fn item_of(
    state: &EditState,
    mode: GuestMode,
    price: &Option<TonPrice>,
    overrides: &[DateOverride],
    calendars: &Calendars,
) -> EventTypeItem {
    EventTypeItem {
        slug: state.slug.clone(),
        title: Some(state.title.clone()),
        description: Some(state.description.clone()),
        duration_min: Some(state.duration_min),
        slot_interval_min: Some(state.slot_interval_min),
        buffer_before: state.buffer_before,
        buffer_after: state.buffer_after,
        min_notice_min: state.min_notice_min,
        requires_confirmation: Some(state.requires_confirmation),
        pending_blocks_slot: Some(state.pending_blocks_slot),
        pending_expiry_hours: Some(state.pending_expiry_hours),
        visibility: Visibility::from_key(&state.visibility),
        location_type: LocationType::from_key(&state.location_type),
        location_value: Some(state.location_value.clone()),
        meeting_pattern_override: Some(state.meeting_pattern_override.clone()),
        timezone: Some(state.timezone.clone()),
        weekly_hours: Some(Some(days_of(&state.weekly_hours))),
        booking_horizon_days: Some(state.booking_horizon_days),
        bookable_from: Some(state.bookable_from.clone()),
        bookable_until: Some(state.bookable_until.clone()),
        reminder_minutes: Some(state.reminder_minutes),
        max_additional_guests: Some(state.max_additional_guests),
        first_slot_only: Some(state.first_slot_only),
        default_calendar_view: CalendarView::from_key(&state.default_calendar_view),
        week_start: Some(state.week_start.as_deref().and_then(Day::from_key)),
        cancel_notice_min: Some(state.cancel_notice_min),
        reschedule_notice_min: Some(state.reschedule_notice_min),
        sms_phone_mode: PhoneMode::from_key(&state.sms_phone_mode),
        frequency_limits: Some(limits_of(&state.frequency_limits)),
        busy_calendars: Some(calendars.labels(&state.calendar_ids)),
        overrides: Some(overrides.to_vec()),
        ton_price: Some(price.clone()),
        telegram_guest_mode: Some(mode),
        enabled: Some(state.enabled),
    }
}

fn as_map(item: &EventTypeItem) -> Result<Map<String, Value>, ApplyError> {
    match serde_json::to_value(item) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err(ApplyError::internal("event type item", "not an object")),
        Err(e) => Err(ApplyError::internal("event type item", e)),
    }
}

// --- The desired state ---

/// An item's desired state: the form state with the item applied, the
/// Telegram mode, the price and the date overrides.
struct Wanted {
    state: EditState,
    mode: GuestMode,
    /// The item gives `telegram_guest_mode`: written after the price.
    mode_given: bool,
    price: Option<TonPrice>,
    /// What the price section stores, when the item gives `ton_price`.
    price_write: Option<PriceWrite>,
    /// From `today` on, sorted.
    overrides: Vec<DateOverride>,
    /// The item gives `overrides`.
    overrides_given: bool,
    today: NaiveDate,
}

impl Wanted {
    fn item(&self, calendars: &Calendars) -> EventTypeItem {
        item_of(
            &self.state,
            self.mode,
            &self.price,
            &self.overrides,
            calendars,
        )
    }
}

/// `busy_calendars` as the form's `calendar_ids`, sorted as stored.
fn calendars_wanted(
    slug: &str,
    names: Option<&[String]>,
    calendars: &Calendars,
) -> Result<Vec<String>, ApplyError> {
    let Some(names) = names else {
        return Ok(Vec::new());
    };
    let path = at(slug, "busy_calendars");
    if names.is_empty() {
        return Err(ApplyError::invalid(
            path,
            format!("use null for every synced calendar ({BUSY_CALENDARS_SHAPE})"),
        ));
    }
    let mut ids = BTreeSet::new();
    let mut listed = BTreeSet::new();
    for (i, name) in names.iter().enumerate() {
        let here = format!("{path}[{i}]");
        check_text(&here, name, None)?;
        if !listed.insert(name.as_str()) {
            return Err(ApplyError::invalid(here, "listed twice"));
        }
        ids.insert(calendars.named(&here, name)?);
    }
    Ok(ids.into_iter().collect())
}

/// A date written exactly as `YYYY-MM-DD`.
fn exact_date(path: &str, date: &str) -> Result<NaiveDate, ApplyError> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()
        .filter(|d| d.format("%Y-%m-%d").to_string() == date)
        .ok_or_else(|| ApplyError::invalid(path, "use YYYY-MM-DD"))
}

/// A time of day written exactly as `HH:MM`.
fn exact_time(path: &str, time: &str) -> Result<NaiveTime, ApplyError> {
    NaiveTime::parse_from_str(time, "%H:%M")
        .ok()
        .filter(|t| t.format("%H:%M").to_string() == time)
        .ok_or_else(|| ApplyError::invalid(path, "use HH:MM, 00:00 to 23:59"))
}

/// One entry of `overrides` checked as the MCP `add_date_override` checks
/// it (a blocked day or custom hours, start before end, at most
/// `MAX_DAYS_AHEAD` days ahead), spelled canonically. `None`: dated before
/// `today`, left out.
fn override_wanted(
    path: &str,
    entry: &DateOverride,
    today: NaiveDate,
) -> Result<Option<DateOverride>, ApplyError> {
    let date = exact_date(&format!("{path}.date"), &entry.date)?;
    match (entry.blocked, &entry.start, &entry.end) {
        (Some(true), None, None) => {}
        (None, Some(start), Some(end)) => {
            let s = exact_time(&format!("{path}.start"), start)?;
            let e = exact_time(&format!("{path}.end"), end)?;
            if s >= e {
                return Err(ApplyError::invalid(
                    format!("{path}.end"),
                    "must be after start",
                ));
            }
        }
        (Some(false), _, _) => {
            return Err(ApplyError::invalid(
                format!("{path}.blocked"),
                "only true; for custom hours give start and end and leave blocked out",
            ))
        }
        _ => return Err(ApplyError::invalid(path, OVERRIDES_SHAPE)),
    }
    let last = today + chrono::Duration::days(MAX_DAYS_AHEAD);
    if date > last {
        return Err(ApplyError::invalid(
            format!("{path}.date"),
            format!("must be {last} at the latest"),
        ));
    }
    Ok((date >= today).then(|| entry.clone()))
}

/// `overrides` from `today` on, checked, without repeats, sorted.
fn overrides_wanted(
    slug: &str,
    entries: &[DateOverride],
    today: NaiveDate,
) -> Result<Vec<DateOverride>, ApplyError> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(entries.len());
    for (i, entry) in entries.iter().enumerate() {
        let path = at(slug, &format!("overrides[{i}]"));
        let wanted = override_wanted(&path, entry, today)?;
        if !seen.insert(entry.clone()) {
            return Err(ApplyError::invalid(path, "listed twice"));
        }
        out.extend(wanted);
    }
    out.sort();
    Ok(out)
}

/// `ton_price` as the fields a browser posts for the form's price section,
/// checked by the form's own check and turned into what the form stores.
/// null is the section unchecked and blank (no price). The amount must be
/// spelled as the form shows the stored price, so the document and an
/// export compare as text.
fn price_wanted(slug: &str, price: Option<&TonPrice>) -> Result<PriceWrite, ApplyError> {
    let path = at(slug, "ton_price");
    let window = price.and_then(|p| p.window_min).map(|w| w.to_string());
    let (currency, amount, mode) = match price.map(|p| (&p.ton, &p.usd, p.ladder)) {
        None => (None, "", None),
        Some((Some(a), None, None)) => (Some("TON"), a.as_str(), None),
        Some((None, Some(a), None)) => (Some("USD"), a.as_str(), None),
        Some((None, None, Some(true))) => (None, "", Some("ladder")),
        Some(_) => return Err(ApplyError::invalid(&path, TON_PRICE_LADDER_SHAPE)),
    };
    let form = PriceForm {
        enabled: price.filter(|p| p.enabled).map(|_| "on"),
        amount: Some(amount),
        currency,
        window_min: window.as_deref(),
        guest_telegram: None,
        mode,
    };
    ton_pay::validate_price_form(&form, None)
        .map_err(|key| ApplyError::invalid(&path, crate::i18n::translate("en", key, None)))?;
    let write = ton_pay::price_write(&form).ok_or_else(|| {
        ApplyError::internal(
            "price",
            "the price section stores nothing for a valid price",
        )
    })?;
    if let (
        Some(given),
        PriceWrite::Save {
            terms: Terms::Fixed(stored),
            ..
        },
    ) = (price, write)
    {
        if stored.amount() != amount {
            let key = if given.ton.is_some() { "ton" } else { "usd" };
            return Err(ApplyError::invalid(
                format!("{path}.{key}"),
                format!("spell it {:?}", stored.amount()),
            ));
        }
    }
    Ok(write)
}

/// The item as the MCP edit (`EventTypePatch`), without what `EditState`
/// does not take from a patch (timezone, weekly hours, frequency limits,
/// Telegram, enabled).
fn patch_of(item: &EventTypeItem) -> Result<dto::EventTypePatch, ApplyError> {
    Ok(dto::EventTypePatch {
        slug: None,
        title: item.title.clone(),
        description: item.description.clone(),
        duration_min: item.duration_min,
        slot_interval_min: item.slot_interval_min,
        buffer_before: item.buffer_before,
        buffer_after: item.buffer_after,
        min_notice_min: item.min_notice_min,
        requires_confirmation: item.requires_confirmation,
        pending_blocks_slot: item.pending_blocks_slot,
        pending_expiry_hours: item.pending_expiry_hours,
        visibility: item.visibility.map(|v| mcp(v.key())).transpose()?,
        location_type: item.location_type.map(|v| mcp(v.key())).transpose()?,
        location_value: item.location_value.clone(),
        meeting_pattern_override: item.meeting_pattern_override.clone(),
        timezone: None,
        booking_horizon_days: item.booking_horizon_days,
        bookable_from: item.bookable_from.clone(),
        bookable_until: item.bookable_until.clone(),
        reminder_minutes: item.reminder_minutes,
        max_additional_guests: item.max_additional_guests,
        first_slot_only: item.first_slot_only,
        default_calendar_view: item
            .default_calendar_view
            .map(|v| mcp(v.key()))
            .transpose()?,
        week_start: item.week_start.map(|day| day.map(Day::to_mcp)),
        cancel_notice_min: item.cancel_notice_min,
        reschedule_notice_min: item.reschedule_notice_min,
        sms_phone_mode: item.sms_phone_mode.map(|v| mcp(v.key())).transpose()?,
    })
}

/// Frequency limits as `booking_frequency_limits` rows: each count 1 to
/// what the handler stores (a positive `i32`), each period once.
fn limit_rows(
    slug: &str,
    limits: &[FrequencyLimit],
) -> Result<Vec<(i64, String, bool)>, ApplyError> {
    let mut seen = BTreeSet::new();
    let mut rows = Vec::with_capacity(limits.len());
    for (i, limit) in limits.iter().enumerate() {
        let here = at(slug, &format!("frequency_limits[{i}]"));
        if !(1..=i64::from(i32::MAX)).contains(&limit.count) {
            return Err(ApplyError::invalid(
                format!("{here}.count"),
                "must be a positive number of bookings",
            ));
        }
        if !seen.insert(limit.period.key()) {
            return Err(ApplyError::invalid(
                format!("{here}.period"),
                "each period may appear once",
            ));
        }
        rows.push((limit.count, limit.period.key().to_string(), false));
    }
    Ok(rows)
}

/// `base` with `item` applied, checked the way MCP checks an edit (and the
/// price the way the form's price section checks it). `profile` is what
/// `weekly_hours: null` stands for.
async fn wanted(
    ctx: &Ctx,
    item: &EventTypeItem,
    base: Current,
    profile: &[Rule],
    calendars: &Calendars,
) -> Result<Wanted, ApplyError> {
    let slug = item.slug.as_str();
    let Current {
        mut state,
        mode: base_mode,
        price: base_price,
        overrides: base_overrides,
        today,
    } = base;
    state
        .apply(ctx.pool(), "", "", patch_of(item)?)
        .await
        .map_err(|e| tool_error(slug, e))?;
    if let Some(timezone) = &item.timezone {
        state.timezone = timezone.clone();
    }
    if let Some(hours) = &item.weekly_hours {
        state.weekly_hours = match hours {
            Some(days) => ops_write::weekly_rules("weekly_hours", &DayHours::to_mcp(days))
                .map_err(|e| tool_error(slug, e))?,
            None => profile.to_vec(),
        };
    }
    if let Some(limits) = &item.frequency_limits {
        state.frequency_limits = limit_rows(slug, limits)?;
    }
    if let Some(enabled) = item.enabled {
        state.enabled = enabled;
    }
    if let Some(names) = &item.busy_calendars {
        state.calendar_ids = calendars_wanted(slug, names.as_deref(), calendars)?;
    }
    // Today in the timezone the type follows once written: the one verify
    // reads it with, so a run that changes the timezone near midnight
    // replaces the same days it compares.
    let today = if item.timezone.is_some() && item.overrides.is_some() {
        today_for(ctx, state.timezone.as_deref()).await
    } else {
        today
    };
    let overrides = match &item.overrides {
        Some(entries) => overrides_wanted(slug, entries, today)?,
        None => base_overrides,
    };
    let (price, price_write) = match &item.ton_price {
        Some(price) => (price.clone(), Some(price_wanted(slug, price.as_ref())?)),
        None => (base_price, None),
    };
    Ok(Wanted {
        state,
        mode: item.telegram_guest_mode.unwrap_or(base_mode),
        mode_given: item.telegram_guest_mode.is_some(),
        price,
        price_write,
        overrides,
        overrides_given: item.overrides.is_some(),
        today,
    })
}

/// What the web's new event type form starts with (`new_event_type_form`),
/// as a form state. Its timezone is the admin's, pinned, as the create
/// handler stores it.
fn new_form(slug: &str, me: &crate::models::User, profile: &[Rule]) -> EditState {
    let pending = crate::web::pending::PendingSettings::DEFAULT;
    EditState {
        slug: slug.to_string(),
        title: String::new(),
        description: None,
        duration_min: 30,
        slot_interval_min: None,
        buffer_before: Some(0),
        buffer_after: Some(0),
        min_notice_min: Some(ops_write::NEW_FORM_MIN_NOTICE_MIN),
        requires_confirmation: false,
        pending_blocks_slot: pending.blocks_slot,
        pending_expiry_hours: pending.expiry_hours.map(i64::from),
        visibility: Visibility::Public.key().to_string(),
        location_type: LocationType::Link.key().to_string(),
        location_value: None,
        meeting_pattern_override: None,
        timezone: Some(me.timezone.clone()),
        booking_horizon_days: None,
        bookable_from: None,
        bookable_until: None,
        reminder_minutes: Some(ops_write::NEW_FORM_REMINDER_MINUTES),
        max_additional_guests: 0,
        first_slot_only: false,
        default_calendar_view: CalendarView::Month.key().to_string(),
        week_start: None,
        cancel_notice_min: None,
        reschedule_notice_min: None,
        sms_phone_mode: PhoneMode::Off.key().to_string(),
        scheduling_mode: "round_robin".to_string(),
        resource_scheduling_mode: "all".to_string(),
        enabled: true,
        weekly_hours: profile.to_vec(),
        calendar_ids: Vec::new(),
        resource_ids: Vec::new(),
        frequency_limits: Vec::new(),
    }
}

/// The keys `item` gives whose stored value (`before`) is not the desired
/// one: key, stored value, desired value, in the document's shape.
fn differences(
    item: &EventTypeItem,
    before: &Current,
    want: &Wanted,
    calendars: &Calendars,
) -> Result<Vec<(String, Value, Value)>, ApplyError> {
    let given = as_map(item)?;
    let stored = as_map(&before.item(calendars))?;
    let desired = as_map(&want.item(calendars))?;
    Ok(KEYS
        .iter()
        .filter(|key| given.contains_key(**key))
        .filter_map(|key| {
            let b = stored.get(*key).cloned().unwrap_or(Value::Null);
            let a = desired.get(*key).cloned().unwrap_or(Value::Null);
            (b != a).then(|| (key.to_string(), b, a))
        })
        .collect())
}

/// `item` without the keys whose value is already the stored one: those
/// need neither a check nor a write, so a document that says what is
/// stored (an export) applies as it is, even where the stored value is one
/// the checks would refuse today. `telegram_guest_mode` stays: a price the
/// run switches on can change the stored mode (`off` when none was chosen,
/// then `required`), and a mode the item gives is written after it.
fn without_stored(
    item: &EventTypeItem,
    current: &Current,
    calendars: &Calendars,
) -> Result<EventTypeItem, ApplyError> {
    let stored = as_map(&current.item(calendars))?;
    let mut given = as_map(item)?;
    given.retain(|key, value| {
        key == "slug" || key == "telegram_guest_mode" || stored.get(key) != Some(value)
    });
    serde_json::from_value(Value::Object(given))
        .map_err(|e| ApplyError::internal("event type item", e))
}

// --- Canonical input ---

/// What `check` refuses: a slug the form would rewrite, text the handler
/// would trim or drop, a timezone spelled otherwise than `chrono_tz` names
/// it, unordered windows.
fn check_slug(slug: &str) -> Result<(), ApplyError> {
    ops_write::check_slug("slug", slug)
        .map(|_| ())
        .map_err(|e| {
            ApplyError::invalid(
                item_path(slug),
                e.message()
                    .strip_prefix("invalid slug: ")
                    .unwrap_or(e.message()),
            )
        })
}

/// The canonical checks of the keys an item gives (its slug is checked
/// by `check_slug`).
fn check_item(item: &EventTypeItem) -> Result<(), ApplyError> {
    let slug = item.slug.as_str();
    if let Some(title) = &item.title {
        check_text(&at(slug, "title"), title, None)?;
    }
    for (key, value) in [
        ("description", &item.description),
        ("location_value", &item.location_value),
        ("meeting_pattern_override", &item.meeting_pattern_override),
        ("bookable_from", &item.bookable_from),
        ("bookable_until", &item.bookable_until),
    ] {
        if let Some(value) = value {
            check_opt_text(&at(slug, key), value.as_deref(), None)?;
        }
    }
    if let Some(Some(timezone)) = &item.timezone {
        let path = at(slug, "timezone");
        let tz = check_timezone(&path, timezone)?;
        if tz.name() != timezone {
            return Err(ApplyError::invalid(
                path,
                format!("spell it {:?}", tz.name()),
            ));
        }
    }
    if let Some(Some(price)) = &item.ton_price {
        match (&price.ton, &price.usd, price.ladder) {
            (Some(amount), None, None) => check_text(&at(slug, "ton_price.ton"), amount, None)?,
            (None, Some(amount), None) => check_text(&at(slug, "ton_price.usd"), amount, None)?,
            (None, None, Some(true)) => {}
            (None, None, Some(false)) => {
                return Err(ApplyError::invalid(
                    at(slug, "ton_price.ladder"),
                    "only true; for a fixed price give ton or usd and leave ladder out",
                ))
            }
            _ => {
                return Err(ApplyError::invalid(
                    at(slug, "ton_price"),
                    format!(
                        "give exactly one of ton, usd and ladder: true ({TON_PRICE_LADDER_SHAPE}; \
                         older form: {TON_PRICE_SHAPE})"
                    ),
                ))
            }
        }
    }
    if let Some(Some(days)) = &item.weekly_hours {
        for (i, day) in days.iter().enumerate() {
            for (j, window) in day.windows.iter().enumerate() {
                let here = at(slug, &format!("weekly_hours[{i}].windows[{j}]"));
                check_text(&format!("{here}.start"), &window.start, None)?;
                check_text(&format!("{here}.end"), &window.end, None)?;
            }
            if day
                .windows
                .windows(2)
                .any(|pair| pair[1].start < pair[0].start)
            {
                return Err(ApplyError::invalid(
                    at(slug, &format!("weekly_hours[{i}].windows")),
                    "list the windows of a day in order",
                ));
            }
        }
    }
    Ok(())
}

// --- The section ---

/// Every check of the section, reading only (pipeline step 1).
pub(super) async fn check(ctx: &Ctx, section: &EventTypes) -> Result<(), ApplyError> {
    let mut slugs = BTreeSet::new();
    for item in &section.items {
        if !slugs.insert(item.slug.as_str()) {
            return Err(ApplyError::invalid(item_path(&item.slug), "listed twice"));
        }
        check_slug(&item.slug)?;
    }

    let profile = profile_rules(ctx).await?;
    let calendars = Calendars::load(ctx).await?;
    let has_account: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM accounts WHERE user_id = ?)")
            .bind(&ctx.me.id)
            .fetch_one(ctx.pool())
            .await?;
    for item in &section.items {
        let slug = item.slug.as_str();
        let taken_by_team: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM event_types et
                            JOIN accounts a ON a.id = et.account_id
                            WHERE a.user_id = ? AND et.slug = ? AND et.team_id IS NOT NULL)",
        )
        .bind(&ctx.me.id)
        .bind(slug)
        .fetch_one(ctx.pool())
        .await?;
        if taken_by_team {
            return Err(ApplyError::invalid(
                item_path(slug),
                "the slug is taken by a team event type of this account",
            ));
        }
        match personal_id(ctx, slug).await? {
            Some(et_id) => {
                let base = Current::load(ctx, &et_id).await?;
                let item = without_stored(item, &base, &calendars)?;
                check_item(&item)?;
                wanted(ctx, &item, base, &profile, &calendars).await?;
            }
            None => {
                if !has_account {
                    return Err(ApplyError::invalid(
                        item_path(slug),
                        format!("{} has no scheduling account to create it in", ctx.me.email),
                    ));
                }
                if item.title.is_none() {
                    return Err(ApplyError::invalid(
                        at(slug, "title"),
                        "required to create the event type",
                    ));
                }
                check_item(item)?;
                let base = Current::new_type(ctx, slug, &profile).await;
                wanted(ctx, item, base, &profile, &calendars).await?;
            }
        }
    }

    if section.prune() == Prune::Delete {
        for (et_id, slug) in personal_types(ctx).await? {
            if slugs.contains(slug.as_str()) {
                continue;
            }
            let active: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM bookings
                 WHERE event_type_id = ? AND status IN ('confirmed', 'pending')",
            )
            .bind(&et_id)
            .fetch_one(ctx.pool())
            .await?;
            if active > 0 {
                return Err(ApplyError::invalid(
                    item_path(&slug),
                    format!(
                        "{active} pending or confirmed bookings: prune \"delete\" would leave them \
                         without an event type; use \"disable\", or cancel them first"
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Post the forms and record the changes (pipeline step 3).
pub(super) async fn apply(ctx: &mut Ctx, section: &EventTypes) -> Result<(), ApplyError> {
    let profile = profile_rules(ctx).await?;
    let calendars = Calendars::load(ctx).await?;
    for item in &section.items {
        match personal_id(ctx, &item.slug).await? {
            Some(et_id) => update(ctx, item, &et_id, &profile, &calendars).await?,
            None => create(ctx, item, &profile, &calendars).await?,
        }
    }

    let managed: BTreeSet<&str> = section.items.iter().map(|i| i.slug.as_str()).collect();
    let unmanaged: Vec<(String, String)> = personal_types(ctx)
        .await?
        .into_iter()
        .filter(|(_, slug)| !managed.contains(slug.as_str()))
        .collect();
    ctx.unmanaged_event_types = unmanaged.iter().map(|(_, slug)| slug.clone()).collect();
    for (et_id, slug) in &unmanaged {
        match section.prune() {
            Prune::None => {}
            Prune::Disable => {
                let enabled = EditState::load(ctx.pool(), et_id)
                    .await
                    .map_err(read_error)?
                    .enabled;
                if enabled {
                    ctx.record(at(slug, "enabled"), true, false);
                    toggle(ctx, slug).await?;
                }
            }
            Prune::Delete => {
                ctx.record(item_path(slug), "present", Value::Null);
                let (headers, csrf) = form::<crate::web::CsrfForm>(&[]).await?;
                let response = crate::web::event_types::delete_event_type(
                    State(ctx.state.clone()),
                    ctx.auth_user(),
                    headers,
                    Path(slug.clone()),
                    csrf,
                )
                .await
                .into_response();
                answered(&item_path(slug), response).await?;
            }
        }
    }
    Ok(())
}

/// A new event type: the new form's defaults with the item applied, posted
/// to `create_event_type`.
async fn create(
    ctx: &mut Ctx,
    item: &EventTypeItem,
    profile: &[Rule],
    calendars: &Calendars,
) -> Result<(), ApplyError> {
    let slug = item.slug.as_str();
    let path = item_path(slug);
    let base = Current::new_type(ctx, slug, profile).await;
    let want = wanted(ctx, item, base, profile, calendars).await?;
    ctx.record(&path, Value::Null, "created");

    let fields = want.state.form_fields(&ctx.me.timezone, true, true);
    let (headers, form) = form::<EventTypeForm>(&fields).await?;
    ops_write::web_form_checks(ctx.pool(), &ctx.me, None, &form.0)
        .await
        .map_err(|e| tool_error(slug, e))?;
    let response = crate::web::event_types::create_event_type(
        State(ctx.state.clone()),
        ctx.auth_user(),
        headers,
        form,
    )
    .await
    .into_response();
    answered(&path, response).await?;
    let et_id = personal_id(ctx, slug).await?.ok_or_else(|| {
        ApplyError::refused(
            &path,
            "the form answered as saved but created no event type",
        )
    })?;
    off_form(ctx, slug, &et_id, &want, false).await
}

/// An event type that exists: each differing setting recorded, the form
/// posted when one it carries differs.
async fn update(
    ctx: &mut Ctx,
    item: &EventTypeItem,
    et_id: &str,
    profile: &[Rule],
    calendars: &Calendars,
) -> Result<(), ApplyError> {
    let slug = item.slug.as_str();
    let before = Current::load(ctx, et_id).await?;
    let item = &without_stored(item, &before, calendars)?;
    let want = wanted(ctx, item, before.clone(), profile, calendars).await?;
    let changes = differences(item, &before, &want, calendars)?;
    if changes.is_empty() {
        return Ok(());
    }
    let changed = |key: &str| changes.iter().any(|(k, _, _)| k == key);
    let by_form = changes
        .iter()
        .any(|(key, _, _)| !OFF_FORM.contains(&key.as_str()));
    let send_pending = changed("pending_blocks_slot") || changed("pending_expiry_hours");
    let send_week_start = changed("week_start");
    for (key, b, a) in &changes {
        ctx.record(at(slug, key), b, a);
    }

    let before = before.state;
    if by_form {
        if want.state.weekly_hours.is_empty() {
            // The form would give it the profile's default hours.
            return Err(ApplyError::refused(
                at(slug, "weekly_hours"),
                "the event type has no weekly hours, and saving its form would give it the \
                 profile's; give weekly_hours",
            ));
        }
        let followed = crate::web::availability::event_type_followed_tz(ctx.pool(), et_id).await;
        let fields = want
            .state
            .form_fields(&followed, send_pending, send_week_start);
        let (headers, form) = form::<EventTypeForm>(&fields).await?;
        ops_write::web_form_checks(ctx.pool(), &ctx.me, Some(et_id), &form.0)
            .await
            .map_err(|e| tool_error(slug, e))?;
        let response = crate::web::event_types::update_event_type(
            State(ctx.state.clone()),
            ctx.auth_user(),
            headers,
            Path(before.slug.clone()),
            form,
        )
        .await
        .into_response();
        answered(&item_path(slug), response).await?;
    }
    off_form(ctx, slug, et_id, &want, true).await
}

/// The settings the form does not carry, each written only when it differs.
/// `record`: a Telegram mode the price changes (the item giving none) is
/// recorded, as the form's own effect (not for a created type, one entry).
async fn off_form(
    ctx: &mut Ctx,
    slug: &str,
    et_id: &str,
    want: &Wanted,
    record: bool,
) -> Result<(), ApplyError> {
    let now = EditState::load(ctx.pool(), et_id)
        .await
        .map_err(read_error)?;
    if now.enabled != want.state.enabled {
        toggle(ctx, slug).await?;
    }
    if let Some(write) = want.price_write {
        if stored_price(ctx, et_id).await? != want.price {
            let mode_before = guest_mode(ctx, et_id).await;
            let mut conn = ctx.pool().acquire().await?;
            ton_pay::write_price(&mut conn, et_id, write, chrono::Utc::now()).await?;
            drop(conn);
            if record && !want.mode_given {
                let mode_after = guest_mode(ctx, et_id).await;
                ctx.record(at(slug, "telegram_guest_mode"), mode_before, mode_after);
            }
        }
    }
    if want.mode_given && guest_mode(ctx, et_id).await != want.mode {
        crate::telegram::store::save_event_type_mode(ctx.pool(), et_id, want.mode.key()).await?;
    }
    if want.overrides_given && stored_overrides(ctx, et_id, want.today).await? != want.overrides {
        write_overrides(ctx, et_id, want).await?;
    }
    if now.timezone != want.state.timezone {
        sqlx::query("UPDATE event_types SET timezone = ? WHERE id = ?")
            .bind(&want.state.timezone)
            .bind(et_id)
            .execute(ctx.pool())
            .await?;
    }
    Ok(())
}

/// The date overrides of `et_id` from `want.today` on replaced by the
/// desired ones, with the columns `create_override` writes. Earlier rows
/// stay as they are.
async fn write_overrides(ctx: &Ctx, et_id: &str, want: &Wanted) -> Result<(), ApplyError> {
    sqlx::query("DELETE FROM availability_overrides WHERE event_type_id = ? AND date >= ?")
        .bind(et_id)
        .bind(want.today.format("%Y-%m-%d").to_string())
        .execute(ctx.pool())
        .await?;
    for entry in &want.overrides {
        let blocked = entry.blocked == Some(true);
        sqlx::query(
            "INSERT INTO availability_overrides
                 (id, event_type_id, date, start_time, end_time, is_blocked)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(et_id)
        .bind(&entry.date)
        .bind(&entry.start)
        .bind(&entry.end)
        .bind(i64::from(blocked))
        .execute(ctx.pool())
        .await?;
    }
    Ok(())
}

/// The event type list's on/off switch for `slug`.
async fn toggle(ctx: &Ctx, slug: &str) -> Result<(), ApplyError> {
    let (headers, toggle) = form::<crate::web::event_types::ToggleForm>(&[]).await?;
    let response = crate::web::event_types::toggle_event_type(
        State(ctx.state.clone()),
        ctx.auth_user(),
        headers,
        Path(slug.to_string()),
        toggle,
    )
    .await
    .into_response();
    answered(&at(slug, "enabled"), response).await
}

/// Read back and compare with the document (pipeline step 4).
pub(super) async fn verify(ctx: &Ctx, section: &EventTypes) -> Result<(), ApplyError> {
    let profile = profile_rules(ctx).await?;
    let calendars = Calendars::load(ctx).await?;
    for item in &section.items {
        let slug = item.slug.as_str();
        let Some(et_id) = personal_id(ctx, slug).await? else {
            return Err(ApplyError::stored_differs(
                item_path(slug),
                &Value::from("present"),
                &Value::Null,
            ));
        };
        let now = Current::load(ctx, &et_id).await?;
        let item = &without_stored(item, &now, &calendars)?;
        let want = wanted(ctx, item, now.clone(), &profile, &calendars).await?;
        if let Some((key, stored, desired)) = differences(item, &now, &want, &calendars)?
            .into_iter()
            .next()
        {
            return Err(ApplyError::stored_differs(
                at(slug, &key),
                &desired,
                &stored,
            ));
        }
    }
    for slug in &ctx.unmanaged_event_types {
        let stored = personal_id(ctx, slug).await?;
        match (section.prune(), stored) {
            (Prune::None, _) => {}
            (Prune::Disable, Some(et_id)) => {
                if EditState::load(ctx.pool(), &et_id)
                    .await
                    .map_err(read_error)?
                    .enabled
                {
                    return Err(ApplyError::stored_differs(
                        at(slug, "enabled"),
                        &Value::Bool(false),
                        &Value::Bool(true),
                    ));
                }
            }
            (Prune::Disable, None) => {
                return Err(ApplyError::stored_differs(
                    item_path(slug),
                    &Value::from("present"),
                    &Value::Null,
                ))
            }
            (Prune::Delete, Some(_)) => {
                return Err(ApplyError::stored_differs(
                    item_path(slug),
                    &Value::Null,
                    &Value::from("present"),
                ))
            }
            (Prune::Delete, None) => {}
        }
    }
    Ok(())
}

/// The stored state in the document's shape (`--export`), reading only:
/// every personal event type of the admin, in the order they were made.
/// `weekly_hours` is null where the hours are the profile's default ones,
/// and `prune` is left out (`none`). A key is left out where the document
/// cannot say what is stored: `busy_calendars` naming a calendar whose name
/// another one shares (or one no source lists), `overrides` with a row the
/// document could not give (a hand edit).
pub(super) async fn export(ctx: &Ctx) -> Result<Option<EventTypes>, ApplyError> {
    let profile = profile_rules(ctx).await?;
    let calendars = Calendars::load(ctx).await?;
    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT et.id FROM event_types et
         JOIN accounts a ON a.id = et.account_id
         WHERE a.user_id = ? AND et.team_id IS NULL
         ORDER BY et.created_at, et.slug, et.id",
    )
    .bind(&ctx.me.id)
    .fetch_all(ctx.pool())
    .await?;
    let mut items = Vec::with_capacity(ids.len());
    for et_id in ids {
        let current = Current::load(ctx, &et_id).await?;
        let mut item = current.item(&calendars);
        if current.state.weekly_hours == profile {
            item.weekly_hours = Some(None);
        }
        if !calendars.nameable(&current.state.calendar_ids) {
            item.busy_calendars = None;
        }
        let canonical = current.overrides.iter().enumerate().all(|(i, entry)| {
            override_wanted(&format!("overrides[{i}]"), entry, current.today)
                .is_ok_and(|kept| kept.is_some())
        });
        let unique = current.overrides.windows(2).all(|pair| pair[0] != pair[1]);
        if !canonical || !unique {
            item.overrides = None;
        }
        items.push(item);
    }
    Ok(Some(EventTypes { prune: None, items }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every keyword of the document reaches the MCP enum of its spelling.
    #[test]
    fn keywords_convert_to_the_mcp_enums() {
        for v in [
            Visibility::Public,
            Visibility::Internal,
            Visibility::Private,
        ] {
            assert_eq!(mcp::<dto::Visibility>(v.key()).unwrap().key(), v.key());
        }
        for v in [
            LocationType::Link,
            LocationType::Phone,
            LocationType::InPerson,
            LocationType::Custom,
            LocationType::JitsiAuto,
            LocationType::WebhookAuto,
            LocationType::GoogleMeet,
        ] {
            assert_eq!(mcp::<dto::LocationType>(v.key()).unwrap().key(), v.key());
        }
        for v in [
            CalendarView::Month,
            CalendarView::Week,
            CalendarView::Column,
        ] {
            assert_eq!(mcp::<dto::CalendarView>(v.key()).unwrap().key(), v.key());
        }
        for v in [PhoneMode::Off, PhoneMode::Optional, PhoneMode::Required] {
            assert_eq!(mcp::<dto::PhoneMode>(v.key()).unwrap().key(), v.key());
        }
    }

    #[test]
    fn tool_errors_keep_the_field_under_the_item() {
        let e = tool_error(
            "15min",
            ToolError::invalid("patch.duration_min", "must be 1 to 1440"),
        );
        assert_eq!(
            e.to_string(),
            "invalid event_types[15min].duration_min: must be 1 to 1440"
        );
        let e = tool_error(
            "15min",
            ToolError::invalid("weekly_hours[0].windows", "windows must not overlap"),
        );
        assert_eq!(
            e.to_string(),
            "invalid event_types[15min].weekly_hours[0].windows: windows must not overlap"
        );
    }

    /// `KEYS` names every key of an item but the slug.
    #[test]
    fn keys_cover_every_item_key() {
        let me = crate::models::User {
            id: "u".into(),
            email: "a@example.com".into(),
            name: "A".into(),
            timezone: "UTC".into(),
            password_hash: None,
            role: "admin".into(),
            auth_provider: "local".into(),
            oidc_subject: None,
            enabled: true,
            created_at: String::new(),
            updated_at: String::new(),
            username: None,
            booking_email: None,
            title: None,
            bio: None,
            avatar_path: None,
            allow_dynamic_group: false,
            language: None,
            public_page_language: "visitor".into(),
        };
        let mut state = new_form("x", &me, &[(1, "09:00".into(), "17:00".into())]);
        state.title = "X".into();
        let map = as_map(&item_of(
            &state,
            GuestMode::Off,
            &None,
            &[],
            &Calendars(Vec::new()),
        ))
        .unwrap();
        let mut keys: Vec<&str> = map
            .keys()
            .map(String::as_str)
            .filter(|k| *k != "slug")
            .collect();
        let mut listed = KEYS.to_vec();
        keys.sort_unstable();
        listed.sort_unstable();
        assert_eq!(keys, listed);
    }

    #[test]
    fn rules_become_days_in_week_order() {
        let rules: Vec<Rule> = vec![
            (0, "10:00".into(), "12:00".into()),
            (1, "09:00".into(), "12:00".into()),
            (1, "13:00".into(), "17:00".into()),
        ];
        let days = days_of(&rules);
        assert_eq!(days.len(), 2);
        assert_eq!(days[0].day, Day::Mon);
        assert_eq!(days[0].windows.len(), 2);
        assert_eq!(days[1].day, Day::Sun);
    }

    #[test]
    fn limits_sort_by_period_then_count() {
        let limits = limits_of(&[
            (10, "year".into(), false),
            (2, "day".into(), false),
            (5, "week".into(), false),
            (1, "bogus".into(), false),
        ]);
        let periods: Vec<&str> = limits.iter().map(|l| l.period.key()).collect();
        assert_eq!(periods, ["day", "week", "year"]);
    }
}
