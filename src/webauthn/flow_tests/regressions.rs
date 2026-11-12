//! ilia fork: regressions from the review of the passkey work: the last
//! factor that works here, the policy while impersonating, the algorithm
//! order, the ceremony lifetime and the passkey refusal text, the browser
//! notices, and what the settings form may put into the log.

use std::sync::Mutex;

use super::*;

const LAST_FACTOR: &str = "your last sign-in method cannot be removed";
const PASSKEY_INVALID: &str = "The passkey or security key could not be verified";
const CODE_INVALID: &str = "Verification failed.";

async fn has_totp(pool: &SqlitePool, user_id: &str) -> bool {
    count(
        pool,
        "SELECT COUNT(*) FROM user_mfa WHERE user_id = ?",
        user_id,
    )
    .await
        == 1
}

async fn key_id(pool: &SqlitePool, user_id: &str) -> String {
    sqlx::query_scalar("SELECT id FROM ilia_webauthn_credentials WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Moves every key of `user_id` to another RP ID, as a change of
/// `CALRS_BASE_URL`'s host name leaves them.
async fn make_stale(pool: &SqlitePool, user_id: &str) {
    sqlx::query(
        "UPDATE ilia_webauthn_credentials SET rp_id = 'old.example.test' WHERE user_id = ?",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn post_settings(app: &Router, session: &str, fields: &str) -> Response {
    request(
        app,
        "POST",
        "/dashboard/settings/mfa",
        &session_cookies(session),
        &format!("_csrf=test&password={PASSWORD}&{fields}"),
    )
    .await
}

fn credential(id: &str, rp_id: &str) -> store::Credential {
    store::Credential {
        id: id.to_string(),
        rp_id: rp_id.to_string(),
        credential_id: id.as_bytes().to_vec(),
        public_key: Vec::new(),
        algorithm: verify::ALG_ES256,
        sign_count: 0,
        backup_eligible: false,
        backup_state: false,
        transports: "[]".to_string(),
        name: id.to_string(),
        created_at: 0,
        last_used_at: None,
    }
}

// --- The last factor that works here (finding: stale keys counted) ---

#[test]
fn removal_counts_only_keys_that_work_here() {
    use flows::{removal, Removal};
    let here = rp();
    let good = [credential("good", RP_ID)];
    let stale = [credential("stale", "old.example.test")];
    let on = Some(&here);
    // The app next to a key that works here may go; next to a stale key or
    // with passkeys off it is the last working method.
    assert_eq!(removal("disable", "", true, &good, on), Removal::Fine);
    assert_eq!(
        removal("disable", "", true, &stale, on),
        Removal::LastUsable
    );
    assert_eq!(
        removal("disable", "", true, &good, None),
        Removal::LastUsable
    );
    assert_eq!(removal("disable", "", true, &[], on), Removal::LastFactor);
    // A stale key may go while the app remains, and while another stale
    // key remains when nothing works here anyway.
    assert_eq!(
        removal("passkey_delete", "stale", true, &stale, on),
        Removal::Fine
    );
    let both = [good[0].clone(), stale[0].clone()];
    assert_eq!(
        removal("passkey_delete", "stale", false, &both, on),
        Removal::Fine
    );
    assert_eq!(
        removal("passkey_delete", "good", false, &both, on),
        Removal::LastUsable
    );
    let two_stale = [stale[0].clone(), credential("stale2", "old.example.test")];
    assert_eq!(
        removal("passkey_delete", "stale", false, &two_stale, on),
        Removal::Fine
    );
    assert_eq!(
        removal("passkey_delete", "stale", false, &stale, on),
        Removal::LastFactor
    );
    assert_eq!(
        removal("passkey_delete", "good", false, &good, None),
        Removal::LastFactor
    );
    // Other actions remove nothing.
    for action in ["regenerate", "totp_add", "passkey_add", "passkey_rename"] {
        assert_eq!(removal(action, "good", false, &good, on), Removal::Fine);
    }
}

/// MFA required, the app and a key registered for an old host name: the
/// app is the only method that works here, so it stays (no button, a
/// refusal on the confirm page and on a direct post); the stale key can
/// go. With passkeys off, a key for this very host does not count either.
#[tokio::test]
async fn a_key_that_does_not_work_here_does_not_let_the_app_go() {
    let pool = migrated().await;
    let tia = user(&pool, "tia@example.test").await;
    let app = app(&pool).await;
    let (session, _) = enroll_key(&app, &pool, &tia, &SoftKey::es256(60)).await;
    let secret = add_totp(&pool, &tia).await;
    make_stale(&pool, &tia.id).await;
    require_mfa(&pool).await;

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
    assert!(
        !html.contains("confirm?action=disable"),
        "no button to remove the only working method"
    );
    let (status, options, html) = confirm_page(&app, &session, "action=disable").await;
    assert_eq!(status, 200);
    assert!(html.contains(LAST_FACTOR), "{html}");
    assert!(options.is_none(), "no form to confirm with");
    let r = post_settings(
        &app,
        &session,
        &format!("action=disable&code={}", totp_now(&secret)),
    )
    .await;
    assert!(body(r).await.contains(LAST_FACTOR));
    assert!(has_totp(&pool, &tia.id).await, "the app stays");

    // The stale key may go: the app remains and works.
    let id = key_id(&pool, &tia.id).await;
    let delete = format!("action=passkey_delete&credential={id}");
    let (_, _, html) = confirm_page(&app, &session, &delete).await;
    assert!(!html.contains(LAST_FACTOR));
    let r = post_settings(
        &app,
        &session,
        &format!("{delete}&code={}", totp_now(&secret)),
    )
    .await;
    assert_eq!(r.status(), 303);
    assert_eq!(keys_of(&pool, &tia.id).await, 0);
    assert!(has_totp(&pool, &tia.id).await);

    // Passkeys off: a key for this host cannot sign in either.
    let uli = user(&pool, "uli@example.test").await;
    let (session, codes) = enroll_key(&app, &pool, &uli, &SoftKey::es256(61)).await;
    add_totp(&pool, &uli).await;
    let off = app_with(&pool, None).await;
    let (_, _, html) = confirm_page(&off, &session, "action=disable").await;
    assert!(html.contains(LAST_FACTOR));
    let r = post_settings(&off, &session, &format!("action=disable&code={}", codes[0])).await;
    assert!(body(r).await.contains(LAST_FACTOR));
    assert!(has_totp(&pool, &uli.id).await);
    // The same change is fine while passkeys are on.
    let (_, _, html) = confirm_page(&app, &session, "action=disable").await;
    assert!(!html.contains(LAST_FACTOR));
}

/// Key only, MFA required, one key that works and one stale: the working
/// one is the last way in besides the codes and stays; the stale one goes.
#[tokio::test]
async fn the_last_working_key_stays_while_a_stale_one_is_left() {
    let pool = migrated().await;
    let vic = user(&pool, "vic@example.test").await;
    let app = app(&pool).await;
    let (session, codes) = enroll_key(&app, &pool, &vic, &SoftKey::es256(62)).await;
    let good = key_id(&pool, &vic.id).await;
    sqlx::query(
        "INSERT INTO ilia_webauthn_credentials
             (id, user_id, rp_id, credential_id, public_key, algorithm, sign_count,
              uv_initialized, backup_eligible, backup_state, transports, aaguid, name,
              created_at, last_used_at)
         SELECT 'stale-key', user_id, 'old.example.test', ?, public_key,
              algorithm, sign_count, uv_initialized, backup_eligible, backup_state, transports,
              aaguid, 'Old key', created_at + 1, NULL
         FROM ilia_webauthn_credentials WHERE id = ?",
    )
    .bind(vec![0x5a_u8; 16])
    .bind(&good)
    .execute(&pool)
    .await
    .unwrap();
    require_mfa(&pool).await;
    let delete_good = format!("action=passkey_delete&credential={good}");
    let (_, _, html) = confirm_page(&app, &session, &delete_good).await;
    assert!(html.contains(LAST_FACTOR));
    let r = post_settings(&app, &session, &format!("{delete_good}&code={}", codes[0])).await;
    assert!(body(r).await.contains(LAST_FACTOR));
    assert_eq!(keys_of(&pool, &vic.id).await, 2);
    let r = post_settings(
        &app,
        &session,
        &format!(
            "action=passkey_delete&credential=stale-key&code={}",
            codes[0]
        ),
    )
    .await;
    assert_eq!(r.status(), 303);
    assert_eq!(keys_of(&pool, &vic.id).await, 1);
}

// --- The policy while impersonating ---

/// An admin who impersonates someone still sees the policy form on
/// `/dashboard/admin` (the real user's page) and can confirm the change on
/// the confirm page, as the real user; the per-user actions stay 403.
#[tokio::test]
async fn an_impersonating_admin_can_confirm_the_policy() {
    let pool = migrated().await;
    let admin = user(&pool, "admin@example.test").await;
    let target = user(&pool, "target@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::es256(63);
    let (session, _) = enroll_key(&app, &pool, &admin, &key).await;
    let cookies = format!(
        "{}; __Host-calrs_impersonate={}",
        session_cookies(&session),
        target.id
    );
    let html = body(request(&app, "GET", "/dashboard/admin", &cookies, "").await).await;
    assert!(html.contains("action=\"/dashboard/settings/mfa/confirm\""));
    let r = request(
        &app,
        "GET",
        "/dashboard/settings/mfa/confirm?action=policy&policy=required",
        &cookies,
        "",
    )
    .await;
    assert_eq!(r.status(), 200);
    let options = page_options(&body(r).await).expect("the admin's key is offered");
    let asserted = key.assert(&challenge_of(&options));
    let fields = format!(
        "_csrf=test&password={PASSWORD}&policy=required&{}",
        assertion_fields(&asserted, &key.credential_id, &user_handle(&admin.id))
    );
    let r = request(&app, "POST", "/dashboard/admin/mfa", &cookies, &fields).await;
    assert_eq!(r.headers()["location"], "/dashboard/admin");
    assert!(mfa::required(&pool).await.unwrap());
    for query in ["action=regenerate", "action=passkey_add", "action=disable"] {
        let r = request(
            &app,
            "GET",
            &format!("/dashboard/settings/mfa/confirm?{query}"),
            &cookies,
            "",
        )
        .await;
        assert_eq!(r.status(), 403, "{query}");
    }
    // A member impersonates no one: the cookie is ignored, and the policy
    // is still not theirs to confirm.
    let (member_session, _) = enroll_key(&app, &pool, &target, &SoftKey::es256(64)).await;
    let r = request(
        &app,
        "GET",
        "/dashboard/settings/mfa/confirm?action=policy&policy=optional",
        &format!(
            "{}; __Host-calrs_impersonate={}",
            session_cookies(&member_session),
            admin.id
        ),
        "",
    )
    .await;
    assert_eq!(r.status(), 404);
}

// --- Algorithms and the browser notices ---

/// ES256 leads, so a YubiKey 5 makes a P-256 key, which Safari's
/// `getPublicKey()` can return; Ed25519 and RS256 are still offered.
#[tokio::test]
async fn new_keys_are_asked_for_es256_first() {
    assert_eq!(verify::ALGORITHMS[0], verify::ALG_ES256);
    let pool = migrated().await;
    let wil = user(&pool, "wil@example.test").await;
    let app = app(&pool).await;
    let token = challenge_token(&pool, &wil, true).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let options = page_options(&html).unwrap();
    let algs: Vec<i64> = options["pubKeyCredParams"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["alg"].as_i64().unwrap())
        .collect();
    assert_eq!(algs, vec![-7, -8, -257]);
    // The browser could not read the key, or the page is on another host:
    // each has its own notice, which says what to do instead of "try again".
    assert!(
        html.contains("data-webauthn-key-type hidden>This browser cannot read this type of key")
    );
    // The origin is HTML-escaped (`/` as `&#x2f;`), so look for its host.
    let notice = &html[html.find("data-webauthn-wrong-site hidden>").unwrap()..];
    let notice = &notice[..notice.find("</p>").unwrap()];
    assert!(notice.contains("Passkeys work only at https:"), "{notice}");
    assert!(
        notice.contains(&format!("{RP_ID}. Open this page")),
        "{notice}"
    );
    assert!(ORIGIN.ends_with(RP_ID));
    let js = include_str!("../../../assets/webauthn.js");
    assert!(js.contains("'key-type'") && js.contains("'wrong-site'"));
    assert!(js.contains("name === 'SecurityError'"));
    // The login and confirm pages carry the wrong-site notice too.
    let (session, _) = enroll_key(&app, &pool, &wil, &SoftKey::es256(65)).await;
    let (_, _, html) = confirm_page(&app, &session, "action=regenerate").await;
    assert!(html.contains("data-webauthn-wrong-site hidden>"));
}

/// Every fork key the review added exists in every locale.
#[test]
fn the_new_notices_exist_in_every_locale() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("i18n");
    let mut seen = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let locale = path.file_name().unwrap().to_string_lossy().to_string();
        if locale == "de-formal" {
            continue; // an overlay on de
        }
        let text = std::fs::read_to_string(path.join("main.ftl")).unwrap();
        for key in [
            "ilia-passkey-invalid",
            "ilia-passkey-key-type",
            "ilia-passkey-wrong-site",
        ] {
            assert!(
                text.lines().any(|l| l.starts_with(&format!("{key} = "))),
                "{key} missing in {locale}"
            );
        }
        seen += 1;
    }
    assert_eq!(seen, 10);
}

// --- Ceremony lifetime and the passkey refusal text ---

/// Shifts every challenge and ceremony `seconds` into the past.
async fn age(pool: &SqlitePool, seconds: i64) {
    for table in ["mfa_challenges", "ilia_webauthn_ceremonies"] {
        sqlx::query(&format!("UPDATE {table} SET expires_at = expires_at - ?"))
            .bind(seconds)
            .execute(pool)
            .await
            .unwrap();
    }
}

/// The sign-in page says 10 minutes; a key pressed after 6 still signs
/// in, and the browser's own wait stays 5 minutes from the press.
#[tokio::test]
async fn a_key_pressed_after_six_minutes_still_signs_in() {
    let pool = migrated().await;
    let xan = user(&pool, "xan@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::es256(66);
    enroll_key(&app, &pool, &xan, &key).await;
    let token = challenge_token(&pool, &xan, false).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    assert!(html.contains("The sign-in expires after 10 minutes."));
    let options = page_options(&html).unwrap();
    assert_eq!(options["timeout"], 300_000);
    age(&pool, 6 * 60).await;
    let asserted = key.assert(&challenge_of(&options));
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&token),
        &format!(
            "_csrf=test&{}",
            assertion_fields(&asserted, &key.credential_id, &user_handle(&xan.id))
        ),
    )
    .await;
    assert!(new_session(&r).is_some(), "signed in after 6 minutes");
}

/// A refused key says so, not "check your password or code ... wait 15
/// minutes": at sign-in and in the settings.
#[tokio::test]
async fn a_refused_key_gets_the_passkey_message() {
    let pool = migrated().await;
    let yul = user(&pool, "yul@example.test").await;
    let app = app(&pool).await;
    let mut key = SoftKey::es256(67);
    let (session, _) = enroll_key(&app, &pool, &yul, &key).await;
    let token = challenge_token(&pool, &yul, false).await;
    let html = body(request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await).await;
    let asserted = key.assert(&challenge_of(&page_options(&html).unwrap()));
    // The request expired (or was used up) before the key answered.
    sqlx::query("DELETE FROM ilia_webauthn_ceremonies")
        .execute(&pool)
        .await
        .unwrap();
    let r = request(
        &app,
        "POST",
        "/auth/mfa/passkey",
        &challenge_cookies(&token),
        &format!(
            "_csrf=test&{}",
            assertion_fields(&asserted, &key.credential_id, &user_handle(&yul.id))
        ),
    )
    .await;
    assert!(new_session(&r).is_none());
    let html = body(r).await;
    assert!(html.contains(PASSKEY_INVALID), "{html}");
    assert!(!html.contains(CODE_INVALID));
    // A wrong code keeps the upstream message.
    let r = request(
        &app,
        "POST",
        "/auth/mfa",
        &challenge_cookies(&token),
        "_csrf=test&method=recovery&code=0123456789abcdef0123456789abcdef",
    )
    .await;
    let html = body(r).await;
    assert!(html.contains(CODE_INVALID));
    assert!(!html.contains(PASSKEY_INVALID));

    // Settings: the confirmation's ceremony is gone when the key answers.
    let (_, options, _) = confirm_page(&app, &session, "action=regenerate").await;
    let asserted = key.assert(&challenge_of(&options.unwrap()));
    sqlx::query("DELETE FROM ilia_webauthn_ceremonies")
        .execute(&pool)
        .await
        .unwrap();
    let r = post_settings(
        &app,
        &session,
        &format!(
            "action=regenerate&{}",
            assertion_fields(&asserted, &key.credential_id, &user_handle(&yul.id))
        ),
    )
    .await;
    let html = body(r).await;
    assert!(html.contains(PASSKEY_INVALID), "{html}");
    assert!(!html.contains(CODE_INVALID));
}

// --- What reaches the log and the confirm form ---

#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for Captured {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Captured {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

/// A newline posted in `credential` (with an action that does not use it)
/// or in `action` does not start a line of its own in the server's text
/// log, the format `calrs serve` writes.
#[tokio::test]
async fn posted_fields_cannot_forge_log_lines() {
    let pool = migrated().await;
    let zed = user(&pool, "zed@example.test").await;
    let app = app(&pool).await;
    let (session, codes) = enroll_key(&app, &pool, &zed, &SoftKey::es256(68)).await;
    let log = Captured::default();
    let writer = log.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);
    // A callsite first hit with no subscriber caches "never"; see
    // `capture_tracing` in src/web/tests/security.rs.
    tracing::callsite::rebuild_interest_cache();
    let forged = enc("x\nFORGED INFO calrs::mfa: MFA completed user_id=other");
    let r = post_settings(
        &app,
        &session,
        &format!("action=regenerate&credential={forged}&code={}", codes[0]),
    )
    .await;
    assert_eq!(r.status(), 200, "the change itself is made");
    let session = new_session(&r).unwrap();
    let r = post_settings(
        &app,
        &session,
        &format!("action={forged}&code={}", codes[1]),
    )
    .await;
    assert_eq!(r.status(), 200);
    drop(_guard);
    let text = log.text();
    if !cfg!(tarpaulin) {
        assert!(text.contains("MFA settings changed"), "{text}");
        assert!(text.contains("MFA settings refused"), "{text}");
    }
    assert!(
        !text.lines().any(|l| l.starts_with("FORGED")),
        "a forged line in:\n{text}"
    );

    // The confirm page puts back only what the action uses.
    let (status, _, html) = confirm_page(
        &app,
        &session,
        "action=regenerate&credential=abc&policy=required",
    )
    .await;
    assert_eq!(status, 200);
    assert!(!html.contains("name=\"credential\""), "{html}");
    assert!(!html.contains("credential=abc"));
    assert!(!html.contains("policy=required"));
    assert!(!html.contains("name=\"policy\""));
}
