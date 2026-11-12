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
//! Three settings are not on that form and are written the way their own
//! pages write them:
//! - `enabled` through the list's toggle (`toggle_event_type`), only when it
//!   differs;
//! - `telegram_guest_mode` with `telegram::store::save_event_type_mode`, the
//!   statement of Settings → Telegram;
//! - `timezone`: the form keeps a NULL timezone NULL when it posts back the
//!   one it follows, and pins one otherwise, so it can neither clear a
//!   timezone nor pin the followed one. After the form, a timezone that is
//!   still not the asked one is set with one `UPDATE`, as MCP's
//!   `set_weekly_hours` does.
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
    answered, check_opt_text, check_text, check_timezone, form, ApplyError, CalendarView, Ctx, Day,
    DayHours, EventTypeItem, EventTypes, FrequencyLimit, GuestMode, LocationType, Period,
    PhoneMode, Prune, Visibility, Window,
};
use crate::web::agent::dto;
use crate::web::agent::error::ToolError;
use crate::web::agent::ops_write::{self, EditState, Rule};
use crate::web::event_types::EventTypeForm;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
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
const KEYS: [&str; 30] = [
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
    "telegram_guest_mode",
    "enabled",
];

/// Settings not carried by the event type form (see the module comment).
const OFF_FORM: [&str; 3] = ["timezone", "enabled", "telegram_guest_mode"];

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

/// A form state and its Telegram mode as a document item, every key given.
fn item_of(state: &EditState, mode: GuestMode) -> EventTypeItem {
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

/// An item's desired state: the form state with the item applied, and the
/// Telegram mode.
struct Wanted {
    state: EditState,
    mode: GuestMode,
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

/// `base` with `item` applied, checked the way MCP checks an edit.
/// `profile` is what `weekly_hours: null` stands for.
async fn wanted(
    ctx: &Ctx,
    item: &EventTypeItem,
    base: EditState,
    base_mode: GuestMode,
    profile: &[Rule],
) -> Result<Wanted, ApplyError> {
    let slug = item.slug.as_str();
    let mut state = base;
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
    Ok(Wanted {
        state,
        mode: item.telegram_guest_mode.unwrap_or(base_mode),
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
    before: &EditState,
    before_mode: GuestMode,
    want: &Wanted,
) -> Result<Vec<(String, Value, Value)>, ApplyError> {
    let given = as_map(item)?;
    let stored = as_map(&item_of(before, before_mode))?;
    let desired = as_map(&item_of(&want.state, want.mode))?;
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
/// the checks would refuse today.
fn without_stored(
    item: &EventTypeItem,
    state: &EditState,
    mode: GuestMode,
) -> Result<EventTypeItem, ApplyError> {
    let stored = as_map(&item_of(state, mode))?;
    let mut given = as_map(item)?;
    given.retain(|key, value| key == "slug" || stored.get(key) != Some(value));
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
                let base = EditState::load(ctx.pool(), &et_id)
                    .await
                    .map_err(read_error)?;
                let mode = guest_mode(ctx, &et_id).await;
                let item = without_stored(item, &base, mode)?;
                check_item(&item)?;
                wanted(ctx, &item, base, mode, &profile).await?;
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
                let base = new_form(slug, &ctx.me, &profile);
                wanted(ctx, item, base, GuestMode::Off, &profile).await?;
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
    for item in &section.items {
        match personal_id(ctx, &item.slug).await? {
            Some(et_id) => update(ctx, item, &et_id, &profile).await?,
            None => create(ctx, item, &profile).await?,
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
async fn create(ctx: &mut Ctx, item: &EventTypeItem, profile: &[Rule]) -> Result<(), ApplyError> {
    let slug = item.slug.as_str();
    let path = item_path(slug);
    let base = new_form(slug, &ctx.me, profile);
    let want = wanted(ctx, item, base, GuestMode::Off, profile).await?;
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
    off_form(ctx, slug, &et_id, &want).await
}

/// An event type that exists: each differing setting recorded, the form
/// posted when one it carries differs.
async fn update(
    ctx: &mut Ctx,
    item: &EventTypeItem,
    et_id: &str,
    profile: &[Rule],
) -> Result<(), ApplyError> {
    let slug = item.slug.as_str();
    let before = EditState::load(ctx.pool(), et_id)
        .await
        .map_err(read_error)?;
    let before_mode = guest_mode(ctx, et_id).await;
    let item = &without_stored(item, &before, before_mode)?;
    let want = wanted(ctx, item, before.clone(), before_mode, profile).await?;
    let changes = differences(item, &before, before_mode, &want)?;
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
    off_form(ctx, slug, et_id, &want).await
}

/// The settings the form does not carry, each written only when it differs.
async fn off_form(ctx: &Ctx, slug: &str, et_id: &str, want: &Wanted) -> Result<(), ApplyError> {
    let now = EditState::load(ctx.pool(), et_id)
        .await
        .map_err(read_error)?;
    if now.enabled != want.state.enabled {
        toggle(ctx, slug).await?;
    }
    if guest_mode(ctx, et_id).await != want.mode {
        crate::telegram::store::save_event_type_mode(ctx.pool(), et_id, want.mode.key()).await?;
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
    for item in &section.items {
        let slug = item.slug.as_str();
        let Some(et_id) = personal_id(ctx, slug).await? else {
            return Err(ApplyError::stored_differs(
                item_path(slug),
                &Value::from("present"),
                &Value::Null,
            ));
        };
        let now = EditState::load(ctx.pool(), &et_id)
            .await
            .map_err(read_error)?;
        let mode = guest_mode(ctx, &et_id).await;
        let item = &without_stored(item, &now, mode)?;
        let want = wanted(ctx, item, now.clone(), mode, &profile).await?;
        if let Some((key, stored, desired)) =
            differences(item, &now, mode, &want)?.into_iter().next()
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
/// and `prune` is left out (`none`).
pub(super) async fn export(ctx: &Ctx) -> Result<Option<EventTypes>, ApplyError> {
    let profile = profile_rules(ctx).await?;
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
        let state = EditState::load(ctx.pool(), &et_id)
            .await
            .map_err(read_error)?;
        let mut item = item_of(&state, guest_mode(ctx, &et_id).await);
        if state.weekly_hours == profile {
            item.weekly_hours = Some(None);
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
        let map = as_map(&item_of(&state, GuestMode::Off)).unwrap();
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
