//! ilia fork: the passkey routes and the passkey half of the MFA pages
//! (docs/passkeys-plan.md §4, §5).
//!
//! - `POST /auth/mfa/passkey`: the second step of a login, with a key;
//! - `POST /auth/mfa/passkey/enroll`: a key as the first factor;
//! - `GET /dashboard/settings/mfa/confirm`: "confirm it's you" before a
//!   change (password and a key or a code), which posts to
//!   `/dashboard/settings/mfa` or, for the policy, `/dashboard/admin/mfa`;
//! - `POST /dashboard/settings/mfa/passkeys`: registers a key after that
//!   confirmation;
//! - `GET /static/webauthn.js`: the browser side.
//!
//! Every ceremony route answers 404 while passkeys are off (no usable
//! `CALRS_BASE_URL`). The confirm page works then too, with codes only, so
//! a user whose keys no longer work can still remove them.

use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    extract::{Form, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use axum_extra::extract::CookieJar;
use chrono::{TimeZone, Utc};
use serde::Deserialize;
use sqlx::SqliteConnection;

use super::store::{self, Binding, Credential, Kind, NotAdded};
use super::{
    creation_options, request_options, AssertionFields, Descriptor, RegistrationFields, MAX_KEYS,
};
use crate::{
    auth,
    mfa::{self, LoginResult},
    models::{Session, User},
    web::{self, AppState},
};

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/mfa/passkey", post(login_passkey))
        .route("/auth/mfa/passkey/enroll", post(enroll_passkey))
        .route("/dashboard/settings/mfa/confirm", get(confirm_page))
        .route("/dashboard/settings/mfa/passkeys", post(register_finish))
        .route("/static/webauthn.js", get(script))
}

fn descriptors(keys: &[Credential]) -> Vec<Descriptor<'_>> {
    keys.iter()
        .map(|key| Descriptor {
            credential_id: &key.credential_id,
            transports: &key.transports,
        })
        .collect()
}

fn json_value(value: &serde_json::Value) -> minijinja::Value {
    minijinja::Value::from_serialize(value)
}

/// The method a page shows: the one asked for when it is on offer, else
/// the passkey, else the authenticator code, else a recovery code.
fn pick_method(asked: &str, passkey: bool, totp: bool) -> &'static str {
    match asked {
        "passkey" if passkey => "passkey",
        "totp" if totp => "totp",
        "recovery" => "recovery",
        _ if passkey => "passkey",
        _ if totp => "totp",
        _ => "recovery",
    }
}

/// What removing a factor (`disable`: the authenticator app;
/// `passkey_delete`: the key `credential`) would leave the account. A key
/// works here only while passkeys are on and it was registered for this RP
/// ID; the others are listed and removable but cannot sign in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Removal {
    /// Another factor that works here remains, or there was none before
    /// and another stored one remains.
    Fine,
    /// Factors remain, but none that works here: only the recovery codes
    /// would sign in. Refused while MFA is required.
    LastUsable,
    /// No factor at all: MFA goes off for the account, with its recovery
    /// codes. Refused while MFA is required.
    LastFactor,
}

impl Removal {
    /// Refused while MFA is required.
    pub(crate) fn refused_when_required(self) -> bool {
        self != Removal::Fine
    }
}

pub(crate) fn removal(
    action: &str,
    credential: &str,
    has_totp: bool,
    keys: &[Credential],
    rp: Option<&super::Rp>,
) -> Removal {
    let (totp_after, gone) = match action {
        "disable" => (false, None),
        "passkey_delete" => (has_totp, Some(credential)),
        _ => return Removal::Fine,
    };
    let works = |key: &Credential| rp.is_some_and(|rp| key.rp_id == rp.id);
    let left: Vec<&Credential> = keys
        .iter()
        .filter(|k| Some(k.id.as_str()) != gone)
        .collect();
    let usable_before = usize::from(has_totp) + keys.iter().filter(|k| works(k)).count();
    let usable_after = usize::from(totp_after) + left.iter().filter(|k| works(k)).count();
    if !totp_after && left.is_empty() {
        Removal::LastFactor
    } else if usable_after == 0 && usable_before > 0 {
        Removal::LastUsable
    } else {
        Removal::Fine
    }
}

/// The passkey side of the MFA challenge page.
#[derive(Default)]
pub(crate) struct ChallengeView {
    /// This page is the passkey one (its options are set).
    pub passkey_view: bool,
    /// The passkey method is on offer (a link to it from the code views).
    pub passkey_available: bool,
    pub has_totp: bool,
    pub method: &'static str,
    pub options: Option<serde_json::Value>,
}

impl ChallengeView {
    pub(crate) fn context(&self) -> minijinja::Value {
        minijinja::context! {
            passkey_view => self.passkey_view,
            passkey_available => self.passkey_available,
            has_totp => self.has_totp,
            method => self.method,
            options => self.options.as_ref().map(json_value),
        }
    }
}

/// What the challenge page offers, and for the passkey view a new
/// ceremony bound to this challenge (the earlier one is dropped).
///
/// - login: the passkey when the user has a key for this RP ID, else the
///   authenticator code when TOTP is set up, else a recovery code;
/// - first factor (no factor yet): a passkey when passkeys are on, else the
///   authenticator app;
/// - TOTP added next to keys (`totp_add`): the authenticator app only.
pub(crate) async fn challenge_methods(
    state: &AppState,
    token: &str,
    challenge: &mfa::Challenge,
    asked: &str,
) -> Result<ChallengeView> {
    let token_hash = mfa::hash(token);
    let mut tx = state.pool.begin().await?;
    // Write first, as every MFA transaction does.
    sqlx::query("UPDATE mfa_challenges SET expires_at = expires_at WHERE token_hash = ?")
        .bind(&token_hash)
        .execute(&mut *tx)
        .await?;
    let user_id = &challenge.user_id;
    let has_totp = store::has_totp(&mut tx, user_id).await?;
    let has_factor = store::has_factor(&mut tx, user_id).await?;
    let mut view = ChallengeView {
        has_totp,
        ..Default::default()
    };
    let rp = state.passkeys.as_ref();
    match (challenge.purpose.as_str(), rp) {
        ("login", Some(rp)) => {
            let keys = store::usable(&mut tx, user_id, rp).await?;
            view.passkey_available = !keys.is_empty();
            view.method = pick_method(asked, view.passkey_available, has_totp);
            if view.method == "passkey" {
                let binding = Binding::Challenge(&token_hash);
                if let Some(c) = store::begin(&mut tx, user_id, Kind::Login, binding).await? {
                    view.options = Some(request_options(rp, &c, &descriptors(&keys)));
                    view.passkey_view = true;
                }
            }
        }
        ("login", None) => view.method = pick_method(asked, false, has_totp),
        ("enroll", Some(rp)) if !has_factor => {
            view.passkey_available = true;
            view.method = if asked == "totp" { "totp" } else { "passkey" };
            if view.method == "passkey" {
                let binding = Binding::Challenge(&token_hash);
                if let Some(c) = store::begin(&mut tx, user_id, Kind::Enroll, binding).await? {
                    view.options = Some(creation_options(
                        rp,
                        user_id,
                        &challenge.email,
                        &challenge.name,
                        &c,
                        &[],
                    ));
                    view.passkey_view = true;
                }
            }
        }
        _ => view.method = "totp",
    }
    tx.commit().await?;
    Ok(view)
}

/// One key as the settings page lists it.
#[derive(serde::Serialize)]
struct KeyRow {
    id: String,
    name: String,
    /// BE: a synced passkey; else a key bound to one device.
    synced: bool,
    added: String,
    last_used: Option<String>,
    /// Registered for another RP ID: listed and removable, not usable here.
    other_site: Option<String>,
}

fn local_date(ts: i64, tz: chrono_tz::Tz, lang: &str) -> String {
    match Utc.timestamp_opt(ts, 0).single() {
        Some(at) => crate::i18n::format_long_date(at.with_timezone(&tz).date_naive(), lang),
        None => String::new(),
    }
}

/// The MFA settings page's factors: keys, TOTP, recovery codes.
pub(crate) async fn settings_context(
    state: &AppState,
    user: &User,
    lang: &str,
) -> Result<minijinja::Value> {
    let mut conn = state.pool.acquire().await?;
    let keys = store::list(&mut conn, &user.id).await?;
    let has_totp = store::has_totp(&mut conn, &user.id).await?;
    let recovery_left = store::recovery_codes_left(&mut conn, &user.id).await?;
    let rp = state.passkeys.as_ref();
    let tz: chrono_tz::Tz = user.timezone.parse().unwrap_or(chrono_tz::UTC);
    let rows: Vec<KeyRow> = keys
        .iter()
        .map(|key| KeyRow {
            id: key.id.clone(),
            name: key.name.clone(),
            synced: key.backup_eligible,
            added: local_date(key.created_at, tz, lang),
            last_used: key.last_used_at.map(|ts| local_date(ts, tz, lang)),
            other_site: rp
                .filter(|rp| rp.id != key.rp_id)
                .map(|_| key.rp_id.clone()),
        })
        .collect();
    let has_factor = has_totp || !keys.is_empty();
    // The authenticator app may go while MFA is required only when another
    // factor that works here remains (the rule `manage` applies).
    let totp_removable = removal("disable", "", has_totp, &keys, rp) == Removal::Fine;
    Ok(minijinja::context! {
        passkeys_on => rp.is_some(),
        keys => rows,
        key_count => keys.len(),
        totp_removable => totp_removable,
        max_keys => MAX_KEYS,
        limit_reached => keys.len() as i64 >= MAX_KEYS,
        has_totp => has_totp,
        has_factor => has_factor,
        recovery_left => recovery_left,
    })
}

/// A `register` ceremony bound to `session_id` (just issued after a
/// confirmed `passkey_add`) and its creation options.
pub(crate) async fn register_options(
    conn: &mut SqliteConnection,
    state: &AppState,
    user: &User,
    session_id: &str,
) -> Result<serde_json::Value> {
    let rp = state.passkeys.as_ref().context("passkeys are off")?;
    let challenge = store::begin(
        &mut *conn,
        &user.id,
        Kind::Register,
        Binding::Session(session_id),
    )
    .await?
    .context("session for the register ceremony is gone")?;
    let keys = store::usable(conn, &user.id, rp).await?;
    Ok(creation_options(
        rp,
        &user.id,
        &user.email,
        &user.name,
        &challenge,
        &descriptors(&keys),
    ))
}

/// The add button a user chose on the settings page: `security-key` (a
/// USB or NFC key) or `device` (Touch ID, Face ID, Windows Hello, a
/// phone); anything else is no choice, and the page leads with the
/// security key.
fn add_kind(raw: &str) -> &'static str {
    match raw {
        "security-key" => "security-key",
        "device" => "device",
        _ => "",
    }
}

/// The page that registers a key from the settings, under the session the
/// confirmation issued, leading with the button `kind` names.
pub(crate) fn register_page(
    state: &AppState,
    lang: &str,
    session: &Session,
    options: serde_json::Value,
    kind: &str,
) -> Response {
    let mut response = mfa::render(
        state,
        lang,
        minijinja::context! {
            mode => "register",
            options => json_value(&options),
            kind => add_kind(kind),
        },
    );
    if let Ok(cookie) = mfa::session_cookie(session).parse() {
        response.headers_mut().append("Set-Cookie", cookie);
    }
    response
}

/// `POST /auth/mfa/passkey`: completes a login challenge with a key. A key
/// signature cannot be guessed, so this takes nothing from the account's
/// wrong-code allowance (someone holding the password could otherwise lock
/// the owner out of their key by failing codes on purpose); every refusal
/// is logged with the account and the address.
async fn login_passkey(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<AssertionFields>,
) -> Response {
    let Some(rp) = state.passkeys.clone() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if let Err(r) = web::verify_csrf_token(&headers, &form._csrf) {
        return r;
    }
    let lang = crate::i18n::detect_from_headers(&headers);
    let token = jar
        .get(mfa::CHALLENGE_COOKIE)
        .map(|c| c.value())
        .unwrap_or("");
    let ip = web::client_ip_for_rate_limit(&headers);
    let result: Result<Response> = async {
        let token_hash = mfa::hash(token);
        let mut tx = state.pool.begin().await?;
        sqlx::query("UPDATE mfa_challenges SET expires_at = expires_at WHERE token_hash = ?")
            .bind(&token_hash)
            .execute(&mut *tx)
            .await?;
        let Some(c) = mfa::challenge(&mut tx, token).await? else {
            return Ok(mfa::error(&state, lang));
        };
        if c.purpose != "login" {
            return Ok(mfa::error(&state, lang));
        }
        let binding = Binding::Challenge(&token_hash);
        match store::check_assertion(&mut tx, &rp, &c.user_id, &form, Kind::Login, binding).await? {
            Ok(credential) => {
                sqlx::query("DELETE FROM mfa_challenges WHERE token_hash = ?")
                    .bind(&token_hash)
                    .execute(&mut *tx)
                    .await?;
                mfa::clear_attempts(&mut tx, &c.user_id).await?;
                let session = mfa::verified_session(&mut tx, &c.user_id).await?;
                tx.commit().await?;
                tracing::info!(user_id = %c.user_id, credential = %credential, "MFA completed with a passkey");
                Ok(mfa::login_response(LoginResult::Session(session)))
            }
            Err(refusal) => {
                // The ceremony stays used up; the page offers a new one.
                tx.commit().await?;
                tracing::warn!(user_id = %c.user_id, ip = %ip, reason = refusal.code(), "passkey sign-in refused");
                mfa::challenge_view(&state, lang, token, "passkey", "passkey").await
            }
        }
    }
    .await;
    result.unwrap_or_else(|e| web::internal_error_response("passkey sign-in", &e))
}

/// The name typed for a new key: `Ok(None)` when left empty (a default is
/// given), `Err` when it is not a valid name.
fn typed_name(raw: &str) -> std::result::Result<Option<String>, ()> {
    if raw.trim().is_empty() {
        return Ok(None);
    }
    super::clean_name(raw).map(Some).ok_or(())
}

async fn name_or_default(
    conn: &mut SqliteConnection,
    user_id: &str,
    lang: &str,
    typed: Option<String>,
    synced: bool,
) -> Result<String> {
    match typed {
        Some(name) => Ok(name),
        None => store::default_name(conn, user_id, lang, synced).await,
    }
}

/// `POST /auth/mfa/passkey/enroll`: a key as the user's first factor. Like
/// a first TOTP enrolment it brings recovery codes, shown once, revokes
/// every other session and pending challenge, and signs in.
async fn enroll_passkey(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<RegistrationFields>,
) -> Response {
    let Some(rp) = state.passkeys.clone() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if let Err(r) = web::verify_csrf_token(&headers, &form._csrf) {
        return r;
    }
    let lang = crate::i18n::detect_from_headers(&headers);
    let token = jar
        .get(mfa::CHALLENGE_COOKIE)
        .map(|c| c.value())
        .unwrap_or("");
    let ip = web::client_ip_for_rate_limit(&headers);
    let result: Result<Response> = async {
        let Ok(typed) = typed_name(&form.name) else {
            return mfa::challenge_view(&state, lang, token, "code", "passkey").await;
        };
        let token_hash = mfa::hash(token);
        let mut tx = state.pool.begin().await?;
        sqlx::query("UPDATE mfa_challenges SET expires_at = expires_at WHERE token_hash = ?")
            .bind(&token_hash)
            .execute(&mut *tx)
            .await?;
        let Some(c) = mfa::challenge(&mut tx, token).await? else {
            return Ok(mfa::error(&state, lang));
        };
        // A first factor only: someone may have finished the other way
        // (the authenticator app) in another tab.
        if c.purpose != "enroll" || store::has_factor(&mut tx, &c.user_id).await? {
            return Ok(mfa::error(&state, lang));
        }
        let binding = Binding::Challenge(&token_hash);
        let checked =
            store::check_registration(&mut tx, &rp, &c.user_id, &form, Kind::Enroll, binding)
                .await?;
        let registration = match checked {
            Ok(r) => r,
            Err(refusal) => {
                tx.commit().await?;
                tracing::warn!(user_id = %c.user_id, ip = %ip, reason = refusal.code(), "passkey enrolment refused");
                return mfa::challenge_view(&state, lang, token, "passkey", "passkey").await;
            }
        };
        let name = name_or_default(
            &mut tx,
            &c.user_id,
            lang,
            typed,
            registration.backup_eligible,
        )
        .await?;
        let added = store::add(&mut tx, &c.user_id, &rp, &registration, &form.transports, &name)
            .await?;
        let credential = match added {
            Ok(id) => id,
            Err(why) => {
                tx.commit().await?;
                tracing::warn!(user_id = %c.user_id, ip = %ip, reason = ?why, "passkey enrolment refused");
                return mfa::challenge_view(&state, lang, token, "passkey", "passkey").await;
            }
        };
        let codes = mfa::new_recovery_codes(&mut tx, &c.user_id).await?;
        mfa::revoke_access(&mut tx, &c.user_id).await?;
        mfa::clear_attempts(&mut tx, &c.user_id).await?;
        let session = mfa::verified_session(&mut tx, &c.user_id).await?;
        tx.commit().await?;
        tracing::info!(user_id = %c.user_id, credential = %credential, "MFA enrolled with a passkey");
        Ok(mfa::completion(&state, lang, session, codes))
    }
    .await;
    result.unwrap_or_else(|e| web::internal_error_response("passkey enrolment", &e))
}

#[derive(Deserialize, Default)]
struct ConfirmQuery {
    #[serde(default)]
    action: String,
    #[serde(default)]
    credential: String,
    #[serde(default)]
    policy: String,
    #[serde(default)]
    method: String,
    #[serde(default)]
    kind: String,
}

/// `GET /dashboard/settings/mfa/confirm`: "confirm it's you" before a
/// change. Shows what is about to happen, asks for the password and a key
/// (a `confirm` ceremony bound to this session) or a code, and posts to the
/// handler that makes the change and checks both.
///
/// The policy is the real admin's change, as `/dashboard/admin` and
/// `policy_save` treat it (both `AdminUser`), so an impersonation does not
/// stand in its way; every other action is about the visitor's own factors
/// and is refused while impersonating.
async fn confirm_page(
    State(state): State<Arc<AppState>>,
    user: auth::AuthUser,
    headers: HeaderMap,
    jar: CookieJar,
    Query(query): Query<ConfirmQuery>,
) -> Response {
    let session_id = jar
        .get(mfa::SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .unwrap_or_default();
    let (user, lang) = if query.action == "policy" {
        match auth::validate_session(&state.pool, &session_id).await {
            Some(real) => {
                let lang = crate::i18n::resolve(real.language.as_deref(), &headers);
                (real, lang)
            }
            None => return Redirect::to("/auth/login").into_response(),
        }
    } else if user.impersonation.is_some() {
        return StatusCode::FORBIDDEN.into_response();
    } else {
        (user.user, user.lang)
    };
    if user.auth_provider != "local" {
        return StatusCode::FORBIDDEN.into_response();
    }
    let result: Result<Response> = async {
        let user = &user;
        let mut tx = state.pool.begin().await?;
        sqlx::query("UPDATE user_mfa SET last_step = last_step WHERE user_id = ?")
            .bind(&user.id)
            .execute(&mut *tx)
            .await?;
        let has_totp = store::has_totp(&mut tx, &user.id).await?;
        let keys = store::list(&mut tx, &user.id).await?;
        if !has_totp && keys.is_empty() {
            // Nothing to confirm with; setting up the first factor is
            // `enable`, from the settings page.
            return Ok(Redirect::to("/dashboard/settings/mfa").into_response());
        }
        let rp = state.passkeys.as_ref();
        let mut key_name = String::new();
        let known = match query.action.as_str() {
            "regenerate" => true,
            "disable" => has_totp,
            "totp_add" => !has_totp,
            "passkey_add" => rp.is_some() && (keys.len() as i64) < MAX_KEYS,
            "passkey_rename" | "passkey_delete" => {
                match keys.iter().find(|k| k.id == query.credential) {
                    Some(key) => {
                        key_name = key.name.clone();
                        true
                    }
                    None => false,
                }
            }
            "policy" => {
                user.role == "admin" && matches!(query.policy.as_str(), "required" | "optional")
            }
            _ => false,
        };
        if !known {
            return Ok(StatusCode::NOT_FOUND.into_response());
        }
        // Removing this would leave no factor, or none that works here:
        // refused while MFA is required (as `manage` refuses it). With no
        // factor at all MFA goes off, which is said otherwise (the recovery
        // codes go too).
        let leaves = removal(&query.action, &query.credential, has_totp, &keys, rp);
        let last_factor = leaves.refused_when_required();
        let turns_off = leaves == Removal::LastFactor;
        // Only what this action uses goes back into the form and the
        // links, each already checked above: the key is one of the user's,
        // the policy one of the two values.
        let credential = match query.action.as_str() {
            "passkey_rename" | "passkey_delete" => query.credential.as_str(),
            _ => "",
        };
        let policy = if query.action == "policy" {
            query.policy.as_str()
        } else {
            ""
        };
        let mandatory: bool =
            sqlx::query_scalar("SELECT mfa_required FROM auth_config WHERE id = 'singleton'")
                .fetch_one(&mut *tx)
                .await?;
        let usable: Vec<Credential> = match rp {
            Some(rp) => keys.into_iter().filter(|k| k.rp_id == rp.id).collect(),
            None => Vec::new(),
        };
        let mut method = pick_method(&query.method, !usable.is_empty(), has_totp);
        let mut options = None;
        if let (Some(rp), "passkey") = (rp, method) {
            let binding = Binding::Session(&session_id);
            match store::begin(&mut tx, &user.id, Kind::Confirm, binding).await? {
                Some(c) => options = Some(request_options(rp, &c, &descriptors(&usable))),
                None => method = pick_method("", false, has_totp),
            }
        }
        tx.commit().await?;
        let post_to = if query.action == "policy" {
            "/dashboard/admin/mfa"
        } else {
            "/dashboard/settings/mfa"
        };
        Ok(mfa::render(
            &state,
            lang,
            minijinja::context! {
                mode => "confirm",
                action => query.action,
                credential => credential,
                key_name => key_name,
                policy => policy,
                method => method,
                has_totp => has_totp,
                passkey_available => !usable.is_empty(),
                last_factor => last_factor,
                turns_off => turns_off,
                mandatory => mandatory,
                options => options.as_ref().map(json_value),
                post_to => post_to,
                kind => add_kind(&query.kind),
            },
        ))
    }
    .await;
    result.unwrap_or_else(|e| web::internal_error_response("MFA confirmation", &e))
}

/// `POST /dashboard/settings/mfa/passkeys`: registers a key after a
/// confirmed `passkey_add`. The `register` ceremony is the proof: it was
/// issued after the password and a factor, to this session, under this
/// password hash. Signs out every other session.
async fn register_finish(
    State(state): State<Arc<AppState>>,
    user: auth::AuthUser,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<RegistrationFields>,
) -> Response {
    let Some(rp) = state.passkeys.clone() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if let Err(r) = web::verify_csrf_token(&headers, &form._csrf) {
        return r;
    }
    if user.impersonation.is_some() || user.user.auth_provider != "local" {
        return StatusCode::FORBIDDEN.into_response();
    }
    let lang = user.lang;
    let ip = web::client_ip_for_rate_limit(&headers);
    let session_id = jar
        .get(mfa::SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .unwrap_or_default();
    let user = &user.user;
    let result: Result<Response> = async {
        let Ok(typed) = typed_name(&form.name) else {
            return Ok(mfa::settings_error(&state, lang));
        };
        let mut tx = state.pool.begin().await?;
        sqlx::query("UPDATE user_mfa SET last_step = last_step WHERE user_id = ?")
            .bind(&user.id)
            .execute(&mut *tx)
            .await?;
        if !mfa::same_local_user(&mut tx, user).await? {
            return Ok(mfa::settings_error(&state, lang));
        }
        let binding = Binding::Session(&session_id);
        let checked =
            store::check_registration(&mut tx, &rp, &user.id, &form, Kind::Register, binding)
                .await?;
        let registration = match checked {
            Ok(r) => r,
            Err(refusal) => {
                tx.commit().await?;
                tracing::warn!(user_id = %user.id, ip = %ip, reason = refusal.code(), "passkey registration refused");
                return Ok(mfa::settings_notice(&state, lang, "passkey"));
            }
        };
        let name =
            name_or_default(&mut tx, &user.id, lang, typed, registration.backup_eligible).await?;
        let credential =
            match store::add(&mut tx, &user.id, &rp, &registration, &form.transports, &name)
                .await?
            {
                Ok(id) => id,
                Err(why) => {
                    tx.commit().await?;
                    tracing::warn!(user_id = %user.id, ip = %ip, reason = ?why, "passkey registration refused");
                    let notice = match why {
                        NotAdded::Exists => "exists",
                        NotAdded::Limit => "limit",
                    };
                    return Ok(mfa::settings_notice(&state, lang, notice));
                }
            };
        // Every other session and pending challenge ends; this session was
        // issued by the confirmation that allowed this key.
        sqlx::query("DELETE FROM sessions WHERE user_id = ? AND id != ?")
            .bind(&user.id)
            .bind(&session_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM mfa_challenges WHERE user_id = ?")
            .bind(&user.id)
            .execute(&mut *tx)
            .await?;
        mfa::clear_attempts(&mut tx, &user.id).await?;
        tx.commit().await?;
        tracing::info!(user_id = %user.id, credential = %credential, action = "passkey_add", "MFA settings changed");
        Ok(Redirect::to("/dashboard/settings/mfa").into_response())
    }
    .await;
    result.unwrap_or_else(|e| web::internal_error_response("passkey registration", &e))
}

/// `GET /static/webauthn.js`: same origin, no inline handlers, so the page
/// needs no CSP change.
async fn script(State(state): State<Arc<AppState>>) -> Response {
    static SCRIPT: &str = include_str!("../../assets/webauthn.js");
    if state.passkeys.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }
    (
        [
            ("Content-Type", "application/javascript; charset=utf-8"),
            ("Cache-Control", "no-cache"),
        ],
        SCRIPT,
    )
        .into_response()
}
