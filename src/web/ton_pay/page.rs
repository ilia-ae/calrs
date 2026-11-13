//! The guest's payment page, `/booking/pay/{pay_token}`, its status JSON,
//! "I have paid" and "Email me this link".
//!
//! The pay token is its own 128-bit secret, so the link can be passed to
//! whoever pays without giving them the right to cancel. The page shows the
//! meeting, the amount, the address and the comment code, never the guest's
//! details and never anything written on chain by a sender. It is not
//! frameable (`/booking` is outside the embed surface) and is sent with
//! `Referrer-Policy: no-referrer` and `Cache-Control: no-store`.

use axum::extract::{Form, Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use chrono::{DateTime, Utc};
use minijinja::context;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::alerts::date_and_time;
use super::{parse_stamp, stamp, TARGET};
use crate::email::TonGuestMail;
use crate::ton::address::TonAddress;
use crate::ton::amount::{format_ton, format_usd_cents, from_i64};
use crate::ton::{link, qr, reference, Network};
use crate::web::{
    client_ip_for_rate_limit, internal_error_body, internal_error_response, verify_csrf_token,
    AppState, CsrfForm,
};

/// An invoice with what the page needs of its booking.
#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct PayRow {
    pub(crate) id: String,
    pub(crate) booking_id: String,
    pub(crate) network: String,
    pub(crate) address_raw: String,
    pub(crate) event_title: String,
    pub(crate) start_at_utc: String,
    /// `TON`: the amount is the price, the rate columns are neutral.
    pub(crate) currency: String,
    pub(crate) price_usd_cents: i64,
    pub(crate) rate_usd_micro: i64,
    pub(crate) rate_sources: String,
    pub(crate) rate_at: String,
    pub(crate) amount_nano: i64,
    pub(crate) paid_nano: i64,
    pub(crate) status: String,
    pub(crate) problem: Option<String>,
    pub(crate) expires_at: String,
    pub(crate) settle_by: String,
    pub(crate) reference: String,
    pub(crate) link_emailed_at: Option<String>,
    pub(crate) event_type_id: String,
    pub(crate) booking_status: Option<String>,
    pub(crate) booking_start: Option<String>,
    pub(crate) booking_tz: Option<String>,
    pub(crate) booking_lang: Option<String>,
    pub(crate) time_version: Option<i64>,
    pub(crate) cancel_token: Option<String>,
    pub(crate) booking_uid: Option<String>,
    pub(crate) renewals: i64,
    pub(crate) renewals_max: i64,
}

pub(crate) async fn load(pool: &SqlitePool, pay_token: &str) -> Option<PayRow> {
    // Tokens are 32 lowercase hex characters; anything else names nothing.
    if pay_token.len() != 32
        || !pay_token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    sqlx::query_as::<_, PayRow>(
        "SELECT i.id, i.booking_id, i.network, i.address_raw, i.event_title, i.start_at_utc,
                i.currency, i.price_usd_cents, i.rate_usd_micro, i.rate_sources, i.rate_at, i.amount_nano,
                i.paid_nano, i.status, i.problem, i.expires_at, i.settle_by, i.reference,
                i.link_emailed_at, i.event_type_id,
                b.status AS booking_status, b.start_at AS booking_start,
                b.guest_timezone AS booking_tz, b.language AS booking_lang,
                b.time_version, b.cancel_token, b.uid AS booking_uid,
                i.renewals, i.renewals_max
         FROM ton_invoices i LEFT JOIN bookings b ON b.id = i.booking_id
         WHERE i.pay_token = ?",
    )
    .bind(pay_token)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::error!(target: TARGET, error = %e, "cannot read an invoice for its page");
        None
    })
}

/// What the guest sees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PayState {
    /// Open, nothing arrived yet.
    Waiting,
    /// A matching transfer is visible but not final.
    Seen,
    /// Part of the amount arrived.
    Partial,
    /// The window closed; a payment sent in time may still be counted.
    Checking,
    /// Paid; the confirmation is under way.
    Paid,
    /// Paid, but the host is no longer free.
    Busy,
    Confirmed,
    /// The host confirmed it without payment, the confirmation is pending.
    Waived,
    Expired,
    /// Paid after the deadline.
    Late,
    /// Cancelled or declined.
    Closed,
    /// The owner revoked the payment address.
    Revoked,
    /// The host's time was taken before anything was paid: released.
    Taken,
    /// Money arrived, but the booking ended unconfirmed: it goes back.
    Refund,
}

impl PayState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Seen => "seen",
            Self::Partial => "partial",
            Self::Checking => "checking",
            Self::Paid => "paid",
            Self::Busy => "busy",
            Self::Confirmed => "confirmed",
            Self::Waived => "waived",
            Self::Expired => "expired",
            Self::Late => "late",
            Self::Closed => "closed",
            Self::Revoked => "revoked",
            Self::Taken => "taken",
            Self::Refund => "refund",
        }
    }

    /// The payment details (address, link, QR) are shown.
    pub(crate) fn can_pay(self) -> bool {
        matches!(self, Self::Waiting | Self::Seen | Self::Partial)
    }

    /// The page keeps refreshing itself. `Busy` too: the owner settles it
    /// (confirms by hand, or moves the booking), and the open page follows.
    pub(crate) fn live(self) -> bool {
        matches!(
            self,
            Self::Waiting
                | Self::Seen
                | Self::Partial
                | Self::Checking
                | Self::Paid
                | Self::Waived
                | Self::Busy
        )
    }

    /// The guest may need the host (a refund, a late payment): the page
    /// shows how to reach them.
    pub(crate) fn shows_contact(self) -> bool {
        matches!(
            self,
            Self::Busy | Self::Expired | Self::Late | Self::Taken | Self::Refund
        )
    }
}

/// The state from the invoice and its booking. Pure.
pub(crate) fn state_of(row: &PayRow, seen: bool, late: bool, now: DateTime<Utc>) -> PayState {
    let booking = row.booking_status.as_deref();
    let expires = parse_stamp(&row.expires_at).unwrap_or(DateTime::UNIX_EPOCH);
    let paid_some = row.paid_nano > 0;
    // Ended unconfirmed: whatever arrived goes back.
    let ended = |nothing_paid: PayState| {
        if paid_some {
            PayState::Refund
        } else {
            nothing_paid
        }
    };
    match row.status.as_str() {
        "open" => {
            if booking != Some("pending") {
                ended(PayState::Closed)
            } else if now > expires {
                PayState::Checking
            } else if paid_some {
                PayState::Partial
            } else if seen {
                PayState::Seen
            } else {
                PayState::Waiting
            }
        }
        "paid" if row.problem.as_deref() == Some("host_busy") => PayState::Busy,
        "paid" | "confirmed" => match booking {
            Some("confirmed") => PayState::Confirmed,
            Some("pending") if row.status == "paid" => PayState::Paid,
            _ => ended(PayState::Closed),
        },
        "waived" => match booking {
            Some("confirmed") => PayState::Confirmed,
            Some("pending") => PayState::Waived,
            _ => ended(PayState::Closed),
        },
        "expired" if late => PayState::Late,
        "expired" => PayState::Expired,
        "closed" if row.problem.as_deref() == Some("address_revoked") => PayState::Revoked,
        "closed" if row.problem.as_deref() == Some("host_busy_unpaid") => ended(PayState::Taken),
        _ if late => PayState::Late,
        _ => ended(PayState::Closed),
    }
}

async fn has_late_payment(pool: &SqlitePool, invoice_id: &str) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM ton_payments WHERE invoice_id = ? AND outcome = 'late')",
    )
    .bind(invoice_id)
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
    .unwrap_or(false)
}

async fn current_state(state: &AppState, row: &PayRow, now: DateTime<Utc>) -> PayState {
    let seen = state.ton.get().is_some_and(|rt| {
        rt.seen
            .lock()
            .map(|s| s.contains_key(&row.id))
            .unwrap_or(false)
    });
    let late = has_late_payment(&state.pool, &row.id).await;
    state_of(row, seen, late, now)
}

/// The booking's language, unless the guest picked one with the switcher.
fn page_lang(row: Option<&PayRow>, headers: &HeaderMap) -> &'static str {
    crate::i18n::page_lang(row.and_then(|r| r.booking_lang.as_deref()), headers)
}

fn private(mut resp: Response) -> Response {
    let h = resp.headers_mut();
    h.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    resp
}

/// "CoinGecko + Binance" from `coingecko=1.03;binance=1.04`.
fn source_names(sources: &str) -> String {
    sources
        .split(';')
        .filter_map(|p| p.split('=').next())
        .map(|name| match name {
            "coingecko" => "CoinGecko".to_string(),
            "binance" => "Binance".to_string(),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn remaining(row: &PayRow) -> u128 {
    from_i64(row.amount_nano).saturating_sub(from_i64(row.paid_nano))
}

/// What the amount still due is worth in US cents at the invoice's rate:
/// the price itself while nothing is paid, else the rest, rounded.
fn remaining_usd_cents(row: &PayRow) -> u128 {
    if row.paid_nano <= 0 {
        return from_i64(row.price_usd_cents);
    }
    // nano TON × micro-USD per TON / 1e9 = micro-USD; / 1e4 = cents.
    let micro_cents = remaining(row).saturating_mul(from_i64(row.rate_usd_micro));
    (micro_cents + 5_000_000_000_000) / 10_000_000_000_000
}

/// The address guests pay to, user-friendly: the one the invoice was made
/// for, whatever the configuration says now.
fn guest_address(row: &PayRow) -> Option<String> {
    let network = Network::parse(&row.network)?;
    let (addr, _) = TonAddress::parse(&row.address_raw).ok()?;
    Some(addr.for_guest(network))
}

/// The ways a wallet is handed the payment. Two links because no single one
/// works everywhere (`crate::ton::link`): Tonkeeper's carries `asset_id`,
/// which MyTonWallet refuses, and without which a multichain Tonkeeper
/// drops the amount. Each has its own QR code: Tonkeeper's scanner and the
/// phone's camera read the `https://` link, other wallets the `ton://` one.
#[derive(Default)]
struct PayLinks {
    ton: String,
    keeper: String,
    ton_qr: String,
    keeper_qr: String,
}

impl PayLinks {
    fn new(network: Network, address: &str, nano: u128, code: &str, exp_unix: i64) -> Self {
        let ton = link::transfer_uri(address, nano, code, exp_unix);
        let keeper = link::tonkeeper_uri(network, address, nano, code, exp_unix);
        Self {
            ton_qr: qr::svg(&ton).unwrap_or_default(),
            keeper_qr: qr::svg(&keeper).unwrap_or_default(),
            ton,
            keeper,
        }
    }
}

fn meeting_start(row: &PayRow) -> DateTime<Utc> {
    match (row.time_version, row.booking_start.as_deref()) {
        (Some(1), Some(s)) => parse_stamp(s),
        _ => None,
    }
    .or_else(|| parse_stamp(&row.start_at_utc))
    .unwrap_or(DateTime::UNIX_EPOCH)
}

#[derive(Deserialize, Default)]
pub(crate) struct PayQuery {
    #[serde(default)]
    sent: Option<String>,
    #[serde(default)]
    checked: Option<String>,
}

async fn limited(state: &AppState, headers: &HeaderMap) -> Option<Response> {
    let ip = client_ip_for_rate_limit(headers);
    if !state.ton.page_limiter.check_limited(&ip).await {
        return None;
    }
    let lang = crate::i18n::detect_from_headers(headers);
    let body = crate::i18n::translate(lang, "error-too-many-requests", None);
    Some(private(
        (StatusCode::TOO_MANY_REQUESTS, Html(body)).into_response(),
    ))
}

pub(crate) async fn pay_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(token): Path<String>,
    Query(query): Query<PayQuery>,
) -> Response {
    if let Some(resp) = limited(&state, &headers).await {
        return resp;
    }
    let Some(row) = load(&state.pool, &token).await else {
        return private(super::flow::not_found(&state, page_lang(None, &headers)));
    };
    let lang = page_lang(Some(&row), &headers);
    let now = Utc::now();
    let pay_state = current_state(&state, &row, now).await;
    let tz_name = row.booking_tz.clone().unwrap_or_else(|| "UTC".into());
    let (meeting_date, meeting_time) = date_and_time(lang, meeting_start(&row), &tz_name);
    let expires = parse_stamp(&row.expires_at).unwrap_or(now);
    let tz = tz_name.parse::<chrono_tz::Tz>().unwrap_or(chrono_tz::UTC);
    let held_until = expires.with_timezone(&tz).format("%H:%M").to_string();
    let rest = remaining(&row);
    let code = reference::display(&row.reference);
    let address = guest_address(&row).filter(|_| pay_state.can_pay());
    let links = match (&address, Network::parse(&row.network)) {
        (Some(a), Some(network)) => PayLinks::new(network, a, rest, &code, expires.timestamp()),
        _ => PayLinks::default(),
    };
    let rate_at = parse_stamp(&row.rate_at).unwrap_or(now);
    let usd = row.currency == "USD";
    let rebook_url = if matches!(
        pay_state,
        PayState::Expired
            | PayState::Late
            | PayState::Closed
            | PayState::Revoked
            | PayState::Taken
            | PayState::Refund
    ) {
        crate::web::event_type_rebook_url(&state.pool, &row.event_type_id).await
    } else {
        None
    };
    let (host_name, host_email) = match (pay_state.shows_contact(), row.booking_uid.as_deref()) {
        // ilia fork (`crate::inbox`): the booking's system reply address,
        // never the owner's own (none when no mailbox is read).
        (true, Some(uid)) => crate::web::booking_host_identity(&state.pool, uid)
            .await
            .map(|(n, e, _)| (n, crate::inbox::reply::guest_contact(Some(uid), e)))
            .unwrap_or_default(),
        _ => Default::default(),
    };
    // Offered only when it can work: a base URL for the link, and SMTP.
    let show_email = address.is_some()
        && row.link_emailed_at.is_none()
        && crate::settings::base_url().is_some()
        && matches!(
            crate::email::load_smtp_config(&state.pool, &state.secret_key).await,
            Ok(Some(_))
        );
    // ilia fork: the booking's Telegram link, while the booking stands.
    let telegram_link = super::telegram::pay_page_link(&state, &row.booking_id, &token).await;
    let tmpl = match state.templates.get_template("ton_pay.html") {
        Ok(t) => t,
        Err(e) => return internal_error_response("internal", &e),
    };
    let base = format!("/booking/pay/{token}");
    let rendered = tmpl
        .render(context! {
            lang => lang,
            event_title => &row.event_title,
            meeting_date => meeting_date,
            meeting_time => meeting_time,
            pay_state => pay_state.as_str(),
            can_pay => address.is_some(),
            live => pay_state.live(),
            amount_ton => format_ton(rest),
            total_ton => format_ton(from_i64(row.amount_nano)),
            paid_ton => format_ton(from_i64(row.paid_nano)),
            // A TON price shows no dollars and no rate.
            usd => usd.then(|| format_usd_cents(remaining_usd_cents(&row))),
            rate => usd.then(|| super::alerts::rate_display(row.rate_usd_micro)),
            rate_sources => usd.then(|| source_names(&row.rate_sources)),
            rate_time => usd.then(|| rate_at.format("%H:%M").to_string()),
            address => address.clone().unwrap_or_default(),
            comment => code,
            ton_link => links.ton,
            keeper_link => links.keeper,
            keeper_qr_svg => links.keeper_qr,
            qr_svg => links.ton_qr,
            held_until => held_until,
            zone => tz.name(),
            seconds_left => (expires - now).num_seconds().max(0),
            status_url => format!("{base}/status"),
            check_url => format!("{base}/check"),
            email_url => format!("{base}/email"),
            show_email => show_email,
            host_name => host_name,
            host_email => host_email,
            emailed => query.sent.as_deref() == Some("1"),
            email_failed => query.sent.as_deref() == Some("0"),
            checked => query.checked.is_some(),
            rebook_url => rebook_url,
            telegram_link => telegram_link,
            // ilia fork: "Add to calendar" once the booking is confirmed (a
            // waived one is still pending: the host has yet to confirm it).
            ics_url => (pay_state == PayState::Confirmed).then(|| format!("{base}/ics")),
            renewals => row.renewals,
            renewals_max => row.renewals_max,
            expires_at => row.expires_at,
            // A booking the admin's debug page made: TEST.
            test_booking => crate::web::debug_booking::is_marked(&state.pool, &row.booking_id).await,
        })
        .unwrap_or_else(|e| internal_error_body("template render", &e));
    private(Html(rendered).into_response())
}

/// `GET …/ics` (ilia fork): the booking's invite, the page's "Add to
/// calendar", while the booking is confirmed. Scoped to the pay token, so the
/// page never carries the cancel token behind upstream's `/booking/ics/…`
/// (whoever holds that can cancel). Without the guest's name, email or
/// notes, nor the host's address: the pay link may have been passed to
/// whoever pays. The meeting's place stays, as the page shows the meeting
/// (`crate::email::generate_ics_payer`).
pub(crate) async fn pay_ics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Response {
    if let Some(resp) = limited(&state, &headers).await {
        return resp;
    }
    let row = load(&state.pool, &token).await;
    let lang = page_lang(row.as_ref(), &headers);
    let Some(row) = row else {
        return private(super::flow::not_found(&state, lang));
    };
    if current_state(&state, &row, Utc::now()).await != PayState::Confirmed {
        return private(super::flow::not_found(&state, lang));
    }
    let Some(ics) = confirmed_ics(&state.pool, &row.booking_id).await else {
        return private(super::flow::not_found(&state, lang));
    };
    private(
        (
            [
                (header::CONTENT_TYPE, "text/calendar; charset=UTF-8"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"booking.ics\"",
                ),
            ],
            ics,
        )
            .into_response(),
    )
}

/// The invite of a confirmed booking, as upstream's `booking_ics` builds it
/// (same UID and SEQUENCE, so a calendar updates the event the email added),
/// but with no one's identity: no guest name, email or notes, no ATTENDEE or
/// ORGANIZER (`generate_ics_payer`). The place stays: the page shows the
/// meeting, and the invite is no use without it.
async fn confirmed_ics(pool: &SqlitePool, booking_id: &str) -> Option<String> {
    #[allow(clippy::type_complexity)]
    let (uid, guest_timezone, start_at, end_at, title, host_name, location, et_id): (
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        String,
    ) = sqlx::query_as(
        "SELECT b.uid, COALESCE(b.guest_timezone, 'UTC'),
                CASE b.time_version WHEN 1 THEN b.start_at ELSE rtrim(b.start_at, 'Z') END,
                CASE b.time_version WHEN 1 THEN b.end_at ELSE rtrim(b.end_at, 'Z') END,
                et.title, u.name,
                COALESCE(NULLIF(b.meeting_url, ''), et.location_value), et.id
         FROM bookings b
         JOIN event_types et ON et.id = b.event_type_id
         JOIN accounts a ON a.id = et.account_id
         JOIN users u ON u.id = COALESCE(b.assigned_user_id, a.user_id)
         WHERE b.id = ? AND b.status = 'confirmed'",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::error!(target: TARGET, error = %e, "cannot read a booking for its invite");
        None
    })?;
    let stored_tz = crate::web::availability::get_host_tz(pool, &et_id).await;
    let guest_tz = guest_timezone
        .parse::<chrono_tz::Tz>()
        .unwrap_or(chrono_tz::UTC);
    let (date, start_time, end_time) =
        crate::web::format::booking_strings_in_guest_tz(&start_at, &end_at, stored_tz, guest_tz);
    let details = crate::email::BookingDetails {
        utc_times: crate::booking_time::ics_times(&start_at, &end_at),
        event_title: title,
        date,
        start_time,
        end_time,
        guest_timezone,
        host_name,
        sequence: crate::web::calendar_push::booking_ics_sequence(pool, &uid).await,
        uid,
        location: location.filter(|l| !l.is_empty()),
        host_timezone: stored_tz.name().to_string(),
        ..Default::default()
    };
    Some(crate::email::generate_ics_payer(&details))
}

/// `GET …/status`: the state and what is left to pay, nothing else (no
/// guest, transaction or comment data).
pub(crate) async fn pay_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Response {
    if let Some(resp) = limited(&state, &headers).await {
        return resp;
    }
    let Some(row) = load(&state.pool, &token).await else {
        return private(
            (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "state": "unknown" })),
            )
                .into_response(),
        );
    };
    let now = Utc::now();
    let pay_state = current_state(&state, &row, now).await;
    let expires = parse_stamp(&row.expires_at).unwrap_or(now);
    let rest = remaining(&row);
    private(
        axum::Json(serde_json::json!({
            "state": pay_state.as_str(),
            "remaining_ton": format_ton(rest),
            "remaining_nano": rest.to_string(),
            "expires_at": row.expires_at,
            "settle_by": row.settle_by,
            "seconds_left": (expires - now).num_seconds().max(0),
            "renewals": row.renewals,
        }))
        .into_response(),
    )
}

/// `POST …/check`: wake the watcher now (at most once per 10 s per
/// invoice), then back to the page.
pub(in crate::web) async fn pay_check(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(token): Path<String>,
    Form(form): Form<CsrfForm>,
) -> Response {
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    let Some(row) = load(&state.pool, &token).await else {
        return private(super::flow::not_found(&state, page_lang(None, &headers)));
    };
    if !state.ton.check_limiter.check_limited(&row.id).await {
        if let Some(rt) = state.ton.get() {
            rt.wake.notify_one();
        }
    }
    Redirect::to(&format!("/booking/pay/{token}?checked=1")).into_response()
}

/// The guest email about a booking, written for its guest; also returns
/// the event type id. `None` for a booking that is gone.
pub(crate) async fn guest_mail(
    pool: &SqlitePool,
    booking_id: &str,
) -> Option<(TonGuestMail, String)> {
    #[allow(clippy::type_complexity)]
    let (uid, name, email, language, tz, start_at, time_version, et_id, title): (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        i64,
        String,
        String,
    ) = sqlx::query_as(
        "SELECT b.uid, b.guest_name, b.guest_email, b.language, b.guest_timezone, b.start_at,
                b.time_version, b.event_type_id, et.title
         FROM bookings b JOIN event_types et ON et.id = b.event_type_id WHERE b.id = ?",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    let lang = language
        .as_deref()
        .and_then(crate::i18n::supported_code)
        .unwrap_or("en");
    let (host_name, host_email) = crate::web::booking_host_identity(pool, &uid)
        .await
        .map(|(n, e, _)| (n, e))
        .unwrap_or_default();
    let start = if time_version == 1 {
        parse_stamp(&start_at)
    } else {
        None
    }
    .unwrap_or(DateTime::UNIX_EPOCH);
    let (date, time) = date_and_time(lang, start, tz.as_deref().unwrap_or("UTC"));
    Some((
        TonGuestMail {
            lang: lang.to_string(),
            guest_name: name,
            guest_email: email,
            host_name,
            host_email,
            booking_uid: uid.clone(),
            event_title: title,
            date,
            time,
            pay_by: String::new(),
        },
        et_id,
    ))
}

/// Email the guest the links of an open invoice (its payment page, the
/// booking's cancel link, and its Telegram link when its event type offers
/// Telegram, as the payment page shows it), to the address the booking was
/// made with, with when the time stops being held. No amount, address or
/// coin in it.
/// `false` when it cannot go out (no base URL, no SMTP, booking gone).
async fn email_links(
    state: &AppState,
    booking_id: &str,
    pay_token: &str,
    cancel_token: Option<&str>,
    expires_at: &str,
) -> bool {
    let (Some(base), Ok(Some(smtp))) = (
        crate::settings::base_url(),
        crate::email::load_smtp_config(&state.pool, &state.secret_key).await,
    ) else {
        return false;
    };
    let Some((mut mail, _)) = guest_mail(&state.pool, booking_id).await else {
        return false;
    };
    let tz: Option<String> = sqlx::query_scalar("SELECT guest_timezone FROM bookings WHERE id = ?")
        .bind(booking_id)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
        .flatten();
    if let Some(expires) = parse_stamp(expires_at) {
        let (date, time) = date_and_time(&mail.lang, expires, tz.as_deref().unwrap_or("UTC"));
        mail.pay_by = format!("{date}, {time}");
    }
    let base = base.trim_end_matches('/').to_string();
    let pay_url = format!("{base}/booking/pay/{pay_token}");
    let cancel_url = cancel_token.map(|t| format!("{base}/booking/cancel/{t}"));
    let telegram_link = super::telegram::pay_page_link(state, booking_id, pay_token).await;
    let booking_id = booking_id.to_string();
    tracing::info!(target: TARGET, booking_id = %booking_id, "payment link emailed to the guest");
    crate::web::spawn_emails(async move {
        let result = crate::email::send_guest_ton_pay_link(
            &smtp,
            &mail,
            &pay_url,
            cancel_url.as_deref(),
            telegram_link.as_deref(),
        )
        .await;
        crate::web::log_email_result(result, "guest_ton_pay_link", &booking_id);
    });
    true
}

/// ilia fork: the guest's "pay again by …" email after a renewal, to the
/// address the booking was made with, with the new deadline, the payment
/// page, the Telegram link and the cancel link. Only to a proven address:
/// the guest asked for the link by email or paid any of it (anyone can type
/// someone else's address into the form; such a guest hears of the renewal
/// in their linked Telegram chat only). Nothing without a base URL (the
/// links need one) or for an invoice no longer open.
pub(crate) async fn email_renewal(
    pool: &SqlitePool,
    secret_key: &[u8; 32],
    smtp: &crate::email::SmtpConfig,
    invoice_id: &str,
) -> anyhow::Result<()> {
    let Some(base) = crate::settings::base_url() else {
        tracing::warn!(target: TARGET, invoice = invoice_id, "renewal not emailed: CALRS_BASE_URL is not set");
        return Ok(());
    };
    let row: Option<(String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT i.booking_id, i.pay_token, i.expires_at, b.cancel_token, b.guest_timezone
         FROM ton_invoices i JOIN bookings b ON b.id = i.booking_id
         WHERE i.id = ? AND i.status = 'open' AND b.status = 'pending'
           AND (i.link_emailed_at IS NOT NULL OR i.paid_nano > 0)",
    )
    .bind(invoice_id)
    .fetch_optional(pool)
    .await?;
    let Some((booking_id, pay_token, expires_at, cancel_token, tz)) = row else {
        return Ok(());
    };
    let Some((mut mail, _)) = guest_mail(pool, &booking_id).await else {
        return Ok(());
    };
    if let Some(expires) = parse_stamp(&expires_at) {
        let (date, time) = date_and_time(&mail.lang, expires, tz.as_deref().unwrap_or("UTC"));
        mail.pay_by = format!("{date}, {time}");
    }
    let base = base.trim_end_matches('/');
    let pay_url = format!("{base}/booking/pay/{pay_token}");
    let cancel_url = cancel_token.map(|t| format!("{base}/booking/cancel/{t}"));
    let telegram_link = super::telegram::pay_link(pool, secret_key, &booking_id, &pay_token).await;
    tracing::info!(target: TARGET, booking_id = %booking_id, "payment renewal emailed to the guest");
    crate::email::send_guest_ton_renewed(
        smtp,
        &mail,
        &pay_url,
        cancel_url.as_deref(),
        telegram_link.as_deref(),
    )
    .await
}

/// `POST …/email`: send the guest this page's link, once per invoice, to
/// the address they booked with.
pub(in crate::web) async fn pay_email(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(token): Path<String>,
    Form(form): Form<CsrfForm>,
) -> Response {
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    let Some(row) = load(&state.pool, &token).await else {
        return private(super::flow::not_found(&state, page_lang(None, &headers)));
    };
    let back = |ok: bool| {
        Redirect::to(&format!("/booking/pay/{token}?sent={}", u8::from(ok))).into_response()
    };
    let ip = client_ip_for_rate_limit(&headers);
    if state.ton.email_limiter.check_limited(&ip).await {
        return back(false);
    }
    let now = Utc::now();
    if !current_state(&state, &row, now).await.can_pay() {
        return back(false);
    }
    if crate::settings::base_url().is_none()
        || !matches!(
            crate::email::load_smtp_config(&state.pool, &state.secret_key).await,
            Ok(Some(_))
        )
    {
        return back(false);
    }
    let claimed = sqlx::query(
        "UPDATE ton_invoices SET link_emailed_at = ? WHERE id = ? AND link_emailed_at IS NULL AND status = 'open'",
    )
    .bind(stamp(now))
    .bind(&row.id)
    .execute(&state.pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or(false);
    if !claimed {
        return back(false);
    }
    let sent = email_links(
        &state,
        &row.booking_id,
        &token,
        row.cancel_token.as_deref(),
        &row.expires_at,
    )
    .await;
    back(sent)
}

#[derive(Deserialize)]
pub(crate) struct ResendForm {
    #[serde(default)]
    _csrf: Option<String>,
    #[serde(default)]
    email: String,
}

/// `POST /booking/pay/resend`: from the "you already have an unpaid
/// booking" page. Emails the open booking of that address its payment and
/// cancel links, to that address only; the answer is the same whether or
/// not there is one (it says nothing about who booked). At most once per
/// address every 10 minutes, five times an hour per client address.
pub(in crate::web) async fn resend_links(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(form): Form<ResendForm>,
) -> Response {
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    let lang = crate::i18n::detect_from_headers(&headers);
    let done = || {
        private(crate::web::render_booking_action_error_keys_in(
            &state,
            lang,
            "ton-resend-title",
            "ton-resend-done",
        ))
    };
    let email = super::invoice::normalize_email(&form.email);
    if email.is_empty() || email.len() > 320 {
        return done();
    }
    let ip = client_ip_for_rate_limit(&headers);
    if state.ton.resend_limiter.check_limited(&ip).await
        || state.ton.resend_email_limiter.check_limited(&email).await
    {
        tracing::warn!(target: TARGET, "payment link resend rate-limited");
        return done();
    }
    let open: Option<(String, String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT i.id, i.booking_id, i.pay_token, b.cancel_token, i.expires_at
         FROM ton_invoices i JOIN bookings b ON b.id = i.booking_id
         WHERE i.guest_email_norm = ? AND i.status = 'open' AND b.status = 'pending'
         ORDER BY i.created_at DESC LIMIT 1",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or_else(|e| {
        tracing::error!(target: TARGET, error = %e, "cannot look up an open invoice to resend");
        None
    });
    let Some((invoice_id, booking_id, pay_token, cancel_token, expires_at)) = open else {
        return done();
    };
    // The guest has the link now: a release later is worth an email too.
    let _ = sqlx::query(
        "UPDATE ton_invoices SET link_emailed_at = COALESCE(link_emailed_at, ?) WHERE id = ?",
    )
    .bind(stamp(Utc::now()))
    .bind(&invoice_id)
    .execute(&state.pool)
    .await;
    email_links(
        &state,
        &booking_id,
        &pay_token,
        cancel_token.as_deref(),
        &expires_at,
    )
    .await;
    done()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(status: &str, booking: Option<&str>) -> PayRow {
        PayRow {
            id: "i".into(),
            booking_id: "b".into(),
            network: "mainnet".into(),
            address_raw: "0:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8"
                .into(),
            event_title: "Call".into(),
            start_at_utc: "2030-01-01T10:00:00Z".into(),
            currency: "USD".into(),
            price_usd_cents: 4000,
            rate_usd_micro: 2_000_000,
            rate_sources: "coingecko=2.000000;binance=2.000000".into(),
            rate_at: "2029-12-31T09:00:00Z".into(),
            amount_nano: 20_000_000_000,
            paid_nano: 0,
            status: status.into(),
            problem: None,
            expires_at: "2029-12-31T09:30:00Z".into(),
            settle_by: "2029-12-31T09:45:00Z".into(),
            reference: "B7KQ3M9XD2F".into(),
            link_emailed_at: None,
            event_type_id: "et".into(),
            booking_status: booking.map(str::to_string),
            booking_start: None,
            booking_tz: None,
            booking_lang: None,
            time_version: Some(1),
            cancel_token: None,
            booking_uid: None,
            renewals: 0,
            renewals_max: 0,
        }
    }

    #[test]
    fn states_follow_the_invoice_and_its_booking() {
        let before = parse_stamp("2029-12-31T09:10:00Z").unwrap();
        let after = parse_stamp("2029-12-31T09:40:00Z").unwrap();
        let open = row("open", Some("pending"));
        assert_eq!(state_of(&open, false, false, before), PayState::Waiting);
        assert_eq!(state_of(&open, true, false, before), PayState::Seen);
        assert_eq!(state_of(&open, false, false, after), PayState::Checking);
        let mut partial = open.clone();
        partial.paid_nano = 1;
        assert_eq!(state_of(&partial, true, false, before), PayState::Partial);
        assert_eq!(
            state_of(&row("open", Some("cancelled")), false, false, before),
            PayState::Closed
        );
        assert_eq!(
            state_of(&row("paid", Some("pending")), false, false, after),
            PayState::Paid
        );
        let mut busy = row("paid", Some("pending"));
        busy.problem = Some("host_busy".into());
        assert_eq!(state_of(&busy, false, false, after), PayState::Busy);
        assert_eq!(
            state_of(&row("confirmed", Some("confirmed")), false, false, after),
            PayState::Confirmed
        );
        assert_eq!(
            state_of(&row("confirmed", Some("cancelled")), false, false, after),
            PayState::Closed
        );
        assert_eq!(
            state_of(&row("waived", Some("pending")), false, false, after),
            PayState::Waived
        );
        assert_eq!(
            state_of(&row("expired", Some("declined")), false, false, after),
            PayState::Expired
        );
        assert_eq!(
            state_of(&row("expired", Some("declined")), false, true, after),
            PayState::Late
        );
        let mut revoked = row("closed", Some("pending"));
        revoked.problem = Some("address_revoked".into());
        assert_eq!(state_of(&revoked, false, false, before), PayState::Revoked);
        for s in [PayState::Waiting, PayState::Seen, PayState::Partial] {
            assert!(s.can_pay() && s.live());
        }
        for s in [PayState::Checking, PayState::Expired, PayState::Revoked] {
            assert!(!s.can_pay());
        }
        // The owner settles a busy booking: the open page follows.
        assert!(PayState::Busy.live() && !PayState::Busy.can_pay());
        // Money on a booking that ended unconfirmed goes back, and says so.
        let mut ended = row("paid", Some("declined"));
        ended.paid_nano = 20_000_000_000;
        assert_eq!(state_of(&ended, false, false, after), PayState::Refund);
        let mut cancelled_part = row("closed", Some("cancelled"));
        cancelled_part.paid_nano = 5;
        assert_eq!(
            state_of(&cancelled_part, false, false, after),
            PayState::Refund
        );
        let mut taken = row("closed", Some("pending"));
        taken.problem = Some("host_busy_unpaid".into());
        assert_eq!(state_of(&taken, false, false, before), PayState::Taken);
        for s in [
            PayState::Busy,
            PayState::Expired,
            PayState::Late,
            PayState::Taken,
            PayState::Refund,
        ] {
            assert!(s.shows_contact(), "{s:?}");
        }
        assert!(!PayState::Waiting.shows_contact());
    }

    #[test]
    fn the_rest_is_shown_at_its_own_worth() {
        let mut r = row("open", Some("pending"));
        assert_eq!(remaining_usd_cents(&r), 4000);
        r.paid_nano = 5_000_000_000;
        // 15 TON at $2.00.
        assert_eq!(remaining_usd_cents(&r), 3000);
        r.paid_nano = 19_999_999_999;
        assert_eq!(remaining_usd_cents(&r), 0);
    }

    #[test]
    fn the_guest_pays_to_the_invoice_address_in_its_network_form() {
        assert_eq!(
            guest_address(&row("open", None)).as_deref(),
            Some("UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI")
        );
        let mut t = row("open", None);
        t.network = "testnet".into();
        assert!(guest_address(&t).unwrap().starts_with("0Q"));
        assert_eq!(source_names("coingecko=1;binance=2"), "CoinGecko + Binance");
        assert_eq!(source_names("binance=2"), "Binance");
    }

    #[test]
    fn each_wallet_gets_the_link_it_honours() {
        let address = guest_address(&row("open", None)).unwrap();
        let links = PayLinks::new(
            Network::Mainnet,
            &address,
            20_000_000_000,
            "B-7KQ3M9XD2F",
            1_893_403_800,
        );
        assert_eq!(
            links.keeper,
            "https://app.tonkeeper.com/transfer/UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI?amount=20000000000&text=B-7KQ3M9XD2F&exp=1893403800&asset_id=ton/mainnet/coin"
        );
        assert_eq!(
            links.ton,
            "ton://transfer/UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI?amount=20000000000&text=B-7KQ3M9XD2F&exp=1893403800"
        );
        // Each QR code draws its own link.
        assert_eq!(links.keeper_qr, qr::svg(&links.keeper).unwrap());
        assert_eq!(links.ton_qr, qr::svg(&links.ton).unwrap());
        assert_ne!(links.keeper_qr, links.ton_qr);
    }
}
