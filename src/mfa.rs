//! Local-auth MFA. A challenge is never a session; only completing it grants
//! dashboard access. SQLite transactions serialize code consumption and session
//! creation, including recovery codes and concurrent enrollment attempts.
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use axum::{
    extract::{Form, Query, State},
    http::{HeaderMap, StatusCode},
    response::{AppendHeaders, Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use rand::{rngs::OsRng, RngCore};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{SqliteConnection, SqlitePool};
use subtle::ConstantTimeEq;
use totp_rs::{Builder, Totp};
use zeroize::Zeroizing;

use crate::{
    auth,
    crypto,
    models::{Session, User},
    web::{self, AppState},
    // ilia fork: passkeys as a second factor (docs/passkeys-plan.md).
    webauthn::{self, store as passkeys},
};

pub(crate) const CHALLENGE_COOKIE: &str = "__Host-calrs_mfa";
pub(crate) const SESSION_COOKIE: &str = "__Host-calrs_session";
pub(crate) const CHALLENGE_SECONDS: i64 = 600;

fn random_hex(bytes: usize) -> String {
    let mut value = Zeroizing::new(vec![0; bytes]);
    OsRng.fill_bytes(&mut value);
    hex::encode(&*value)
}

pub(crate) fn hash(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

fn totp(secret: &str, email: &str) -> Result<Totp> {
    Ok(Builder::new()
        .with_secret(hex::decode(secret)?)
        .with_account_name(email.replace(':', "_"))
        .with_issuer(Some("calrs"))
        .build()?)
}

// Return the matched counter so it can be consumed once, even across processes.
fn matching_step(secret: &str, code: &str, now: i64) -> Result<Option<i64>> {
    if code.len() != 6 || !code.bytes().all(|c| c.is_ascii_digit()) || now < 30 {
        return Ok(None);
    }
    let generator = totp(secret, "user")?;
    for step in (now / 30 - 1..=now / 30 + 1).rev() {
        let expected = generator.generate((step * 30) as u64).to_string();
        if bool::from(expected.as_bytes().ct_eq(code.as_bytes())) {
            return Ok(Some(step));
        }
    }
    Ok(None)
}

pub async fn required(pool: &SqlitePool) -> Result<bool> {
    Ok(
        sqlx::query_scalar("SELECT mfa_required FROM auth_config WHERE id = 'singleton'")
            .fetch_one(pool)
            .await?,
    )
}

/// Has a second factor: TOTP or (ilia fork) at least one passkey.
pub async fn enrolled(pool: &SqlitePool, user_id: &str) -> Result<bool> {
    passkeys::has_factor(&mut *pool.acquire().await?, user_id).await
}

/// Wrong codes an account may submit in a row before it has to wait.
const FREE_FAILURES: i64 = 10;
/// Wait after each failure past the free ones. It doubles with every further
/// run of `FREE_FAILURES`, up to `MAX_WAIT_SECONDS`, so guessing a six-digit
/// code stays out of reach over months, not just over 15 minutes.
const FIRST_WAIT_SECONDS: i64 = 900;
const MAX_WAIT_SECONDS: i64 = 86_400;

/// How long an account must wait after its last failure, given how many
/// consecutive failures it has.
fn wait_after(failures: i64) -> i64 {
    if failures < FREE_FAILURES {
        return 0;
    }
    let doublings = (failures / FREE_FAILURES - 1).min(10);
    (FIRST_WAIT_SECONDS << doublings).min(MAX_WAIT_SECONDS)
}

/// A recovery code carries 128 random bits, so it cannot be guessed and is
/// never throttled: it is how the owner gets in while someone holding their
/// password keeps failing codes on purpose.
fn is_recovery_code(code: &str) -> bool {
    code.len() == 32 && code.bytes().all(|c| c.is_ascii_hexdigit())
}

/// Reserve one attempt at a code or password for `user_id`, counted as a
/// failure until a success clears it (`clear_attempts`), so parallel requests
/// cannot overrun the budget. The budget is per account: starting a new
/// challenge or changing IPs does not reset it. Returns false, and counts
/// nothing, while the account must wait.
pub(crate) async fn reserve_attempt(pool: &SqlitePool, user_id: &str) -> Result<bool> {
    let now = Utc::now().timestamp();
    let mut tx = pool.begin().await?;
    // Take the write lock before reading, so the check and the count are one step.
    sqlx::query("UPDATE mfa_attempts SET attempts = attempts WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    let previous: Option<(i64, i64)> =
        sqlx::query_as("SELECT attempts, window_start FROM mfa_attempts WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?;
    if let Some((failures, last)) = previous {
        if now < last + wait_after(failures) {
            tracing::warn!(user_id = %user_id, failures, "MFA attempt refused: too many wrong codes");
            return Ok(false);
        }
    }
    // `window_start` holds the time of the latest counted attempt.
    sqlx::query(
        "INSERT INTO mfa_attempts (user_id, window_start, attempts) VALUES (?, ?, 1)
         ON CONFLICT(user_id) DO UPDATE SET attempts = attempts + 1, window_start = excluded.window_start",
    )
    .bind(user_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

/// A verified code or password: the account's consecutive failures start over.
pub(crate) async fn clear_attempts(conn: &mut SqliteConnection, user_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM mfa_attempts WHERE user_id = ?")
        .bind(user_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// The budget check for `code`: recovery codes always pass.
async fn may_try(pool: &SqlitePool, user_id: &str, code: &str) -> Result<bool> {
    Ok(is_recovery_code(code) || reserve_attempt(pool, user_id).await?)
}

pub enum LoginResult {
    Session(Session),
    Challenge(String),
}

/// Called only after password verification (or successful registration).
/// `setup` is an explicit, password-confirmed request to enable optional MFA.
pub async fn begin_login(
    pool: &SqlitePool,
    key: &[u8; 32],
    user: &User,
    setup: bool,
) -> Result<LoginResult> {
    let now = Utc::now().timestamp();
    let mut tx = pool.begin().await?;
    // Acquire the SQLite write lock before checking policy/enrollment.
    sqlx::query("DELETE FROM mfa_challenges WHERE expires_at <= ?")
        .bind(now)
        .execute(&mut *tx)
        .await?;
    let current: User = sqlx::query_as(
        "SELECT * FROM users WHERE id = ? AND enabled = 1 AND auth_provider = 'local'",
    )
    .bind(&user.id)
    .fetch_one(&mut *tx)
    .await?;
    if current.password_hash != user.password_hash {
        bail!("credentials changed");
    }
    // ilia fork: TOTP or a passkey.
    let active = passkeys::has_factor(&mut tx, &user.id).await?;
    let mandatory: bool =
        sqlx::query_scalar("SELECT mfa_required FROM auth_config WHERE id = 'singleton'")
            .fetch_one(&mut *tx)
            .await?;
    if setup && active {
        bail!("MFA already enabled");
    }
    let result = if active || mandatory || setup {
        LoginResult::Challenge(insert_challenge(&mut tx, key, user, !active, now).await?)
    } else {
        LoginResult::Session(auth::create_session(&mut *tx, &user.id).await?)
    };
    tx.commit().await?;
    Ok(result)
}

/// A new MFA challenge for `user`, replacing any earlier one: a login one,
/// or an enrolment one carrying a fresh TOTP secret.
async fn insert_challenge(
    conn: &mut SqliteConnection,
    key: &[u8; 32],
    user: &User,
    enroll: bool,
    now: i64,
) -> Result<String> {
    let token = random_hex(32);
    let secret_enc = if enroll {
        Some(crypto::encrypt_value(key, &Zeroizing::new(random_hex(20)))?)
    } else {
        None
    };
    // Only the latest password-authenticated challenge survives.
    sqlx::query("DELETE FROM mfa_challenges WHERE user_id = ?")
        .bind(&user.id)
        .execute(&mut *conn)
        .await?;
    sqlx::query("INSERT INTO mfa_challenges (token_hash, user_id, password_hash, purpose, secret_enc, expires_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(hash(&token)).bind(&user.id).bind(user.password_hash.as_deref().context("missing password")?)
        .bind(if enroll { "enroll" } else { "login" }).bind(secret_enc)
        .bind(now + CHALLENGE_SECONDS).execute(&mut *conn).await?;
    Ok(token)
}

pub(crate) fn session_cookie(session: &Session) -> String {
    format!(
        "{SESSION_COOKIE}={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=2592000",
        session.id
    )
}

pub(crate) fn challenge_cookie(token: &str, age: i64) -> String {
    format!("{CHALLENGE_COOKIE}={token}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={age}")
}

pub async fn cancel_challenge(pool: &SqlitePool, jar: &CookieJar) -> Result<String> {
    if let Some(cookie) = jar.get(CHALLENGE_COOKIE) {
        sqlx::query("DELETE FROM mfa_challenges WHERE token_hash = ?")
            .bind(hash(cookie.value()))
            .execute(pool)
            .await?;
    }
    Ok(challenge_cookie("", 0))
}

pub fn login_response(result: LoginResult) -> Response {
    match result {
        LoginResult::Session(session) => (
            AppendHeaders([
                ("Set-Cookie", session_cookie(&session)),
                ("Set-Cookie", challenge_cookie("", 0)),
            ]),
            Redirect::to("/dashboard"),
        )
            .into_response(),
        LoginResult::Challenge(token) => (
            [("Set-Cookie", challenge_cookie(&token, CHALLENGE_SECONDS))],
            Redirect::to("/auth/mfa"),
        )
            .into_response(),
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct Challenge {
    pub user_id: String,
    pub purpose: String,
    secret_enc: Option<String>,
    pub email: String,
    pub name: String,
}

pub(crate) async fn challenge(
    conn: &mut SqliteConnection,
    token: &str,
) -> Result<Option<Challenge>> {
    Ok(sqlx::query_as(
        "SELECT c.user_id, c.purpose, c.secret_enc, u.email, u.name FROM mfa_challenges c
         JOIN users u ON u.id = c.user_id
         WHERE c.token_hash = ? AND c.expires_at > ? AND u.enabled = 1
           AND u.auth_provider = 'local' AND c.password_hash = u.password_hash",
    )
    .bind(hash(token))
    .bind(Utc::now().timestamp())
    .fetch_optional(conn)
    .await?)
}

pub(crate) async fn new_recovery_codes(
    conn: &mut SqliteConnection,
    user_id: &str,
) -> Result<Vec<String>> {
    sqlx::query("DELETE FROM mfa_recovery_codes WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *conn)
        .await?;
    let mut codes = Vec::new();
    for _ in 0..10 {
        // 128 random bits: SHA-256 is sufficient for these high-entropy codes.
        let code = random_hex(16);
        sqlx::query("INSERT INTO mfa_recovery_codes (user_id, code_hash) VALUES (?, ?)")
            .bind(user_id)
            .bind(hash(&code))
            .execute(&mut *conn)
            .await?;
        codes.push(code);
    }
    Ok(codes)
}

async fn consume_code(
    conn: &mut SqliteConnection,
    key: &[u8; 32],
    user_id: &str,
    code: &str,
) -> Result<bool> {
    // ilia fork: recovery codes belong to the user, not to the TOTP row (a
    // passkey-only account has them too), and exist only while the user has
    // a factor, so they are checked before the TOTP row is looked up.
    if is_recovery_code(code) {
        return Ok(sqlx::query(
            "DELETE FROM mfa_recovery_codes WHERE user_id = ? AND code_hash = ?",
        )
        .bind(user_id)
        .bind(hash(&code.to_ascii_lowercase()))
        .execute(conn)
        .await?
        .rows_affected()
            == 1);
    }
    let credential: Option<(String, i64)> =
        sqlx::query_as("SELECT secret_enc, last_step FROM user_mfa WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await?;
    let Some((encrypted, last_step)) = credential else {
        return Ok(false);
    };
    let secret = Zeroizing::new(crypto::decrypt_value(key, &encrypted)?);
    let Some(step) = matching_step(&secret, code, Utc::now().timestamp())? else {
        return Ok(false);
    };
    if step <= last_step {
        return Ok(false);
    }
    Ok(
        sqlx::query("UPDATE user_mfa SET last_step = ? WHERE user_id = ? AND last_step < ?")
            .bind(step)
            .bind(user_id)
            .bind(step)
            .execute(conn)
            .await?
            .rows_affected()
            == 1,
    )
}

pub(crate) async fn verified_session(
    conn: &mut SqliteConnection,
    user_id: &str,
) -> Result<Session> {
    let session = auth::create_session(&mut *conn, user_id).await?;
    sqlx::query("UPDATE sessions SET mfa_verified = 1 WHERE id = ?")
        .bind(&session.id)
        .execute(conn)
        .await?;
    Ok(session)
}

pub(crate) async fn revoke_access(
    conn: &mut SqliteConnection,
    user_id: &str,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *conn)
        .await?;
    sqlx::query("DELETE FROM mfa_challenges WHERE user_id = ?")
        .bind(user_id)
        .execute(conn)
        .await?;
    Ok(())
}

#[cfg(test)]
async fn finish_challenge(
    pool: &SqlitePool,
    key: &[u8; 32],
    token: &str,
    code: &str,
) -> Result<Option<(Session, Vec<String>)>> {
    finish_challenge_from(pool, key, token, code, "test").await
}

/// Complete the challenge `token` with `code`, submitted from `ip`. A wrong
/// code is logged with the account and the address, so an admin can see a
/// guessing attempt in progress.
async fn finish_challenge_from(
    pool: &SqlitePool,
    key: &[u8; 32],
    token: &str,
    code: &str,
    ip: &str,
) -> Result<Option<(Session, Vec<String>)>> {
    let Some(initial) = challenge(&mut *pool.acquire().await?, token).await? else {
        return Ok(None);
    };
    if !may_try(pool, &initial.user_id, code).await? {
        return Ok(None);
    }
    let result = verify_challenge(pool, key, token, code).await?;
    if result.is_none() {
        tracing::warn!(user_id = %initial.user_id, ip = %ip, "MFA code refused");
    }
    Ok(result)
}

async fn verify_challenge(
    pool: &SqlitePool,
    key: &[u8; 32],
    token: &str,
    code: &str,
) -> Result<Option<(Session, Vec<String>)>> {
    let mut tx = pool.begin().await?;
    // Write first: concurrent consumers cannot both observe the same challenge.
    sqlx::query("UPDATE mfa_challenges SET expires_at = expires_at WHERE token_hash = ?")
        .bind(hash(token))
        .execute(&mut *tx)
        .await?;
    let Some(c) = challenge(&mut tx, token).await? else {
        return Ok(None);
    };
    let mut codes = Vec::new();
    if c.purpose == "enroll" {
        let encrypted = c.secret_enc.context("missing enrollment secret")?;
        let secret = Zeroizing::new(crypto::decrypt_value(key, &encrypted)?);
        let Some(step) = matching_step(&secret, code, Utc::now().timestamp())? else {
            return Ok(None);
        };
        // ilia fork: a user who has passkeys and adds TOTP keeps the codes
        // they have; the first factor brings new ones.
        let had_factor = passkeys::has_factor(&mut tx, &c.user_id).await?;
        let inserted = sqlx::query(
            "INSERT OR IGNORE INTO user_mfa (user_id, secret_enc, last_step) VALUES (?, ?, ?)",
        )
        .bind(&c.user_id)
        .bind(encrypted)
        .bind(step)
        .execute(&mut *tx)
        .await?;
        if inserted.rows_affected() != 1 {
            return Ok(None);
        }
        if !had_factor {
            codes = new_recovery_codes(&mut tx, &c.user_id).await?;
        }
        revoke_access(&mut tx, &c.user_id).await?;
    } else if !consume_code(&mut tx, key, &c.user_id, code).await? {
        return Ok(None);
    }
    sqlx::query("DELETE FROM mfa_challenges WHERE token_hash = ?")
        .bind(hash(token))
        .execute(&mut *tx)
        .await?;
    clear_attempts(&mut tx, &c.user_id).await?;
    let session = verified_session(&mut tx, &c.user_id).await?;
    tx.commit().await?;
    tracing::info!(user_id = %c.user_id, action = %c.purpose, "MFA completed");
    Ok(Some((session, codes)))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/mfa", get(challenge_page).post(challenge_submit))
        .route(
            "/dashboard/settings/mfa",
            get(settings_page).post(settings_action),
        )
        .route("/dashboard/admin/mfa", post(policy_save))
}

pub(crate) fn render(state: &AppState, lang: &str, values: minijinja::Value) -> Response {
    let csrf = web::generate_csrf_token();
    let result = state.templates.get_template("auth/mfa.html").and_then(|t| {
        t.render(minijinja::context! {
            lang => lang, csrf_token => csrf,
            // ilia fork: the only address passkeys work at, for the notice
            // a browser on another one gets.
            passkey_origin => state.passkeys.as_ref().map(|rp| rp.origin.clone()),
            ..values
        })
    });
    match result {
        Ok(body) => (
            [
                ("Cache-Control", "no-store"),
                ("Referrer-Policy", "no-referrer"),
                ("Set-Cookie", &web::csrf_cookie_value(&csrf)),
            ],
            Html(body),
        )
            .into_response(),
        Err(e) => web::internal_error_response("MFA template", &e),
    }
}

/// The challenge failed or expired: the way back is a new login.
pub(crate) fn error(state: &AppState, lang: &str) -> Response {
    render(state, lang, minijinja::context! { mode => "error" })
}

/// A settings or policy change was refused: the visitor is logged in, so
/// offer the MFA settings page as well as the login.
pub(crate) fn settings_error(state: &AppState, lang: &str) -> Response {
    settings_notice(state, lang, "")
}

/// ilia fork: a refused change with its reason (`notice`, empty for the
/// generic one): `last_factor`, `limit`, `exists`.
pub(crate) fn settings_notice(state: &AppState, lang: &str, notice: &str) -> Response {
    render(
        state,
        lang,
        minijinja::context! { mode => "error", logged_in => true, notice => notice },
    )
}

/// ilia fork: which view of the challenge page to show (`?method=passkey`,
/// `totp` or `recovery`). A choice of view only: `POST /auth/mfa` takes a
/// TOTP or a recovery code whichever is shown.
#[derive(Deserialize, Default)]
struct MethodQuery {
    #[serde(default)]
    method: String,
}

pub(crate) async fn challenge_view(
    state: &AppState,
    lang: &str,
    token: &str,
    // ilia fork: why the last attempt failed: "" (it did not), "code" (a
    // code, the upstream message) or "passkey" (a key, its own message).
    invalid: &str,
    method: &str,
) -> Result<Response> {
    let c = challenge(&mut *state.pool.acquire().await?, token).await?;
    let Some(c) = c else {
        return Ok(error(state, lang));
    };
    // ilia fork: the passkey view and the methods on offer.
    let view = webauthn::flows::challenge_methods(state, token, &c, method).await?;
    let (secret, qr) = match &c.secret_enc {
        Some(encrypted) if !view.passkey_view => {
            let secret = Zeroizing::new(crypto::decrypt_value(&state.secret_key, encrypted)?);
            let generator = totp(&secret, &c.email)?;
            (generator.secret().to_base32(), generator.to_qr_base64()?)
        }
        _ => (String::new(), String::new()),
    };
    Ok(render(
        state,
        lang,
        minijinja::context! {
            mode => c.purpose, secret => secret, qr => qr, invalid => invalid,
            ..view.context()
        },
    ))
}

async fn challenge_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    jar: CookieJar,
    Query(query): Query<MethodQuery>,
) -> Response {
    let lang = crate::i18n::detect_from_headers(&headers);
    let token = jar.get(CHALLENGE_COOKIE).map(|c| c.value()).unwrap_or("");
    match challenge_view(&state, lang, token, "", &query.method).await {
        Ok(r) => r,
        Err(e) => web::internal_error_response("MFA challenge", &e),
    }
}

#[derive(Deserialize)]
struct CodeForm {
    _csrf: Option<String>,
    code: String,
    /// ilia fork: the view the code was typed in, shown again on failure.
    #[serde(default)]
    method: String,
}

async fn challenge_submit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<CodeForm>,
) -> Response {
    if let Err(r) = web::verify_csrf_token(&headers, &form._csrf) {
        return r;
    }
    let lang = crate::i18n::detect_from_headers(&headers);
    let token = jar.get(CHALLENGE_COOKIE).map(|c| c.value()).unwrap_or("");
    let ip = web::client_ip_for_rate_limit(&headers);
    match finish_challenge_from(&state.pool, &state.secret_key, token, form.code.trim(), &ip).await
    {
        Ok(Some((session, codes))) => completion(&state, lang, session, codes),
        Ok(None) => match challenge_view(&state, lang, token, "code", &form.method).await {
            Ok(r) => r,
            Err(e) => web::internal_error_response("MFA challenge", &e),
        },
        Err(e) => web::internal_error_response("MFA verification", &e),
    }
}

pub(crate) fn completion(
    state: &AppState,
    lang: &str,
    session: Session,
    codes: Vec<String>,
) -> Response {
    if codes.is_empty() {
        return login_response(LoginResult::Session(session));
    }
    let mut response = render(
        state,
        lang,
        minijinja::context! { mode => "recovery", codes => codes },
    );
    response
        .headers_mut()
        .append("Set-Cookie", session_cookie(&session).parse().unwrap());
    response
        .headers_mut()
        .append("Set-Cookie", challenge_cookie("", 0).parse().unwrap());
    response
}

async fn settings_page(State(state): State<Arc<AppState>>, user: auth::AuthUser) -> Response {
    if user.impersonation.is_some() || user.user.auth_provider != "local" {
        return StatusCode::FORBIDDEN.into_response();
    }
    let result: Result<_> = async {
        Ok((
            enrolled(&state.pool, &user.user.id).await?,
            required(&state.pool).await?,
            // ilia fork: keys, TOTP and recovery codes, one by one.
            webauthn::flows::settings_context(&state, &user.user, user.lang).await?,
        ))
    }
    .await;
    match result {
        Ok((active, mandatory, factors)) => render(
            &state,
            user.lang,
            minijinja::context! {
                mode => "settings", active => active, mandatory => mandatory, ..factors
            },
        ),
        Err(e) => web::internal_error_response("MFA settings", &e),
    }
}

#[derive(Deserialize)]
struct ManagementForm {
    _csrf: Option<String>,
    password: String,
    #[serde(default)]
    code: String,
    action: String,
    // ilia fork: the key an action is about, its new name, and a passkey
    // assertion in place of the code.
    #[serde(default)]
    credential: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    credential_id: String,
    #[serde(default)]
    client_data: String,
    #[serde(default)]
    authenticator_data: String,
    #[serde(default)]
    signature: String,
    #[serde(default)]
    user_handle: String,
    /// ilia fork: which add button was chosen (`security-key` or `device`),
    /// so the register page leads with it.
    #[serde(default)]
    kind: String,
}

/// ilia fork: the passkey assertion fields of a form.
fn assertion_of(
    credential_id: &str,
    client_data: &str,
    authenticator_data: &str,
    signature: &str,
    user_handle: &str,
) -> webauthn::AssertionFields {
    webauthn::AssertionFields {
        credential_id: credential_id.to_string(),
        client_data: client_data.to_string(),
        authenticator_data: authenticator_data.to_string(),
        signature: signature.to_string(),
        user_handle: user_handle.to_string(),
        ..Default::default()
    }
}

/// ilia fork: what a change proves besides the password: a TOTP or
/// recovery code, or a passkey assertion answering the "confirm it's you"
/// ceremony of this session.
pub(crate) enum Proof<'a> {
    Code(&'a str),
    Passkey(&'a webauthn::AssertionFields),
}

impl<'a> Proof<'a> {
    fn of(fields: &'a webauthn::AssertionFields, code: &'a str) -> Proof<'a> {
        if fields.present() {
            Proof::Passkey(fields)
        } else {
            Proof::Code(code.trim())
        }
    }
}

/// ilia fork: consumes `proof` for `user_id` inside the caller's
/// transaction. A passkey proof burns the session's `confirm` ceremony,
/// which the caller commits even when this returns false.
pub(crate) async fn consume_proof(
    conn: &mut SqliteConnection,
    state: &AppState,
    user_id: &str,
    proof: Proof<'_>,
    session_id: &str,
    ip: &str,
) -> Result<bool> {
    match proof {
        Proof::Code(code) => consume_code(conn, &state.secret_key, user_id, code).await,
        Proof::Passkey(fields) => {
            let Some(rp) = state.passkeys.as_ref() else {
                return Ok(false);
            };
            let outcome = passkeys::check_assertion(
                conn,
                rp,
                user_id,
                fields,
                passkeys::Kind::Confirm,
                passkeys::Binding::Session(session_id),
            )
            .await?;
            match outcome {
                Ok(credential) => {
                    tracing::info!(user_id = %user_id, credential = %credential, "passkey confirmed a change");
                    Ok(true)
                }
                Err(refusal) => {
                    tracing::warn!(user_id = %user_id, ip = %ip, reason = refusal.code(), "passkey confirmation refused");
                    Ok(false)
                }
            }
        }
    }
}

// Recheck enabled/provider/password after obtaining the write lock, protecting
// management requests racing account disable, password reset or OIDC linking.
pub(crate) async fn same_local_user(conn: &mut SqliteConnection, user: &User) -> Result<bool> {
    Ok(sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND enabled = 1 AND auth_provider = 'local' AND password_hash = ?)")
        .bind(&user.id).bind(&user.password_hash).fetch_one(conn).await?)
}

/// What a settings change led to.
enum Managed {
    /// Done. `session` replaces the visitor's (`None` for `totp_add`, whose
    /// enrolment issues it); `codes` are shown once.
    Done {
        session: Option<Session>,
        codes: Vec<String>,
        next: Next,
    },
    /// Refused before anything changed, with the reason shown (`""` for
    /// the generic one, `last_factor`, `limit`, or `not_found` for a 404).
    Refused(&'static str),
}

enum Next {
    /// Upstream: the recovery codes page, or the dashboard.
    Completion,
    /// ilia fork: back to the MFA settings.
    Settings,
    /// ilia fork: the page that registers a new key, with its options.
    Register(serde_json::Value),
    /// ilia fork: the TOTP enrolment of this challenge token.
    TotpEnroll(String),
}

/// The changes `POST /dashboard/settings/mfa` makes, besides `enable`.
const ACTIONS: &[&str] = &[
    "regenerate",
    "disable",
    // ilia fork: passkeys, and TOTP next to them.
    "totp_add",
    "passkey_add",
    "passkey_rename",
    "passkey_delete",
];

async fn manage(
    state: &AppState,
    user: &User,
    form: &ManagementForm,
    session_id: &str,
    ip: &str,
) -> Result<Managed> {
    if !ACTIONS.contains(&form.action.as_str()) {
        return Ok(Managed::Refused(""));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE user_mfa SET last_step = last_step WHERE user_id = ?")
        .bind(&user.id)
        .execute(&mut *tx)
        .await?;
    if !same_local_user(&mut tx, user).await? {
        return Ok(Managed::Refused(""));
    }
    let mandatory: bool =
        sqlx::query_scalar("SELECT mfa_required FROM auth_config WHERE id = 'singleton'")
            .fetch_one(&mut *tx)
            .await?;
    // ilia fork: a factor is TOTP or a passkey. Without one there is
    // nothing to manage (`enable` is the way in).
    let has_totp = passkeys::has_totp(&mut tx, &user.id).await?;
    let key_list = passkeys::list(&mut tx, &user.id).await?;
    let keys = key_list.len() as i64;
    if !has_totp && keys == 0 {
        return Ok(Managed::Refused(""));
    }
    // ilia fork: the last factor, or the last one that works here (a key
    // registered for another RP ID, or any key while passkeys are off,
    // cannot sign in), cannot go while MFA is required.
    let removal = webauthn::flows::removal(
        &form.action,
        &form.credential,
        has_totp,
        &key_list,
        state.passkeys.as_ref(),
    );
    let name = webauthn::clean_name(&form.name);
    // What can be refused without a proof is refused before one is used up.
    // The last factor cannot go while MFA is required.
    match form.action.as_str() {
        "disable" if !has_totp => return Ok(Managed::Refused("")),
        "disable" if mandatory && removal.refused_when_required() => {
            return Ok(Managed::Refused("last_factor"))
        }
        "totp_add" if has_totp => return Ok(Managed::Refused("")),
        "passkey_add" if state.passkeys.is_none() => return Ok(Managed::Refused("not_found")),
        "passkey_add" if keys >= webauthn::MAX_KEYS => return Ok(Managed::Refused("limit")),
        "passkey_rename" | "passkey_delete" => {
            if passkeys::get(&mut tx, &user.id, &form.credential)
                .await?
                .is_none()
            {
                return Ok(Managed::Refused("not_found"));
            }
            if form.action == "passkey_rename" && name.is_none() {
                return Ok(Managed::Refused(""));
            }
            if form.action == "passkey_delete" && mandatory && removal.refused_when_required() {
                return Ok(Managed::Refused("last_factor"));
            }
        }
        _ => {}
    }
    let assertion = assertion_of(
        &form.credential_id,
        &form.client_data,
        &form.authenticator_data,
        &form.signature,
        &form.user_handle,
    );
    let proof = Proof::of(&assertion, &form.code);
    if !consume_proof(&mut tx, state, &user.id, proof, session_id, ip).await? {
        // A passkey proof's ceremony stays used up.
        tx.commit().await?;
        return Ok(Managed::Refused(if assertion.present() {
            "passkey"
        } else {
            ""
        }));
    }
    let mut codes = Vec::new();
    let next = match form.action.as_str() {
        "disable" => {
            sqlx::query("DELETE FROM user_mfa WHERE user_id = ?")
                .bind(&user.id)
                .execute(&mut *tx)
                .await?;
            Next::Completion
        }
        "regenerate" => {
            codes = new_recovery_codes(&mut tx, &user.id).await?;
            Next::Completion
        }
        "passkey_delete" => {
            passkeys::delete(&mut tx, &user.id, &form.credential).await?;
            Next::Settings
        }
        "passkey_rename" => {
            let name = name.as_deref().unwrap_or_default();
            passkeys::rename(&mut tx, &user.id, &form.credential, name).await?;
            Next::Settings
        }
        "totp_add" => {
            // The enrolment page shows the QR; completing it revokes the
            // other sessions and issues a verified one, as a first one does.
            let token = insert_challenge(
                &mut tx,
                &state.secret_key,
                user,
                true,
                Utc::now().timestamp(),
            )
            .await?;
            clear_attempts(&mut tx, &user.id).await?;
            tx.commit().await?;
            tracing::info!(user_id = %user.id, action = %form.action, "MFA settings changed");
            return Ok(Managed::Done {
                session: None,
                codes,
                next: Next::TotpEnroll(token),
            });
        }
        // passkey_add: the session issued below carries the ceremony.
        _ => Next::Register(serde_json::Value::Null),
    };
    let factor_left = passkeys::has_factor(&mut tx, &user.id).await?;
    if !factor_left {
        // ilia fork: the last factor went (the policy is optional), and the
        // recovery codes with it.
        sqlx::query("DELETE FROM mfa_recovery_codes WHERE user_id = ?")
            .bind(&user.id)
            .execute(&mut *tx)
            .await?;
    }
    revoke_access(&mut tx, &user.id).await?;
    clear_attempts(&mut tx, &user.id).await?;
    let session = if factor_left {
        verified_session(&mut tx, &user.id).await?
    } else {
        auth::create_session(&mut *tx, &user.id).await?
    };
    let next = match next {
        Next::Register(_) => Next::Register(
            webauthn::flows::register_options(&mut tx, state, user, &session.id).await?,
        ),
        other => other,
    };
    tx.commit().await?;
    // ilia fork: `credential` is logged only where it was matched against
    // the user's keys above; posted with another action it is any text.
    match form.action.as_str() {
        "passkey_rename" | "passkey_delete" => {
            tracing::info!(user_id = %user.id, action = %form.action, credential = %form.credential, "MFA settings changed")
        }
        _ => tracing::info!(user_id = %user.id, action = %form.action, "MFA settings changed"),
    }
    Ok(Managed::Done {
        session: Some(session),
        codes,
        next,
    })
}

async fn settings_action(
    State(state): State<Arc<AppState>>,
    user: auth::AuthUser,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<ManagementForm>,
) -> Response {
    if let Err(r) = web::verify_csrf_token(&headers, &form._csrf) {
        return r;
    }
    if user.impersonation.is_some() || user.user.auth_provider != "local" {
        return StatusCode::FORBIDDEN.into_response();
    }
    let ip = web::client_ip_for_rate_limit(&headers);
    // ilia fork: the session a passkey confirmation is bound to.
    let session_id = jar.get(SESSION_COOKIE).map(|c| c.value()).unwrap_or("");
    let result: Result<Response> = async {
        // Always counted, recovery code or not: the password is checked here
        // too, and it must not be guessable through this form.
        if !reserve_attempt(&state.pool, &user.user.id).await? {
            return Ok(settings_error(&state, user.lang));
        }
        if !auth::verify_password(
            &form.password,
            user.user.password_hash.as_deref().unwrap_or(""),
        ) {
            tracing::warn!(user_id = %user.user.id, ip = %ip, "MFA settings refused: wrong password");
            return Ok(settings_error(&state, user.lang));
        }
        if form.action == "enable" {
            // A second submit from another tab, after the first completed.
            if enrolled(&state.pool, &user.user.id).await? {
                return Ok(settings_error(&state, user.lang));
            }
            return Ok(login_response(
                begin_login(&state.pool, &state.secret_key, &user.user, true).await?,
            ));
        }
        Ok(match manage(&state, &user.user, &form, session_id, &ip).await? {
            Managed::Done {
                session,
                codes,
                next,
            } => match (next, session) {
                (Next::TotpEnroll(token), _) => login_response(LoginResult::Challenge(token)),
                (Next::Completion, Some(session)) => {
                    completion(&state, user.lang, session, codes)
                }
                (Next::Settings, Some(session)) => (
                    AppendHeaders([("Set-Cookie", session_cookie(&session))]),
                    Redirect::to("/dashboard/settings/mfa"),
                )
                    .into_response(),
                (Next::Register(options), Some(session)) => {
                    webauthn::flows::register_page(&state, user.lang, &session, options, &form.kind)
                }
                (_, None) => settings_error(&state, user.lang),
            },
            Managed::Refused(notice) => {
                // ilia fork: `action` is as posted here (perhaps none of
                // ACTIONS), so Debug escapes it: no forged log lines.
                tracing::warn!(user_id = %user.user.id, ip = %ip, action = ?form.action, notice = %notice, "MFA settings refused: wrong code or not allowed");
                if notice == "not_found" {
                    StatusCode::NOT_FOUND.into_response()
                } else {
                    settings_notice(&state, user.lang, notice)
                }
            }
        })
    }
    .await;
    result.unwrap_or_else(|e| web::internal_error_response("MFA management", &e))
}

#[derive(Deserialize)]
struct PolicyForm {
    _csrf: Option<String>,
    policy: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    code: String,
    // ilia fork: a passkey assertion in place of the code.
    #[serde(default)]
    credential_id: String,
    #[serde(default)]
    client_data: String,
    #[serde(default)]
    authenticator_data: String,
    #[serde(default)]
    signature: String,
    #[serde(default)]
    user_handle: String,
}

async fn policy_save(
    State(state): State<Arc<AppState>>,
    admin: auth::AdminUser,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<PolicyForm>,
) -> Response {
    if let Err(r) = web::verify_csrf_token(&headers, &form._csrf) {
        return r;
    }
    let mandatory = match form.policy.as_str() {
        "required" => true,
        "optional" => false,
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    let local = admin.user.auth_provider == "local";
    let ip = web::client_ip_for_rate_limit(&headers);
    let session_id = jar.get(SESSION_COOKIE).map(|c| c.value()).unwrap_or("");
    let assertion = assertion_of(
        &form.credential_id,
        &form.client_data,
        &form.authenticator_data,
        &form.signature,
        &form.user_handle,
    );
    let result: Result<bool> = async {
        if local && !reserve_attempt(&state.pool, &admin.user.id).await? {
            return Ok(false);
        }
        if local && !auth::verify_password(&form.password, admin.user.password_hash.as_deref().unwrap_or("")) {
            tracing::warn!(user_id = %admin.user.id, ip = %ip, "MFA policy change refused: wrong password");
            return Ok(false);
        }
        let mut tx = state.pool.begin().await?;
        sqlx::query("UPDATE auth_config SET mfa_required = mfa_required WHERE id = 'singleton'").execute(&mut *tx).await?;
        let still_admin: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND enabled = 1 AND role = 'admin' AND auth_provider = ?)")
            .bind(&admin.user.id).bind(&admin.user.auth_provider).fetch_one(&mut *tx).await?;
        if !still_admin { return Ok(false); }
        if local {
            let proof = Proof::of(&assertion, &form.code);
            if !same_local_user(&mut tx, &admin.user).await?
                || !consume_proof(&mut tx, &state, &admin.user.id, proof, session_id, &ip).await?
            {
                // ilia fork: a passkey proof's ceremony stays used up.
                tx.commit().await?;
                tracing::warn!(user_id = %admin.user.id, ip = %ip, "MFA policy change refused: wrong code");
                return Ok(false);
            }
            clear_attempts(&mut tx, &admin.user.id).await?;
        }
        store_policy(&mut tx, mandatory).await?;
        tx.commit().await?;
        tracing::info!(admin = %admin.user.id, required = mandatory, "MFA policy changed");
        Ok(true)
    }.await;
    match result {
        Ok(true) => Redirect::to("/dashboard/admin").into_response(),
        Ok(false) => settings_error(&state, admin.lang),
        Err(e) => web::internal_error_response("MFA policy", &e),
    }
}

/// Store the enrollment policy and, when it becomes mandatory, end every local
/// session that has not passed MFA. The caller has authorised the change and
/// owns the transaction: `policy_save` after the password and code, and (ilia
/// fork) `calrs config apply`, run by the server's OS user, on its own.
pub(crate) async fn store_policy(conn: &mut SqliteConnection, mandatory: bool) -> Result<()> {
    sqlx::query("UPDATE auth_config SET mfa_required = ?, updated_at = datetime('now') WHERE id = 'singleton'")
        .bind(mandatory).execute(&mut *conn).await?;
    if mandatory {
        sqlx::query("DELETE FROM sessions WHERE mfa_verified = 0 AND user_id IN (SELECT id FROM users WHERE auth_provider = 'local')")
            .execute(&mut *conn).await?;
    }
    Ok(())
}

/// Server-operator recovery. The policy remains in force, so the next local
/// login must enroll again if MFA is required. Never reset MFA on password save.
pub async fn reset(pool: &SqlitePool, email: &str) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM mfa_challenges WHERE expires_at <= ?")
        .bind(Utc::now().timestamp())
        .execute(&mut *tx)
        .await?;
    let id: String =
        sqlx::query_scalar("SELECT id FROM users WHERE email = ? AND auth_provider = 'local'")
            .bind(email)
            .fetch_one(&mut *tx)
            .await
            .context("local user not found")?;
    sqlx::query("DELETE FROM user_mfa WHERE user_id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    // ilia fork: passkeys, and the recovery codes, which no longer go with
    // the TOTP row.
    sqlx::query("DELETE FROM ilia_webauthn_credentials WHERE user_id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM mfa_recovery_codes WHERE user_id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM mfa_attempts WHERE user_id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    revoke_access(&mut tx, &id).await?;
    tx.commit().await?;
    tracing::info!(user_id = %id, "MFA reset by server operator");
    Ok(())
}

#[cfg(test)]
mod tests;
