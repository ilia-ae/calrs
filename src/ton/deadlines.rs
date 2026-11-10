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

/// When an unpaid window may be renewed (ilia fork, docs/ton.md «Срок»).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenewalRules {
    /// The meeting must still be at least this far away.
    pub min_lead_hours: i64,
    /// A renewed window's length (`CALRS_TON_RENEW_WINDOW_MIN`), at most
    /// half the time left before the meeting and never shorter than the
    /// type's own window.
    pub window_min: i64,
}

/// The new times of an open, unpaid invoice whose window ended (`now ≥
/// expires_at`), or `None` to let it expire as before: when the renewals
/// allowed (`renewals_max`) are used up, when the meeting is nearer than
/// `min_lead_hours`, or when no full window fits before it. The new window
/// starts now and lasts `renew.window_min` (a guest asked again at a random
/// moment needs hours, not the first window's minutes), at most half the
/// time left before the meeting, at least the type's window, and capped by
/// [`compute`]'s latest expiry. `settle_by` only ever grows, so a transfer
/// sent in any earlier window (`utime ≤ old settle_by`) still counts.
#[allow(clippy::too_many_arguments)]
pub fn renewal(
    now: DateTime<Utc>,
    start: DateTime<Utc>,
    min_notice_min: i64,
    old: InvoiceTimes,
    renewals: i64,
    renewals_max: i64,
    renew: RenewalRules,
    rules: WindowRules,
) -> Option<InvoiceTimes> {
    if now < old.expires_at || renewals >= renewals_max {
        return None;
    }
    if start - now < Duration::hours(renew.min_lead_hours.max(0)) {
        return None;
    }
    let half_left = (start - now).num_minutes() / 2;
    let window_min = renew.window_min.min(half_left).max(rules.window_min.max(0));
    let new = compute(
        now,
        start,
        min_notice_min,
        WindowRules {
            window_min,
            ..rules
        },
    )
    .ok()?;
    (new.settle_by > old.settle_by && new.hard_deadline >= old.hard_deadline).then_some(new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renewal_decision_table() {
        let t = t0();
        let start = t + Duration::days(3);
        let old = compute(t, start, 0, RULES).unwrap();
        let renew = RenewalRules {
            min_lead_hours: 24,
            window_min: 360,
        };
        let at = old.expires_at;
        // Far, renewals left: a new, longer window from now.
        let new = renewal(at, start, 0, old, 0, 2, renew, RULES).unwrap();
        assert_eq!(new.expires_at, at + Duration::hours(6));
        assert_eq!(
            new.settle_by,
            at + Duration::hours(6) + Duration::minutes(15)
        );
        assert!(new.settle_by > old.settle_by);
        assert!(new.hard_deadline > old.hard_deadline);
        // Not yet at the end of the window.
        assert_eq!(
            renewal(at - Duration::seconds(1), start, 0, old, 0, 2, renew, RULES),
            None
        );
        // Used up, or off.
        assert_eq!(renewal(at, start, 0, old, 2, 2, renew, RULES), None);
        assert_eq!(renewal(at, start, 0, old, 0, 0, renew, RULES), None);
        // Near: under the threshold, measured from the meeting's start.
        let near = at + Duration::hours(23) + Duration::minutes(59);
        let old_near = compute(t, near, 0, RULES).unwrap();
        assert_eq!(renewal(at, near, 0, old_near, 0, 2, renew, RULES), None);
        let edge = at + Duration::hours(24);
        assert!(renewal(
            at,
            edge,
            0,
            compute(t, edge, 0, RULES).unwrap(),
            0,
            2,
            renew,
            RULES
        )
        .is_some());
        // No window fits before the meeting (TooLate): released.
        let short = RenewalRules {
            min_lead_hours: 0,
            window_min: 360,
        };
        let soon = at + Duration::minutes(25);
        let old_soon = InvoiceTimes {
            expires_at: at,
            settle_by: at + Duration::minutes(15),
            hard_deadline: soon,
        };
        assert_eq!(renewal(at, soon, 0, old_soon, 0, 2, short, RULES), None);
        // After an outage past the old settle_by: still renewed, later.
        let late = old.settle_by + Duration::minutes(5);
        let new = renewal(late, start, 0, old, 1, 2, renew, RULES).unwrap();
        assert!(new.settle_by > old.settle_by);
        // settle_by never shrinks: a shorter window that would end earlier
        // than the old settle_by is not a renewal.
        let shrink = WindowRules {
            window_min: 10,
            ..RULES
        };
        let wide = InvoiceTimes {
            expires_at: at,
            settle_by: at + Duration::minutes(60),
            hard_deadline: at + Duration::hours(7),
        };
        let short_renewal = RenewalRules {
            window_min: 10,
            ..renew
        };
        assert_eq!(
            renewal(at, start, 0, wide, 0, 2, short_renewal, shrink),
            None
        );
    }

    /// A renewed window lasts `renew.window_min`, at most half the time
    /// left before the meeting, at least the type's window, and its
    /// settle_by stays before the meeting's notice.
    #[test]
    fn a_renewed_window_is_long_but_bounded_by_the_meeting() {
        let t = t0();
        let renew = RenewalRules {
            min_lead_hours: 1,
            window_min: 360,
        };
        // Three days ahead: the full 6 hours.
        let start = t + Duration::days(3);
        let old = compute(t, start, 0, RULES).unwrap();
        let new = renewal(old.expires_at, start, 0, old, 0, 2, renew, RULES).unwrap();
        assert_eq!(new.expires_at - old.expires_at, Duration::hours(6));
        // Five hours ahead: half of what is left.
        let start = t + Duration::hours(5) + Duration::minutes(30);
        let old = compute(t, start, 60, RULES).unwrap();
        let at = old.expires_at;
        let new = renewal(at, start, 60, old, 0, 2, renew, RULES).unwrap();
        assert_eq!(new.expires_at - at, (start - at) / 2);
        assert!(new.settle_by < start - Duration::minutes(60));
        // A short renewal setting never shortens the type's window.
        let tiny = RenewalRules {
            window_min: 10,
            ..renew
        };
        let start = t + Duration::days(3);
        let old = compute(t, start, 0, RULES).unwrap();
        let new = renewal(old.expires_at, start, 0, old, 0, 2, tiny, RULES).unwrap();
        assert_eq!(new.expires_at - old.expires_at, Duration::minutes(30));
    }

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
