//! ton-watcher: the background task of `calrs serve` that reads what arrived
//! at the payment address and moves invoices on (docs/ton.md, "Фоновая
//! задача ton-watcher").
//!
//! One step:
//! 1. the configured address has a cursor row (first run: the newest
//!    transaction, so old history is never replayed); the wallet status is
//!    checked at start and hourly;
//! 2. reconcile invoices with their bookings (database only);
//! 3. `masterchainInfo` BEFORE any page, and its `global_id` must be the
//!    configured network's, or nothing is credited or expired;
//! 4. for each watched address, pages from `cursor + 1` in ascending `lt`,
//!    each page one SQL transaction (payment rows, invoice credits and the
//!    cursor commit together); the cursor stops at the first transaction
//!    that is not final;
//! 5. paid invoices confirm their booking through the same handler as the
//!    host's Approve link (a busy host is final, any other failure is
//!    retried); open invoices whose host is no longer free (a request
//!    approved over them, a calendar event) are released before anyone
//!    pays for a time that is gone;
//! 6. open invoices expire only when their address was caught up in THIS
//!    step and the indexer's height is past `settle_by` plus a margin; the
//!    booking's deadline becomes now and calrs's sweep declines it;
//! 7. alerts, each claimed before it is sent.
//!
//! All state is in the database, so a restart continues from the cursor;
//! whether a payment was on time is judged by its own `now`, never by when
//! it was processed. Nothing here signs or sends anything to the chain.

use std::collections::HashSet;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use sqlx::SqlitePool;
use tokio::sync::watch;

use super::store::{self, Credit, NewPayment};
use super::{parse_stamp, stamp, AlertSink, TonAlert, TonRuntime, TARGET};
use crate::ton::amount;
use crate::ton::decide::{
    against_invoice, classify, clean_comment, parse_tx, sender_info, Class, IgnoreReason, Outcome,
    ParsedTx,
};
use crate::ton::toncenter::{expected_global_id, value_u128, value_u64, IndexerError, TxPage};
use crate::web::AppState;

/// Transactions per page.
pub(crate) const PAGE: u32 = 100;
/// Pages per address per step (within the 1 request/s budget).
pub(crate) const MAX_PAGES: usize = 30;
/// A non-final transaction blocking the cursor this long raises an alert;
/// so does a cursor that has not caught up for this long.
const STUCK_ALERT_MIN: i64 = 15;
/// Indexer failures for this long raise `indexer_down`.
const DOWN_ALERT_MIN: i64 = 15;
/// The wallet status is re-read this often.
const ACCOUNT_CHECK_SECS: i64 = 3600;
/// A paid invoice whose confirmation failed for another reason than a busy
/// host is retried this many times, then left to the owner.
pub(crate) const MAX_CONFIRM_ATTEMPTS: i64 = 5;
/// Payment alerts per invoice: one email an hour, the rest go into the
/// hourly digest.
const PAYMENT_ALERT_SPACING_MIN: i64 = 60;
/// Steps woken early are at least this far apart.
const MIN_SPACING: std::time::Duration = std::time::Duration::from_secs(2);
const SUPERVISOR_BACKOFF: std::time::Duration = std::time::Duration::from_secs(10);
const SUPERVISOR_BACKOFF_MAX: std::time::Duration = std::time::Duration::from_secs(300);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StepError {
    Indexer(IndexerError),
    WrongNetwork(i64),
    Db(String),
    /// The indexer returned a page out of `lt` order.
    Unordered,
}

impl StepError {
    pub(crate) fn class(&self) -> String {
        match self {
            Self::Indexer(e) => e.class(),
            Self::WrongNetwork(id) => format!("wrong_network_{id}"),
            Self::Db(_) => "database".into(),
            Self::Unordered => "unordered".into(),
        }
    }
}

impl From<sqlx::Error> for StepError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e.to_string())
    }
}

impl From<IndexerError> for StepError {
    fn from(e: IndexerError) -> Self {
        Self::Indexer(e)
    }
}

/// What one step did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct StepReport {
    pub(crate) error: Option<StepError>,
    /// Every watched address reached the indexer's end.
    pub(crate) caught_up: bool,
    pub(crate) credited: usize,
    pub(crate) confirmed: usize,
    pub(crate) expired: usize,
    /// Open invoices released because their host is no longer free.
    pub(crate) released: usize,
}

/// A health alert, at most once an hour per kind (claimed in `ton_alerts`).
/// Always logged; delivered when a sink is installed.
pub(crate) async fn raise(
    rt: &TonRuntime,
    pool: &SqlitePool,
    kind: &str,
    detail: &str,
    now: DateTime<Utc>,
) {
    raise_with(rt.alerts.get(), pool, kind, detail, now).await;
}

async fn raise_with(
    sink: Option<&Arc<dyn AlertSink>>,
    pool: &SqlitePool,
    kind: &str,
    detail: &str,
    now: DateTime<Utc>,
) {
    if !store::claim_global_alert(pool, kind, detail, now).await {
        return;
    }
    tracing::warn!(target: TARGET, kind, detail, "ton alert");
    if let Some(sink) = sink {
        sink.deliver(TonAlert::Health {
            kind: kind.to_string(),
            detail: detail.to_string(),
        })
        .await;
    }
}

// --- task, supervisor, shutdown ---------------------------------------------

/// The watcher task, as `calrs serve` holds it.
pub(crate) struct TonTask {
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl TonTask {
    /// Wait for the task to stop (after the stop signal), at most `grace`;
    /// then abort it. An open SQL transaction commits or rolls back whole.
    pub(crate) async fn finish(self, grace: std::time::Duration) {
        let Some(mut handle) = self.handle else {
            return;
        };
        if tokio::time::timeout(grace, &mut handle).await.is_err() {
            tracing::warn!(target: TARGET, "ton watcher still busy at shutdown, aborted");
            handle.abort();
        }
    }
}

/// Start the watcher beside the MCP agent. Payments off: a one-time check
/// for invoices left open (their hard deadlines free the slots), no polling.
pub(crate) fn spawn(state: Arc<AppState>, stop: watch::Receiver<bool>) -> TonTask {
    if let Some(rt) = state.ton.get() {
        // The owner hears about payments and problems by email; the
        // booking's Telegram chats (guest and hosts) when Telegram is on.
        let _ = rt.alerts.set(Arc::new(super::telegram::serve_sink(&state)));
    }
    let handle = tokio::spawn(supervise(state, stop, SUPERVISOR_BACKOFF));
    TonTask {
        handle: Some(handle),
    }
}

/// Resolves once the stop signal is set (or its sender is gone).
async fn stopped(stop: &mut watch::Receiver<bool>) {
    loop {
        if *stop.borrow() {
            return;
        }
        if stop.changed().await.is_err() {
            return;
        }
    }
}

/// Aborts the wrapped task when dropped, so aborting the supervisor also
/// stops the run it supervises.
struct AbortOnDrop(tokio::task::JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub(crate) async fn supervise(
    state: Arc<AppState>,
    mut stop: watch::Receiver<bool>,
    base_backoff: std::time::Duration,
) {
    let Some(rt) = state.ton.get() else {
        let reason = state.ton.disabled_reason().unwrap_or_default();
        tracing::info!(target: TARGET, %reason, "ton payments disabled");
        let open: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ton_invoices WHERE status IN ('open', 'paid')",
        )
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
        if open > 0 {
            tracing::error!(
                target: TARGET,
                open,
                "ton payments are disabled with invoices open: their bookings drop at their hard deadline"
            );
            raise_with(
                None,
                &state.pool,
                "disabled_with_open",
                &format!("open={open}"),
                Utc::now(),
            )
            .await;
        }
        return;
    };
    tracing::info!(
        target: TARGET,
        network = rt.cfg.network.as_str(),
        address = %rt.cfg.address_friendly,
        indexer_host = %rt.cfg.indexer_host(),
        "ton watcher: started"
    );
    let mut backoff = base_backoff;
    let mut crashes = 0u32;
    loop {
        let mut run_task = AbortOnDrop(tokio::spawn(run(state.clone(), rt.clone(), stop.clone())));
        match (&mut run_task.0).await {
            Ok(()) => return,
            Err(e) if e.is_cancelled() => return,
            Err(e) => {
                crashes += 1;
                tracing::error!(target: TARGET, error = %e, crashes, "ton watcher crashed, restarting");
                if crashes >= 3 {
                    raise(
                        &rt,
                        &state.pool,
                        "watcher_crashed",
                        &format!("crashes={crashes}"),
                        Utc::now(),
                    )
                    .await;
                }
                tokio::select! {
                    _ = stopped(&mut stop) => return,
                    _ = tokio::time::sleep(backoff) => {}
                }
                backoff = (backoff * 2).min(SUPERVISOR_BACKOFF_MAX);
            }
        }
    }
}

async fn run(state: Arc<AppState>, rt: Arc<TonRuntime>, mut stop: watch::Receiver<bool>) {
    loop {
        if *stop.borrow() {
            return;
        }
        let started = tokio::time::Instant::now();
        let report = step_until(&state, &rt, Utc::now(), &stop).await;
        let delay = next_delay(&state.pool, &rt, &report, Utc::now()).await;
        tokio::select! {
            _ = stopped(&mut stop) => return,
            _ = rt.wake.notified() => {
                // Woken by a new invoice: still keep steps apart.
                tokio::select! {
                    _ = stopped(&mut stop) => return,
                    _ = tokio::time::sleep_until(started + MIN_SPACING) => {}
                }
            }
            _ = tokio::time::sleep(delay) => {}
        }
    }
}

/// How long to wait before the next step.
pub(crate) async fn next_delay(
    pool: &SqlitePool,
    rt: &TonRuntime,
    report: &StepReport,
    now: DateTime<Utc>,
) -> std::time::Duration {
    use std::time::Duration as StdDuration;
    if let Some(err) = &report.error {
        return match err {
            StepError::Indexer(IndexerError::Auth(_)) | StepError::WrongNetwork(_) => {
                StdDuration::from_secs(300)
            }
            _ => {
                let failures = rt.health().failures.clamp(1, 8);
                StdDuration::from_secs((10u64 << (failures - 1)).min(120))
            }
        };
    }
    if !report.caught_up {
        // More pages wait: carry on (the request bucket paces it).
        return StdDuration::from_secs(1);
    }
    if store::active_invoices(pool).await {
        return StdDuration::from_secs(rt.cfg.poll_secs);
    }
    // While a price is enabled the heartbeat must stay fresh for new
    // bookings; a day after the last invoice, late payments are still
    // noticed every minute; after that, hourly.
    let recent: bool = sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM ton_invoices WHERE created_at >= ?)",
    )
    .bind(stamp(now - Duration::hours(24)))
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
    .unwrap_or(true);
    // Any price row, even switched off: switching it on (a form, the MCP
    // tool) wakes the watcher, and this bounds the wait if a wake is lost.
    if recent || store::any_price(pool).await {
        return StdDuration::from_secs(60);
    }
    StdDuration::from_secs(3600)
}

// --- one step ---------------------------------------------------------------

pub(crate) async fn step(state: &Arc<AppState>, rt: &TonRuntime, now: DateTime<Utc>) -> StepReport {
    let (_keep, never) = watch::channel(false);
    step_until(state, rt, now, &never).await
}

/// One step that stops early once `stop` is set: between pages and
/// addresses, and before confirming anything (the approval runs calendar
/// writes and emails that a shutdown abort must not cut in half; a paid
/// invoice is confirmed on the next start instead).
pub(crate) async fn step_until(
    state: &Arc<AppState>,
    rt: &TonRuntime,
    now: DateTime<Utc>,
    stop: &watch::Receiver<bool>,
) -> StepReport {
    let stopping = || *stop.borrow();
    let pool = &state.pool;
    let cfg = &rt.cfg;
    let mut report = StepReport::default();

    if let Err(e) = bootstrap(pool, rt, now).await {
        report.error = Some(e);
        finish_step(pool, rt, &mut report, None, now).await;
        return report;
    }
    check_account(pool, rt, now).await;
    if let Err(e) = reconcile(pool, now).await {
        tracing::error!(target: TARGET, error = %e, "ton reconcile failed");
    }
    if let Err(e) = store::purge_client_tags(pool, now).await {
        tracing::error!(target: TARGET, error = %e, "cannot forget old client networks");
    }

    // Masterchain info first: the height bounds what "caught up" means.
    let mc = match rt.indexer.masterchain_info().await {
        Ok(mc) => mc,
        Err(e) => {
            report.error = Some(e.into());
            if !stopping() {
                report.confirmed = confirm_paid(state, now).await;
            }
            deliver_alerts(pool, rt, now).await;
            finish_step(pool, rt, &mut report, None, now).await;
            return report;
        }
    };
    let expected = expected_global_id(cfg.network);
    if mc.global_id != expected {
        rt.with_health(|h| h.network_ok = Some(false));
        raise(
            rt,
            pool,
            "wrong_network",
            &format!("global_id={} expected={expected}", mc.global_id),
            now,
        )
        .await;
        report.error = Some(StepError::WrongNetwork(mc.global_id));
        finish_step(
            pool,
            rt,
            &mut report,
            Some((mc.gen_utime, mc.global_id)),
            now,
        )
        .await;
        return report;
    }
    rt.with_health(|h| h.network_ok = Some(true));

    let mut caught: HashSet<String> = HashSet::new();
    let addresses = match store::watched_addresses(pool, cfg.network, &cfg.address_raw).await {
        Ok(a) => a,
        Err(e) => {
            report.error = Some(e.into());
            vec![]
        }
    };
    report.caught_up = report.error.is_none();
    for addr in &addresses {
        if stopping() {
            report.caught_up = false;
            break;
        }
        match poll_address(pool, rt, addr, now, stop).await {
            Ok(poll) => {
                report.credited += poll.credited;
                if poll.caught_up {
                    caught.insert(addr.clone());
                } else {
                    report.caught_up = false;
                }
            }
            Err(e) => {
                report.caught_up = false;
                let fatal = matches!(e, StepError::Indexer(_) | StepError::Db(_));
                report.error = Some(e);
                if fatal {
                    break;
                }
            }
        }
    }

    if !stopping() {
        report.confirmed = confirm_paid(state, now).await;
        report.released = release_taken(state, rt, now).await;
    }
    report.expired = expire_open(pool, rt, &caught, mc.gen_utime, now).await;
    deliver_alerts(pool, rt, now).await;
    refresh_rate(pool, rt, now).await;
    finish_step(
        pool,
        rt,
        &mut report,
        Some((mc.gen_utime, mc.global_id)),
        now,
    )
    .await;
    report
}

/// The configured address gets its row on the first step; afterwards the
/// row only gets marked current (once per runtime).
async fn bootstrap(
    pool: &SqlitePool,
    rt: &TonRuntime,
    now: DateTime<Utc>,
) -> Result<(), StepError> {
    let cfg = &rt.cfg;
    if rt.bootstrapped.load(std::sync::atomic::Ordering::Acquire) {
        return Ok(());
    }
    let first = if store::has_watch_row(pool, cfg.network, &cfg.address_raw).await? {
        None
    } else {
        // History before the first start is never replayed.
        let latest = rt.indexer.latest_tx(&cfg.address_raw).await?;
        tracing::info!(
            target: TARGET,
            cursor_lt = latest.as_ref().map(|l| l.0).unwrap_or(0),
            "ton watcher: new address, cursor at its newest transaction"
        );
        Some(latest.unwrap_or((0, String::new())))
    };
    store::ensure_current(pool, cfg.network, &cfg.address_raw, first).await?;
    let elsewhere: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ton_invoices WHERE status IN ('open', 'paid') AND network != ?",
    )
    .bind(cfg.network.as_str())
    .fetch_one(pool)
    .await?;
    if elsewhere > 0 {
        raise(
            rt,
            pool,
            "other_network_open",
            &format!("open_on_other_network={elsewhere}"),
            now,
        )
        .await;
    }
    rt.bootstrapped
        .store(true, std::sync::atomic::Ordering::Release);
    Ok(())
}

async fn check_account(pool: &SqlitePool, rt: &TonRuntime, now: DateTime<Utc>) {
    let due = rt
        .health()
        .account_checked_at
        .is_none_or(|t| (now - t).num_seconds() >= ACCOUNT_CHECK_SECS);
    if !due {
        return;
    }
    let cfg = &rt.cfg;
    match rt.indexer.account_status(&cfg.address_raw).await {
        Ok(status) => {
            let status = status.unwrap_or_else(|| "nonexist".into());
            rt.with_health(|h| {
                h.account_status = Some(status.clone());
                h.account_checked_at = Some(now);
            });
            if let Err(e) =
                store::record_account(pool, cfg.network, &cfg.address_raw, &status, now).await
            {
                tracing::error!(target: TARGET, error = %e, "cannot store the wallet status");
            }
            if status != "active" {
                tracing::error!(
                    target: TARGET,
                    %status,
                    "the receiving wallet is not active: payments are off until it is"
                );
                raise(
                    rt,
                    pool,
                    "wallet_inactive",
                    &format!("status={status}"),
                    now,
                )
                .await;
            }
        }
        Err(e) => {
            tracing::warn!(target: TARGET, error = %e, "cannot read the wallet status");
        }
    }
}

async fn finish_step(
    pool: &SqlitePool,
    rt: &TonRuntime,
    report: &mut StepReport,
    mc: Option<(u64, i64)>,
    now: DateTime<Utc>,
) {
    let cfg = &rt.cfg;
    match &report.error {
        None => {
            rt.with_health(|h| {
                h.last_ok = Some(now);
                h.last_error = None;
                h.failures = 0;
                h.failing_since = None;
            });
            if let Err(e) =
                store::record_step(pool, cfg.network, &cfg.address_raw, now, mc, None).await
            {
                tracing::error!(target: TARGET, error = %e, "cannot store the watcher heartbeat");
            }
        }
        Some(err) => {
            let class = err.class();
            let mut since = now;
            rt.with_health(|h| {
                h.last_error = Some(class.clone());
                h.failures = h.failures.saturating_add(1);
                since = *h.failing_since.get_or_insert(now);
            });
            tracing::warn!(target: TARGET, error = %class, "ton watcher step failed");
            let _ = store::record_step(pool, cfg.network, &cfg.address_raw, now, mc, Some(&class))
                .await;
            if let StepError::Indexer(IndexerError::Auth(code)) = err {
                raise(rt, pool, "auth_failed", &format!("http_{code}"), now).await;
            } else if (now - since).num_minutes() >= DOWN_ALERT_MIN {
                raise(rt, pool, "indexer_down", &class, now).await;
            }
        }
    }
}

// --- reconcile --------------------------------------------------------------

/// Bring invoices in line with their bookings (docs/ton.md, the step-1
/// table). Each change is one guarded UPDATE.
pub(crate) async fn reconcile(pool: &SqlitePool, now: DateTime<Utc>) -> Result<(), sqlx::Error> {
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT i.id, i.status, b.status FROM ton_invoices i
         LEFT JOIN bookings b ON b.id = i.booking_id
         WHERE i.status IN ('open', 'paid')
            OR (i.status = 'confirmed' AND (b.id IS NULL OR b.status NOT IN ('confirmed', 'pending'))
                AND (i.problem IS NULL OR i.problem != 'refund_due'))",
    )
    .fetch_all(pool)
    .await?;
    let now_s = stamp(now);
    for (id, status, booking) in rows {
        let ended = !matches!(booking.as_deref(), Some("pending") | Some("confirmed"));
        match (status.as_str(), booking.as_deref()) {
            ("open", Some("pending")) => {}
            ("open", Some("confirmed")) => {
                // A guard was bypassed: say so, never call it paid.
                let n = sqlx::query(
                    "UPDATE ton_invoices SET status = 'closed', closed_at = ?,
                         problem = 'confirmed_unpaid'
                     WHERE id = ? AND status = 'open'",
                )
                .bind(&now_s)
                .bind(&id)
                .execute(pool)
                .await?;
                if n.rows_affected() == 1 {
                    tracing::error!(target: TARGET, invoice = %id, "an unpaid booking was confirmed");
                }
            }
            ("open", _) => {
                // Declined by its emailed link, cancelled from the CLI: a
                // part already paid has to go back.
                sqlx::query(
                    "UPDATE ton_invoices SET status = 'closed', closed_at = ?,
                         problem = CASE WHEN paid_nano > 0 THEN 'partial' ELSE problem END
                     WHERE id = ? AND status = 'open'",
                )
                .bind(&now_s)
                .bind(&id)
                .execute(pool)
                .await?;
            }
            ("paid", Some("confirmed")) => {
                // Confirmed by hand or by a reschedule after `host_busy`.
                sqlx::query(
                    "UPDATE ton_invoices SET status = 'confirmed', confirmed_at = COALESCE(confirmed_at, ?),
                         problem = CASE WHEN excess_nano > 0 THEN 'overpaid' ELSE NULL END
                     WHERE id = ? AND status = 'paid'",
                )
                .bind(&now_s)
                .bind(&id)
                .execute(pool)
                .await?;
            }
            ("paid", Some("pending")) => {}
            ("paid" | "confirmed", _) if ended => {
                sqlx::query(
                    "UPDATE ton_invoices SET problem = 'refund_due'
                     WHERE id = ? AND (problem IS NULL OR problem != 'refund_due')",
                )
                .bind(&id)
                .execute(pool)
                .await?;
            }
            _ => {}
        }
    }
    Ok(())
}

// --- polling ----------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AddressPoll {
    pub(crate) caught_up: bool,
    pub(crate) credited: usize,
}

/// Read one address forward from its cursor, up to [`MAX_PAGES`] pages.
pub(crate) async fn poll_address(
    pool: &SqlitePool,
    rt: &TonRuntime,
    addr: &str,
    now: DateTime<Utc>,
    halt: &watch::Receiver<bool>,
) -> Result<AddressPoll, StepError> {
    let network = rt.cfg.network;
    let mut out = AddressPoll::default();
    let Some(start) = store::watch_row(&mut *pool.acquire().await?, network, addr).await? else {
        // An invoice address without a cursor row (never created by calrs):
        // reading it from lt 0 is unbounded, so it is reported, not guessed.
        tracing::error!(target: TARGET, address = addr, "no cursor for a watched address");
        raise(rt, pool, "no_cursor", addr, now).await;
        return Ok(out);
    };
    let mut cursor = u64::try_from(start.cursor_lt).unwrap_or(0);
    for page_no in 0..MAX_PAGES {
        if *halt.borrow() {
            // Shutting down: the cursor is saved, the next start goes on.
            return Ok(out);
        }
        let page = rt
            .indexer
            .transactions_from(addr, cursor.saturating_add(1), PAGE)
            .await?;
        let mut tx = pool.begin().await?;
        store::lock_watch(&mut tx, network, addr).await?;
        // Re-read under the write lock: the cursor is the truth.
        let row = store::watch_row(&mut tx, network, addr)
            .await?
            .ok_or_else(|| StepError::Db("cursor row vanished".into()))?;
        let base = u64::try_from(row.cursor_lt).unwrap_or(0);
        let mut progress: Option<(u64, String)> = None;
        let mut stop: Option<u64> = None;
        let mut unordered = false;
        let mut prev = base;
        for v in &page.transactions {
            match parse_tx(v) {
                Ok(t) => {
                    if t.lt <= base {
                        continue; // row 1: a repeat
                    }
                    if t.lt <= prev {
                        unordered = true;
                        break;
                    }
                    let class = classify(&t, addr, prev);
                    if class == Class::Stop {
                        note_unfinal(&mut tx, rt, &t).await?;
                        stop = Some(t.lt);
                        break;
                    }
                    if record_tx(&mut tx, rt, &t, &page, &class, now).await? {
                        out.credited += 1;
                    }
                    prev = t.lt;
                    progress = Some((t.lt, t.hash_hex.clone()));
                }
                Err(issue) => match (issue.lt, issue.hash_hex.clone(), issue.final_) {
                    (Some(lt), _, _) if lt <= base => continue,
                    (Some(lt), Some(hash), true) if lt > prev => {
                        record_unparsed(&mut tx, rt, addr, v, lt, &hash, issue.what, now).await?;
                        prev = lt;
                        progress = Some((lt, hash));
                    }
                    _ => {
                        // Unreadable and not known final: wait on it.
                        stop = Some(issue.lt.unwrap_or(prev.saturating_add(1)));
                        break;
                    }
                },
            }
        }
        let stuck = match stop {
            Some(lt) => {
                let since = match (
                    row.stuck_lt,
                    row.stuck_since.as_deref().and_then(parse_stamp),
                ) {
                    (Some(s), Some(t)) if u64::try_from(s).ok() == Some(lt) => t,
                    _ => now,
                };
                Some((lt, since))
            }
            None => None,
        };
        let full = page.transactions.len() >= PAGE as usize;
        let caught_now = stop.is_none() && !unordered && !full;
        store::save_cursor(
            &mut tx,
            network,
            addr,
            progress.as_ref().map(|(lt, h)| (*lt, h.as_str())),
            stuck,
            caught_now.then_some(now),
        )
        .await?;
        tx.commit().await?;
        if let Some((lt, _)) = &progress {
            cursor = *lt;
        }
        if let Some((lt, since)) = stuck {
            if (now - since).num_minutes() >= STUCK_ALERT_MIN {
                raise(rt, pool, "stuck_finality", &format!("lt={lt}"), now).await;
            }
            return Ok(out);
        }
        if unordered {
            return Err(StepError::Unordered);
        }
        if caught_now {
            out.caught_up = true;
            return Ok(out);
        }
        if progress.is_none() {
            // A full page with nothing new: the indexer ignored start_lt.
            tracing::warn!(target: TARGET, address = addr, page = page_no, "indexer page made no progress");
            return Ok(out);
        }
    }
    // Still full after MAX_PAGES: lagging behind a flood of transactions.
    let behind_since = start
        .caught_up_at
        .as_deref()
        .and_then(parse_stamp)
        .unwrap_or(now);
    if (now - behind_since).num_minutes() >= STUCK_ALERT_MIN {
        raise(rt, pool, "lagging", addr, now).await;
    }
    Ok(out)
}

/// A payment that is visible but not final yet: remember its invoice so
/// the payment page can say "seen".
async fn note_unfinal(
    tx: &mut sqlx::SqliteConnection,
    rt: &TonRuntime,
    t: &ParsedTx,
) -> Result<(), sqlx::Error> {
    let reference = t
        .in_msg
        .as_ref()
        .and_then(|m| m.comment.as_deref())
        .and_then(crate::ton::reference::normalize);
    if let Some(reference) = reference {
        if let Some(inv) = store::invoice_by_reference(tx, &reference).await? {
            if inv.status == "open" {
                if let Ok(mut seen) = rt.seen.lock() {
                    // Bounded: an hour is far past any invoice's settle_by.
                    seen.retain(|_, at| at.elapsed() < std::time::Duration::from_secs(3600));
                    seen.insert(inv.id, std::time::Instant::now());
                }
            }
        }
    }
    Ok(())
}

/// Record one classified final transaction. Returns true when it credited
/// an invoice. Below the record threshold, transfers that match no invoice
/// are not stored (the cursor passes them all the same).
async fn record_tx(
    tx: &mut sqlx::SqliteConnection,
    rt: &TonRuntime,
    t: &ParsedTx,
    page: &TxPage,
    class: &Class,
    now: DateTime<Utc>,
) -> Result<bool, StepError> {
    let cfg = &rt.cfg;
    let value = t.value_nano();
    let (source_friendly, source_is_wallet) = sender_info(t, page);
    let comment = t
        .in_msg
        .as_ref()
        .and_then(|m| m.comment.as_deref())
        .map(clean_comment)
        .filter(|c| !c.is_empty());
    let Some(value_i64) = amount::to_i64(value) else {
        tracing::error!(target: TARGET, tx = %t.hash_hex, "transfer value out of range");
        return Ok(false);
    };
    let base = |outcome: &'static str| NewPayment {
        tx_hex: &t.hash_hex,
        network: cfg.network,
        address_raw: &t.account,
        lt: t.lt,
        utime: t.now,
        mc_block_seqno: t.mc_block_seqno.unwrap_or(0),
        source_raw: t.source(),
        source_friendly: source_friendly.as_deref(),
        source_is_wallet,
        value_nano: value_i64,
        comment: comment.clone(),
        invoice_id: None,
        outcome,
        reason: None,
        alert_due: true,
        seen_at: now,
    };
    match class {
        Class::SkipSilent | Class::Pass | Class::Stop => Ok(false),
        Class::Ignored(reason) => {
            if value < cfg.record_min_nano {
                tracing::debug!(target: TARGET, tx = %t.hash_hex, reason = reason.as_str(), "small transfer ignored");
                return Ok(false);
            }
            let p = NewPayment {
                reason: Some(reason.as_str()),
                ..base("ignored")
            };
            store::insert_payment(tx, &p).await?;
            Ok(false)
        }
        Class::Candidate { reference } => {
            let inv = match reference {
                Some(r) => store::invoice_by_reference(tx, r).await?,
                None => None,
            };
            let view = inv.as_ref().and_then(|i| i.view());
            let outcome = against_invoice(t, cfg.network, view.as_ref());
            let matched = view.is_some();
            if !matched && value < cfg.record_min_nano {
                tracing::debug!(target: TARGET, tx = %t.hash_hex, "small unmatched transfer");
                return Ok(false);
            }
            let invoice_id = match outcome {
                Outcome::Unmatched { .. } => None,
                _ => inv.as_ref().map(|i| i.id.as_str()),
            };
            let reason = match outcome {
                Outcome::Unmatched {
                    matched_elsewhere: true,
                } => Some("other_address"),
                Outcome::BeforeInvoice => Some(IgnoreReason::BeforeInvoice.as_str()),
                _ => None,
            };
            // A matched transfer below the record threshold (anyone who
            // knows a code can send dust with it) is kept, but emails no one.
            let worth_an_email = value >= cfg.record_min_nano;
            let p = NewPayment {
                invoice_id,
                reason,
                alert_due: outcome != Outcome::Credited && worth_an_email,
                ..base(outcome.as_str())
            };
            let inserted = store::insert_payment(tx, &p).await?;
            tracing::info!(
                target: TARGET,
                tx = %t.hash_hex,
                lt = t.lt,
                value_nano = value_i64,
                outcome = outcome.as_str(),
                invoice = invoice_id.unwrap_or(""),
                new = inserted == 1,
                "ton transfer"
            );
            if inserted != 1 || outcome != Outcome::Credited {
                return Ok(false);
            }
            let Some(inv) = inv else {
                return Ok(false);
            };
            match store::credit_invoice(tx, &inv.id, value_i64, now).await? {
                Credit::Partial if worth_an_email => {
                    store::mark_payment_alert(tx, &t.hash_hex).await?;
                }
                Credit::Partial => {}
                Credit::Paid { excess } => {
                    tracing::info!(target: TARGET, invoice = %inv.id, excess_nano = excess, "ton invoice paid");
                }
                Credit::NotOpen => {
                    tracing::error!(target: TARGET, invoice = %inv.id, "credit to an invoice that is not open");
                    store::mark_payment_alert(tx, &t.hash_hex).await?;
                }
            }
            Ok(true)
        }
    }
}

/// A final transaction toncenter answered in a shape we cannot read: kept
/// (with whatever value could be read) for the owner, the cursor passes.
#[allow(clippy::too_many_arguments)]
async fn record_unparsed(
    tx: &mut sqlx::SqliteConnection,
    rt: &TonRuntime,
    addr: &str,
    v: &Value,
    lt: u64,
    hash: &str,
    what: &str,
    now: DateTime<Utc>,
) -> Result<(), StepError> {
    let value = v
        .get("in_msg")
        .and_then(|m| m.get("value"))
        .and_then(value_u128)
        .and_then(amount::to_i64)
        .unwrap_or(0);
    tracing::warn!(target: TARGET, tx = %hash, lt, what, "ton transaction not readable, recorded");
    let p = NewPayment {
        tx_hex: hash,
        network: rt.cfg.network,
        address_raw: addr,
        lt,
        utime: v.get("now").and_then(value_u64).unwrap_or(0),
        mc_block_seqno: v.get("mc_block_seqno").and_then(value_u64).unwrap_or(0),
        source_raw: None,
        source_friendly: None,
        source_is_wallet: None,
        value_nano: value,
        comment: None,
        invoice_id: None,
        outcome: "ignored",
        reason: Some(IgnoreReason::Unparsed.as_str()),
        alert_due: true,
        seen_at: now,
    };
    store::insert_payment(tx, &p).await?;
    Ok(())
}

// --- confirm and expire -----------------------------------------------------

/// Paid invoices confirm their pending booking through the host's Approve
/// handler: calendar re-check, booking lock, resources, guarded flip,
/// calendar write, emails and SMS, all upstream's. Never inside a
/// transaction. Returns how many bookings got confirmed.
///
/// Only a refusal the handler marks as final ([`super::ApprovalBusy`]: the
/// host or a required resource is taken) makes the invoice `host_busy`.
/// Anything else that leaves the booking pending (a database error, a
/// calendar or resource read that failed) is retried on the next step, up
/// to [`MAX_CONFIRM_ATTEMPTS`] times, then left to the owner
/// (`confirm_failed`). Both hold the time until the meeting starts.
pub(crate) async fn confirm_paid(state: &Arc<AppState>, now: DateTime<Utc>) -> usize {
    let pool = &state.pool;
    #[allow(clippy::type_complexity)]
    let rows: Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        i64,
    )> = match sqlx::query_as(
        "SELECT i.id, i.booking_id, b.status, b.confirm_token,
                    CASE WHEN b.time_version = 1 THEN b.start_at END, i.confirm_attempts
             FROM ton_invoices i LEFT JOIN bookings b ON b.id = i.booking_id
             WHERE i.status = 'paid' AND (i.problem IS NULL OR i.problem = 'overpaid')",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!(target: TARGET, error = %e, "cannot list paid invoices");
            return 0;
        }
    };
    let mut confirmed = 0;
    for (invoice_id, booking_id, status, token, start_at, attempts) in rows {
        let mut status = status;
        let mut busy = false;
        if let (Some("pending"), Some(token)) = (status.as_deref(), token.as_deref()) {
            let response = crate::web::booking_actions::approve_booking_by_token(
                State(state.clone()),
                HeaderMap::new(),
                Path(token.to_string()),
            )
            .await
            .into_response();
            busy = response.extensions().get::<super::ApprovalBusy>().is_some();
            status = match sqlx::query_scalar("SELECT status FROM bookings WHERE id = ?")
                .bind(&booking_id)
                .fetch_optional(pool)
                .await
            {
                Ok(status) => status,
                Err(e) => {
                    // Unknown is not "gone": try again on the next step.
                    tracing::warn!(target: TARGET, error = %e, invoice = %invoice_id, "cannot re-read a booking after its approval");
                    continue;
                }
            };
        }
        let now_s = stamp(now);
        let hold_until_start = || async {
            if let Some(start) = &start_at {
                let _ = sqlx::query(
                    "UPDATE bookings SET pending_expires_at = ? WHERE id = ? AND status = 'pending'",
                )
                .bind(start)
                .bind(&booking_id)
                .execute(pool)
                .await;
            }
        };
        let res = match status.as_deref() {
            Some("confirmed") => {
                let r = sqlx::query(
                    "UPDATE ton_invoices SET status = 'confirmed', confirmed_at = ?
                     WHERE id = ? AND status = 'paid'",
                )
                .bind(&now_s)
                .bind(&invoice_id)
                .execute(pool)
                .await;
                if matches!(&r, Ok(d) if d.rows_affected() == 1) {
                    confirmed += 1;
                    tracing::info!(target: TARGET, invoice = %invoice_id, booking_id, "paid booking confirmed");
                }
                r.map(|_| ())
            }
            Some("pending") if busy => {
                // The host is no longer free: hold the slot until the start,
                // the owner decides (reschedule, confirm, or decline + refund).
                tracing::warn!(target: TARGET, invoice = %invoice_id, booking_id, "paid, but the host is busy");
                let r = sqlx::query(
                    "UPDATE ton_invoices SET problem = 'host_busy' WHERE id = ? AND status = 'paid'",
                )
                .bind(&invoice_id)
                .execute(pool)
                .await;
                hold_until_start().await;
                r.map(|_| ())
            }
            Some("pending") => {
                let attempts = attempts + 1;
                let give_up = attempts >= MAX_CONFIRM_ATTEMPTS;
                tracing::warn!(
                    target: TARGET,
                    invoice = %invoice_id,
                    booking_id,
                    attempts,
                    give_up,
                    "a paid booking could not be confirmed"
                );
                let r = sqlx::query(
                    "UPDATE ton_invoices SET confirm_attempts = ?,
                         problem = CASE WHEN ? THEN 'confirm_failed' ELSE problem END
                     WHERE id = ? AND status = 'paid'",
                )
                .bind(attempts)
                .bind(give_up)
                .bind(&invoice_id)
                .execute(pool)
                .await;
                if give_up {
                    hold_until_start().await;
                }
                r.map(|_| ())
            }
            _ => sqlx::query(
                "UPDATE ton_invoices SET problem = 'refund_due'
                 WHERE id = ? AND (problem IS NULL OR problem != 'refund_due')",
            )
            .bind(&invoice_id)
            .execute(pool)
            .await
            .map(|_| ()),
        };
        if let Err(e) = res {
            tracing::error!(target: TARGET, error = %e, invoice = %invoice_id, "cannot store a confirmation outcome");
        }
    }
    confirmed
}

/// Open invoices whose host is no longer free at their time (the host
/// approved an overlapping request, or put an event in the calendar):
/// released now, before the guest pays for a time that is gone. Read-only
/// checks, no calendar refresh and no lock; an invoice with a transfer in
/// flight, or one the check cannot read, is left alone. Returns how many
/// were released.
pub(crate) async fn release_taken(
    state: &Arc<AppState>,
    rt: &TonRuntime,
    now: DateTime<Utc>,
) -> usize {
    let pool = &state.pool;
    type Row = (String, String, String, String, String, String, String);
    let rows: Vec<Row> = match sqlx::query_as(
        "SELECT i.id, b.id, b.uid, a.user_id, b.event_type_id,
                CASE b.time_version WHEN 1 THEN b.start_at ELSE rtrim(b.start_at, 'Z') END,
                CASE b.time_version WHEN 1 THEN b.end_at ELSE rtrim(b.end_at, 'Z') END
         FROM ton_invoices i
         JOIN bookings b ON b.id = i.booking_id
         JOIN event_types et ON et.id = b.event_type_id
         JOIN accounts a ON a.id = et.account_id
         WHERE i.status = 'open' AND b.status = 'pending'",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!(target: TARGET, error = %e, "cannot list open invoices to re-check");
            return 0;
        }
    };
    let in_flight: HashSet<String> = rt
        .seen
        .lock()
        .map(|s| s.keys().cloned().collect())
        .unwrap_or_default();
    let mut released = 0;
    for (invoice_id, booking_id, uid, owner, et_id, start_at, end_at) in rows {
        if in_flight.contains(&invoice_id) {
            continue;
        }
        let hosts = match crate::web::try_reschedule_hosts(pool, &uid, &owner).await {
            Ok(h) => h,
            Err(_) => continue,
        };
        let free = crate::web::approval_hosts_free(
            pool,
            &hosts,
            &booking_id,
            &uid,
            &et_id,
            &start_at,
            &end_at,
        )
        .await;
        if !matches!(free, Ok(false)) {
            continue;
        }
        match release_one(pool, &invoice_id, &booking_id, now).await {
            Ok(true) => {
                released += 1;
                tracing::warn!(target: TARGET, invoice = %invoice_id, booking_id, "the host is no longer free: unpaid booking released");
            }
            Ok(false) => {}
            Err(e) => {
                tracing::error!(target: TARGET, error = %e, invoice = %invoice_id, "cannot release an invoice")
            }
        }
    }
    released
}

async fn release_one(
    pool: &SqlitePool,
    invoice_id: &str,
    booking_id: &str,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let n = sqlx::query(
        "UPDATE ton_invoices SET status = 'closed', closed_at = ?, problem = 'host_busy_unpaid'
         WHERE id = ? AND status = 'open'",
    )
    .bind(stamp(now))
    .bind(invoice_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n == 1 {
        // calrs's own sweep declines it within a minute.
        sqlx::query(
            "UPDATE bookings SET pending_expires_at = ? WHERE id = ? AND status = 'pending'",
        )
        .bind(stamp(now))
        .bind(booking_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(n == 1)
}

/// Open invoices whose address was caught up in this step and whose
/// `settle_by` plus the margin lies under the indexer's height: expired.
pub(crate) async fn expire_open(
    pool: &SqlitePool,
    rt: &TonRuntime,
    caught: &HashSet<String>,
    indexed_utime: u64,
    now: DateTime<Utc>,
) -> usize {
    let open = match store::invoices_with_status(pool, rt.cfg.network, "open").await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!(target: TARGET, error = %e, "cannot list open invoices");
            return 0;
        }
    };
    let margin = Duration::minutes(rt.cfg.expiry_margin_min);
    let height = i64::try_from(indexed_utime).unwrap_or(0);
    let mut expired = 0;
    for inv in open {
        if !caught.contains(&inv.address_raw) {
            continue;
        }
        let Some(settle_by) = parse_stamp(&inv.settle_by) else {
            continue;
        };
        if height < (settle_by + margin).timestamp() {
            continue;
        }
        match expire_one(pool, &inv, now).await {
            Ok(true) => {
                expired += 1;
                tracing::info!(
                    target: TARGET,
                    invoice = %inv.id,
                    booking_id = %inv.booking_id,
                    paid_nano = inv.paid_nano,
                    "ton invoice expired unpaid"
                );
            }
            Ok(false) => {}
            Err(e) => {
                tracing::error!(target: TARGET, error = %e, invoice = %inv.id, "cannot expire an invoice")
            }
        }
    }
    expired
}

async fn expire_one(
    pool: &SqlitePool,
    inv: &store::InvoiceRow,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let n = sqlx::query(
        "UPDATE ton_invoices SET status = 'expired', closed_at = ?,
             problem = CASE WHEN paid_nano > 0 THEN 'partial' ELSE problem END
         WHERE id = ? AND status = 'open'",
    )
    .bind(stamp(now))
    .bind(&inv.id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n == 1 {
        // calrs's own sweep declines it within a minute.
        sqlx::query(
            "UPDATE bookings SET pending_expires_at = ? WHERE id = ? AND status = 'pending'",
        )
        .bind(stamp(now))
        .bind(&inv.booking_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(n == 1)
}

/// Keep the rate cache warm while a price is enabled, so pages can show an
/// approximate amount without fetching on a GET. At most every 5 minutes
/// (the cache's age); a refusal is only logged here, a booking refuses on
/// its own.
async fn refresh_rate(pool: &SqlitePool, rt: &TonRuntime, now: DateTime<Utc>) {
    if rt.cached_rate().is_some() || !store::any_enabled_price(pool).await {
        return;
    }
    if let Err(refusal) = rt.rate(pool, now).await {
        tracing::warn!(target: TARGET, reason = %refusal, "no acceptable TON rate");
    }
}

// --- alerts -----------------------------------------------------------------

/// Payment and invoice alerts, claimed one by one before delivery. Without
/// a sink nothing is claimed: the rows wait for one. One payment email per
/// invoice an hour: a flood of transfers carrying a known code (each costs
/// its sender only the fee) is claimed into the hourly digest instead.
pub(crate) async fn deliver_alerts(pool: &SqlitePool, rt: &TonRuntime, now: DateTime<Utc>) {
    let Some(sink) = rt.alerts.get() else {
        return;
    };
    if let Ok(rows) = store::due_payment_alerts(pool).await {
        let since = now - Duration::minutes(PAYMENT_ALERT_SPACING_MIN);
        let mut held = 0usize;
        for (tx_hex, outcome, reason, invoice_id) in rows {
            let kind = match (outcome.as_str(), reason) {
                ("credited", _) => "partial".to_string(),
                ("ignored", Some(r)) => r,
                (o, _) => o.to_string(),
            };
            let recent = match &invoice_id {
                Some(id) => store::payment_alerted_since(pool, id, since).await,
                None => false,
            };
            if recent {
                if store::hold_payment_alert(pool, &tx_hex, now).await {
                    held += 1;
                }
                continue;
            }
            if store::claim_payment_alert(pool, &tx_hex, now).await {
                sink.deliver(TonAlert::Payment { tx_hex, kind }).await;
            }
        }
        if held > 0 {
            let total = store::held_payment_alerts_since(pool, since).await;
            raise_with(
                Some(sink),
                pool,
                "payment_alerts_held",
                &format!("transfers_without_email_last_hour={total}"),
                now,
            )
            .await;
        }
    }
    if let Ok(rows) = store::due_problem_alerts(pool).await {
        for (invoice_id, problem) in rows {
            if store::claim_problem_alert(pool, &invoice_id, &problem, now).await {
                sink.deliver(TonAlert::InvoiceProblem {
                    invoice_id,
                    problem,
                })
                .await;
            }
        }
    }
    if let Ok(ids) = store::due_received_notices(pool).await {
        for invoice_id in ids {
            if store::claim_received_notice(pool, &invoice_id, now).await {
                sink.deliver(TonAlert::PaymentReceived { invoice_id }).await;
            }
        }
    }
}
