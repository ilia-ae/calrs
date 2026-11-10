//! ilia fork, the Optimize stage: an idle watcher with a price steps every
//! 15 minutes instead of every minute, and its heartbeat is made fresh on
//! demand (a priced page wakes it; a booking that finds it stale wakes it
//! and waits for the step). A child of the TON tests.

use super::perf::simulate;
use super::*;
use crate::ton::amount::NANO_PER_TON;
use crate::web::ton_pay::watcher::IDLE_PRICED;

async fn ton_env() -> Env {
    env_priced(&[], Some(100 * NANO_PER_TON)).await
}

/// One hour of each case, on the watcher's own delays: idle with a price
/// is 4 steps (was 60), an open invoice still polls every 10 seconds, no
/// price is hourly, and a TON price never asks a rate source.
#[tokio::test]
async fn an_idle_priced_watcher_steps_every_fifteen_minutes() {
    assert_eq!(IDLE_PRICED, std::time::Duration::from_secs(900));
    let e = ton_env().await;
    let idle = simulate(&e, e.now + Duration::minutes(1), 60).await;
    assert_eq!(idle.steps, 4, "{idle:?}");
    assert_eq!(idle.masterchain, 4);
    assert_eq!(idle.pages, 4, "one page per step for the one address");
    assert_eq!(idle.rates, 0);

    let e = ton_env().await;
    e.booking("guest@example.com").await;
    let open = simulate(&e, e.now + Duration::seconds(10), 60).await;
    assert_eq!(open.steps, 360, "{open:?}");
    assert_eq!(open.rates, 0);

    // A day after the last invoice: still every minute (late payments).
    let e = ton_env().await;
    e.booking("guest@example.com").await;
    sqlx::query("UPDATE ton_invoices SET status = 'expired', closed_at = ?")
        .bind(crate::web::ton_pay::stamp(e.now))
        .execute(&e.pool)
        .await
        .unwrap();
    let recent = simulate(&e, e.now + Duration::minutes(1), 60).await;
    assert_eq!(recent.steps, 60, "{recent:?}");

    // An enabled USD price: a step every rate-cache age, so the rate
    // stays warm for the slot page (12 steps, 12 rate fetches an hour; the
    // first of the hour is still cached from the setup's step).
    let e = env().await;
    let usd = simulate(&e, e.now + Duration::minutes(1), 60).await;
    assert_eq!(usd.steps, 12, "{usd:?}");
    assert_eq!(usd.rates, 11, "{usd:?}");

    let e = ton_env().await;
    sqlx::query("DELETE FROM ton_prices")
        .execute(&e.pool)
        .await
        .unwrap();
    let none = simulate(&e, e.now + Duration::minutes(1), 60).await;
    assert_eq!(none.steps, 1, "{none:?}");
}

/// A failing indexer keeps its back-off: page views of a priced type do
/// not wake the watcher into retrying it.
#[tokio::test]
async fn page_views_do_not_retry_a_failing_indexer() {
    let e = ton_env().await;
    let wake = || async {
        tokio::time::timeout(std::time::Duration::from_millis(100), e.rt.wake.notified())
            .await
            .is_ok()
    };
    let _ = wake().await;
    e.chain.edit(|c| {
        c.fail_next = vec![crate::ton::toncenter::IndexerError::Auth(401)];
    });
    let r = e.step(e.now + Duration::minutes(5)).await;
    assert!(r.error.is_some());
    e.rt.with_health(|h| h.last_ok = Some(Utc::now() - Duration::minutes(5)));
    for _ in 0..5 {
        e.get_page("/u/testuser/test-meeting").await;
        assert!(!wake().await, "a failing indexer is left to its back-off");
    }
    // Recovered: page views wake an idle watcher again.
    assert!(e.step(e.now + Duration::minutes(10)).await.error.is_none());
    e.rt.with_health(|h| h.last_ok = Some(Utc::now() - Duration::minutes(5)));
    e.get_page("/u/testuser/test-meeting").await;
    assert!(wake().await);
}

/// An idle, healthy watcher with a slow indexer (each call longer than a
/// booking would wait): the booking is taken at once, and the woken step
/// picks the invoice up when the indexer answers.
#[tokio::test]
async fn a_slow_indexer_does_not_refuse_a_booking() {
    let e = ton_env().await;
    let _ = tokio::time::timeout(std::time::Duration::from_millis(100), e.rt.wake.notified()).await;
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let handle = tokio::spawn(watcher::supervise(
        e.state.clone(),
        stop_rx,
        std::time::Duration::from_millis(5),
    ));
    let mut steps = e.rt.steps.subscribe();
    tokio::time::timeout(std::time::Duration::from_secs(5), steps.changed())
        .await
        .expect("a first step")
        .unwrap();
    e.chain
        .edit(|c| c.delay = Some(std::time::Duration::from_secs(30)));
    e.rt.with_health(|h| h.last_ok = Some(Utc::now() - Duration::minutes(10)));
    let t = std::time::Instant::now();
    let taken = invoice::prepare_invoice(
        &e.state,
        &e.et_id,
        e.now + Duration::days(3),
        0,
        "slow@example.com",
        "1.1.1.1",
        Utc::now(),
    )
    .await;
    assert!(matches!(taken, Ok(Some(_))), "{taken:?}");
    assert!(
        t.elapsed() < std::time::Duration::from_secs(1),
        "{:?}",
        t.elapsed()
    );
    stop_tx.send(true).unwrap();
    handle.abort();
}

/// The slot page and the form of a priced type wake a watcher whose last
/// step is over a minute old, and leave a fresh one alone.
#[tokio::test]
async fn a_priced_page_wakes_an_idle_watcher_only_when_stale() {
    let e = ton_env().await;
    let wake = || async {
        tokio::time::timeout(std::time::Duration::from_millis(100), e.rt.wake.notified())
            .await
            .is_ok()
    };
    let _ = wake().await;
    // The first step just ran: no wake.
    e.rt.with_health(|h| h.last_ok = Some(Utc::now()));
    let (status, _, _) = e.get_page("/u/testuser/test-meeting").await;
    assert!(status.is_success());
    assert!(!wake().await, "a fresh heartbeat needs no step");
    e.rt.with_health(|h| h.last_ok = Some(Utc::now() - Duration::minutes(5)));
    e.get_page("/u/testuser/test-meeting").await;
    assert!(wake().await, "the slot page woke the watcher");
    e.get_page(&format!(
        "/u/testuser/test-meeting/book?date={}&time=10:00",
        far_future_monday()
    ))
    .await;
    assert!(wake().await, "the form woke the watcher");
}

/// A booking that finds the heartbeat stale while the loop sleeps wakes it,
/// waits for the step and is taken; without a loop it is refused at once,
/// as before.
#[tokio::test]
async fn a_booking_wakes_an_idle_watcher_and_waits_for_its_step() {
    let e = ton_env().await;
    let start = e.now + Duration::days(3);
    // No loop (the other tests): refused at once, the wake left behind.
    e.rt.with_health(|h| h.last_ok = Some(Utc::now() - Duration::minutes(20)));
    let t = std::time::Instant::now();
    let refused =
        invoice::prepare_invoice(&e.state, &e.et_id, start, 0, "a@b.c", "1.1.1.1", Utc::now())
            .await;
    assert!(
        matches!(refused, Err(Refusal::Unavailable(_))),
        "{refused:?}"
    );
    assert!(t.elapsed() < std::time::Duration::from_secs(1));
    // Taken back, so the loop below sleeps until it is woken.
    let _ = tokio::time::timeout(std::time::Duration::from_millis(100), e.rt.wake.notified()).await;

    // The loop runs and sleeps its 15 minutes after a first step.
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let handle = tokio::spawn(watcher::supervise(
        e.state.clone(),
        stop_rx,
        std::time::Duration::from_millis(5),
    ));
    let mut steps = e.rt.steps.subscribe();
    tokio::time::timeout(std::time::Duration::from_secs(5), steps.changed())
        .await
        .expect("a first step")
        .unwrap();
    assert!(e.rt.looping.load(std::sync::atomic::Ordering::Acquire));
    // Idle and well: the owner is not warned (MCP), a booking still waits.
    e.rt.with_health(|h| h.last_ok = Some(Utc::now() - Duration::minutes(10)));
    assert!(e.rt.ready(Utc::now()).is_err());
    assert!(e.rt.ready_on_demand(Utc::now()).is_ok());
    let t = std::time::Instant::now();
    let taken =
        invoice::prepare_invoice(&e.state, &e.et_id, start, 0, "a@b.c", "1.1.1.1", Utc::now())
            .await;
    assert!(matches!(taken, Ok(Some(_))), "{taken:?}");
    assert!(t.elapsed() < crate::web::ton_pay::STEP_WAIT);
    // Taken at once; the woken step refreshes the heartbeat.
    tokio::time::timeout(std::time::Duration::from_secs(5), steps.changed())
        .await
        .expect("the woken step")
        .unwrap();
    assert!(e.rt.ready(Utc::now()).is_ok(), "the step refreshed it");
    // Too long idle, or a failing indexer: warned again.
    e.rt.with_health(|h| h.last_ok = Some(Utc::now() - Duration::minutes(40)));
    assert!(e.rt.ready_on_demand(Utc::now()).is_err());
    e.rt.with_health(|h| {
        h.last_ok = Some(Utc::now() - Duration::minutes(5));
        h.last_error = Some("http_502".into());
    });
    assert!(e.rt.ready_on_demand(Utc::now()).is_err());

    stop_tx.send(true).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), handle)
        .await
        .expect("the supervisor stops on the signal")
        .unwrap();
    assert!(!e.rt.looping.load(std::sync::atomic::Ordering::Acquire));
}

async fn plan(pool: &SqlitePool, sql: &str) -> String {
    let rows: Vec<(i64, i64, i64, String)> = sqlx::query_as(&format!("EXPLAIN QUERY PLAN {sql}"))
        .fetch_all(pool)
        .await
        .unwrap();
    rows.into_iter()
        .map(|r| r.3)
        .collect::<Vec<_>>()
        .join(" | ")
}

/// The watcher's per-step queries and the slot page's recurring busy query
/// use the ilia_007 indexes instead of walking every invoice or event
/// (`ton_perf::perf_explain_hot_paths` prints them all on seeded data).
#[tokio::test]
async fn the_hot_queries_use_the_fork_indexes() {
    let pool = setup_test_db().await;
    let p = plan(
        &pool,
        "SELECT DISTINCT address_raw FROM ton_invoices WHERE status IN ('open', 'paid') AND network = ? AND address_raw != ? ORDER BY address_raw",
    )
    .await;
    assert!(p.contains("idx_ton_invoices_status"), "{p}");
    let p = plan(
        &pool,
        "SELECT id FROM ton_invoices INDEXED BY idx_ton_invoices_paid_notice
         WHERE status = 'confirmed' AND paid_nano > 0 AND paid_notice_sent_at IS NULL
         ORDER BY created_at LIMIT 50",
    )
    .await;
    assert!(p.contains("idx_ton_invoices_paid_notice"), "{p}");
    let p = plan(
        &pool,
        "SELECT id, problem FROM ton_invoices WHERE problem IS NOT NULL ORDER BY created_at LIMIT 50",
    )
    .await;
    assert!(p.contains("idx_ton_invoices_problem"), "{p}");
    let p = plan(
        &pool,
        "SELECT e.start_at FROM events e JOIN calendars c ON c.id = e.calendar_id
         WHERE c.is_busy = 1 AND e.rrule IS NOT NULL AND e.rrule != '' AND (e.start_at <= ? OR e.start_at <= ?)",
    )
    .await;
    assert!(p.contains("idx_events_recurring"), "{p}");
    // The query the watcher really runs still names the index that exists.
    let e = env_priced_in(&[], Some(100 * NANO_PER_TON), pool).await;
    assert!(crate::web::ton_pay::store::due_received_notices(&e.pool)
        .await
        .unwrap()
        .is_empty());
}
