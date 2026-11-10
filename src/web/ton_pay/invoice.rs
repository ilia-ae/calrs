//! Invoice lifecycle: prepared before the booking transaction (rate, limits,
//! deadlines; the only HTTP), opened inside it (rows and the booking's hard
//! deadline), then settled by the watcher, closed, expired or waived. Plus
//! the guards the upstream approval and reschedule paths call.
//!
//! `paid` is derived only from a `ton_prices` row, never from the event
//! type's editable flags.

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use sqlx::{SqliteConnection, SqlitePool};

use super::{stamp, store, watcher, TonRuntime, TARGET};
use crate::ton::amount;
use crate::ton::deadlines::{self, InvoiceTimes};
use crate::ton::rate::{AcceptedRate, RateRefusal};
use crate::ton::reference;
use crate::ton::Network;
use crate::web::AppState;

/// Why a priced booking is not taken. Nothing is created in any case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// Payments are off or not ready (config, wallet, network, watcher).
    Unavailable(String),
    /// A priced team type: payments are for personal types only (v1).
    PersonalOnly,
    /// Too little time is left before the meeting to pay.
    TooLate,
    /// This guest (email or address) already holds an unpaid booking.
    AlreadyOpen,
    /// Too many open invoices overall, or too many today.
    Capacity,
    /// Too many bookings from this client network today.
    ClientLimit,
    /// No acceptable rate.
    Rate(RateRefusal),
    /// A database error: refuse rather than guess.
    Internal,
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(why) => write!(f, "payments unavailable: {why}"),
            Self::PersonalOnly => f.write_str("priced team event types are not bookable"),
            Self::TooLate => f.write_str("too little time left to pay before the meeting"),
            Self::AlreadyOpen => f.write_str("the guest already holds an unpaid booking"),
            Self::Capacity => f.write_str("too many open invoices"),
            Self::ClientLimit => f.write_str("too many invoices from this network today"),
            Self::Rate(r) => write!(f, "no acceptable rate: {r}"),
            Self::Internal => f.write_str("database error"),
        }
    }
}

/// The limits on open invoices, from the configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Limits {
    pub(crate) per_email: i64,
    pub(crate) per_ip: i64,
    pub(crate) total: i64,
    pub(crate) per_day: i64,
    pub(crate) per_ip_day: i64,
}

impl Limits {
    fn check(&self, c: store::OpenCounts) -> Result<(), Refusal> {
        if c.by_email >= self.per_email || c.by_tag >= self.per_ip {
            return Err(Refusal::AlreadyOpen);
        }
        if c.tag_day >= self.per_ip_day {
            return Err(Refusal::ClientLimit);
        }
        if c.total >= self.total || c.last_day >= self.per_day {
            return Err(Refusal::Capacity);
        }
        Ok(())
    }

    /// This invoice takes the last open place or the last of today's
    /// budget: the owner hears about it before anyone is refused.
    fn nearly_full(&self, c: store::OpenCounts) -> bool {
        c.total + 1 >= self.total || c.last_day + 1 >= self.per_day
    }
}

/// Everything an invoice needs, computed before the booking transaction.
#[derive(Clone, Debug)]
pub(crate) struct PreparedInvoice {
    pub(crate) event_type_id: String,
    pub(crate) event_title: String,
    pub(crate) start_utc: DateTime<Utc>,
    pub(crate) guest_email_norm: String,
    pub(crate) client_tag: String,
    pub(crate) price_usd_cents: i64,
    pub(crate) rate: AcceptedRate,
    pub(crate) amount_nano: i64,
    pub(crate) tolerance_nano: i64,
    pub(crate) times: InvoiceTimes,
    pub(crate) network: Network,
    pub(crate) address_raw: String,
    pub(crate) limits: Limits,
    pub(crate) created_at: DateTime<Utc>,
}

/// What the booking handler needs once the invoice is open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OpenedInvoice {
    pub(crate) id: String,
    pub(crate) pay_token: String,
    /// Display form, `B-XXXXXXXXXX`.
    pub(crate) reference: String,
    pub(crate) amount_nano: i64,
    pub(crate) expires_at: DateTime<Utc>,
}

/// The lower-cased, trimmed guest email the per-email limit counts by.
pub(crate) fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// The client's network: an IPv4 address's /24, an IPv6 address's /64 (an
/// IPv4-mapped one counts as IPv4). One person rarely holds more than one
/// of these; anything unparsable is its own network.
pub(crate) fn client_network(client_ip: &str) -> String {
    match client_ip.trim().parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(v4)) => {
            let o = v4.octets();
            format!("v4:{}.{}.{}", o[0], o[1], o[2])
        }
        Ok(std::net::IpAddr::V6(v6)) => match v6.to_ipv4_mapped() {
            Some(v4) => client_network(&v4.to_string()),
            None => {
                let s = v6.segments();
                format!("v6:{:x}:{:x}:{:x}:{:x}", s[0], s[1], s[2], s[3])
            }
        },
        Err(_) => format!("raw:{}", client_ip.trim()),
    }
}

/// HMAC of the client's network ([`client_network`]) under the instance
/// key: the per-network limits without storing the address. Hex, 32
/// characters.
pub(crate) fn client_tag(secret_key: &[u8; 32], client_ip: &str) -> String {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret_key)
        .unwrap_or_else(|_| unreachable!("HMAC accepts any key length"));
    mac.update(b"ton-net:");
    mac.update(client_network(client_ip).as_bytes());
    hex::encode(&mac.finalize().into_bytes()[..16])
}

/// Step 1 of a priced booking, before `BEGIN`: is this type priced, can it
/// be paid for now, and for how much. `Ok(None)`: not priced, the booking
/// goes the upstream way unchanged. The only network call (the rate) is
/// here, never inside the SQL transaction.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn prepare_invoice(
    state: &AppState,
    event_type_id: &str,
    start_utc: DateTime<Utc>,
    min_notice_min: i64,
    guest_email: &str,
    client_ip: &str,
    now: DateTime<Utc>,
) -> Result<Option<PreparedInvoice>, Refusal> {
    // Whole seconds, as stored: the deadlines handed back match the rows.
    let now = DateTime::from_timestamp(now.timestamp(), 0).unwrap_or(now);
    let price = match store::price_for(&state.pool, event_type_id).await {
        Ok(Some(p)) => p,
        Ok(None) => return Ok(None),
        Err(e) => {
            tracing::error!(target: TARGET, error = %e, event_type_id, "cannot read the price");
            return Err(Refusal::Internal);
        }
    };
    let Some(rt) = state.ton.get() else {
        let why = state
            .ton
            .disabled_reason()
            .unwrap_or_else(|| "not configured".into());
        return Err(Refusal::Unavailable(why));
    };
    if price.is_team != 0 {
        return Err(Refusal::PersonalOnly);
    }
    if let Err(e) = rt.ready(now) {
        if e == super::NotReady::WatcherStale {
            // An idle watcher may be in its long sleep (a price was just
            // switched on): a step now makes the next try succeed.
            rt.wake.notify_one();
        }
        return Err(Refusal::Unavailable(e.to_string()));
    }
    let times = deadlines::compute(
        now,
        start_utc,
        min_notice_min,
        rt.cfg.window_rules(price.window_min),
    )
    .map_err(|_| Refusal::TooLate)?;
    let limits = Limits {
        per_email: rt.cfg.max_open_per_email,
        per_ip: rt.cfg.max_open_per_ip,
        total: rt.cfg.max_open_total,
        per_day: rt.cfg.max_per_day,
        per_ip_day: rt.cfg.max_per_ip_day,
    };
    let guest_email_norm = normalize_email(guest_email);
    let tag = client_tag(&state.secret_key, client_ip);
    let counts = match state.pool.acquire().await {
        Ok(mut conn) => store::open_counts(&mut conn, &guest_email_norm, &tag, now).await,
        Err(e) => Err(e),
    }
    .map_err(|e| {
        tracing::error!(target: TARGET, error = %e, "cannot count open invoices");
        Refusal::Internal
    })?;
    if let Err(refusal) = limits.check(counts) {
        if refusal == Refusal::Capacity {
            watcher::raise(
                &rt,
                &state.pool,
                "capacity",
                &format!("open={} last_24h={}", counts.total, counts.last_day),
                now,
            )
            .await;
        }
        return Err(refusal);
    }
    if limits.nearly_full(counts) {
        watcher::raise(
            &rt,
            &state.pool,
            "capacity_near",
            &format!(
                "open={}/{} last_24h={}/{}",
                counts.total + 1,
                limits.total,
                counts.last_day + 1,
                limits.per_day
            ),
            now,
        )
        .await;
    }
    let rate = match rt.rate(&state.pool, now).await {
        Ok(r) => r,
        Err(refusal) => {
            watcher::raise(
                &rt,
                &state.pool,
                "rate_unavailable",
                &refusal.to_string(),
                now,
            )
            .await;
            return Err(Refusal::Rate(refusal));
        }
    };
    let cents = u64::try_from(price.price_usd_cents).map_err(|_| Refusal::Internal)?;
    let amount_nano = amount::cents_to_nano(cents, rate.usd_micro)
        .and_then(amount::to_i64)
        .ok_or(Refusal::Internal)?;
    let tolerance_nano = amount::to_i64(rt.cfg.underpay_tolerance_nano).unwrap_or(0);
    Ok(Some(PreparedInvoice {
        event_type_id: event_type_id.to_string(),
        event_title: price.title,
        start_utc,
        guest_email_norm,
        client_tag: tag,
        price_usd_cents: price.price_usd_cents,
        rate,
        amount_nano,
        tolerance_nano,
        times,
        network: rt.cfg.network,
        address_raw: rt.cfg.address_raw.clone(),
        limits,
        created_at: now,
    }))
}

fn random_token() -> String {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Step 2, inside the booking's transaction, right after the pending
/// booking row and its deadline were written: re-check the limits (the
/// transaction holds the write lock, so concurrent bookings serialise),
/// insert the invoice, and replace the booking's deadline with the hard
/// one. No halfway host nudge (`pending_since` is NULL).
pub(crate) async fn open_invoice(
    conn: &mut SqliteConnection,
    booking_id: &str,
    p: &PreparedInvoice,
) -> Result<OpenedInvoice, Refusal> {
    let counts = store::open_counts(conn, &p.guest_email_norm, &p.client_tag, p.created_at)
        .await
        .map_err(|_| Refusal::Internal)?;
    p.limits.check(counts)?;

    let id = uuid::Uuid::new_v4().to_string();
    let mut attempt = 0;
    let (pay_token, code) = loop {
        attempt += 1;
        let pay_token = random_token();
        let code = reference::generate(&mut rand::rngs::OsRng);
        let stored = reference::normalize(&code).ok_or(Refusal::Internal)?;
        let inserted = sqlx::query(
            "INSERT INTO ton_invoices (id, booking_id, event_type_id, pay_token, reference, network,
                 address_raw, event_title, start_at_utc, guest_email_norm, client_tag,
                 price_usd_cents, rate_usd_micro, rate_sources, rate_at, amount_nano, tolerance_nano,
                 status, created_at, expires_at, settle_by, hard_deadline)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'open', ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(booking_id)
        .bind(&p.event_type_id)
        .bind(&pay_token)
        .bind(&stored)
        .bind(p.network.as_str())
        .bind(&p.address_raw)
        .bind(&p.event_title)
        .bind(stamp(p.start_utc))
        .bind(&p.guest_email_norm)
        .bind(&p.client_tag)
        .bind(p.price_usd_cents)
        .bind(i64::try_from(p.rate.usd_micro).map_err(|_| Refusal::Internal)?)
        .bind(&p.rate.sources)
        .bind(stamp(p.rate.at))
        .bind(p.amount_nano)
        .bind(p.tolerance_nano)
        .bind(stamp(p.created_at))
        .bind(stamp(p.times.expires_at))
        .bind(stamp(p.times.settle_by))
        .bind(stamp(p.times.hard_deadline))
        .execute(&mut *conn)
        .await;
        match inserted {
            Ok(_) => break (pay_token, code),
            // A clash of the random code or token (practically never):
            // a constraint failure aborts the statement, not the transaction.
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() && attempt < 3 => continue,
            Err(e) => {
                tracing::error!(target: TARGET, error = %e, booking_id, "cannot insert the invoice");
                return Err(Refusal::Internal);
            }
        }
    };

    let held = sqlx::query(
        "UPDATE bookings SET pending_expires_at = ?, pending_since = NULL, pending_reminder_sent_at = NULL
         WHERE id = ? AND status = 'pending'",
    )
    .bind(stamp(p.times.hard_deadline))
    .bind(booking_id)
    .execute(&mut *conn)
    .await
    .map_err(|_| Refusal::Internal)?
    .rows_affected();
    if held != 1 {
        // Not a pending booking: it would not wait for the payment.
        tracing::error!(target: TARGET, booking_id, "a priced booking is not pending");
        return Err(Refusal::Internal);
    }
    Ok(OpenedInvoice {
        id,
        pay_token,
        reference: code,
        amount_nano: p.amount_nano,
        expires_at: p.times.expires_at,
    })
}

/// After the booking transaction committed: log it and wake the watcher.
pub(crate) fn invoice_opened(rt: &TonRuntime, booking_id: &str, inv: &OpenedInvoice) {
    tracing::info!(
        target: TARGET,
        booking_id,
        invoice = %inv.id,
        amount_nano = inv.amount_nano,
        expires_at = %stamp(inv.expires_at),
        "ton invoice opened"
    );
    rt.wake.notify_one();
}

/// The only free path: the host explicitly confirms an unpaid booking.
/// Moves an `open` invoice to `waived` (and nothing else); the caller then
/// runs the ordinary confirmation. `false` when there was no open invoice.
pub(crate) async fn waive(
    pool: &SqlitePool,
    booking_id: &str,
    by: &str,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    waive_if_due(pool, booking_id, by, None, now).await
}

/// [`waive`], and with `remaining_nano` only while exactly that much is
/// still due (MCP `waive_payment`: the owner confirmed that amount, so a
/// part paid meanwhile makes it refuse instead of waiving another sum). A
/// part already received is flagged (`partial`): the owner decides whether
/// it goes back.
pub(crate) async fn waive_if_due(
    pool: &SqlitePool,
    booking_id: &str,
    by: &str,
    remaining_nano: Option<i64>,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let n = sqlx::query(
        "UPDATE ton_invoices SET status = 'waived', closed_at = ?, waived_by = ?,
             problem = CASE WHEN paid_nano > 0 THEN 'partial' ELSE problem END
         WHERE booking_id = ? AND status = 'open'
           AND (? IS NULL OR MAX(amount_nano - paid_nano, 0) = ?)",
    )
    .bind(stamp(now))
    .bind(by)
    .bind(booking_id)
    .bind(remaining_nano)
    .bind(remaining_nano)
    .execute(pool)
    .await?
    .rows_affected();
    if n == 1 {
        tracing::warn!(target: TARGET, booking_id, by, "payment waived");
    }
    Ok(n == 1)
}

/// A booking was cancelled or declined: close its open invoice now (a part
/// already paid is flagged `partial`, to be refunded), or flag a paid one
/// for a refund (the watcher would do the same on its next step; this
/// makes it immediate).
pub(crate) async fn on_booking_ended(pool: &SqlitePool, booking_id: &str, now: DateTime<Utc>) {
    let closed = sqlx::query(
        "UPDATE ton_invoices SET status = 'closed', closed_at = ?,
             problem = CASE WHEN paid_nano > 0 THEN 'partial' ELSE problem END
         WHERE booking_id = ? AND status = 'open'",
    )
    .bind(stamp(now))
    .bind(booking_id)
    .execute(pool)
    .await;
    let refund = sqlx::query(
        "UPDATE ton_invoices SET problem = 'refund_due'
         WHERE booking_id = ? AND status IN ('paid', 'confirmed') AND paid_nano > 0
           AND (problem IS NULL OR problem != 'refund_due')",
    )
    .bind(booking_id)
    .execute(pool)
    .await;
    if let (Err(e), _) | (_, Err(e)) = (&closed, &refund) {
        tracing::error!(target: TARGET, error = %e, booking_id, "cannot update the invoice of an ended booking");
    }
}

/// Revoke an address (compromise): its open invoices close with
/// `address_revoked`, their bookings expire on the next sweep. Returns the
/// number of invoices closed.
pub(crate) async fn close_open_for_address(
    pool: &SqlitePool,
    network: Network,
    address_raw: &str,
    now: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let ids: Vec<String> = sqlx::query_scalar(
        "UPDATE ton_invoices SET status = 'closed', closed_at = ?,
             problem = 'address_revoked'
         WHERE network = ? AND address_raw = ? AND status = 'open'
         RETURNING booking_id",
    )
    .bind(stamp(now))
    .bind(network.as_str())
    .bind(address_raw)
    .fetch_all(&mut *tx)
    .await?;
    for bid in &ids {
        sqlx::query(
            "UPDATE bookings SET pending_expires_at = ? WHERE id = ? AND status = 'pending'",
        )
        .bind(stamp(now))
        .bind(bid)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(ids.len() as u64)
}

/// The invoice status of a booking: `Ok(None)` without an invoice.
async fn invoice_status(
    pool: &SqlitePool,
    booking_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT status FROM ton_invoices WHERE booking_id = ?")
        .bind(booking_id)
        .fetch_optional(pool)
        .await
}

fn is_settled(status: &str) -> bool {
    matches!(status, "paid" | "confirmed" | "waived")
}

/// Guard of every approval path (web, email link, MCP): refuse when the
/// booking has an invoice that is not paid, confirmed or waived. Those three
/// are absorbing, so one early check has no race. A database error refuses.
pub(crate) async fn approval_refused(pool: &SqlitePool, booking_id: &str) -> bool {
    approval_check(pool, booking_id).await.unwrap_or(true)
}

/// [`approval_refused`], telling a database error apart (the caller then
/// still refuses, but without calling the booking unpaid).
pub(crate) async fn approval_check(
    pool: &SqlitePool,
    booking_id: &str,
) -> Result<bool, sqlx::Error> {
    match invoice_status(pool, booking_id).await {
        Ok(None) => Ok(false),
        Ok(Some(status)) => Ok(!is_settled(&status)),
        Err(e) => {
            tracing::error!(target: TARGET, error = %e, booking_id, "cannot read the invoice: approval refused");
            Err(e)
        }
    }
}

/// Guard of the reschedule paths: an invoice that is open, expired or
/// closed means the booking is unpaid. Same rule as [`approval_refused`].
pub(crate) async fn unpaid(pool: &SqlitePool, booking_id: &str) -> bool {
    approval_refused(pool, booking_id).await
}

/// The booking's invoice is paid, confirmed or waived (a guest reschedule
/// then keeps it confirmed). A database error answers `false`.
pub(crate) async fn settled(pool: &SqlitePool, booking_id: &str) -> bool {
    matches!(invoice_status(pool, booking_id).await, Ok(Some(s)) if is_settled(&s))
}

/// The hosts have not been told about a priced booking: nothing was paid
/// on its invoice, which is open (awaiting the payment) or ended unpaid
/// (expired, or released as taken, in the minute before calrs's sweep
/// declines the booking), and the booking was not confirmed. One rule for
/// the guest-cancel emails and SMS ([`cancel_is_quiet`]) and the hosts'
/// Telegram chats (`telegram::PaymentFacts::hosts_unaware`).
pub(crate) fn hosts_unaware(invoice_status: &str, paid_nano: i64, booking_status: &str) -> bool {
    paid_nano <= 0
        && matches!(invoice_status, "open" | "expired" | "closed")
        && booking_status != "confirmed"
}

/// The hosts were never told about this booking ([`hosts_unaware`]): a
/// guest cancel then sends nothing. With a part paid, the usual
/// cancellation notices go out: the host has money to return.
pub(crate) async fn cancel_is_quiet(pool: &SqlitePool, booking_id: &str) -> bool {
    sqlx::query_as::<_, (String, i64, String)>(
        "SELECT i.status, i.paid_nano, b.status FROM ton_invoices i
         JOIN bookings b ON b.id = i.booking_id
         WHERE i.booking_id = ?",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .is_some_and(|(status, paid, booking)| hosts_unaware(&status, paid, &booking))
}

/// The event type has an enabled price. A database error answers `true`:
/// the group paths then refuse rather than book a priced type for free.
pub(crate) async fn is_priced(pool: &SqlitePool, event_type_id: &str) -> bool {
    match store::price_for(pool, event_type_id).await {
        Ok(p) => p.is_some(),
        Err(_) => true,
    }
}
