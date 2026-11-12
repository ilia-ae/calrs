//! ilia fork: TON payments, the part that needs `crate::web` (ilia-booking
//! docs/ton.md): the runtime held in `AppState.ton`, invoices and their
//! guards, and the ton-watcher background task. The pure rules are in
//! `crate::ton`.
//!
//! The rule everything here keeps: a priced booking is confirmed only after
//! its payment is final on chain, or by an explicit waive. A bad setting, a
//! missing rate, an indexer that is down or on the wrong network makes a
//! priced type unbookable or lets an open booking run to its hard deadline;
//! none of these ever makes a booking free.

pub(crate) mod admin;
pub(crate) mod alerts;
pub(crate) mod flow;
pub(crate) mod invoice;
pub(crate) mod page;
pub(crate) mod store;
pub(crate) mod telegram;
pub(crate) mod watcher;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::ton::config::TonConfig;
use crate::ton::rate::{AcceptedRate, HttpRates, RateRefusal, RateSource};
use crate::ton::toncenter::{HttpIndexer, TonIndexer};
use crate::web::{AppState, RateLimiter};

pub(crate) use admin::{
    copy_price, dashboard_badges, price_badge, price_form_ctx, price_form_from_post, price_write,
    save_price_form, validate_price_form, write_price, PriceForm, PriceWrite,
};
pub(in crate::web) use flow::{
    after_booking, approval_refused_for_token, approval_refused_page, booking_ended,
    guest_reschedule_refused, handle_expired_notice, open_for_booking, prepare_for_booking,
    refuse_group,
};
pub(crate) use invoice::{approval_refused, cancel_is_quiet, is_priced, settled, unpaid};
pub(crate) use watcher::spawn;

/// The payment page, its status and actions, and the dashboard waive.
pub(crate) fn router() -> axum::Router<Arc<AppState>> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/booking/pay/{token}", get(page::pay_page))
        .route("/booking/pay/{token}/status", get(page::pay_status))
        .route("/booking/pay/{token}/check", post(page::pay_check))
        .route("/booking/pay/{token}/email", post(page::pay_email))
        .route("/booking/pay/resend", post(page::resend_links))
        .route(
            "/dashboard/ton/waive/{booking_id}",
            get(admin::waive_form).post(admin::waive),
        )
}

/// Marks the Approve handler's answer when the booking was refused because
/// its host (or a required resource) is taken, as opposed to an error: the
/// watcher then holds a paid booking for the owner instead of retrying.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ApprovalBusy;

/// The Approve handler's "host busy" and "resource gone" pages, marked.
pub(crate) fn mark_busy(mut resp: axum::response::Response) -> axum::response::Response {
    resp.extensions_mut().insert(ApprovalBusy);
    resp
}

/// Wake the watcher (a price was switched on or changed): it then keeps
/// its heartbeat fresh, so the first guest is not refused.
pub(crate) fn wake_watcher(state: &AppState) {
    if let Some(rt) = state.ton.get() {
        rt.wake.notify_one();
    }
}

/// The minimum notice a slot page applies: a priced type adds the time
/// an invoice needs before the meeting
/// ([`crate::ton::deadlines::min_lead_min`]), so no slot is offered that
/// the booking would refuse as too late. Unchanged for a free type.
pub(crate) async fn slot_min_notice(state: &AppState, event_type_id: &str, min_notice: i32) -> i32 {
    if !matches!(
        store::price_for(&state.pool, event_type_id).await,
        Ok(Some(_))
    ) {
        return min_notice;
    }
    let grace = state.ton.get().map(|rt| rt.cfg.grace_min).unwrap_or(15);
    let lead = i32::try_from(crate::ton::deadlines::min_lead_min(grace)).unwrap_or(i32::MAX);
    min_notice.max(0).saturating_add(lead)
}

/// Log target of everything TON.
pub(crate) const TARGET: &str = "calrs::ton";

/// The time format of every TON column: comparable as text, like
/// `bookings.pending_expires_at`.
pub(crate) fn stamp(t: DateTime<Utc>) -> String {
    t.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub(crate) fn parse_stamp(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

/// Something the owner must hear about. Delivery (email) is plugged in
/// through [`AlertSink`]; claims make each one go out once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TonAlert {
    /// One `ton_payments` row: `kind` is `partial`, `overpaid`, `late`,
    /// `unmatched` or an ignore reason (`failed`, `not_text`, …).
    Payment { tx_hex: String, kind: String },
    /// An invoice's `problem` was set or changed.
    InvoiceProblem { invoice_id: String, problem: String },
    /// An invoice reached `confirmed` with money on it.
    PaymentReceived { invoice_id: String },
    /// Watcher health, rate, capacity: at most once an hour per kind.
    Health { kind: String, detail: String },
    /// ilia fork: an unpaid window was renewed; the guest is asked to pay
    /// again by the new deadline (email and Telegram; not the owner).
    Renewed { invoice_id: String },
}

/// Where alerts go. Without one, payment and invoice alerts stay unclaimed
/// in the database (so nothing is lost) and health alerts are only logged.
#[async_trait]
pub(crate) trait AlertSink: Send + Sync {
    async fn deliver(&self, alert: TonAlert);
}

/// What the watcher last learned about the indexer and the wallet. "Payments
/// enabled" is derived from it ([`TonRuntime::ready`]).
#[derive(Clone, Debug, Default)]
pub(crate) struct Health {
    pub(crate) account_status: Option<String>,
    pub(crate) account_checked_at: Option<DateTime<Utc>>,
    pub(crate) network_ok: Option<bool>,
    pub(crate) last_ok: Option<DateTime<Utc>>,
    pub(crate) last_error: Option<String>,
    pub(crate) failures: u32,
    pub(crate) failing_since: Option<DateTime<Utc>>,
}

/// Why a priced booking cannot be taken right now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NotReady {
    WalletNotActive,
    WrongNetwork,
    WatcherStale,
}

impl std::fmt::Display for NotReady {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WalletNotActive => "the receiving wallet is not active (or not checked yet)",
            Self::WrongNetwork => "the indexer is on another network (or not checked yet)",
            Self::WatcherStale => "the watcher has not completed a step in the last 2 minutes",
        })
    }
}

/// Rates are reused for this long.
pub(crate) const RATE_CACHE: std::time::Duration = std::time::Duration::from_secs(300);
/// Upper bound of fetching both rate sources.
const RATE_FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
/// The heartbeat a new invoice requires.
const HEARTBEAT_MAX_AGE_SECS: i64 = 120;
/// ilia fork (Optimize): a priced page wakes an idle watcher whose last
/// step is older than this, so the guest's booking finds it fresh.
const WAKE_AFTER_SECS: i64 = 60;
/// How long a booking waits for the step its wake asked for.
pub(crate) const STEP_WAIT: std::time::Duration = std::time::Duration::from_secs(5);
/// The wallet status a new invoice requires is at most this old.
const ACCOUNT_CHECK_MAX_AGE_SECS: i64 = 2 * 3600;

pub(crate) struct TonRuntime {
    pub(crate) cfg: TonConfig,
    pub(crate) indexer: Arc<dyn TonIndexer>,
    pub(crate) rates: Arc<dyn RateSource>,
    /// Set once: by the test that builds the runtime, or by `spawn` (the
    /// owner's emails) in `calrs serve`.
    pub(crate) alerts: std::sync::OnceLock<Arc<dyn AlertSink>>,
    /// New invoices (and, later, "I have paid") wake the watcher.
    pub(crate) wake: tokio::sync::Notify,
    pub(crate) health: std::sync::Mutex<Health>,
    rate_cache: tokio::sync::Mutex<Option<(AcceptedRate, std::time::Instant)>>,
    /// The last rate both sources agreed on: anchors the one-source rule.
    anchor: std::sync::Mutex<Option<(u64, DateTime<Utc>)>>,
    /// Invoices whose payment is visible but not final yet.
    pub(crate) seen: std::sync::Mutex<HashMap<String, std::time::Instant>>,
    /// The configured address has its cursor row and is marked current.
    pub(crate) bootstrapped: std::sync::atomic::AtomicBool,
    /// ilia fork (Optimize): finished steps, counted, so a booking that
    /// woke an idle watcher can wait for its step.
    pub(crate) steps: tokio::sync::watch::Sender<u64>,
    /// The watcher's loop runs (`calrs serve`): a stale heartbeat is then
    /// idleness, which a wake ends within a step.
    pub(crate) looping: std::sync::atomic::AtomicBool,
    /// ilia fork: the last capacity refusal (unix seconds, 0 never), at
    /// every refusal: renewals pause for an hour after it. Kept in memory
    /// (a restart forgets it, as it forgets the refused guests).
    capacity_refused_at: std::sync::atomic::AtomicI64,
}

impl TonRuntime {
    pub(crate) fn new(
        cfg: TonConfig,
        indexer: Arc<dyn TonIndexer>,
        rates: Arc<dyn RateSource>,
        alerts: Option<Arc<dyn AlertSink>>,
    ) -> Self {
        let slot = std::sync::OnceLock::new();
        if let Some(sink) = alerts {
            let _ = slot.set(sink);
        }
        Self {
            cfg,
            indexer,
            rates,
            alerts: slot,
            wake: tokio::sync::Notify::new(),
            health: std::sync::Mutex::new(Health::default()),
            rate_cache: tokio::sync::Mutex::new(None),
            anchor: std::sync::Mutex::new(None),
            seen: std::sync::Mutex::new(HashMap::new()),
            bootstrapped: std::sync::atomic::AtomicBool::new(false),
            steps: tokio::sync::watch::channel(0).0,
            looping: std::sync::atomic::AtomicBool::new(false),
            capacity_refused_at: std::sync::atomic::AtomicI64::new(0),
        }
    }

    /// The production runtime: toncenter over HTTPS and both rate sources.
    pub(crate) fn http(cfg: TonConfig) -> Self {
        // One request a second overall, toncenter's keyless limit; a key
        // gives head room, not a faster loop.
        let indexer = HttpIndexer::new(
            &cfg.api_url,
            cfg.api_key.clone(),
            std::time::Duration::from_secs(1),
        );
        let rates = HttpRates::new(&cfg);
        Self::new(cfg, Arc::new(indexer), Arc::new(rates), None)
    }

    pub(crate) fn health(&self) -> Health {
        self.health.lock().map(|h| h.clone()).unwrap_or_default()
    }

    pub(crate) fn with_health(&self, f: impl FnOnce(&mut Health)) {
        if let Ok(mut h) = self.health.lock() {
            f(&mut h);
        }
    }

    /// May a new invoice be opened now? The wallet is active, the indexer is
    /// on our network, and the watcher completed a step recently.
    pub(crate) fn ready(&self, now: DateTime<Utc>) -> Result<(), NotReady> {
        let h = self.health();
        let checked = h
            .account_checked_at
            .is_some_and(|t| (now - t).num_seconds() <= ACCOUNT_CHECK_MAX_AGE_SECS);
        if !checked || h.account_status.as_deref() != Some("active") {
            return Err(NotReady::WalletNotActive);
        }
        if h.network_ok != Some(true) {
            return Err(NotReady::WrongNetwork);
        }
        if !h
            .last_ok
            .is_some_and(|t| (now - t).num_seconds() <= HEARTBEAT_MAX_AGE_SECS)
        {
            return Err(NotReady::WatcherStale);
        }
        Ok(())
    }

    /// As [`Self::ready`], for the owner's warnings: an idle watcher (its
    /// loop running, its last step fine, at most one idle sleep ago) is
    /// ready, since a booking wakes it and waits for the step.
    pub(crate) fn ready_on_demand(&self, now: DateTime<Utc>) -> Result<(), NotReady> {
        match self.ready(now) {
            Err(NotReady::WatcherStale) if self.idle_but_well(now) => Ok(()),
            other => other,
        }
    }

    pub(crate) fn idle_but_well(&self, now: DateTime<Utc>) -> bool {
        let h = self.health();
        let max = i64::try_from(watcher::IDLE_PRICED.as_secs()).unwrap_or(i64::MAX) + 120;
        self.looping.load(std::sync::atomic::Ordering::Acquire)
            && h.last_error.is_none()
            && h.last_ok.is_some_and(|t| (now - t).num_seconds() <= max)
    }

    /// A page of a priced type was shown: wake the watcher when its last
    /// step is over a minute old, so the booking that may follow finds a
    /// fresh heartbeat (at most one such step a minute, as the old idle
    /// cadence). Only an idle watcher whose last step went well: a failing
    /// indexer keeps its back-off, page views do not retry it.
    pub(crate) fn wake_if_idle(&self, now: DateTime<Utc>) {
        let h = self.health();
        if h.last_error.is_some() || h.failures > 0 {
            return;
        }
        if !h
            .last_ok
            .is_some_and(|t| (now - t).num_seconds() <= WAKE_AFTER_SECS)
        {
            self.wake.notify_one();
        }
    }

    /// A booking was refused for capacity (any refusal, alerted or not).
    pub(crate) fn note_capacity_refusal(&self, now: DateTime<Utc>) {
        self.capacity_refused_at
            .fetch_max(now.timestamp(), std::sync::atomic::Ordering::AcqRel);
    }

    /// A capacity refusal happened in the hour before `now`.
    pub(crate) fn capacity_pressed(&self, now: DateTime<Utc>) -> bool {
        let at = self
            .capacity_refused_at
            .load(std::sync::atomic::Ordering::Acquire);
        at > 0 && now.timestamp() - at < 3600
    }

    /// Wake the watcher and wait (at most `wait`) for a step to finish.
    /// False at once when no loop runs (tests, payments off): the wake is
    /// still left for it.
    pub(crate) async fn step_now(&self, wait: std::time::Duration) -> bool {
        let mut steps = self.steps.subscribe();
        steps.borrow_and_update();
        self.wake.notify_one();
        if !self.looping.load(std::sync::atomic::Ordering::Acquire) {
            return false;
        }
        matches!(
            tokio::time::timeout(wait, steps.changed()).await,
            Ok(Ok(()))
        )
    }

    /// The rate for a new invoice: the cached one when younger than five
    /// minutes, else freshly decided from both sources (at most 5 s).
    pub(crate) async fn rate(
        &self,
        pool: &sqlx::SqlitePool,
        now: DateTime<Utc>,
    ) -> Result<AcceptedRate, RateRefusal> {
        // ilia fork: without a corridor nothing is fetched (TON prices).
        let Some(policy) = self.cfg.rate_policy.as_ref() else {
            return Err(RateRefusal::NoCorridor);
        };
        let mut cache = self.rate_cache.lock().await;
        if let Some((rate, at)) = cache.as_ref() {
            if at.elapsed() < RATE_CACHE {
                return Ok(rate.clone());
            }
        }
        let quotes: Vec<_> = tokio::time::timeout(RATE_FETCH_TIMEOUT, self.rates.quotes())
            .await
            .unwrap_or_default();
        let mut ok = Vec::new();
        for q in quotes {
            match q {
                Ok(q) => ok.push(q),
                Err(e) => tracing::warn!(target: TARGET, error = %e, "rate source failed"),
            }
        }
        let anchor = match self.anchor.lock().ok().and_then(|a| *a) {
            Some(a) => Some(a),
            None => store::last_confirmed_rate(pool).await,
        };
        let rate = crate::ton::rate::decide(&ok, policy, anchor, now)?;
        if rate.confirmed {
            if let Ok(mut a) = self.anchor.lock() {
                *a = Some((rate.usd_micro, rate.at));
            }
        }
        *cache = Some((rate.clone(), std::time::Instant::now()));
        Ok(rate)
    }

    /// The cached rate, for showing an approximate amount; never fetches.
    pub(crate) fn cached_rate(&self) -> Option<AcceptedRate> {
        let cache = self.rate_cache.try_lock().ok()?;
        cache
            .as_ref()
            .filter(|(_, at)| at.elapsed() < RATE_CACHE)
            .map(|(r, _)| r.clone())
    }

    #[cfg(test)]
    pub(crate) fn forget_rate(&self) {
        if let Ok(mut c) = self.rate_cache.try_lock() {
            *c = None;
        }
    }
}

/// `AppState.ton`: the runtime when payments are configured, else `None`.
pub(crate) struct TonSlot {
    runtime: std::sync::RwLock<Option<Arc<TonRuntime>>>,
    /// Why payments are off, for the log and status (no secrets).
    disabled: std::sync::RwLock<Option<String>>,
    /// Payment page and status GETs per client address.
    pub(crate) page_limiter: RateLimiter,
    /// "I have paid" per invoice.
    pub(crate) check_limiter: RateLimiter,
    /// "Email me this link" per client address (each invoice once anyway).
    pub(crate) email_limiter: RateLimiter,
    /// "Email me the links" of an open booking, per client address and per
    /// guest email.
    pub(crate) resend_limiter: RateLimiter,
    pub(crate) resend_email_limiter: RateLimiter,
}

impl TonSlot {
    fn with(runtime: Option<Arc<TonRuntime>>, reason: Option<String>) -> Self {
        Self {
            runtime: std::sync::RwLock::new(runtime),
            disabled: std::sync::RwLock::new(reason),
            page_limiter: RateLimiter::new(120, 60),
            check_limiter: RateLimiter::new(1, 10),
            email_limiter: RateLimiter::new(5, 3600),
            resend_limiter: RateLimiter::new(5, 3600),
            resend_email_limiter: RateLimiter::new(1, 600),
        }
    }

    pub(crate) fn off(reason: Option<String>) -> Self {
        Self::with(None, reason)
    }

    /// From the `CALRS_TON_*` environment. Unset: off. Invalid: off, with
    /// the reason logged as an error.
    pub(crate) fn from_env() -> Self {
        match TonConfig::from_env() {
            Ok(Some(cfg)) => Self::with(Some(Arc::new(TonRuntime::http(cfg))), None),
            Ok(None) => Self::off(Some("CALRS_TON_NETWORK is not set".into())),
            Err(e) => {
                tracing::error!(
                    target: TARGET,
                    reason = %e,
                    "ton payments disabled: invalid configuration; priced event types are not bookable"
                );
                Self::off(Some(e.0))
            }
        }
    }

    pub(crate) fn get(&self) -> Option<Arc<TonRuntime>> {
        self.runtime.read().ok().and_then(|r| r.clone())
    }

    pub(crate) fn disabled_reason(&self) -> Option<String> {
        self.disabled.read().ok().and_then(|r| r.clone())
    }

    #[cfg(test)]
    pub(crate) fn install(&self, rt: Arc<TonRuntime>) {
        if let Ok(mut r) = self.runtime.write() {
            *r = Some(rt);
        }
    }

    /// Payments off, as with an unset or invalid configuration.
    #[cfg(test)]
    pub(crate) fn uninstall(&self) {
        if let Ok(mut r) = self.runtime.write() {
            *r = None;
        }
    }
}
