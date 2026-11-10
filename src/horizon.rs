//! ilia fork: the fork-wide maximum horizon (owner's decision 2026-11-10,
//! "no planning more than +14 days, or a month"). Nothing is offered,
//! fetched from a calendar or kept in the event cache beyond it.
//!
//! `CALRS_MAX_HORIZON_DAYS`: default 14, allowed 1 to 31, read once. A
//! malformed or out-of-range value warns (without the value) and uses 14.
//!
//! - Booking: [`max_days_ahead`] caps every event type's last bookable day
//!   (`booking_window::effective_date_bounds`), whatever its own
//!   `booking_horizon_days` says.
//! - Sync: [`sync_window`] is the time range a calendar is read for, from
//!   [`LOOKBACK_DAYS`] back to the horizon plus [`MARGIN_DAYS`] ahead, and
//!   further when a live booking lies beyond (one booked before the limit was
//!   lowered still needs the busy times around it).
//!
//! In tests [`limit`] is `None` (upstream behaviour everywhere: no cap,
//! open-ended fetch, no pruning) unless a test sets [`test_limit`]: upstream's
//! tests book years ahead.

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use sqlx::SqlitePool;

/// The horizon when `CALRS_MAX_HORIZON_DAYS` is unset or invalid.
pub const DEFAULT_DAYS: i64 = 14;
/// The largest horizon allowed: "a month".
pub const MAX_DAYS: i64 = 31;
/// How far back a calendar is read: meetings in progress, "today" and
/// "yesterday" in the troubleshoot view, and up to 14 h of zone skew.
pub const LOOKBACK_DAYS: i64 = 2;
/// How far past the horizon a calendar is read: a day of zone skew, a day of
/// buffers at the edge, a day between two full fetches.
pub const MARGIN_DAYS: i64 = 3;
/// The sync window never reaches further than this, whatever a booking says.
const MAX_SYNC_AHEAD_DAYS: i64 = 366;

#[cfg(test)]
thread_local! {
    /// Tests only: the horizon for this thread. `#[tokio::test]` runs on one
    /// thread, so the override covers the whole test, mock servers included.
    static OVERRIDE: std::cell::Cell<Option<i64>> = const { std::cell::Cell::new(None) };
}

/// Parses `CALRS_MAX_HORIZON_DAYS`: the days, and whether to warn.
pub(crate) fn parse(value: Option<&str>) -> (i64, bool) {
    match value {
        None => (DEFAULT_DAYS, false),
        Some(v) => match v.trim().parse::<i64>() {
            Ok(days) if (1..=MAX_DAYS).contains(&days) => (days, false),
            _ => (DEFAULT_DAYS, true),
        },
    }
}

/// The configured horizon, read once per process.
#[cfg_attr(test, allow(dead_code))]
fn configured() -> i64 {
    static DAYS: std::sync::OnceLock<i64> = std::sync::OnceLock::new();
    *DAYS.get_or_init(|| {
        let raw = std::env::var("CALRS_MAX_HORIZON_DAYS").ok();
        let (days, warn) = parse(raw.as_deref());
        if warn {
            tracing::warn!("CALRS_MAX_HORIZON_DAYS must be 1-31; using {DEFAULT_DAYS}");
        }
        days
    })
}

/// The fork-wide horizon in days. Always `Some` in a real build; in tests
/// `None` unless [`test_limit`] set it.
pub fn limit() -> Option<i64> {
    #[cfg(test)]
    {
        OVERRIDE.with(std::cell::Cell::get)
    }
    #[cfg(not(test))]
    {
        Some(configured())
    }
}

/// How many days ahead anything may be booked: upstream's 366, capped by the
/// horizon.
pub fn max_days_ahead() -> i64 {
    let max = crate::booking_window::MAX_DAYS_AHEAD;
    limit().map_or(max, |days| days.min(max))
}

/// Tests only: sets this thread's horizon until the guard drops.
#[cfg(test)]
pub fn test_limit(days: i64) -> LimitGuard {
    LimitGuard(OVERRIDE.with(|c| c.replace(Some(days))))
}

#[cfg(test)]
pub struct LimitGuard(Option<i64>);

#[cfg(test)]
impl Drop for LimitGuard {
    fn drop(&mut self) {
        OVERRIDE.with(|c| c.set(self.0));
    }
}

/// The time range calendars are read for, `[since, until)` in UTC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncWindow {
    pub since: DateTime<Utc>,
    pub until: DateTime<Utc>,
}

impl SyncWindow {
    /// `YYYYMMDD` of `since`: compact (`YYYYMMDDTHHMMSS`) and all-day
    /// (`YYYYMMDD`) event times both sort against it.
    pub fn since_prefix(&self) -> String {
        self.since.format("%Y%m%d").to_string()
    }
    pub fn until_prefix(&self) -> String {
        self.until.format("%Y%m%d").to_string()
    }
    /// RFC 4791 time-range form, `YYYYMMDDTHHMMSSZ`.
    pub fn since_utc(&self) -> String {
        self.since.format("%Y%m%dT%H%M%SZ").to_string()
    }
    pub fn until_utc(&self) -> String {
        self.until.format("%Y%m%dT%H%M%SZ").to_string()
    }
    /// ISO form the bookings table sorts against.
    pub fn since_iso(&self) -> String {
        self.since.format("%Y-%m-%dT%H:%M:%S").to_string()
    }
    /// `(from, to)` `YYYYMMDD` prefixes of the cached one-off rows a full
    /// fetch's answer must hold if they still exist: those ending on or
    /// after `from` and starting before `to`, compared in compact form. A
    /// day inside the window on each side, since stored times are in the
    /// event's own zone while the window is in UTC: a row at the edge is
    /// never taken for deleted, and is pruned once outside.
    pub fn orphan_scope(&self) -> (String, String) {
        (
            (self.since + Duration::days(1))
                .format("%Y%m%d")
                .to_string(),
            (self.until - Duration::days(1))
                .format("%Y%m%d")
                .to_string(),
        )
    }
    /// One day short of `until`, so a booking at the edge whose event sits in
    /// another zone is never taken for an orphan.
    pub fn sweep_until_iso(&self) -> String {
        (self.until - Duration::days(1))
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string()
    }
}

/// The sync window at `now` for a horizon of `days`, reaching at least a day
/// past `latest_booking_end` (the end of the latest live booking).
pub fn window_for(
    now: DateTime<Utc>,
    days: i64,
    latest_booking_end: Option<DateTime<Utc>>,
) -> SyncWindow {
    let since = now - Duration::days(LOOKBACK_DAYS);
    let mut until = now + Duration::days(days + MARGIN_DAYS);
    if let Some(end) = latest_booking_end {
        until = until.max(end + Duration::days(1));
    }
    until = until.min(now + Duration::days(MAX_SYNC_AHEAD_DAYS));
    SyncWindow { since, until }
}

/// The end of the latest pending or confirmed booking, read as UTC (legacy
/// host-wall-clock rows are off by at most the zone offset, inside the day
/// [`window_for`] adds).
pub async fn latest_live_booking_end(pool: &SqlitePool) -> Option<DateTime<Utc>> {
    let end: Option<String> = sqlx::query_scalar(
        "SELECT MAX(end_at) FROM bookings WHERE status IN ('pending', 'confirmed')",
    )
    .fetch_one(pool)
    .await
    .ok()
    .flatten();
    let end = end?;
    NaiveDateTime::parse_from_str(end.get(..19)?, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|t| t.and_utc())
}

/// The sync window now, or `None` when there is no horizon (tests).
pub async fn sync_window(pool: &SqlitePool) -> Option<SyncWindow> {
    let days = limit()?;
    Some(window_for(
        Utc::now(),
        days,
        latest_live_booking_end(pool).await,
    ))
}

/// Whether a calendar whose cache is complete up to `synced_until`
/// (`YYYYMMDDTHHMMSSZ`) still covers the horizon plus a day, so an unchanged
/// calendar may be skipped. Without this, events sliding into the horizon
/// would never be read from a calendar whose ctag does not move.
pub fn covers_horizon(synced_until: Option<&str>, now: DateTime<Utc>) -> bool {
    let Some(days) = limit() else {
        return true;
    };
    let needed = (now + Duration::days(days + 1))
        .format("%Y%m%dT%H%M%SZ")
        .to_string();
    synced_until.is_some_and(|until| until >= needed.as_str())
}

/// Whether one VEVENT may matter inside the window with these `YYYYMMDD`
/// prefixes: recurring events always (their occurrences are expanded per
/// query), one-off events when they end on or after `since_prefix` and start
/// before `until_prefix`, compared the way the cache prune compares stored
/// times. An event whose times cannot be read is kept.
pub fn vevent_in_window(vevent: &str, since_prefix: &str, until_prefix: &str) -> bool {
    let fields = crate::utils::unfold_ical(vevent);
    if crate::utils::extract_vevent_field(&fields, "RRULE").is_some_and(|r| !r.is_empty()) {
        return true;
    }
    let Some(start) = crate::utils::extract_vevent_field(&fields, "DTSTART") else {
        return true;
    };
    let end = crate::utils::extract_vevent_end(&fields, None).unwrap_or_else(|| start.clone());
    let (start, end) = (compact(&start), compact(&end));
    end.as_str() >= since_prefix && start.as_str() < until_prefix
}

/// An iCal time in the compact form the cache stores (`2026-11-10` → `20261110`).
fn compact(value: &str) -> String {
    value.replace(['-', ':'], "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_table() {
        assert_eq!(parse(None), (14, false));
        assert_eq!(parse(Some("1")), (1, false));
        assert_eq!(parse(Some("14")), (14, false));
        assert_eq!(parse(Some("31")), (31, false));
        assert_eq!(parse(Some(" 7 ")), (7, false));
        for bad in ["0", "32", "-1", "abc", "", "7d", "1e1"] {
            assert_eq!(parse(Some(bad)), (14, true), "{bad:?}");
        }
    }

    #[test]
    fn default_is_fourteen_days() {
        assert_eq!(DEFAULT_DAYS, 14);
        assert_eq!(parse(None).0, 14);
    }

    #[test]
    fn tests_have_no_limit_unless_asked() {
        assert_eq!(limit(), None);
        assert_eq!(max_days_ahead(), crate::booking_window::MAX_DAYS_AHEAD);
        {
            let _g = test_limit(14);
            assert_eq!(limit(), Some(14));
            assert_eq!(max_days_ahead(), 14);
        }
        assert_eq!(limit(), None);
    }

    #[test]
    fn window_spans_lookback_to_horizon_plus_margin() {
        let now = "2026-11-10T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let w = window_for(now, 14, None);
        assert_eq!(w.since_utc(), "20261108T120000Z");
        assert_eq!(w.until_utc(), "20261127T120000Z");
        assert_eq!(w.since_prefix(), "20261108");
        assert_eq!(w.until_prefix(), "20261127");
        assert_eq!(w.since_iso(), "2026-11-08T12:00:00");
        assert_eq!(w.sweep_until_iso(), "2026-11-26T12:00:00");
    }

    #[test]
    fn window_reaches_past_a_far_live_booking_but_not_beyond_a_year() {
        let now = "2026-11-10T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let day20 = now + Duration::days(20);
        assert_eq!(
            window_for(now, 14, Some(day20)).until,
            day20 + Duration::days(1)
        );
        // An earlier booking never shrinks it.
        assert_eq!(
            window_for(now, 14, Some(now)).until,
            now + Duration::days(17)
        );
        let far = now + Duration::days(900);
        assert_eq!(
            window_for(now, 14, Some(far)).until,
            now + Duration::days(366)
        );
    }

    #[test]
    fn vevent_window_keeps_recurring_and_overlapping_events() {
        let ev = |body: &str| format!("BEGIN:VEVENT\r\nUID:x\r\n{body}END:VEVENT\r\n");
        let (since, until) = ("20261108", "20261127");
        let inside = ev("DTSTART:20261110T100000Z\r\nDTEND:20261110T110000Z\r\n");
        let past = ev("DTSTART:20261001T100000Z\r\nDTEND:20261001T110000Z\r\n");
        let far = ev("DTSTART:20261201T100000\r\nDTEND:20261201T110000\r\n");
        let edge = ev("DTSTART:20261127T000000\r\nDTEND:20261127T010000\r\n");
        let running = ev("DTSTART;VALUE=DATE:20261101\r\nDURATION:P10D\r\n");
        let ended_before = ev("DTSTART;VALUE=DATE:20261101\r\nDTEND;VALUE=DATE:20261107\r\n");
        let weekly =
            ev("DTSTART:20200101T100000\r\nDTEND:20200101T110000\r\nRRULE:FREQ=WEEKLY\r\n");
        assert!(vevent_in_window(&inside, since, until));
        assert!(!vevent_in_window(&past, since, until));
        assert!(!vevent_in_window(&far, since, until));
        assert!(!vevent_in_window(&edge, since, until));
        assert!(vevent_in_window(&running, since, until));
        assert!(!vevent_in_window(&ended_before, since, until));
        assert!(vevent_in_window(&weekly, since, until));
        assert!(vevent_in_window(
            "BEGIN:VEVENT\r\nUID:x\r\nEND:VEVENT\r\n",
            since,
            until
        ));
    }

    #[test]
    fn covers_horizon_needs_horizon_plus_a_day() {
        let now = "2026-11-10T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(covers_horizon(None, now), "no limit: always covered");
        let _g = test_limit(14);
        assert!(!covers_horizon(None, now));
        assert!(covers_horizon(Some("20261125T120000Z"), now));
        assert!(!covers_horizon(Some("20261125T115959Z"), now));
    }
}
