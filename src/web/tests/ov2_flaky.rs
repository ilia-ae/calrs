//! ilia fork: tests for the second overnight review round, group `flaky`.
//!
//! The log-capture helper `test_support::capture_default`: a thread-local
//! capture sees its own events even when another thread, with no
//! subscriber, hits the callsite first. Before the helper, a callsite first
//! registered by a sibling test while the capture's dispatcher was the only
//! live one was cached as `never` for the whole process, and
//! `posted_fields_cannot_forge_log_lines` and two agent audit tests failed
//! now and then under a parallel `cargo test`.

use std::sync::{Arc, Mutex};
use tracing_subscriber::layer::SubscriberExt;

/// The messages of the events with target `calrs::ov2_flaky` seen on the
/// capturing thread.
#[derive(Clone, Default)]
struct Seen(Arc<Mutex<Vec<String>>>);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Seen {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        struct Message(String);
        impl tracing::field::Visit for Message {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = format!("{value:?}");
                }
            }
        }
        if event.metadata().target() != "calrs::ov2_flaky" {
            return;
        }
        let mut message = Message(String::new());
        event.record(&mut message);
        self.0.lock().unwrap().push(message.0);
    }
}

impl Seen {
    fn all(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

/// Each probe is its own callsite, used by one test only, so its first hit
/// is the one the test arranges.
fn probe_first_hit_elsewhere(from: &str) {
    tracing::info!(target: "calrs::ov2_flaky", "first-hit-elsewhere from {from}");
}

fn probe_hit_before_capture(from: &str) {
    tracing::info!(target: "calrs::ov2_flaky", "hit-before-capture from {from}");
}

/// The race itself: the capture is live, another thread with no subscriber
/// registers the callsite first, then the capturing thread hits it.
#[test]
fn capture_sees_a_callsite_another_thread_registered_first() {
    let seen = Seen::default();
    let _guard =
        crate::test_support::capture_default(tracing_subscriber::registry().with(seen.clone()));
    std::thread::spawn(|| probe_first_hit_elsewhere("sibling"))
        .join()
        .unwrap();
    probe_first_hit_elsewhere("capture");
    assert_eq!(
        seen.all(),
        vec!["first-hit-elsewhere from capture".to_string()],
        "the capture misses its own event, or sees the other thread's"
    );
}

/// A callsite cached while no capture was live is re-checked when one starts.
#[test]
fn capture_sees_a_callsite_cached_before_it_began() {
    crate::test_support::pin_dispatch();
    std::thread::spawn(|| probe_hit_before_capture("sibling"))
        .join()
        .unwrap();
    let seen = Seen::default();
    let _guard =
        crate::test_support::capture_default(tracing_subscriber::registry().with(seen.clone()));
    probe_hit_before_capture("capture");
    assert_eq!(
        seen.all(),
        vec!["hit-before-capture from capture".to_string()]
    );
}

/// The pinned dispatcher is nobody's default: an event on a thread without a
/// capture reaches no capture, and a capture ends with its guard.
#[test]
fn capture_is_scoped_to_its_thread_and_guard() {
    fn probe_scope(from: &str) {
        tracing::info!(target: "calrs::ov2_flaky", "scope from {from}");
    }
    let seen = Seen::default();
    let guard =
        crate::test_support::capture_default(tracing_subscriber::registry().with(seen.clone()));
    probe_scope("inside");
    std::thread::spawn(|| probe_scope("other thread"))
        .join()
        .unwrap();
    drop(guard);
    probe_scope("after the guard");
    assert_eq!(seen.all(), vec!["scope from inside".to_string()]);
}
