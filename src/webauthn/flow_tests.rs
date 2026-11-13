//! ilia fork: passkey flows over HTTP (tower `oneshot`), with the software
//! authenticator of `softkey.rs` and the RP injected into `AppState` (no
//! test reads or sets `CALRS_BASE_URL`).

use std::{str::FromStr, sync::Arc, sync::OnceLock};

use axum::{body::Body, http::Request, response::Response, Router};
use chrono::Utc;
use http_body_util::BodyExt;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tower::ServiceExt;

use super::softkey::{self, SoftKey, BE, BS, ORIGIN, RP_ID, UV};
use super::*;
use crate::{
    auth, crypto,
    mfa::{self, LoginResult},
    models::User,
    web,
};

const KEY: [u8; 32] = [42; 32];
const PASSWORD: &str = "correct-horse-battery";
const CSRF: &str = "__Host-calrs_csrf=test";

fn rp() -> Rp {
    Rp {
        id: RP_ID.to_string(),
        origin: ORIGIN.to_string(),
    }
}

async fn database() -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::from_str("sqlite::memory:")
                .unwrap()
                .foreign_keys(true),
        )
        .await
        .unwrap()
}

async fn migrated() -> SqlitePool {
    let pool = database().await;
    crate::db::migrate(&pool).await.unwrap();
    pool
}

async fn user_named(pool: &SqlitePool, email: &str, name: &str) -> User {
    static PASSWORD_HASH: OnceLock<String> = OnceLock::new();
    let id = uuid::Uuid::new_v4().to_string();
    auth::create_local_user(
        pool,
        &id,
        email,
        name,
        PASSWORD_HASH.get_or_init(|| auth::hash_password(PASSWORD).unwrap()),
        &id,
        false,
    )
    .await
    .unwrap();
    sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn user(pool: &SqlitePool, email: &str) -> User {
    user_named(pool, email, "Test").await
}

async fn app_with(pool: &SqlitePool, passkeys: Option<Rp>) -> Router {
    let mut state = web::build_state(pool.clone(), std::env::temp_dir(), KEY).await;
    Arc::get_mut(&mut state).unwrap().passkeys = passkeys;
    web::router(state)
}

async fn app(pool: &SqlitePool) -> Router {
    app_with(pool, Some(rp())).await
}

async fn request(app: &Router, method: &str, path: &str, cookies: &str, fields: &str) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("cookie", cookies)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(fields.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap()
}

fn response_cookie(response: &Response, name: &str) -> Option<String> {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|v| {
            v.strip_prefix(&format!("{name}="))
                .map(|v| v.split(';').next().unwrap().to_owned())
        })
}

async fn body(response: Response) -> String {
    String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap()
}

fn challenge_cookies(token: &str) -> String {
    format!("{}={token}; {CSRF}", mfa::CHALLENGE_COOKIE)
}

fn session_cookies(session: &str) -> String {
    format!("{}={session}; {CSRF}", mfa::SESSION_COOKIE)
}

/// The options JSON a page embeds, if any.
fn page_options(html: &str) -> Option<serde_json::Value> {
    let start = r#"<script type="application/json" id="webauthn-options">"#;
    let from = html.find(start)? + start.len();
    let to = from + html[from..].find("</script>")?;
    serde_json::from_str(&html[from..to]).ok()
}

fn challenge_of(options: &serde_json::Value) -> Vec<u8> {
    verify::decode_field(options["challenge"].as_str().unwrap(), 64).unwrap()
}

fn enc(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

fn assertion_fields(asserted: &softkey::Asserted, credential_id: &[u8], handle: &[u8]) -> String {
    format!(
        "credential_id={}&client_data={}&authenticator_data={}&signature={}&user_handle={}",
        b64url(credential_id),
        b64url(&asserted.client_data),
        b64url(&asserted.auth_data),
        b64url(&asserted.signature),
        b64url(handle),
    )
}

fn registration_fields(registered: &softkey::Registered, name: &str) -> String {
    format!(
        "credential_id={}&client_data={}&authenticator_data={}&public_key={}&algorithm={}&transports={}&name={}",
        b64url(&registered.raw_id),
        b64url(&registered.client_data),
        b64url(&registered.auth_data),
        b64url(&registered.public_key),
        registered.algorithm,
        enc(r#"["usb","nfc","bogus"]"#),
        enc(name),
    )
}

async fn count(pool: &SqlitePool, sql: &str, user_id: &str) -> i64 {
    sqlx::query_scalar(sql)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn keys_of(pool: &SqlitePool, user_id: &str) -> i64 {
    count(
        pool,
        "SELECT COUNT(*) FROM ilia_webauthn_credentials WHERE user_id = ?",
        user_id,
    )
    .await
}

async fn codes_of(pool: &SqlitePool, user_id: &str) -> i64 {
    count(
        pool,
        "SELECT COUNT(*) FROM mfa_recovery_codes WHERE user_id = ?",
        user_id,
    )
    .await
}

async fn challenge_token(pool: &SqlitePool, user: &User, setup: bool) -> String {
    match mfa::begin_login(pool, &KEY, user, setup).await.unwrap() {
        LoginResult::Challenge(token) => token,
        LoginResult::Session(_) => panic!("expected an MFA challenge"),
    }
}

/// Enrols `key` as the first factor through the HTTP flow: returns the
/// verified session and the recovery codes shown.
async fn enroll_key(
    app: &Router,
    pool: &SqlitePool,
    user: &User,
    key: &SoftKey,
) -> (String, Vec<String>) {
    let token = challenge_token(pool, user, true).await;
    let page = request(app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await;
    assert_eq!(page.status(), 200);
    let html = body(page).await;
    let options = page_options(&html).expect("the enrolment page offers a passkey");
    assert_eq!(options["rp"]["id"], RP_ID);
    let registered = key.register(&challenge_of(&options));
    let r = request(
        app,
        "POST",
        "/auth/mfa/passkey/enroll",
        &challenge_cookies(&token),
        &format!("_csrf=test&{}", registration_fields(&registered, "")),
    )
    .await;
    assert_eq!(r.status(), 200, "enrolment completes on the codes page");
    let session = response_cookie(&r, mfa::SESSION_COOKIE).expect("a session");
    let html = body(r).await;
    let codes: Vec<String> = html
        .split("<code class=\"recovery-code\">")
        .skip(1)
        .map(|s| s.split('<').next().unwrap().to_string())
        .collect();
    (session, codes)
}

/// Password, then the login page's ceremony, then the key: the response of
/// `POST /auth/mfa/passkey`.
async fn login_with_key(
    app: &Router,
    pool: &SqlitePool,
    user: &User,
    key: &mut SoftKey,
) -> Response {
    let token = challenge_token(pool, user, false).await;
    let (_, response) = assert_on_login_page(app, &token, key, user).await;
    response
}

/// Opens the login page of `token`, answers its ceremony with `key`.
async fn assert_on_login_page(
    app: &Router,
    token: &str,
    key: &mut SoftKey,
    user: &User,
) -> (String, Response) {
    let page = request(app, "GET", "/auth/mfa", &challenge_cookies(token), "").await;
    let html = body(page).await;
    let options = page_options(&html).expect("the login page offers the passkey");
    let asserted = key.assert(&challenge_of(&options));
    let fields = format!(
        "_csrf=test&{}",
        assertion_fields(&asserted, &key.credential_id, &user_handle(&user.id))
    );
    let r = request(
        app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(token),
        &fields,
    )
    .await;
    (fields, r)
}

fn new_session(response: &Response) -> Option<String> {
    response_cookie(response, mfa::SESSION_COOKIE).filter(|s| !s.is_empty())
}

async fn require_mfa(pool: &SqlitePool) {
    sqlx::query("UPDATE auth_config SET mfa_required = 1")
        .execute(pool)
        .await
        .unwrap();
}

/// A TOTP row for `user`, as an enrolment would store it; returns the hex
/// secret.
async fn add_totp(pool: &SqlitePool, user: &User) -> String {
    let secret = hex::encode([7u8; 20]);
    sqlx::query("INSERT INTO user_mfa (user_id, secret_enc, last_step) VALUES (?, ?, 0)")
        .bind(&user.id)
        .bind(crypto::encrypt_value(&KEY, &secret).unwrap())
        .execute(pool)
        .await
        .unwrap();
    secret
}

fn totp_now(secret: &str) -> String {
    totp_rs::Builder::new()
        .with_secret(hex::decode(secret).unwrap())
        .with_account_name("user")
        .with_issuer(Some("calrs"))
        .build()
        .unwrap()
        .generate(Utc::now().timestamp() as u64)
        .to_string()
}

/// The confirm page for `query` under `session`, and its ceremony.
async fn confirm_page(
    app: &Router,
    session: &str,
    query: &str,
) -> (u16, Option<serde_json::Value>, String) {
    let r = request(
        app,
        "GET",
        &format!("/dashboard/settings/mfa/confirm?{query}"),
        &session_cookies(session),
        "",
    )
    .await;
    let status = r.status().as_u16();
    let html = body(r).await;
    (status, page_options(&html), html)
}

/// Confirms `action` (with `extra` fields) with `key` on the confirm page
/// and posts it to `to`.
async fn confirm_with_key(
    app: &Router,
    session: &str,
    user: &User,
    key: &mut SoftKey,
    query: &str,
    to: &str,
    extra: &str,
) -> Response {
    let (status, options, _) = confirm_page(app, session, query).await;
    assert_eq!(status, 200, "{query}");
    let options = options.expect("the confirm page offers the passkey");
    let asserted = key.assert(&challenge_of(&options));
    let fields = format!(
        "_csrf=test&password={PASSWORD}&{extra}&{}",
        assertion_fields(&asserted, &key.credential_id, &user_handle(&user.id))
    );
    request(app, "POST", to, &session_cookies(session), &fields).await
}

// --- The RP, the options, the user handle ---

#[test]
fn rp_comes_from_an_https_base_url_with_a_host_name() {
    let ok = |raw: &str, id: &str, origin: &str| {
        assert_eq!(
            Rp::from_base_url(raw),
            Some(Rp {
                id: id.to_string(),
                origin: origin.to_string()
            }),
            "{raw}"
        );
    };
    ok(
        "https://book.ilia.ae",
        "book.ilia.ae",
        "https://book.ilia.ae",
    );
    ok(
        "https://book.ilia.ae/",
        "book.ilia.ae",
        "https://book.ilia.ae",
    );
    ok(
        " https://Book.Ilia.AE/x/y ",
        "book.ilia.ae",
        "https://book.ilia.ae",
    );
    ok(
        "https://h.example:8443/x",
        "h.example",
        "https://h.example:8443",
    );
    ok("https://h.example:443", "h.example", "https://h.example");
    ok(
        "http://localhost:3000",
        "localhost",
        "http://localhost:3000",
    );
    ok(
        "https://bücher.example",
        "xn--bcher-kva.example",
        "https://xn--bcher-kva.example",
    );
    for refused in [
        "",
        "garbage",
        "http://book.ilia.ae",
        "https://1.2.3.4",
        "https://[::1]",
        "http://127.0.0.1:3000",
        "https://u:p@book.ilia.ae",
        "https://u@book.ilia.ae",
        "ftp://book.ilia.ae",
        "https://book.ilia.ae.",
    ] {
        assert_eq!(Rp::from_base_url(refused), None, "{refused}");
    }
    // Tests never read the environment.
    assert_eq!(Rp::from_env(), None);
}

#[test]
fn user_handle_is_stable_and_carries_no_personal_data() {
    let a = user_handle("6f1c1c7e-0000-4000-8000-000000000001");
    assert_eq!(a.len(), 16);
    assert_eq!(a, user_handle("6f1c1c7e-0000-4000-8000-000000000001"));
    assert_ne!(a, user_handle("6f1c1c7e-0000-4000-8000-000000000002"));
}

#[test]
fn options_json_shape() {
    let key = [1u8, 2, 3];
    let create = creation_options(
        &rp(),
        "user-1",
        "a@example.test",
        "A",
        &[9; 32],
        &[Descriptor {
            credential_id: &key,
            transports: r#"["usb","bogus"]"#,
        }],
    );
    let algs: Vec<i64> = create["pubKeyCredParams"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["alg"].as_i64().unwrap())
        .collect();
    // ES256 first: see `verify::ALGORITHMS`.
    assert_eq!(algs, vec![-7, -8, -257]);
    assert_eq!(create["attestation"], "none");
    assert_eq!(
        create["authenticatorSelection"]["userVerification"],
        "preferred"
    );
    assert_eq!(
        create["authenticatorSelection"]["residentKey"],
        "discouraged"
    );
    assert_eq!(create["rp"]["id"], RP_ID);
    assert_eq!(create["user"]["id"], b64url(&user_handle("user-1")));
    assert_eq!(create["timeout"], 300_000);
    assert_eq!(create["excludeCredentials"][0]["id"], b64url(&key));
    assert_eq!(
        create["excludeCredentials"][0]["transports"],
        serde_json::json!(["usb"])
    );
    let get = request_options(&rp(), &[9; 32], &[]);
    assert_eq!(get["rpId"], RP_ID);
    assert_eq!(get["userVerification"], "preferred");
    assert_eq!(get["challenge"], b64url(&[9; 32]));
}

#[test]
fn key_names_are_trimmed_short_and_printable() {
    assert_eq!(clean_name("  YubiKey 5C  ").as_deref(), Some("YubiKey 5C"));
    assert_eq!(
        clean_name(&"ж".repeat(64)).map(|n| n.chars().count()),
        Some(64)
    );
    assert_eq!(clean_name(&"ж".repeat(65)), None);
    assert_eq!(clean_name("   "), None);
    assert_eq!(clean_name("a\u{7}b"), None);
    assert_eq!(clean_name("a\nb"), None);
}

// --- Migration ---

/// The recovery-code table is rebuilt to belong to the user: rows made
/// before `ilia_010` survive, and still log in.
#[tokio::test]
async fn recovery_codes_survive_the_table_rebuild() {
    let pool = migrated().await;
    // Undo ilia_010, as a database from before it looks.
    sqlx::raw_sql(
        "DROP TABLE ilia_webauthn_ceremonies; DROP TABLE ilia_webauthn_credentials;
         DROP TABLE mfa_recovery_codes;
         CREATE TABLE mfa_recovery_codes (
             user_id TEXT NOT NULL REFERENCES user_mfa(user_id) ON DELETE CASCADE,
             code_hash TEXT NOT NULL, PRIMARY KEY (user_id, code_hash));
         DELETE FROM _migrations WHERE name = 'ilia_010_webauthn';",
    )
    .execute(&pool)
    .await
    .unwrap();
    let alice = user(&pool, "alice@example.test").await;
    add_totp(&pool, &alice).await;
    let code = "0123456789abcdef0123456789abcdef";
    sqlx::query("INSERT INTO mfa_recovery_codes (user_id, code_hash) VALUES (?, ?)")
        .bind(&alice.id)
        .bind(mfa::hash(code))
        .execute(&pool)
        .await
        .unwrap();
    crate::db::migrate(&pool).await.unwrap();
    assert_eq!(codes_of(&pool, &alice.id).await, 1);
    // The TOTP row can go without taking the codes (the code removes them
    // with the last factor instead).
    let fk: String =
        sqlx::query_scalar("SELECT \"table\" FROM pragma_foreign_key_list('mfa_recovery_codes')")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(fk, "users");
    let app = app(&pool).await;
    let token = challenge_token(&pool, &alice, false).await;
    let r = request(
        &app,
        "POST",
        "/auth/mfa",
        &challenge_cookies(&token),
        &format!("_csrf=test&code={code}"),
    )
    .await;
    assert_eq!(r.headers()["location"], "/dashboard");
}

// --- Key-only accounts ---

/// The owner's main scenario: a passkey as the only factor, from the first
/// enrolment to a later login, with MFA required.
#[tokio::test]
async fn key_only_user_enrols_and_logs_in_with_a_passkey() {
    let pool = migrated().await;
    let alice = user(&pool, "alice@example.test").await;
    let old = auth::create_session(&pool, &alice.id).await.unwrap();
    let app = app(&pool).await;
    let key = SoftKey::es256(1);
    let (session, codes) = enroll_key(&app, &pool, &alice, &key).await;
    assert_eq!(codes.len(), 10, "a first factor brings recovery codes");
    assert_eq!(codes_of(&pool, &alice.id).await, 10);
    assert!(
        auth::validate_session(&pool, &old.id).await.is_none(),
        "other sessions end"
    );
    let (name, transports, rp_id): (String, String, String) = sqlx::query_as(
        "SELECT name, transports, rp_id FROM ilia_webauthn_credentials WHERE user_id = ?",
    )
    .bind(&alice.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        name, "Security key 1",
        "a device-bound key gets the default name"
    );
    assert_eq!(
        transports, r#"["usb","nfc"]"#,
        "unknown transports are dropped"
    );
    assert_eq!(rp_id, RP_ID);
    assert!(mfa::enrolled(&pool, &alice.id).await.unwrap());

    require_mfa(&pool).await;
    assert!(
        auth::validate_session(&pool, &session).await.is_some(),
        "a verified session of a key-only user passes the required policy"
    );

    // A later login: password, then the key; the page offers the key first
    // and a recovery code, and no authenticator code.
    let r = request(
        &app,
        "POST",
        "/auth/login",
        CSRF,
        &format!("_csrf=test&email=alice%40example.test&password={PASSWORD}"),
    )
    .await;
    assert_eq!(r.headers()["location"], "/auth/mfa");
    let token = response_cookie(&r, mfa::CHALLENGE_COOKIE).unwrap();
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    assert!(html.contains("Sign in with passkey"));
    assert!(html.contains("href=\"/auth/mfa?method=recovery\""));
    assert!(!html.contains("href=\"/auth/mfa?method=totp\""));
    assert!(html.contains("<script src=\"/static/webauthn.js\" defer>"));
    assert!(
        !html.contains("data-webauthn=\"get\" onclick"),
        "no inline handlers"
    );
    let options = page_options(&html).unwrap();
    assert_eq!(options["rpId"], RP_ID);
    assert_eq!(
        options["allowCredentials"][0]["id"],
        b64url(&key.credential_id)
    );
    let mut key = key;
    let (_, r) = assert_on_login_page(&app, &token, &mut key, &alice).await;
    assert_eq!(r.status(), 303);
    assert_eq!(r.headers()["location"], "/dashboard");
    assert_eq!(
        response_cookie(&r, mfa::CHALLENGE_COOKIE).as_deref(),
        Some("")
    );
    let session = new_session(&r).unwrap();
    assert!(auth::validate_session(&pool, &session).await.is_some());
    let (count, last_used): (i64, Option<i64>) = sqlx::query_as(
        "SELECT sign_count, last_used_at FROM ilia_webauthn_credentials WHERE user_id = ?",
    )
    .bind(&alice.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
    assert!(last_used.is_some());
    assert_eq!(
        count_ceremonies(&pool).await,
        0,
        "the ceremony and the challenge are gone"
    );

    // A recovery code works without TOTP, and only once.
    let token = challenge_token(&pool, &alice, false).await;
    let html = body(
        request(
            &app,
            "GET",
            "/auth/mfa?method=recovery",
            &challenge_cookies(&token),
            "",
        )
        .await,
    )
    .await;
    assert!(html.contains("Recovery code"));
    assert!(html.contains("href=\"/auth/mfa?method=passkey\""));
    let fields = format!("_csrf=test&method=recovery&code={}", codes[0]);
    let r = request(
        &app,
        "POST",
        "/auth/mfa",
        &challenge_cookies(&token),
        &fields,
    )
    .await;
    assert_eq!(r.headers()["location"], "/dashboard");
    let token = challenge_token(&pool, &alice, false).await;
    let r = request(
        &app,
        "POST",
        "/auth/mfa",
        &challenge_cookies(&token),
        &fields,
    )
    .await;
    assert!(new_session(&r).is_none());
}

async fn count_ceremonies(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM ilia_webauthn_ceremonies")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// A TOTP-only account sees upstream's page and flow, passkeys on or off.
#[tokio::test]
async fn totp_only_user_is_unchanged() {
    let pool = migrated().await;
    let bob = user(&pool, "bob@example.test").await;
    let secret = add_totp(&pool, &bob).await;
    for passkeys in [Some(rp()), None] {
        let app = app_with(&pool, passkeys).await;
        let token = challenge_token(&pool, &bob, false).await;
        let html =
            body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
        assert!(html.contains(
            "Enter a code from your authenticator app, or use one of your saved recovery codes."
        ));
        assert!(page_options(&html).is_none());
        assert!(!html.contains("method=passkey"));
        assert_eq!(count_ceremonies(&pool).await, 0);
        sqlx::query("UPDATE user_mfa SET last_step = 0 WHERE user_id = ?")
            .bind(&bob.id)
            .execute(&pool)
            .await
            .unwrap();
        let r = request(
            &app,
            "POST",
            "/auth/mfa",
            &challenge_cookies(&token),
            &format!("_csrf=test&code={}", totp_now(&secret)),
        )
        .await;
        assert_eq!(r.headers()["location"], "/dashboard");
    }
    // The first-factor page offers the passkey first and the app as an
    // alternative; with passkeys off it is upstream's QR page.
    let carol = user(&pool, "carol@example.test").await;
    let token = challenge_token(&pool, &carol, true).await;
    let app = app(&pool).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    assert!(page_options(&html).is_some());
    assert!(html.contains("href=\"/auth/mfa?method=totp\""));
    assert!(!html.contains("data:image/png;base64,"));
    let html = body(
        request(
            &app,
            "GET",
            "/auth/mfa?method=totp",
            &challenge_cookies(&token),
            "",
        )
        .await,
    )
    .await;
    assert!(html.contains("data:image/png;base64,"));
    assert!(html.contains("href=\"/auth/mfa?method=passkey\""));
    let off = app_with(&pool, None).await;
    let html = body(request(&off, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    assert!(html.contains("data:image/png;base64,"));
    assert!(!html.contains("method=passkey"));
}

/// TOTP and a key: the key is the default, the code is one click away,
/// and either signs in.
#[tokio::test]
async fn both_factors_offer_the_passkey_first() {
    let pool = migrated().await;
    let dana = user(&pool, "dana@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::ed25519(2);
    enroll_key(&app, &pool, &dana, &key).await;
    let secret = add_totp(&pool, &dana).await;
    let token = challenge_token(&pool, &dana, false).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    assert!(page_options(&html).is_some());
    assert!(html.contains("href=\"/auth/mfa?method=totp\""));
    assert!(html.contains("href=\"/auth/mfa?method=recovery\""));
    let html = body(
        request(
            &app,
            "GET",
            "/auth/mfa?method=totp",
            &challenge_cookies(&token),
            "",
        )
        .await,
    )
    .await;
    assert!(page_options(&html).is_none());
    assert!(html.contains("Enter the six-digit code from your authenticator app."));
    assert!(html.contains("href=\"/auth/mfa?method=passkey\""));
    let r = request(
        &app,
        "POST",
        "/auth/mfa",
        &challenge_cookies(&token),
        &format!("_csrf=test&method=totp&code={}", totp_now(&secret)),
    )
    .await;
    assert_eq!(r.headers()["location"], "/dashboard");
    let r = login_with_key(&app, &pool, &dana, &mut key).await;
    assert_eq!(r.headers()["location"], "/dashboard");
}

// --- Ceremonies: single use, expiry, binding ---

#[tokio::test]
async fn an_assertion_is_single_use_and_a_failure_burns_the_ceremony() {
    let pool = migrated().await;
    let erin = user(&pool, "erin@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::es256(3);
    enroll_key(&app, &pool, &erin, &key).await;

    // Replay: the same POST twice.
    let token = challenge_token(&pool, &erin, false).await;
    let (fields, r) = assert_on_login_page(&app, &token, &mut key, &erin).await;
    assert_eq!(r.status(), 303);
    let token = challenge_token(&pool, &erin, false).await;
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&token),
        &fields,
    )
    .await;
    assert!(new_session(&r).is_none(), "replayed on a new challenge");

    // A bad signature burns the ceremony: the right assertion afterwards,
    // for the same challenge, is refused too.
    let token = challenge_token(&pool, &erin, false).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let challenge = challenge_of(&page_options(&html).unwrap());
    let good = key.assert(&challenge);
    let mut bad = softkey::Asserted {
        client_data: good.client_data.clone(),
        auth_data: good.auth_data.clone(),
        signature: good.signature.clone(),
    };
    *bad.signature.last_mut().unwrap() ^= 1;
    let credential_id = key.credential_id.clone();
    let post = |a: &softkey::Asserted| {
        format!(
            "_csrf=test&{}",
            assertion_fields(a, &credential_id, &user_handle(&erin.id))
        )
    };
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&token),
        &post(&bad),
    )
    .await;
    assert_eq!(r.status(), 200);
    assert!(new_session(&r).is_none());
    let html = body(r).await;
    assert!(
        page_options(&html).is_some(),
        "the page offers a fresh ceremony"
    );
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&token),
        &post(&good),
    )
    .await;
    assert!(new_session(&r).is_none(), "the burnt ceremony stays burnt");

    // Expired.
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let fresh = key.assert(&challenge_of(&page_options(&html).unwrap()));
    sqlx::query("UPDATE ilia_webauthn_ceremonies SET expires_at = 0")
        .execute(&pool)
        .await
        .unwrap();
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&token),
        &post(&fresh),
    )
    .await;
    assert!(new_session(&r).is_none(), "expired");

    // Only the latest ceremony of a challenge survives.
    let first = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let first = key.assert(&challenge_of(&page_options(&first).unwrap()));
    let second =
        body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let second = key.assert(&challenge_of(&page_options(&second).unwrap()));
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&token),
        &post(&first),
    )
    .await;
    assert!(new_session(&r).is_none(), "superseded");
    assert_eq!(
        count_ceremonies(&pool).await,
        1,
        "the failure offered a new one"
    );
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let last = key.assert(&challenge_of(&page_options(&html).unwrap()));
    let _ = second;
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&token),
        &post(&last),
    )
    .await;
    assert_eq!(r.status(), 303);
}

/// Someone else's ceremony, someone else's key, a user handle that is not
/// this user's: all refused.
#[tokio::test]
async fn foreign_ceremonies_credentials_and_handles_are_refused() {
    let pool = migrated().await;
    let alice = user(&pool, "alice@example.test").await;
    let mallory = user(&pool, "mallory@example.test").await;
    let app = app(&pool).await;
    let mut alice_key = SoftKey::es256(4);
    let mut mallory_key = SoftKey::es256(5);
    enroll_key(&app, &pool, &alice, &alice_key).await;
    enroll_key(&app, &pool, &mallory, &mallory_key).await;

    // Alice's ceremony answered under Mallory's challenge, with Mallory's key.
    let alice_token = challenge_token(&pool, &alice, false).await;
    let html = body(
        request(
            &app,
            "GET",
            "/auth/mfa",
            &challenge_cookies(&alice_token),
            "",
        )
        .await,
    )
    .await;
    let alice_challenge = challenge_of(&page_options(&html).unwrap());
    let mallory_token = challenge_token(&pool, &mallory, false).await;
    let a = mallory_key.assert(&alice_challenge);
    let fields = format!(
        "_csrf=test&{}",
        assertion_fields(&a, &mallory_key.credential_id, &user_handle(&mallory.id))
    );
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&mallory_token),
        &fields,
    )
    .await;
    assert!(new_session(&r).is_none(), "another login's ceremony");

    // Mallory's key on Alice's ceremony, under Alice's challenge.
    let html = body(
        request(
            &app,
            "GET",
            "/auth/mfa",
            &challenge_cookies(&alice_token),
            "",
        )
        .await,
    )
    .await;
    let a = mallory_key.assert(&challenge_of(&page_options(&html).unwrap()));
    let fields = format!(
        "_csrf=test&{}",
        assertion_fields(&a, &mallory_key.credential_id, &user_handle(&mallory.id))
    );
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&alice_token),
        &fields,
    )
    .await;
    assert!(new_session(&r).is_none(), "another user's credential");

    // Alice's key with Mallory's user handle.
    let html = body(
        request(
            &app,
            "GET",
            "/auth/mfa",
            &challenge_cookies(&alice_token),
            "",
        )
        .await,
    )
    .await;
    let a = alice_key.assert(&challenge_of(&page_options(&html).unwrap()));
    let fields = format!(
        "_csrf=test&{}",
        assertion_fields(&a, &alice_key.credential_id, &user_handle(&mallory.id))
    );
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&alice_token),
        &fields,
    )
    .await;
    assert!(new_session(&r).is_none(), "user handle mismatch");

    // No CSRF token: refused before anything.
    let html = body(
        request(
            &app,
            "GET",
            "/auth/mfa",
            &challenge_cookies(&alice_token),
            "",
        )
        .await,
    )
    .await;
    let a = alice_key.assert(&challenge_of(&page_options(&html).unwrap()));
    let fields = assertion_fields(&a, &alice_key.credential_id, &[]);
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&alice_token),
        &fields,
    )
    .await;
    assert_eq!(r.status(), 403);
    // The same, with the token and an empty user handle (allowed).
    let fields = format!("_csrf=test&{fields}");
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&alice_token),
        &fields,
    )
    .await;
    assert_eq!(r.status(), 303);
}

/// Two tabs post the same assertion at once: one session, not two.
#[tokio::test]
async fn parallel_assertions_give_one_session() {
    let dir = std::env::temp_dir().join(format!("calrs-passkeys-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(
            SqliteConnectOptions::from_str(&format!("sqlite://{}", dir.join("db").display()))
                .unwrap()
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .busy_timeout(std::time::Duration::from_secs(10))
                .foreign_keys(true),
        )
        .await
        .unwrap();
    crate::db::migrate(&pool).await.unwrap();
    let frank = user(&pool, "frank@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::es256(6);
    enroll_key(&app, &pool, &frank, &key).await;
    let before: i64 = count(
        &pool,
        "SELECT COUNT(*) FROM sessions WHERE user_id = ?",
        &frank.id,
    )
    .await;
    let token = challenge_token(&pool, &frank, false).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let a = key.assert(&challenge_of(&page_options(&html).unwrap()));
    let fields = format!(
        "_csrf=test&{}",
        assertion_fields(&a, &key.credential_id, &user_handle(&frank.id))
    );
    let cookies = challenge_cookies(&token);
    let (r1, r2) = tokio::join!(
        request(&app, "POST", "/auth/mfa/passkey", &cookies, &fields),
        request(&app, "POST", "/auth/mfa/passkey", &cookies, &fields),
    );
    let won = [&r1, &r2]
        .iter()
        .filter(|r| new_session(r).is_some())
        .count();
    assert_eq!(won, 1);
    let after: i64 = count(
        &pool,
        "SELECT COUNT(*) FROM sessions WHERE user_id = ?",
        &frank.id,
    )
    .await;
    assert_eq!(after, before + 1);
    pool.close().await;
    std::fs::remove_dir_all(dir).unwrap();
}

/// §6.1.1: a counter that does not grow is refused (a cloned key, maybe);
/// a synced passkey's 0/0 is fine every time.
#[tokio::test]
async fn counter_rules() {
    let pool = migrated().await;
    let gina = user(&pool, "gina@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::es256(7);
    enroll_key(&app, &pool, &gina, &key).await;
    assert_eq!(
        login_with_key(&app, &pool, &gina, &mut key).await.status(),
        303
    );
    // The authenticator's counter falls behind the stored one.
    sqlx::query("UPDATE ilia_webauthn_credentials SET sign_count = 100 WHERE user_id = ?")
        .bind(&gina.id)
        .execute(&pool)
        .await
        .unwrap();
    let r = login_with_key(&app, &pool, &gina, &mut key).await;
    assert!(new_session(&r).is_none(), "counter regression refused");
    let stored: i64 = count(
        &pool,
        "SELECT sign_count FROM ilia_webauthn_credentials WHERE user_id = ?",
        &gina.id,
    )
    .await;
    assert_eq!(stored, 100, "the key is not changed, nor disabled");
    key.counter = 100;
    let r = login_with_key(&app, &pool, &gina, &mut key).await;
    assert!(new_session(&r).is_some(), "101 > 100");

    // A synced passkey: BE and BS, counter always 0.
    let hana = user(&pool, "hana@example.test").await;
    let mut synced = SoftKey::ed25519(8);
    synced.flags = UV | BE | BS;
    synced.counting = false;
    enroll_key(&app, &pool, &hana, &synced).await;
    let (name, be, bs): (String, bool, bool) = sqlx::query_as(
        "SELECT name, backup_eligible, backup_state FROM ilia_webauthn_credentials WHERE user_id = ?",
    )
    .bind(&hana.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(name, "Passkey 1");
    assert!(be && bs);
    for _ in 0..2 {
        let r = login_with_key(&app, &pool, &hana, &mut synced).await;
        assert!(new_session(&r).is_some(), "0/0 accepted");
    }
}

/// A key's signature cannot be guessed, so a login with it takes nothing
/// from the wrong-code allowance and works while codes must wait.
#[tokio::test]
async fn passkey_login_ignores_the_wrong_code_budget() {
    let pool = migrated().await;
    let ivan = user(&pool, "ivan@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::rs256(9);
    enroll_key(&app, &pool, &ivan, &key).await;
    sqlx::query("INSERT INTO mfa_attempts (user_id, window_start, attempts) VALUES (?, ?, 50)")
        .bind(&ivan.id)
        .bind(Utc::now().timestamp())
        .execute(&pool)
        .await
        .unwrap();
    let r = login_with_key(&app, &pool, &ivan, &mut key).await;
    assert!(new_session(&r).is_some());
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM mfa_attempts WHERE user_id = ?",
            &ivan.id
        )
        .await,
        0,
        "a success clears the count"
    );
}

// --- Enrolment ---

#[tokio::test]
async fn enrolment_refuses_duplicates_and_a_second_first_factor() {
    let pool = migrated().await;
    let alice = user(&pool, "alice@example.test").await;
    let bob = user(&pool, "bob@example.test").await;
    let app = app(&pool).await;
    let key = SoftKey::es256(10);
    enroll_key(&app, &pool, &alice, &key).await;

    // Bob registers a key with Alice's credential ID.
    let token = challenge_token(&pool, &bob, true).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let registered = key.register(&challenge_of(&page_options(&html).unwrap()));
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey/enroll",
        &challenge_cookies(&token),
        &format!("_csrf=test&{}", registration_fields(&registered, "Mine")),
    )
    .await;
    assert!(new_session(&r).is_none());
    assert_eq!(keys_of(&pool, &bob.id).await, 0);
    assert_eq!(codes_of(&pool, &bob.id).await, 0);

    // Bob finishes with the authenticator app in another tab; the passkey
    // tab is then refused.
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let other = SoftKey::es256(11).register(&challenge_of(&page_options(&html).unwrap()));
    add_totp(&pool, &bob).await;
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey/enroll",
        &challenge_cookies(&token),
        &format!("_csrf=test&{}", registration_fields(&other, "")),
    )
    .await;
    assert!(new_session(&r).is_none());
    assert_eq!(keys_of(&pool, &bob.id).await, 0);

    // A name with a control character is refused.
    let carol = user(&pool, "carol@example.test").await;
    let token = challenge_token(&pool, &carol, true).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let registered = SoftKey::es256(12).register(&challenge_of(&page_options(&html).unwrap()));
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey/enroll",
        &challenge_cookies(&token),
        &format!("_csrf=test&{}", registration_fields(&registered, "a\u{1}b")),
    )
    .await;
    assert!(new_session(&r).is_none());
}

/// A key registered for another site address is not offered, but still
/// counts as a factor (fail closed): the user signs in with a recovery
/// code, and the settings say why the key does not work.
#[tokio::test]
async fn a_key_for_another_rp_id_counts_but_is_not_offered() {
    let pool = migrated().await;
    let jane = user(&pool, "jane@example.test").await;
    let app = app(&pool).await;
    let (session, codes) = enroll_key(&app, &pool, &jane, &SoftKey::es256(13)).await;
    sqlx::query("UPDATE ilia_webauthn_credentials SET rp_id = 'old.example.test'")
        .execute(&pool)
        .await
        .unwrap();
    require_mfa(&pool).await;
    assert!(auth::validate_session(&pool, &session).await.is_some());
    let token = challenge_token(&pool, &jane, false).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    assert!(page_options(&html).is_none());
    assert!(html.contains("Recovery code"));
    let r = request(
        &app,
        "POST",
        "/auth/mfa",
        &challenge_cookies(&token),
        &format!("_csrf=test&method=recovery&code={}", codes[0]),
    )
    .await;
    let session = new_session(&r).unwrap();
    let html = body(
        request(
            &app,
            "GET",
            "/dashboard/settings/mfa",
            &session_cookies(&session),
            "",
        )
        .await,
    )
    .await;
    assert!(html.contains("Registered for old.example.test"));
}

// --- Management ---

/// Every change needs the password and a factor: no password, a wrong one,
/// no proof, a wrong code, or a key answering another session's ceremony
/// change nothing.
#[tokio::test]
async fn management_requires_password_and_a_factor() {
    let pool = migrated().await;
    let kate = user(&pool, "kate@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::es256(14);
    let (session, codes) = enroll_key(&app, &pool, &kate, &key).await;
    let id: String = sqlx::query_scalar("SELECT id FROM ilia_webauthn_credentials")
        .fetch_one(&pool)
        .await
        .unwrap();
    let rename = format!("action=passkey_rename&credential={id}&name=Renamed");
    let name = || async {
        sqlx::query_scalar::<_, String>("SELECT name FROM ilia_webauthn_credentials")
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    for fields in [
        format!("_csrf=test&{rename}&code={}", codes[0]),
        format!("_csrf=test&{rename}&password=wrong&code={}", codes[0]),
        format!("_csrf=test&{rename}&password={PASSWORD}"),
        format!("_csrf=test&{rename}&password={PASSWORD}&code=123456"),
    ] {
        let r = request(
            &app,
            "POST",
            "/dashboard/settings/mfa",
            &session_cookies(&session),
            &fields,
        )
        .await;
        assert!(new_session(&r).is_none(), "{fields}");
        assert_eq!(name().await, "Security key 1");
    }
    // A ceremony of another session of the same user.
    let other = auth::create_session(&pool, &kate.id).await.unwrap();
    sqlx::query("UPDATE sessions SET mfa_verified = 1 WHERE id = ?")
        .bind(&other.id)
        .execute(&pool)
        .await
        .unwrap();
    let (_, options, _) = confirm_page(
        &app,
        &other.id,
        &format!("action=passkey_rename&credential={id}"),
    )
    .await;
    let a = key.assert(&challenge_of(&options.unwrap()));
    let fields = format!(
        "_csrf=test&password={PASSWORD}&{rename}&{}",
        assertion_fields(&a, &key.credential_id, &user_handle(&kate.id))
    );
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa",
        &session_cookies(&session),
        &fields,
    )
    .await;
    assert!(new_session(&r).is_none());
    assert_eq!(name().await, "Security key 1");
    // With this session's own ceremony: renamed, other sessions end.
    let r = confirm_with_key(
        &app,
        &session,
        &kate,
        &mut key,
        &format!("action=passkey_rename&credential={id}"),
        "/dashboard/settings/mfa",
        &rename,
    )
    .await;
    assert_eq!(r.status(), 303);
    assert_eq!(r.headers()["location"], "/dashboard/settings/mfa");
    assert_eq!(name().await, "Renamed");
    let renewed = new_session(&r).unwrap();
    assert!(auth::validate_session(&pool, &session).await.is_none());
    assert!(auth::validate_session(&pool, &other.id).await.is_none());
    assert!(auth::validate_session(&pool, &renewed).await.is_some());
    // A foreign or unknown key is a 404 on the confirm page and changes
    // nothing on the POST.
    let (status, _, _) =
        confirm_page(&app, &renewed, "action=passkey_delete&credential=nope").await;
    assert_eq!(status, 404);
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa",
        &session_cookies(&renewed),
        &format!(
            "_csrf=test&action=passkey_delete&credential=nope&password={PASSWORD}&code={}",
            codes[1]
        ),
    )
    .await;
    assert_eq!(r.status(), 404);
    assert_eq!(keys_of(&pool, &kate.id).await, 1);
    assert_eq!(
        codes_of(&pool, &kate.id).await,
        10,
        "the code was not used up"
    );
}

/// Adding a key: "confirm it's you" with a code, then the register page
/// under a new session; the register ceremony works once.
#[tokio::test]
async fn adding_a_key_needs_a_confirmation_and_works_once() {
    let pool = migrated().await;
    let liam = user(&pool, "liam@example.test").await;
    let app = app(&pool).await;
    let first = SoftKey::es256(15);
    let (session, codes) = enroll_key(&app, &pool, &liam, &first).await;
    let html = body(
        request(
            &app,
            "GET",
            "/dashboard/settings/mfa",
            &session_cookies(&session),
            "",
        )
        .await,
    )
    .await;
    assert!(html.contains("Security key 1"));
    assert!(html.contains("Hardware key"));
    assert!(html.contains("10 recovery codes left"));
    assert!(html.contains("Add a second key"));
    assert!(html.contains(
        "href=\"/dashboard/settings/mfa/confirm?action=passkey_add&amp;kind=security-key\""
    ));
    let (status, _, html) =
        confirm_page(&app, &session, "action=passkey_add&method=recovery").await;
    assert_eq!(status, 200);
    assert!(html.contains("You are adding a passkey or security key."));
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa",
        &session_cookies(&session),
        &format!(
            "_csrf=test&action=passkey_add&password={PASSWORD}&code={}",
            codes[0]
        ),
    )
    .await;
    assert_eq!(r.status(), 200);
    let renewed = new_session(&r).unwrap();
    assert!(auth::validate_session(&pool, &session).await.is_none());
    let html = body(r).await;
    let options = page_options(&html).unwrap();
    assert_eq!(
        options["excludeCredentials"][0]["id"],
        b64url(&first.credential_id)
    );
    let second = SoftKey::ed25519(16);
    let registered = second.register(&challenge_of(&options));
    let fields = format!(
        "_csrf=test&{}",
        registration_fields(&registered, "  Backup key ")
    );
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa/passkeys",
        &session_cookies(&renewed),
        &fields,
    )
    .await;
    assert_eq!(r.status(), 303);
    assert_eq!(keys_of(&pool, &liam.id).await, 2);
    assert!(
        auth::validate_session(&pool, &renewed).await.is_some(),
        "this session stays"
    );
    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM ilia_webauthn_credentials ORDER BY created_at, name")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(names.contains(&"Backup key".to_string()));
    assert_eq!(
        codes_of(&pool, &liam.id).await,
        9,
        "no new codes for a second key"
    );
    // The same registration again: the ceremony is gone.
    let third = SoftKey::es256(17).register(&challenge_of(&options));
    let fields = format!("_csrf=test&{}", registration_fields(&third, ""));
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa/passkeys",
        &session_cookies(&renewed),
        &fields,
    )
    .await;
    assert_eq!(r.status(), 200);
    assert_eq!(keys_of(&pool, &liam.id).await, 2);
}

/// A password change ends a register ceremony even when the session
/// survives it.
#[tokio::test]
async fn a_register_ceremony_dies_with_a_password_change() {
    let pool = migrated().await;
    let mia = user(&pool, "mia@example.test").await;
    let app = app(&pool).await;
    let (session, codes) = enroll_key(&app, &pool, &mia, &SoftKey::es256(18)).await;
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa",
        &session_cookies(&session),
        &format!(
            "_csrf=test&action=passkey_add&password={PASSWORD}&code={}",
            codes[0]
        ),
    )
    .await;
    let renewed = new_session(&r).unwrap();
    let options = page_options(&body(r).await).unwrap();
    // The new hash in the ceremony's place: the session itself is kept.
    sqlx::query("UPDATE ilia_webauthn_ceremonies SET password_hash = 'old'")
        .execute(&pool)
        .await
        .unwrap();
    let registered = SoftKey::es256(19).register(&challenge_of(&options));
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa/passkeys",
        &session_cookies(&renewed),
        &format!("_csrf=test&{}", registration_fields(&registered, "")),
    )
    .await;
    assert_eq!(r.status(), 200);
    assert_eq!(keys_of(&pool, &mia.id).await, 1);
}

#[tokio::test]
async fn the_eleventh_key_is_refused() {
    let pool = migrated().await;
    let noah = user(&pool, "noah@example.test").await;
    let app = app(&pool).await;
    let (session, codes) = enroll_key(&app, &pool, &noah, &SoftKey::es256(20)).await;
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa",
        &session_cookies(&session),
        &format!(
            "_csrf=test&action=passkey_add&password={PASSWORD}&code={}",
            codes[0]
        ),
    )
    .await;
    let renewed = new_session(&r).unwrap();
    let options = page_options(&body(r).await).unwrap();
    // Eight more arrive in the meantime (another tab): ten in all.
    for i in 0..9u8 {
        sqlx::query(
            "INSERT INTO ilia_webauthn_credentials (id, user_id, rp_id, credential_id, public_key,
                 algorithm, sign_count, uv_initialized, backup_eligible, backup_state, name, created_at)
             VALUES (?, ?, ?, ?, x'00', -7, 0, 0, 0, 0, 'k', 0)",
        )
        .bind(format!("k{i}"))
        .bind(&noah.id)
        .bind(RP_ID)
        .bind(vec![b'k', i])
        .execute(&pool)
        .await
        .unwrap();
    }
    assert_eq!(keys_of(&pool, &noah.id).await, 10);
    let registered = SoftKey::es256(21).register(&challenge_of(&options));
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa/passkeys",
        &session_cookies(&renewed),
        &format!("_csrf=test&{}", registration_fields(&registered, "")),
    )
    .await;
    assert!(body(r)
        .await
        .contains("You have reached the maximum number of keys (10)."));
    assert_eq!(keys_of(&pool, &noah.id).await, 10);
    let (status, _, _) = confirm_page(&app, &renewed, "action=passkey_add").await;
    assert_eq!(status, 404);
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa",
        &session_cookies(&renewed),
        &format!(
            "_csrf=test&action=passkey_add&password={PASSWORD}&code={}",
            codes[1]
        ),
    )
    .await;
    assert!(body(r).await.contains("maximum number of keys"));
}

/// The last factor cannot go while MFA is required; under an optional
/// policy it can, and the recovery codes go with it. Removing a key ends
/// the other sessions.
#[tokio::test]
async fn the_last_factor_rule_and_removal() {
    let pool = migrated().await;
    let olga = user(&pool, "olga@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::es256(22);
    let (session, _) = enroll_key(&app, &pool, &olga, &key).await;
    let id: String = sqlx::query_scalar("SELECT id FROM ilia_webauthn_credentials")
        .fetch_one(&pool)
        .await
        .unwrap();
    let delete = format!("action=passkey_delete&credential={id}");
    require_mfa(&pool).await;
    let (_, options, html) = confirm_page(&app, &session, &delete).await;
    assert!(html.contains("your last sign-in method cannot be removed"));
    assert!(options.is_none(), "no form to confirm with");
    // Posted anyway: refused before any proof is looked at.
    let other = auth::create_session(&pool, &olga.id).await.unwrap();
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa",
        &session_cookies(&session),
        &format!("_csrf=test&{delete}&password={PASSWORD}&code=0123456789abcdef0123456789abcdef"),
    )
    .await;
    assert!(body(r)
        .await
        .contains("your last sign-in method cannot be removed"));
    assert_eq!(keys_of(&pool, &olga.id).await, 1);
    // TOTP too: the key may go, MFA stays on, sessions are revoked.
    add_totp(&pool, &olga).await;
    let r = confirm_with_key(
        &app,
        &session,
        &olga,
        &mut key,
        &delete,
        "/dashboard/settings/mfa",
        &delete,
    )
    .await;
    assert_eq!(r.status(), 303);
    assert_eq!(keys_of(&pool, &olga.id).await, 0);
    assert_eq!(
        codes_of(&pool, &olga.id).await,
        10,
        "a factor remains: codes kept"
    );
    assert!(auth::validate_session(&pool, &other.id).await.is_none());
    assert!(auth::validate_session(&pool, &new_session(&r).unwrap())
        .await
        .is_some());

    // Optional policy, key only: removing the key turns MFA off.
    sqlx::query("UPDATE auth_config SET mfa_required = 0")
        .execute(&pool)
        .await
        .unwrap();
    let pia = user(&pool, "pia@example.test").await;
    let mut key = SoftKey::es256(23);
    let (session, _) = enroll_key(&app, &pool, &pia, &key).await;
    let id: String =
        sqlx::query_scalar("SELECT id FROM ilia_webauthn_credentials WHERE user_id = ?")
            .bind(&pia.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let delete = format!("action=passkey_delete&credential={id}");
    let (_, _, html) = confirm_page(&app, &session, &delete).await;
    assert!(html.contains("This is your last method"));
    // A consequence to confirm, not a refusal: the amber note, not the red alert.
    assert!(html.contains(r#"<p class="warn" role="note">This is your last method"#));
    let r = confirm_with_key(
        &app,
        &session,
        &pia,
        &mut key,
        &delete,
        "/dashboard/settings/mfa",
        &delete,
    )
    .await;
    assert_eq!(r.status(), 303);
    assert_eq!(keys_of(&pool, &pia.id).await, 0);
    assert_eq!(
        codes_of(&pool, &pia.id).await,
        0,
        "no orphaned recovery codes"
    );
    assert!(!mfa::enrolled(&pool, &pia.id).await.unwrap());
    let plain = new_session(&r).unwrap();
    let verified: bool = sqlx::query_scalar("SELECT mfa_verified FROM sessions WHERE id = ?")
        .bind(&plain)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!verified);
}

/// Removing TOTP keeps the codes while a key remains; adding TOTP next to a
/// key keeps them too.
#[tokio::test]
async fn totp_next_to_keys_keeps_the_recovery_codes() {
    let pool = migrated().await;
    let rita = user(&pool, "rita@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::es256(24);
    let (session, _) = enroll_key(&app, &pool, &rita, &key).await;
    let r = confirm_with_key(
        &app,
        &session,
        &rita,
        &mut key,
        "action=totp_add",
        "/dashboard/settings/mfa",
        "action=totp_add",
    )
    .await;
    assert_eq!(r.headers()["location"], "/auth/mfa");
    let token = response_cookie(&r, mfa::CHALLENGE_COOKIE).unwrap();
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    assert!(
        html.contains("data:image/png;base64,"),
        "the QR only, no passkey choice"
    );
    assert!(page_options(&html).is_none());
    let encrypted: String =
        sqlx::query_scalar("SELECT secret_enc FROM mfa_challenges WHERE user_id = ?")
            .bind(&rita.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let secret = crypto::decrypt_value(&KEY, &encrypted).unwrap();
    let r = request(
        &app,
        "POST",
        "/auth/mfa",
        &challenge_cookies(&token),
        &format!("_csrf=test&method=totp&code={}", totp_now(&secret)),
    )
    .await;
    assert_eq!(r.headers()["location"], "/dashboard");
    assert_eq!(
        codes_of(&pool, &rita.id).await,
        10,
        "the existing codes are kept"
    );
    let session = new_session(&r).unwrap();
    let r = confirm_with_key(
        &app,
        &session,
        &rita,
        &mut key,
        "action=disable",
        "/dashboard/settings/mfa",
        "action=disable",
    )
    .await;
    assert_eq!(r.status(), 303);
    let has_totp: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_mfa WHERE user_id = ?)")
            .bind(&rita.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!has_totp);
    assert_eq!(codes_of(&pool, &rita.id).await, 10);
}

/// `calrs user reset-mfa` removes TOTP, keys, codes, attempts and sessions.
#[tokio::test]
async fn reset_mfa_removes_keys_and_codes() {
    let pool = migrated().await;
    let sam = user(&pool, "sam@example.test").await;
    let app = app(&pool).await;
    let (session, _) = enroll_key(&app, &pool, &sam, &SoftKey::es256(25)).await;
    add_totp(&pool, &sam).await;
    mfa::reset(&pool, &sam.email).await.unwrap();
    assert_eq!(keys_of(&pool, &sam.id).await, 0);
    assert_eq!(codes_of(&pool, &sam.id).await, 0);
    assert!(!mfa::enrolled(&pool, &sam.id).await.unwrap());
    assert!(auth::validate_session(&pool, &session).await.is_none());
    require_mfa(&pool).await;
    // The next login enrols again.
    let token = challenge_token(&pool, &sam, false).await;
    let purpose: String =
        sqlx::query_scalar("SELECT purpose FROM mfa_challenges WHERE token_hash = ?")
            .bind(mfa::hash(&token))
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(purpose, "enroll");
}

/// The admin policy, confirmed with a key on the "confirm it's you" page.
#[tokio::test]
async fn the_policy_accepts_a_passkey() {
    let pool = migrated().await;
    let admin = user(&pool, "admin@example.test").await;
    let member = user(&pool, "member@example.test").await;
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT role FROM users WHERE id = ?")
            .bind(&admin.id)
            .fetch_one(&pool)
            .await
            .unwrap(),
        "admin"
    );
    let app = app(&pool).await;
    let mut key = SoftKey::es256(27);
    let (session, _) = enroll_key(&app, &pool, &admin, &key).await;
    let html = body(
        request(
            &app,
            "GET",
            "/dashboard/admin",
            &session_cookies(&session),
            "",
        )
        .await,
    )
    .await;
    assert!(html.contains("action=\"/dashboard/settings/mfa/confirm\""));
    let r = confirm_with_key(
        &app,
        &session,
        &admin,
        &mut key,
        "action=policy&policy=required",
        "/dashboard/admin/mfa",
        "policy=required",
    )
    .await;
    assert_eq!(r.headers()["location"], "/dashboard/admin");
    assert!(mfa::required(&pool).await.unwrap());
    assert!(auth::validate_session(&pool, &session).await.is_some());
    // A member cannot reach the policy confirmation.
    let (member_session, _) = enroll_key(&app, &pool, &member, &SoftKey::es256(28)).await;
    let (status, _, _) = confirm_page(&app, &member_session, "action=policy&policy=optional").await;
    assert_eq!(status, 404);
}

// --- Access and the feature switch ---

#[tokio::test]
async fn impersonation_and_sso_users_get_403() {
    let pool = migrated().await;
    let admin = user(&pool, "admin@example.test").await;
    let target = user(&pool, "target@example.test").await;
    let app = app(&pool).await;
    let (session, _) = enroll_key(&app, &pool, &admin, &SoftKey::es256(29)).await;
    let impersonating = format!(
        "{}; __Host-calrs_impersonate={}",
        session_cookies(&session),
        target.id
    );
    let r = request(
        &app,
        "GET",
        "/dashboard/settings/mfa/confirm?action=regenerate",
        &impersonating,
        "",
    )
    .await;
    assert_eq!(r.status(), 403);
    let r = request(
        &app,
        "POST",
        "/dashboard/settings/mfa/passkeys",
        &impersonating,
        "_csrf=test",
    )
    .await;
    assert_eq!(r.status(), 403);
    let sso = user(&pool, "sso@example.test").await;
    sqlx::query("UPDATE users SET auth_provider = 'oidc' WHERE id = ?")
        .bind(&sso.id)
        .execute(&pool)
        .await
        .unwrap();
    let sso_session = auth::create_session(&pool, &sso.id).await.unwrap();
    for (method, path) in [
        ("GET", "/dashboard/settings/mfa/confirm?action=regenerate"),
        ("POST", "/dashboard/settings/mfa/passkeys"),
        ("GET", "/dashboard/settings/mfa"),
    ] {
        let r = request(
            &app,
            method,
            path,
            &session_cookies(&sso_session.id),
            "_csrf=test",
        )
        .await;
        assert_eq!(r.status(), 403, "{method} {path}");
    }
}

/// Without a usable `CALRS_BASE_URL`: the ceremony routes and the script
/// are 404, no page offers a key, and keys already there are listed and
/// removable with a code.
#[tokio::test]
async fn feature_off_routes_404_and_keys_stay_removable() {
    let pool = migrated().await;
    let uma = user(&pool, "uma@example.test").await;
    let on = app(&pool).await;
    let (session, codes) = enroll_key(&on, &pool, &uma, &SoftKey::es256(30)).await;
    let off = app_with(&pool, None).await;
    for (method, path) in [
        ("POST", "/auth/mfa/passkey"),
        ("POST", "/auth/mfa/passkey/enroll"),
        ("POST", "/dashboard/settings/mfa/passkeys"),
        ("GET", "/static/webauthn.js"),
    ] {
        let r = request(&off, method, path, &session_cookies(&session), "_csrf=test").await;
        assert_eq!(r.status(), 404, "{method} {path}");
    }
    let token = challenge_token(&pool, &uma, false).await;
    let html = body(request(&off, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    assert!(page_options(&html).is_none());
    assert!(!html.contains("webauthn.js"));
    assert!(html.contains("Recovery code"));
    let r = request(
        &off,
        "POST",
        "/auth/mfa",
        &challenge_cookies(&token),
        &format!("_csrf=test&method=recovery&code={}", codes[0]),
    )
    .await;
    let session = new_session(&r).unwrap();
    let html = body(
        request(
            &off,
            "GET",
            "/dashboard/settings/mfa",
            &session_cookies(&session),
            "",
        )
        .await,
    )
    .await;
    assert!(html.contains("Passkeys are off"));
    assert!(html.contains("Security key 1"));
    assert!(!html.contains("action=passkey_add"));
    let id: String = sqlx::query_scalar("SELECT id FROM ilia_webauthn_credentials")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (status, options, html) = confirm_page(
        &off,
        &session,
        &format!("action=passkey_delete&credential={id}"),
    )
    .await;
    assert_eq!(status, 200);
    assert!(options.is_none());
    assert!(html.contains("name=\"code\""));
    let r = request(
        &off,
        "POST",
        "/dashboard/settings/mfa",
        &session_cookies(&session),
        &format!(
            "_csrf=test&action=passkey_delete&credential={id}&password={PASSWORD}&code={}",
            codes[1]
        ),
    )
    .await;
    assert_eq!(r.status(), 303);
    assert_eq!(keys_of(&pool, &uma.id).await, 0);
    assert_eq!(codes_of(&pool, &uma.id).await, 0);
}

#[tokio::test]
async fn the_script_is_same_origin_javascript_and_pages_keep_their_headers() {
    let pool = migrated().await;
    let app = app(&pool).await;
    let r = request(&app, "GET", "/static/webauthn.js", "", "").await;
    assert_eq!(r.status(), 200);
    assert_eq!(
        r.headers()["content-type"],
        "application/javascript; charset=utf-8"
    );
    assert_eq!(r.headers()["x-content-type-options"], "nosniff");
    assert!(r.headers().get("access-control-allow-origin").is_none());
    let js = body(r).await;
    assert!(js.contains("navigator.credentials.get"));
    assert!(!js.contains("fetch("), "plain form posts, no fetch");

    // A user name that tries to close the script tag.
    let vera = user_named(&pool, "vera@example.test", "</script><b>x</b>").await;
    let token = challenge_token(&pool, &vera, true).await;
    let r = request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await;
    assert_eq!(r.headers()["cache-control"], "no-store");
    assert_eq!(r.headers()["referrer-policy"], "no-referrer");
    let csp = r.headers()["content-security-policy"]
        .to_str()
        .unwrap()
        .to_string();
    let permissions = r
        .headers()
        .get("permissions-policy")
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let html = body(r).await;
    assert!(!html.contains("</script><b>"));
    assert_eq!(
        page_options(&html).unwrap()["user"]["displayName"],
        "</script><b>x</b>"
    );
    assert!(!csp.contains("unsafe-eval"));
    assert!(csp.contains("connect-src 'self'"));
    assert!(!permissions.contains("publickey-credentials"));
}

/// Revoking access (suspension, a password change, OIDC linking, any MFA
/// change) ends the pending ceremonies with their challenge.
#[tokio::test]
async fn ceremonies_end_when_access_is_revoked() {
    let pool = migrated().await;
    user(&pool, "first@example.test").await;
    let wes = user(&pool, "wes@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::es256(31);
    enroll_key(&app, &pool, &wes, &key).await;
    let token = challenge_token(&pool, &wes, false).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let a = key.assert(&challenge_of(&page_options(&html).unwrap()));
    assert_eq!(count_ceremonies(&pool).await, 1);
    let mut tx = pool.begin().await.unwrap();
    mfa::revoke_access(&mut tx, &wes.id).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        count_ceremonies(&pool).await,
        0,
        "revoked with the challenge"
    );
    let fields = format!(
        "_csrf=test&{}",
        assertion_fields(&a, &key.credential_id, &[])
    );
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&token),
        &fields,
    )
    .await;
    assert!(new_session(&r).is_none());
}

mod regressions;
mod ui;
