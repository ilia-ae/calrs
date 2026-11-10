//! The three times of an invoice (docs/ton.md, "Срок"):
//!
//! - `expires_at`: shown on the page and put in the link's `exp=`;
//!   `min(now + window, start − min_notice − grace − 5 min)`;
//! - `settle_by = expires_at + grace`: a transfer sent in the last minute
//!   still counts;
//! - `hard_deadline = min(settle_by + fuse, start)`: written to
//!   `bookings.pending_expires_at` at creation, so calrs's own sweep frees
//!   the slot even when the watcher never runs.
//!
//! A slot too close to leave a usable window (`< MIN_WINDOW`) is refused, so
//! `settle_by` always falls before the meeting starts.

use chrono::{DateTime, Duration, Utc};

/// The shortest payment window worth offering.
pub const MIN_WINDOW_MIN: i64 = 10;
/// Kept between `settle_by` and `start − min_notice`.
const SAFETY_MIN: i64 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvoiceTimes {
    pub expires_at: DateTime<Utc>,
    pub settle_by: DateTime<Utc>,
    pub hard_deadline: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooLate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowRules {
    pub window_min: i64,
    pub grace_min: i64,
    pub fuse_hours: i64,
}

/// How much later than its own minimum notice a priced slot must start to
/// leave a payment window ([`compute`] refuses anything sooner). The slot
/// pages add it to the notice, so they never offer a time that the booking
/// would then refuse.
pub fn min_lead_min(grace_min: i64) -> i64 {
    grace_min.max(0) + SAFETY_MIN + MIN_WINDOW_MIN
}

pub fn compute(
    now: DateTime<Utc>,
    start: DateTime<Utc>,
    min_notice_min: i64,
    rules: WindowRules,
) -> Result<InvoiceTimes, TooLate> {
    let grace = Duration::minutes(rules.grace_min.max(0));
    let latest_expiry =
        start - Duration::minutes(min_notice_min.max(0)) - grace - Duration::minutes(SAFETY_MIN);
    let expires_at = (now + Duration::minutes(rules.window_min.max(0))).min(latest_expiry);
    if expires_at - now < Duration::minutes(MIN_WINDOW_MIN) {
        return Err(TooLate);
    }
    let settle_by = expires_at + grace;
    let hard_deadline = (settle_by + Duration::hours(rules.fuse_hours.max(0))).min(start);
    Ok(InvoiceTimes {
        expires_at,
        settle_by,
        hard_deadline,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: WindowRules = WindowRules {
        window_min: 30,
        grace_min: 15,
        fuse_hours: 6,
    };

    fn t0() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-17T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn a_meeting_days_ahead_gets_the_full_window() {
        let now = t0();
        let start = now + Duration::days(3);
        let t = compute(now, start, 60, RULES).unwrap();
        assert_eq!(t.expires_at, now + Duration::minutes(30));
        assert_eq!(t.settle_by, now + Duration::minutes(45));
        assert_eq!(
            t.hard_deadline,
            now + Duration::minutes(45) + Duration::hours(6)
        );
    }

    #[test]
    fn a_close_meeting_caps_the_window_before_the_notice() {
        let now = t0();
        // start − 60 notice − 15 grace − 5 = now + 20 min.
        let start = now + Duration::minutes(100);
        let t = compute(now, start, 60, RULES).unwrap();
        assert_eq!(t.expires_at, now + Duration::minutes(20));
        assert_eq!(t.settle_by, now + Duration::minutes(35));
        assert!(t.settle_by < start - Duration::minutes(60));
        // The fuse is capped at the start.
        assert_eq!(t.hard_deadline, start);
    }

    #[test]
    fn the_lead_slot_pages_add_is_exactly_what_compute_needs() {
        let now = t0();
        let lead = min_lead_min(RULES.grace_min);
        assert_eq!(lead, 30);
        let first = now + Duration::minutes(60 + lead);
        assert!(compute(now, first, 60, RULES).is_ok());
        assert_eq!(
            compute(now, first - Duration::minutes(1), 60, RULES),
            Err(TooLate)
        );
    }

    #[test]
    fn too_little_time_is_refused() {
        let now = t0();
        // Leaves 9 minutes.
        let start = now + Duration::minutes(89);
        assert_eq!(compute(now, start, 60, RULES), Err(TooLate));
        // Exactly 10 is enough.
        assert!(compute(now, now + Duration::minutes(90), 60, RULES).is_ok());
        assert_eq!(
            compute(now, now - Duration::hours(1), 0, RULES),
            Err(TooLate)
        );
    }
}
