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
    compute_with(now, start, min_notice_min, Rules::site(rules))
}

/// The rules [`compute`] and [`renewal`] apply, in one shape: the site's
/// ([`Rules::site`]) or a debug booking's own ([`DebugTiming::rules`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rules {
    window_min: i64,
    grace: Duration,
    fuse: Duration,
    /// The shortest window offered ([`MIN_WINDOW_MIN`] for the site).
    min_window: Duration,
}

impl Rules {
    fn site(rules: WindowRules) -> Self {
        Self {
            window_min: rules.window_min.max(0),
            grace: Duration::minutes(rules.grace_min.max(0)),
            fuse: Duration::hours(rules.fuse_hours.max(0)),
            min_window: Duration::minutes(MIN_WINDOW_MIN),
        }
    }
}

fn compute_with(
    now: DateTime<Utc>,
    start: DateTime<Utc>,
    min_notice_min: i64,
    rules: Rules,
) -> Result<InvoiceTimes, TooLate> {
    let latest_expiry = start
        - Duration::minutes(min_notice_min.max(0))
        - rules.grace
        - Duration::minutes(SAFETY_MIN);
    let expires_at = (now + Duration::minutes(rules.window_min)).min(latest_expiry);
    if expires_at - now < rules.min_window {
        return Err(TooLate);
    }
    let settle_by = expires_at + rules.grace;
    let hard_deadline = (settle_by + rules.fuse).min(start);
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
    renewal_with(
        now,
        start,
        min_notice_min,
        old,
        (renewals, renewals_max),
        Duration::hours(renew.min_lead_hours.max(0)),
        renew.window_min,
        Rules::site(rules),
    )
}

#[allow(clippy::too_many_arguments)]
fn renewal_with(
    now: DateTime<Utc>,
    start: DateTime<Utc>,
    min_notice_min: i64,
    old: InvoiceTimes,
    (renewals, renewals_max): (i64, i64),
    min_lead: Duration,
    renew_window_min: i64,
    rules: Rules,
) -> Option<InvoiceTimes> {
    if now < old.expires_at || renewals >= renewals_max {
        return None;
    }
    if start - now < min_lead {
        return None;
    }
    let half_left = (start - now).num_minutes() / 2;
    let window_min = renew_window_min.min(half_left).max(rules.window_min);
    let new = compute_with(
        now,
        start,
        min_notice_min,
        Rules {
            window_min,
            ..rules
        },
    )
    .ok()?;
    (new.settle_by > old.settle_by && new.hard_deadline >= old.hard_deadline).then_some(new)
}

// --- the debug booking's own timing (ilia fork) ------------------------------

/// ilia fork: the timing of one test booking made on the admin's debug
/// booking page (`crate::web::debug_booking`), so the owner can watch a
/// whole unpaid cycle in minutes: the window ends, it is renewed (the guest
/// is asked to pay again), it ends again, the invoice expires and the
/// booking is released. It replaces, for that booking's invoice only, the
/// site's window, grace, fuse, renewal rules and expiry margin; it is set
/// only by that page (never by a guest's form) and stored on the invoice
/// (`ton_invoices.debug_timing`, [`DebugTiming::text`]).
///
/// It may go below the site's minimums, down to the bounds below, and
/// keeps every other rule of [`compute`]: the window still ends
/// [`SAFETY_MIN`] minutes plus the grace before the meeting's notice, the
/// hard deadline never passes the start, and a renewal never shortens
/// `settle_by` or the hard deadline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DebugTiming {
    /// The first payment window, minutes ([`DEBUG_WINDOW_MIN`] ..= 240).
    pub window_min: i64,
    /// A renewed window, minutes ([`DEBUG_WINDOW_MIN`] ..= 1440).
    pub renew_window_min: i64,
    /// Renewals before it expires (0 ..= 5).
    pub renewals_max: i64,
    /// A transfer sent this late still counts ([`DebugTiming::FAST`]: 1).
    pub grace_min: i64,
    /// The watcher expires it once the indexer is this far past
    /// `settle_by`, and sends a renewal's notice once it is this far past
    /// the renewal (the site's `CALRS_TON_EXPIRY_MARGIN_MIN`, at least 1).
    pub margin_min: i64,
    /// `hard_deadline = settle_by + fuse` (capped at the start): calrs's own
    /// sweep frees the time then even if the watcher stops.
    pub fuse_min: i64,
}

/// The shortest debug window (first or renewed): two minutes leave the
/// watcher (a step every `CALRS_TON_POLL_SECS`, 10 by default) a dozen
/// steps in the window and the guest time to send a transfer.
pub const DEBUG_WINDOW_MIN: i64 = 2;
/// The longest first window, as the site's (`CALRS_TON_WINDOW_MIN`).
pub const DEBUG_WINDOW_MAX: i64 = 240;
/// The longest renewed window, as the site's (`CALRS_TON_RENEW_WINDOW_MIN`).
pub const DEBUG_RENEW_WINDOW_MAX: i64 = 1440;
/// The most renewals, as the site's (`CALRS_TON_MAX_RENEWALS`).
pub const DEBUG_RENEWALS_MAX: i64 = 5;

impl DebugTiming {
    /// "Fast test": 3 minutes to pay, one renewal of 3 minutes, then the
    /// invoice expires; a minute of grace and of margin, a 10-minute fuse.
    /// Unpaid, the booking is released about 9 minutes after it was made.
    pub const FAST: Self = Self {
        window_min: 3,
        renew_window_min: 3,
        renewals_max: 1,
        grace_min: 1,
        margin_min: 1,
        fuse_min: 10,
    };

    /// The fast timing with the owner's own window, renewed window and
    /// number of renewals, each checked against its bounds (`None`: out of
    /// bounds).
    pub fn custom(window_min: i64, renew_window_min: i64, renewals_max: i64) -> Option<Self> {
        let t = Self {
            window_min,
            renew_window_min,
            renewals_max,
            ..Self::FAST
        };
        t.valid().then_some(t)
    }

    fn valid(&self) -> bool {
        (DEBUG_WINDOW_MIN..=DEBUG_WINDOW_MAX).contains(&self.window_min)
            && (DEBUG_WINDOW_MIN..=DEBUG_RENEW_WINDOW_MAX).contains(&self.renew_window_min)
            && (0..=DEBUG_RENEWALS_MAX).contains(&self.renewals_max)
            && (1..=60).contains(&self.grace_min)
            && (1..=30).contains(&self.margin_min)
            && (1..=48 * 60).contains(&self.fuse_min)
    }

    fn rules(&self) -> Rules {
        Rules {
            window_min: self.window_min,
            grace: Duration::minutes(self.grace_min),
            fuse: Duration::minutes(self.fuse_min),
            min_window: Duration::minutes(DEBUG_WINDOW_MIN),
        }
    }

    /// The first window's times, as [`compute`] with this timing.
    pub fn compute(
        &self,
        now: DateTime<Utc>,
        start: DateTime<Utc>,
        min_notice_min: i64,
    ) -> Result<InvoiceTimes, TooLate> {
        compute_with(now, start, min_notice_min, self.rules())
    }

    /// A renewal, as [`renewal`] with this timing: no minimum lead before
    /// the meeting (a window that fits before it is enough), the renewed
    /// window [`Self::renew_window_min`] (at most half the time left, at
    /// least the first window).
    pub fn renewal(
        &self,
        now: DateTime<Utc>,
        start: DateTime<Utc>,
        min_notice_min: i64,
        old: InvoiceTimes,
        renewals: i64,
    ) -> Option<InvoiceTimes> {
        renewal_with(
            now,
            start,
            min_notice_min,
            old,
            (renewals, self.renewals_max),
            Duration::zero(),
            self.renew_window_min,
            self.rules(),
        )
    }

    /// The stored form: `window=3,renew_window=3,renewals=1,grace=1,margin=1,fuse=10`.
    pub fn text(&self) -> String {
        format!(
            "window={},renew_window={},renewals={},grace={},margin={},fuse={}",
            self.window_min,
            self.renew_window_min,
            self.renewals_max,
            self.grace_min,
            self.margin_min,
            self.fuse_min
        )
    }

    /// [`Self::text`] read back; `None` for anything else (every key once,
    /// in bounds), which the watcher treats as the site's rules.
    pub fn parse(text: &str) -> Option<Self> {
        let mut t = Self::FAST;
        let mut seen = [false; 6];
        for part in text.trim().split(',') {
            let (k, v) = part.split_once('=')?;
            let v: i64 = v.trim().parse().ok()?;
            let (i, slot) = match k.trim() {
                "window" => (0, &mut t.window_min),
                "renew_window" => (1, &mut t.renew_window_min),
                "renewals" => (2, &mut t.renewals_max),
                "grace" => (3, &mut t.grace_min),
                "margin" => (4, &mut t.margin_min),
                "fuse" => (5, &mut t.fuse_min),
                _ => return None,
            };
            if std::mem::replace(&mut seen[i], true) {
                return None;
            }
            *slot = v;
        }
        (seen.iter().all(|s| *s) && t.valid()).then_some(t)
    }
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

    /// The fast debug timing: 3 minutes, one renewal of 3, then expired;
    /// below the site's minimums, never past the meeting's notice.
    #[test]
    fn the_fast_debug_timing_gives_minutes_not_hours() {
        let now = t0();
        let start = now + Duration::days(1);
        let fast = DebugTiming::FAST;
        let t = fast.compute(now, start, 60).unwrap();
        assert_eq!(t.expires_at, now + Duration::minutes(3));
        assert_eq!(t.settle_by, now + Duration::minutes(4));
        assert_eq!(t.hard_deadline, now + Duration::minutes(14));
        // The site would refuse a 3-minute window.
        let site = WindowRules {
            window_min: 3,
            ..RULES
        };
        assert_eq!(compute(now, start, 60, site), Err(TooLate));
        // Renewed once, at the end of the window, for 3 more minutes, and
        // no lead before the meeting is asked (the site's is 24 hours).
        let at = t.expires_at + Duration::seconds(5);
        assert_eq!(
            fast.renewal(at - Duration::seconds(6), start, 60, t, 0),
            None
        );
        let r = fast.renewal(at, start, 60, t, 0).unwrap();
        assert_eq!(r.expires_at, at + Duration::minutes(3));
        assert_eq!(r.settle_by, at + Duration::minutes(4));
        assert_eq!(r.hard_deadline, at + Duration::minutes(14));
        assert!(r.settle_by > t.settle_by && r.hard_deadline >= t.hard_deadline);
        // Then no more.
        assert_eq!(fast.renewal(r.expires_at, start, 60, r, 1), None);
        // The meeting's notice still bounds it: 60 notice + 1 grace + 5.
        let near = now + Duration::minutes(68);
        let t = fast.compute(now, near, 60).unwrap();
        assert_eq!(t.expires_at, now + Duration::minutes(2));
        assert!(t.settle_by < near - Duration::minutes(60));
        assert!(t.hard_deadline <= near);
        assert_eq!(
            fast.compute(now, near - Duration::minutes(1), 60),
            Err(TooLate)
        );
    }

    #[test]
    fn a_custom_debug_timing_is_bounded_and_round_trips() {
        let t = DebugTiming::custom(5, 7, 2).unwrap();
        assert_eq!(
            (t.window_min, t.renew_window_min, t.renewals_max),
            (5, 7, 2)
        );
        assert_eq!(
            (t.grace_min, t.margin_min, t.fuse_min),
            (1, 1, 10),
            "the rest is fast"
        );
        for (w, r, n) in [
            (1, 3, 1),
            (241, 3, 1),
            (3, 1, 1),
            (3, 1441, 1),
            (3, 3, 6),
            (3, 3, -1),
        ] {
            assert_eq!(DebugTiming::custom(w, r, n), None, "{w} {r} {n}");
        }
        assert!(DebugTiming::custom(2, 2, 0).is_some());
        assert!(DebugTiming::custom(240, 1440, 5).is_some());
        let text = DebugTiming::FAST.text();
        assert_eq!(
            text,
            "window=3,renew_window=3,renewals=1,grace=1,margin=1,fuse=10"
        );
        assert_eq!(DebugTiming::parse(&text), Some(DebugTiming::FAST));
        assert_eq!(DebugTiming::parse(&t.text()), Some(t));
        for bad in [
            "",
            "window=3",
            "window=3,renew_window=3,renewals=1,grace=1,margin=1,fuse=10,window=4",
            "window=1,renew_window=3,renewals=1,grace=1,margin=1,fuse=10",
            "window=3,renew_window=3,renewals=1,grace=0,margin=1,fuse=10",
            "window=3,renew_window=3,renewals=1,grace=1,margin=1,lead=0",
            "window=x,renew_window=3,renewals=1,grace=1,margin=1,fuse=10",
        ] {
            assert_eq!(DebugTiming::parse(bad), None, "{bad:?}");
        }
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
