//! ilia fork: real HTTP statuses and a styled page for upstream's bare
//! error answers.
//!
//! Upstream answers an unknown user, event type or team with HTTP 200 and
//! the bare message as the whole body ("User not found.", no page around
//! it), an unknown address with an empty 404, and a dead booking link with
//! the styled booking error page but still 200. This layer, just above the
//! language switcher's (`lang_switch`, the innermost one), fixes the status
//! and the page without touching the ~40 upstream call sites, so a rebase
//! onto a new release stays cheap:
//!
//! - a 200 `text/html` answer whose whole body is one of the messages of
//!   [`BARE`], in any shipped language, becomes [`page`] with that message
//!   and its status (404, 410, 403 or 400), whatever the method: a booking
//!   form refused with one of them (an invite expired before the guest sent
//!   it, a date that is not one) gets its status too;
//! - the booking error page (`render_booking_action_error_html_in`) asks for
//!   the status of its title through [`booking_error`], read back here from a
//!   task-local: 404 for an unknown or invalid link, 410 for one already used
//!   up, 400 for invalid booking details. Only for `GET` and `HEAD`: after a
//!   POST the page is a form's answer and keeps upstream's 200;
//! - an empty 404 to a browser (`GET` or `HEAD` accepting `text/html`)
//!   becomes the generic "Not found" page. Images, scripts and `fetch`
//!   calls keep upstream's empty answer.
//!
//! The page shows only what upstream already showed (the message) or a
//! generic text: no path, slug, token or id is echoed, and an unknown user
//! and a private team without its invite still read alike. Rendered outside
//! the switcher's page scope, it shows no language menu, whose links would
//! carry the request's path and query.

use super::branding::EmbedParams;
use super::*;
use axum::body::HttpBody;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::OnceLock;

use axum::http::StatusCode;

/// Upstream's bare messages and the status each one means. Only messages
/// that are the whole answer of a page; form errors after a POST that a
/// test or a script reads as 200 stay as they are.
pub(super) const BARE: &[(&str, StatusCode)] = &[
    ("error-user-not-found", StatusCode::NOT_FOUND),
    ("error-event-type-not-found", StatusCode::NOT_FOUND),
    ("error-private-event-type-not-found", StatusCode::NOT_FOUND),
    ("error-team-not-found", StatusCode::NOT_FOUND),
    ("error-team-not-found-or-not-admin", StatusCode::NOT_FOUND),
    ("error-source-not-found", StatusCode::NOT_FOUND),
    ("error-dynamic-group-public-only", StatusCode::NOT_FOUND),
    ("error-invite-invalid", StatusCode::NOT_FOUND),
    ("error-invite-required", StatusCode::FORBIDDEN),
    ("error-invite-expired", StatusCode::GONE),
    ("error-invite-used", StatusCode::GONE),
    ("error-invalid-date", StatusCode::BAD_REQUEST),
    ("error-invalid-time", StatusCode::BAD_REQUEST),
    ("error-invalid-date-format", StatusCode::BAD_REQUEST),
    ("error-invalid-time-format", StatusCode::BAD_REQUEST),
];

/// Titles of the booking error page and the status each one means. The
/// others (captcha, daily limit, host busy) keep upstream's 200.
pub(super) const BOOKING_ERRORS: &[(&str, StatusCode)] = &[
    ("bae-title-booking-not-found", StatusCode::NOT_FOUND),
    ("bae-title-invalid-link", StatusCode::NOT_FOUND),
    ("bae-title-invalid-or-expired", StatusCode::NOT_FOUND),
    ("bae-title-already-approved", StatusCode::GONE),
    ("bae-title-already-declined", StatusCode::GONE),
    ("bae-title-already-cancelled", StatusCode::GONE),
    ("bae-title-booking-cancelled", StatusCode::GONE),
    ("bae-title-booking-declined", StatusCode::GONE),
    ("bae-title-invalid-booking", StatusCode::BAD_REQUEST),
];

/// The longest body that can be a bare message. Real pages are tens of KB,
/// and only a body of known size up to this is ever read here.
const BARE_MAX: u64 = 1024;

tokio::task_local! {
    /// The status a handler asked for in this request ([`booking_error`]).
    static WANTED: Cell<Option<StatusCode>>;
}

/// Called by the booking error page with its (translated) title: records
/// the status that title means, applied by [`middleware`] to the answer.
/// Outside a request (unit tests rendering the page directly) it does
/// nothing.
pub(super) fn booking_error(lang: &str, title: &str) {
    let status = BOOKING_ERRORS
        .iter()
        .find(|(key, _)| crate::i18n::translate(lang, key, None) == title)
        .map(|(_, status)| *status);
    if let Some(status) = status {
        let _ = WANTED.try_with(|wanted| wanted.set(Some(status)));
    }
}

/// Every shipped translation of every [`BARE`] message, to its entry.
fn bare_messages() -> &'static HashMap<String, usize> {
    static MESSAGES: OnceLock<HashMap<String, usize>> = OnceLock::new();
    MESSAGES.get_or_init(|| {
        let mut map = HashMap::new();
        for (lang, _) in crate::i18n::supported_with_labels() {
            for (i, (key, _)) in BARE.iter().enumerate() {
                map.entry(crate::i18n::translate(lang, key, None))
                    .or_insert(i);
            }
        }
        map
    })
}

/// The [`BARE`] entry a body is, if it is one.
pub(super) fn bare_entry(body: &str) -> Option<usize> {
    bare_messages().get(body).copied()
}

/// The language to show a matched message in: the visitor's when the
/// message reads the same in it, else the first shipped language it is
/// written in (the handler may have answered in the host's language).
fn message_lang(preferred: &'static str, key: &str, message: &str) -> &'static str {
    if crate::i18n::translate(preferred, key, None) == message {
        return preferred;
    }
    crate::i18n::supported_with_labels()
        .map(|(lang, _)| lang)
        .find(|lang| crate::i18n::translate(lang, key, None) == message)
        .unwrap_or(preferred)
}

/// The page title for a status.
fn title_key(status: StatusCode) -> &'static str {
    match status {
        StatusCode::GONE => "ilia-error-gone",
        StatusCode::FORBIDDEN => "ilia-error-forbidden",
        StatusCode::BAD_REQUEST => "ilia-error-bad-request",
        _ => "ilia-error-not-found",
    }
}

/// How the request asked to be shown: embedded (only on the booking paths
/// that may be framed, as for the CSP) and its theme and accent.
pub(super) struct Chrome {
    pub(super) lang: &'static str,
    pub(super) embed: Option<EmbedParams>,
    pub(super) company_link: Option<String>,
}

/// The error page: base.html (theme, logo, footer), a title for the status
/// and the message, in `chrome.lang`.
pub(super) fn page(
    templates: &minijinja::Environment<'static>,
    chrome: &Chrome,
    status: StatusCode,
    message: &str,
) -> Response {
    let lang = chrome.lang;
    let embed = chrome.embed.as_ref();
    let html = templates
        .get_template("ilia/error_page.html")
        .and_then(|tmpl| {
            tmpl.render(context! {
                lang => lang,
                title => crate::i18n::translate(lang, title_key(status), None),
                message => message,
                company_link => chrome.company_link,
                embed => embed.is_some(),
                embed_theme => embed.and_then(|e| e.theme.as_deref()).unwrap_or(""),
                embed_brand => embed.and_then(|e| e.brand_css()).unwrap_or_default(),
            })
        });
    match html {
        Ok(html) => (status, Html(html)).into_response(),
        // The page is a nicety: without it the message still goes out,
        // escaped, with the right status.
        Err(e) => {
            tracing::error!(error = %e, "error page render failed");
            (status, Html(crate::email::h(message))).into_response()
        }
    }
}

fn is_page_method(method: &axum::http::Method) -> bool {
    method == axum::http::Method::GET || method == axum::http::Method::HEAD
}

/// Whether the request is a browser asking for a page.
fn wants_page(method: &axum::http::Method, headers: &HeaderMap) -> bool {
    is_page_method(method)
        && headers
            .get(axum::http::header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|accept| accept.contains("text/html"))
}

fn is_html(response: &Response) -> bool {
    response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("text/html"))
}

/// See the module documentation.
pub(super) async fn middleware(
    State(state): State<Arc<AppState>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let method = request.method().clone();
    let wants_page = wants_page(&method, request.headers());
    let lang = crate::i18n::detect_from_headers(request.headers());
    let embedded = is_embedded_booking_request(request.uri().path(), request.uri().query());
    let embed = embedded
        .then(|| axum::extract::Query::<EmbedParams>::try_from_uri(request.uri()).ok())
        .flatten()
        .map(|q| q.0);

    let (mut response, wanted) = WANTED
        .scope(Cell::new(None), async {
            let response = next.run(request).await;
            (response, WANTED.with(Cell::get))
        })
        .await;

    let status = response.status();
    // Only for a page opened from a link: after a POST the booking error
    // page is a form's answer and keeps upstream's 200.
    if status == StatusCode::OK && is_page_method(&method) {
        if let Some(wanted) = wanted {
            *response.status_mut() = wanted;
            return response;
        }
    }
    let size = response.body().size_hint().exact();

    if status == StatusCode::OK && is_html(&response) && size.is_some_and(|n| n <= BARE_MAX) {
        let (parts, body) = response.into_parts();
        let bytes = match axum::body::to_bytes(body, BARE_MAX as usize).await {
            Ok(bytes) => bytes,
            Err(e) => return internal_error_response("error page body", &e),
        };
        let entry = std::str::from_utf8(&bytes).ok().and_then(bare_entry);
        let Some(i) = entry else {
            return Response::from_parts(parts, axum::body::Body::from(bytes));
        };
        let (key, wanted) = BARE[i];
        // The body is a whole message of ours, valid UTF-8.
        let message = String::from_utf8_lossy(&bytes);
        let chrome = Chrome {
            lang: message_lang(lang, key, &message),
            embed,
            company_link: state.company_link.read().await.clone(),
        };
        let page = page(&state.templates, &chrome, wanted, &message);
        return without_body_for_head(&method, keep_headers(parts, page));
    }

    if status == StatusCode::NOT_FOUND && size == Some(0) && wants_page {
        let (parts, _) = response.into_parts();
        let chrome = Chrome {
            lang,
            embed,
            company_link: state.company_link.read().await.clone(),
        };
        let message = crate::i18n::translate(lang, "ilia-error-not-found-body", None);
        let page = page(&state.templates, &chrome, status, &message);
        return without_body_for_head(&method, keep_headers(parts, page));
    }
    response
}

/// A HEAD answer: the page's headers and length, no body. axum strips the
/// body of a matched route's HEAD answer below this layer, but the empty 404
/// of an unknown address comes from the fallback and the page from here, so
/// the body is dropped here too.
fn without_body_for_head(method: &axum::http::Method, response: Response) -> Response {
    if method != axum::http::Method::HEAD {
        return response;
    }
    let (mut parts, body) = response.into_parts();
    if let Some(len) = body.size_hint().exact() {
        parts
            .headers
            .insert(axum::http::header::CONTENT_LENGTH, len.into());
    }
    Response::from_parts(parts, axum::body::Body::empty())
}

/// The new page with the headers the handler set (cookies, cache control),
/// its own status, type and length.
fn keep_headers(mut parts: axum::http::response::Parts, page: Response) -> Response {
    let (new_parts, body) = page.into_parts();
    parts.status = new_parts.status;
    parts.headers.remove(axum::http::header::CONTENT_LENGTH);
    for (name, value) in &new_parts.headers {
        parts.headers.insert(name.clone(), value.clone());
    }
    Response::from_parts(parts, body)
}
