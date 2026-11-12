//! ilia fork: TON payments, invoices and the ton-watcher against an
//! in-memory indexer (no network). Kept apart from upstream's test files.
//!
//! The chain is a `FakeChain` holding transactions derived from the recorded
//! mainnet answer (`ton::decide::tests::payment`); rates come from
//! `FakeRates`; alerts are captured by `CaptureSink`.

use super::*;
use crate::ton::config::TonConfig;
use crate::ton::rate::{RateError, RateQuote, RateSource, SourceKind};
use crate::ton::toncenter::{IndexerError, MasterchainInfo, TonIndexer, TxPage};
use crate::web::ton_pay::invoice::{self, OpenedInvoice, Refusal};
use crate::web::ton_pay::watcher::{self, StepError};
use crate::web::ton_pay::{AlertSink, TonAlert, TonRuntime};
use serde_json::{json, Value};
use std::collections::HashMap;

// ilia fork integration: the payment events in Telegram.
#[path = "ton_telegram.rs"]
mod telegram;
// Booking flow v2: TON prices, Telegram by default, renewals.
#[path = "ton_flow2.rs"]
mod flow2;
// The Optimize stage: measurements on a seeded database (ignored tests).
#[path = "ton_perf.rs"]
mod perf;
// The Optimize stage: the watcher's on-demand heartbeat.
#[path = "ton_optimize.rs"]
mod optimize;
// The price by demand (the ladder).
#[path = "ton_ladder.rs"]
mod ladder;
// The TON settings page, its hot apply and the alert email.
#[path = "ton_settings.rs"]
mod settings_page;
// The admin's debug booking page (CALRS_DEBUG_BOOKING).
#[path = "debug_booking.rs"]
mod debug_booking;

const OUR: &str = "0:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8";
const OTHER: &str = "0:077F69B02858C4A6BC33E5F052E64E5BAD13AB95DB61442CFE0958B97959BDE4";
const OTHER_UQ: &str = "UQAHf2mwKFjEprwz5fBS5k5brROrldthRCz-CVi5eVm95EuN";
const SPAMMER: &str = "0:ADDC9B4339BB6F09753FCCCE69E55D0BECA46575F8F02DF0DFCEDEBC71A4F08F";
/// $40 at the fake rate of $2.00 per TON.
const AMOUNT: u128 = 20_000_000_000;

// --- fakes -------------------------------------------------------------------

#[derive(Default)]
struct Chain {
    gen_utime: u64,
    global_id: i64,
    status: Option<String>,
    txs: Vec<Value>,
    fail_next: Vec<IndexerError>,
    panics_left: u32,
    pages_served: usize,
    /// masterchainInfo and accountStates calls (the Optimize stage counts
    /// requests per hour).
    mc_calls: usize,
    status_calls: usize,
    /// Each masterchainInfo call takes this long (a slow indexer).
    delay: Option<std::time::Duration>,
}

#[derive(Default)]
struct FakeChain(std::sync::Mutex<Chain>);

impl FakeChain {
    fn new(gen_utime: u64) -> Arc<Self> {
        let chain = FakeChain::default();
        chain.edit(|c| {
            c.gen_utime = gen_utime;
            c.global_id = -239;
            c.status = Some("active".into());
        });
        Arc::new(chain)
    }
    fn edit<R>(&self, f: impl FnOnce(&mut Chain) -> R) -> R {
        f(&mut self.0.lock().unwrap())
    }
    fn push(&self, tx: Value) {
        self.edit(|c| c.txs.push(tx));
    }
    fn height(&self, utime: u64) {
        self.edit(|c| c.gen_utime = utime);
    }
}

fn lt_of(v: &Value) -> u64 {
    crate::ton::toncenter::value_u64(&v["lt"]).unwrap_or(0)
}

#[async_trait::async_trait]
impl TonIndexer for FakeChain {
    async fn masterchain_info(&self) -> Result<MasterchainInfo, IndexerError> {
        // Panic outside the lock, so the mutex is not poisoned.
        let panic_now = self.edit(|c| {
            let p = c.panics_left > 0;
            c.panics_left = c.panics_left.saturating_sub(1);
            p
        });
        assert!(!panic_now, "fake indexer panic");
        if let Some(delay) = self.edit(|c| c.delay) {
            tokio::time::sleep(delay).await;
        }
        self.edit(|c| {
            c.mc_calls += 1;
            if let Some(e) = c.fail_next.pop() {
                return Err(e);
            }
            Ok(MasterchainInfo {
                seqno: 1,
                gen_utime: c.gen_utime,
                global_id: c.global_id,
            })
        })
    }
    async fn transactions_from(
        &self,
        account_raw: &str,
        start_lt: u64,
        limit: u32,
    ) -> Result<TxPage, IndexerError> {
        self.edit(|c| {
            c.pages_served += 1;
            let mut refs: Vec<&Value> = c
                .txs
                .iter()
                .filter(|t| t["account"].as_str() == Some(account_raw) && lt_of(t) >= start_lt)
                .collect();
            refs.sort_by_key(|t| lt_of(t));
            let txs: Vec<Value> = refs.into_iter().take(limit as usize).cloned().collect();
            let mut book = HashMap::new();
            book.insert(
                OTHER.to_string(),
                crate::ton::toncenter::BookEntry {
                    user_friendly: Some(OTHER_UQ.into()),
                    interfaces: vec!["wallet_v4r2".into()],
                },
            );
            Ok(TxPage {
                transactions: txs,
                address_book: book,
            })
        })
    }
    async fn latest_tx(&self, account_raw: &str) -> Result<Option<(u64, String)>, IndexerError> {
        Ok(self.edit(|c| {
            c.txs
                .iter()
                .filter(|t| t["account"].as_str() == Some(account_raw))
                .map(|t| (lt_of(t), "00".repeat(32)))
                .max_by_key(|(lt, _)| *lt)
        }))
    }
    async fn account_status(&self, _account_raw: &str) -> Result<Option<String>, IndexerError> {
        Ok(self.edit(|c| {
            c.status_calls += 1;
            c.status.clone()
        }))
    }
}

struct FakeRates(
    std::sync::Mutex<Vec<(SourceKind, u64)>>,
    /// Calls made: a TON price must make none.
    std::sync::atomic::AtomicUsize,
);

impl FakeRates {
    fn new(quotes: Vec<(SourceKind, u64)>) -> Self {
        Self(
            std::sync::Mutex::new(quotes),
            std::sync::atomic::AtomicUsize::new(0),
        )
    }
    fn calls(&self) -> usize {
        self.1.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl RateSource for FakeRates {
    async fn quotes(&self) -> Vec<Result<RateQuote, RateError>> {
        self.1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let now = Utc::now();
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|(source, usd_micro)| {
                Ok(RateQuote {
                    source: *source,
                    usd_micro: *usd_micro,
                    at: now,
                })
            })
            .collect()
    }
}

#[derive(Default)]
struct CaptureSink(std::sync::Mutex<Vec<TonAlert>>);

#[async_trait::async_trait]
impl AlertSink for CaptureSink {
    async fn deliver(&self, alert: TonAlert) {
        self.0.lock().unwrap().push(alert);
    }
}

impl CaptureSink {
    fn all(&self) -> Vec<TonAlert> {
        self.0.lock().unwrap().clone()
    }
    fn health_kinds(&self) -> Vec<String> {
        self.all()
            .into_iter()
            .filter_map(|a| match a {
                TonAlert::Health { kind, .. } => Some(kind),
                _ => None,
            })
            .collect()
    }
    fn payment_kinds(&self) -> Vec<String> {
        self.all()
            .into_iter()
            .filter_map(|a| match a {
                TonAlert::Payment { kind, .. } => Some(kind),
                _ => None,
            })
            .collect()
    }
    fn problems(&self) -> Vec<String> {
        self.all()
            .into_iter()
            .filter_map(|a| match a {
                TonAlert::InvoiceProblem { problem, .. } => Some(problem),
                _ => None,
            })
            .collect()
    }
    fn received(&self) -> usize {
        self.all()
            .iter()
            .filter(|a| matches!(a, TonAlert::PaymentReceived { .. }))
            .count()
    }
}

// --- environment -------------------------------------------------------------

struct Env {
    pool: SqlitePool,
    state: Arc<AppState>,
    rt: Arc<TonRuntime>,
    chain: Arc<FakeChain>,
    rates: Arc<FakeRates>,
    sink: Arc<CaptureSink>,
    et_id: String,
    account_id: String,
    now: chrono::DateTime<Utc>,
    next_lt: std::sync::atomic::AtomicU64,
    next_slot: std::sync::atomic::AtomicI64,
    _dir: TestDataDir,
}

fn vars(extra: &[(&'static str, &str)]) -> HashMap<&'static str, String> {
    let mut v = crate::ton::config::tests::base();
    // Renewals off unless a test asks for them: the expiry tests keep the
    // first deadline (renewals: tests below).
    v.insert("CALRS_TON_MAX_RENEWALS", "0".to_string());
    for (k, val) in extra {
        v.insert(*k, val.to_string());
    }
    v
}

fn runtime(
    vars: &HashMap<&'static str, String>,
    chain: Arc<FakeChain>,
    rates: Arc<FakeRates>,
    sink: Arc<CaptureSink>,
) -> Arc<TonRuntime> {
    let cfg = TonConfig::from_vars(|k| vars.get(k).cloned())
        .unwrap()
        .unwrap();
    Arc::new(TonRuntime::new(cfg, chain, rates, Some(sink)))
}

async fn env_with(extra: &[(&'static str, &str)]) -> Env {
    env_priced(extra, None).await
}

/// As [`env_with`], priced at `ton` nanotons (in TON) instead of $40.
async fn env_priced(extra: &[(&'static str, &str)], ton: Option<u128>) -> Env {
    env_priced_in(extra, ton, setup_test_db().await).await
}

/// As [`env_priced`], on a migrated `pool` of the caller's (the Optimize
/// stage's measurements write theirs to a file).
async fn env_priced_in(extra: &[(&'static str, &str)], ton: Option<u128>, pool: SqlitePool) -> Env {
    let (_user_id, account_id, et_id) = seed_test_data(&pool).await;
    // The editable flags say "free and not held": payment must not care.
    sqlx::query(
        "UPDATE event_types SET requires_confirmation = 0, pending_blocks_slot = 0 WHERE id = ?",
    )
    .bind(&et_id)
    .execute(&pool)
    .await
    .unwrap();
    match ton {
        None => set_price(&pool, &et_id, 4000).await,
        Some(nano) => set_ton_price(&pool, &et_id, nano).await,
    }
    let dir = TestDataDir::new();
    let state = build_state(pool.clone(), dir.to_path_buf(), [0u8; 32]).await;
    let now = Utc::now();
    let chain = FakeChain::new(now.timestamp() as u64);
    let rates = Arc::new(FakeRates::new(vec![
        (SourceKind::CoinGecko, 2_000_000),
        (SourceKind::Binance, 2_000_000),
    ]));
    let sink = Arc::new(CaptureSink::default());
    let rt = runtime(&vars(extra), chain.clone(), rates.clone(), sink.clone());
    state.ton.install(rt.clone());
    let env = Env {
        pool,
        state,
        rt,
        chain,
        rates,
        sink,
        et_id,
        account_id,
        now,
        next_lt: std::sync::atomic::AtomicU64::new(1_000),
        next_slot: std::sync::atomic::AtomicI64::new(0),
        _dir: dir,
    };
    // A first step: cursor row, wallet status, network, heartbeat.
    let r = env.step(now).await;
    assert_eq!(r.error, None);
    assert!(env.rt.ready(now).is_ok());
    env
}

async fn env() -> Env {
    env_with(&[]).await
}

/// A price in TON: the amount asked, no rate.
async fn set_ton_price(pool: &SqlitePool, et_id: &str, nano: u128) {
    sqlx::query(
        "INSERT INTO ton_prices (event_type_id, currency, price_nano, updated_at) VALUES (?, 'TON', ?, '2026-11-09T00:00:00Z')
         ON CONFLICT (event_type_id) DO UPDATE SET currency = 'TON', price_usd_cents = NULL,
             price_nano = excluded.price_nano, enabled = 1",
    )
    .bind(et_id)
    .bind(i64::try_from(nano).unwrap())
    .execute(pool)
    .await
    .unwrap();
}

/// A price in USD (most tests: $40 at $2 per TON is 20 TON).
async fn set_price(pool: &SqlitePool, et_id: &str, cents: i64) {
    sqlx::query(
        "INSERT INTO ton_prices (event_type_id, currency, price_usd_cents, updated_at) VALUES (?, 'USD', ?, '2026-11-09T00:00:00Z')
         ON CONFLICT (event_type_id) DO UPDATE SET currency = 'USD', price_nano = NULL,
             price_usd_cents = excluded.price_usd_cents, enabled = 1",
    )
    .bind(et_id)
    .bind(cents)
    .execute(pool)
    .await
    .unwrap();
}

#[derive(Debug)]
struct Inv {
    status: String,
    problem: Option<String>,
    paid: i64,
    excess: i64,
    settle_by: chrono::DateTime<Utc>,
    hard_deadline: String,
    reference: String,
    amount: i64,
}

impl Env {
    async fn step(&self, now: chrono::DateTime<Utc>) -> watcher::StepReport {
        watcher::step(&self.state, &self.rt, now).await
    }

    fn lt(&self) -> u64 {
        self.next_lt
            .fetch_add(1_000, std::sync::atomic::Ordering::SeqCst)
    }

    /// A pending booking 3 days out (each one an hour after the last) with
    /// an open invoice.
    async fn booking(&self, email: &str) -> (String, OpenedInvoice) {
        let n = self
            .next_slot
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.booking_at(
            email,
            self.now + Duration::days(3) + Duration::hours(n),
            "203.0.113.5",
        )
        .await
        .unwrap()
    }

    async fn booking_at(
        &self,
        email: &str,
        start: chrono::DateTime<Utc>,
        ip: &str,
    ) -> Result<(String, OpenedInvoice), Refusal> {
        let p = invoice::prepare_invoice(&self.state, &self.et_id, start, 0, email, ip, self.now)
            .await?
            .expect("the type is priced");
        let bid =
            insert_v1_booking(&self.pool, &self.et_id, email, start, 30, "pending", &[]).await;
        let mut tx = self.pool.begin().await.unwrap();
        let opened = invoice::open_invoice(&mut tx, &bid, &p).await;
        match opened {
            Ok(o) => {
                tx.commit().await.unwrap();
                invoice::invoice_opened(&self.rt, &bid, &o);
                Ok((bid, o))
            }
            Err(r) => {
                tx.rollback().await.unwrap();
                sqlx::query("DELETE FROM bookings WHERE id = ?")
                    .bind(&bid)
                    .execute(&self.pool)
                    .await
                    .unwrap();
                Err(r)
            }
        }
    }

    /// Move every invoice's creation (and close) back by `by`.
    async fn age_invoices(&self, by: Duration) {
        let rows: Vec<(String, String, Option<String>)> =
            sqlx::query_as("SELECT id, created_at, closed_at FROM ton_invoices")
                .fetch_all(&self.pool)
                .await
                .unwrap();
        let back =
            |s: &str| crate::web::ton_pay::stamp(crate::web::ton_pay::parse_stamp(s).unwrap() - by);
        for (id, created, closed) in rows {
            sqlx::query("UPDATE ton_invoices SET created_at = ?, closed_at = ? WHERE id = ?")
                .bind(back(&created))
                .bind(closed.as_deref().map(back))
                .bind(&id)
                .execute(&self.pool)
                .await
                .unwrap();
        }
    }

    async fn inv(&self, booking_id: &str) -> Inv {
        let (status, problem, paid, excess, settle_by, hard_deadline, reference, amount): (
            String,
            Option<String>,
            i64,
            i64,
            String,
            String,
            String,
            i64,
        ) = sqlx::query_as(
            "SELECT status, problem, paid_nano, excess_nano, settle_by, hard_deadline, reference, amount_nano
             FROM ton_invoices WHERE booking_id = ?",
        )
        .bind(booking_id)
        .fetch_one(&self.pool)
        .await
        .unwrap();
        Inv {
            status,
            problem,
            paid,
            excess,
            settle_by: crate::web::ton_pay::parse_stamp(&settle_by).unwrap(),
            hard_deadline,
            reference,
            amount,
        }
    }

    /// A final transfer to `addr` with `comment`, dated `at`.
    fn pay_to(&self, addr: &str, at: i64, value: u128, comment: &str) -> Value {
        let lt = self.lt();
        let mut tx = crate::ton::decide::tests::payment(lt, at as u64, value, comment);
        tx["account"] = json!(addr);
        tx["in_msg"]["destination"] = json!(addr);
        tx["in_msg"]["source"] = json!(OTHER);
        tx["mc_block_seqno"] = json!(1);
        tx
    }

    fn pay(&self, value: u128, comment: &str) -> Value {
        let tx = self.pay_to(OUR, self.now.timestamp(), value, comment);
        self.chain.push(tx.clone());
        tx
    }

    async fn payments(&self) -> Vec<(String, Option<String>, i64)> {
        sqlx::query_as("SELECT outcome, reason, value_nano FROM ton_payments ORDER BY lt")
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }

    async fn cursor(&self, addr: &str) -> i64 {
        sqlx::query_scalar(
            "SELECT cursor_lt FROM ton_watch WHERE network = 'mainnet' AND address_raw = ?",
        )
        .bind(addr)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }
}

fn code(stored: &str) -> String {
    crate::ton::reference::display(stored)
}

async fn pending_expires_at(pool: &SqlitePool, id: &str) -> Option<String> {
    sqlx::query_scalar("SELECT pending_expires_at FROM bookings WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

// --- invoices and the slot hold ----------------------------------------------

#[tokio::test]
async fn an_open_invoice_holds_the_slot_whatever_the_flags_say() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    assert!(opened.reference.starts_with("B-"));
    assert_eq!(opened.amount_nano as u128, AMOUNT);
    let inv = e.inv(&bid).await;
    assert_eq!(inv.status, "open");
    assert_eq!(inv.amount as u128, AMOUNT);
    assert_eq!(inv.settle_by, opened.expires_at + Duration::minutes(15));
    // The hard deadline replaces calrs's own; no halfway nudge.
    assert_eq!(
        pending_expires_at(&e.pool, &bid).await,
        Some(inv.hard_deadline.clone())
    );
    let since: Option<String> =
        sqlx::query_scalar("SELECT pending_since FROM bookings WHERE id = ?")
            .bind(&bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!(since, None);
    let busy = |pool: SqlitePool| async move {
        let user: String = sqlx::query_scalar("SELECT id FROM users LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        let n: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM bookings b JOIN event_types et ON et.id = b.event_type_id
             JOIN accounts a ON a.id = et.account_id WHERE {}",
            crate::booking_time::user_busy_bookings(true)
        ))
        .bind(&user)
        .bind(&user)
        .bind(&user)
        .fetch_one(&pool)
        .await
        .unwrap();
        n
    };
    // pending_blocks_slot = 0, yet the invoice holds the time.
    assert_eq!(busy(e.pool.clone()).await, 1);
    // An unpriced pending booking on the same type does not.
    insert_v1_booking(
        &e.pool,
        &e.et_id,
        "free@example.com",
        e.now + Duration::days(4),
        30,
        "pending",
        &[],
    )
    .await;
    assert_eq!(busy(e.pool.clone()).await, 1);
    // Waived still holds (a confirmation may be retried); expired does not.
    sqlx::query("UPDATE ton_invoices SET status = 'waived'")
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(busy(e.pool.clone()).await, 1);
    sqlx::query("UPDATE ton_invoices SET status = 'expired'")
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(busy(e.pool.clone()).await, 0);
    // The approval re-check (no pending holds) is unchanged.
    assert!(!crate::booking_time::user_busy_bookings(false).contains("ton_invoices"));
}

#[tokio::test]
async fn refusals_never_create_a_free_booking() {
    let e = env().await;
    let start = e.now + Duration::days(3);
    // Not priced: the upstream path.
    sqlx::query("UPDATE ton_prices SET enabled = 0")
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(
        invoice::prepare_invoice(&e.state, &e.et_id, start, 0, "a@b.c", "1.1.1.1", e.now)
            .await
            .map(|p| p.is_some()),
        Ok(false)
    );
    assert!(!invoice::is_priced(&e.pool, &e.et_id).await);
    sqlx::query("UPDATE ton_prices SET enabled = 1")
        .execute(&e.pool)
        .await
        .unwrap();
    assert!(invoice::is_priced(&e.pool, &e.et_id).await);
    // Too close to the start.
    assert_eq!(
        e.booking_at("a@b.c", e.now + Duration::minutes(25), "1.1.1.1")
            .await
            .err(),
        Some(Refusal::TooLate)
    );
    // One open invoice per email (case and spaces ignored).
    e.booking_at("Guest@Example.com", start, "1.1.1.1")
        .await
        .unwrap();
    assert_eq!(
        e.booking_at(" guest@example.com ", start + Duration::hours(1), "9.9.9.9")
            .await
            .err(),
        Some(Refusal::AlreadyOpen)
    );
    // Two per client address.
    e.booking_at("two@example.com", start + Duration::hours(2), "1.1.1.1")
        .await
        .unwrap();
    assert_eq!(
        e.booking_at("three@example.com", start + Duration::hours(3), "1.1.1.1")
            .await
            .err(),
        Some(Refusal::AlreadyOpen)
    );
    // Five open in total.
    for (i, ip) in ["2.2.2.2", "3.3.3.3", "4.4.4.4"].iter().enumerate() {
        e.booking_at(
            &format!("t{i}@example.com"),
            start + Duration::hours(4 + i as i64),
            ip,
        )
        .await
        .unwrap();
    }
    assert_eq!(
        e.booking_at("six@example.com", start + Duration::hours(9), "5.5.5.5")
            .await
            .err(),
        Some(Refusal::Capacity)
    );
    assert!(e.sink.health_kinds().contains(&"capacity".to_string()));
    let open: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ton_invoices WHERE status = 'open'")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(open, 5);
    // No booking row was left behind by any refusal.
    let bookings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(bookings, 5);
}

#[tokio::test]
async fn the_per_day_limit_and_the_in_transaction_recheck() {
    let e = env_with(&[
        ("CALRS_TON_MAX_PER_DAY", "2"),
        ("CALRS_TON_MAX_OPEN_TOTAL", "10"),
    ])
    .await;
    let start = e.now + Duration::days(3);
    // Two prepared concurrently for the same email: the second open fails
    // inside its transaction.
    let p1 = invoice::prepare_invoice(&e.state, &e.et_id, start, 0, "same@x.io", "1.1.1.1", e.now)
        .await
        .unwrap()
        .unwrap();
    let p2 = invoice::prepare_invoice(
        &e.state,
        &e.et_id,
        start + Duration::hours(1),
        0,
        "same@x.io",
        "1.1.1.2",
        e.now,
    )
    .await
    .unwrap()
    .unwrap();
    let b1 = insert_v1_booking(&e.pool, &e.et_id, "same@x.io", start, 30, "pending", &[]).await;
    let b2 = insert_v1_booking(
        &e.pool,
        &e.et_id,
        "same@x.io",
        start + Duration::hours(1),
        30,
        "pending",
        &[],
    )
    .await;
    let mut tx = e.pool.begin().await.unwrap();
    invoice::open_invoice(&mut tx, &b1, &p1).await.unwrap();
    tx.commit().await.unwrap();
    let mut tx = e.pool.begin().await.unwrap();
    assert_eq!(
        invoice::open_invoice(&mut tx, &b2, &p2).await,
        Err(Refusal::AlreadyOpen)
    );
    tx.rollback().await.unwrap();
    // A second invoice today, then the daily cap: both have been open for
    // long enough to count.
    e.booking_at("other@x.io", start + Duration::hours(2), "1.1.2.3")
        .await
        .unwrap();
    e.age_invoices(Duration::minutes(11)).await;
    assert_eq!(
        e.booking_at("third@x.io", start + Duration::hours(3), "1.1.3.4")
            .await
            .err(),
        Some(Refusal::Capacity)
    );
    // The cap holds at prepare time too.
    let p3 =
        invoice::prepare_invoice(&e.state, &e.et_id, start, 0, "c@x.io", "1.1.4.9", e.now).await;
    assert_eq!(p3.err(), Some(Refusal::Capacity));
}

#[tokio::test]
async fn payments_off_wallet_inactive_network_or_rate_problems_refuse() {
    let e = env().await;
    let start = e.now + Duration::days(3);
    // A priced team type: personal types only.
    let (_, team_et) = insert_team_with_et(&e.pool, "crew", "crew-call", &[]).await;
    set_price(&e.pool, &team_et, 4000).await;
    assert_eq!(
        invoice::prepare_invoice(&e.state, &team_et, start, 0, "a@b.c", "1.1.1.1", e.now)
            .await
            .err(),
        Some(Refusal::PersonalOnly)
    );
    assert!(invoice::is_priced(&e.pool, &team_et).await);

    // Rates that disagree by more than 3 %.
    *e.rates.0.lock().unwrap() = vec![
        (SourceKind::CoinGecko, 2_000_000),
        (SourceKind::Binance, 2_100_000),
    ];
    e.rt.forget_rate();
    let r = e.booking_at("r@b.c", start, "1.1.1.1").await.err();
    assert!(matches!(r, Some(Refusal::Rate(_))), "{r:?}");
    assert!(e
        .sink
        .health_kinds()
        .contains(&"rate_unavailable".to_string()));
    // Outside the owner's corridor ($0.50–$5).
    *e.rates.0.lock().unwrap() = vec![
        (SourceKind::CoinGecko, 9_000_000),
        (SourceKind::Binance, 9_000_000),
    ];
    let r = e.booking_at("r@b.c", start, "1.1.1.1").await.err();
    assert!(matches!(r, Some(Refusal::Rate(_))), "{r:?}");
    *e.rates.0.lock().unwrap() = vec![
        (SourceKind::CoinGecko, 2_000_000),
        (SourceKind::Binance, 2_000_000),
    ];

    // A stale heartbeat (the watcher stopped) refuses.
    assert!(e.rt.ready(e.now + Duration::minutes(3)).is_err());
    assert!(e.rt.ready(e.now + Duration::minutes(1)).is_ok());

    // The wallet became inactive: checked again an hour later.
    e.chain.edit(|c| c.status = Some("uninit".into()));
    let later = e.now + Duration::minutes(61);
    e.step(later).await;
    assert!(e.rt.ready(later).is_err());
    assert!(e
        .sink
        .health_kinds()
        .contains(&"wallet_inactive".to_string()));
    e.chain.edit(|c| c.status = Some("active".into()));

    // No runtime at all: unavailable, never free.
    let off = build_state(e.pool.clone(), e._dir.to_path_buf(), [0u8; 32]).await;
    assert!(matches!(
        invoice::prepare_invoice(&off, &e.et_id, start, 0, "a@b.c", "1.1.1.1", e.now).await,
        Err(Refusal::Unavailable(_))
    ));
}

// --- the watcher -------------------------------------------------------------

#[tokio::test]
async fn a_final_payment_confirms_the_booking_once() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    e.pay(AMOUNT, &opened.reference.to_lowercase().replace('-', " "));
    let r = e.step(e.now).await;
    assert_eq!(r.error, None);
    assert_eq!((r.credited, r.confirmed), (1, 1));
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    let inv = e.inv(&bid).await;
    assert_eq!(
        (inv.status.as_str(), inv.problem.as_deref()),
        ("confirmed", None)
    );
    assert_eq!(inv.paid as u128, AMOUNT);
    assert_eq!(
        e.payments().await,
        vec![("credited".into(), None, AMOUNT as i64)]
    );
    assert_eq!(e.sink.received(), 1);
    // A second step changes nothing and says nothing new.
    let r = e.step(e.now + Duration::seconds(10)).await;
    assert_eq!((r.credited, r.confirmed, r.expired), (0, 0, 0));
    assert_eq!(e.payments().await.len(), 1);
    assert_eq!(e.sink.received(), 1);
    assert!(e.sink.payment_kinds().is_empty());
}

#[tokio::test]
async fn partial_then_the_rest_confirms() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    e.pay(AMOUNT / 4, &opened.reference);
    e.step(e.now).await;
    let inv = e.inv(&bid).await;
    assert_eq!(inv.status, "open");
    assert_eq!(inv.paid as u128, AMOUNT / 4);
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    assert_eq!(e.sink.payment_kinds(), vec!["partial".to_string()]);
    e.pay(AMOUNT - AMOUNT / 4, &opened.reference);
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    let inv = e.inv(&bid).await;
    assert_eq!((inv.status.as_str(), inv.excess), ("confirmed", 0));
    assert_eq!(e.sink.payment_kinds(), vec!["partial".to_string()]);
}

#[tokio::test]
async fn an_underpayment_never_confirms_and_expires_as_partial() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    // One nano short: tolerance is 0 by default.
    e.pay(AMOUNT - 1, &opened.reference);
    e.step(e.now).await;
    assert_eq!(e.inv(&bid).await.status, "open");
    let settle_by = e.inv(&bid).await.settle_by;
    // The indexer is past settle_by + 3 min and the cursor caught up.
    e.chain
        .height((settle_by + Duration::minutes(3)).timestamp() as u64);
    let at = settle_by + Duration::minutes(4);
    let r = e.step(at).await;
    assert_eq!(r.expired, 1);
    let inv = e.inv(&bid).await;
    assert_eq!(
        (inv.status.as_str(), inv.problem.as_deref()),
        ("expired", Some("partial"))
    );
    assert_eq!(
        pending_expires_at(&e.pool, &bid).await,
        Some(crate::web::ton_pay::stamp(at))
    );
    assert!(e.sink.problems().contains(&"partial".to_string()));
    // calrs's own sweep declines it.
    let swept = sweep_pending_requests(&e.pool, &[0u8; 32], at + Duration::seconds(30)).await;
    assert_eq!(swept.expired, 1);
    assert_eq!(booking_status(&e.pool, &bid).await, "declined");
}

#[tokio::test]
async fn a_tolerance_accepts_a_fee_taken_from_the_amount() {
    let e = env_with(&[("CALRS_TON_UNDERPAY_TOLERANCE_NANO", "10000000")]).await;
    let (bid, opened) = e.booking("guest@example.com").await;
    e.pay(AMOUNT - 5_000_000, &opened.reference);
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    assert_eq!(e.inv(&bid).await.excess, 0);
}

#[tokio::test]
async fn an_overpayment_confirms_and_records_the_excess() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    e.pay(AMOUNT + 1_000_000_000, &opened.reference);
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    let inv = e.inv(&bid).await;
    assert_eq!(inv.status, "confirmed");
    assert_eq!(inv.problem.as_deref(), Some("overpaid"));
    assert_eq!(inv.excess, 1_000_000_000);
    assert_eq!(e.sink.problems(), vec!["overpaid".to_string()]);
    // Another payment with the same code: overpaid as a whole.
    e.pay(AMOUNT, &opened.reference);
    e.step(e.now).await;
    assert_eq!(e.inv(&bid).await.paid as u128, AMOUNT + 1_000_000_000);
    assert_eq!(e.payments().await[1].0, "overpaid");
    assert_eq!(e.sink.payment_kinds(), vec!["overpaid".to_string()]);
}

#[tokio::test]
async fn a_payment_after_settle_by_is_late_and_never_revives_the_booking() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let settle_by = e.inv(&bid).await.settle_by;
    let late = e.pay_to(OUR, settle_by.timestamp() + 1, AMOUNT, &opened.reference);
    e.chain.push(late);
    e.chain
        .height((settle_by + Duration::minutes(5)).timestamp() as u64);
    let r = e.step(settle_by + Duration::minutes(5)).await;
    assert_eq!(r.credited, 0);
    assert_eq!(e.payments().await[0].0, "late");
    assert_eq!(e.inv(&bid).await.status, "expired");
    assert_eq!(e.sink.payment_kinds(), vec!["late".to_string()]);
    sweep_pending_requests(&e.pool, &[0u8; 32], settle_by + Duration::minutes(6)).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "declined");
    // Paying again later: still late, still declined.
    e.pay(AMOUNT, &opened.reference);
    e.step(settle_by + Duration::minutes(7)).await;
    assert_eq!(e.payments().await[1].0, "late");
    assert_eq!(booking_status(&e.pool, &bid).await, "declined");
}

#[tokio::test]
async fn a_lagging_indexer_does_not_expire_an_invoice_paid_in_time() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let settle_by = e.inv(&bid).await.settle_by;
    // Our clock is far past settle_by, the indexer is not.
    e.chain
        .height((settle_by + Duration::minutes(2)).timestamp() as u64);
    let r = e.step(settle_by + Duration::hours(1)).await;
    assert_eq!(r.expired, 0);
    assert_eq!(e.inv(&bid).await.status, "open");
    // The transfer sent in the last minute shows up later: credited.
    let tx = e.pay_to(OUR, settle_by.timestamp() - 30, AMOUNT, &opened.reference);
    e.chain.push(tx);
    e.chain
        .height((settle_by + Duration::minutes(10)).timestamp() as u64);
    e.step(settle_by + Duration::hours(1)).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
}

#[tokio::test]
async fn spam_before_the_payment_delays_expiry_until_the_cursor_catches_up() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let settle_by = e.inv(&bid).await.settle_by;
    for i in 0..3_200 {
        let mut t = e.pay_to(OUR, e.now.timestamp(), 1_000, &format!("free airdrop {i}"));
        t["in_msg"]["source"] = json!(SPAMMER);
        e.chain.push(t);
    }
    let tx = e.pay_to(OUR, settle_by.timestamp() - 60, AMOUNT, &opened.reference);
    e.chain.push(tx);
    e.chain
        .height((settle_by + Duration::minutes(30)).timestamp() as u64);
    let at = settle_by + Duration::minutes(30);
    let r = e.step(at).await;
    assert!(!r.caught_up);
    assert_eq!((r.credited, r.expired), (0, 0));
    assert_eq!(e.inv(&bid).await.status, "open");
    let r = e.step(at).await;
    assert!(r.caught_up);
    assert_eq!(r.credited, 1);
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    // Nothing below the threshold was stored or alerted.
    assert_eq!(e.payments().await.len(), 1);
    assert!(e.sink.payment_kinds().is_empty());
}

#[tokio::test]
async fn unmatched_transfers_alert_once_and_small_ones_are_not_stored() {
    let e = env().await;
    let small = e.pay(1_000, "hello");
    let big = e.pay(500_000_000, "B-7KQ3M9XD2F");
    for i in 0..5 {
        e.step(e.now + Duration::seconds(i)).await;
    }
    assert_eq!(
        e.payments().await,
        vec![("unmatched".into(), None, 500_000_000)]
    );
    assert_eq!(e.sink.payment_kinds(), vec!["unmatched".to_string()]);
    // The cursor passed both.
    assert_eq!(e.cursor(OUR).await as u64, lt_of(&big));
    assert!(lt_of(&small) < lt_of(&big));
    // The sender's comment is stored for the owner, cleaned.
    let comment: Option<String> = sqlx::query_scalar("SELECT comment FROM ton_payments")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(comment.as_deref(), Some("B-7KQ3M9XD2F"));
}

#[tokio::test]
async fn the_same_transaction_is_never_credited_twice() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let first = e.pay(AMOUNT / 2, &opened.reference);
    // The same hash again at a later lt (an indexer replaying it).
    let mut again = first.clone();
    again["lt"] = json!((lt_of(&first) + 500).to_string());
    e.chain.push(again);
    e.step(e.now).await;
    let inv = e.inv(&bid).await;
    assert_eq!(inv.paid as u128, AMOUNT / 2);
    assert_eq!(inv.status, "open");
    assert_eq!(e.payments().await.len(), 1);
    assert_eq!(e.sink.payment_kinds(), vec!["partial".to_string()]);
    // Replaying from an older cursor gives nothing new either.
    sqlx::query("UPDATE ton_watch SET cursor_lt = 0")
        .execute(&e.pool)
        .await
        .unwrap();
    e.step(e.now).await;
    assert_eq!(e.inv(&bid).await.paid as u128, AMOUNT / 2);
    assert_eq!(e.payments().await.len(), 1);
}

#[tokio::test]
async fn ignored_transfers_above_the_threshold_are_recorded() {
    let e = env().await;
    let (_bid, opened) = e.booking("guest@example.com").await;
    // A jetton notify carrying the code: never TON.
    let mut jetton = e.pay_to(OUR, e.now.timestamp(), 500_000_000, &opened.reference);
    jetton["in_msg"]["opcode"] = json!("0x7362d09c");
    jetton["in_msg"]["message_content"]["decoded"] =
        json!({"type": "jetton_notify", "comment": opened.reference});
    e.chain.push(jetton);
    // A failed transfer.
    let mut failed = e.pay_to(OUR, e.now.timestamp(), 500_000_000, &opened.reference);
    failed["description"]["aborted"] = json!(true);
    e.chain.push(failed);
    // A bounce of our own transfer.
    let mut bounced = e.pay_to(OUR, e.now.timestamp(), 500_000_000, "x");
    bounced["in_msg"]["bounced"] = json!(true);
    e.chain.push(bounced);
    // An external message (our wallet sending): passed, not recorded.
    let mut external = e.pay_to(OUR, e.now.timestamp(), 500_000_000, "x");
    external["in_msg"]["source"] = Value::Null;
    e.chain.push(external);
    // A final transaction whose value cannot be read.
    let mut odd = e.pay_to(OUR, e.now.timestamp(), 1, "x");
    odd["in_msg"]["value"] = json!("a lot");
    e.chain.push(odd);
    e.step(e.now).await;
    let reasons: Vec<Option<String>> = e.payments().await.into_iter().map(|p| p.1).collect();
    assert_eq!(
        reasons,
        vec![
            Some("not_text".into()),
            Some("failed".into()),
            Some("bounced".into()),
            Some("unparsed".into())
        ]
    );
    assert!(e.payments().await.iter().all(|p| p.0 == "ignored"));
    let inv = e.inv(&_bid).await;
    assert_eq!((inv.status.as_str(), inv.paid), ("open", 0));
    assert_eq!(
        e.sink.payment_kinds(),
        vec!["not_text", "failed", "bounced", "unparsed"]
    );
}

#[tokio::test]
async fn a_transfer_dated_before_the_invoice_is_ignored() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let tx = e.pay_to(OUR, e.now.timestamp() - 3600, AMOUNT, &opened.reference);
    e.chain.push(tx);
    e.step(e.now).await;
    assert_eq!(
        e.payments().await,
        vec![(
            "ignored".into(),
            Some("before_invoice".into()),
            AMOUNT as i64
        )]
    );
    assert_eq!(e.inv(&bid).await.status, "open");
}

#[tokio::test]
async fn the_wrong_network_credits_and_expires_nothing() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    e.pay(AMOUNT, &opened.reference);
    let settle_by = e.inv(&bid).await.settle_by;
    e.chain.edit(|c| {
        c.global_id = -3;
        c.gen_utime = (settle_by + Duration::hours(1)).timestamp() as u64;
    });
    let r = e.step(e.now).await;
    assert_eq!(r.error, Some(StepError::WrongNetwork(-3)));
    assert_eq!(e.inv(&bid).await.status, "open");
    assert!(e.payments().await.is_empty());
    assert!(e.sink.health_kinds().contains(&"wrong_network".to_string()));
    assert!(e.rt.ready(e.now).is_err());
    assert_eq!(
        watcher::next_delay(&e.pool, &e.rt, &r, e.now).await,
        std::time::Duration::from_secs(300)
    );
}

#[tokio::test]
async fn a_transfer_that_is_not_final_stops_the_cursor_until_it_is() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let mut tx = e.pay_to(OUR, e.now.timestamp(), AMOUNT, &opened.reference);
    tx["finality"] = json!("confirmed");
    e.chain.push(tx.clone());
    // A later final transaction must wait behind it.
    let after = e.pay(500_000_000, "unrelated");
    let r = e.step(e.now).await;
    assert!(!r.caught_up);
    assert_eq!(r.credited, 0);
    assert!(e.payments().await.is_empty());
    assert!((e.cursor(OUR).await as u64) < lt_of(&tx));
    assert!(e.rt.seen.lock().unwrap().contains_key(&opened.id));
    // Still stuck a quarter of an hour later: an alert.
    e.step(e.now + Duration::minutes(16)).await;
    assert!(e
        .sink
        .health_kinds()
        .contains(&"stuck_finality".to_string()));
    // Final now.
    e.chain.edit(|c| {
        for t in c.txs.iter_mut() {
            if t["lt"] == tx["lt"] {
                t["finality"] = json!(2);
            }
        }
    });
    let r = e.step(e.now + Duration::minutes(17)).await;
    assert!(r.caught_up);
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    assert_eq!(e.cursor(OUR).await as u64, lt_of(&after));
}

#[tokio::test]
async fn a_restart_continues_from_the_cursor() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    e.pay(500_000_000, "not a code");
    e.step(e.now).await;
    let cursor = e.cursor(OUR).await;
    assert!(cursor > 0);
    // A new runtime over the same database: the first-run cursor is not
    // taken again, nothing is processed twice.
    let sink2 = Arc::new(CaptureSink::default());
    let rt2 = runtime(&vars(&[]), e.chain.clone(), e.rates.clone(), sink2.clone());
    e.pay(AMOUNT, &opened.reference);
    let r = watcher::step(&e.state, &rt2, e.now).await;
    assert_eq!(r.error, None);
    assert_eq!(r.credited, 1);
    assert_eq!(e.payments().await.len(), 2);
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    assert!(sink2.payment_kinds().is_empty());
}

#[tokio::test]
async fn invoices_on_a_previous_address_are_still_watched() {
    let e = env().await;
    let (old_bid, old) = e.booking("old@example.com").await;
    // The owner moves to another address; serve restarts with it.
    let rt2 = runtime(
        &vars(&[("CALRS_TON_ADDRESS", OTHER_UQ)]),
        e.chain.clone(),
        e.rates.clone(),
        e.sink.clone(),
    );
    e.state.ton.install(rt2.clone());
    watcher::step(&e.state, &rt2, e.now).await;
    assert!(rt2.ready(e.now).is_ok());
    let (new_bid, new) = e.booking("new@example.com").await;
    let new_addr: String =
        sqlx::query_scalar("SELECT address_raw FROM ton_invoices WHERE booking_id = ?")
            .bind(&new_bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!(new_addr, OTHER);
    let a = e.pay_to(OUR, e.now.timestamp(), AMOUNT, &old.reference);
    let mut b = e.pay_to(OTHER, e.now.timestamp(), AMOUNT, &new.reference);
    b["in_msg"]["source"] = json!(SPAMMER);
    e.chain.push(a);
    e.chain.push(b);
    let r = watcher::step(&e.state, &rt2, e.now).await;
    assert_eq!(r.error, None);
    assert_eq!(r.credited, 2);
    assert_eq!(booking_status(&e.pool, &old_bid).await, "confirmed");
    assert_eq!(booking_status(&e.pool, &new_bid).await, "confirmed");
    let current: Vec<String> =
        sqlx::query_scalar("SELECT address_raw FROM ton_watch WHERE is_current = 1")
            .fetch_all(&e.pool)
            .await
            .unwrap();
    assert_eq!(current, vec![OTHER.to_string()]);
    // A code paid to the wrong address does not credit.
    let (third_bid, third) = e.booking("third@example.com").await;
    let wrong = e.pay_to(OUR, e.now.timestamp(), AMOUNT, &third.reference);
    e.chain.push(wrong);
    // OUR is no longer watched once its invoices settled: the transfer is
    // not even read. Nothing credits the third invoice.
    watcher::step(&e.state, &rt2, e.now).await;
    assert_eq!(e.inv(&third_bid).await.status, "open");
}

#[tokio::test]
async fn host_busy_keeps_the_paid_slot_until_the_start() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let start: String = sqlx::query_scalar("SELECT start_at FROM bookings WHERE id = ?")
        .bind(&bid)
        .fetch_one(&e.pool)
        .await
        .unwrap();
    // The host took something else at that time on another event type.
    let other_et = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO event_types (id, account_id, slug, title, duration_min, buffer_before, buffer_after, min_notice_min, enabled) VALUES (?, ?, 'other', 'Other', 30, 0, 0, 0, 1)")
        .bind(&other_et)
        .bind(&e.account_id)
        .execute(&e.pool)
        .await
        .unwrap();
    let blocker = insert_v1_booking(
        &e.pool,
        &other_et,
        "blocker@example.com",
        crate::web::ton_pay::parse_stamp(&start).unwrap(),
        30,
        "confirmed",
        &[],
    )
    .await;
    e.pay(AMOUNT, &opened.reference);
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    let inv = e.inv(&bid).await;
    assert_eq!(
        (inv.status.as_str(), inv.problem.as_deref()),
        ("paid", Some("host_busy"))
    );
    assert_eq!(pending_expires_at(&e.pool, &bid).await, Some(start));
    assert_eq!(e.sink.problems(), vec!["host_busy".to_string()]);
    // Not retried on its own.
    e.step(e.now).await;
    assert_eq!(e.sink.problems(), vec!["host_busy".to_string()]);
    // The paid booking may be approved by hand once the conflict is gone.
    assert!(!invoice::approval_refused(&e.pool, &bid).await);
    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
        .bind(&blocker)
        .execute(&e.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE bookings SET status = 'confirmed' WHERE id = ?")
        .bind(&bid)
        .execute(&e.pool)
        .await
        .unwrap();
    e.step(e.now).await;
    let inv = e.inv(&bid).await;
    assert_eq!(
        (inv.status.as_str(), inv.problem.as_deref()),
        ("confirmed", None)
    );
    assert_eq!(e.sink.received(), 1);
}

#[tokio::test]
async fn a_paid_booking_that_ended_is_flagged_for_a_refund() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    // Paid, but the guest cancelled before the watcher confirmed it.
    e.pay(AMOUNT, &opened.reference);
    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
        .bind(&bid)
        .execute(&e.pool)
        .await
        .unwrap();
    // Reconcile runs first: open + cancelled → closed; the payment is late.
    e.step(e.now).await;
    assert_eq!(e.inv(&bid).await.status, "closed");
    assert_eq!(e.payments().await[0].0, "late");

    let (bid2, opened2) = e.booking("guest2@example.com").await;
    e.pay(AMOUNT, &opened2.reference);
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid2).await, "confirmed");
    // The host cancels the confirmed, paid booking.
    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
        .bind(&bid2)
        .execute(&e.pool)
        .await
        .unwrap();
    invoice::on_booking_ended(&e.pool, &bid2, e.now).await;
    let inv = e.inv(&bid2).await;
    assert_eq!(
        (inv.status.as_str(), inv.problem.as_deref()),
        ("confirmed", Some("refund_due"))
    );
    e.step(e.now).await;
    assert_eq!(e.sink.problems(), vec!["refund_due".to_string()]);
}

#[tokio::test]
async fn an_unpaid_booking_found_confirmed_is_reported_not_called_paid() {
    let e = env().await;
    let (bid, _) = e.booking("guest@example.com").await;
    sqlx::query("UPDATE bookings SET status = 'confirmed' WHERE id = ?")
        .bind(&bid)
        .execute(&e.pool)
        .await
        .unwrap();
    e.step(e.now).await;
    let inv = e.inv(&bid).await;
    assert_eq!(
        (inv.status.as_str(), inv.problem.as_deref()),
        ("closed", Some("confirmed_unpaid"))
    );
    assert_eq!(e.sink.problems(), vec!["confirmed_unpaid".to_string()]);
}

#[tokio::test]
async fn without_a_watcher_the_hard_deadline_frees_the_slot() {
    let e = env().await;
    let (bid, _) = e.booking("guest@example.com").await;
    let hard = crate::web::ton_pay::parse_stamp(&e.inv(&bid).await.hard_deadline).unwrap();
    assert_eq!(
        sweep_pending_requests(&e.pool, &[0u8; 32], hard - Duration::seconds(1))
            .await
            .expired,
        0
    );
    sweep_pending_requests(&e.pool, &[0u8; 32], hard + Duration::seconds(1)).await;
    drain_emails(std::time::Duration::from_secs(5)).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "declined");
    // The invoice expires with its booking (a payment found later is late),
    // and the next step leaves it so.
    assert_eq!(e.inv(&bid).await.status, "expired");
    e.step(e.now).await;
    assert_eq!(e.inv(&bid).await.status, "expired");
}

// --- guards and the only free path -------------------------------------------

#[tokio::test]
async fn guards_follow_the_invoice_status() {
    let e = env().await;
    let (bid, _) = e.booking("guest@example.com").await;
    let free = insert_v1_booking(
        &e.pool,
        &e.et_id,
        "free@example.com",
        e.now + Duration::days(5),
        30,
        "pending",
        &[],
    )
    .await;
    assert!(!invoice::approval_refused(&e.pool, &free).await);
    assert!(!invoice::settled(&e.pool, &free).await);
    assert!(!invoice::cancel_is_quiet(&e.pool, &free).await);
    for (status, refused) in [
        ("open", true),
        ("expired", true),
        ("closed", true),
        ("paid", false),
        ("confirmed", false),
        ("waived", false),
    ] {
        sqlx::query("UPDATE ton_invoices SET status = ? WHERE booking_id = ?")
            .bind(status)
            .bind(&bid)
            .execute(&e.pool)
            .await
            .unwrap();
        assert_eq!(
            invoice::approval_refused(&e.pool, &bid).await,
            refused,
            "{status}"
        );
        assert_eq!(invoice::unpaid(&e.pool, &bid).await, refused, "{status}");
        assert_eq!(invoice::settled(&e.pool, &bid).await, !refused, "{status}");
        // Nothing paid and the booking pending: the hosts never heard of
        // it, also in the minute between an expiry or a release and the
        // sweep that declines it.
        assert_eq!(
            invoice::cancel_is_quiet(&e.pool, &bid).await,
            refused,
            "{status}"
        );
    }
}

#[tokio::test]
async fn a_waive_moves_only_an_open_invoice() {
    let e = env().await;
    let (bid, _) = e.booking("guest@example.com").await;
    assert!(invoice::approval_refused(&e.pool, &bid).await);
    assert!(invoice::waive(&e.pool, &bid, "dashboard:u1", e.now)
        .await
        .unwrap());
    assert!(!invoice::waive(&e.pool, &bid, "dashboard:u1", e.now)
        .await
        .unwrap());
    let (status, by): (String, Option<String>) =
        sqlx::query_as("SELECT status, waived_by FROM ton_invoices WHERE booking_id = ?")
            .bind(&bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!(
        (status.as_str(), by.as_deref()),
        ("waived", Some("dashboard:u1"))
    );
    assert!(!invoice::approval_refused(&e.pool, &bid).await);
    // The watcher leaves a waived invoice alone, and a payment to it is
    // excess, not a credit.
    let code_ = code(&e.inv(&bid).await.reference);
    e.pay(AMOUNT, &code_);
    e.step(e.now).await;
    assert_eq!(e.inv(&bid).await.status, "waived");
    assert_eq!(e.payments().await[0].0, "overpaid");
    // Expired, closed: no waive.
    let (bid2, _) = e.booking("other@example.com").await;
    sqlx::query("UPDATE ton_invoices SET status = 'expired' WHERE booking_id = ?")
        .bind(&bid2)
        .execute(&e.pool)
        .await
        .unwrap();
    assert!(!invoice::waive(&e.pool, &bid2, "mcp:u1", e.now)
        .await
        .unwrap());
}

#[tokio::test]
async fn closing_a_revoked_address_expires_its_open_bookings() {
    let e = env().await;
    let (bid, _) = e.booking("guest@example.com").await;
    let n = invoice::close_open_for_address(&e.pool, crate::ton::Network::Mainnet, OUR, e.now)
        .await
        .unwrap();
    assert_eq!(n, 1);
    let inv = e.inv(&bid).await;
    assert_eq!(
        (inv.status.as_str(), inv.problem.as_deref()),
        ("closed", Some("address_revoked"))
    );
    assert_eq!(
        pending_expires_at(&e.pool, &bid).await,
        Some(crate::web::ton_pay::stamp(e.now))
    );
}

// --- supervisor, shutdown, health --------------------------------------------

#[tokio::test]
async fn the_supervisor_restarts_a_crashed_watcher_and_stops_on_signal() {
    let e = env().await;
    e.chain.edit(|c| c.panics_left = 3);
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let handle = tokio::spawn(watcher::supervise(
        e.state.clone(),
        stop_rx,
        std::time::Duration::from_millis(5),
    ));
    // It comes back after three panics and completes a step.
    let before = e.rt.health().last_ok;
    let mut ok = false;
    for _ in 0..200 {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        if e.rt.health().last_ok > before && e.chain.edit(|c| c.panics_left) == 0 {
            ok = true;
            break;
        }
    }
    assert!(ok, "the watcher did not recover");
    assert!(e
        .sink
        .health_kinds()
        .contains(&"watcher_crashed".to_string()));
    stop_tx.send(true).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), handle)
        .await
        .expect("the supervisor stops on the signal")
        .unwrap();
}

#[tokio::test]
async fn spawn_without_payments_configured_returns_at_once() {
    let pool = setup_test_db().await;
    let dir = TestDataDir::new();
    let state = build_state(pool, dir.to_path_buf(), [0u8; 32]).await;
    assert!(state.ton.get().is_none());
    let (_tx, rx) = tokio::sync::watch::channel(false);
    let task = crate::web::ton_pay::spawn(state, rx);
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        task.finish(std::time::Duration::from_secs(5)),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn indexer_failures_back_off_and_keep_the_last_good_state() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    e.chain.edit(|c| {
        c.fail_next = vec![
            IndexerError::Auth(403),
            IndexerError::RateLimited,
            IndexerError::RateLimited,
        ]
    });
    let r = e.step(e.now).await;
    assert_eq!(r.error, Some(StepError::Indexer(IndexerError::RateLimited)));
    let d1 = watcher::next_delay(&e.pool, &e.rt, &r, e.now).await;
    let r = e.step(e.now).await;
    let d2 = watcher::next_delay(&e.pool, &e.rt, &r, e.now).await;
    assert!(d2 > d1, "{d1:?} {d2:?}");
    let r = e.step(e.now).await;
    assert_eq!(r.error, Some(StepError::Indexer(IndexerError::Auth(403))));
    assert!(e.sink.health_kinds().contains(&"auth_failed".to_string()));
    let (err, failures): (Option<String>, i64) =
        sqlx::query_as("SELECT last_error, failures FROM ton_watch WHERE is_current = 1")
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!((err.as_deref(), failures), (Some("http_403"), 3));
    // Nothing expired while the indexer was down; it recovers.
    e.pay(AMOUNT, &opened.reference);
    let r = e.step(e.now).await;
    assert_eq!(r.error, None);
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    assert_eq!(e.rt.health().failures, 0);
    // While invoices are open the poll is fast; a day after one, a minute.
    assert_eq!(
        watcher::next_delay(&e.pool, &e.rt, &r, e.now).await,
        std::time::Duration::from_secs(60)
    );
    e.booking("again@example.com").await;
    assert_eq!(
        watcher::next_delay(&e.pool, &e.rt, &r, e.now).await,
        std::time::Duration::from_secs(10)
    );
}

#[tokio::test]
async fn alerts_wait_in_the_database_without_a_sink() {
    let e = env().await;
    let rt = Arc::new(TonRuntime::new(
        e.rt.cfg.clone(),
        e.chain.clone(),
        e.rates.clone(),
        None,
    ));
    e.state.ton.install(rt.clone());
    e.pay(500_000_000, "who knows");
    watcher::step(&e.state, &rt, e.now).await;
    let unclaimed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ton_payments WHERE alert_due = 1 AND owner_alerted_at IS NULL",
    )
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(unclaimed, 1);
    // With a sink installed, it goes out once.
    watcher::step(&e.state, &e.rt, e.now).await;
    watcher::step(&e.state, &e.rt, e.now).await;
    assert_eq!(e.sink.payment_kinds(), vec!["unmatched".to_string()]);
}

#[test]
fn client_tags_are_keyed_and_do_not_reveal_the_address() {
    let a = invoice::client_tag(&[1u8; 32], "203.0.113.5");
    assert_eq!(a.len(), 32);
    assert!(!a.contains("203"));
    assert_ne!(a, invoice::client_tag(&[2u8; 32], "203.0.113.5"));
    assert_ne!(a, invoice::client_tag(&[1u8; 32], "203.0.114.5"));
    assert_eq!(invoice::normalize_email("  A@B.c "), "a@b.c");
}

/// One person rarely holds more than one IPv4 /24 or IPv6 /64: the limits
/// count by network, so rotating addresses inside one does not help.
#[test]
fn client_tags_group_an_ipv4_24_and_an_ipv6_64() {
    let tag = |ip: &str| invoice::client_tag(&[1u8; 32], ip);
    assert_eq!(tag("203.0.113.5"), tag("203.0.113.250"));
    assert_eq!(tag("203.0.113.5"), tag("::ffff:203.0.113.9"));
    assert_ne!(tag("203.0.113.5"), tag("203.0.112.5"));
    assert_eq!(tag("2001:db8:1:2::1"), tag("2001:db8:1:2:ffff::9"));
    assert_ne!(tag("2001:db8:1:2::1"), tag("2001:db8:1:3::1"));
    assert_eq!(
        invoice::client_network("2001:db8:1:2:aaaa::1"),
        "v6:2001:db8:1:2"
    );
    assert_eq!(invoice::client_network("10.1.2.3"), "v4:10.1.2");
}

#[tokio::test]
async fn the_rate_is_cached_and_fixed_in_the_invoice() {
    let e = env().await;
    let (bid, _) = e.booking("guest@example.com").await;
    // The market moves; the invoice keeps its rate and amount.
    *e.rates.0.lock().unwrap() = vec![
        (SourceKind::CoinGecko, 4_000_000),
        (SourceKind::Binance, 4_000_000),
    ];
    let (rate, sources, amount): (i64, String, i64) = sqlx::query_as(
        "SELECT rate_usd_micro, rate_sources, amount_nano FROM ton_invoices WHERE booking_id = ?",
    )
    .bind(&bid)
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(rate, 2_000_000);
    assert_eq!(sources, "coingecko=2.000000;binance=2.000000");
    assert_eq!(amount as u128, AMOUNT);
    assert_eq!(e.rt.cached_rate().unwrap().usd_micro, 2_000_000);
    // One source down: the last two-source rate anchors it (within 10 %).
    e.rt.forget_rate();
    *e.rates.0.lock().unwrap() = vec![(SourceKind::Binance, 2_100_000)];
    let r = e.rt.rate(&e.pool, Utc::now()).await.unwrap();
    assert_eq!(r.usd_micro, 2_100_000);
    assert!(!r.confirmed);
    e.rt.forget_rate();
    *e.rates.0.lock().unwrap() = vec![(SourceKind::Binance, 3_000_000)];
    assert!(e.rt.rate(&e.pool, Utc::now()).await.is_err());
}

#[tokio::test]
async fn setting_a_price_changes_the_event_type_version() {
    let pool = setup_test_db().await;
    let (_, _, et_id) = seed_test_data(&pool).await;
    let before = crate::web::agent::snapshot::snapshot(&pool, &et_id)
        .await
        .unwrap()
        .unwrap();
    set_price(&pool, &et_id, 4000).await;
    let priced = crate::web::agent::snapshot::snapshot(&pool, &et_id)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(before, priced);
    set_price(&pool, &et_id, 5000).await;
    let repriced = crate::web::agent::snapshot::snapshot(&pool, &et_id)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(priced, repriced);
}

// --- the booking flow over HTTP ----------------------------------------------

const CSRF: &str = "ton-test-csrf";

impl Env {
    fn app(&self) -> Router {
        crate::web::router(self.state.clone()).layer(local_proxy_peer())
    }

    async fn session(&self) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO sessions (id, user_id, expires_at)
             SELECT ?, id, ? FROM users WHERE username = 'testuser'",
        )
        .bind(&token)
        .bind(
            (Utc::now() + Duration::days(1))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        )
        .execute(&self.pool)
        .await
        .unwrap();
        token
    }

    /// POST the booking form of `/u/testuser/test-meeting` (or `uri`).
    async fn post_booking(
        &self,
        uri: &str,
        email: &str,
        time: &str,
        ip: &str,
    ) -> axum::http::Response<Body> {
        let body = format!(
            "_csrf={CSRF}&date={}&time={time}&name=Guest+Name&email={}&tz=UTC",
            far_future_monday(),
            urlencoding::encode(email)
        );
        let req = axum::http::Request::builder()
            .method("POST")
            .uri(uri)
            .header("cookie", format!("__Host-calrs_csrf={CSRF}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .header("x-forwarded-for", ip)
            .body(Body::from(body))
            .unwrap();
        self.app().oneshot(req).await.unwrap()
    }

    /// Book the 10:00 slot through the web form: (booking id, pay token).
    async fn book_http(&self, email: &str, time: &str, ip: &str) -> (String, String) {
        let resp = self
            .post_booking("/u/testuser/test-meeting/book", email, time, ip)
            .await;
        assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
        let location = resp.headers()["location"].to_str().unwrap().to_string();
        let token = location.strip_prefix("/booking/pay/").unwrap().to_string();
        let bid: String =
            sqlx::query_scalar("SELECT booking_id FROM ton_invoices WHERE pay_token = ?")
                .bind(&token)
                .fetch_one(&self.pool)
                .await
                .unwrap();
        (bid, token)
    }

    async fn get_page(&self, uri: &str) -> (axum::http::StatusCode, HeaderMapOwned, String) {
        let resp = self.app().oneshot(get(uri)).await.unwrap();
        let status = resp.status();
        let headers = resp.headers().clone();
        (status, headers, unescape(&body_string(resp).await))
    }

    async fn post_authed(&self, uri: &str, session: &str) -> axum::http::Response<Body> {
        self.app()
            .oneshot(post_form(uri, session, CSRF, &format!("_csrf={CSRF}")))
            .await
            .unwrap()
    }

    async fn post_guest(&self, uri: &str, body: &str) -> axum::http::Response<Body> {
        self.app()
            .oneshot(post_form_unauthed(
                uri,
                CSRF,
                &format!("_csrf={CSRF}{body}"),
            ))
            .await
            .unwrap()
    }

    async fn booking_col(&self, bid: &str, col: &str) -> Option<String> {
        sqlx::query_scalar(&format!("SELECT {col} FROM bookings WHERE id = ?"))
            .bind(bid)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}

type HeaderMapOwned = axum::http::HeaderMap;

/// A page as the browser reads it: minijinja escapes `/` in attributes.
fn unescape(html: &str) -> String {
    html.replace("&#x2f;", "/")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
}

fn location(resp: &axum::http::Response<Body>) -> String {
    resp.headers()
        .get("location")
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default()
}

#[tokio::test]
async fn a_paid_booking_goes_to_its_payment_page_and_holds_the_slot() {
    let e = env().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (bid, token) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    assert_eq!(token.len(), 32);
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    let inv = e.inv(&bid).await;
    assert_eq!(inv.status, "open");
    assert_eq!(inv.amount as u128, AMOUNT);
    // The hard deadline, no halfway nudge.
    assert_eq!(
        e.booking_col(&bid, "pending_expires_at").await,
        Some(inv.hard_deadline.clone())
    );
    assert_eq!(e.booking_col(&bid, "pending_since").await, None);
    // Nobody is emailed at creation: nothing to approve, the guest is on
    // the payment page.
    assert!(settled_mails_to(&mails, "guest@test.com").await.is_empty());
    assert!(settled_mails_to(&mails, "test@example.com")
        .await
        .is_empty());

    // The time is held, though pending_blocks_slot = 0.
    let resp = e
        .post_booking(
            "/u/testuser/test-meeting/book",
            "other@test.com",
            "10:00",
            "198.51.100.2",
        )
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body = body_string(resp).await;
    assert!(body.contains(&crate::i18n::translate(
        "en",
        "error-slot-unavailable",
        None
    )));
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 1);

    // The page.
    let (status, headers, page) = e.get_page(&format!("/booking/pay/{token}")).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(headers["referrer-policy"], "no-referrer");
    assert_eq!(headers["cache-control"], "no-store");
    let csp = headers["content-security-policy"].to_str().unwrap();
    assert!(csp.contains("frame-ancestors 'self'"));
    assert!(page.contains("20.00 TON"));
    assert!(page.contains(&code(&inv.reference)));
    assert!(page.contains("UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI"));
    assert!(page.contains("ton://transfer/UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI?amount=20000000000&amp;text="));
    assert!(page.contains("https://app.tonkeeper.com/transfer/UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI?amount=20000000000&amp;text="));
    // Only Tonkeeper's link names the coin: MyTonWallet refuses `asset_id`.
    assert!(page.contains("&amp;asset_id=ton/mainnet/coin\""));
    assert_eq!(page.matches("asset_id=").count(), 1);
    // A QR code per link, the plain one behind a toggle.
    assert_eq!(page.matches("class=\"ton-qr\"").count(), 2);
    assert!(page.contains("<details class=\"ton-other\">"));
    assert!(page.contains("$40.00"));
    assert!(page.contains("CoinGecko + Binance"));
    // Never the guest's details.
    assert!(!page.contains("guest@test.com"));
    assert!(!page.contains("Guest Name"));

    // Status JSON: the state and what is left, nothing else.
    let (status, headers, json) = e.get_page(&format!("/booking/pay/{token}/status")).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(headers["cache-control"], "no-store");
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["state"], "waiting");
    assert_eq!(v["remaining_nano"], AMOUNT.to_string());
    let keys: Vec<&String> = v.as_object().unwrap().keys().collect();
    assert_eq!(keys.len(), 7, "{keys:?}");
    assert!(!json.contains("guest"));

    // The payment arrives: confirmed by the watcher, through the approval
    // path (the guest gets the ordinary confirmation).
    e.pay(AMOUNT, &code(&inv.reference));
    let r = e.step(e.now).await;
    assert_eq!((r.credited, r.confirmed), (1, 1));
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    let (_, _, json) = e.get_page(&format!("/booking/pay/{token}/status")).await;
    assert!(json.contains("\"confirmed\""));
    let (_, _, page) = e.get_page(&format!("/booking/pay/{token}")).await;
    assert!(page.contains(&crate::i18n::translate(
        "en",
        "ton-pay-state-confirmed",
        None
    )));
    assert!(!page.contains("ton://"), "no payment details once paid");
    assert_eq!(settled_mails_to(&mails, "guest@test.com").await.len(), 1);
}

#[tokio::test]
async fn unknown_or_malformed_pay_links_name_nothing() {
    let e = env().await;
    for uri in [
        "/booking/pay/00000000000000000000000000000000".to_string(),
        "/booking/pay/not-a-token".to_string(),
        format!("/booking/pay/{}", "A".repeat(32)),
    ] {
        let (status, _, page) = e.get_page(&uri).await;
        assert_eq!(status, axum::http::StatusCode::NOT_FOUND, "{uri}");
        assert!(page.contains(&crate::i18n::translate(
            "en",
            "ton-bae-body-pay-link-invalid",
            None
        )));
    }
    let (status, _, _) = e
        .get_page("/booking/pay/00000000000000000000000000000000/status")
        .await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn inside_the_widget_the_payment_page_opens_in_the_whole_window() {
    let e = env().await;
    let resp = e
        .post_booking(
            "/u/testuser/test-meeting/book?embed=1",
            "guest@test.com",
            "10:00",
            "198.51.100.1",
        )
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let csp = resp.headers()["content-security-policy"]
        .to_str()
        .unwrap()
        .to_string();
    assert!(csp.contains("frame-ancestors *"));
    let body = unescape(&body_string(resp).await);
    let token: String = sqlx::query_scalar("SELECT pay_token FROM ton_invoices")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert!(body.contains(&format!("href=\"/booking/pay/{token}\" target=\"_top\"")));
    // The payment page itself stays unframeable, embed flag or not.
    let (_, headers, _) = e.get_page(&format!("/booking/pay/{token}?embed=1")).await;
    assert!(headers["content-security-policy"]
        .to_str()
        .unwrap()
        .contains("frame-ancestors 'self'"));
}

#[tokio::test]
async fn the_legacy_route_takes_payment_too() {
    let e = env().await;
    let resp = e
        .post_booking(
            "/test-meeting/book",
            "guest@test.com",
            "10:00",
            "198.51.100.1",
        )
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    assert!(location(&resp).starts_with("/booking/pay/"));
    let status: String = sqlx::query_scalar("SELECT status FROM bookings")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(status, "pending");
}

#[tokio::test]
async fn a_priced_type_is_never_booked_free_when_payment_is_unavailable() {
    let e = env().await;
    // No runtime: payments are off. The type stays priced, so no booking.
    let off = build_state(e.pool.clone(), e._dir.to_path_buf(), [0u8; 32]).await;
    let app = crate::web::router(off).layer(local_proxy_peer());
    let body = format!(
        "_csrf={CSRF}&date={}&time=10:00&name=G&email=g%40test.com&tz=UTC",
        far_future_monday()
    );
    let resp = app
        .oneshot(post_form_unauthed(
            "/u/testuser/test-meeting/book",
            CSRF,
            &body,
        ))
        .await
        .unwrap();
    let page = body_string(resp).await;
    assert!(page.contains(&crate::i18n::translate(
        "en",
        "ton-bae-body-unavailable",
        None
    )));
    // A stale heartbeat refuses too.
    e.rt.with_health(|h| h.last_ok = Some(Utc::now() - Duration::minutes(10)));
    let resp = e
        .post_booking(
            "/u/testuser/test-meeting/book",
            "g@test.com",
            "10:00",
            "198.51.100.9",
        )
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 0, "no booking row from any refusal");
}

#[tokio::test]
async fn group_paths_refuse_a_priced_type() {
    let e = env().await;
    // A dynamic group link takes the first member's personal (priced) type.
    sqlx::query("UPDATE users SET allow_dynamic_group = 1")
        .execute(&e.pool)
        .await
        .unwrap();
    let bob = insert_role_user(&e.pool, "bob@test.com", "user").await;
    sqlx::query("UPDATE users SET allow_dynamic_group = 1 WHERE id = ?")
        .bind(&bob)
        .execute(&e.pool)
        .await
        .unwrap();
    let resp = e
        .post_booking(
            "/u/testuser+bob/test-meeting/book",
            "g@test.com",
            "10:00",
            "198.51.100.3",
        )
        .await;
    let page = unescape(&body_string(resp).await);
    let personal_only = crate::i18n::translate("en", "ton-bae-body-personal-only", None);
    assert!(
        page.contains(&personal_only),
        "{}",
        page.split("<h1>").nth(1).unwrap_or(&page)
    );
    // A team type with a price forced in by hand.
    let (_, team_et) = insert_team_with_et(&e.pool, "crew", "crew-call", &[]).await;
    set_price(&e.pool, &team_et, 4000).await;
    let resp = e
        .post_booking(
            "/team/crew/crew-call/book",
            "g@test.com",
            "10:00",
            "198.51.100.4",
        )
        .await;
    assert!(unescape(&body_string(resp).await).contains(&personal_only));
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 0);
}

#[tokio::test]
async fn no_approval_path_confirms_an_unpaid_booking() {
    let e = env().await;
    let session = e.session().await;
    let (bid, _) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    let confirm: String = e.booking_col(&bid, "confirm_token").await.unwrap();
    let unpaid = crate::i18n::translate("en", "ton-bae-body-unpaid", None);
    for status in ["open", "expired", "closed"] {
        sqlx::query("UPDATE ton_invoices SET status = ? WHERE booking_id = ?")
            .bind(status)
            .bind(&bid)
            .execute(&e.pool)
            .await
            .unwrap();
        // The emailed Approve link, GET and POST.
        let (_, _, page) = e.get_page(&format!("/booking/approve/{confirm}")).await;
        assert!(page.contains(&unpaid), "{status}");
        let resp = e
            .post_guest(&format!("/booking/approve/{confirm}"), "")
            .await;
        assert!(
            unescape(&body_string(resp).await).contains(&unpaid),
            "{status}"
        );
        // The dashboard (and the MCP approval, which goes through it).
        let resp = e
            .post_authed(&format!("/dashboard/bookings/{bid}/confirm"), &session)
            .await;
        assert_eq!(location(&resp), "/dashboard/bookings?error=unpaid");
        // A host reschedule would confirm it at the guest's pick.
        let resp = e
            .app()
            .oneshot(get_authed(
                &format!("/dashboard/bookings/{bid}/reschedule"),
                &session,
            ))
            .await
            .unwrap();
        assert_eq!(location(&resp), "/dashboard/bookings?error=unpaid");
        let resp = e
            .post_authed(&format!("/dashboard/bookings/{bid}/reschedule"), &session)
            .await;
        assert_eq!(location(&resp), "/dashboard/bookings?error=unpaid");
        assert_eq!(booking_status(&e.pool, &bid).await, "pending", "{status}");
        let by_host: i64 =
            sqlx::query_scalar("SELECT reschedule_by_host FROM bookings WHERE id = ?")
                .bind(&bid)
                .fetch_one(&e.pool)
                .await
                .unwrap();
        assert_eq!(by_host, 0, "{status}");
    }
    // The dashboard explains it.
    let resp = e
        .app()
        .oneshot(get_authed("/dashboard/bookings?error=unpaid", &session))
        .await
        .unwrap();
    assert!(
        unescape(&body_string(resp).await).contains(&crate::i18n::translate(
            "en",
            "ton-dash-error-unpaid",
            None
        ))
    );
}

#[tokio::test]
async fn a_stale_host_reschedule_flag_does_not_let_the_guest_confirm_an_unpaid_booking() {
    let e = env().await;
    let (bid, _) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    sqlx::query("UPDATE bookings SET reschedule_by_host = 1 WHERE id = ?")
        .bind(&bid)
        .execute(&e.pool)
        .await
        .unwrap();
    let token = e.booking_col(&bid, "reschedule_token").await.unwrap();
    let refused = crate::i18n::translate("en", "ton-bae-body-reschedule-unpaid", None);
    let (_, _, page) = e.get_page(&format!("/booking/reschedule/{token}")).await;
    assert!(page.contains(&refused));
    let resp = e
        .post_guest(
            &format!("/booking/reschedule/{token}"),
            &format!("&date={}&time=11:00&tz=UTC", far_future_monday()),
        )
        .await;
    assert!(unescape(&body_string(resp).await).contains(&refused));
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
}

#[tokio::test]
async fn a_paid_booking_moved_by_its_guest_stays_confirmed() {
    let e = env().await;
    // Even when the type asks for approval.
    sqlx::query("UPDATE event_types SET requires_confirmation = 1")
        .execute(&e.pool)
        .await
        .unwrap();
    let (bid, _) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    let inv = e.inv(&bid).await;
    e.pay(AMOUNT, &code(&inv.reference));
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    let token = e.booking_col(&bid, "reschedule_token").await.unwrap();
    let resp = e
        .post_guest(
            &format!("/booking/reschedule/{token}"),
            &format!("&date={}&time=11:00&tz=UTC", far_future_monday()),
        )
        .await;
    assert!(resp.status().is_success() || resp.status().is_redirection());
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    assert!(e
        .booking_col(&bid, "start_at")
        .await
        .unwrap()
        .contains("T11:00"));
    assert_eq!(e.inv(&bid).await.status, "confirmed");
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ton_invoices")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 1, "no second invoice");
}

#[tokio::test]
async fn the_dashboard_waive_is_the_only_free_path() {
    let e = env().await;
    let session = e.session().await;
    let (bid, _) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    // The bookings page: a badge, "Confirm without payment", no Confirm.
    let resp = e
        .app()
        .oneshot(get_authed("/dashboard/bookings", &session))
        .await
        .unwrap();
    let page = unescape(&body_string(resp).await);
    assert!(page.contains("awaiting payment · 20.00 TON"), "{page}");
    assert!(page.contains(&format!("/dashboard/ton/waive/{bid}")));
    assert!(!page.contains(&format!("/dashboard/bookings/{bid}/confirm")));
    // The confirmation page says it is not paid.
    let resp = e
        .app()
        .oneshot(get_authed(&format!("/dashboard/ton/waive/{bid}"), &session))
        .await
        .unwrap();
    let page = body_string(resp).await;
    assert!(page.contains("NOT paid (20.00 TON"), "{page}");
    // CSRF is required.
    let resp = e
        .app()
        .oneshot(post_form(
            &format!("/dashboard/ton/waive/{bid}"),
            &session,
            CSRF,
            "_csrf=wrong",
        ))
        .await
        .unwrap();
    assert!(!resp.status().is_redirection());
    assert_eq!(e.inv(&bid).await.status, "open");
    // Another user cannot waive it.
    let (_, other) = insert_test_user_with_session(&e.pool, "O", "o@test.com", "other").await;
    let resp = e
        .post_authed(&format!("/dashboard/ton/waive/{bid}"), &other)
        .await;
    assert_eq!(location(&resp), "/dashboard/bookings?error=waive-failed");
    assert_eq!(e.inv(&bid).await.status, "open");
    // The owner can.
    e.post_authed(&format!("/dashboard/ton/waive/{bid}"), &session)
        .await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    let (status, by): (String, Option<String>) =
        sqlx::query_as("SELECT status, waived_by FROM ton_invoices WHERE booking_id = ?")
            .bind(&bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!(status, "waived");
    assert!(by.unwrap().starts_with("dashboard:"));
    // Not twice.
    let resp = e
        .post_authed(&format!("/dashboard/ton/waive/{bid}"), &session)
        .await;
    assert_eq!(location(&resp), "/dashboard/bookings?error=waive-failed");
    // The watcher leaves a waived invoice alone.
    e.step(e.now).await;
    assert_eq!(e.inv(&bid).await.status, "waived");
}

#[tokio::test]
async fn a_guest_cancelling_an_unpaid_booking_tells_nobody() {
    let e = env().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (bid, _) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    let token = e.booking_col(&bid, "cancel_token").await.unwrap();
    let resp = e.post_guest(&format!("/booking/cancel/{token}"), "").await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    assert_eq!(booking_status(&e.pool, &bid).await, "cancelled");
    assert_eq!(e.inv(&bid).await.status, "closed");
    assert!(settled_mails_to(&mails, "guest@test.com").await.is_empty());
    assert!(settled_mails_to(&mails, "test@example.com")
        .await
        .is_empty());
}

#[tokio::test]
async fn the_host_declining_a_paid_booking_flags_a_refund_at_once() {
    let e = env().await;
    let session = e.session().await;
    let (bid, _) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    let inv = e.inv(&bid).await;
    e.pay(AMOUNT, &code(&inv.reference));
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    e.post_authed(&format!("/dashboard/bookings/{bid}/cancel"), &session)
        .await;
    assert_eq!(booking_status(&e.pool, &bid).await, "cancelled");
    assert_eq!(e.inv(&bid).await.problem.as_deref(), Some("refund_due"));
}

#[tokio::test]
async fn an_unpaid_booking_expires_quietly_unless_the_guest_asked_for_the_link() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let _guard = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", "https://cal.example.com");
    let e = env().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (quiet, _) = e.book_http("quiet@test.com", "10:00", "198.51.100.1").await;
    let (asked, token) = e.book_http("asked@test.com", "11:00", "198.51.100.2").await;

    // The guest asks for the link: one email, with no amount, address or coin.
    let resp = e
        .post_guest(&format!("/booking/pay/{token}/email"), "")
        .await;
    assert_eq!(location(&resp), format!("/booking/pay/{token}?sent=1"));
    let link_mail = mail_to(&mails, "asked@test.com").await;
    assert!(link_mail.contains(&format!("https://cal.example.com/booking/pay/{token}")));
    assert!(link_mail.contains("https://cal.example.com/booking/cancel/"));
    for word in ["20.00", "UQCD", "TON"] {
        assert!(!link_mail.contains(word), "{word}");
    }
    // Once per invoice.
    let resp = e
        .post_guest(&format!("/booking/pay/{token}/email"), "")
        .await;
    assert_eq!(location(&resp), format!("/booking/pay/{token}?sent=0"));
    assert_eq!(settled_mails_to(&mails, "asked@test.com").await.len(), 1);

    // No payment: the watcher expires both once caught up past settle_by.
    let settle_by = e.inv(&asked).await.settle_by;
    e.chain
        .height((settle_by + Duration::minutes(4)).timestamp() as u64);
    let r = e.step(settle_by + Duration::minutes(4)).await;
    assert_eq!(r.expired, 2);
    sweep_pending_requests(&e.pool, &[0u8; 32], settle_by + Duration::minutes(5)).await;
    drain_emails(std::time::Duration::from_secs(5)).await;
    assert_eq!(booking_status(&e.pool, &quiet).await, "declined");
    assert_eq!(booking_status(&e.pool, &asked).await, "declined");
    // No host email, no upstream "request expired".
    assert!(settled_mails_to(&mails, "test@example.com")
        .await
        .is_empty());
    assert!(settled_mails_to(&mails, "quiet@test.com").await.is_empty());
    let asked_mails = settled_mails_to(&mails, "asked@test.com").await;
    assert_eq!(asked_mails.len(), 2);
    assert!(asked_mails[1].contains("did not arrive in time"));
    assert!(asked_mails[1].contains("https://cal.example.com/u/testuser/test-meeting"));
    // The page says so.
    let (_, _, page) = e.get_page(&format!("/booking/pay/{token}")).await;
    assert!(page.contains(&crate::i18n::translate("en", "ton-pay-state-expired", None)));
}

#[tokio::test]
async fn i_have_paid_wakes_the_watcher_at_most_every_ten_seconds() {
    let e = env().await;
    let (_, token) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    // Without CSRF: refused.
    let resp = e
        .app()
        .oneshot(post_bare(&format!("/booking/pay/{token}/check")))
        .await
        .unwrap();
    assert!(!resp.status().is_redirection());
    let resp = e
        .post_guest(&format!("/booking/pay/{token}/check"), "")
        .await;
    assert_eq!(location(&resp), format!("/booking/pay/{token}?checked=1"));
    // The wake is stored as a permit: it is there now.
    tokio::time::timeout(std::time::Duration::from_millis(200), e.rt.wake.notified())
        .await
        .expect("woken");
    // A second press within 10 s does not wake it again.
    e.post_guest(&format!("/booking/pay/{token}/check"), "")
        .await;
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(200), e.rt.wake.notified())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn the_price_is_set_on_the_event_type_form() {
    let e = env().await;
    let session = e.session().await;
    sqlx::query("DELETE FROM ton_prices")
        .execute(&e.pool)
        .await
        .unwrap();
    let edit = "/dashboard/event-types/test-meeting/edit";
    let resp = e.app().oneshot(get_authed(edit, &session)).await.unwrap();
    let page = body_string(resp).await;
    assert!(page.contains("name=\"ton_price_amount\""));
    assert!(
        page.contains("<option value=\"TON\" selected>"),
        "TON is the default"
    );
    let post = |extra: &str| {
        format!("_csrf={CSRF}&title=Test+Meeting&slug=test-meeting&duration_min=30&location_value=https%3A%2F%2Fmeet.example.com&avail_days=1,2,3,4,5&avail_start=09:00&avail_end=17:00{extra}")
    };
    // Invalid: refused, nothing stored.
    let resp = e
        .app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_enabled=on&ton_price_currency=USD&ton_price_amount=0.5"),
        ))
        .await
        .unwrap();
    let page = body_string(resp).await;
    assert!(page.contains(&crate::i18n::translate("en", "ton-form-error-price", None)));
    assert!(
        page.contains("value=\"0.5\""),
        "the posted price is echoed back"
    );
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ton_prices")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 0);
    // Valid.
    let resp = e
        .app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_enabled=on&ton_price_currency=USD&ton_price_amount=40.50&ton_window_min=20"),
        ))
        .await
        .unwrap();
    let status = resp.status();
    let page = body_string(resp).await;
    assert!(
        status.is_redirection(),
        "{status} {}",
        page.split("class=\"error").nth(1).unwrap_or(&page)
    );
    let row: (i64, Option<i64>, i64) = sqlx::query_as(
        "SELECT price_usd_cents, window_min, enabled FROM ton_prices WHERE event_type_id = ?",
    )
    .bind(&e.et_id)
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(row, (4050, Some(20), 1));
    let resp = e.app().oneshot(get_authed(edit, &session)).await.unwrap();
    assert!(body_string(resp).await.contains("value=\"40.50\""));
    // Unchecked: kept, off. Then cleared: removed.
    e.app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_currency=USD&ton_price_amount=40.50"),
        ))
        .await
        .unwrap();
    let enabled: i64 = sqlx::query_scalar("SELECT enabled FROM ton_prices")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(enabled, 0);
    assert!(!invoice::is_priced(&e.pool, &e.et_id).await);
    e.app()
        .oneshot(post_form(edit, &session, CSRF, &post("&ton_price_amount=")))
        .await
        .unwrap();
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ton_prices")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 0);
    // A form without the section changes nothing.
    set_price(&e.pool, &e.et_id, 4000).await;
    e.app()
        .oneshot(post_form(edit, &session, CSRF, &post("")))
        .await
        .unwrap();
    assert!(invoice::is_priced(&e.pool, &e.et_id).await);
}

#[tokio::test]
async fn slot_and_form_pages_show_the_price() {
    let e = env().await;
    // The first step warmed the rate: $40 at $2 is about 20 TON.
    let (_, _, page) = e.get_page("/u/testuser/test-meeting").await;
    assert!(page.contains("Price: $40.00"), "{page}");
    assert!(page.contains("(≈ 20.00 TON)"));
    let (_, _, page) = e
        .get_page(&format!(
            "/u/testuser/test-meeting/book?date={}&time=10:00",
            far_future_monday()
        ))
        .await;
    assert!(page.contains("Price: $40.00"));
    // A free type shows nothing.
    sqlx::query("UPDATE ton_prices SET enabled = 0")
        .execute(&e.pool)
        .await
        .unwrap();
    let (_, _, page) = e.get_page("/u/testuser/test-meeting").await;
    assert!(!page.contains("Price: $"));
}

#[tokio::test]
async fn the_owner_is_emailed_about_payments_without_the_senders_comment() {
    let e = env().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (bid, _) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    let inv = e.inv(&bid).await;
    // A partial payment, then the rest.
    e.pay(AMOUNT / 2, &code(&inv.reference));
    e.step(e.now).await;
    e.pay(AMOUNT / 2, &code(&inv.reference));
    e.step(e.now).await;
    // An unmatched transfer carrying a phishing comment, above the threshold.
    e.pay(AMOUNT, "your wallet is hacked, send everything to EQ-evil");
    e.step(e.now).await;
    for alert in e.sink.all() {
        crate::web::ton_pay::alerts::deliver_now(&e.pool, &[0u8; 32], alert).await;
    }
    let owner = settled_mails_to(&mails, "test@example.com").await;
    // Partial, received (confirmed), unmatched (to the admin, who is the
    // owner here), plus the upstream "booking confirmed".
    let ton_mails: Vec<&String> = owner.iter().filter(|m| m.contains(" TON")).collect();
    assert!(ton_mails.len() >= 3, "{owner:?}");
    assert!(owner.iter().any(|m| m.contains("Payment received")));
    assert!(owner.iter().any(|m| m.contains("partial payment")));
    assert!(owner.iter().any(|m| m.contains("unrecognised payment")));
    for m in &owner {
        assert!(!m.contains("hacked"), "a sender's comment reached an email");
    }
}

// --- follow-up fixes (review findings) ---------------------------------------

/// Book and cancel at once, over and over: free for the caller, so it must
/// not use up everybody's daily budget. Each client network has its own
/// daily count, cancelled bookings included.
#[tokio::test]
async fn cancelled_bookings_do_not_use_up_the_day_and_each_network_has_its_own_cap() {
    let e = env_with(&[
        ("CALRS_TON_MAX_PER_DAY", "2"),
        ("CALRS_TON_MAX_OPEN_TOTAL", "10"),
        ("CALRS_TON_MAX_PER_IP_DAY", "3"),
    ])
    .await;
    let start = e.now + Duration::days(3);
    let er = &e;
    let cancel = move |bid: String| async move {
        sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
            .bind(&bid)
            .execute(&er.pool)
            .await
            .unwrap();
        invoice::on_booking_ended(&er.pool, &bid, er.now).await;
    };
    // Five book-and-cancel rounds from five networks: more than the day's
    // budget of two, and none of them counts.
    for i in 0..5 {
        let (bid, _) = e
            .booking_at(
                &format!("g{i}@x.io"),
                start + Duration::hours(i),
                &format!("10.0.{i}.1"),
            )
            .await
            .unwrap();
        cancel(bid).await;
    }
    // Two real bookings, open long enough to count, fill the day.
    e.booking_at("a@x.io", start + Duration::hours(10), "10.1.0.1")
        .await
        .unwrap();
    e.booking_at("b@x.io", start + Duration::hours(11), "10.1.1.1")
        .await
        .unwrap();
    e.age_invoices(Duration::minutes(11)).await;
    assert_eq!(
        e.booking_at("c@x.io", start + Duration::hours(12), "10.1.2.1")
            .await
            .err(),
        Some(Refusal::Capacity)
    );
    // A quick cancel with money on it counts: it was real.
    let e2 = env_with(&[
        ("CALRS_TON_MAX_PER_DAY", "1"),
        ("CALRS_TON_MAX_OPEN_TOTAL", "10"),
    ])
    .await;
    let (bid, opened) = e2.booking("paid-part@x.io").await;
    e2.pay(AMOUNT / 4, &opened.reference);
    e2.step(e2.now).await;
    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
        .bind(&bid)
        .execute(&e2.pool)
        .await
        .unwrap();
    invoice::on_booking_ended(&e2.pool, &bid, e2.now).await;
    assert_eq!(
        e2.booking_at("next@x.io", e2.now + Duration::days(4), "10.7.0.1")
            .await
            .err(),
        Some(Refusal::Capacity)
    );

    // One network, three rounds today (each cancelled, other addresses in
    // the same /24): the fourth is refused, another network is not.
    let e3 = env_with(&[("CALRS_TON_MAX_PER_IP_DAY", "3")]).await;
    for i in 0..3 {
        let (bid, _) = e3
            .booking_at(
                &format!("n{i}@x.io"),
                e3.now + Duration::days(3) + Duration::hours(i),
                &format!("198.51.100.{}", i + 10),
            )
            .await
            .unwrap();
        sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
            .bind(&bid)
            .execute(&e3.pool)
            .await
            .unwrap();
        invoice::on_booking_ended(&e3.pool, &bid, e3.now).await;
    }
    assert_eq!(
        e3.booking_at(
            "n9@x.io",
            e3.now + Duration::days(3) + Duration::hours(9),
            "198.51.100.200"
        )
        .await
        .err(),
        Some(Refusal::ClientLimit)
    );
    e3.booking_at(
        "n9@x.io",
        e3.now + Duration::days(3) + Duration::hours(9),
        "198.51.101.1",
    )
    .await
    .unwrap();
}

/// The owner hears that the last open place was taken before anyone is
/// refused.
#[tokio::test]
async fn the_owner_is_paged_when_the_last_open_place_is_taken() {
    let e = env_with(&[("CALRS_TON_MAX_OPEN_TOTAL", "2")]).await;
    e.booking("one@x.io").await;
    assert!(!e.sink.health_kinds().contains(&"capacity_near".to_string()));
    e.booking_at("two@x.io", e.now + Duration::days(4), "10.2.0.1")
        .await
        .unwrap();
    assert!(e.sink.health_kinds().contains(&"capacity_near".to_string()));
    assert!(!e.sink.health_kinds().contains(&"capacity".to_string()));
}

/// Only a busy host is final. Any other failure to confirm a paid booking
/// (here, a resource read that errors) is retried, and after a few tries
/// left to the owner with the time held.
#[tokio::test]
async fn a_failed_confirmation_is_retried_and_only_a_busy_host_is_final() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let start: String = sqlx::query_scalar("SELECT start_at FROM bookings WHERE id = ?")
        .bind(&bid)
        .fetch_one(&e.pool)
        .await
        .unwrap();
    let break_reads = |on: bool| {
        let pool = e.pool.clone();
        async move {
            let sql = if on {
                "ALTER TABLE event_type_resources RENAME TO etr_broken"
            } else {
                "ALTER TABLE etr_broken RENAME TO event_type_resources"
            };
            sqlx::query(sql).execute(&pool).await.unwrap();
        }
    };
    e.pay(AMOUNT, &opened.reference);
    break_reads(true).await;
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    let inv = e.inv(&bid).await;
    assert_eq!((inv.status.as_str(), inv.problem), ("paid", None));
    // Retried once the error is gone.
    break_reads(false).await;
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    assert_eq!(e.inv(&bid).await.status, "confirmed");
    assert!(e.sink.problems().is_empty());

    // An error that does not go away: given up after a few tries.
    let (bid, opened) = e.booking("guest2@example.com").await;
    let start2: String = sqlx::query_scalar("SELECT start_at FROM bookings WHERE id = ?")
        .bind(&bid)
        .fetch_one(&e.pool)
        .await
        .unwrap();
    e.pay(AMOUNT, &opened.reference);
    break_reads(true).await;
    for _ in 0..watcher::MAX_CONFIRM_ATTEMPTS {
        e.step(e.now).await;
    }
    let inv = e.inv(&bid).await;
    assert_eq!(
        (inv.status.as_str(), inv.problem.as_deref()),
        ("paid", Some("confirm_failed"))
    );
    assert_eq!(pending_expires_at(&e.pool, &bid).await, Some(start2));
    assert_eq!(e.sink.problems(), vec!["confirm_failed".to_string()]);
    assert_ne!(start, String::new());
    // The bookings page says what to do, and offers Confirm (it is paid).
    let session = e.session().await;
    let resp = e
        .app()
        .oneshot(get_authed("/dashboard/bookings", &session))
        .await
        .unwrap();
    let page = unescape(&body_string(resp).await);
    assert!(page.contains("paid, not confirmed"), "{page}");
    assert!(page.contains(&format!("/dashboard/bookings/{bid}/confirm")));
}

/// Transfers below the record threshold that carry a known code are kept
/// but email no one; past one email an hour per invoice, the rest go into
/// the hourly digest.
#[tokio::test]
async fn transfers_with_a_known_code_cannot_flood_the_owner() {
    let e = env().await;
    let (bid, opened) = e.booking("guest@example.com").await;
    // Dust: credited, recorded, no email.
    e.pay(1, &opened.reference);
    e.pay(50_000_000, &opened.reference);
    // Three real partial payments in a row.
    for _ in 0..3 {
        e.pay(1_000_000_000, &opened.reference);
    }
    e.step(e.now).await;
    assert_eq!(e.payments().await.len(), 5);
    assert_eq!(e.inv(&bid).await.paid, 3_050_000_001);
    assert_eq!(e.sink.payment_kinds(), vec!["partial".to_string()]);
    assert_eq!(
        e.sink
            .health_kinds()
            .iter()
            .filter(|k| *k == "payment_alerts_held")
            .count(),
        1
    );
    // More of them in the same hour: no more emails at all.
    for _ in 0..3 {
        e.pay(1_000_000_000, &opened.reference);
    }
    e.step(e.now).await;
    assert_eq!(e.sink.payment_kinds().len(), 1);
    assert_eq!(
        e.sink
            .health_kinds()
            .iter()
            .filter(|k| *k == "payment_alerts_held")
            .count(),
        1
    );
    // Late dust with an old code: kept, no email.
    sqlx::query("UPDATE ton_invoices SET status = 'expired' WHERE booking_id = ?")
        .bind(&bid)
        .execute(&e.pool)
        .await
        .unwrap();
    e.pay(1, &opened.reference);
    e.step(e.now).await;
    assert_eq!(e.payments().await.last().unwrap().0, "late");
    assert_eq!(e.sink.payment_kinds().len(), 1);
}

/// The host approved a request over an unpaid priced booking (or a
/// calendar event appeared): the invoice is released before anyone pays
/// for a time that is gone.
#[tokio::test]
async fn an_unpaid_booking_whose_time_was_taken_is_released_before_payment() {
    let e = env().await;
    let (bid, _) = e.booking("guest@example.com").await;
    let start: String = sqlx::query_scalar("SELECT start_at FROM bookings WHERE id = ?")
        .bind(&bid)
        .fetch_one(&e.pool)
        .await
        .unwrap();
    // Untouched while the host is free.
    let r = e.step(e.now).await;
    assert_eq!(r.released, 0);
    let other_et = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO event_types (id, account_id, slug, title, duration_min, buffer_before, buffer_after, min_notice_min, enabled) VALUES (?, ?, 'free-req', 'Free request', 30, 0, 0, 0, 1)")
        .bind(&other_et)
        .bind(&e.account_id)
        .execute(&e.pool)
        .await
        .unwrap();
    insert_v1_booking(
        &e.pool,
        &other_et,
        "approved@example.com",
        crate::web::ton_pay::parse_stamp(&start).unwrap(),
        30,
        "confirmed",
        &[],
    )
    .await;
    let r = e.step(e.now).await;
    assert_eq!(r.released, 1);
    let inv = e.inv(&bid).await;
    assert_eq!(
        (inv.status.as_str(), inv.problem.as_deref()),
        ("closed", Some("host_busy_unpaid"))
    );
    assert_eq!(
        pending_expires_at(&e.pool, &bid).await,
        Some(crate::web::ton_pay::stamp(e.now))
    );
    assert_eq!(e.sink.problems(), vec!["host_busy_unpaid".to_string()]);
    // The guest's page says the time is gone; nothing to pay.
    let token: String =
        sqlx::query_scalar("SELECT pay_token FROM ton_invoices WHERE booking_id = ?")
            .bind(&bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    let (_, _, page) = e.get_page(&format!("/booking/pay/{token}")).await;
    assert!(page.contains(&crate::i18n::translate("en", "ton-pay-state-taken", None)));
    assert!(!page.contains("ton://transfer"));
    assert!(!page.contains("app.tonkeeper.com"));
}

/// A part paid, then the booking ends unconfirmed: the owner is told the
/// part has to go back, and the guest's cancel is not a quiet one.
#[tokio::test]
async fn a_partial_payment_on_an_ended_booking_is_flagged_for_a_refund() {
    let e = env().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    // Guest cancel through the link.
    let (bid, _) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    let inv = e.inv(&bid).await;
    e.pay(AMOUNT / 2, &code(&inv.reference));
    e.step(e.now).await;
    assert!(!invoice::cancel_is_quiet(&e.pool, &bid).await);
    let token = e.booking_col(&bid, "cancel_token").await.unwrap();
    e.post_guest(&format!("/booking/cancel/{token}"), "").await;
    let inv = e.inv(&bid).await;
    assert_eq!(
        (inv.status.as_str(), inv.problem.as_deref()),
        ("closed", Some("partial"))
    );
    e.step(e.now).await;
    assert_eq!(e.sink.problems(), vec!["partial".to_string()]);
    assert!(!settled_mails_to(&mails, "test@example.com")
        .await
        .is_empty());
    // The page tells the guest the money goes back.
    let pay: String = sqlx::query_scalar("SELECT pay_token FROM ton_invoices WHERE booking_id = ?")
        .bind(&bid)
        .fetch_one(&e.pool)
        .await
        .unwrap();
    let (_, _, page) = e.get_page(&format!("/booking/pay/{pay}")).await;
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("paid", "10.00");
    assert!(page.contains(&crate::i18n::translate(
        "en",
        "ton-pay-state-refund",
        Some(&args)
    )));
    assert!(page.contains("test@example.com"), "the host's contact");

    // Declined by the emailed link: the watcher's reconcile does the same.
    let (bid, opened) = e.booking("declined@test.com").await;
    e.pay(AMOUNT / 4, &opened.reference);
    e.step(e.now).await;
    sqlx::query("UPDATE bookings SET status = 'declined' WHERE id = ?")
        .bind(&bid)
        .execute(&e.pool)
        .await
        .unwrap();
    e.step(e.now).await;
    assert_eq!(e.inv(&bid).await.problem.as_deref(), Some("partial"));

    // Waived with a part paid: the page says what arrived, and the part is
    // flagged.
    let session = e.session().await;
    let (bid, opened) = e.booking("waived@test.com").await;
    e.pay(AMOUNT / 4, &opened.reference);
    e.step(e.now).await;
    let resp = e
        .app()
        .oneshot(get_authed(&format!("/dashboard/ton/waive/{bid}"), &session))
        .await
        .unwrap();
    let page = body_string(resp).await;
    assert!(page.contains("5.00 of 20.00 TON received"), "{page}");
    assert!(page.contains("remaining 15.00 TON"), "{page}");
    assert!(invoice::waive(&e.pool, &bid, "test", e.now).await.unwrap());
    assert_eq!(e.inv(&bid).await.problem.as_deref(), Some("partial"));
}

/// A price switched on after a quiet day: the watcher is woken, so the
/// first guest is not refused for up to an hour.
#[tokio::test]
async fn switching_a_price_on_wakes_the_watcher() {
    let e = env().await;
    let drain = || async {
        let _ =
            tokio::time::timeout(std::time::Duration::from_millis(20), e.rt.wake.notified()).await;
    };
    drain().await;
    let form = crate::web::ton_pay::PriceForm {
        enabled: Some("on"),
        amount: Some("50"),
        currency: Some("USD"),
        ..Default::default()
    };
    crate::web::ton_pay::save_price_form(&e.state, &e.et_id, &form).await;
    tokio::time::timeout(std::time::Duration::from_millis(200), e.rt.wake.notified())
        .await
        .expect("a saved price wakes the watcher");
    // A refusal for a stale watcher wakes it too.
    drain().await;
    e.rt.with_health(|h| h.last_ok = Some(e.now - Duration::minutes(10)));
    let refused = invoice::prepare_invoice(
        &e.state,
        &e.et_id,
        e.now + Duration::days(3),
        0,
        "g@x.io",
        "10.0.0.1",
        e.now,
    )
    .await;
    assert!(matches!(refused, Err(Refusal::Unavailable(_))));
    tokio::time::timeout(std::time::Duration::from_millis(200), e.rt.wake.notified())
        .await
        .expect("a stale-watcher refusal wakes the watcher");
    // While any price row exists the idle watcher sleeps 15 minutes, not an
    // hour, even when the price is switched off (ilia fork, Optimize: the
    // heartbeat is made on demand, was a minute).
    let caught_up = watcher::StepReport {
        caught_up: true,
        ..Default::default()
    };
    sqlx::query("UPDATE ton_prices SET enabled = 0")
        .execute(&e.pool)
        .await
        .unwrap();
    let in_two_days = e.now + Duration::days(2);
    assert_eq!(
        watcher::next_delay(&e.pool, &e.rt, &caught_up, in_two_days).await,
        watcher::IDLE_PRICED
    );
    sqlx::query("DELETE FROM ton_prices")
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(
        watcher::next_delay(&e.pool, &e.rt, &caught_up, in_two_days).await,
        std::time::Duration::from_secs(3600)
    );
}

/// The sweep drops a pending booking that was paid (its time was taken)
/// or waived: the paying guest hears the money goes back; the waived one
/// gets upstream's ordinary "request expired".
#[tokio::test]
async fn a_paid_or_waived_booking_dropped_by_the_sweep_tells_the_guest() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let e = env().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    // Paid, host busy, held until the start, then dropped.
    let (paid, opened) = e.booking("paid@test.com").await;
    let start: String = sqlx::query_scalar("SELECT start_at FROM bookings WHERE id = ?")
        .bind(&paid)
        .fetch_one(&e.pool)
        .await
        .unwrap();
    e.pay(AMOUNT, &opened.reference);
    sqlx::query("UPDATE ton_invoices SET status = 'paid', paid_nano = ?, problem = 'host_busy' WHERE booking_id = ?")
        .bind(AMOUNT as i64)
        .bind(&paid)
        .execute(&e.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE bookings SET pending_expires_at = ? WHERE id = ?")
        .bind(&start)
        .bind(&paid)
        .execute(&e.pool)
        .await
        .unwrap();
    // Waived, the confirmation refused, so still pending.
    let (waived, _) = e.booking("waived@test.com").await;
    assert!(invoice::waive(&e.pool, &waived, "test", e.now)
        .await
        .unwrap());
    sqlx::query("UPDATE bookings SET pending_expires_at = ? WHERE id = ?")
        .bind(crate::web::ton_pay::stamp(e.now))
        .bind(&waived)
        .execute(&e.pool)
        .await
        .unwrap();
    let after = crate::web::ton_pay::parse_stamp(&start).unwrap() + Duration::minutes(1);
    sweep_pending_requests(&e.pool, &[0u8; 32], after).await;
    drain_emails(std::time::Duration::from_secs(5)).await;
    assert_eq!(booking_status(&e.pool, &paid).await, "declined");
    assert_eq!(booking_status(&e.pool, &waived).await, "declined");
    let to_paid = settled_mails_to(&mails, "paid@test.com").await;
    assert_eq!(to_paid.len(), 1, "{to_paid:?}");
    assert!(
        to_paid[0].contains("will refund the payment"),
        "{}",
        to_paid[0]
    );
    assert!(!settled_mails_to(&mails, "waived@test.com").await.is_empty());
}

/// At shutdown a step confirms nothing: the approval writes calendars and
/// sends emails, which an abort must not cut in half. The next step does.
#[tokio::test]
async fn a_stopping_step_leaves_confirmation_to_the_next_start() {
    let e = env().await;
    let (bid, _) = e.booking("guest@example.com").await;
    sqlx::query(
        "UPDATE ton_invoices SET status = 'paid', paid_nano = amount_nano WHERE booking_id = ?",
    )
    .bind(&bid)
    .execute(&e.pool)
    .await
    .unwrap();
    let (_tx, stop) = tokio::sync::watch::channel(true);
    let r = watcher::step_until(&e.state, &e.rt, e.now, &stop).await;
    assert_eq!(r.confirmed, 0);
    assert!(!r.caught_up);
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    let r = e.step(e.now).await;
    assert_eq!(r.confirmed, 1);
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
}

/// A failed invoice read refuses an approval without calling a free
/// booking unpaid, and does not swallow a free booking's expiry notices.
#[tokio::test]
async fn a_failed_invoice_read_does_not_call_a_free_booking_unpaid() {
    let e = env().await;
    let free_et = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO event_types (id, account_id, slug, title, duration_min, buffer_before, buffer_after, min_notice_min, enabled, requires_confirmation) VALUES (?, ?, 'free', 'Free', 30, 0, 0, 0, 1, 1)")
        .bind(&free_et)
        .bind(&e.account_id)
        .execute(&e.pool)
        .await
        .unwrap();
    let bid = insert_v1_booking(
        &e.pool,
        &free_et,
        "free@test.com",
        e.now + Duration::days(5),
        30,
        "pending",
        &[],
    )
    .await;
    sqlx::query("ALTER TABLE ton_invoices RENAME TO ton_invoices_broken")
        .execute(&e.pool)
        .await
        .unwrap();
    let resp = e
        .post_guest(&format!("/booking/approve/confirm-{bid}"), "")
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    let body = body_string(resp).await;
    assert!(!body.contains("not been paid"), "{body}");
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    // Expiry: a type that has never been priced gets upstream's notices.
    assert!(!crate::web::ton_pay::handle_expired_notice(&e.pool, None, &bid).await);
    // A priced type's booking stays silent rather than get the wrong one.
    let priced = insert_v1_booking(
        &e.pool,
        &e.et_id,
        "priced@test.com",
        e.now + Duration::days(5),
        30,
        "pending",
        &[],
    )
    .await;
    assert!(crate::web::ton_pay::handle_expired_notice(&e.pool, None, &priced).await);
}

/// The guest cancelling an unpaid booking gets no SMS either; with a part
/// paid the usual notices (SMS included) go out.
#[tokio::test]
async fn a_quiet_cancel_sends_no_sms() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let e = env().await;
    let (port, received) = super::emails::spawn_sms_webhook().await;
    super::emails::use_sms_webhook(&e.pool, port).await;
    sqlx::query("UPDATE event_types SET sms_phone_mode = 'optional' WHERE id = ?")
        .bind(&e.et_id)
        .execute(&e.pool)
        .await
        .unwrap();
    let er = &e;
    let cancel = move |bid: String| async move {
        sqlx::query("UPDATE bookings SET guest_phone = '+33612345678' WHERE id = ?")
            .bind(&bid)
            .execute(&er.pool)
            .await
            .unwrap();
        let token = er.booking_col(&bid, "cancel_token").await.unwrap();
        let resp = er.post_guest(&format!("/booking/cancel/{token}"), "").await;
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        assert_eq!(booking_status(&er.pool, &bid).await, "cancelled");
    };
    let (unpaid, _) = e.book_http("quiet@test.com", "10:00", "198.51.100.1").await;
    cancel(unpaid).await;
    assert!(
        received.lock().unwrap().is_empty(),
        "no SMS for a quiet cancel"
    );
    let (part, _) = e.book_http("part@test.com", "11:00", "198.51.101.1").await;
    let inv = e.inv(&part).await;
    e.pay(AMOUNT / 2, &code(&inv.reference));
    e.step(e.now).await;
    cancel(part).await;
    assert_eq!(received.lock().unwrap().len(), 1, "the control is texted");
}

/// Duplicating a priced personal type keeps its price.
#[tokio::test]
async fn duplicating_a_priced_type_keeps_the_price() {
    let e = env().await;
    let session = e.session().await;
    sqlx::query("UPDATE ton_prices SET window_min = 20")
        .execute(&e.pool)
        .await
        .unwrap();
    let resp = e
        .post_authed("/dashboard/event-types/test-meeting/duplicate", &session)
        .await;
    assert!(resp.status().is_redirection());
    let copies: Vec<(String, i64, Option<i64>, i64)> = sqlx::query_as(
        "SELECT event_type_id, price_usd_cents, window_min, enabled FROM ton_prices WHERE event_type_id != ?",
    )
    .bind(&e.et_id)
    .fetch_all(&e.pool)
    .await
    .unwrap();
    assert_eq!(copies.len(), 1);
    assert_eq!((copies[0].1, copies[0].2, copies[0].3), (4000, Some(20), 1));
    assert_ne!(copies[0].0, e.et_id);
}

/// "You already have an unpaid booking": the page offers to email that
/// booking's links to its address, which is the way to pay or cancel it.
/// The answer says nothing about whether such a booking exists.
#[tokio::test]
async fn an_open_booking_blocks_another_but_its_links_can_be_emailed() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let _guard = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", "https://cal.example.com");
    let e = env().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (bid, token) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    let resp = e
        .post_booking(
            "/u/testuser/test-meeting/book",
            "guest@test.com",
            "11:00",
            "198.51.101.1",
        )
        .await;
    let page = unescape(&body_string(resp).await);
    assert!(page.contains("action=\"/booking/pay/resend\""), "{page}");
    assert!(page.contains("value=\"guest@test.com\""));
    assert!(page.contains(&crate::i18n::translate("en", "ton-resend-submit", None)));
    // Without CSRF: refused.
    let resp = e
        .app()
        .oneshot(post_bare("/booking/pay/resend"))
        .await
        .unwrap();
    assert!(!resp.status().is_success());
    // The links go to that address.
    let done = crate::i18n::translate("en", "ton-resend-done", None);
    let resp = e
        .post_guest("/booking/pay/resend", "&email=Guest%40Test.com")
        .await;
    assert!(unescape(&body_string(resp).await).contains(&done));
    let mail = mail_to(&mails, "guest@test.com").await;
    assert!(mail.contains(&format!("https://cal.example.com/booking/pay/{token}")));
    assert!(mail.contains("https://cal.example.com/booking/cancel/"));
    assert!(mail.contains(&crate::i18n::translate("en", "ton-email-row-pay-by", None)));
    assert!(e.inv(&bid).await.status == "open");
    // Not again within ten minutes; an unknown address gets the same page.
    e.post_guest("/booking/pay/resend", "&email=guest%40test.com")
        .await;
    let resp = e
        .post_guest("/booking/pay/resend", "&email=nobody%40test.com")
        .await;
    assert!(unescape(&body_string(resp).await).contains(&done));
    drain_emails(std::time::Duration::from_secs(5)).await;
    assert_eq!(settled_mails_to(&mails, "guest@test.com").await.len(), 1);
    assert!(settled_mails_to(&mails, "nobody@test.com").await.is_empty());
}

/// What the payment page shows: the remainder in USD after a partial
/// payment, copy buttons with their own names, "Email me this link" only
/// when email can go out, and the host's contact once the guest may need
/// it.
#[tokio::test]
async fn the_payment_page_shows_what_the_guest_needs() {
    let e = env().await;
    let (bid, token) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    let (_, _, page) = e.get_page(&format!("/booking/pay/{token}")).await;
    assert!(page.contains("≈ $40.00"));
    assert!(page.contains(&format!(
        "aria-label=\"{}\"",
        crate::i18n::translate("en", "ton-pay-copy-address", None)
    )));
    assert!(page.contains(&format!(
        "aria-label=\"{}\"",
        crate::i18n::translate("en", "ton-pay-copy-comment", None)
    )));
    // No SMTP here: no button that can only fail.
    assert!(!page.contains(&crate::i18n::translate("en", "ton-pay-email", None)));
    assert!(!page.contains("Questions? Write to"));
    // Half paid: the rest, and what the rest is worth.
    let inv = e.inv(&bid).await;
    e.pay(AMOUNT / 2, &code(&inv.reference));
    e.step(e.now).await;
    let (_, _, page) = e.get_page(&format!("/booking/pay/{token}")).await;
    assert!(page.contains("10.00 TON"));
    assert!(page.contains("≈ $20.00"), "{page}");
    assert!(!page.contains("≈ $40.00"));
    // Expired: how to reach the host.
    sqlx::query("UPDATE ton_invoices SET status = 'expired' WHERE booking_id = ?")
        .bind(&bid)
        .execute(&e.pool)
        .await
        .unwrap();
    let (_, _, page) = e.get_page(&format!("/booking/pay/{token}")).await;
    assert!(
        page.contains("Questions? Write to Test User at test@example.com."),
        "{page}"
    );
}

/// A priced type's slot pages leave the time an invoice needs.
#[tokio::test]
async fn slot_pages_offer_no_time_too_close_to_pay_for() {
    let e = env().await;
    use crate::web::ton_pay::slot_min_notice;
    assert_eq!(slot_min_notice(&e.state, &e.et_id, 60).await, 90);
    assert_eq!(slot_min_notice(&e.state, &e.et_id, 0).await, 30);
    sqlx::query("UPDATE ton_prices SET enabled = 0")
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(slot_min_notice(&e.state, &e.et_id, 60).await, 60);
}

#[test]
fn owner_and_guest_texts_say_what_can_actually_be_done() {
    // A late payment's booking is already released: nothing to confirm.
    let late = crate::i18n::translate("en", "ton-alert-late-body", None);
    assert!(!late.contains("still free"), "{late}");
    // Russian: «время» once in the expiry state.
    let ru = crate::i18n::translate("ru", "ton-pay-state-expired", None);
    assert_eq!(ru.matches("ремя").count(), 1, "{ru}");
}
