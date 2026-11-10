//! ilia fork: TON payments in Telegram (ilia-booking docs/ton.md, «Telegram»;
//! the seam is `crate::telegram::TelegramEvent`). Everything here only
//! queues (`crate::telegram::notify`), so neither the watcher nor a booking
//! ever waits on Telegram; with Telegram off it is a no-op.
//!
//! - The guest links a chat on the payment page (the same deep link as on
//!   a confirmation page, derived from the payment link so the page can show
//!   it on every load) and then hears: what to pay and by when, a partial
//!   payment, the confirmation, a late payment, and the release.
//! - Hosts hear nothing about a priced booking until it is paid: then
//!   "paid", and every problem the owner is emailed about for a booking.
//! - The payment link (it carries the payment token) only ever goes to the
//!   guest's own linked chat; no message carries the comment code.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::SqlitePool;

use super::{parse_stamp, AlertSink, TonAlert, TARGET};
use crate::telegram::{PaymentRelease, TelegramEvent};
use crate::ton::amount::{format_ton, format_usd_cents, from_i64};
use crate::web::AppState;

/// Every `ton_invoices.problem`, as the schema allows them. A host message
/// reads `ton-problem-<problem>` (underscores as dashes).
pub(crate) const PROBLEMS: [&str; 8] = [
    "host_busy",
    "partial",
    "overpaid",
    "refund_due",
    "confirmed_unpaid",
    "address_revoked",
    "host_busy_unpaid",
    "confirm_failed",
];

/// A booking's invoice, as the Telegram worker reads it when it writes a
/// message (so a queued notice never states an amount that is out of date).
#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct PaymentFacts {
    pub(crate) status: String,
    pub(crate) amount_nano: i64,
    pub(crate) paid_nano: i64,
    pub(crate) currency: String,
    pub(crate) price_usd_cents: i64,
    pub(crate) expires_at: String,
    pub(crate) pay_token: String,
}

impl PaymentFacts {
    /// The guest can still pay.
    pub(crate) fn open(&self) -> bool {
        self.status == "open"
    }

    /// The hosts have not been told about this booking: it awaits its
    /// payment, or ended before anything was paid. They hear payment events
    /// only (the first is "paid").
    /// The same rule keeps a guest's cancel quiet by email and SMS
    /// ([`super::invoice::hosts_unaware`]).
    pub(crate) fn hosts_unaware(&self, booking_status: &str) -> bool {
        super::invoice::hosts_unaware(&self.status, self.paid_nano, booking_status)
    }

    pub(crate) fn amount(&self) -> String {
        format_ton(from_i64(self.amount_nano))
    }

    pub(crate) fn paid(&self) -> String {
        format_ton(from_i64(self.paid_nano))
    }

    pub(crate) fn rest(&self) -> String {
        format_ton(from_i64(self.amount_nano).saturating_sub(from_i64(self.paid_nano)))
    }

    /// The price in USD; `None` for a price in TON.
    pub(crate) fn usd(&self) -> Option<String> {
        (self.currency == "USD").then(|| format_usd_cents(from_i64(self.price_usd_cents)))
    }

    pub(crate) fn expires(&self) -> Option<DateTime<Utc>> {
        parse_stamp(&self.expires_at)
    }

    /// The payment page, absolute: for the guest's own chat only.
    pub(crate) fn pay_url(&self, base: Option<&str>) -> Option<String> {
        base.map(|b| format!("{b}/booking/pay/{}", self.pay_token))
    }
}

/// The invoice of `booking_id`, if it has one.
pub(crate) async fn facts(pool: &SqlitePool, booking_id: &str) -> Option<PaymentFacts> {
    sqlx::query_as::<_, PaymentFacts>(
        "SELECT status, amount_nano, paid_nano, currency, price_usd_cents, expires_at, pay_token
         FROM ton_invoices WHERE booking_id = ?",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(target: TARGET, error = %e, booking_id, "cannot read an invoice for telegram");
        None
    })
}

// --- the guest's link -------------------------------------------------------

/// The booking's Telegram deep-link token, derived from the server's secret
/// key and the payment token: the payment page shows the same link on every
/// load (it refreshes itself), where a confirmation page shows a random one
/// once. Only whoever holds the payment link sees it, as with a
/// confirmation page; the database alone (its SHA-256 only) or the payment
/// link alone gives nothing.
pub(crate) fn link_token(secret_key: &[u8; 32], pay_token: &str) -> String {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret_key)
        .expect("HMAC accepts a key of any length");
    mac.update(b"ilia-telegram-pay-link\0");
    mac.update(pay_token.as_bytes());
    crate::telegram::tokens::booking_token_from(&mac.finalize().into_bytes().into())
}

/// Right after a priced booking committed: its Telegram link row (when its
/// event type offers Telegram), under the token the payment page shows.
/// Hosts are not told: there is nothing to approve until it is paid.
pub(crate) async fn booking_created(
    state: &AppState,
    booking_id: &str,
    pay_token: &str,
    entered_username: Option<&str>,
) {
    let token = link_token(&state.secret_key, pay_token);
    crate::telegram::priced_booking_created(&state.pool, booking_id, entered_username, &token)
        .await;
}

/// The deep link the payment page and the "your payment link" email offer:
/// while the booking is pending or confirmed, Telegram is on, and the
/// booking has a link row (its event type offered Telegram when it was
/// booked). A row made under another secret key is re-keyed to this one
/// (`crate::telegram::offered_priced_link`).
pub(crate) async fn pay_page_link(
    state: &AppState,
    booking_id: &str,
    pay_token: &str,
) -> Option<String> {
    pay_link(&state.pool, &state.secret_key, booking_id, pay_token).await
}

/// [`pay_page_link`] from the pool and key (the alert sinks have no state).
pub(crate) async fn pay_link(
    pool: &SqlitePool,
    secret_key: &[u8; 32],
    booking_id: &str,
    pay_token: &str,
) -> Option<String> {
    if !crate::telegram::enabled() {
        return None;
    }
    let token = link_token(secret_key, pay_token);
    crate::telegram::offered_priced_link(pool, booking_id, &token).await
}

// --- events -----------------------------------------------------------------

/// The guest's Telegram event when the expiry sweep declines `booking_id`:
/// for a priced booking, why it was released (as the "not booked" email
/// says); otherwise upstream's "request expired". A waived booking is an
/// ordinary request by now.
pub(crate) async fn expiry_event(pool: &SqlitePool, booking_id: &str) -> TelegramEvent {
    let row: Option<(String, Option<String>, i64)> = sqlx::query_as(
        "SELECT status, problem, paid_nano FROM ton_invoices WHERE booking_id = ?",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(target: TARGET, error = %e, booking_id, "cannot read an invoice for telegram");
        None
    });
    let Some((status, problem, paid_nano)) = row else {
        return TelegramEvent::Expired;
    };
    let why = match status.as_str() {
        "waived" => return TelegramEvent::Expired,
        "paid" | "confirmed" => PaymentRelease::Refund,
        "closed" if problem.as_deref() == Some("host_busy_unpaid") => PaymentRelease::Taken,
        _ => PaymentRelease::Expired,
    };
    TelegramEvent::PaymentExpired {
        why,
        partly_paid: paid_nano > 0,
    }
}

/// An approval of a booking whose payment arrived (the watcher confirming
/// it, or the host confirming one held as "paid, host busy"): the guest
/// hears "payment received, booking confirmed" from the payment notice, so
/// upstream's "approved" is not sent as well. Only while a watcher with its
/// alerts runs: that notice comes from the watcher (its reconcile, then
/// `deliver_alerts`). With payments off (unset or invalid configuration),
/// a paid booking the host confirms gets upstream's "approved".
pub(crate) async fn approval_told_by_payment(state: &AppState, booking_id: &str) -> bool {
    if !state.ton.get().is_some_and(|rt| rt.alerts.get().is_some()) {
        return false;
    }
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM ton_invoices WHERE booking_id = ? AND status = 'paid')",
    )
    .bind(booking_id)
    .fetch_one(&state.pool)
    .await
    .map(|v| v != 0)
    .unwrap_or(false)
}

/// The event an owner alert means for a booking's Telegram chats, and the
/// booking. `None`: nothing to say there (a transfer that matched no
/// booking, health, an ignored transfer: those stay email only).
pub(crate) async fn event_for_alert(
    pool: &SqlitePool,
    alert: &TonAlert,
) -> Option<(String, TelegramEvent)> {
    match alert {
        TonAlert::Payment { tx_hex, kind } => {
            let (booking_id, value): (String, i64) = sqlx::query_as(
                "SELECT i.booking_id, p.value_nano FROM ton_payments p
                 JOIN ton_invoices i ON i.id = p.invoice_id
                 WHERE p.tx_hex = ?",
            )
            .bind(tx_hex)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()?;
            let value = format_ton(from_i64(value));
            let event = match kind.as_str() {
                "partial" => TelegramEvent::PaymentPartial,
                "late" => TelegramEvent::PaymentLate {
                    value,
                    to_guest: from_first_payer(pool, tx_hex).await,
                },
                "overpaid" => TelegramEvent::PaymentExtra { value },
                _ => return None,
            };
            Some((booking_id, event))
        }
        TonAlert::InvoiceProblem {
            invoice_id,
            problem,
        } => {
            if !PROBLEMS.contains(&problem.as_str()) {
                return None;
            }
            let booking_id = booking_of(pool, invoice_id).await?;
            Some((
                booking_id,
                TelegramEvent::PaymentProblem {
                    problem: problem.clone(),
                },
            ))
        }
        TonAlert::PaymentReceived { invoice_id } => {
            let booking_id = booking_of(pool, invoice_id).await?;
            Some((booking_id, TelegramEvent::PaymentReceived))
        }
        TonAlert::Renewed { invoice_id } => {
            let (booking_id, renewals): (String, i64) =
                sqlx::query_as("SELECT booking_id, renewals FROM ton_invoices WHERE id = ?")
                    .bind(invoice_id)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten()?;
            Some((
                booking_id,
                TelegramEvent::PaymentRenewed {
                    renewal: u8::try_from(renewals).unwrap_or(u8::MAX),
                },
            ))
        }
        TonAlert::Health { .. } => None,
    }
}

/// The transfer `tx_hex` is the first one recorded for its invoice, or
/// comes from the same wallet as that first one. Its comment code is on
/// chain once anything was paid, so anyone watching the address can send a
/// "late" transfer carrying it; before that, only whoever holds the payment
/// link knows the code. The guest hears of a late transfer only from that
/// first payer (an unknown sender's goes to the hosts alone), so nobody
/// else can make the guest believe their own money is stuck.
pub(crate) async fn from_first_payer(pool: &SqlitePool, tx_hex: &str) -> bool {
    let first: Option<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT f.tx_hex, f.source_raw, p.source_raw
         FROM ton_payments p
         JOIN ton_payments f ON f.tx_hex = (
             SELECT x.tx_hex FROM ton_payments x WHERE x.invoice_id = p.invoice_id
             ORDER BY x.utime, x.lt, x.tx_hex LIMIT 1)
         WHERE p.tx_hex = ? AND p.invoice_id IS NOT NULL",
    )
    .bind(tx_hex)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(target: TARGET, error = %e, "cannot read a transfer's first payer");
        None
    });
    match first {
        Some((first_tx, _, _)) if first_tx == tx_hex => true,
        Some((_, Some(first), Some(this))) => first == this,
        _ => false,
    }
}

async fn booking_of(pool: &SqlitePool, invoice_id: &str) -> Option<String> {
    sqlx::query_scalar("SELECT booking_id FROM ton_invoices WHERE id = ?")
        .bind(invoice_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

/// The owner alerts, told to the booking's Telegram chats: the watcher
/// claims each alert once, so each is told once. Installed by `spawn` next
/// to the emails.
pub(crate) struct TelegramSink {
    pool: SqlitePool,
}

impl TelegramSink {
    pub(crate) fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AlertSink for TelegramSink {
    async fn deliver(&self, alert: TonAlert) {
        if !crate::telegram::enabled() {
            return;
        }
        if let Some((booking_id, event)) = event_for_alert(&self.pool, &alert).await {
            crate::telegram::notify(&self.pool, &booking_id, event);
        }
    }
}

/// The owner alerts' sink in `calrs serve`: the owner's emails, then the
/// booking's Telegram chats (a no-op with Telegram off).
pub(crate) fn serve_sink(state: &AppState) -> Fanout {
    Fanout(vec![
        Arc::new(super::alerts::EmailSink::new(
            state.pool.clone(),
            state.secret_key,
        )),
        Arc::new(TelegramSink::new(state.pool.clone())),
    ])
}

/// Every alert to each sink in turn (`calrs serve`: email, then Telegram).
pub(crate) struct Fanout(pub(crate) Vec<Arc<dyn AlertSink>>);

#[async_trait]
impl AlertSink for Fanout {
    async fn deliver(&self, alert: TonAlert) {
        for sink in &self.0 {
            sink.deliver(alert.clone()).await;
        }
    }
}
