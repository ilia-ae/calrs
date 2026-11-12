//! ilia fork: the passkey pages as a browser gets them (docs/passkeys-plan.md
//! §4.9, §6): which buttons each page shows and in what order, only while
//! passkeys are on; what the settings page lists; the script and the
//! headers. The ceremonies themselves are tested in `flow_tests`.

use super::*;

/// Where `needle` first appears in `html`; panics with the page when absent.
fn at(html: &str, needle: &str) -> usize {
    html.find(needle)
        .unwrap_or_else(|| panic!("{needle:?} missing from:\n{html}"))
}

/// No inline event handler, no `javascript:` URL.
fn no_inline_handlers(html: &str) {
    let lower = html.to_ascii_lowercase();
    for bad in [
        " onclick=",
        " onsubmit=",
        " onload=",
        " onerror=",
        " onchange=",
        " oninput=",
        "javascript:",
    ] {
        assert!(!lower.contains(bad), "{bad} in:\n{html}");
    }
}

async fn page(app: &Router, path: &str, cookies: &str) -> (u16, String) {
    let r = request(app, "GET", path, cookies, "").await;
    (r.status().as_u16(), body(r).await)
}

async fn settings(app: &Router, session: &str) -> String {
    let (status, html) = page(app, "/dashboard/settings/mfa", &session_cookies(session)).await;
    assert_eq!(status, 200);
    html
}

/// A verified session for a user who has TOTP only.
async fn totp_session(pool: &SqlitePool, user: &User) -> String {
    add_totp(pool, user).await;
    let mut tx = pool.begin().await.unwrap();
    let session = mfa::verified_session(&mut tx, &user.id).await.unwrap();
    tx.commit().await.unwrap();
    session.id
}

/// The login page leads with the big passkey button; the authenticator
/// code and the recovery code are secondary links that work without
/// JavaScript, and the code views link back to the key.
#[tokio::test]
async fn login_page_leads_with_the_passkey_button() {
    let pool = migrated().await;
    let ann = user(&pool, "ann@example.test").await;
    let app = app(&pool).await;
    enroll_key(&app, &pool, &ann, &SoftKey::es256(40)).await;
    add_totp(&pool, &ann).await;
    let token = challenge_token(&pool, &ann, false).await;
    let (status, html) = page(&app, "/auth/mfa", &challenge_cookies(&token)).await;
    assert_eq!(status, 200);
    let button = at(
        &html,
        r#"<button type="button" class="btn btn-passkey" data-webauthn="get" data-form="webauthn-form" hidden>Sign in with passkey</button>"#,
    );
    let totp = at(
        &html,
        r#"<a class="btn btn-secondary" href="/auth/mfa?method=totp">Use authenticator code</a>"#,
    );
    let recovery = at(
        &html,
        r#"<a class="btn btn-secondary" href="/auth/mfa?method=recovery">Use recovery code</a>"#,
    );
    assert!(
        button < totp && totp < recovery,
        "passkey first, then the codes"
    );
    assert!(html.contains(r#"<form method="post" action="/auth/mfa/passkey" id="webauthn-form">"#));
    assert!(html.contains("<noscript>"));
    assert!(html.contains("Passkeys need JavaScript"));
    assert!(html.contains("data-webauthn-unsupported hidden>This browser cannot use passkeys"));
    assert!(html.contains("data-webauthn-cancelled hidden>"));
    assert!(html.contains(r#"<script src="/static/webauthn.js" defer></script>"#));
    assert!(
        !html.contains(r#"name="code""#),
        "no code field on the key view"
    );
    no_inline_handlers(&html);

    // The code views: a code field, and the way back to the key.
    let (_, html) = page(&app, "/auth/mfa?method=totp", &challenge_cookies(&token)).await;
    assert!(html.contains(r#"name="code""#));
    assert!(html.contains("Six-digit authenticator code"));
    assert!(html.contains(r#"href="/auth/mfa?method=passkey">Use passkey</a>"#));
    assert!(html.contains(r#"href="/auth/mfa?method=recovery">Use recovery code</a>"#));
    assert!(!html.contains("data-webauthn="));
    assert!(!html.contains("webauthn.js"));
    let (_, html) = page(
        &app,
        "/auth/mfa?method=recovery",
        &challenge_cookies(&token),
    )
    .await;
    assert!(html.contains("Enter one of the recovery codes you saved."));
    assert!(html.contains(r#"href="/auth/mfa?method=totp">Use authenticator code</a>"#));

    // Without TOTP there is no authenticator link.
    let bob = user(&pool, "bob@example.test").await;
    enroll_key(&app, &pool, &bob, &SoftKey::es256(41)).await;
    let token = challenge_token(&pool, &bob, false).await;
    let (_, html) = page(&app, "/auth/mfa", &challenge_cookies(&token)).await;
    assert!(html.contains("Sign in with passkey"));
    assert!(!html.contains("Use authenticator code"));
    assert!(html.contains("Use recovery code"));
}

/// The first factor: two add buttons (security key first) and the
/// authenticator app as the alternative; with passkeys off, upstream's QR
/// page only, with no button, script or passkey wording.
#[tokio::test]
async fn first_factor_page_offers_keys_only_while_the_feature_is_on() {
    let pool = migrated().await;
    let cy = user(&pool, "cy@example.test").await;
    let on = app(&pool).await;
    let token = challenge_token(&pool, &cy, true).await;
    let (_, html) = page(&on, "/auth/mfa", &challenge_cookies(&token)).await;
    let key = at(&html, r#"data-webauthn="create" data-hints="security-key""#);
    let device = at(
        &html,
        r#"data-webauthn="create" data-hints="client-device,hybrid""#,
    );
    assert!(key < device);
    assert!(html.contains("Add a security key"));
    assert!(html.contains("Add a passkey on this device or phone"));
    assert!(html.contains(r#"<input type="text" name="name" id="passkey-name" maxlength="64""#));
    assert!(html.contains(r#"href="/auth/mfa?method=totp">Use an authenticator app instead</a>"#));
    assert!(!html.contains("data:image/png"), "no QR on the key view");
    no_inline_handlers(&html);
    let (_, html) = page(&on, "/auth/mfa?method=totp", &challenge_cookies(&token)).await;
    assert!(html.contains("data:image/png;base64,"));
    assert!(html.contains("Use a passkey or security key instead"));

    let off = app_with(&pool, None).await;
    for path in ["/auth/mfa", "/auth/mfa?method=passkey"] {
        let (status, html) = page(&off, path, &challenge_cookies(&token)).await;
        assert_eq!(status, 200);
        assert!(html.contains("data:image/png;base64,"));
        assert!(!html.contains("data-webauthn"));
        assert!(!html.contains("webauthn.js"));
        assert!(!html.contains("security key"), "{path}: no passkey wording");
        assert!(!html.contains("Use a passkey"), "{path}");
    }
}

/// The settings page lists every key with its kind, dates and links, and
/// offers the two add buttons only while passkeys are on.
#[tokio::test]
async fn settings_page_lists_keys_and_offers_both_add_buttons() {
    let pool = migrated().await;
    let dee = user(&pool, "dee@example.test").await;
    let app = app(&pool).await;
    let (session, _) = enroll_key(&app, &pool, &dee, &SoftKey::es256(42)).await;
    // A synced passkey, used once, whose name tries to inject markup.
    let synced_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO ilia_webauthn_credentials (id, user_id, rp_id, credential_id, public_key,
         algorithm, sign_count, uv_initialized, backup_eligible, backup_state, transports,
         name, created_at, last_used_at)
         VALUES (?, ?, ?, x'0102', x'00', -7, 0, 1, 1, 1, '[\"internal\"]', ?, 1767268800, 1767355200)",
    )
    .bind(&synced_id)
    .bind(&dee.id)
    .bind(RP_ID)
    .bind("<b>Phone</b>")
    .execute(&pool)
    .await
    .unwrap();
    let html = settings(&app, &session).await;
    assert!(html.contains(r#"class="mfa-status mfa-status-on">Two-factor authentication is on."#));
    assert!(html.contains("Security key 1"));
    assert!(html.contains(r#"<span class="passkey-kind">Hardware key</span>"#));
    assert!(
        html.contains(r#"<span class="passkey-kind passkey-kind-synced">Synced passkey</span>"#)
    );
    assert!(html.contains("&lt;b&gt;Phone&lt;&#x2f;b&gt;"));
    assert!(!html.contains("<b>Phone</b>"));
    assert!(html.contains("Not used yet"));
    assert!(html.contains("Last used Friday, January 2, 2026"));
    assert!(html.contains("Added Thursday, January 1, 2026"));
    assert!(html.contains(&format!(
        "confirm?action=passkey_rename&amp;credential={synced_id}\">Rename</a>"
    )));
    assert!(html.contains(&format!(
        "confirm?action=passkey_delete&amp;credential={synced_id}\">Remove</a>"
    )));
    let key = at(
        &html,
        r#"<a class="btn" href="/dashboard/settings/mfa/confirm?action=passkey_add&amp;kind=security-key">Add a security key</a>"#,
    );
    let device = at(
        &html,
        r#"<a class="btn btn-secondary" href="/dashboard/settings/mfa/confirm?action=passkey_add&amp;kind=device">Add a passkey on this device or phone</a>"#,
    );
    assert!(key < device);
    assert!(html.contains("A security key is a USB or NFC key"));
    assert!(
        !html.contains("Add a second key"),
        "two keys: no backup advice"
    );
    assert!(html.contains("No authenticator app is set up."));
    assert!(html.contains("confirm?action=totp_add\">Set up authenticator app</a>"));
    assert!(html.contains("10 recovery codes left"));
    assert!(html.contains("confirm?action=regenerate\">Generate new recovery codes</a>"));
    no_inline_handlers(&html);

    // Russian: the reader's saved language, plural included.
    sqlx::query("UPDATE users SET language = 'ru' WHERE id = ?")
        .bind(&dee.id)
        .execute(&pool)
        .await
        .unwrap();
    let html = settings(&app, &session).await;
    assert!(html.contains("Синхронизируемый ключ доступа"));
    assert!(html.contains("Аппаратный ключ"));
    assert!(html.contains("Осталось 10 резервных кодов"));
    assert!(html.contains("Добавить ключ доступа на этом устройстве или телефоне"));
}

/// A TOTP-only account on a server without passkeys sees upstream's page
/// with no passkey section at all; with passkeys on, the add buttons.
#[tokio::test]
async fn settings_page_without_keys_follows_the_feature_switch() {
    let pool = migrated().await;
    let eve = user(&pool, "eve@example.test").await;
    let session = totp_session(&pool, &eve).await;
    let off = app_with(&pool, None).await;
    let html = settings(&off, &session).await;
    assert!(html.contains("An authenticator app is set up."));
    assert!(!html.contains("Passkeys and security keys"));
    assert!(!html.contains("Passkeys are off"));
    assert!(!html.contains("action=passkey_add"));
    let on = app(&pool).await;
    let html = settings(&on, &session).await;
    assert!(html.contains("No passkeys or security keys yet."));
    assert!(html.contains("action=passkey_add&amp;kind=security-key"));
    assert!(html.contains("action=passkey_add&amp;kind=device"));
    assert!(html.contains("confirm?action=disable\">Remove authenticator app</a>"));

    // No factor yet: the password form that starts the enrolment.
    let fay = user(&pool, "fay@example.test").await;
    let mut tx = pool.begin().await.unwrap();
    let plain = mfa::verified_session(&mut tx, &fay.id).await.unwrap();
    tx.commit().await.unwrap();
    let html = settings(&on, &plain.id).await;
    assert!(html.contains("Two-factor authentication is off."));
    assert!(html.contains("a passkey or security key (YubiKey"));
    assert!(html.contains(r#"name="action" value="enable""#));
    let html = settings(&off, &plain.id).await;
    assert!(
        !html.contains("security key"),
        "nothing about keys while off"
    );
}

/// "Confirm it's you" renders for every action: what is about to happen,
/// the password, the key first, and the code methods as links.
#[tokio::test]
async fn confirm_page_renders_for_every_action() {
    let pool = migrated().await;
    let gus = user(&pool, "gus@example.test").await;
    let app = app(&pool).await;
    let (session, _) = enroll_key(&app, &pool, &gus, &SoftKey::es256(43)).await;
    add_totp(&pool, &gus).await;
    sqlx::query("UPDATE users SET role = 'admin' WHERE id = ?")
        .bind(&gus.id)
        .execute(&pool)
        .await
        .unwrap();
    let id: String = sqlx::query_scalar("SELECT id FROM ilia_webauthn_credentials")
        .fetch_one(&pool)
        .await
        .unwrap();
    let rename = format!("action=passkey_rename&credential={id}");
    let delete = format!("action=passkey_delete&credential={id}");
    for (query, sentence) in [
        ("action=regenerate", "You are replacing all recovery codes."),
        ("action=disable", "You are removing the authenticator app."),
        (
            "action=passkey_add",
            "You are adding a passkey or security key.",
        ),
        (
            rename.as_str(),
            "You are renaming the key “Security key 1”.",
        ),
        (
            delete.as_str(),
            "You are removing the key “Security key 1”.",
        ),
        (
            "action=policy&policy=required",
            "You are setting the two-factor policy for local accounts to:",
        ),
    ] {
        let (status, options, html) = confirm_page(&app, &session, query).await;
        assert_eq!(status, 200, "{query}");
        assert!(options.is_some(), "{query}: the key is the default");
        assert!(html.contains(sentence), "{query}");
        assert!(
            html.contains(r#"type="password" name="password""#),
            "{query}"
        );
        let key = at(&html, r#"data-webauthn="get""#);
        let totp = at(&html, "&amp;method=totp\">Use authenticator code</a>");
        let recovery = at(&html, "&amp;method=recovery\">Use recovery code</a>");
        assert!(key < totp && totp < recovery, "{query}");
        assert!(html.contains("Confirm with passkey"));
        no_inline_handlers(&html);
    }
    let (_, _, html) = confirm_page(&app, &session, &rename).await;
    assert!(html.contains(r#"name="name" id="passkey-name" value="Security key 1""#));
    let (_, _, html) = confirm_page(&app, &session, "action=policy&policy=required").await;
    // `post_to` is a variable, so its slashes come escaped.
    assert!(html.contains(r#"action="&#x2f;dashboard&#x2f;admin&#x2f;mfa""#));
    assert!(html.contains(r#"<input type="hidden" name="policy" value="required">"#));
    assert!(!html.contains(r#"name="action""#));
    let (_, options, html) = confirm_page(&app, &session, "action=regenerate&method=totp").await;
    assert!(options.is_none());
    assert!(html.contains(r#"name="code""#));
    assert!(html.contains("Confirm with code"));
    assert!(html.contains("&amp;method=passkey\">Use passkey</a>"));
}

/// The add button chosen on the settings page travels through the
/// confirmation and leads on the register page; an unknown value is no
/// choice.
#[tokio::test]
async fn register_page_leads_with_the_chosen_button() {
    let pool = migrated().await;
    let hal = user(&pool, "hal@example.test").await;
    let app = app(&pool).await;
    let (mut session, codes) = enroll_key(&app, &pool, &hal, &SoftKey::es256(44)).await;
    let (_, _, html) = confirm_page(
        &app,
        &session,
        "action=passkey_add&kind=device&method=recovery",
    )
    .await;
    assert!(html.contains(r#"<input type="hidden" name="kind" value="device">"#));
    assert!(html.contains("&amp;kind=device&amp;method=passkey\">Use passkey</a>"));
    let (_, _, html) = confirm_page(
        &app,
        &session,
        "action=passkey_add&kind=bogus&method=recovery",
    )
    .await;
    assert!(!html.contains(r#"name="kind""#));
    for (i, (kind, first, second)) in [
        ("device", "client-device,hybrid", "security-key"),
        ("security-key", "security-key", "client-device,hybrid"),
        ("bogus", "security-key", "client-device,hybrid"),
    ]
    .into_iter()
    .enumerate()
    {
        let r = request(
            &app,
            "POST",
            "/dashboard/settings/mfa",
            &session_cookies(&session),
            &format!(
                "_csrf=test&action=passkey_add&kind={kind}&password={PASSWORD}&code={}",
                codes[i]
            ),
        )
        .await;
        assert_eq!(r.status(), 200, "{kind}");
        session = new_session(&r).unwrap();
        let html = body(r).await;
        assert!(page_options(&html).is_some());
        let lead = at(
            &html,
            &format!(r#"class="btn btn-passkey" data-webauthn="create" data-hints="{first}""#),
        );
        let next = at(
            &html,
            &format!(r#"class="btn btn-secondary" data-webauthn="create" data-hints="{second}""#),
        );
        assert!(lead < next, "{kind}");
        assert!(html.contains(r#"action="/dashboard/settings/mfa/passkeys""#));
        no_inline_handlers(&html);
    }
}

/// Every passkey page keeps the site's CSP and Permissions-Policy (the
/// script is same-origin, nothing is inline), and the script served is the
/// asset, byte for byte.
#[tokio::test]
async fn passkey_pages_keep_the_site_headers_and_serve_the_asset() {
    let pool = migrated().await;
    let ivy = user(&pool, "ivy@example.test").await;
    let app = app(&pool).await;
    let login = request(&app, "GET", "/auth/login", CSRF, "").await;
    let csp = login.headers()["content-security-policy"].clone();
    let permissions = login.headers().get("permissions-policy").cloned();
    let token = challenge_token(&pool, &ivy, true).await;
    let enroll = request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await;
    let (session, codes) = enroll_key(&app, &pool, &ivy, &SoftKey::es256(45)).await;
    let token = challenge_token(&pool, &ivy, false).await;
    let sign_in = request(&app, "GET", "/auth/mfa", &challenge_cookies(&token), "").await;
    let settings_page = request(
        &app,
        "GET",
        "/dashboard/settings/mfa",
        &session_cookies(&session),
        "",
    )
    .await;
    let confirm = request(
        &app,
        "GET",
        "/dashboard/settings/mfa/confirm?action=passkey_add",
        &session_cookies(&session),
        "",
    )
    .await;
    let register = request(
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
    for (name, r) in [
        ("enroll", &enroll),
        ("sign in", &sign_in),
        ("settings", &settings_page),
        ("confirm", &confirm),
        ("register", &register),
    ] {
        assert_eq!(r.status(), 200, "{name}");
        assert_eq!(r.headers()["content-security-policy"], csp, "{name}");
        assert_eq!(
            r.headers().get("permissions-policy"),
            permissions.as_ref(),
            "{name}"
        );
        assert_eq!(r.headers()["cache-control"], "no-store", "{name}");
        assert_eq!(r.headers()["x-content-type-options"], "nosniff", "{name}");
    }
    let permissions = permissions
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    assert!(
        !permissions.contains("publickey-credentials"),
        "the WebAuthn features keep their default, self"
    );

    let r = request(&app, "GET", "/static/webauthn.js", "", "").await;
    assert_eq!(r.status(), 200);
    assert_eq!(
        r.headers()["content-type"],
        "application/javascript; charset=utf-8"
    );
    assert_eq!(r.headers()["x-content-type-options"], "nosniff");
    let js = body(r).await;
    assert_eq!(js, include_str!("../../../assets/webauthn.js"));
    for bad in [
        "fetch(",
        "XMLHttpRequest",
        "eval(",
        "innerHTML",
        "new Function",
    ] {
        assert!(!js.contains(bad), "{bad} in webauthn.js");
    }
    assert!(js.contains("parseRequestOptionsFromJSON"));
    assert!(js.contains("parseCreationOptionsFromJSON"));
    assert!(js.contains("getPublicKey"));
}

/// The passkey templates hold no inline script other than the options
/// JSON, and no inline handler, so they need nothing from `'unsafe-inline'`.
#[test]
fn passkey_templates_have_no_inline_code() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/auth");
    let mut seen = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.starts_with("ilia_") {
            continue;
        }
        seen += 1;
        let text = std::fs::read_to_string(&path).unwrap();
        no_inline_handlers(&text);
        for tag in text.split("<script").skip(1) {
            let open = tag.split('>').next().unwrap();
            assert!(
                open.contains(r#"type="application/json""#)
                    || open.contains(r#"src="/static/webauthn.js""#),
                "{name}: inline <script{open}>"
            );
        }
    }
    assert!(seen >= 10, "found the passkey templates");
}
