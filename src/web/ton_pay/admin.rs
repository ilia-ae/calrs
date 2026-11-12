//! The host side of payments in the web dashboard: the price on the event
//! type form, the "paid" badges on the bookings page, the explicit "Confirm
//! without payment" (the only way an unpaid priced booking is confirmed
//! for free), and the price line on the public slot and form pages.

use axum::extract::{Form, Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use chrono::Utc;
use minijinja::context;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{invoice, parse_stamp, store, TARGET};
use crate::ton::amount::{self, format_ton, format_usd_cents, from_i64};
use crate::web::{
    confirm_booking, internal_error_body, internal_error_html, sidebar_context, verify_csrf_token,
    AppState, CsrfForm,
};

// --- the price on the event type form ----------------------------------------

/// Cheapest and dearest price accepted, in cents (as the table's CHECK).
const MIN_CENTS: u64 = 100;
const MAX_CENTS: u64 = 1_000_000;

fn checked(v: Option<&str>) -> bool {
    matches!(v, Some("on") | Some("true") | Some("1"))
}

fn blank(v: Option<&str>) -> bool {
    v.is_none_or(|s| s.trim().is_empty())
}

/// The price section of the event type form as posted. `amount = None`
/// means the form did not carry the section.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PriceForm<'a> {
    pub(crate) enabled: Option<&'a str>,
    pub(crate) amount: Option<&'a str>,
    /// `TON` (the default when absent) or `USD`.
    pub(crate) currency: Option<&'a str>,
    pub(crate) window_min: Option<&'a str>,
    /// The guest Telegram mode chosen in the section (only shown while
    /// Telegram is on).
    pub(crate) guest_telegram: Option<&'a str>,
}

/// `TON` unless the form says `USD`; `None` for anything else.
fn currency_of(v: Option<&str>) -> Option<&'static str> {
    match v.map(str::trim) {
        None | Some("") | Some("TON") => Some("TON"),
        Some("USD") => Some("USD"),
        _ => None,
    }
}

/// The posted amount as a price in the posted currency. A blank TON price
/// is the default, 100 TON.
fn parse_price(currency: &str, amount: &str) -> Option<store::Price> {
    match currency {
        "TON" if amount.trim().is_empty() => {
            amount::parse_ton_price(amount::DEFAULT_PRICE_TON).map(store::Price::Ton)
        }
        "TON" => amount::parse_ton_price(amount).map(store::Price::Ton),
        _ => amount::parse_usd_cents(amount)
            .filter(|c| (MIN_CENTS..=MAX_CENTS).contains(c))
            .map(store::Price::Usd),
    }
}

/// Server-side check of the form's price section. `Err` is the Fluent key
/// of the first problem. Team types cannot be priced (v1).
pub(crate) fn validate_price_form(
    form: &PriceForm<'_>,
    team_id: Option<&str>,
) -> Result<(), &'static str> {
    let on = checked(form.enabled);
    if on && !blank(team_id) {
        return Err("ton-form-error-team");
    }
    let currency = currency_of(form.currency).ok_or("ton-form-error-currency")?;
    if (on || !blank(form.amount)) && parse_price(currency, form.amount.unwrap_or("")).is_none() {
        return Err(if currency == "TON" {
            "ton-form-error-price-ton"
        } else {
            "ton-form-error-price"
        });
    }
    if !blank(form.window_min) {
        match form.window_min.and_then(|w| w.trim().parse::<i64>().ok()) {
            Some(w) if (10..=240).contains(&w) => {}
            _ => return Err("ton-form-error-window"),
        }
    }
    if !blank(form.guest_telegram)
        && !crate::telegram::username::mode::ALL.contains(&form.guest_telegram.unwrap_or("").trim())
    {
        return Err("ton-form-error-telegram");
    }
    Ok(())
}

/// What a price section asks to store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PriceWrite {
    /// The price, its payment window (`None`: the server's default) and
    /// whether it is switched on. Unchecked with a price keeps it, off.
    Save {
        price: store::Price,
        window_min: Option<i64>,
        enabled: bool,
    },
    /// No price: unchecked and blank.
    Remove,
}

impl PriceWrite {
    fn price(&self) -> Option<store::Price> {
        match self {
            Self::Save { price, .. } => Some(*price),
            Self::Remove => None,
        }
    }
}

/// What a price section that passed `validate_price_form` stores; `None`
/// when the form did not carry the section (`amount = None`), or for a
/// value the validation refuses.
pub(crate) fn price_write(form: &PriceForm<'_>) -> Option<PriceWrite> {
    let raw = form.amount?;
    let on = checked(form.enabled);
    // Unchecked and blank means no price, whatever the default.
    let price = currency_of(form.currency)
        .filter(|_| on || !raw.trim().is_empty())
        .and_then(|c| parse_price(c, raw));
    let window_min = form
        .window_min
        .and_then(|w| w.trim().parse::<i64>().ok())
        .filter(|w| (10..=240).contains(w));
    match price {
        Some(price) => Some(PriceWrite::Save {
            price,
            window_min,
            enabled: on,
        }),
        None if !on && raw.trim().is_empty() => Some(PriceWrite::Remove),
        None => None,
    }
}

/// Store `write` for event type `event_type_id` on `conn`, inside the
/// caller's transaction (`store::save_price`: a price switched on makes a
/// type with no chosen Telegram mode ask for it, `required`). True when a
/// price row was created. The caller checks that the type is personal.
pub(crate) async fn write_price(
    conn: &mut sqlx::SqliteConnection,
    event_type_id: &str,
    write: PriceWrite,
    now: chrono::DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    match write {
        PriceWrite::Save {
            price,
            window_min,
            enabled,
        } => store::save_price(conn, event_type_id, price, window_min, enabled, now).await,
        PriceWrite::Remove => sqlx::query("DELETE FROM ton_prices WHERE event_type_id = ?")
            .bind(event_type_id)
            .execute(&mut *conn)
            .await
            .map(|_| false),
    }
}

/// Store the form's price section (already validated). `amount = None`
/// means the form did not carry the section: nothing changes. Unchecked
/// with a price keeps it, switched off; unchecked and blank removes it
/// (`price_write`). Only a personal type is ever priced. A new price makes
/// the type ask for the guest's Telegram (`required`), and the section's own
/// Telegram choice, when shown, is saved with it. Open invoices are not
/// touched: they run to their end whatever the price does now. A saved
/// price wakes the watcher, so a type priced after a quiet day is bookable
/// at once.
pub(crate) async fn save_price_form(state: &AppState, event_type_id: &str, form: &PriceForm<'_>) {
    let Some(write) = price_write(form) else {
        return;
    };
    let pool = &state.pool;
    let personal: bool =
        sqlx::query_scalar::<_, i64>("SELECT team_id IS NULL FROM event_types WHERE id = ?")
            .bind(event_type_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .is_some_and(|v| v != 0);
    let on = checked(form.enabled);
    let price = write.price();
    if price.is_some() && !personal {
        tracing::warn!(target: TARGET, event_type_id, "a team event type cannot be priced");
        return;
    }
    let result = async {
        let mut tx = pool.begin().await?;
        let created = write_price(&mut tx, event_type_id, write, Utc::now()).await?;
        tx.commit().await?;
        Ok::<bool, sqlx::Error>(created)
    }
    .await;
    match result {
        Ok(created) => {
            tracing::info!(
                target: TARGET,
                event_type_id,
                currency = price.map(|p| p.currency()).unwrap_or(""),
                created,
                enabled = on && price.is_some(),
                "event type price saved"
            );
            // The section's Telegram select: with the price on; with it
            // off only when the owner already chose a mode for this type
            // (lowering it stays possible), so an amount typed into a free
            // type never makes its guests give a Telegram username.
            let mode_chosen = || async {
                sqlx::query_scalar::<_, i64>(
                    "SELECT EXISTS (SELECT 1 FROM telegram_event_types WHERE event_type_id = ?)",
                )
                .bind(event_type_id)
                .fetch_one(pool)
                .await
                .map(|v| v != 0)
                .unwrap_or(false)
            };
            if price.is_some() && (on || mode_chosen().await) {
                if let Some(mode) = form
                    .guest_telegram
                    .map(str::trim)
                    .filter(|m| crate::telegram::username::mode::ALL.contains(m))
                {
                    if let Err(e) =
                        crate::telegram::store::save_event_type_mode(pool, event_type_id, mode)
                            .await
                    {
                        tracing::error!(target: TARGET, error = %e, event_type_id, "cannot save the guest Telegram mode");
                    }
                }
            }
            super::wake_watcher(state);
        }
        Err(e) => {
            tracing::error!(target: TARGET, error = %e, event_type_id, "cannot save the event type price")
        }
    }
}

/// A duplicated personal event type keeps its price (inside the
/// duplication's transaction). A team type has none to copy.
pub(crate) async fn copy_price(
    conn: &mut sqlx::SqliteConnection,
    from: &str,
    to: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO ton_prices (event_type_id, currency, price_nano, price_usd_cents, window_min,
             enabled, updated_at)
         SELECT ?, p.currency, p.price_nano, p.price_usd_cents, p.window_min, p.enabled, ?
         FROM ton_prices p JOIN event_types et ON et.id = ?
         WHERE p.event_type_id = ? AND et.team_id IS NULL",
    )
    .bind(to)
    .bind(super::stamp(Utc::now()))
    .bind(to)
    .bind(from)
    .execute(&mut *conn)
    .await
    .map(|_| ())
}

/// What the price section shows.
struct PriceSection {
    enabled: bool,
    amount: String,
    currency: &'static str,
    window: String,
    guest_telegram: &'static str,
}

fn price_ctx(state: &AppState, section: PriceSection) -> minijinja::Value {
    let (payments_on, default_window, rates_on) = match state.ton.get() {
        Some(rt) => (true, rt.cfg.window_min, rt.cfg.rate_policy.is_some()),
        None => (false, 30, false),
    };
    context! {
        enabled => section.enabled,
        amount => section.amount,
        currency => section.currency,
        window => section.window,
        default_window => default_window,
        payments_on => payments_on,
        usd_unbookable => payments_on && !rates_on && section.currency == "USD",
        placeholder => if section.currency == "USD" { "40.00" } else { amount::DEFAULT_PRICE_TON },
        telegram_on => crate::telegram::enabled(),
        guest_telegram => section.guest_telegram,
    }
}

/// The price section of a personal event type's form; `None` (no section)
/// for a team type. `event_type_id = None` for a new one: switched off, in
/// TON, the amount blank (100 TON when switched on so).
pub(crate) async fn price_form_ctx(
    state: &AppState,
    event_type_id: Option<&str>,
) -> Option<minijinja::Value> {
    let fresh = PriceSection {
        enabled: false,
        amount: String::new(),
        currency: "TON",
        window: String::new(),
        guest_telegram: crate::telegram::username::mode::REQUIRED,
    };
    let Some(et_id) = event_type_id else {
        return Some(price_ctx(state, fresh));
    };
    #[allow(clippy::type_complexity)]
    let row: Option<(
        Option<String>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        i64,
    )> = sqlx::query_as(
        "SELECT p.currency, p.price_nano, p.price_usd_cents, p.window_min, p.enabled,
                et.team_id IS NULL
         FROM event_types et LEFT JOIN ton_prices p ON p.event_type_id = et.id
         WHERE et.id = ?",
    )
    .bind(et_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    let (currency, nano, cents, window, enabled, personal) = row?;
    if personal == 0 {
        return None;
    }
    let price = currency
        .as_deref()
        .and_then(|c| store::Price::from_columns(c, nano, cents));
    let Some(price) = price else {
        // No price yet: the defaults, with the type's own Telegram mode
        // if one was chosen.
        let stored = crate::telegram::store::event_type_mode(&state.pool, et_id).await;
        let chosen: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM telegram_event_types WHERE event_type_id = ?")
                .bind(et_id)
                .fetch_optional(&state.pool)
                .await
                .ok()
                .flatten();
        return Some(price_ctx(
            state,
            PriceSection {
                guest_telegram: if chosen.is_some() {
                    stored
                } else {
                    fresh.guest_telegram
                },
                ..fresh
            },
        ));
    };
    Some(price_ctx(
        state,
        PriceSection {
            enabled: enabled == Some(1),
            amount: price.amount(),
            currency: price.currency(),
            window: window.map(|w| w.to_string()).unwrap_or_default(),
            guest_telegram: crate::telegram::store::event_type_mode(&state.pool, et_id).await,
        },
    ))
}

/// The section echoed back after a validation error, as posted.
pub(crate) fn price_form_from_post(
    state: &AppState,
    form: &PriceForm<'_>,
    personal: bool,
) -> Option<minijinja::Value> {
    personal.then(|| {
        price_ctx(
            state,
            PriceSection {
                enabled: checked(form.enabled),
                amount: form.amount.unwrap_or("").trim().to_string(),
                currency: currency_of(form.currency).unwrap_or("TON"),
                window: form.window_min.unwrap_or("").trim().to_string(),
                guest_telegram: crate::telegram::username::mode::parse(
                    form.guest_telegram
                        .unwrap_or(crate::telegram::username::mode::REQUIRED),
                ),
            },
        )
    })
}

/// The price line of a public slot or form page. A TON price is shown as
/// it is (the exact amount); a USD price with, when a rate is cached, the
/// approximate amount in TON (the exact one is fixed when the guest
/// books). Never fetches on a page view.
pub(crate) async fn price_badge(state: &AppState, event_type_id: &str) -> Option<minijinja::Value> {
    let row = store::price_for(&state.pool, event_type_id).await.ok()??;
    if row.is_team != 0 {
        return None;
    }
    // A guest is looking at a priced slot or form: an idle watcher steps
    // now, so the booking finds its heartbeat fresh (ilia fork, Optimize).
    if let Some(rt) = state.ton.get() {
        rt.wake_if_idle(Utc::now());
    }
    match row.price()? {
        store::Price::Ton(nano) => Some(context! {
            ton_exact => format_ton(nano),
        }),
        store::Price::Usd(cents) => {
            let ton = state
                .ton
                .get()
                .and_then(|rt| rt.cached_rate())
                .and_then(|rate| amount::cents_to_nano(cents, rate.usd_micro))
                .map(format_ton);
            Some(context! {
                usd => format_usd_cents(u128::from(cents)),
                ton => ton,
            })
        }
    }
}

// --- the bookings dashboard ---------------------------------------------------

/// Badges for the user's bookings that have an invoice, by booking id:
/// `{label, kind, unpaid, waive_url}`. `unpaid` rows lose Confirm and
/// Reschedule (both would be refused) and offer "Confirm without payment".
pub(crate) async fn dashboard_badges(
    pool: &SqlitePool,
    user_id: &str,
    timezone: &str,
    lang: &str,
) -> minijinja::Value {
    #[allow(clippy::type_complexity)]
    let rows: Vec<(
        String,
        String,
        Option<String>,
        i64,
        i64,
        i64,
        String,
        String,
        i64,
        i64,
    )> = sqlx::query_as(
        "SELECT i.booking_id, i.status, i.problem, i.amount_nano, i.paid_nano, i.excess_nano,
                    i.expires_at, b.status, i.renewals, i.renewals_max
             FROM ton_invoices i
             JOIN bookings b ON b.id = i.booking_id
             JOIN event_types et ON et.id = b.event_type_id
             JOIN accounts a ON a.id = et.account_id
             WHERE a.user_id = ? AND b.status IN ('pending', 'confirmed')",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let tz = timezone.parse::<chrono_tz::Tz>().unwrap_or(chrono_tz::UTC);
    let mut out = std::collections::BTreeMap::new();
    for (
        booking_id,
        status,
        problem,
        amount_nano,
        paid_nano,
        excess_nano,
        expires,
        _booking,
        renewals,
        renewals_max,
    ) in rows
    {
        let amount = format_ton(from_i64(amount_nano));
        let until = parse_stamp(&expires)
            .map(|t| t.with_timezone(&tz).format("%H:%M").to_string())
            .unwrap_or_default();
        let tr = |key: &str, args: &[(&str, &str)]| {
            let mut fa = fluent_bundle::FluentArgs::new();
            for (k, v) in args {
                fa.set(*k, fluent_bundle::FluentValue::from(*v));
            }
            crate::i18n::translate(lang, key, Some(&fa))
        };
        let (label, kind) = match (status.as_str(), problem.as_deref()) {
            (_, Some("refund_due")) => {
                (tr("ton-dash-badge-refund", &[("amount", &amount)]), "error")
            }
            ("open", _) if paid_nano > 0 => (
                tr(
                    "ton-dash-badge-partial",
                    &[
                        ("paid", &format_ton(from_i64(paid_nano))),
                        ("amount", &amount),
                        ("time", &until),
                    ],
                ),
                "warning",
            ),
            ("open", _) => (
                tr(
                    "ton-dash-badge-awaiting",
                    &[("amount", &amount), ("time", &until)],
                ),
                "warning",
            ),
            ("paid", Some("host_busy")) => (tr("ton-dash-badge-busy", &[]), "error"),
            ("paid", Some("confirm_failed")) => (tr("ton-dash-badge-confirm-failed", &[]), "error"),
            ("paid" | "confirmed", _) if excess_nano > 0 => (
                tr(
                    "ton-dash-badge-overpaid",
                    &[("excess", &format_ton(from_i64(excess_nano)))],
                ),
                "warning",
            ),
            ("paid" | "confirmed", _) => {
                (tr("ton-dash-badge-paid", &[("amount", &amount)]), "success")
            }
            ("waived", _) => (tr("ton-dash-badge-waived", &[]), "success"),
            _ => (tr("ton-dash-badge-unpaid", &[]), "error"),
        };
        // ilia fork: a renewed window says so ("renewal 1 of 2").
        let label = if status == "open" && renewals > 0 {
            let mut fa = fluent_bundle::FluentArgs::new();
            fa.set("n", fluent_bundle::FluentValue::from(renewals));
            fa.set("max", fluent_bundle::FluentValue::from(renewals_max));
            format!(
                "{label} · {}",
                crate::i18n::translate(lang, "ton-dash-badge-renewed", Some(&fa))
            )
        } else {
            label
        };
        out.insert(
            booking_id.clone(),
            context! {
                label => label,
                kind => kind,
                unpaid => !matches!(status.as_str(), "paid" | "confirmed" | "waived"),
                waivable => status == "open",
                waive_url => format!("/dashboard/ton/waive/{booking_id}"),
            },
        );
    }
    minijinja::Value::from_iter(out)
}

/// The booking, if it is the user's and pending, with its open invoice:
/// (event title, guest name, amount nano, paid nano, expires_at, renewals,
/// renewals_max).
#[allow(clippy::type_complexity)]
async fn waivable(
    pool: &SqlitePool,
    booking_id: &str,
    user_id: &str,
) -> Option<(String, String, i64, i64, String, i64, i64)> {
    sqlx::query_as(
        "SELECT et.title, b.guest_name, i.amount_nano, i.paid_nano, i.expires_at, i.renewals,
                i.renewals_max
         FROM bookings b
         JOIN event_types et ON et.id = b.event_type_id
         JOIN accounts a ON a.id = et.account_id
         JOIN ton_invoices i ON i.booking_id = b.id
         WHERE b.id = ? AND a.user_id = ? AND b.status = 'pending' AND i.status = 'open'",
    )
    .bind(booking_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// `GET /dashboard/ton/waive/{booking_id}`: "this booking is NOT paid;
/// confirm it free of charge?".
pub(crate) async fn waive_form(
    State(state): State<Arc<AppState>>,
    auth_user: crate::auth::AuthUser,
    Path(booking_id): Path<String>,
) -> Response {
    let Some((title, guest, amount_nano, paid_nano, expires, renewals, renewals_max)) =
        waivable(&state.pool, &booking_id, &auth_user.user.id).await
    else {
        return Redirect::to("/dashboard/bookings?error=waive-failed").into_response();
    };
    let tz = auth_user
        .user
        .timezone
        .parse::<chrono_tz::Tz>()
        .unwrap_or(chrono_tz::UTC);
    let until = parse_stamp(&expires)
        .map(|t| t.with_timezone(&tz).format("%H:%M").to_string())
        .unwrap_or_default();
    let tmpl = match state.templates.get_template("ton_waive_confirm.html") {
        Ok(t) => t,
        Err(e) => return internal_error_html("template render", &e).into_response(),
    };
    Html(
        tmpl.render(context! {
            sidebar => sidebar_context(&auth_user, "bookings"),
            lang => auth_user.lang,
            booking_id => booking_id,
            event_title => title,
            guest_name => guest,
            amount => format_ton(from_i64(amount_nano)),
            paid => (paid_nano > 0).then(|| format_ton(from_i64(paid_nano))),
            rest => format_ton(from_i64(amount_nano).saturating_sub(from_i64(paid_nano))),
            until => until,
            renewals => renewals,
            renewals_max => renewals_max,
        })
        .unwrap_or_else(|e| internal_error_body("template render", &e)),
    )
    .into_response()
}

/// `POST /dashboard/ton/waive/{booking_id}`: the invoice becomes `waived`
/// (only from `open`), then the ordinary dashboard confirmation runs. If
/// that is refused (the host is busy), the booking stays pending and keeps
/// its time until its deadline; the waived invoice lets a later Confirm
/// through.
pub(in crate::web) async fn waive(
    State(state): State<Arc<AppState>>,
    auth_user: crate::auth::AuthUser,
    headers: HeaderMap,
    Path(booking_id): Path<String>,
    Form(form): Form<CsrfForm>,
) -> Response {
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    if waivable(&state.pool, &booking_id, &auth_user.user.id)
        .await
        .is_none()
    {
        return Redirect::to("/dashboard/bookings?error=waive-failed").into_response();
    }
    let by = format!("dashboard:{}", auth_user.user.id);
    match invoice::waive(&state.pool, &booking_id, &by, Utc::now()).await {
        Ok(true) => {}
        Ok(false) => {
            return Redirect::to("/dashboard/bookings?error=waive-failed").into_response();
        }
        Err(e) => {
            tracing::error!(target: TARGET, error = %e, booking_id, "cannot waive the payment");
            return Redirect::to("/dashboard/bookings?error=waive-failed").into_response();
        }
    }
    confirm_booking(
        State(state),
        auth_user,
        headers,
        Path(booking_id),
        Form(form),
    )
    .await
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form<'a>(
        enabled: Option<&'a str>,
        amount: Option<&'a str>,
        currency: Option<&'a str>,
        window_min: Option<&'a str>,
    ) -> PriceForm<'a> {
        PriceForm {
            enabled,
            amount,
            currency,
            window_min,
            guest_telegram: None,
        }
    }

    #[test]
    fn the_price_section_is_validated_on_the_server() {
        let usd = Some("USD");
        let v = |f: PriceForm<'_>, team: Option<&str>| validate_price_form(&f, team);
        assert_eq!(v(form(None, None, None, None), None), Ok(()));
        assert_eq!(v(form(None, Some(""), None, Some("")), None), Ok(()));
        assert_eq!(v(form(Some("on"), Some("40"), usd, None), None), Ok(()));
        assert_eq!(
            v(form(Some("on"), Some("40.50"), usd, Some("15")), Some("")),
            Ok(())
        );
        for bad in ["", "0.99", "10000.01", "abc", "40.505", "-5", "1e3"] {
            assert_eq!(
                v(form(Some("on"), Some(bad), usd, None), None),
                Err("ton-form-error-price"),
                "{bad:?}"
            );
        }
        // TON, the default currency: 1 to 100000 TON in 0.01 steps.
        for ok in ["100", "1", "12.5", "100000", "99.99"] {
            assert_eq!(
                v(form(Some("on"), Some(ok), None, None), None),
                Ok(()),
                "{ok}"
            );
            assert_eq!(
                v(form(Some("on"), Some(ok), Some("TON"), None), None),
                Ok(())
            );
        }
        // Blank and switched on: the default, 100 TON.
        assert_eq!(v(form(Some("on"), Some(""), None, None), None), Ok(()));
        for bad in ["0.5", "100000.01", "1.001", "x"] {
            assert_eq!(
                v(form(Some("on"), Some(bad), None, None), None),
                Err("ton-form-error-price-ton"),
                "{bad:?}"
            );
        }
        assert_eq!(
            v(form(Some("on"), Some("100"), Some("EUR"), None), None),
            Err("ton-form-error-currency")
        );
        // A price kept switched off must still be a price.
        assert_eq!(
            v(form(None, Some("x"), usd, None), None),
            Err("ton-form-error-price")
        );
        for bad in ["9", "241", "x"] {
            assert_eq!(
                v(form(Some("on"), Some("40"), usd, Some(bad)), None),
                Err("ton-form-error-window")
            );
        }
        assert_eq!(
            v(form(Some("on"), Some("40"), usd, None), Some("team-1")),
            Err("ton-form-error-team")
        );
        // Unpriced team types are fine.
        assert_eq!(v(form(None, None, None, None), Some("team-1")), Ok(()));
        let mut f = form(Some("on"), Some("100"), None, None);
        f.guest_telegram = Some("optional");
        assert_eq!(validate_price_form(&f, None), Ok(()));
        f.guest_telegram = Some("always");
        assert_eq!(
            validate_price_form(&f, None),
            Err("ton-form-error-telegram")
        );
    }
}
