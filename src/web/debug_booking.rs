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
//!   so nobody can book "as" someone else from here).
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
//! - "Telegram: deliver guest messages to my own linked Telegram chat": the
//!   booking's guest link row is linked to the chat the admin linked in
//!   Profile & Settings → Telegram (`telegram_user_chats`), as if the admin
//!   had opened the guest's deep link from it, and the usual welcome (or
//!   `PaymentDue` for an unpaid invoice) is queued. The chat id stays in SQL
//!   and is never read into a page or a log.
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
/// The deployment variable (and the `ton_settings` key).
pub(crate) const ENV: &str = crate::ton::settings::DEBUG_BOOKING;

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

/// The admin's test bookings, newest first, for the page.
async fn listed(pool: &SqlitePool, user_id: &str, tz: Tz, lang: &str) -> Vec<minijinja::Value> {
    #[allow(clippy::type_complexity)]
    let rows: Vec<(
        String,
        String,
        String,
        String,
        i64,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT b.id, et.title, b.start_at, b.status, d.telegram_own_chat, i.status, i.pay_token
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
        .map(
            |(id, title, start_at, status, own_chat, invoice, pay_token)| {
                let when = crate::booking_time::local(&start_at, Tz::UTC, tz)
                    .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or(start_at);
                let standing = matches!(status.as_str(), "pending" | "confirmed");
                context! {
                    id => id,
                    title => title,
                    when => when,
                    status_label => status_label(lang, &status),
                    status => status,
                    invoice_label => invoice.as_deref().map(|i| invoice_label(lang, i)),
                    invoice => invoice,
                    // The payment page while the booking stands (the admin's own).
                    pay_url => pay_token.filter(|_| standing).map(|t| format!("/booking/pay/{t}")),
                    own_chat => own_chat != 0,
                }
            },
        )
        .collect()
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
    own_chat: bool,
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
    let own_username = own_chat.as_ref().and_then(|(u, _)| u.clone());
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
    let telegram = kept.telegram.clone().or_else(|| own_username.clone());
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
            notes => kept.notes.unwrap_or_else(|| crate::i18n::translate(lang, "dbg-notes-default", None)),
            telegram_on => telegram_on,
            own_chat_linked => own_chat.is_some(),
            own_chat_label => own_chat_label,
            own_chat_checked => kept.own_chat,
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
            own_chat: self.own_chat(),
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
    let resp = handle_booking_for_user(
        State(state.clone()),
        headers.clone(),
        Path((username, et.slug.clone())),
        Query(EmbedParams::default()),
        Form(guest_form),
    )
    .await;

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
