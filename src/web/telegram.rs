//! Telegram in the web layer (ilia fork; the feature itself is
//! `crate::telegram`): Profile & Settings → Telegram, where a user links their
//! own chat for host notifications and sets each event type's guest field, and
//! the helpers the booking pages call.
//!
//! The guest field is set here rather than in the event type form, so the
//! upstream form stays untouched. Not admin-gated, unlike SMS: a message costs
//! nothing, and only the chat that proved a booking's token receives anything
//! about it, so there is no third-party recipient to abuse. Whoever may manage
//! an event type sets its mode.

use super::*;
use crate::telegram::username::mode;
use std::sync::OnceLock;

/// Host link generations per user: 5 an hour.
fn link_limiter() -> &'static RateLimiter {
    static LIMITER: OnceLock<RateLimiter> = OnceLock::new();
    LIMITER.get_or_init(|| RateLimiter::new(5, 3600))
}

pub(super) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/dashboard/settings/telegram", get(settings_page))
        .route("/dashboard/settings/telegram/link", post(link))
        .route("/dashboard/settings/telegram/unlink", post(unlink))
        .route("/dashboard/settings/telegram/modes", post(save_modes))
}

/// The guest field's mode on the booking form: `off` while Telegram is off.
pub(super) async fn form_mode(pool: &SqlitePool, et_id: &str) -> &'static str {
    if !crate::telegram::enabled() {
        return mode::OFF;
    }
    crate::telegram::store::event_type_mode(pool, et_id).await
}

/// The guest's Telegram username for a booking POST, honouring the mode.
/// `Err` is the localised (title, message) for the booking error page.
pub(super) async fn resolve_guest_telegram(
    pool: &SqlitePool,
    et_id: &str,
    raw: Option<&str>,
    lang: &str,
) -> Result<Option<String>, (String, String)> {
    let guest_mode = form_mode(pool, et_id).await;
    crate::telegram::username::resolve(guest_mode, raw, lang)
}

/// The deep link as a template value: built from a validated bot username and
/// a base64url token, so it needs no entity escaping in an `href`.
pub(super) fn link_value(link: Option<String>) -> Option<minijinja::Value> {
    link.filter(|l| {
        l.starts_with("https://t.me/")
            && l.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-:/.?=".contains(&b))
    })
    .map(minijinja::Value::from_safe_string)
}

/// "Telegram @Real (entered @Typed)" per booking id that has a link row, for
/// the bookings dashboard. Never the chat id.
pub(super) async fn dashboard_labels(
    pool: &SqlitePool,
    ids: &[String],
    lang: &str,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return out;
    }
    let at = |u: &Option<String>| {
        u.as_deref()
            .filter(|u| crate::telegram::username::valid(u))
            .map(|u| format!("@{u}"))
    };
    for (id, entered, actual, linked) in crate::telegram::store::guest_labels(pool, ids).await {
        let entered = at(&entered);
        let label = if linked {
            let shown = at(&actual)
                .unwrap_or_else(|| crate::i18n::translate(lang, "tg-host-no-username", None));
            let mut label = tr1(lang, "tg-dashboard-label", "username", &shown);
            if let Some(entered) = entered.filter(|e| !e.eq_ignore_ascii_case(&shown)) {
                label.push(' ');
                label.push_str(&tr1(lang, "tg-dashboard-entered", "entered", &entered));
            }
            label
        } else if let Some(entered) = entered {
            format!(
                "{} {}",
                tr1(lang, "tg-dashboard-label", "username", &entered),
                crate::i18n::translate(lang, "tg-dashboard-not-linked", None)
            )
        } else {
            continue;
        };
        out.insert(id, label);
    }
    out
}

/// The event types `user` may set the guest field of: their own, and those of
/// teams they administer.
async fn manageable_event_types(
    pool: &SqlitePool,
    user_id: &str,
) -> Vec<(String, String, Option<String>)> {
    sqlx::query_as(
        "SELECT et.id, et.title, t.name
           FROM event_types et
           LEFT JOIN accounts a ON a.id = et.account_id
           LEFT JOIN teams t ON t.id = et.team_id
           LEFT JOIN team_members tm
             ON tm.team_id = et.team_id AND tm.user_id = ?1 AND tm.role = 'admin'
          WHERE (et.team_id IS NULL AND a.user_id = ?1)
             OR (et.team_id IS NOT NULL AND tm.user_id IS NOT NULL)
          ORDER BY t.name IS NOT NULL, t.name, et.title",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

/// "Host notifications go to @name. Linked: <date, time (zone)>", the date
/// in the user's own zone and language (stored as UTC). A chat with no
/// username has its own sentence rather than a "go to no username".
fn linked_line(lang: &str, timezone: &str, username: Option<&str>, linked_at: &str) -> String {
    let date = match crate::telegram::store::parse_sql_time(linked_at) {
        Some(at) => {
            let tz: chrono_tz::Tz = timezone.parse().unwrap_or(chrono_tz::Tz::UTC);
            let local = at.with_timezone(&tz);
            crate::telegram::message::when(
                lang,
                local.date_naive(),
                &local.format("%H:%M").to_string(),
                tz.name(),
            )
        }
        None => linked_at.to_string(),
    };
    match username.filter(|u| crate::telegram::username::valid(u)) {
        Some(name) => {
            let mut args = fluent_bundle::FluentArgs::new();
            args.set("username", format!("@{name}"));
            args.set("date", date);
            crate::i18n::translate(lang, "tg-settings-linked", Some(&args))
        }
        None => tr1(lang, "tg-settings-linked-no-username", "date", &date),
    }
}

#[derive(Default)]
struct PageState {
    deep_link: Option<String>,
    link_limited: bool,
    saved: bool,
}

async fn render_page(
    state: &AppState,
    auth_user: &crate::auth::AuthUser,
    page: PageState,
) -> Response {
    let user = &auth_user.user;
    let lang = auth_user.lang;
    let enabled = crate::telegram::enabled();
    let linked = crate::telegram::store::user_chat(&state.pool, &user.id).await;
    let event_types: Vec<minijinja::Value> = if enabled {
        let mut rows = Vec::new();
        for (id, title, team) in manageable_event_types(&state.pool, &user.id).await {
            let current = crate::telegram::store::event_type_mode(&state.pool, &id).await;
            rows.push(context! {
                id => id,
                title => title,
                team => team,
                mode => current,
            });
        }
        rows
    } else {
        Vec::new()
    };
    let (impersonating, impersonating_name, _) = impersonation_ctx(auth_user);
    let tmpl = match state.templates.get_template("ilia/telegram_settings.html") {
        Ok(t) => t,
        Err(e) => return internal_error_response("template", &e),
    };
    let linked_line = linked
        .as_ref()
        .map(|(username, at)| linked_line(lang, &user.timezone, username.as_deref(), at));
    let rendered = tmpl.render(context! {
        lang => lang,
        sidebar => sidebar_context(auth_user, "settings"),
        impersonating => impersonating,
        impersonating_name => impersonating_name,
        enabled => enabled,
        bot_ready => crate::telegram::bot_username().is_some(),
        linked => linked.is_some(),
        linked_line => linked_line,
        deep_link => link_value(page.deep_link),
        link_limited => page.link_limited,
        saved => page.saved,
        event_types => event_types,
        modes => mode::ALL,
    });
    match rendered {
        Ok(html) => Html(html).into_response(),
        Err(e) => internal_error_response("template render", &e),
    }
}

#[derive(Deserialize)]
struct SavedQuery {
    #[serde(default)]
    saved: Option<String>,
}

async fn settings_page(
    State(state): State<Arc<AppState>>,
    auth_user: crate::auth::AuthUser,
    Query(query): Query<SavedQuery>,
) -> Response {
    render_page(
        &state,
        &auth_user,
        PageState {
            saved: query.saved.is_some(),
            ..Default::default()
        },
    )
    .await
}

#[derive(Deserialize)]
struct CsrfForm {
    _csrf: Option<String>,
}

/// A fresh single-use `u_` deep link, shown once on the page.
async fn link(
    State(state): State<Arc<AppState>>,
    auth_user: crate::auth::AuthUser,
    headers: HeaderMap,
    Form(form): Form<CsrfForm>,
) -> Response {
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    // An admin looking through someone's eyes does not link their own chat
    // to that account.
    if auth_user.impersonation.is_some() {
        return axum::http::StatusCode::FORBIDDEN.into_response();
    }
    let Some(rt) = crate::telegram::current() else {
        return render_page(&state, &auth_user, PageState::default()).await;
    };
    if rt.bot_username().is_none() {
        return render_page(&state, &auth_user, PageState::default()).await;
    }
    if link_limiter().check_limited(&auth_user.user.id).await {
        tracing::warn!(user_id = %auth_user.user.id, "telegram link generation rate limited");
        return render_page(
            &state,
            &auth_user,
            PageState {
                link_limited: true,
                ..Default::default()
            },
        )
        .await;
    }
    let (token, hash) = crate::telegram::tokens::new_token(crate::telegram::tokens::Kind::User);
    if let Err(e) =
        crate::telegram::store::new_user_token(&state.pool, &auth_user.user.id, &hash).await
    {
        return internal_error_response("telegram link token", &e);
    }
    tracing::info!(user_id = %auth_user.user.id, "telegram host link created");
    render_page(
        &state,
        &auth_user,
        PageState {
            deep_link: rt.deep_link(&token),
            ..Default::default()
        },
    )
    .await
}

async fn unlink(
    State(state): State<Arc<AppState>>,
    auth_user: crate::auth::AuthUser,
    headers: HeaderMap,
    Form(form): Form<CsrfForm>,
) -> Response {
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    if auth_user.impersonation.is_some() {
        return axum::http::StatusCode::FORBIDDEN.into_response();
    }
    crate::telegram::store::unlink_user(&state.pool, &auth_user.user.id).await;
    tracing::info!(user_id = %auth_user.user.id, "telegram host chat unlinked");
    Redirect::to("/dashboard/settings/telegram").into_response()
}

/// `mode_<event type id>=off|optional|required` for each event type shown.
/// Nothing is saved when one of them is not the user's to manage.
async fn save_modes(
    State(state): State<Arc<AppState>>,
    auth_user: crate::auth::AuthUser,
    headers: HeaderMap,
    Form(fields): Form<Vec<(String, String)>>,
) -> Response {
    let csrf = fields
        .iter()
        .find(|(k, _)| k == "_csrf")
        .map(|(_, v)| v.clone());
    if let Err(resp) = verify_csrf_token(&headers, &csrf) {
        return resp;
    }
    let mut changes = Vec::new();
    for (key, value) in &fields {
        let Some(et_id) = key.strip_prefix("mode_") else {
            continue;
        };
        if !can_manage_event_type(&state.pool, &auth_user.user, et_id).await {
            tracing::warn!(user_id = %auth_user.user.id, "telegram mode refused: event type not manageable");
            return axum::http::StatusCode::FORBIDDEN.into_response();
        }
        changes.push((et_id.to_string(), mode::parse(value)));
    }
    for (et_id, guest_mode) in changes {
        if let Err(e) =
            crate::telegram::store::save_event_type_mode(&state.pool, &et_id, guest_mode).await
        {
            return internal_error_response("telegram mode", &e);
        }
    }
    Redirect::to("/dashboard/settings/telegram?saved=1").into_response()
}
