//! A sliding-window cap for the agent's own small limits.
//!
//! Upstream's `RateLimiter` (web/mod.rs) counts in a fixed window that
//! starts at the first hit and starts afresh once it has passed, so hits
//! bunched at the end of one window and the start of the next get nearly
//! twice its maximum within a minute. The caps on changes, guest emails and
//! full booking reads bound what a compromised session can do, so they
//! count the hits of the last window instead: at most `max` in any span of
//! that length.

use std::collections::VecDeque;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

/// At most `max` accepted hits in any `window`.
pub(crate) struct SlidingLimit {
    max: usize,
    window: Duration,
    /// When the accepted hits of the last `window` came, oldest first; never
    /// more than `max` of them.
    hits: Mutex<VecDeque<Instant>>,
}

impl SlidingLimit {
    pub(crate) fn new(max: u32, window: Duration) -> Self {
        Self {
            max: usize::try_from(max).unwrap_or(usize::MAX),
            window,
            hits: Mutex::new(VecDeque::new()),
        }
    }

    /// Whether a hit now would go over the cap. A hit that does not is
    /// counted; a refused one is not.
    pub(crate) fn check_limited(&self) -> bool {
        self.check_limited_at(Instant::now())
    }

    fn check_limited_at(&self, now: Instant) -> bool {
        let mut hits = self.hits.lock().unwrap_or_else(PoisonError::into_inner);
        while hits
            .front()
            .is_some_and(|hit| now.saturating_duration_since(*hit) >= self.window)
        {
            hits.pop_front();
        }
        if hits.len() >= self.max {
            return true;
        }
        hits.push_back(now);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: Duration = Duration::from_secs(60);

    #[test]
    fn hits_across_a_window_boundary_stay_within_the_cap() {
        let limit = SlidingLimit::new(2, MINUTE);
        let t0 = Instant::now();
        let at = |secs: u64| t0 + Duration::from_secs(secs);
        assert!(!limit.check_limited_at(at(0)));
        assert!(!limit.check_limited_at(at(59)));
        assert!(limit.check_limited_at(at(59)), "a third within a minute");
        // A fixed window from t0 would start afresh here and take two more.
        assert!(!limit.check_limited_at(at(60)), "the first hit has left");
        assert!(
            limit.check_limited_at(at(61)),
            "the hits at 59 and 60 are both within the last minute"
        );
        assert!(!limit.check_limited_at(at(119)), "the hit at 59 has left");
        assert!(limit.check_limited_at(at(119)));
    }

    /// The review's case: one decline, four more just before the hour is
    /// up, then five more just after. A fixed window took all ten, nine of
    /// them within a minute.
    #[test]
    fn five_an_hour_means_five_in_any_hour() {
        let limit = SlidingLimit::new(5, 60 * MINUTE);
        let t0 = Instant::now();
        let mut accepted = Vec::new();
        let mut hit = |at: Instant| {
            if !limit.check_limited_at(at) {
                accepted.push(at);
            }
        };
        hit(t0);
        for _ in 0..4 {
            hit(t0 + 59 * MINUTE);
        }
        for _ in 0..5 {
            hit(t0 + 60 * MINUTE + Duration::from_secs(1));
        }
        assert_eq!(accepted.len(), 6, "only the first hit's place came free");
        for start in &accepted {
            let within = accepted
                .iter()
                .filter(|at| *at >= start && at.duration_since(*start) < 60 * MINUTE)
                .count();
            assert!(within <= 5, "{within} hits within an hour");
        }
    }

    #[test]
    fn refused_hits_are_not_counted() {
        let limit = SlidingLimit::new(1, MINUTE);
        let t0 = Instant::now();
        assert!(!limit.check_limited_at(t0));
        for secs in 1..60 {
            assert!(limit.check_limited_at(t0 + Duration::from_secs(secs)));
        }
        // Refusals did not push the window on: the place is free a minute
        // after the accepted hit.
        assert!(!limit.check_limited_at(t0 + MINUTE));
    }
}
