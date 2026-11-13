//! The hooks upstream's booking, approval, reschedule, cancel and expiry
//! paths call. Each is one line at its call site; the logic is here.
//!
//! The rule they keep: a priced booking waits, holding its time, until its
//! payment is final or the host waives it. Every way to confirm or move an
//! unpaid priced booking (web Approve, the emailed Approve link, the MCP
//! approval through the dashboard handler, a host reschedule, a guest
//! reschedule) refuses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use chrono::{DateTime, Utc};
use minijinja::context;
use sqlx::{SqliteConnection, SqlitePool};

use super::invoice::{self, BookingAsk, OpenedInvoice, Prepared, Refusal};
use super::{parse_stamp, stamp, TARGET};
use crate::ton::ladder::{self, Step};
use crate::web::{
    html_response, internal_error_body, internal_error_response,
    render_booking_action_error_keys_in, AppState, EmbedParams,
};

/// ilia fork (the ladder): what a booking POST carried, to post it again
/// from the "the price changed" page. `fields` are the form's own values
/// (never `_csrf`, which the page adds, nor the captcha token, which is
/// single-use); `quoted_nano` is replaced by the new price.
pub(in crate::web) struct Repost<'a> {
    /// The form's action, with the embed query.
    pub(in crate::web) action: String,
    pub(in crate::web) fields: Vec<(&'static str, String)>,
    pub(in crate::web) embed: &'a EmbedParams,
}

/// ilia fork (debug booking): marks the "the price changed" page with the
/// price now, as a response extension (neither the body nor the status
/// changes). The debug booking handler, which posts its own quote and
/// cannot let the page's form repost outside its context, books once more
/// at that price instead of returning the page.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Requoted(pub(crate) Step);

/// The error page a refused priced booking shows, in the page language.
/// Nothing was created. `guest_email` is what the form carried: the "you
/// already have an unpaid booking" page offers to email that address the
/// links of its open booking (pay, or cancel to free the time). A changed
/// ladder price ([`Refusal::Requote`]) is the quote page instead.
pub(in crate::web) async fn refusal_page(
    state: &AppState,
    lang: &str,
    refusal: &Refusal,
    guest_email: Option<&str>,
    repost: &Repost<'_>,
) -> Response {
    match refusal {
        Refusal::TooLate => render_booking_action_error_keys_in(
            state,
            lang,
            "ton-bae-title-unavailable",
            "ton-bae-body-too-late",
        ),
        Refusal::AlreadyOpen => already_open_page(state, lang, guest_email),
        Refusal::PersonalOnly => render_booking_action_error_keys_in(
            state,
            lang,
            "ton-bae-title-unavailable",
            "ton-bae-body-personal-only",
        ),
        Refusal::FreeUsed => render_booking_action_error_keys_in(
            state,
            lang,
            "ton-bae-title-unavailable",
            "ton-bae-body-free-used",
        ),
        Refusal::Requote {
            quoted,
            now,
            free_used,
        } => quote_page(state, lang, *quoted, *now, *free_used, repost).await,
        Refusal::Unavailable(_)
        | Refusal::Capacity
        | Refusal::ClientLimit
        | Refusal::Rate(_)
        | Refusal::Internal => render_booking_action_error_keys_in(
            state,
            lang,
            "ton-bae-title-unavailable",
            "ton-bae-body-unavailable",
        ),
    }
}

/// "The price changed": the old and the new price and one button that
/// books at the new one (the same form, posted again with the new quote).
/// Nothing was booked; the guest is never charged more than they saw. The
/// reason given is neutral (the price may have gone up or down, from a
/// booking, a cancellation or a ladder change), or, with `free_used`, that
/// this guest's free step is used, so they book at the first price.
async fn quote_page(
    state: &AppState,
    lang: &str,
    quoted: Option<Step>,
    now: Step,
    free_used: bool,
    repost: &Repost<'_>,
) -> Response {
    let tmpl = match state.templates.get_template("ton_quote.html") {
        Ok(t) => t,
        Err(e) => return internal_error_response("internal", &e),
    };
    let fields: Vec<minijinja::Value> = repost
        .fields
        .iter()
        .filter(|(name, _)| *name != "quoted_nano")
        .map(|(name, value)| context! { name => *name, value => value })
        .collect();
    let captcha =
        crate::web::captcha::CaptchaVars::from_config(&*state.captcha_config.read().await);
    let embed = repost.embed;
    let rendered = tmpl
        .render(context! {
            company_link => state.company_link_now(),
            lang => lang,
            action => &repost.action,
            fields => fields,
            quoted_nano => now.quote_value(),
            was_shown => quoted.is_some(),
            why_key => quote_why_key(quoted, free_used),
            was_ton => quoted.and_then(Step::nano).map(ladder::ton_text),
            now_ton => now.nano().map(ladder::ton_text),
            captcha_enabled => captcha.enabled,
            captcha_api_endpoint => captcha.api_endpoint,
            captcha_widget_url => captcha.widget_url,
            captcha_wasm_url => captcha.wasm_url,
            embed => embed.is_embedded(),
            embed_brand => embed.brand_css().unwrap_or_default(),
            embed_theme => embed.theme.as_deref().unwrap_or(""),
        })
        .unwrap_or_else(|e| internal_error_body("template render", &e));
    let mut resp = html_response(rendered, embed);
    resp.extensions_mut().insert(Requoted(now));
    resp
}

/// The quote page's explanation: the guest's free step is used, the price
/// changed since it was shown (in either direction, for any reason), or
/// the form carried no price to compare.
pub(in crate::web) fn quote_why_key(quoted: Option<Step>, free_used: bool) -> &'static str {
    match (free_used, quoted.is_some()) {
        (true, _) => "ton-quote-why-free-used",
        (false, true) => "ton-quote-why",
        (false, false) => "ton-quote-confirm-help",
    }
}

fn already_open_page(state: &AppState, lang: &str, guest_email: Option<&str>) -> Response {
    let tmpl = match state.templates.get_template("ton_already_open.html") {
        Ok(t) => t,
        Err(e) => return internal_error_response("internal", &e),
    };
    let rendered = tmpl
        .render(context! {
            company_link => state.company_link_now(),
            lang => lang,
            email => guest_email.map(str::trim).filter(|e| !e.is_empty()),
        })
        .unwrap_or_else(|e| internal_error_body("template render", &e));
    axum::response::Html(rendered).into_response()
}

/// Before the booking transaction: `Prepared::Unpriced` for a type without
/// a price (the upstream path, unchanged); the prepared invoice for a
/// priced one; a free ladder step (pending, the owner approves it); the
/// refusal page when it cannot be booked now, or the quote page when a
/// ladder price is not the one the guest was shown.
#[allow(clippy::result_large_err)] // one per request, returned at once
pub(in crate::web) async fn prepare_for_booking(
    state: &AppState,
    ask: &BookingAsk<'_>,
    lang: &str,
    repost: &Repost<'_>,
) -> Result<Prepared, Response> {
    match invoice::prepare(state, ask, Utc::now()).await {
        Ok(prepared) => Ok(prepared),
        Err(refusal) => {
            tracing::warn!(target: TARGET, event_type_id = ask.event_type_id, reason = %refusal, "priced booking refused");
            Err(refusal_page(state, lang, &refusal, Some(ask.guest_email), repost).await)
        }
    }
}

/// The booking's stored start for [`BookingAsk`]: an unreadable start
/// leaves no time to pay (refused when priced).
pub(crate) fn start_of(start_at: &str) -> DateTime<Utc> {
    parse_stamp(start_at).unwrap_or(DateTime::UNIX_EPOCH)
}

/// Inside the booking transaction, right after the booking row and its
/// deadline: recount a ladder price and open the invoice (it re-checks
/// the limits under the write lock), or record a free step. On a refusal
/// the caller rolls back and returns the page.
#[allow(clippy::result_large_err)] // one per request, returned at once
pub(in crate::web) async fn open_for_booking(
    conn: &mut SqliteConnection,
    booking_id: &str,
    prepared: &Prepared,
    state: &AppState,
    lang: &str,
    repost: &Repost<'_>,
) -> Result<Option<OpenedInvoice>, Response> {
    match invoice::open(conn, booking_id, prepared).await {
        Ok(opened) => Ok(opened),
        Err(refusal) => {
            let invoice = match prepared {
                Prepared::Invoice(p) => Some(&**p),
                _ => None,
            };
            if refusal == Refusal::Capacity {
                if let (Some(rt), Some(p)) = (state.ton.get(), invoice) {
                    rt.note_capacity_refusal(p.created_at);
                }
            }
            tracing::warn!(target: TARGET, booking_id, reason = %refusal, "booking refused in the booking transaction");
            let email = match prepared {
                Prepared::Invoice(p) => Some(p.guest_email_norm.as_str()),
                Prepared::Free(f) => Some(f.quote.guest.email_norm.as_str()),
                Prepared::Unpriced => None,
            };
            Err(refusal_page(state, lang, &refusal, email, repost).await)
        }
    }
}

/// After the booking committed: no booking emails, no calendar push, no
/// SMS (nothing is confirmed), and no host Telegram message. The guest goes
/// straight to the payment page, which offers the Telegram link when the
/// event type asks for it (`telegram_username`: what the form carried);
/// inside the embedded widget, to a short page whose link opens it in the
/// whole window (the payment page is not frameable, and wallets do not
/// open `ton://` from an iframe).
pub(in crate::web) async fn after_booking(
    state: &AppState,
    booking_id: &str,
    opened: &OpenedInvoice,
    telegram_username: Option<&str>,
    embed: &EmbedParams,
    lang: &str,
) -> Response {
    if let Some(rt) = state.ton.get() {
        invoice::invoice_opened(&rt, booking_id, opened);
    }
    super::telegram::booking_created(state, booking_id, &opened.pay_token, telegram_username).await;
    let pay_url = format!("/booking/pay/{}", opened.pay_token);
    if !embed.is_embedded() {
        return Redirect::to(&pay_url).into_response();
    }
    let guest_tz: Option<String> =
        sqlx::query_scalar("SELECT guest_timezone FROM bookings WHERE id = ?")
            .bind(booking_id)
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten()
            .flatten();
    let tz = guest_tz
        .as_deref()
        .and_then(|z| z.parse::<chrono_tz::Tz>().ok())
        .unwrap_or(chrono_tz::UTC);
    let held_until = opened
        .expires_at
        .with_timezone(&tz)
        .format("%H:%M")
        .to_string();
    let tmpl = match state.templates.get_template("ton_pay_exit.html") {
        Ok(t) => t,
        Err(e) => return internal_error_response("internal", &e),
    };
    let rendered = tmpl
        .render(context! {
            company_link => state.company_link_now(),
            pay_url => pay_url,
            held_until => held_until,
            zone => tz.name(),
            lang => lang,
            embed => true,
            embed_brand => embed.brand_css().unwrap_or_default(),
            embed_theme => embed.theme.as_deref().unwrap_or(""),
        })
        .unwrap_or_else(|e| internal_error_body("template render", &e));
    html_response(rendered, embed)
}

/// The group paths (team types, dynamic group links) never book a priced
/// type: a dynamic group link takes the first member's personal type.
pub(crate) fn refuse_group(state: &AppState, lang: &str) -> Response {
    render_booking_action_error_keys_in(
        state,
        lang,
        "ton-bae-title-unavailable",
        "ton-bae-body-personal-only",
    )
}

/// The emailed Approve link (and the watcher's own call, which only comes
/// once the invoice is paid): the page an unpaid priced booking answers.
pub(crate) async fn approval_refused_page(
    state: &AppState,
    booking_id: &str,
    lang: &str,
) -> Option<Response> {
    match invoice::approval_check(&state.pool, booking_id).await {
        Ok(false) => return None,
        Ok(true) => {}
        // Refused all the same, but not called unpaid: the booking may
        // well be free of charge.
        Err(e) => return Some(internal_error_response("booking payment check", &e)),
    }
    tracing::warn!(target: TARGET, booking_id, "approval refused: the booking is not paid");
    // The page sends the host to the bookings list, where "Confirm without
    // payment" is: a link to it, so the page is no dead end.
    Some(render_booking_action_error_keys_with_action_in(
        state,
        lang,
        "ton-bae-title-unpaid",
        "ton-bae-body-unpaid",
        "/dashboard/bookings",
        "nav-bookings",
    ))
}

/// The same guard for the Approve form (GET), found by its token.
pub(crate) async fn approval_refused_for_token(
    state: &AppState,
    confirm_token: &str,
    lang: &str,
) -> Option<Response> {
    let booking_id: Option<String> =
        sqlx::query_scalar("SELECT id FROM bookings WHERE confirm_token = ?")
            .bind(confirm_token)
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten();
    approval_refused_page(state, &booking_id?, lang).await
}

/// A guest may move a paid (or waived) booking, never an unpaid one: not
/// even after the host asked for a new time (`reschedule_by_host`), which
/// would confirm it at the guest's pick.
pub(crate) async fn guest_reschedule_refused(
    state: &AppState,
    booking_id: &str,
    lang: &str,
) -> Option<Response> {
    if !invoice::unpaid(&state.pool, booking_id).await {
        return None;
    }
    // While the invoice is still open, the page links to the payment page
    // (the guest reached it with the reschedule token, which the same email
    // as the pay link carries). An expired or closed invoice can no longer
    // be paid: no link then.
    let pay_token: Option<String> = sqlx::query_scalar(
        "SELECT pay_token FROM ton_invoices WHERE booking_id = ? AND status = 'open'",
    )
    .bind(booking_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    Some(match pay_token {
        Some(pay_token) => render_booking_action_error_keys_with_action_in(
            state,
            lang,
            "ton-bae-title-unpaid",
            "ton-bae-body-reschedule-unpaid",
            &format!("/booking/pay/{pay_token}"),
            "ton-exit-go",
        ),
        None => render_booking_action_error_keys_in(
            state,
            lang,
            "ton-bae-title-unpaid",
            "ton-bae-body-reschedule-unpaid",
        ),
    })
}

/// ilia fork: upstream's booking error page (`booking_action_error.html`)
/// with one button under the message, for the unpaid-booking refusals whose
/// next step is known. The title, message and button label are i18n keys.
fn render_booking_action_error_keys_with_action_in(
    state: &AppState,
    lang: &str,
    title_key: &str,
    message_key: &str,
    href: &str,
    label_key: &str,
) -> Response {
    let title = crate::i18n::translate(lang, title_key, None);
    // The status this title means, as upstream's helper sets it.
    crate::web::error_page::booking_error(lang, &title);
    let tmpl = match state.templates.get_template("booking_action_error.html") {
        Ok(t) => t,
        Err(e) => return internal_error_response("internal", &e),
    };
    axum::response::Html(
        tmpl.render(context! {
            company_link => state.company_link_now(),
            title => title,
            message => crate::i18n::translate(lang, message_key, None),
            action_href => href,
            action_label => crate::i18n::translate(lang, label_key, None),
            lang => lang,
        })
        .unwrap_or_else(|e| internal_error_body("template render", &e)),
    )
    .into_response()
}

/// A booking was cancelled or declined: its open invoice closes now, a paid
/// one is flagged for a refund.
pub(crate) async fn booking_ended(pool: &SqlitePool, booking_id: &str) {
    invoice::on_booking_ended(pool, booking_id, Utc::now()).await;
}

/// The expiry sweep declined `booking_id`. For a booking with an invoice,
/// upstream's notices (guest, host, attendees, SMS) do not apply: the host
/// was never asked to approve, and the guest watched the countdown. The
/// guest hears that the booking was released when they asked for the link
/// by email or paid any of it (the address is then proven); a guest who
/// paid in full (the time was taken meanwhile) always hears that the
/// payment will be refunded. A
/// waived booking is an ordinary request by now: upstream's notices go out.
/// Returns `true` when this handled the booking (upstream then sends
/// nothing).
pub(crate) async fn handle_expired_notice(
    pool: &SqlitePool,
    smtp: Option<&crate::email::SmtpConfig>,
    booking_id: &str,
) -> bool {
    type Row = (String, String, Option<String>, i64, Option<String>);
    let row: Option<Row> = match sqlx::query_as(
        "SELECT id, status, problem, paid_nano, link_emailed_at
         FROM ton_invoices WHERE booking_id = ?",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            // Unknown: no notice rather than the wrong one, unless the
            // booking's type has never been priced (then it is upstream's).
            let free = matches!(priced_type_of(pool, booking_id).await, Ok(false));
            tracing::error!(
                target: TARGET,
                error = %e,
                booking_id,
                upstream_notices = free,
                "cannot read the invoice of an expired booking"
            );
            return !free;
        }
    };
    let Some((invoice_id, status, problem, paid_nano, link_emailed)) = row else {
        return false;
    };
    if status == "waived" {
        return false;
    }
    let now = Utc::now();
    // The hard deadline (no watcher) leaves the invoice open: it expires
    // with the booking, and a payment found later is late.
    let _ = sqlx::query(
        "UPDATE ton_invoices SET status = 'expired', closed_at = ?,
             problem = CASE WHEN paid_nano > 0 THEN 'partial' ELSE problem END
         WHERE id = ? AND status = 'open'",
    )
    .bind(stamp(now))
    .bind(&invoice_id)
    .execute(pool)
    .await;
    tracing::info!(target: TARGET, booking_id, invoice = %invoice_id, paid_nano, "priced booking released");
    let reason = match status.as_str() {
        "paid" | "confirmed" => crate::email::TonReleased::Refund,
        "closed" if problem.as_deref() == Some("host_busy_unpaid") => {
            crate::email::TonReleased::Taken
        }
        "open" | "expired" | "closed" => crate::email::TonReleased::Expired,
        _ => return true,
    };
    let told_anyway = reason == crate::email::TonReleased::Refund;
    // The typed address is not proven: only a guest who asked for the
    // link by email or paid any of it hears the end (ilia fork: the same
    // guests a renewal emails; others heard of a renewal in Telegram only).
    if !told_anyway && link_emailed.is_none() && paid_nano == 0 {
        return true;
    }
    let Some(smtp) = smtp else {
        return true;
    };
    let claimed = sqlx::query(
        "UPDATE ton_invoices SET guest_expiry_sent_at = ? WHERE id = ? AND guest_expiry_sent_at IS NULL",
    )
    .bind(stamp(now))
    .bind(&invoice_id)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or(false);
    if !claimed {
        return true;
    }
    let Some((mail, et_id)) = super::page::guest_mail(pool, booking_id).await else {
        return true;
    };
    let rebook = crate::web::event_type_rebook_url(pool, &et_id).await;
    let result = crate::email::send_guest_ton_released(
        smtp,
        &mail,
        reason,
        rebook.as_deref(),
        paid_nano > 0,
    )
    .await;
    crate::web::log_email_result(result, "guest_ton_released", booking_id);
    true
}

/// Whether the booking's event type has a price row (enabled or not).
async fn priced_type_of(pool: &SqlitePool, booking_id: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM bookings b JOIN ton_prices p ON p.event_type_id = b.event_type_id
                        WHERE b.id = ?)",
    )
    .bind(booking_id)
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
}

/// A plain 404 for a payment link that names nothing.
pub(crate) fn not_found(state: &AppState, lang: &str) -> Response {
    let page = render_booking_action_error_keys_in(
        state,
        lang,
        "bae-title-invalid-link",
        "ton-bae-body-pay-link-invalid",
    );
    (StatusCode::NOT_FOUND, page).into_response()
}
