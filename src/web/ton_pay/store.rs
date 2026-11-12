//! SQL of the TON tables. Functions taking a `SqliteConnection` run inside a
//! caller's transaction; the caller makes a write its first statement
//! (sqlx 0.7 opens deferred transactions) and never touches the pool while
//! the transaction is open.

use chrono::{DateTime, Utc};
use sqlx::{SqliteConnection, SqlitePool};

use super::{parse_stamp, stamp};
use crate::ton::decide::{InvoiceStatus, InvoiceView};
use crate::ton::Network;

/// A price: in TON (the default; exactly the invoice amount, no rate) or
/// in US dollars (converted at the rate of the moment the guest books).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Price {
    Ton(u128),
    Usd(u64),
}

impl Price {
    /// From the `ton_prices` columns; `None` for a row the CHECKs would
    /// not allow.
    pub(crate) fn from_columns(
        currency: &str,
        price_nano: Option<i64>,
        price_usd_cents: Option<i64>,
    ) -> Option<Self> {
        match currency {
            "TON" => price_nano
                .and_then(|n| u128::try_from(n).ok())
                .filter(|n| *n > 0)
                .map(Self::Ton),
            "USD" => price_usd_cents
                .and_then(|c| u64::try_from(c).ok())
                .filter(|c| *c > 0)
                .map(Self::Usd),
            _ => None,
        }
    }

    pub(crate) fn currency(&self) -> &'static str {
        match self {
            Self::Ton(_) => "TON",
            Self::Usd(_) => "USD",
        }
    }

    /// `(currency, price_nano, price_usd_cents)` as stored.
    pub(crate) fn columns(&self) -> (&'static str, Option<i64>, Option<i64>) {
        match *self {
            Self::Ton(n) => ("TON", i64::try_from(n).ok(), None),
            Self::Usd(c) => ("USD", None, i64::try_from(c).ok()),
        }
    }

    /// "100.00" (TON) or "40.00" (USD), without the unit.
    pub(crate) fn amount(&self) -> String {
        match *self {
            Self::Ton(n) => crate::ton::amount::format_ton(n),
            Self::Usd(c) => crate::ton::amount::format_usd_cents(u128::from(c)),
        }
    }
}

/// How an event type is priced (ilia fork, the demand ladder): a fixed
/// price, or the site-wide ladder (TON, the step of the owner's demand).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Terms {
    Fixed(Price),
    Ladder,
}

impl Terms {
    /// From the `ton_prices` columns; `None` for a row the CHECKs would
    /// not allow.
    pub(crate) fn from_columns(
        mode: &str,
        currency: &str,
        price_nano: Option<i64>,
        price_usd_cents: Option<i64>,
    ) -> Option<Self> {
        match mode {
            "fixed" => Price::from_columns(currency, price_nano, price_usd_cents).map(Self::Fixed),
            "ladder" if currency == "TON" && price_nano.is_none() && price_usd_cents.is_none() => {
                Some(Self::Ladder)
            }
            _ => None,
        }
    }

    /// `(mode, currency, price_nano, price_usd_cents)` as stored.
    pub(crate) fn columns(&self) -> (&'static str, &'static str, Option<i64>, Option<i64>) {
        match self {
            Self::Fixed(p) => {
                let (currency, nano, cents) = p.columns();
                ("fixed", currency, nano, cents)
            }
            Self::Ladder => ("ladder", "TON", None, None),
        }
    }

    pub(crate) fn mode(&self) -> &'static str {
        self.columns().0
    }

    pub(crate) fn currency(&self) -> &'static str {
        self.columns().1
    }

    pub(crate) fn fixed(&self) -> Option<Price> {
        match self {
            Self::Fixed(p) => Some(*p),
            Self::Ladder => None,
        }
    }
}

/// The price row of an event type, with what an invoice snapshots.
#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct PriceRow {
    pub(crate) event_type_id: String,
    pub(crate) mode: String,
    pub(crate) currency: String,
    pub(crate) price_nano: Option<i64>,
    pub(crate) price_usd_cents: Option<i64>,
    pub(crate) window_min: Option<i64>,
    pub(crate) is_team: i64,
    pub(crate) title: String,
    /// The owner of the type's account: whose demand a ladder counts.
    pub(crate) owner_user_id: Option<String>,
}

impl PriceRow {
    pub(crate) fn terms(&self) -> Option<Terms> {
        Terms::from_columns(
            &self.mode,
            &self.currency,
            self.price_nano,
            self.price_usd_cents,
        )
    }
}

/// The enabled price of an event type, if any. A database error is an
/// error: the caller fails closed (refuses) rather than treating the type
/// as free.
pub(crate) async fn price_for(
    pool: &SqlitePool,
    event_type_id: &str,
) -> Result<Option<PriceRow>, sqlx::Error> {
    sqlx::query_as::<_, PriceRow>(
        "SELECT p.event_type_id, p.mode, p.currency, p.price_nano, p.price_usd_cents, p.window_min,
                CASE WHEN et.team_id IS NULL THEN 0 ELSE 1 END AS is_team, et.title,
                a.user_id AS owner_user_id
         FROM ton_prices p JOIN event_types et ON et.id = p.event_type_id
         LEFT JOIN accounts a ON a.id = et.account_id
         WHERE p.event_type_id = ? AND p.enabled = 1",
    )
    .bind(event_type_id)
    .fetch_optional(pool)
    .await
}

/// Write the price of `event_type_id` (insert or replace its terms) on a
/// caller's connection. Returns whether the row is new. An enabled price
/// makes the type ask for the guest's Telegram unless a mode was chosen
/// for it ([`crate::telegram::store::default_mode_for_priced`]).
pub(crate) async fn save_price(
    conn: &mut SqliteConnection,
    event_type_id: &str,
    terms: Terms,
    window_min: Option<i64>,
    enabled: bool,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let (mode, currency, nano, cents) = terms.columns();
    let existed: bool = sqlx::query_scalar::<_, i64>(
        "UPDATE ton_prices SET mode = ?, currency = ?, price_nano = ?, price_usd_cents = ?,
             window_min = ?, enabled = ?, updated_at = ?
         WHERE event_type_id = ? RETURNING 1",
    )
    .bind(mode)
    .bind(currency)
    .bind(nano)
    .bind(cents)
    .bind(window_min)
    .bind(i64::from(enabled))
    .bind(stamp(now))
    .bind(event_type_id)
    .fetch_optional(&mut *conn)
    .await?
    .is_some();
    if enabled {
        // Only inserts when no mode was ever chosen for the type.
        crate::telegram::store::default_mode_for_priced(&mut *conn, event_type_id).await?;
    }
    if existed {
        return Ok(false);
    }
    sqlx::query(
        "INSERT INTO ton_prices (event_type_id, mode, currency, price_nano, price_usd_cents,
             window_min, enabled, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(event_type_id)
    .bind(mode)
    .bind(currency)
    .bind(nano)
    .bind(cents)
    .bind(window_min)
    .bind(i64::from(enabled))
    .bind(stamp(now))
    .execute(&mut *conn)
    .await?;
    Ok(true)
}

/// Any enabled price at all (the watcher then keeps its heartbeat fresh).
pub(crate) async fn any_enabled_price(pool: &SqlitePool) -> bool {
    sqlx::query_scalar::<_, i64>("SELECT EXISTS (SELECT 1 FROM ton_prices WHERE enabled = 1)")
        .fetch_one(pool)
        .await
        .map(|v| v != 0)
        .unwrap_or(true)
}

/// Any enabled price in USD: only then is a rate ever fetched. A TON price
/// needs none.
pub(crate) async fn any_enabled_usd_price(pool: &SqlitePool) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM ton_prices WHERE enabled = 1 AND currency = 'USD')",
    )
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
    .unwrap_or(false)
}

// --- the demand ladder (ilia fork) --------------------------------------------

/// The demand `n` of `owner_user_id` at `now`: the bookings they host that
/// start between now and the end of the booking horizon
/// ([`crate::horizon::max_days_ahead`]), on every event type, confirmed
/// (paid, waived and free ones) or pending without an unsettled invoice
/// (free requests awaiting the owner's approval, free ladder steps
/// included, and paid or waived bookings not confirmed yet).
/// Not counted: payment holds (an open invoice; anyone could push the
/// price up), cancelled and declined bookings, the past, the owner's
/// calendar busy time, other users' bookings, and `exclude_booking_id`
/// (the booking being made). "Hosts": the assigned member, else the
/// owner of the event type, else (a collective team type) any member, as
/// for busy time (`booking_time::user_busy_bookings`).
///
/// Runs on a caller's connection: inside the booking transaction, after
/// its INSERT took the write lock, it sees every committed booking and
/// none can commit meanwhile.
pub(crate) async fn demand_n(
    conn: &mut SqliteConnection,
    owner_user_id: &str,
    now: DateTime<Utc>,
    exclude_booking_id: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let until = now + chrono::Duration::days(crate::horizon::max_days_ahead());
    let iso = |t: DateTime<Utc>| t.format("%Y-%m-%dT%H:%M:%S").to_string();
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM bookings b
         JOIN event_types et ON et.id = b.event_type_id
         JOIN accounts a ON a.id = et.account_id
         WHERE (b.assigned_user_id = ?
                OR (b.assigned_user_id IS NULL AND a.user_id = ?)
                OR (b.assigned_user_id IS NULL AND et.team_id IS NOT NULL AND EXISTS (
                      SELECT 1 FROM team_members tm
                      WHERE tm.team_id = et.team_id AND tm.user_id = ?)))
           AND b.start_at >= ? AND b.start_at < ?
           AND b.id != ?
           AND (b.status = 'confirmed'
                OR (b.status = 'pending'
                    AND NOT EXISTS (SELECT 1 FROM ton_invoices ti WHERE ti.booking_id = b.id
                                    AND ti.status NOT IN ('paid', 'confirmed', 'waived'))))",
    )
    .bind(owner_user_id)
    .bind(owner_user_id)
    .bind(owner_user_id)
    .bind(iso(now))
    .bind(iso(until))
    .bind(exclude_booking_id.unwrap_or(""))
    .fetch_one(&mut *conn)
    .await
}

/// The TON settings stored in the database (`ton_settings`), the keys of
/// [`crate::ton::settings::KEYS`] only. Read on a caller's connection.
pub(crate) async fn load_settings(
    conn: &mut SqliteConnection,
) -> Result<crate::ton::settings::DbValues, sqlx::Error> {
    let rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT key, value, owner FROM ton_settings")
            .fetch_all(&mut *conn)
            .await?;
    Ok(crate::ton::settings::DbValues(
        rows.into_iter()
            .filter(|(k, _, _)| crate::ton::settings::KEYS.contains(&k.as_str()))
            .filter_map(|(k, v, o)| Some((k, (v, crate::ton::settings::Owner::parse(&o)?))))
            .collect(),
    ))
}

/// Who asks for a free ladder step: the per-guest limit matches any of
/// these.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FreeGuest {
    pub(crate) email_norm: String,
    /// The typed Telegram username, lower-cased without `@`.
    pub(crate) telegram_norm: Option<String>,
    pub(crate) client_tag: String,
}

/// A free step was taken by this guest (email, Telegram username or client
/// network) since `since`, cancelled ones included.
pub(crate) async fn free_step_taken(
    conn: &mut SqliteConnection,
    guest: &FreeGuest,
    since: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM ton_free_bookings
                        WHERE created_at >= ?
                          AND (email_norm = ? OR client_tag = ?
                               OR (? IS NOT NULL AND telegram_norm = ?)))",
    )
    .bind(stamp(since))
    .bind(&guest.email_norm)
    .bind(&guest.client_tag)
    .bind(&guest.telegram_norm)
    .bind(&guest.telegram_norm)
    .fetch_one(&mut *conn)
    .await
    .map(|v| v != 0)
}

/// Record a free step, inside the booking's transaction.
pub(crate) async fn record_free_step(
    conn: &mut SqliteConnection,
    booking_id: &str,
    event_type_id: &str,
    guest: &FreeGuest,
    demand_n: i64,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO ton_free_bookings (booking_id, event_type_id, email_norm, telegram_norm,
             client_tag, demand_n, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(booking_id)
    .bind(event_type_id)
    .bind(&guest.email_norm)
    .bind(&guest.telegram_norm)
    .bind(&guest.client_tag)
    .bind(demand_n)
    .bind(stamp(now))
    .execute(&mut *conn)
    .await
    .map(|_| ())
}

/// An invoice counts toward the daily budget once it received money or
/// lived this long: a booking cancelled (or refused) right away, which
/// costs its maker nothing, does not use up everybody's budget.
pub(crate) const BUDGET_MIN_AGE_MIN: i64 = 10;

/// Any price row at all, switched on or not.
pub(crate) async fn any_price(pool: &SqlitePool) -> bool {
    sqlx::query_scalar::<_, i64>("SELECT EXISTS (SELECT 1 FROM ton_prices)")
        .fetch_one(pool)
        .await
        .map(|v| v != 0)
        .unwrap_or(true)
}

/// Open invoices per guest email, per client network, and in total;
/// invoices of the last day that count toward the daily budget; and every
/// invoice of the last day from this client network.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct OpenCounts {
    pub(crate) by_email: i64,
    pub(crate) by_tag: i64,
    pub(crate) total: i64,
    pub(crate) last_day: i64,
    pub(crate) tag_day: i64,
}

pub(crate) async fn open_counts(
    conn: &mut SqliteConnection,
    email_norm: &str,
    client_tag: &str,
    now: DateTime<Utc>,
) -> Result<OpenCounts, sqlx::Error> {
    let day_ago = stamp(now - chrono::Duration::hours(24));
    let row: (i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM ton_invoices WHERE status = 'open' AND guest_email_norm = ?),
            (SELECT COUNT(*) FROM ton_invoices WHERE status = 'open' AND client_tag = ?),
            (SELECT COUNT(*) FROM ton_invoices WHERE status = 'open'),
            (SELECT COUNT(*) FROM ton_invoices WHERE created_at >= ?
                AND (paid_nano > 0
                     OR (julianday(COALESCE(closed_at, ?)) - julianday(created_at)) * 1440 >= ?)),
            (SELECT COUNT(*) FROM ton_invoices WHERE client_tag = ? AND created_at >= ?)",
    )
    .bind(email_norm)
    .bind(client_tag)
    .bind(&day_ago)
    .bind(stamp(now))
    .bind(BUDGET_MIN_AGE_MIN)
    .bind(client_tag)
    .bind(&day_ago)
    .fetch_one(&mut *conn)
    .await?;
    Ok(OpenCounts {
        by_email: row.0,
        by_tag: row.1,
        total: row.2,
        last_day: row.3,
        tag_day: row.4,
    })
}

/// Forget the client networks of invoices older than two days (the limits
/// look back one day).
pub(crate) async fn purge_client_tags(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    // ilia fork: free ladder steps are kept as long as the longest
    // per-guest limit, then forgotten.
    sqlx::query("DELETE FROM ton_free_bookings WHERE created_at < ?")
        .bind(stamp(
            now - chrono::Duration::days(crate::ton::settings::FREE_PER_GUEST_DAYS_MAX + 1),
        ))
        .execute(pool)
        .await?;
    sqlx::query(
        "UPDATE ton_invoices SET client_tag = NULL WHERE client_tag IS NOT NULL AND created_at < ?",
    )
    .bind(stamp(now - chrono::Duration::hours(48)))
    .execute(pool)
    .await
    .map(|r| r.rows_affected())
}

/// The newest invoice priced from two agreeing sources, as an anchor for
/// the one-source rule after a restart.
pub(crate) async fn last_confirmed_rate(pool: &SqlitePool) -> Option<(u64, DateTime<Utc>)> {
    let row: Option<(i64, String)> = sqlx::query_as(
        "SELECT rate_usd_micro, rate_at FROM ton_invoices
         WHERE rate_sources LIKE '%;%' ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let (rate, at) = row?;
    Some((u64::try_from(rate).ok()?, parse_stamp(&at)?))
}

/// An invoice as the watcher and the guards read it.
#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct InvoiceRow {
    pub(crate) id: String,
    pub(crate) booking_id: String,
    pub(crate) network: String,
    pub(crate) address_raw: String,
    pub(crate) status: String,
    pub(crate) problem: Option<String>,
    pub(crate) amount_nano: i64,
    pub(crate) tolerance_nano: i64,
    pub(crate) paid_nano: i64,
    pub(crate) excess_nano: i64,
    pub(crate) created_at: String,
    pub(crate) expires_at: String,
    pub(crate) settle_by: String,
    pub(crate) hard_deadline: String,
    /// ilia fork: a test booking's own timing ([`DEBUG_TIMING_SQL`]).
    pub(crate) debug_timing: Option<String>,
}

/// ilia fork: `ton_invoices.debug_timing` of an invoice whose booking is
/// marked as a test (`debug_bookings`), else NULL: a timing on any other
/// invoice (a hand edit) is never honoured. For a query over
/// `ton_invoices` under its own name or the alias `i`; `{t}` is replaced.
const DEBUG_TIMING_SQL: &str = "CASE WHEN EXISTS (SELECT 1 FROM debug_bookings d \
     WHERE d.booking_id = {t}.booking_id) THEN {t}.debug_timing END";

/// [`DEBUG_TIMING_SQL`] for the table (or alias) `t`.
pub(crate) fn debug_timing_sql(t: &str) -> String {
    DEBUG_TIMING_SQL.replace("{t}", t)
}

/// The test booking's own timing of an invoice row, read by
/// [`debug_timing_sql`]; `None` for the site's rules (unset, or not a
/// timing: logged).
pub(crate) fn parse_debug_timing(
    invoice_id: &str,
    text: Option<&str>,
) -> Option<crate::ton::deadlines::DebugTiming> {
    let text = text?;
    let parsed = crate::ton::deadlines::DebugTiming::parse(text);
    if parsed.is_none() {
        tracing::warn!(target: super::TARGET, invoice = %invoice_id, "a debug timing cannot be read; the site's rules apply");
    }
    parsed
}

fn invoice_cols() -> String {
    format!(
        "id, booking_id, network, address_raw, status, problem, amount_nano, \
         tolerance_nano, paid_nano, excess_nano, created_at, expires_at, settle_by, hard_deadline, \
         {} AS debug_timing",
        debug_timing_sql("ton_invoices")
    )
}

impl InvoiceRow {
    /// What the decision table needs. `None` for a row it cannot read,
    /// which the caller treats as no invoice (the payment is then
    /// recorded unmatched, for the owner).
    pub(crate) fn view(&self) -> Option<InvoiceView> {
        Some(InvoiceView {
            network: Network::parse(&self.network)?,
            address_raw: self.address_raw.clone(),
            status: InvoiceStatus::parse(&self.status)?,
            created_unix: parse_stamp(&self.created_at)?.timestamp(),
            settle_by_unix: parse_stamp(&self.settle_by)?.timestamp(),
        })
    }
}

pub(crate) async fn invoice_by_reference(
    conn: &mut SqliteConnection,
    reference: &str,
) -> Result<Option<InvoiceRow>, sqlx::Error> {
    sqlx::query_as::<_, InvoiceRow>(&format!(
        "SELECT {} FROM ton_invoices WHERE reference = ?",
        invoice_cols()
    ))
    .bind(reference)
    .fetch_optional(&mut *conn)
    .await
}

pub(crate) async fn invoice_for_booking(
    pool: &SqlitePool,
    booking_id: &str,
) -> Result<Option<InvoiceRow>, sqlx::Error> {
    sqlx::query_as::<_, InvoiceRow>(&format!(
        "SELECT {} FROM ton_invoices WHERE booking_id = ?",
        invoice_cols()
    ))
    .bind(booking_id)
    .fetch_optional(pool)
    .await
}

pub(crate) async fn invoices_with_status(
    pool: &SqlitePool,
    network: Network,
    status: &str,
) -> Result<Vec<InvoiceRow>, sqlx::Error> {
    sqlx::query_as::<_, InvoiceRow>(&format!(
        "SELECT {} FROM ton_invoices WHERE status = ? AND network = ? ORDER BY created_at",
        invoice_cols()
    ))
    .bind(status)
    .bind(network.as_str())
    .fetch_all(pool)
    .await
}

/// A payment row to insert.
#[derive(Clone, Debug)]
pub(crate) struct NewPayment<'a> {
    pub(crate) tx_hex: &'a str,
    pub(crate) network: Network,
    pub(crate) address_raw: &'a str,
    pub(crate) lt: u64,
    pub(crate) utime: u64,
    pub(crate) mc_block_seqno: u64,
    pub(crate) source_raw: Option<&'a str>,
    pub(crate) source_friendly: Option<&'a str>,
    pub(crate) source_is_wallet: Option<bool>,
    pub(crate) value_nano: i64,
    pub(crate) comment: Option<String>,
    pub(crate) invoice_id: Option<&'a str>,
    pub(crate) outcome: &'a str,
    pub(crate) reason: Option<&'a str>,
    pub(crate) alert_due: bool,
    pub(crate) seen_at: DateTime<Utc>,
}

/// `INSERT … ON CONFLICT DO NOTHING`: 1 when the row is new, 0 for a
/// transaction already recorded (by hash, or by network+address+lt).
pub(crate) async fn insert_payment(
    conn: &mut SqliteConnection,
    p: &NewPayment<'_>,
) -> Result<u64, sqlx::Error> {
    let to_i64 = |v: u64| i64::try_from(v).unwrap_or(i64::MAX);
    sqlx::query(
        "INSERT INTO ton_payments (tx_hex, network, address_raw, lt, utime, mc_block_seqno,
             source_raw, source_friendly, source_is_wallet, value_nano, comment, invoice_id,
             outcome, reason, linked_by, alert_due, seen_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT DO NOTHING",
    )
    .bind(p.tx_hex)
    .bind(p.network.as_str())
    .bind(p.address_raw)
    .bind(to_i64(p.lt))
    .bind(to_i64(p.utime))
    .bind(to_i64(p.mc_block_seqno))
    .bind(p.source_raw)
    .bind(p.source_friendly)
    .bind(p.source_is_wallet.map(i64::from))
    .bind(p.value_nano)
    .bind(&p.comment)
    .bind(p.invoice_id)
    .bind(p.outcome)
    .bind(p.reason)
    .bind(p.invoice_id.map(|_| "auto"))
    .bind(i64::from(p.alert_due))
    .bind(stamp(p.seen_at))
    .execute(&mut *conn)
    .await
    .map(|r| r.rows_affected())
}

/// What a credit did to its invoice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Credit {
    /// Still short of the amount.
    Partial,
    /// Fully paid now; `excess` over the amount.
    Paid { excess: i64 },
    /// The invoice was not open (cannot happen under the write lock).
    NotOpen,
}

/// Add `value` to an open invoice; settle it when it reaches the amount
/// (less the tolerance). Overpayment is recorded as excess and a problem.
pub(crate) async fn credit_invoice(
    conn: &mut SqliteConnection,
    invoice_id: &str,
    value: i64,
    now: DateTime<Utc>,
) -> Result<Credit, sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE ton_invoices SET paid_nano = paid_nano + ? WHERE id = ? AND status = 'open'",
    )
    .bind(value)
    .bind(invoice_id)
    .execute(&mut *conn)
    .await?
    .rows_affected();
    if updated == 0 {
        return Ok(Credit::NotOpen);
    }
    let (paid, amount, tolerance): (i64, i64, i64) = sqlx::query_as(
        "SELECT paid_nano, amount_nano, tolerance_nano FROM ton_invoices WHERE id = ?",
    )
    .bind(invoice_id)
    .fetch_one(&mut *conn)
    .await?;
    if paid.saturating_add(tolerance) < amount {
        return Ok(Credit::Partial);
    }
    let excess = (paid - amount).max(0);
    sqlx::query(
        "UPDATE ton_invoices SET status = 'paid', paid_at = ?, excess_nano = ?,
             problem = CASE WHEN ? > 0 THEN 'overpaid' ELSE problem END
         WHERE id = ? AND status = 'open'",
    )
    .bind(stamp(now))
    .bind(excess)
    .bind(excess)
    .bind(invoice_id)
    .execute(&mut *conn)
    .await?;
    Ok(Credit::Paid { excess })
}

pub(crate) async fn mark_payment_alert(
    conn: &mut SqliteConnection,
    tx_hex: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE ton_payments SET alert_due = 1 WHERE tx_hex = ?")
        .bind(tx_hex)
        .execute(&mut *conn)
        .await
        .map(|_| ())
}

/// The cursor row of one watched address.
#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct WatchRow {
    pub(crate) cursor_lt: i64,
    pub(crate) stuck_lt: Option<i64>,
    pub(crate) stuck_since: Option<String>,
    pub(crate) caught_up_at: Option<String>,
}

pub(crate) async fn watch_row(
    conn: &mut SqliteConnection,
    network: Network,
    address_raw: &str,
) -> Result<Option<WatchRow>, sqlx::Error> {
    sqlx::query_as::<_, WatchRow>(
        "SELECT cursor_lt, stuck_lt, stuck_since, caught_up_at FROM ton_watch
         WHERE network = ? AND address_raw = ?",
    )
    .bind(network.as_str())
    .bind(address_raw)
    .fetch_optional(&mut *conn)
    .await
}

/// The first statement of every watcher transaction: a write that takes
/// SQLite's write lock at once, changing nothing.
pub(crate) async fn lock_watch(
    conn: &mut SqliteConnection,
    network: Network,
    address_raw: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE ton_watch SET failures = failures WHERE network = ? AND address_raw = ?")
        .bind(network.as_str())
        .bind(address_raw)
        .execute(&mut *conn)
        .await
        .map(|_| ())
}

/// Create the row of the configured address (first run) and mark it the
/// current one; every other row stops being current. An existing row keeps
/// its cursor.
pub(crate) async fn ensure_current(
    pool: &SqlitePool,
    network: Network,
    address_raw: &str,
    first_cursor: Option<(u64, String)>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let (lt, hex) = first_cursor.unwrap_or((0, String::new()));
    sqlx::query(
        "INSERT INTO ton_watch (network, address_raw, is_current, cursor_lt, cursor_tx_hex)
         VALUES (?, ?, 1, ?, ?) ON CONFLICT (network, address_raw) DO NOTHING",
    )
    .bind(network.as_str())
    .bind(address_raw)
    .bind(i64::try_from(lt).unwrap_or(i64::MAX))
    .bind((!hex.is_empty()).then_some(hex))
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE ton_watch SET is_current = CASE WHEN network = ? AND address_raw = ? THEN 1 ELSE 0 END",
    )
    .bind(network.as_str())
    .bind(address_raw)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

pub(crate) async fn has_watch_row(
    pool: &SqlitePool,
    network: Network,
    address_raw: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM ton_watch WHERE network = ? AND address_raw = ?)",
    )
    .bind(network.as_str())
    .bind(address_raw)
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
}

/// Store the cursor reached by a page (inside the page's transaction), and
/// where it stopped, if it did.
pub(crate) async fn save_cursor(
    conn: &mut SqliteConnection,
    network: Network,
    address_raw: &str,
    cursor: Option<(u64, &str)>,
    stuck: Option<(u64, DateTime<Utc>)>,
    caught_up_at: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    if let Some((lt, hex)) = cursor {
        sqlx::query(
            "UPDATE ton_watch SET cursor_lt = ?, cursor_tx_hex = ?
             WHERE network = ? AND address_raw = ? AND cursor_lt < ?",
        )
        .bind(i64::try_from(lt).unwrap_or(i64::MAX))
        .bind(hex)
        .bind(network.as_str())
        .bind(address_raw)
        .bind(i64::try_from(lt).unwrap_or(i64::MAX))
        .execute(&mut *conn)
        .await?;
    }
    let (stuck_lt, stuck_since) = match stuck {
        Some((lt, since)) => (
            Some(i64::try_from(lt).unwrap_or(i64::MAX)),
            Some(stamp(since)),
        ),
        None => (None, None),
    };
    sqlx::query(
        "UPDATE ton_watch SET stuck_lt = ?, stuck_since = ?,
             caught_up_at = COALESCE(?, caught_up_at)
         WHERE network = ? AND address_raw = ?",
    )
    .bind(stuck_lt)
    .bind(stuck_since)
    .bind(caught_up_at.map(stamp))
    .bind(network.as_str())
    .bind(address_raw)
    .execute(&mut *conn)
    .await
    .map(|_| ())
}

/// After a step: the heartbeat and the indexer height on success, or the
/// error class and one more failure.
pub(crate) async fn record_step(
    pool: &SqlitePool,
    network: Network,
    address_raw: &str,
    now: DateTime<Utc>,
    mc: Option<(u64, i64)>,
    error: Option<&str>,
) -> Result<(), sqlx::Error> {
    match error {
        None => sqlx::query(
            "UPDATE ton_watch SET last_ok_at = ?, failures = 0,
                 indexed_utime = COALESCE(?, indexed_utime), global_id = COALESCE(?, global_id)
             WHERE network = ? AND address_raw = ?",
        )
        .bind(stamp(now))
        .bind(mc.map(|m| i64::try_from(m.0).unwrap_or(i64::MAX)))
        .bind(mc.map(|m| m.1))
        .bind(network.as_str())
        .bind(address_raw)
        .execute(pool)
        .await
        .map(|_| ()),
        Some(class) => sqlx::query(
            "UPDATE ton_watch SET last_error = ?, last_error_at = ?, failures = failures + 1,
                 global_id = COALESCE(?, global_id)
             WHERE network = ? AND address_raw = ?",
        )
        .bind(class)
        .bind(stamp(now))
        .bind(mc.map(|m| m.1))
        .bind(network.as_str())
        .bind(address_raw)
        .execute(pool)
        .await
        .map(|_| ()),
    }
}

pub(crate) async fn record_account(
    pool: &SqlitePool,
    network: Network,
    address_raw: &str,
    status: &str,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE ton_watch SET account_status = ?, account_checked_at = ?
         WHERE network = ? AND address_raw = ?",
    )
    .bind(status)
    .bind(stamp(now))
    .bind(network.as_str())
    .bind(address_raw)
    .execute(pool)
    .await
    .map(|_| ())
}

/// The addresses to poll: the configured one, and every address with an
/// open or paid invoice on this network (an older address keeps being
/// watched until its invoices settle).
pub(crate) async fn watched_addresses(
    pool: &SqlitePool,
    network: Network,
    current: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let others: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT address_raw FROM ton_invoices
         WHERE status IN ('open', 'paid') AND network = ? AND address_raw != ?
         ORDER BY address_raw",
    )
    .bind(network.as_str())
    .bind(current)
    .fetch_all(pool)
    .await?;
    let mut all = vec![current.to_string()];
    all.extend(others);
    Ok(all)
}

/// ilia fork: open invoices, still owing, renewed since the guest was last
/// told: (id, address_raw, renewed_at).
/// Its rows: (invoice id, address, renewed_at, the test booking's own
/// timing as [`debug_timing_sql`] reads it).
#[allow(clippy::type_complexity)]
pub(crate) async fn due_renewal_notices(
    pool: &SqlitePool,
) -> Result<Vec<(String, String, Option<String>, Option<String>)>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT id, address_raw, renewed_at, {} FROM ton_invoices
         WHERE status = 'open' AND renewal_notified < renewals AND paid_nano < amount_nano",
        debug_timing_sql("ton_invoices")
    ))
    .fetch_all(pool)
    .await
}

/// Claim the "pay again" notice of an invoice's latest renewal, once.
pub(crate) async fn claim_renewal_notice(pool: &SqlitePool, invoice_id: &str) -> bool {
    sqlx::query(
        "UPDATE ton_invoices SET renewal_notified = renewals
         WHERE id = ? AND status = 'open' AND renewal_notified < renewals",
    )
    .bind(invoice_id)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or(false)
}

/// Open or paid invoices exist (the watcher polls at full speed).
pub(crate) async fn active_invoices(pool: &SqlitePool) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM ton_invoices WHERE status IN ('open', 'paid'))",
    )
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
    .unwrap_or(true)
}

/// ilia fork: an open invoice of a test booking with its own timing exists
/// (the watcher keeps to a short step while its minutes run).
pub(crate) async fn open_debug_invoice(pool: &SqlitePool) -> bool {
    sqlx::query_scalar::<_, i64>(&format!(
        "SELECT EXISTS (SELECT 1 FROM ton_invoices i
          WHERE i.status = 'open' AND ({}) IS NOT NULL)",
        debug_timing_sql("i")
    ))
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
    .unwrap_or(false)
}

/// Claim a global alert of `kind` unless one went out in the last hour.
pub(crate) async fn claim_global_alert(
    pool: &SqlitePool,
    kind: &str,
    detail: &str,
    now: DateTime<Utc>,
) -> bool {
    let hour_ago = stamp(now - chrono::Duration::hours(1));
    sqlx::query(
        "INSERT INTO ton_alerts (kind, last_sent_at, detail) VALUES (?, ?, ?)
         ON CONFLICT (kind) DO UPDATE SET last_sent_at = excluded.last_sent_at,
             detail = excluded.detail
         WHERE ton_alerts.last_sent_at < ?",
    )
    .bind(kind)
    .bind(stamp(now))
    .bind(detail)
    .bind(hour_ago)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or(false)
}

/// Payment rows whose alert is due and unclaimed: (tx_hex, outcome,
/// reason, invoice_id).
#[allow(clippy::type_complexity)]
pub(crate) async fn due_payment_alerts(
    pool: &SqlitePool,
) -> Result<Vec<(String, String, Option<String>, Option<String>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT tx_hex, outcome, reason, invoice_id FROM ton_payments
         WHERE alert_due = 1 AND owner_alerted_at IS NULL ORDER BY lt LIMIT 50",
    )
    .fetch_all(pool)
    .await
}

/// A payment email about this invoice went out since `since`.
pub(crate) async fn payment_alerted_since(
    pool: &SqlitePool,
    invoice_id: &str,
    since: DateTime<Utc>,
) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM ton_payments
                        WHERE invoice_id = ? AND alert_held = 0 AND owner_alerted_at >= ?)",
    )
    .bind(invoice_id)
    .bind(stamp(since))
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
    .unwrap_or(false)
}

/// Claim a payment alert into the digest: no email of its own.
pub(crate) async fn hold_payment_alert(
    pool: &SqlitePool,
    tx_hex: &str,
    now: DateTime<Utc>,
) -> bool {
    sqlx::query(
        "UPDATE ton_payments SET owner_alerted_at = ?, alert_held = 1
         WHERE tx_hex = ? AND alert_due = 1 AND owner_alerted_at IS NULL",
    )
    .bind(stamp(now))
    .bind(tx_hex)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or(false)
}

/// Payment alerts claimed into the digest since `since`.
pub(crate) async fn held_payment_alerts_since(pool: &SqlitePool, since: DateTime<Utc>) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM ton_payments WHERE alert_held = 1 AND owner_alerted_at >= ?",
    )
    .bind(stamp(since))
    .fetch_one(pool)
    .await
    .unwrap_or(0)
}

pub(crate) async fn claim_payment_alert(
    pool: &SqlitePool,
    tx_hex: &str,
    now: DateTime<Utc>,
) -> bool {
    sqlx::query(
        "UPDATE ton_payments SET owner_alerted_at = ?
         WHERE tx_hex = ? AND alert_due = 1 AND owner_alerted_at IS NULL",
    )
    .bind(stamp(now))
    .bind(tx_hex)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or(false)
}

/// Invoices whose problem the owner has not been told about: (id, problem).
pub(crate) async fn due_problem_alerts(
    pool: &SqlitePool,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, problem FROM ton_invoices
         WHERE problem IS NOT NULL
           AND (owner_alerted_problem IS NULL OR owner_alerted_problem != problem)
         ORDER BY created_at LIMIT 50",
    )
    .fetch_all(pool)
    .await
}

pub(crate) async fn claim_problem_alert(
    pool: &SqlitePool,
    invoice_id: &str,
    problem: &str,
    now: DateTime<Utc>,
) -> bool {
    sqlx::query(
        "UPDATE ton_invoices SET owner_alerted_problem = problem, owner_alerted_at = ?
         WHERE id = ? AND problem = ?
           AND (owner_alerted_problem IS NULL OR owner_alerted_problem != problem)",
    )
    .bind(stamp(now))
    .bind(invoice_id)
    .bind(problem)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or(false)
}

/// Confirmed invoices with money on them whose "payment received" is unsent.
/// Every watcher step asks: the partial index holds only the unsent ones
/// (ilia_007), where `(status, created_at)` would walk every confirmed
/// invoice ever paid, and the planner would pick that one unasked.
pub(crate) async fn due_received_notices(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT id FROM ton_invoices INDEXED BY idx_ton_invoices_paid_notice
         WHERE status = 'confirmed' AND paid_nano > 0 AND paid_notice_sent_at IS NULL
         ORDER BY created_at LIMIT 50",
    )
    .fetch_all(pool)
    .await
}

pub(crate) async fn claim_received_notice(
    pool: &SqlitePool,
    invoice_id: &str,
    now: DateTime<Utc>,
) -> bool {
    sqlx::query(
        "UPDATE ton_invoices SET paid_notice_sent_at = ?
         WHERE id = ? AND status = 'confirmed' AND paid_notice_sent_at IS NULL",
    )
    .bind(stamp(now))
    .bind(invoice_id)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or(false)
}
