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

/// Server-side check of the form's price section. `Err` is the Fluent key
/// of the first problem. Team types cannot be priced (v1).
pub(crate) fn validate_price_form(
    enabled: Option<&str>,
    usd: Option<&str>,
    window_min: Option<&str>,
    team_id: Option<&str>,
) -> Result<(), &'static str> {
    let on = checked(enabled);
    if on && !blank(team_id) {
        return Err("ton-form-error-team");
    }
    if on || !blank(usd) {
        match usd.and_then(amount::parse_usd_cents) {
            Some(c) if (MIN_CENTS..=MAX_CENTS).contains(&c) => {}
            _ => return Err("ton-form-error-price"),
        }
    }
    if !blank(window_min) {
        match window_min.and_then(|w| w.trim().parse::<i64>().ok()) {
            Some(w) if (10..=240).contains(&w) => {}
            _ => return Err("ton-form-error-window"),
        }
    }
    Ok(())
}

/// Store the form's price section (already validated). `usd = None` means
/// the form did not carry the section: nothing changes. Unchecked with a
/// price keeps it, switched off; unchecked and blank removes it. Only a
/// personal type is ever priced. Open invoices are not touched: they run
/// to their end whatever the price does now. A saved price wakes the
/// watcher, so a type priced after a quiet day is bookable at once.
pub(crate) async fn save_price_form(
    state: &AppState,
    event_type_id: &str,
    enabled: Option<&str>,
    usd: Option<&str>,
    window_min: Option<&str>,
) {
    let Some(usd) = usd else {
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
    let on = checked(enabled);
    let cents = amount::parse_usd_cents(usd).filter(|c| (MIN_CENTS..=MAX_CENTS).contains(c));
    let window = window_min
        .and_then(|w| w.trim().parse::<i64>().ok())
        .filter(|w| (10..=240).contains(w));
    let result = match cents {
        Some(cents) if personal => {
            sqlx::query(
                "INSERT INTO ton_prices (event_type_id, price_usd_cents, window_min, enabled, updated_at)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT (event_type_id) DO UPDATE SET price_usd_cents = excluded.price_usd_cents,
                     window_min = excluded.window_min, enabled = excluded.enabled,
                     updated_at = excluded.updated_at",
            )
            .bind(event_type_id)
            .bind(i64::try_from(cents).unwrap_or(i64::MAX))
            .bind(window)
            .bind(i64::from(on))
            .bind(super::stamp(Utc::now()))
            .execute(pool)
            .await
        }
        Some(_) => {
            tracing::warn!(target: TARGET, event_type_id, "a team event type cannot be priced");
            return;
        }
        None if !on && usd.trim().is_empty() => {
            sqlx::query("DELETE FROM ton_prices WHERE event_type_id = ?")
                .bind(event_type_id)
                .execute(pool)
                .await
        }
        None => return,
    };
    match result {
        Ok(_) => {
            tracing::info!(
                target: TARGET,
                event_type_id,
                cents = cents.unwrap_or(0),
                enabled = on && cents.is_some(),
                "event type price saved"
            );
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
        "INSERT INTO ton_prices (event_type_id, price_usd_cents, window_min, enabled, updated_at)
         SELECT ?, p.price_usd_cents, p.window_min, p.enabled, ?
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

fn price_ctx(state: &AppState, enabled: bool, usd: String, window: String) -> minijinja::Value {
    let (payments_on, default_window) = match state.ton.get() {
        Some(rt) => (true, rt.cfg.window_min),
        None => (false, 30),
    };
    context! {
        enabled => enabled,
        usd => usd,
        window => window,
        default_window => default_window,
        payments_on => payments_on,
    }
}

/// The price section of a personal event type's form; `None` (no section)
/// for a team type. `event_type_id = None` for a new one.
pub(crate) async fn price_form_ctx(
    state: &AppState,
    event_type_id: Option<&str>,
) -> Option<minijinja::Value> {
    let Some(et_id) = event_type_id else {
        return Some(price_ctx(state, false, String::new(), String::new()));
    };
    let row: Option<(Option<i64>, Option<i64>, Option<i64>, i64)> = sqlx::query_as(
        "SELECT p.price_usd_cents, p.window_min, p.enabled, et.team_id IS NULL
         FROM event_types et LEFT JOIN ton_prices p ON p.event_type_id = et.id
         WHERE et.id = ?",
    )
    .bind(et_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    let (cents, window, enabled, personal) = row?;
    if personal == 0 {
        return None;
    }
    Some(price_ctx(
        state,
        enabled == Some(1),
        cents
            .map(|c| format_usd_cents(from_i64(c)))
            .unwrap_or_default(),
        window.map(|w| w.to_string()).unwrap_or_default(),
    ))
}

/// The section echoed back after a validation error, as posted.
pub(crate) fn price_form_from_post(
    state: &AppState,
    enabled: Option<&str>,
    usd: Option<&str>,
    window_min: Option<&str>,
    personal: bool,
) -> Option<minijinja::Value> {
    personal.then(|| {
        price_ctx(
            state,
            checked(enabled),
            usd.unwrap_or("").trim().to_string(),
            window_min.unwrap_or("").trim().to_string(),
        )
    })
}

/// The price line of a public slot or form page: the USD price and, when
/// a rate is cached, the approximate amount in TON (the exact one is fixed
/// when the guest books). Never fetches on a page view.
pub(crate) async fn price_badge(state: &AppState, event_type_id: &str) -> Option<minijinja::Value> {
    let price = store::price_for(&state.pool, event_type_id).await.ok()??;
    if price.is_team != 0 {
        return None;
    }
    let cents = u64::try_from(price.price_usd_cents).ok()?;
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
    )> = sqlx::query_as(
        "SELECT i.booking_id, i.status, i.problem, i.amount_nano, i.paid_nano, i.excess_nano,
                    i.expires_at, b.status
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
    for (booking_id, status, problem, amount_nano, paid_nano, excess_nano, expires, _booking) in
        rows
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
/// (event title, guest name, amount nano, paid nano, expires_at).
async fn waivable(
    pool: &SqlitePool,
    booking_id: &str,
    user_id: &str,
) -> Option<(String, String, i64, i64, String)> {
    sqlx::query_as(
        "SELECT et.title, b.guest_name, i.amount_nano, i.paid_nano, i.expires_at
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
    let Some((title, guest, amount_nano, paid_nano, expires)) =
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

    #[test]
    fn the_price_section_is_validated_on_the_server() {
        assert_eq!(validate_price_form(None, None, None, None), Ok(()));
        assert_eq!(validate_price_form(None, Some(""), Some(""), None), Ok(()));
        assert_eq!(
            validate_price_form(Some("on"), Some("40"), None, None),
            Ok(())
        );
        assert_eq!(
            validate_price_form(Some("on"), Some("40.50"), Some("15"), Some("")),
            Ok(())
        );
        for bad in ["", "0.99", "10000.01", "abc", "40.505", "-5", "1e3"] {
            assert_eq!(
                validate_price_form(Some("on"), Some(bad), None, None),
                Err("ton-form-error-price"),
                "{bad:?}"
            );
        }
        // A price kept switched off must still be a price.
        assert_eq!(
            validate_price_form(None, Some("x"), None, None),
            Err("ton-form-error-price")
        );
        for bad in ["9", "241", "x"] {
            assert_eq!(
                validate_price_form(Some("on"), Some("40"), Some(bad), None),
                Err("ton-form-error-window")
            );
        }
        assert_eq!(
            validate_price_form(Some("on"), Some("40"), None, Some("team-1")),
            Err("ton-form-error-team")
        );
        // Unpriced team types are fine.
        assert_eq!(
            validate_price_form(None, None, None, Some("team-1")),
            Ok(())
        );
    }
}
