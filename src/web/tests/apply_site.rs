//! ilia fork: `config apply`, the `site` section (docs/apply-schema.md):
//! the admin cards posted through their handlers, the MFA policy, the
//! features switched off with their secrets, the restart flag, the
//! canonical-value checks and the export. Shared fixtures: `apply_support`.

use super::apply_support::*;
use super::*;
use crate::web::apply::{self, ApplyError, Change, Document, Jitsi, Off, Theme};
use serde_json::{json, Value};

/// A document with only `site`.
fn site_doc(site: &str) -> String {
    format!(
        r#"{{"format":"{}","user":"{ADMIN_EMAIL}","site":{site}}}"#,
        apply::FORMAT
    )
}

/// book.ilia.ae's site (settings design §5, MFA kept optional for now).
const FULL_SITE: &str = r##"{
    "theme": "custom",
    "custom": {"accent": "#313131", "accent_hover": "#111111", "bg": "#f6f6f6",
               "surface": "#ffffff", "text": "#313131"},
    "week_start": "mon",
    "german_formal": true,
    "registration": false,
    "allowed_email_domains": "ilia.ae",
    "mfa_policy": "optional",
    "company_link": "https://ilia.ae",
    "jitsi": {"base_url": "https://meet.jit.si", "pattern": null, "display_name": "Jitsi Meet"},
    "oidc": "off",
    "google_oauth2": "off",
    "captcha": "off",
    "meeting_webhook": "off",
    "sms": "off",
    "sms_allow_all_users": false
}"##;

async fn text(env: &ApplyEnv, column: &str) -> Option<String> {
    sqlx::query_scalar(&format!(
        "SELECT {column} FROM auth_config WHERE id = 'singleton'"
    ))
    .fetch_one(&env.pool)
    .await
    .unwrap()
}

async fn flag(env: &ApplyEnv, column: &str) -> bool {
    sqlx::query_scalar(&format!(
        "SELECT {column} FROM auth_config WHERE id = 'singleton'"
    ))
    .fetch_one(&env.pool)
    .await
    .unwrap()
}

async fn set(env: &ApplyEnv, assignments: &str) {
    sqlx::query(&format!(
        "UPDATE auth_config SET {assignments} WHERE id = 'singleton'"
    ))
    .execute(&env.pool)
    .await
    .unwrap();
}

fn paths(changes: &[Change]) -> Vec<&str> {
    changes.iter().map(|c| c.path.as_str()).collect()
}

fn change(path: &str, before: Value, after: Value) -> Change {
    Change {
        path: path.to_string(),
        before,
        after,
    }
}

fn invalid_path(error: &ApplyError) -> &str {
    match error {
        ApplyError::Invalid { path, .. } => path,
        other => panic!("expected an invalid document, got {other}"),
    }
}

#[tokio::test]
async fn a_full_site_is_written_then_a_second_run_changes_nothing() {
    let env = ApplyEnv::new().await;
    let doc = site_doc(FULL_SITE);

    let first = env.apply(&doc, false).await.unwrap();
    assert!(!first.dry_run);
    assert!(
        first.restart,
        "the theme, company link and Jitsi are cached"
    );
    assert_eq!(
        paths(&first.changed),
        [
            "site.theme",
            "site.custom.accent",
            "site.custom.accent_hover",
            "site.custom.bg",
            "site.custom.surface",
            "site.custom.text",
            "site.german_formal",
            "site.registration",
            "site.allowed_email_domains",
            "site.company_link",
            "site.jitsi",
        ]
    );
    assert_eq!(
        first.changed[0],
        change("site.theme", json!("default"), json!("custom"))
    );
    assert_eq!(
        first.changed[10],
        change(
            "site.jitsi",
            json!("off"),
            json!({"base_url": "https://meet.jit.si", "pattern": null, "display_name": "Jitsi Meet"})
        )
    );

    assert_eq!(text(&env, "theme").await.as_deref(), Some("custom"));
    for (column, value) in [
        ("custom_accent", "#313131"),
        ("custom_accent_hover", "#111111"),
        ("custom_bg", "#f6f6f6"),
        ("custom_surface", "#ffffff"),
        ("custom_text", "#313131"),
        ("week_start", "mon"),
        ("allowed_email_domains", "ilia.ae"),
        ("company_link", "https://ilia.ae"),
        ("jitsi_base_url", "https://meet.jit.si"),
        ("jitsi_display_name", "Jitsi Meet"),
    ] {
        assert_eq!(text(&env, column).await.as_deref(), Some(value), "{column}");
    }
    assert_eq!(text(&env, "jitsi_pattern").await, None);
    assert!(flag(&env, "german_formal").await);
    assert!(!flag(&env, "registration_enabled").await);
    assert!(!flag(&env, "mfa_required").await);
    assert!(!flag(&env, "sms_allow_all_users").await);

    let second = env.apply(&doc, false).await.unwrap();
    assert_eq!(second.changed, []);
    assert!(!second.restart);

    // The export says exactly what was applied.
    let exported = env.export().await.unwrap().site.unwrap();
    assert_eq!(exported, apply::parse(&doc).unwrap().site.unwrap());
}

#[tokio::test]
async fn a_dry_run_reports_the_real_run_and_writes_nothing() {
    let env = ApplyEnv::new().await;
    let doc = site_doc(FULL_SITE);
    let before = env.snapshot().await;

    let dry = env.apply(&doc, true).await.unwrap();
    assert!(dry.dry_run);
    assert_eq!(env.snapshot().await, before);

    let real = env.apply(&doc, false).await.unwrap();
    assert_eq!(dry.changed, real.changed);
    assert_eq!(dry.restart, real.restart);
    assert_ne!(env.snapshot().await, before);
}

#[tokio::test]
async fn a_partial_site_leaves_every_other_setting_alone() {
    let env = ApplyEnv::new().await;
    // Stored values the General card carries but the document does not
    // own, and settings of other cards.
    set(
        &env,
        "base_url = 'https://cal.example.com', allow_private_hosts = 'radicale', \
         theme = 'nord', company_link = 'https://example.com', \
         allowed_email_domains = 'Example.com, x.org', week_start = 'sun'",
    )
    .await;
    let before = others(&env).await;

    let report = env
        .apply(&site_doc(r#"{"german_formal":true}"#), false)
        .await
        .unwrap();
    assert_eq!(
        report.changed,
        [change("site.german_formal", json!(false), json!(true))]
    );
    assert!(report.restart, "german_formal is cached at startup");
    assert!(flag(&env, "german_formal").await);
    assert_eq!(others(&env).await, before);
}

/// Every `auth_config` column except `german_formal` and `updated_at`.
async fn others(env: &ApplyEnv) -> String {
    let columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('auth_config') ORDER BY cid")
            .fetch_all(&env.pool)
            .await
            .unwrap();
    let row = columns
        .iter()
        .filter(|c| !matches!(c.as_str(), "german_formal" | "updated_at"))
        .map(|c| format!("quote(\"{c}\")"))
        .collect::<Vec<_>>()
        .join(" || '|' || ");
    sqlx::query_scalar(&format!(
        "SELECT {row} FROM auth_config WHERE id = 'singleton'"
    ))
    .fetch_one(&env.pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn restart_is_asked_only_for_values_serve_caches() {
    let env = ApplyEnv::new().await;
    let report = env
        .apply(
            &site_doc(r#"{"registration":false,"sms_allow_all_users":true,"week_start":"sun"}"#),
            false,
        )
        .await
        .unwrap();
    assert_eq!(
        paths(&report.changed),
        [
            "site.week_start",
            "site.registration",
            "site.sms_allow_all_users"
        ]
    );
    assert!(!report.restart);
    assert_eq!(text(&env, "week_start").await.as_deref(), Some("sun"));
    assert!(flag(&env, "sms_allow_all_users").await);

    // A preset theme leaves the stored custom colours as they are.
    set(&env, "custom_accent = '#000000'").await;
    let report = env
        .apply(&site_doc(r#"{"theme":"nord"}"#), false)
        .await
        .unwrap();
    assert_eq!(
        report.changed,
        [change("site.theme", json!("default"), json!("nord"))]
    );
    assert!(report.restart);
    assert_eq!(
        text(&env, "custom_accent").await.as_deref(),
        Some("#000000")
    );
}

async fn insert_session(env: &ApplyEnv, user_id: &str, verified: bool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, expires_at, mfa_verified) VALUES (?, ?, datetime('now', '+1 day'), ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(verified)
    .execute(&env.pool)
    .await
    .unwrap();
    id
}

async fn sessions(env: &ApplyEnv) -> Vec<String> {
    sqlx::query_scalar("SELECT id FROM sessions ORDER BY id")
        .fetch_all(&env.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn mfa_required_ends_the_local_sessions_that_did_not_pass_mfa() {
    let env = ApplyEnv::new().await;
    let sso = insert_role_user(&env.pool, "sso@example.com", "user").await;
    sqlx::query("UPDATE users SET auth_provider = 'oidc' WHERE id = ?")
        .bind(&sso)
        .execute(&env.pool)
        .await
        .unwrap();
    let _unverified = insert_session(&env, &env.admin_id, false).await;
    let verified = insert_session(&env, &env.admin_id, true).await;
    let single_sign_on = insert_session(&env, &sso, false).await;

    let required = site_doc(r#"{"mfa_policy":"required"}"#);
    let report = env.apply(&required, false).await.unwrap();
    assert_eq!(
        report.changed,
        [change(
            "site.mfa_policy",
            json!("optional"),
            json!("required")
        )]
    );
    assert!(!report.restart);
    assert!(flag(&env, "mfa_required").await);
    let mut kept = vec![verified, single_sign_on];
    kept.sort();
    assert_eq!(sessions(&env).await, kept);

    // An unchanged policy posts nothing, so a later session survives.
    let later = insert_session(&env, &env.admin_id, false).await;
    let report = env.apply(&required, false).await.unwrap();
    assert_eq!(report.changed, []);
    assert!(sessions(&env).await.contains(&later));

    let report = env
        .apply(&site_doc(r#"{"mfa_policy":"optional"}"#), false)
        .await
        .unwrap();
    assert_eq!(
        report.changed,
        [change(
            "site.mfa_policy",
            json!("required"),
            json!("optional")
        )]
    );
    assert!(!flag(&env, "mfa_required").await);
    assert!(sessions(&env).await.contains(&later));
}

#[tokio::test]
async fn off_clears_every_field_and_secret_of_the_feature() {
    let env = ApplyEnv::new().await;
    set(
        &env,
        "oidc_enabled = 1, oidc_issuer_url = 'https://idp.example', oidc_client_id = 'calrs', \
         oidc_client_secret = 'enc-oidc', oidc_auto_register = 1, \
         google_oauth2_client_id = 'gid', google_oauth2_client_secret = 'enc-google', \
         captcha_instance_url = 'https://cap.example', captcha_site_key = 'key', \
         captcha_secret = 'enc-cap', captcha_widget_url = 'https://cap.example/w.js', \
         captcha_wasm_url = 'https://cap.example/w.wasm', \
         meeting_webhook_url = 'https://hook.example', meeting_webhook_auth_mode = 'hmac', \
         meeting_webhook_secret = 'enc-hook', meeting_webhook_display_name = 'Hook'",
    )
    .await;
    let doc =
        site_doc(r#"{"oidc":"off","google_oauth2":"off","captcha":"off","meeting_webhook":"off"}"#);

    let report = env.apply(&doc, false).await.unwrap();
    assert!(report.restart, "captcha and the meeting webhook are cached");
    let secret = || json!(apply::SECRET_SET);
    assert_eq!(
        report.changed,
        [
            change("site.oidc.enabled", json!(true), json!(false)),
            change(
                "site.oidc.issuer_url",
                json!("https://idp.example"),
                Value::Null
            ),
            change("site.oidc.client_id", json!("calrs"), Value::Null),
            change("site.oidc.client_secret", secret(), Value::Null),
            change("site.google_oauth2.client_id", json!("gid"), Value::Null),
            change("site.google_oauth2.client_secret", secret(), Value::Null),
            change(
                "site.captcha.instance_url",
                json!("https://cap.example"),
                Value::Null
            ),
            change("site.captcha.site_key", json!("key"), Value::Null),
            change(
                "site.captcha.widget_url",
                json!("https://cap.example/w.js"),
                Value::Null
            ),
            change(
                "site.captcha.wasm_url",
                json!("https://cap.example/w.wasm"),
                Value::Null
            ),
            change("site.captcha.secret", secret(), Value::Null),
            change(
                "site.meeting_webhook.url",
                json!("https://hook.example"),
                Value::Null
            ),
            change("site.meeting_webhook.auth_mode", json!("hmac"), Value::Null),
            change(
                "site.meeting_webhook.display_name",
                json!("Hook"),
                Value::Null
            ),
            change("site.meeting_webhook.secret", secret(), Value::Null),
        ]
    );
    for column in [
        "oidc_issuer_url",
        "oidc_client_id",
        "oidc_client_secret",
        "google_oauth2_client_id",
        "google_oauth2_client_secret",
        "captcha_instance_url",
        "captcha_site_key",
        "captcha_secret",
        "captcha_widget_url",
        "captcha_wasm_url",
        "meeting_webhook_url",
        "meeting_webhook_auth_mode",
        "meeting_webhook_secret",
        "meeting_webhook_display_name",
    ] {
        assert_eq!(text(&env, column).await, None, "{column}");
    }
    assert!(!flag(&env, "oidc_enabled").await);
    // Not part of "off": the card's own choice is posted back.
    assert!(flag(&env, "oidc_auto_register").await);

    let again = env.apply(&doc, false).await.unwrap();
    assert_eq!(again.changed, []);
    assert!(!again.restart);

    let site = env.export().await.unwrap().site.unwrap();
    assert_eq!(site.oidc, Some(Off::Off));
    assert_eq!(site.google_oauth2, Some(Off::Off));
    assert_eq!(site.captcha, Some(Off::Off));
    assert_eq!(site.meeting_webhook, Some(Off::Off));
}

#[tokio::test]
async fn a_secret_alone_is_cleared_without_its_card() {
    let env = ApplyEnv::new().await;
    set(&env, "google_oauth2_client_secret = 'enc-google'").await;
    let report = env
        .apply(&site_doc(r#"{"google_oauth2":"off"}"#), false)
        .await
        .unwrap();
    assert_eq!(
        report.changed,
        [change(
            "site.google_oauth2.client_secret",
            json!(apply::SECRET_SET),
            Value::Null
        )]
    );
    assert_eq!(text(&env, "google_oauth2_client_secret").await, None);
}

#[tokio::test]
async fn jitsi_is_switched_on_and_off() {
    let env = ApplyEnv::new().await;
    let on = r#"{"jitsi":{"base_url":"https://meet.example.org/rooms","pattern":"{event}-{date}","display_name":null}}"#;
    let report = env.apply(&site_doc(on), false).await.unwrap();
    let settings = json!({"base_url": "https://meet.example.org/rooms", "pattern": "{event}-{date}", "display_name": null});
    assert_eq!(
        report.changed,
        [change("site.jitsi", json!("off"), settings.clone())]
    );
    assert!(report.restart);
    assert_eq!(
        text(&env, "jitsi_pattern").await.as_deref(),
        Some("{event}-{date}")
    );
    assert_eq!(text(&env, "jitsi_display_name").await, None);

    let report = env
        .apply(&site_doc(r#"{"jitsi":"off"}"#), false)
        .await
        .unwrap();
    assert_eq!(
        report.changed,
        [change("site.jitsi", settings, json!("off"))]
    );
    assert!(report.restart);
    for column in ["jitsi_base_url", "jitsi_pattern", "jitsi_display_name"] {
        assert_eq!(text(&env, column).await, None, "{column}");
    }
    assert_eq!(
        env.export().await.unwrap().site.unwrap().jitsi,
        Some(Jitsi::Off(Off::Off))
    );
}

#[tokio::test]
async fn values_the_handlers_would_rewrite_or_reject_are_refused_before_any_write() {
    let env = ApplyEnv::new().await;
    let colours = |accent: &str, bg: &str| {
        format!(
            r##"{{"accent":"{accent}","accent_hover":"#111111","bg":"{bg}","surface":"#ffffff","text":"#313131"}}"##
        )
    };
    let good = colours("#313131", "#f6f6f6");
    let jitsi = |base: &str, pattern: &str, name: &str| {
        format!(r#"{{"jitsi":{{"base_url":"{base}","pattern":{pattern},"display_name":{name}}}}}"#)
    };
    let cases: Vec<(String, &str)> = vec![
        (r#"{"theme":"custom"}"#.into(), "site.custom"),
        (format!(r#"{{"custom":{good}}}"#), "site.custom"),
        (
            format!(r#"{{"theme":"nord","custom":{good}}}"#),
            "site.custom",
        ),
        (
            format!(
                r#"{{"theme":"custom","custom":{}}}"#,
                colours("#31313", "#f6f6f6")
            ),
            "site.custom.accent",
        ),
        (
            format!(
                r#"{{"theme":"custom","custom":{}}}"#,
                colours("#313131", "#F6F6F6")
            ),
            "site.custom.bg",
        ),
        (
            format!(
                r#"{{"theme":"custom","custom":{}}}"#,
                colours("#313131", " #f6f6f6")
            ),
            "site.custom.bg",
        ),
        (
            r#"{"company_link":"javascript:alert(1)"}"#.into(),
            "site.company_link",
        ),
        (
            r#"{"company_link":" https://ilia.ae"}"#.into(),
            "site.company_link",
        ),
        (r#"{"company_link":""}"#.into(), "site.company_link"),
        (r#"{"company_link":"https://"}"#.into(), "site.company_link"),
        (
            r#"{"allowed_email_domains":"Ilia.ae"}"#.into(),
            "site.allowed_email_domains",
        ),
        (
            r#"{"allowed_email_domains":"ilia.ae, example.org"}"#.into(),
            "site.allowed_email_domains",
        ),
        (
            r#"{"allowed_email_domains":""}"#.into(),
            "site.allowed_email_domains",
        ),
        (
            r#"{"allowed_email_domains":"ilia.ae,ilia.ae"}"#.into(),
            "site.allowed_email_domains",
        ),
        (
            jitsi("ftp://meet.example", "null", "null"),
            "site.jitsi.base_url",
        ),
        (
            jitsi("javascript:alert(1)", "null", "null"),
            "site.jitsi.base_url",
        ),
        (
            jitsi("https://meet.jit.si/?room=x", "null", "null"),
            "site.jitsi.base_url",
        ),
        (
            jitsi("https://user:pw@meet.jit.si", "null", "null"),
            "site.jitsi.base_url",
        ),
        (jitsi("", "null", "null"), "site.jitsi.base_url"),
        (
            jitsi("https://meet.jit.si", r#""""#, "null"),
            "site.jitsi.pattern",
        ),
        (
            jitsi("https://meet.jit.si", "null", r#"" Jitsi""#),
            "site.jitsi.display_name",
        ),
        // Refused by the schema itself.
        (r#"{"base_url":"https://x.example"}"#.into(), "(document)"),
        (r#"{"allow_private_hosts":"radicale"}"#.into(), "(document)"),
        (r#"{"oidc":"on"}"#.into(), "(document)"),
        (r#"{"captcha":null}"#.into(), "(document)"),
        (
            r#"{"jitsi":{"base_url":"https://meet.jit.si"}}"#.into(),
            "(document)",
        ),
        (r#"{"jitsi":"on"}"#.into(), "(document)"),
        (r#"{"mfa_policy":null}"#.into(), "(document)"),
        (r#"{"mfa_policy":"always"}"#.into(), "(document)"),
        (r#"{"registration":"yes"}"#.into(), "(document)"),
        (r#"{"theme":"Nord"}"#.into(), "(document)"),
        (r#"{"week_start":"monday"}"#.into(), "(document)"),
    ];
    let before = env.snapshot().await;
    for (site, path) in cases {
        let error = env.apply(&site_doc(&site), false).await.unwrap_err();
        assert_eq!(invalid_path(&error), path, "{site}: {error}");
        assert_eq!(error.exit_code(), 2);
        assert_eq!(env.snapshot().await, before, "{site} wrote something");
    }
}

#[tokio::test]
async fn the_export_of_a_fresh_database_applies_as_no_change() {
    let env = ApplyEnv::new().await;
    let site = env.export().await.unwrap().site.unwrap();
    assert_eq!(site.theme, Some(Theme::Default));
    assert_eq!(site.custom, None);
    assert_eq!(site.registration, Some(true));
    assert_eq!(site.allowed_email_domains, Some(None));
    assert_eq!(site.company_link, Some(None));
    assert_eq!(site.jitsi, Some(Jitsi::Off(Off::Off)));
    assert_eq!(site.oidc, Some(Off::Off));
    assert_eq!(site.captcha, Some(Off::Off));

    let before = env.snapshot().await;
    let report = env.apply(&only_site(site), false).await.unwrap();
    assert_eq!(report.changed, []);
    assert_eq!(env.snapshot().await, before);
}

#[tokio::test]
async fn the_export_leaves_out_what_the_document_cannot_say() {
    let env = ApplyEnv::new().await;
    // A custom theme without its colours, a hand-written domain list, a
    // half-configured Jitsi and OIDC: none can be written as a document
    // value, and the export must still apply as no change.
    set(
        &env,
        "theme = 'custom', allowed_email_domains = 'Example.com, x.org', \
         jitsi_pattern = '{event}', oidc_client_id = 'calrs', company_link = 'javascript:x'",
    )
    .await;
    let site = env.export().await.unwrap().site.unwrap();
    assert_eq!(site.theme, None);
    assert_eq!(site.custom, None);
    assert_eq!(site.allowed_email_domains, None);
    assert_eq!(site.company_link, None);
    assert_eq!(site.jitsi, None);
    assert_eq!(site.oidc, None);
    assert_eq!(site.google_oauth2, Some(Off::Off));

    let before = env.snapshot().await;
    let report = env.apply(&only_site(site), false).await.unwrap();
    assert_eq!(report.changed, []);
    assert_eq!(env.snapshot().await, before);
}

/// A document with `site` and nothing else.
fn only_site(site: apply::Site) -> String {
    serde_json::to_string(&Document {
        format: apply::FORMAT.to_string(),
        user: ADMIN_EMAIL.to_string(),
        profile: None,
        site: Some(site),
        sources: None,
        event_types: None,
    })
    .unwrap()
}
