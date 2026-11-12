//! ilia fork: the debug booking page, `/dashboard/admin/debug-booking`.
//!
//! The owner tests the guest side of booking (the TON invoice, the payment
//! page, Telegram) again and again; this page saves typing the guest form
//! each time.
//!
//! **The switch** is a runtime setting resolved like the TON settings and
//! stored with them (`crate::ton::settings::DEBUG_BOOKING`, field
//! `debug_booking`): the environment `CALRS_DEBUG_BOOKING` (`1`, `true`,
//! `yes`, `on` / `0`, `false`, `no`, `off`; anything else is off), if set,
//! wins and locks the toggle; else its `ton_settings` row (owner `panel`, the
//! toggle at the top of `/dashboard/admin/ton`, or `apply`,
//! `site.ton.debug_booking`); else off. [`middleware`] resolves it on every
//! `/dashboard` request (one primary-key read): while off, the page and its
//! POSTs answer 404 before any extractor runs (so a member, a signed-out
//! visitor and a forged POST all see the same 404), and the sidebar link
//! (template function `debug_booking_on()`, reading [`Shown`]) is not drawn.
//! A toggle, or a `config apply` from another process, is live with the
//! next request: no restart. The toggle itself is [`toggle`] (admin, CSRF,
//! never while impersonating, refused while locked, audited in
//! `ton_settings_audit` and `calrs::audit`).
//!
//! What it does, and what it never does:
//!
//! - Admins only (`AdminUser`, 403 for anyone else), never while
//!   impersonating (403), CSRF on every POST.
//! - It lists the admin's own enabled personal event types (a slug starting
//!   with `test` is chosen first, then a priced one), the next free times
//!   computed the way the slot page computes them, and the guest form
//!   prefilled from the admin's profile: name, and as email only the
//!   admin's own sign-in or booking address (any other address is refused,
//!   so nobody can book "as" someone else from here), and the guest
//!   Telegram username ([`prefill_telegram`]): the username of the chat the
//!   admin linked (`telegram_user_chats`), else the stored setting
//!   `CALRS_DEBUG_BOOKING_TELEGRAM` (the Debug block's own form,
//!   [`save_telegram`], or `config apply`, `site.ton.debug_booking_telegram`;
//!   the environment wins and locks it, like the switch), else empty.
//! - Submitting calls the public booking handler itself
//!   (`handle_booking_for_user`) with the request's own headers: CSRF,
//!   captcha, the per-IP limit, validation, the slot and window checks, the
//!   TON quote and invoice, holds and emails are exactly a guest's. A
//!   private or internal type gets a one-time invite for the admin's own
//!   address first, as the owner would make one. The guest's answer (the
//!   redirect to the payment page, the confirmation page or a refusal) is
//!   returned unchanged. Payment is never skipped and never marked as
//!   received.
//! - A booking it made is marked in `debug_bookings` (migration
//!   `ilia_013_debug_bookings`), listed on the page, and cancelled by
//!   "Cancel all my test bookings" through the dashboard's own cancel
//!   (`cancel_booking`), only the admin's own test bookings.
//! - **Timing for this test booking** ([`TimingForm`]): "Normal" keeps the
//!   site's settings; "Fast test" (`DebugTiming::FAST`: 3 minutes to pay,
//!   one renewal of 3 minutes, a minute of grace and of margin) or custom
//!   minutes (window, renewed window, renewals) replace them for this
//!   booking's invoice only, so the owner can watch the window end, the
//!   renewal ask again (Telegram, email), the window end again and the
//!   booking be released, in minutes. The timing reaches the invoice only
//!   through a task-local set around this page's own call of the booking
//!   handler (`ton_pay::invoice::with_debug_timing`): a guest's form has no
//!   way to set it. The invoice stores it (`ton_invoices.debug_timing`, migration
//!   `ilia_014_debug_timing`), and the watcher honours it only for a booking
//!   marked here. The list shows each booking's timing and, while its
//!   invoice is open, a live countdown, refreshed in place every few seconds.
//! - "Telegram: deliver guest messages to my own linked Telegram chat"
//!   (ticked by default whenever it can be): the
//!   booking's guest link row is linked to the chat the admin linked in
//!   Profile & Settings → Telegram (`telegram_user_chats`), as if the admin
//!   had opened the guest's deep link from it, and the usual welcome (or
//!   `PaymentDue` for an unpaid invoice) is queued. The chat id stays in SQL
//!   and is never read into a page or a log.
//! - **Guest language** ([`guest_headers`]): the language the test guest
//!   books in, chosen on the page (every bundled locale, by its own name;
//!   by default the admin's own interface language, `users.language`, else
//!   the page's). The forwarded request carries it as the switcher's
//!   `calrs_lang` cookie (any other copy dropped, and the instance's
//!   formal-German header too, so `de` stays `de`), which every booking page
//!   reads first (`i18n::page_lang`), so the booking stores it as
//!   `bookings.language` and the guest's pages, emails and Telegram messages
//!   follow it; host emails keep each host's own language. Only this page's
//!   copy of the headers changes: a guest's booking is untouched.
//! - When Telegram is on and the admin has no linked chat, the page says at
//!   the top that the own-chat option is unavailable and links to
//!   Profile & Settings → Telegram.
//! - One `calrs::audit` event per attempt and per cancel-all.

use super::*;
use crate::telegram::username::mode;
use axum::http::StatusCode;

/// The page's address.
pub(crate) const PATH: &str = "/dashboard/admin/debug-booking";
/// "Cancel all my test bookings".
pub(crate) const CANCEL_PATH: &str = "/dashboard/admin/debug-booking/cancel-all";
/// The switch on the TON page.
pub(crate) const TOGGLE_PATH: &str = "/dashboard/admin/ton/debug-booking";
/// The guest Telegram username's form in the Debug block of the TON page.
pub(crate) const TELEGRAM_PATH: &str = "/dashboard/admin/ton/debug-booking/telegram";
/// The deployment variable (and the `ton_settings` key).
pub(crate) const ENV: &str = crate::ton::settings::DEBUG_BOOKING;
/// The prefilled guest Telegram username's variable (and key).
pub(crate) const TELEGRAM_ENV: &str = crate::ton::settings::DEBUG_BOOKING_TELEGRAM;

/// Free times offered on the page.
const SLOTS: usize = 12;
/// How far ahead the page looks for them (the booking window and the fork's
/// horizon cut it further).
const DAYS_AHEAD: i32 = 31;
/// Test bookings listed.
const LISTED: i64 = 20;
/// A one-time invite for a private type lasts this long.
const INVITE_MINUTES: i64 = 60;

/// The switch as last resolved, for the sidebar link: shared with the
/// template function `debug_booking_on()` ([`Shown::install`]), refreshed by
/// [`resolve`] (every `/dashboard` request, the toggle, startup).
#[derive(Clone, Default)]
pub(crate) struct Shown(Arc<std::sync::atomic::AtomicBool>);

impl Shown {
    fn set(&self, on: bool) {
        self.0.store(on, std::sync::atomic::Ordering::Relaxed);
    }

    pub(crate) fn get(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// The template function `debug_booking_on()` of `env`.
    pub(crate) fn install(&self, env: &mut minijinja::Environment<'static>) {
        let shown = self.clone();
        env.add_function("debug_booking_on", move || shown.get());
    }
}

/// The switch in effect now: `calrs serve`'s environment (the TON page's
/// snapshot, which carries `CALRS_DEBUG_BOOKING`) over the stored row over
/// off. An unreadable row is off (fail closed), logged. Updates [`Shown`].
pub(crate) async fn resolve(state: &AppState) -> crate::ton::settings::Resolved<bool> {
    let env = state.ton.env();
    let row: Result<Option<(String, String)>, sqlx::Error> =
        sqlx::query_as("SELECT value, owner FROM ton_settings WHERE key = ?")
            .bind(ENV)
            .fetch_optional(&state.pool)
            .await;
    let mut db = crate::ton::settings::DbValues::default();
    match row {
        Ok(Some((value, owner))) => {
            if let Some(owner) = crate::ton::settings::Owner::parse(&owner) {
                db.set(ENV, value, owner);
            }
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!(error = %e, "debug booking: the switch could not be read; the page stays off");
            state.debug_booking.set(false);
            return crate::ton::settings::Resolved {
                value: false,
                source: crate::ton::settings::Source::Default,
            };
        }
    }
    let resolved = crate::ton::settings::debug_booking(&|k| env.get(k), &db);
    state.debug_booking.set(resolved.value);
    resolved
}

/// Once at startup (`build_state`): resolve the switch for the link and say
/// in the log when the page is on, or when the environment sets it to
/// something that is not on/off (off, locked).
pub(crate) async fn startup(state: &AppState) {
    let resolved = resolve(state).await;
    let env = state.ton.env().get(ENV);
    if let Some(raw) = env.as_deref().filter(|v| !v.trim().is_empty()) {
        if crate::ton::settings::parse_bool(raw).is_none() {
            tracing::error!("{ENV} must be 0 or 1; the debug booking page stays off");
        }
    }
    if resolved.value {
        tracing::warn!(
            source = resolved.source.as_str(),
            "the debug booking page is on: admins get {PATH}; turn it off on {} after debugging",
            super::ton_pay::settings_page::PATH
        );
    }
}

/// Before every `/dashboard` route: resolve the switch (the sidebar link
/// follows it), and answer the page's routes with a plain 404 while it is
/// off, before any extractor (the error page layer above styles it).
pub(crate) async fn middleware(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let path = req.uri().path();
    if path.starts_with("/dashboard") {
        let ours = path == PATH || path.starts_with(&format!("{PATH}/"));
        if !resolve(&state).await.value && ours {
            return StatusCode::NOT_FOUND.into_response();
        }
    }
    next.run(req).await
}

/// The routes, always registered; [`middleware`] keeps them 404 while the
/// switch is off.
pub(super) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(PATH, get(page).post(book))
        .route(CANCEL_PATH, post(cancel_all))
        .route(TOGGLE_PATH, post(toggle))
        .route(TELEGRAM_PATH, post(save_telegram))
}

/// The stored debug rows (one read), for [`telegram_setting`].
async fn debug_rows(state: &AppState) -> crate::ton::settings::DbValues {
    let mut conn = match state.pool.acquire().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "debug booking: the settings could not be read");
            return Default::default();
        }
    };
    ton_pay::store::load_settings(&mut conn)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "debug booking: the settings could not be read");
            Default::default()
        })
}

/// The stored guest Telegram username (environment > row > none).
async fn telegram_setting(state: &AppState) -> crate::ton::settings::Resolved<Option<String>> {
    let db = debug_rows(state).await;
    let env = state.ton.env();
    crate::ton::settings::debug_booking_telegram(&|k| env.get(k), &db)
}

/// Where the prefilled guest Telegram username came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TelegramFrom {
    /// The username of the chat the admin linked.
    OwnChat,
    /// `CALRS_DEBUG_BOOKING_TELEGRAM` (the Debug block, `config apply`, the
    /// environment).
    Setting,
}

/// The guest Telegram username the form opens with: the admin's linked
/// chat's username, else the stored setting, else none.
pub(crate) async fn prefill_telegram(
    state: &AppState,
    user_id: &str,
) -> Option<(String, TelegramFrom)> {
    let own = crate::telegram::store::user_chat(&state.pool, user_id)
        .await
        .and_then(|(u, _)| u)
        .and_then(|u| crate::telegram::username::normalize(&u));
    if let Some(u) = own {
        return Some((u, TelegramFrom::OwnChat));
    }
    telegram_setting(state)
        .await
        .value
        .map(|u| (u, TelegramFrom::Setting))
}

/// What the TON page shows in its Debug block.
pub(crate) async fn block(state: &AppState) -> minijinja::Value {
    use crate::ton::settings::Source;
    let resolved = resolve(state).await;
    let env_value = state.ton.env().get(ENV).unwrap_or_default();
    context! {
        on => resolved.value,
        source => resolved.source.as_str(),
        locked => matches!(resolved.source, Source::Env | Source::Apply),
        var => ENV,
        env_value => env_value.trim(),
        path => PATH,
        toggle_path => TOGGLE_PATH,
        telegram => telegram_block(state).await,
    }
}

/// The guest Telegram username's part of the Debug block.
async fn telegram_block(state: &AppState) -> minijinja::Value {
    use crate::ton::settings::Source;
    let env = state.ton.env();
    let db = debug_rows(state).await;
    let resolved = crate::ton::settings::debug_booking_telegram(&|k| env.get(k), &db);
    let raw = match resolved.source {
        Source::Env => env.get(TELEGRAM_ENV).unwrap_or_default(),
        _ => db.value(TELEGRAM_ENV).unwrap_or_default().to_string(),
    };
    context! {
        value => resolved.value.clone().unwrap_or_else(|| raw.trim().to_string()),
        effective => resolved.value,
        source => resolved.source.as_str(),
        locked => matches!(resolved.source, Source::Env | Source::Apply),
        var => TELEGRAM_ENV,
        env_value => env.get(TELEGRAM_ENV).unwrap_or_default().trim(),
        path => TELEGRAM_PATH,
    }
}

// --- the switch -------------------------------------------------------------

#[derive(Deserialize)]
pub(super) struct ToggleForm {
    _csrf: Option<String>,
    /// `1` to switch on, anything else off.
    #[serde(default)]
    on: String,
}

/// `POST /dashboard/admin/ton/debug-booking`: switch the page on or off
/// (a `ton_settings` row owned by `panel`, with its audit row), unless the
/// environment or `config apply` owns the switch. Back to the TON page with
/// `?debug=on|off|locked`.
pub(super) async fn toggle(
    State(state): State<Arc<AppState>>,
    admin: crate::auth::AdminUser,
    viewer: crate::auth::OptionalAuthUser,
    headers: HeaderMap,
    Form(form): Form<ToggleForm>,
) -> Response {
    use crate::ton::settings::{Owner, Source};
    use ton_pay::settings as stored;
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    if viewer.impersonation.is_some() {
        return impersonating(admin.lang);
    }
    let back = super::ton_pay::settings_page::PATH;
    let want = form.on.trim() == "1";
    let env = state.ton.env();
    let env = |k: &str| env.get(k);
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_error_response("debug booking: begin", &e),
    };
    let before = match ton_pay::store::load_settings(&mut tx).await {
        Ok(db) => db,
        Err(e) => return internal_error_response("debug booking: read", &e),
    };
    if matches!(
        crate::ton::settings::source(&env, &before, ENV),
        Source::Env | Source::Apply
    ) {
        let _ = tx.rollback().await;
        tracing::info!(
            target: "calrs::audit",
            actor = %admin.user.email,
            "debug booking switch refused: locked"
        );
        return Redirect::to(&format!("{back}?debug=locked")).into_response();
    }
    let mut next = before.clone();
    next.set(ENV, if want { "1" } else { "0" }.to_string(), Owner::Panel);
    let changes = stored::diff(&before, &next);
    if !changes.is_empty() {
        if let Err(e) = stored::write(
            &mut tx,
            &changes,
            Some(&admin.user.id),
            Owner::Panel,
            Utc::now(),
        )
        .await
        {
            return internal_error_response("debug booking: write", &e);
        }
    }
    if let Err(e) = tx.commit().await {
        return internal_error_response("debug booking: commit", &e);
    }
    stored::log_changes(&changes, Some(&admin.user.id), Owner::Panel);
    let now = resolve(&state).await.value;
    tracing::info!(
        target: "calrs::audit",
        actor = %admin.user.email,
        on = now,
        changed = !changes.is_empty(),
        "debug booking switched"
    );
    Redirect::to(&format!("{back}?debug={}", if now { "on" } else { "off" })).into_response()
}

#[derive(Deserialize)]
pub(super) struct TelegramForm {
    _csrf: Option<String>,
    #[serde(default)]
    debug_booking_telegram: String,
}

/// `POST /dashboard/admin/ton/debug-booking/telegram`: store the guest
/// Telegram username the test booking page prefills (a `ton_settings` row
/// owned by `panel`, audited; blank removes it), checked like the booking
/// form's field, unless the environment or `config apply` owns it. Back to
/// the TON page with `?debug=tg-saved|tg-same|tg-invalid|tg-locked`.
pub(super) async fn save_telegram(
    State(state): State<Arc<AppState>>,
    admin: crate::auth::AdminUser,
    viewer: crate::auth::OptionalAuthUser,
    headers: HeaderMap,
    Form(form): Form<TelegramForm>,
) -> Response {
    use crate::ton::settings::{self as ts, Owner, Source};
    use ton_pay::settings as stored;
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    if viewer.impersonation.is_some() {
        return impersonating(admin.lang);
    }
    let back = |notice: &str| {
        Redirect::to(&format!(
            "{}?debug={notice}#debug-block",
            super::ton_pay::settings_page::PATH
        ))
        .into_response()
    };
    let Some(key) = ts::by_var(TELEGRAM_ENV) else {
        return internal_error_response("debug booking: telegram key", &"missing");
    };
    let text = match ts::to_env(key, &form.debug_booking_telegram) {
        Ok(t) => t,
        Err(_) => return back("tg-invalid"),
    };
    let env = state.ton.env();
    let env = |k: &str| env.get(k);
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_error_response("debug booking: begin", &e),
    };
    let before = match ton_pay::store::load_settings(&mut tx).await {
        Ok(db) => db,
        Err(e) => return internal_error_response("debug booking: read", &e),
    };
    if matches!(
        ts::source(&env, &before, TELEGRAM_ENV),
        Source::Env | Source::Apply
    ) {
        let _ = tx.rollback().await;
        tracing::info!(
            target: "calrs::audit",
            actor = %admin.user.email,
            "debug booking telegram username refused: locked"
        );
        return back("tg-locked");
    }
    let mut next = before.clone();
    match text {
        Some(t) => next.set(TELEGRAM_ENV, t, Owner::Panel),
        None => next.remove(TELEGRAM_ENV),
    }
    let changes = stored::diff(&before, &next);
    if changes.is_empty() {
        let _ = tx.rollback().await;
        return back("tg-same");
    }
    if let Err(e) = stored::write(
        &mut tx,
        &changes,
        Some(&admin.user.id),
        Owner::Panel,
        Utc::now(),
    )
    .await
    {
        return internal_error_response("debug booking: write", &e);
    }
    if let Err(e) = tx.commit().await {
        return internal_error_response("debug booking: commit", &e);
    }
    stored::log_changes(&changes, Some(&admin.user.id), Owner::Panel);
    tracing::info!(
        target: "calrs::audit",
        actor = %admin.user.email,
        set = next.value(TELEGRAM_ENV).is_some(),
        "debug booking telegram username saved"
    );
    back("tg-saved")
}

// --- the TEST badge ---------------------------------------------------------

/// Of `ids` (the bookings a dashboard list shows), the ones this page made
/// (a `debug_bookings` row), for their TEST badge. Whatever the switch.
pub(crate) async fn marked(pool: &SqlitePool, ids: &[String]) -> Vec<String> {
    if ids.is_empty() {
        return Vec::new();
    }
    let all: Vec<String> = sqlx::query_scalar("SELECT booking_id FROM debug_bookings")
        .fetch_all(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "debug booking: could not read the test marks");
            Vec::new()
        });
    all.into_iter().filter(|id| ids.contains(id)).collect()
}

/// The booking was made by this page (its payment page shows TEST).
pub(crate) async fn is_marked(pool: &SqlitePool, booking_id: &str) -> bool {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM debug_bookings WHERE booking_id = ?)")
        .bind(booking_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false)
}

// --- what the page reads ----------------------------------------------------

/// One of the admin's own event types.
struct OwnType {
    id: String,
    slug: String,
    title: String,
    visibility: String,
    priced: bool,
}

/// The admin's enabled personal event types (team types excluded: the
/// public path for them is another handler and refuses a price anyway).
async fn own_types(pool: &SqlitePool, user_id: &str) -> Vec<OwnType> {
    let rows: Vec<(String, String, String, String, i64)> = sqlx::query_as(
        "SELECT et.id, et.slug, et.title, et.visibility,
                EXISTS (SELECT 1 FROM ton_prices p WHERE p.event_type_id = et.id AND p.enabled = 1)
           FROM event_types et
           JOIN accounts a ON a.id = et.account_id
          WHERE a.user_id = ? AND et.enabled = 1 AND et.team_id IS NULL
          ORDER BY et.slug",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "debug booking: could not read event types");
        Vec::new()
    });
    rows.into_iter()
        .map(|(id, slug, title, visibility, priced)| OwnType {
            id,
            slug,
            title,
            visibility,
            priced: priced != 0,
        })
        .collect()
}

/// The type the page opens on: the one asked for, else the first whose slug
/// starts with `test`, else the first priced one, else the first.
fn chosen<'a>(types: &'a [OwnType], asked: Option<&str>) -> Option<&'a OwnType> {
    if let Some(slug) = asked.filter(|s| !s.is_empty()) {
        if let Some(t) = types.iter().find(|t| t.slug == slug) {
            return Some(t);
        }
    }
    types
        .iter()
        .find(|t| t.slug.starts_with("test"))
        .or_else(|| types.iter().find(|t| t.priced))
        .or_else(|| types.first())
}

/// The addresses the admin may book with: the sign-in email, then the
/// booking email when it is another one.
fn own_emails(user: &crate::models::User) -> Vec<String> {
    let mut out = vec![user.email.clone()];
    if let Some(b) = user
        .booking_email
        .as_deref()
        .map(str::trim)
        .filter(|b| !b.is_empty())
    {
        if !b.eq_ignore_ascii_case(&user.email) {
            out.push(b.to_string());
        }
    }
    out
}

/// The admin's zone, which the times are shown and booked in.
fn admin_tz(user: &crate::models::User) -> Tz {
    user.timezone.parse().unwrap_or(Tz::UTC)
}

/// The next free times of `et_id`, as `(value, label)`: `value` is
/// `YYYY-MM-DDTHH:MM` in `guest_tz`, what the booking form posts. Computed
/// like the slot page (on-demand sync, busy times, the TON notice, the
/// booking window and the horizon).
async fn next_slots(
    state: &AppState,
    user_id: &str,
    et_id: &str,
    guest_tz: Tz,
) -> Vec<(String, String)> {
    let row: Option<(i32, i32, i32, i32)> = sqlx::query_as(
        "SELECT duration_min, buffer_before, buffer_after, min_notice_min FROM event_types WHERE id = ?",
    )
    .bind(et_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    let Some((duration, buffer_before, buffer_after, min_notice)) = row else {
        return Vec::new();
    };
    crate::commands::sync::sync_if_stale(&state.pool, &state.secret_key, user_id).await;
    let host_tz = get_host_tz(&state.pool, et_id).await;
    let window = get_booking_window(&state.pool, et_id).await;
    let now_host = Utc::now().with_timezone(&host_tz).naive_local();
    let window_end = (now_host.date() + Duration::days(i64::from(DAYS_AHEAD) + 1))
        .and_hms_opt(23, 59, 59)
        .unwrap_or(now_host);
    let busy = BusySource::Individual(
        fetch_busy_times_for_user(
            &state.pool,
            user_id,
            now_host,
            window_end,
            host_tz,
            Some(et_id),
        )
        .await,
    );
    let min_notice = ton_pay::slot_min_notice(state, et_id, min_notice).await;
    let days = compute_slots(
        &state.pool,
        et_id,
        duration,
        buffer_before,
        buffer_after,
        min_notice,
        0,
        DAYS_AHEAD,
        window,
        host_tz,
        guest_tz,
        busy,
        None,
    )
    .await;
    days.iter()
        .flat_map(|d| {
            d.slots.iter().map(move |s| {
                (
                    format!("{}T{}", s.guest_date, s.start),
                    format!("{} · {}–{}", d.label, s.start, s.end),
                )
            })
        })
        .take(SLOTS)
        .collect()
}

/// One test booking as the page lists it.
#[derive(sqlx::FromRow)]
struct ListedRow {
    id: String,
    title: String,
    start_at: String,
    status: String,
    language: Option<String>,
    own_chat: i64,
    invoice: Option<String>,
    pay_token: Option<String>,
    debug_timing: Option<String>,
    expires_at: Option<String>,
    settle_by: Option<String>,
    renewals: Option<i64>,
    renewals_max: Option<i64>,
}

/// The admin's test bookings, newest first, for the page: status, invoice,
/// the timing it was made with and, while its invoice is open, its
/// deadlines (unix seconds, for the page's countdown).
async fn listed(pool: &SqlitePool, user_id: &str, tz: Tz, lang: &str) -> Vec<minijinja::Value> {
    let rows: Vec<ListedRow> = sqlx::query_as(
        "SELECT b.id, et.title, b.start_at, b.status, b.language, d.telegram_own_chat AS own_chat,
                i.status AS invoice, i.pay_token, i.debug_timing, i.expires_at, i.settle_by,
                i.renewals, i.renewals_max
               FROM debug_bookings d
               JOIN bookings b ON b.id = d.booking_id
               JOIN event_types et ON et.id = b.event_type_id
               LEFT JOIN ton_invoices i ON i.booking_id = b.id
              WHERE d.user_id = ?
              ORDER BY d.created_at DESC, d.rowid DESC
              LIMIT ?",
    )
    .bind(user_id)
    .bind(LISTED)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "debug booking: could not read test bookings");
        Vec::new()
    });
    rows.into_iter()
        .map(|r| {
            let when = crate::booking_time::local(&r.start_at, Tz::UTC, tz)
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or(r.start_at);
            let standing = matches!(r.status.as_str(), "pending" | "confirmed");
            let open = r.invoice.as_deref() == Some("open") && r.status == "pending";
            let unix = |s: &Option<String>| {
                s.as_deref()
                    .and_then(ton_pay::parse_stamp)
                    .map(|t| t.timestamp())
            };
            let local = |s: &Option<String>| {
                s.as_deref()
                    .and_then(ton_pay::parse_stamp)
                    .map(|t| t.with_timezone(&tz).format("%H:%M:%S").to_string())
            };
            let timing = r
                .debug_timing
                .as_deref()
                .and_then(crate::ton::deadlines::DebugTiming::parse);
            context! {
                id => r.id,
                title => r.title,
                when => when,
                status_label => status_label(lang, &r.status),
                status => r.status,
                language => r.language.as_deref().and_then(crate::i18n::supported_code),
                language_label => r.language.as_deref().and_then(lang_label),
                invoice_label => r.invoice.as_deref().map(|i| invoice_label(lang, i)),
                invoice => r.invoice,
                // The payment page while the booking stands (the admin's own).
                pay_url => r.pay_token.filter(|_| standing).map(|t| format!("/booking/pay/{t}")),
                own_chat => r.own_chat != 0,
                timing => r.invoice.is_some().then(|| timing_label(lang, timing)),
                fast => timing.is_some(),
                // Still moving: the page refreshes the list while one is.
                // Pending whatever the invoice says: an expired (or paid)
                // invoice leaves the booking pending until calrs's sweep
                // (or the confirmation) moves it, which the owner watches.
                live => r.status == "pending",
                open => open,
                expires_unix => open.then(|| unix(&r.expires_at)).flatten(),
                settle_unix => open.then(|| unix(&r.settle_by)).flatten(),
                expires_local => local(&r.expires_at),
                settle_local => local(&r.settle_by),
                renewals => r.renewals.unwrap_or(0),
                renewals_max => r.renewals_max.unwrap_or(0),
            }
        })
        .collect()
}

/// A bundled locale's own name ("Русский"), for the language select and
/// the list.
fn lang_label(code: &str) -> Option<&'static str> {
    let code = code.trim();
    crate::i18n::supported_with_labels()
        .find(|(c, _)| *c == code)
        .map(|(_, label)| label)
}

/// The language a test booking is made in when the form names none: the
/// admin's own interface language (`users.language`, else the page's, as
/// the `AdminUser` extractor resolved it).
fn default_guest_lang(admin: &crate::auth::AdminUser) -> &'static str {
    admin.lang
}

/// The guest language posted: `Ok(None)` when none was (the default
/// applies), `Err` for a value calrs does not ship.
fn posted_guest_lang(value: Option<&str>) -> Result<Option<&'static str>, ()> {
    match value.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(v) => crate::i18n::supported_code(v).map(Some).ok_or(()),
    }
}

/// This page's copy of the request headers for the booking handler, with
/// the guest language as the switcher's pick (`calrs_lang`, which every
/// booking page reads first): every other `calrs_lang` cookie is dropped,
/// the other cookies (the CSRF one) are kept in one `Cookie` header, and
/// the instance's formal-German header is removed so the stored language is
/// exactly the one chosen (`de-formal` is chosen as itself).
pub(crate) fn guest_headers(headers: &HeaderMap, lang: &'static str) -> HeaderMap {
    let mut out = headers.clone();
    let prefix = format!("{}=", crate::i18n::LANG_COOKIE);
    let mut parts: Vec<String> = headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|s| s.split(';'))
        .map(str::trim)
        .filter(|p| !p.is_empty() && !p.starts_with(prefix.as_str()))
        .map(str::to_string)
        .collect();
    parts.push(format!("{prefix}{lang}"));
    out.remove(axum::http::header::COOKIE);
    if let Ok(v) = axum::http::HeaderValue::from_str(&parts.join("; ")) {
        out.insert(axum::http::header::COOKIE, v);
    }
    out.remove(crate::i18n::GERMAN_FORMAL_HEADER);
    out
}

/// A test booking's timing as the page shows it.
fn timing_label(lang: &str, timing: Option<crate::ton::deadlines::DebugTiming>) -> String {
    let Some(t) = timing else {
        return crate::i18n::translate(lang, "dbg-timing-site", None);
    };
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("window", t.window_min);
    args.set("renewals", t.renewals_max);
    args.set("renew", t.renew_window_min);
    args.set("grace", t.grace_min);
    crate::i18n::translate(lang, "dbg-timing-summary", Some(&args))
}

/// A booking status as the page shows it.
fn status_label(lang: &str, status: &str) -> String {
    let key = match status {
        "pending" => "dbg-st-pending",
        "confirmed" => "dbg-st-confirmed",
        "cancelled" => "dbg-st-cancelled",
        "declined" => "dbg-st-declined",
        other => return other.to_string(),
    };
    crate::i18n::translate(lang, key, None)
}

/// An invoice status as the page shows it.
fn invoice_label(lang: &str, status: &str) -> String {
    let key = match status {
        "open" => "dbg-inv-open",
        "paid" => "dbg-inv-paid",
        "confirmed" => "dbg-inv-confirmed",
        "expired" => "dbg-inv-expired",
        "closed" => "dbg-inv-closed",
        "waived" => "dbg-inv-waived",
        other => return other.to_string(),
    };
    crate::i18n::translate(lang, key, None)
}

/// The admin's test bookings that still stand (pending or confirmed).
async fn standing(pool: &SqlitePool, user_id: &str) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT d.booking_id FROM debug_bookings d JOIN bookings b ON b.id = d.booking_id
          WHERE d.user_id = ? AND b.status IN ('pending', 'confirmed')
          ORDER BY d.created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "debug booking: could not read test bookings");
        Vec::new()
    })
}

// --- the page ---------------------------------------------------------------

#[derive(Deserialize, Default)]
pub(super) struct PageQuery {
    #[serde(default)]
    et: Option<String>,
    #[serde(default)]
    cancelled: Option<u32>,
}

/// What a refused POST of the page keeps.
#[derive(Default)]
struct Kept {
    error: Option<String>,
    /// A booking was made, but something the admin asked for was not done.
    warning: Option<String>,
    et: Option<String>,
    slot: Option<String>,
    name: Option<String>,
    email: Option<String>,
    telegram: Option<String>,
    notes: Option<String>,
    /// The Telegram box as posted; `None`: ticked whenever it can be.
    own_chat: Option<bool>,
    timing: TimingForm,
    /// The guest language as posted; `None`: [`default_guest_lang`].
    guest_lang: Option<String>,
}

/// "Timing for this test booking", as the form posts it: `timing` is
/// `normal` (the site's settings), `fast` (`DebugTiming::FAST`) or
/// `custom` (`t_window`, `t_renew_window`, `t_renewals`; a blank one is the
/// fast value).
#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct TimingForm {
    #[serde(default)]
    timing: Option<String>,
    #[serde(default)]
    t_window: Option<String>,
    #[serde(default)]
    t_renew_window: Option<String>,
    #[serde(default)]
    t_renewals: Option<String>,
}

impl TimingForm {
    /// The test booking's own timing: `Ok(None)` for the site's settings,
    /// `Err` for an unknown choice or a custom value out of bounds.
    pub(crate) fn timing(&self) -> Result<Option<crate::ton::deadlines::DebugTiming>, ()> {
        use crate::ton::deadlines::DebugTiming;
        match self.timing.as_deref().map(str::trim).unwrap_or("") {
            "" | "normal" => Ok(None),
            "fast" => Ok(Some(DebugTiming::FAST)),
            "custom" => {
                let num = |v: &Option<String>, fast: i64| match v.as_deref().map(str::trim) {
                    None | Some("") => Ok(fast),
                    Some(v) => v.parse::<i64>().map_err(|_| ()),
                };
                let f = DebugTiming::FAST;
                DebugTiming::custom(
                    num(&self.t_window, f.window_min)?,
                    num(&self.t_renew_window, f.renew_window_min)?,
                    num(&self.t_renewals, f.renewals_max)?,
                )
                .map(Some)
                .ok_or(())
            }
            _ => Err(()),
        }
    }

    /// The choice to show again (`normal` unless one was posted).
    fn choice(&self) -> &str {
        match self.timing.as_deref().map(str::trim) {
            Some("fast") => "fast",
            Some("custom") => "custom",
            _ => "normal",
        }
    }
}

/// The 403 for an admin who is impersonating someone.
fn impersonating(lang: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Html(crate::i18n::translate(lang, "dbg-impersonating", None)),
    )
        .into_response()
}

/// `GET /dashboard/admin/debug-booking`.
pub(super) async fn page(
    State(state): State<Arc<AppState>>,
    admin: crate::auth::AdminUser,
    viewer: crate::auth::OptionalAuthUser,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    if viewer.impersonation.is_some() {
        return impersonating(admin.lang);
    }
    let kept = Kept {
        et: query.et.clone(),
        ..Kept::default()
    };
    let client_ip = client_ip_for_rate_limit(&headers);
    render(
        &state,
        &admin,
        kept,
        query.cancelled,
        StatusCode::OK,
        &client_ip,
    )
    .await
}

/// The ladder step this guest books at now, as the booking path prices
/// them: the admin's chosen address, the typed Telegram username as the
/// booking form reads it for this type, this client's network. `None` for
/// a type not priced by the ladder (or while payments are off).
async fn guest_step(
    state: &AppState,
    et_id: &str,
    email: &str,
    telegram: Option<&str>,
    client_ip: &str,
    lang: &str,
) -> Option<(crate::ton::ladder::Step, bool)> {
    let telegram = super::telegram::resolve_guest_telegram(&state.pool, et_id, telegram, lang)
        .await
        .ok()
        .flatten();
    ton_pay::invoice::guest_ladder_step(
        state,
        et_id,
        email,
        telegram.as_deref(),
        client_ip,
        Utc::now(),
    )
    .await
}

async fn render(
    state: &AppState,
    admin: &crate::auth::AdminUser,
    kept: Kept,
    cancelled: Option<u32>,
    status: StatusCode,
    client_ip: &str,
) -> Response {
    let lang = admin.lang;
    let user = &admin.user;
    let tz = admin_tz(user);
    let types = own_types(&state.pool, &user.id).await;
    let selected = chosen(&types, kept.et.as_deref());
    let telegram_on = crate::telegram::enabled();
    let own_chat = crate::telegram::store::user_chat(&state.pool, &user.id).await;
    let prefill = prefill_telegram(state, &user.id).await;
    let emails = own_emails(user);
    let email = kept
        .email
        .clone()
        .filter(|e| emails.contains(e))
        .unwrap_or_else(|| {
            user.booking_email
                .clone()
                .filter(|b| emails.contains(b))
                .unwrap_or_else(|| user.email.clone())
        });
    // What was posted, else the linked chat's username, else the setting.
    let telegram = kept
        .telegram
        .clone()
        .or_else(|| prefill.as_ref().map(|(u, _)| u.clone()));
    let telegram_from = match (&kept.telegram, &prefill) {
        (None, Some((_, TelegramFrom::OwnChat))) => Some("chat"),
        (None, Some((_, TelegramFrom::Setting))) => Some("setting"),
        _ => None,
    };
    let own_chat_ok = telegram_on && own_chat.is_some();
    let guest_lang = kept
        .guest_lang
        .as_deref()
        .and_then(crate::i18n::supported_code)
        .unwrap_or_else(|| default_guest_lang(admin));
    let guest_langs: Vec<minijinja::Value> = crate::i18n::supported_with_labels()
        .map(|(code, label)| {
            context! {
                code => code,
                label => label,
                selected => code == guest_lang,
            }
        })
        .collect();
    let fast = crate::ton::deadlines::DebugTiming::FAST;
    let (selected_ctx, slots) = match selected {
        Some(t) => {
            let slots = next_slots(state, &user.id, &t.id, tz).await;
            let tg_mode = super::telegram::form_mode(&state.pool, &t.id).await;
            // The price for this guest: a free step this address already
            // used is the first price (the booking is quoted the same way).
            let guest_price =
                guest_step(state, &t.id, &email, telegram.as_deref(), client_ip, lang)
                    .await
                    .map(|(step, free_used)| {
                        context! {
                            now_free => step.is_free(),
                            now_ton => step.nano().map(crate::ton::ladder::ton_text),
                            free_used => free_used,
                        }
                    });
            (
                Some(context! {
                    slug => t.slug,
                    title => t.title,
                    private => t.visibility != "public",
                    priced => t.priced,
                    price => ton_pay::price_badge(state, &t.id).await,
                    guest_price => guest_price,
                    telegram_mode => tg_mode,
                    telegram_field => tg_mode != mode::OFF,
                    telegram_required => tg_mode == mode::REQUIRED,
                }),
                slots,
            )
        }
        None => (None, Vec::new()),
    };
    let types_ctx: Vec<minijinja::Value> = types
        .iter()
        .map(|t| {
            context! {
                slug => t.slug,
                title => t.title,
                private => t.visibility != "public",
                priced => t.priced,
                selected => selected.is_some_and(|s| s.slug == t.slug),
            }
        })
        .collect();
    let slots_ctx: Vec<minijinja::Value> = slots
        .iter()
        .map(|(value, label)| {
            context! {
                value => value,
                label => label,
                selected => kept.slot.as_deref() == Some(value.as_str()),
            }
        })
        .collect();
    let own_chat_label = own_chat.as_ref().map(|(u, _)| match u {
        Some(u) => format!("@{u}"),
        None => crate::i18n::translate(lang, "dbg-own-chat-no-username", None),
    });
    let tmpl = match state.templates.get_template("ilia/debug_booking.html") {
        Ok(t) => t,
        Err(e) => return internal_error_html("template render", &e).into_response(),
    };
    let body = tmpl
        .render(context! {
            sidebar => admin_sidebar_context(user, "admin_debug_booking"),
            lang => lang,
            impersonating => false,
            impersonating_name => "",
            path => PATH,
            cancel_path => CANCEL_PATH,
            error => kept.error,
            warning => kept.warning,
            cancelled => cancelled,
            types => types_ctx,
            selected => selected_ctx,
            slots => slots_ctx,
            tz => tz.name(),
            name => kept.name.unwrap_or_else(|| user.name.clone()),
            emails => emails,
            email => email,
            telegram => telegram,
            telegram_from => telegram_from,
            notes => kept.notes.unwrap_or_else(|| crate::i18n::translate(lang, "dbg-notes-default", None)),
            telegram_on => telegram_on,
            own_chat_linked => own_chat.is_some(),
            // Telegram is on and the admin has no linked chat: the notice at
            // the top of the page.
            own_chat_missing => telegram_on && own_chat.is_none(),
            telegram_settings_path => "/dashboard/settings/telegram",
            guest_langs => guest_langs,
            guest_lang => guest_lang,
            own_chat_label => own_chat_label,
            // Ticked by default whenever it can be (the owner's choice).
            own_chat_checked => own_chat_ok && kept.own_chat.unwrap_or(true),
            timing => kept.timing.choice(),
            t_window => kept.timing.t_window.clone().filter(|v| !v.trim().is_empty()),
            t_renew_window => kept.timing.t_renew_window.clone().filter(|v| !v.trim().is_empty()),
            t_renewals => kept.timing.t_renewals.clone().filter(|v| !v.trim().is_empty()),
            fast => context! {
                window => fast.window_min,
                renew => fast.renew_window_min,
                renewals => fast.renewals_max,
                grace => fast.grace_min,
                margin => fast.margin_min,
            },
            timing_bounds => context! {
                window_min => crate::ton::deadlines::DEBUG_WINDOW_MIN,
                window_max => crate::ton::deadlines::DEBUG_WINDOW_MAX,
                renew_max => crate::ton::deadlines::DEBUG_RENEW_WINDOW_MAX,
                renewals_max => crate::ton::deadlines::DEBUG_RENEWALS_MAX,
            },
            has_username => user.username.is_some(),
            captcha_on => state.captcha_config.read().await.is_some(),
            ton_on => state.ton.get().is_some(),
            bookings => listed(&state.pool, &user.id, tz, lang).await,
            standing => standing(&state.pool, &user.id).await.len(),
        })
        .unwrap_or_else(|e| internal_error_body("template render", &e));
    (status, Html(body)).into_response()
}

// --- booking ----------------------------------------------------------------

#[derive(Deserialize)]
pub(super) struct DebugForm {
    _csrf: Option<String>,
    #[serde(default)]
    et: String,
    #[serde(default)]
    slot: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    telegram: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    own_chat: Option<String>,
    #[serde(default)]
    guest_lang: Option<String>,
    #[serde(flatten)]
    timing: TimingForm,
}

impl DebugForm {
    fn own_chat(&self) -> bool {
        self.own_chat.as_deref().is_some_and(|v| !v.is_empty())
    }

    fn kept(&self, error: String) -> Kept {
        Kept {
            error: Some(error),
            warning: None,
            et: Some(self.et.clone()),
            slot: Some(self.slot.clone()),
            name: Some(self.name.clone()),
            email: Some(self.email.clone()),
            telegram: self.telegram.clone(),
            notes: self.notes.clone(),
            own_chat: Some(self.own_chat()),
            timing: self.timing.clone(),
            guest_lang: self.guest_lang.clone(),
        }
    }
}

/// `YYYY-MM-DDTHH:MM` as the form's `date` and `time`.
pub(crate) fn split_slot(slot: &str) -> Option<(String, String)> {
    let (date, time) = slot.split_once('T')?;
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    NaiveTime::parse_from_str(time, "%H:%M").ok()?;
    Some((date.to_string(), time.to_string()))
}

/// `POST /dashboard/admin/debug-booking`: book through the public path.
pub(super) async fn book(
    State(state): State<Arc<AppState>>,
    admin: crate::auth::AdminUser,
    viewer: crate::auth::OptionalAuthUser,
    headers: HeaderMap,
    Form(form): Form<DebugForm>,
) -> Response {
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    if viewer.impersonation.is_some() {
        return impersonating(admin.lang);
    }
    let lang = admin.lang;
    let user = admin.user.clone();
    let client_ip = client_ip_for_rate_limit(&headers);
    let refuse = |key: &str| form.kept(crate::i18n::translate(lang, key, None));

    let Some(username) = user.username.clone() else {
        let kept = refuse("dbg-err-username");
        return render(
            &state,
            &admin,
            kept,
            None,
            StatusCode::BAD_REQUEST,
            &client_ip,
        )
        .await;
    };
    let types = own_types(&state.pool, &user.id).await;
    let Some(et) = types.iter().find(|t| t.slug == form.et) else {
        let kept = refuse("dbg-err-type");
        return render(
            &state,
            &admin,
            kept,
            None,
            StatusCode::BAD_REQUEST,
            &client_ip,
        )
        .await;
    };
    // Only the admin's own addresses: this page never books as someone else.
    let Some(email) = own_emails(&user)
        .into_iter()
        .find(|e| e.eq_ignore_ascii_case(form.email.trim()))
    else {
        let kept = refuse("dbg-err-email");
        return render(
            &state,
            &admin,
            kept,
            None,
            StatusCode::BAD_REQUEST,
            &client_ip,
        )
        .await;
    };
    let Some((date, time)) = split_slot(&form.slot) else {
        let kept = refuse("dbg-err-slot");
        return render(
            &state,
            &admin,
            kept,
            None,
            StatusCode::BAD_REQUEST,
            &client_ip,
        )
        .await;
    };
    let own_chat = form.own_chat();
    if own_chat
        && (!crate::telegram::enabled()
            || crate::telegram::store::user_chat(&state.pool, &user.id)
                .await
                .is_none())
    {
        let kept = refuse("dbg-err-own-chat");
        return render(
            &state,
            &admin,
            kept,
            None,
            StatusCode::BAD_REQUEST,
            &client_ip,
        )
        .await;
    }
    // The test booking's own invoice timing (only priced types have one).
    let Ok(timing) = form.timing.timing() else {
        let kept = refuse("dbg-err-timing");
        return render(
            &state,
            &admin,
            kept,
            None,
            StatusCode::BAD_REQUEST,
            &client_ip,
        )
        .await;
    };

    // The language the test guest books in (the default: the admin's).
    let Ok(guest_lang) = posted_guest_lang(form.guest_lang.as_deref()) else {
        let kept = refuse("dbg-err-lang");
        return render(
            &state,
            &admin,
            kept,
            None,
            StatusCode::BAD_REQUEST,
            &client_ip,
        )
        .await;
    };
    let guest_lang = guest_lang.unwrap_or_else(|| default_guest_lang(&admin));

    // A private or internal type: a one-time invite for the admin's own
    // address, as the owner would make one in the dashboard.
    let invite = if et.visibility != "public" {
        match one_time_invite(&state.pool, &et.id, &user.id, &user.name, &email).await {
            Ok(invite) => Some(invite),
            Err(e) => return internal_error_response("debug booking: invite", &e),
        }
    } else {
        None
    };

    // The guest form, as the booking page would post it. The quote is the
    // price now for this guest (the owner's demand, and the first price
    // when this address, Telegram username or network already used a free
    // step), as the page showed it; the booking transaction counts it
    // again, as for every guest.
    let quoted = guest_step(
        &state,
        &et.id,
        &email,
        form.telegram.as_deref(),
        &client_ip,
        lang,
    )
    .await
    .map(|(step, _)| step.quote_value());
    let mut fields = serde_json::Map::new();
    let mut put = |k: &str, v: Option<String>| {
        if let Some(v) = v {
            fields.insert(k.to_string(), serde_json::Value::String(v));
        }
    };
    put("_csrf", form._csrf.clone());
    put("date", Some(date));
    put("time", Some(time.clone()));
    put("name", Some(form.name.trim().to_string()));
    put("email", Some(email.clone()));
    put("notes", form.notes.clone().filter(|n| !n.trim().is_empty()));
    put("tz", Some(admin_tz(&user).name().to_string()));
    put(
        "telegram",
        form.telegram.clone().filter(|t| !t.trim().is_empty()),
    );
    put(
        "invite_token",
        invite.as_ref().map(|(_, token)| token.clone()),
    );
    put("quoted_nano", quoted);
    let guest_form: BookForm = match serde_json::from_value(serde_json::Value::Object(fields)) {
        Ok(f) => f,
        Err(e) => return internal_error_response("debug booking: form", &e),
    };

    let since = (Utc::now() - Duration::seconds(1))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let call = handle_booking_for_user(
        State(state.clone()),
        guest_headers(&headers, guest_lang),
        Path((username, et.slug.clone())),
        Query(EmbedParams::default()),
        Form(guest_form),
    );
    // The timing reaches the invoice through a task-local around this call
    // only: nothing a guest posts can set it.
    let resp = match timing {
        Some(t) => ton_pay::invoice::with_debug_timing(t, call).await,
        None => call.await,
    };

    // Only the status and the redirect are read: a `&Response` held across
    // the queries would make this handler's future not `Send`.
    let status = resp.status();
    let location = resp
        .headers()
        .get(axum::http::header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let booking_id = created_booking(
        &state.pool,
        status,
        location.as_deref(),
        &et.id,
        &email,
        &since,
    )
    .await;
    let Some(booking_id) = booking_id else {
        if let Some((invite_id, _)) = &invite {
            let _ = sqlx::query("DELETE FROM booking_invites WHERE id = ? AND used_count = 0")
                .bind(invite_id)
                .execute(&state.pool)
                .await;
        }
        tracing::info!(
            target: "calrs::audit",
            actor = %user.email,
            event_type = %et.slug,
            slot = %form.slot,
            status = status.as_u16(),
            guest_lang,
            timing = timing.map(|t| t.text()).as_deref().unwrap_or("site"),
            "debug booking refused by the booking path"
        );
        return resp;
    };

    // Routed first, so the mark records what was done, not what was asked.
    let routed = own_chat
        && route_to_own_chat(&state.pool, &booking_id, &user.id, form.telegram.as_deref()).await;
    let marked = sqlx::query(
        "INSERT OR IGNORE INTO debug_bookings (booking_id, user_id, telegram_own_chat) VALUES (?, ?, ?)",
    )
    .bind(&booking_id)
    .bind(&user.id)
    .bind(i64::from(routed))
    .execute(&state.pool)
    .await;
    if let Err(e) = marked {
        tracing::error!(error = %e, booking_id = %booking_id, "debug booking: could not mark the booking as a test");
    }
    if routed {
        // The welcome a guest gets on linking, or what to pay when the
        // invoice is open (the worker turns it into `PaymentDue`).
        crate::telegram::notify(
            &state.pool,
            &booking_id,
            crate::telegram::TelegramEvent::GuestLinked,
        );
    }
    tracing::info!(
        target: "calrs::audit",
        actor = %user.email,
        booking_id = %booking_id,
        event_type = %et.slug,
        slot = %form.slot,
        telegram_own_chat_asked = own_chat,
        telegram_own_chat = routed,
        guest_lang,
        timing = timing.map(|t| t.text()).as_deref().unwrap_or("site"),
        "debug booking created"
    );
    if own_chat && !routed {
        // Asked for and not done (the chat was unlinked, the bot stopped,
        // the link expired meanwhile): say so here instead of the payment
        // page, where the admin would wait for a message that never comes.
        // The booking stands; the list links its payment page.
        let kept = Kept {
            warning: Some(crate::i18n::translate(lang, "dbg-own-chat-failed", None)),
            et: Some(et.slug.clone()),
            guest_lang: Some(guest_lang.to_string()),
            ..Kept::default()
        };
        return render(&state, &admin, kept, None, StatusCode::OK, &client_ip).await;
    }
    resp
}

/// A single-use invite for `email`, valid [`INVITE_MINUTES`]: (id, token).
async fn one_time_invite(
    pool: &SqlitePool,
    et_id: &str,
    user_id: &str,
    name: &str,
    email: &str,
) -> Result<(String, String), sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = (Utc::now() + Duration::minutes(INVITE_MINUTES))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    sqlx::query(
        "INSERT INTO booking_invites (id, event_type_id, token, guest_name, guest_email, expires_at, max_uses, created_by_user_id)
         VALUES (?, ?, ?, ?, ?, ?, 1, ?)",
    )
    .bind(&id)
    .bind(et_id)
    .bind(&token)
    .bind(name)
    .bind(email)
    .bind(&expires_at)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok((id, token))
}

/// The booking the public path just made, if it made one: a priced booking
/// redirects to its payment page (found by the pay token), a free one
/// answers its confirmation page (the newest booking of this type and
/// address since the call, not yet marked).
async fn created_booking(
    pool: &SqlitePool,
    status: StatusCode,
    location: Option<&str>,
    et_id: &str,
    email: &str,
    since: &str,
) -> Option<String> {
    if status == StatusCode::SEE_OTHER {
        let token = location?.strip_prefix("/booking/pay/")?.to_string();
        return sqlx::query_scalar(
            "SELECT i.booking_id FROM ton_invoices i JOIN bookings b ON b.id = i.booking_id
              WHERE i.pay_token = ? AND b.event_type_id = ?",
        )
        .bind(token)
        .bind(et_id)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
    }
    if status != StatusCode::OK {
        return None;
    }
    sqlx::query_scalar(
        "SELECT b.id FROM bookings b
          WHERE b.event_type_id = ? AND b.guest_email = ? AND b.created_at >= ?
            AND b.status IN ('pending', 'confirmed')
            AND NOT EXISTS (SELECT 1 FROM debug_bookings d WHERE d.booking_id = b.id)
          ORDER BY b.created_at DESC, b.rowid DESC
          LIMIT 1",
    )
    .bind(et_id)
    .bind(email)
    .bind(since)
    .fetch_optional(pool)
    .await
    .unwrap_or(None)
}

/// Link the booking's guest Telegram to the admin's own linked chat, as if
/// the admin had opened the guest's deep link from it: the chat becomes the
/// one that used the token (`token_chat_id`), so no other chat can take the
/// link. A type that offers no Telegram field has no link row: one is made
/// under a fresh token nobody sees (a priced booking's payment page re-keys
/// it to its own derived link, `telegram::offered_priced_link`). `true` when
/// the booking's guest messages now go to the admin's chat.
async fn route_to_own_chat(
    pool: &SqlitePool,
    booking_id: &str,
    user_id: &str,
    entered: Option<&str>,
) -> bool {
    if !crate::telegram::enabled() {
        return false;
    }
    let has_row: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM telegram_guest_links WHERE booking_id = ?)",
    )
    .bind(booking_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false);
    if !has_row {
        let Some((_, end)) = crate::telegram::store::booking_mode_and_end(pool, booking_id).await
        else {
            return false;
        };
        let entered = entered
            .map(|u| u.trim().trim_start_matches('@'))
            .filter(|u| crate::telegram::username::valid(u));
        let (_, hash) = crate::telegram::tokens::new_token(crate::telegram::tokens::Kind::Booking);
        if !crate::telegram::store::create_guest_link(pool, booking_id, entered, &hash, end).await {
            return false;
        }
    }
    sqlx::query(
        "UPDATE telegram_guest_links
            SET chat_id = (SELECT c.chat_id FROM telegram_user_chats c WHERE c.user_id = ?2),
                token_chat_id = (SELECT c.chat_id FROM telegram_user_chats c WHERE c.user_id = ?2),
                telegram_username = (SELECT c.telegram_username FROM telegram_user_chats c WHERE c.user_id = ?2),
                linked_at = datetime('now')
          WHERE booking_id = ?1
            AND chat_id IS NULL AND token_chat_id IS NULL
            AND link_expires_at > datetime('now')
            AND EXISTS (SELECT 1 FROM telegram_user_chats c WHERE c.user_id = ?2)
            AND EXISTS (SELECT 1 FROM bookings b WHERE b.id = ?1 AND b.status IN ('pending', 'confirmed'))",
    )
    .bind(booking_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, booking_id, "debug booking: could not link the own chat");
        false
    })
}

// --- cancel all -------------------------------------------------------------

#[derive(Deserialize)]
pub(super) struct CancelAllForm {
    _csrf: Option<String>,
}

/// `POST /dashboard/admin/debug-booking/cancel-all`: every standing test
/// booking of this admin goes through the dashboard's cancel (a pending one
/// is declined), with its invoice closed and the usual notices.
pub(super) async fn cancel_all(
    State(state): State<Arc<AppState>>,
    admin: crate::auth::AdminUser,
    viewer: crate::auth::OptionalAuthUser,
    headers: HeaderMap,
    Form(form): Form<CancelAllForm>,
) -> Response {
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    if viewer.impersonation.is_some() {
        return impersonating(admin.lang);
    }
    let ids = standing(&state.pool, &admin.user.id).await;
    let reason = crate::i18n::translate(admin.lang, "dbg-cancel-reason", None);
    for id in &ids {
        let host = crate::auth::AuthUser {
            user: admin.user.clone(),
            impersonation: None,
            lang: admin.lang,
        };
        let _ = cancel_booking(
            State(state.clone()),
            host,
            headers.clone(),
            Path(id.clone()),
            Form(CancelForm {
                _csrf: form._csrf.clone(),
                reason: Some(reason.clone()),
            }),
        )
        .await;
    }
    let left = standing(&state.pool, &admin.user.id).await.len();
    let cancelled = ids.len().saturating_sub(left);
    tracing::info!(
        target: "calrs::audit",
        actor = %admin.user.email,
        cancelled,
        left,
        "debug bookings cancelled"
    );
    Redirect::to(&format!("{PATH}?cancelled={cancelled}")).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slot_is_a_date_and_a_time() {
        assert_eq!(
            split_slot("2026-11-16T10:00"),
            Some(("2026-11-16".into(), "10:00".into()))
        );
        for bad in [
            "2026-11-16 10:00",
            "2026-13-01T10:00",
            "2026-11-16T25:00",
            "",
            "T",
        ] {
            assert_eq!(split_slot(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn the_guest_language_reaches_the_booking_as_the_switchers_pick() {
        use axum::http::header::COOKIE;
        let mut h = HeaderMap::new();
        h.append(
            COOKIE,
            "calrs_lang=it; __Host-calrs_csrf=abc".parse().unwrap(),
        );
        h.append(
            COOKIE,
            "__Host-calrs_session=s; calrs_lang=es".parse().unwrap(),
        );
        h.insert("accept-language", "en-US".parse().unwrap());
        h.insert(crate::i18n::GERMAN_FORMAL_HEADER, "1".parse().unwrap());
        let out = guest_headers(&h, "de");
        let cookies: Vec<_> = out.get_all(COOKIE).iter().collect();
        assert_eq!(cookies.len(), 1);
        assert_eq!(
            cookies[0].to_str().unwrap(),
            "__Host-calrs_csrf=abc; __Host-calrs_session=s; calrs_lang=de"
        );
        // Exactly the language chosen: no formal register added to `de`.
        assert_eq!(crate::i18n::chosen_from_headers(&out), Some("de"));
        assert_eq!(crate::i18n::detect_from_headers(&out), "de");
        assert_eq!(crate::i18n::page_lang(Some("ru"), &out), "de");
        assert_eq!(out.get("accept-language").unwrap(), "en-US");
        // The original is untouched.
        assert_eq!(crate::i18n::chosen_from_headers(&h), Some("it"));
        // No cookie at all: just the pick.
        let out = guest_headers(&HeaderMap::new(), "de-formal");
        assert_eq!(out.get(COOKIE).unwrap(), "calrs_lang=de-formal");
        assert_eq!(crate::i18n::detect_from_headers(&out), "de-formal");
    }

    #[test]
    fn a_posted_guest_language_is_a_shipped_one_or_refused() {
        assert_eq!(posted_guest_lang(None), Ok(None));
        assert_eq!(posted_guest_lang(Some("  ")), Ok(None));
        assert_eq!(posted_guest_lang(Some("ru")), Ok(Some("ru")));
        assert_eq!(posted_guest_lang(Some(" fr ")), Ok(Some("fr")));
        assert_eq!(posted_guest_lang(Some("de-formal")), Ok(Some("de-formal")));
        for bad in ["xx", "fr-FR", "RU", "en; calrs_lang=ru"] {
            assert_eq!(posted_guest_lang(Some(bad)), Err(()), "{bad}");
        }
        assert_eq!(lang_label("ru"), Some("Русский"));
        assert_eq!(lang_label("xx"), None);
    }

    fn t(slug: &str, priced: bool) -> OwnType {
        OwnType {
            id: slug.into(),
            slug: slug.into(),
            title: slug.into(),
            visibility: "public".into(),
            priced,
        }
    }

    #[test]
    fn the_page_opens_on_a_test_type_then_a_priced_one() {
        let types = vec![t("15min", true), t("intro", false), t("test-1usd", true)];
        assert_eq!(chosen(&types, None).unwrap().slug, "test-1usd");
        assert_eq!(chosen(&types, Some("intro")).unwrap().slug, "intro");
        assert_eq!(chosen(&types, Some("nope")).unwrap().slug, "test-1usd");
        let types = vec![t("alpha", false), t("paid", true)];
        assert_eq!(chosen(&types, None).unwrap().slug, "paid");
        let types = vec![t("alpha", false)];
        assert_eq!(chosen(&types, None).unwrap().slug, "alpha");
        assert!(chosen(&[], None).is_none());
    }
}
