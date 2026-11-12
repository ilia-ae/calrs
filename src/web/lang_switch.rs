//! ilia fork: the language switcher of public pages.
//!
//! Every public page (profiles, slots, booking forms, guest and host token
//! pages, TON pay pages, login and MFA) shows a small menu of the bundled
//! languages in its top corner. Each entry is a plain link to the same page
//! with `?lang=<code>` added, so the menu works without JavaScript (a small
//! script only closes it on Escape or a click elsewhere, and points the links
//! at the URL a page changed in place, like the slots page's month arrows):
//!
//! 1. [`middleware`] sees `lang` on a GET, stores a supported code in the
//!    `calrs_lang` cookie ([`crate::i18n::LANG_COOKIE`]) and answers with a
//!    303 to the same path without `lang`, other query parameters kept. An
//!    unknown value sets nothing and is never echoed; the redirect only ever
//!    goes to a path on this origin. The cookie lasts a year only when the
//!    link was followed from this site or one sharing its domain
//!    (`Sec-Fetch-Site`, see [`from_this_site`]); a `?lang=` link from
//!    anywhere else (another site, a chat or mail app) gives that visit the
//!    language and is forgotten when the browser closes, and never replaces a
//!    language the visitor already picked.
//! 2. `crate::i18n::detect_from_headers` and `crate::i18n::page_lang` read the
//!    cookie first, so the choice beats the browser's `Accept-Language`, a
//!    host's "always my language" and the language stored with a booking on
//!    its guest pages. The dashboard keeps the user's saved language and only
//!    follows the cookie when none is saved (`crate::i18n::resolve`).
//! 3. While a GET page renders, [`middleware`] keeps its path and query in a
//!    task-local, from which the template function `lang_switcher()` builds
//!    the links. Embedded booking pages, the dashboard and pages answering a
//!    form POST (which have no URL to come back to) get no switcher.

use axum::extract::Request;
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use minijinja::{Environment, State, Value};

/// How long the choice is kept: a year.
const COOKIE_MAX_AGE_SECS: u32 = 365 * 24 * 60 * 60;

/// The query parameter the switcher's links carry.
const PARAM: &str = "lang";

tokio::task_local! {
    /// Path and query (without `lang`) of the GET page being rendered, when
    /// that page may show the switcher; `None` otherwise.
    static PAGE: Option<String>;
}

/// Handle `?lang=` (cookie and redirect) and remember the page for
/// `lang_switcher()`. Embedded booking pages are left alone: a third-party
/// iframe can neither keep a `SameSite=Lax` cookie nor show the switcher.
pub(super) async fn middleware(request: Request, next: Next) -> Response {
    let uri = request.uri();
    let path = uri.path();
    let query = uri.query();
    let get = matches!(*request.method(), Method::GET | Method::HEAD);
    let embedded = super::is_embedded_booking_request(path, query);
    if get && !embedded {
        if let Some(value) = param_value(query) {
            return switch(path, query, value, request.headers());
        }
    }
    let page = (get && !embedded && shows_switcher(path)).then(|| same_origin_target(path, query));
    PAGE.scope(page, next.run(request)).await
}

/// The dashboard has its own language setting (Profile & Settings).
fn shows_switcher(path: &str) -> bool {
    path != "/dashboard" && !path.starts_with("/dashboard/")
}

/// The raw value of the first `lang` parameter, when there is one.
fn param_value(query: Option<&str>) -> Option<&str> {
    query?.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        (key == PARAM).then_some(value)
    })
}

/// Whether a `?lang=` link was followed from this site (the switcher's own
/// links) or from one sharing its registrable domain, per the
/// `Sec-Fetch-Site` header every current browser sends. A browser too old to
/// send it is trusted, so the switcher keeps working there. `none` (a typed
/// URL, a bookmark, or a link opened from a chat or mail app) is not: such a
/// link may come from anyone.
fn from_this_site(headers: &HeaderMap) -> bool {
    match headers.get("sec-fetch-site") {
        None => true,
        Some(site) => matches!(site.as_bytes(), b"same-origin" | b"same-site"),
    }
}

/// How a `?lang=` request keeps its language: in a cookie lasting a year
/// (the visitor's own pick), for the browser session only (a link from
/// elsewhere, on a visitor with no pick of their own), or not at all (a link
/// from elsewhere must not replace the visitor's pick).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Keep {
    Year,
    Session,
    Not,
}

pub(crate) fn keep(headers: &HeaderMap) -> Keep {
    if from_this_site(headers) {
        Keep::Year
    } else if crate::i18n::chosen_from_headers(headers).is_some() {
        Keep::Not
    } else {
        Keep::Session
    }
}

/// The 303 answering `?lang=<value>`: the cookie when `value` is a bundled
/// language (see [`keep`]), and the page again without `lang`.
fn switch(path: &str, query: Option<&str>, value: &str, request: &HeaderMap) -> Response {
    let target = same_origin_target(path, query);
    let mut response = StatusCode::SEE_OTHER.into_response();
    let headers = response.headers_mut();
    // `same_origin_target` returns printable ASCII only.
    if let Ok(location) = HeaderValue::from_str(&target) {
        headers.insert(header::LOCATION, location);
    }
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Some(code) = crate::i18n::supported_code(value) {
        let persistent = match keep(request) {
            Keep::Year => Some(true),
            Keep::Session => Some(false),
            Keep::Not => None,
        };
        if let Some(persistent) = persistent {
            if let Ok(cookie) = HeaderValue::from_str(&cookie(code, secure_cookie(), persistent)) {
                headers.append(header::SET_COOKIE, cookie);
            }
        }
    }
    response
}

/// `Secure` whenever the instance is served over HTTPS (its base URL).
fn secure_cookie() -> bool {
    crate::settings::base_url().is_some_and(|url| {
        url.trim_start()
            .get(..8)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
    })
}

/// The `Set-Cookie` value keeping `code`, for a year when `persistent`, else
/// for the browser session. Not `HttpOnly`: it holds nothing secret, and a
/// page script may read it.
pub(crate) fn cookie(code: &str, secure: bool, persistent: bool) -> String {
    format!(
        "{}={code}; Path=/{}; SameSite=Lax{}",
        crate::i18n::LANG_COOKIE,
        if persistent {
            format!("; Max-Age={COOKIE_MAX_AGE_SECS}")
        } else {
            String::new()
        },
        if secure { "; Secure" } else { "" }
    )
}

/// `path` and `query` without `lang`, as a path on this origin. Leading
/// slashes and backslashes collapse to one slash, so the result never reads
/// as `//other.host`; anything but printable ASCII (no CR or LF can reach a
/// header) gives `/`. The parts stay percent-encoded as they arrived.
pub(crate) fn same_origin_target(path: &str, query: Option<&str>) -> String {
    let mut target = format!("/{}", path.trim_start_matches(['/', '\\']));
    let kept: Vec<&str> = query
        .into_iter()
        .flat_map(|q| q.split('&'))
        .filter(|pair| !pair.is_empty() && pair.split('=').next() != Some(PARAM))
        .collect();
    if !kept.is_empty() {
        target.push('?');
        target.push_str(&kept.join("&"));
    }
    if target.bytes().all(|b| b.is_ascii_graphic() && b != b'\\') {
        target
    } else {
        "/".to_string()
    }
}

/// The switcher's link to `page` in `code`.
fn link(page: &str, code: &str) -> String {
    let sep = if page.contains('?') { '&' } else { '?' };
    format!("{page}{sep}{PARAM}={code}")
}

/// Escape for a double-quoted HTML attribute, leaving `/` as it is.
fn attr_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

/// The menu entries for the page being rendered: `code`, `label` (the native
/// name), `short` (what the closed menu shows), `href` and `current`, sorted
/// by name. Empty outside a GET public page. German is listed once: on an
/// instance addressing visitors as "Sie" the cookie's `de` is read as
/// `de-formal` (`crate::i18n::chosen_from_headers`), so the entry is current
/// for either register.
pub(crate) fn entries(page: Option<&str>, current: &str) -> Vec<Value> {
    let Some(page) = page else {
        return Vec::new();
    };
    let current = if current == "de-formal" {
        "de"
    } else {
        current
    };
    let mut langs: Vec<(&str, &str)> = crate::i18n::supported_with_labels()
        .filter(|(code, _)| *code != "de-formal")
        .collect();
    langs.sort_by_key(|(_, label)| label.to_lowercase());
    langs
        .into_iter()
        .map(|(code, label)| {
            minijinja::context! {
                code => code,
                label => label,
                short => code.to_ascii_uppercase(),
                href => Value::from_safe_string(attr_escape(&link(page, code))),
                current => code == current,
            }
        })
        .collect()
}

/// `lang_switcher()`: [`entries`] for the page this task is rendering, with
/// the page's own `lang` marked current.
fn lang_switcher(state: &State) -> Vec<Value> {
    let page = PAGE.try_with(|page| page.clone()).ok().flatten();
    let current = state
        .lookup("lang")
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default();
    entries(page.as_deref(), &current)
}

/// The page [`middleware`] kept for the switcher of the request being
/// handled, for tests.
#[cfg(test)]
pub(crate) fn current_page() -> Option<String> {
    PAGE.try_with(|page| page.clone()).ok().flatten()
}

/// Run `f` as if rendering the GET page `page` (path and query), for tests.
#[cfg(test)]
pub(crate) fn with_page<R>(page: &str, f: impl FnOnce() -> R) -> R {
    PAGE.sync_scope(Some(page.to_string()), f)
}

pub(super) fn register(env: &mut Environment<'static>) {
    env.add_function("lang_switcher", lang_switcher);
}
