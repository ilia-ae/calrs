//! ilia fork: tests for the second overnight review round, group `apply`.
//!
//! `site.sms: "off"` (docs/apply-schema.md, "SMS gateway"): the stored
//! gateway deleted as the admin card's Clear button does, one report path
//! per column, the encrypted secret shown only as set, no restart, a second
//! run and the export as no change.

use super::apply_support::*;
use crate::web::apply::{self, ApplyError, Change, Off};
use serde_json::{json, Value};

fn site_doc(site: &str) -> String {
    format!(
        r#"{{"format":"{}","user":"{ADMIN_EMAIL}","site":{site}}}"#,
        apply::FORMAT
    )
}

/// A Twilio gateway saved on the admin page, and a second (disabled) row.
async fn seed_gateway(env: &ApplyEnv) {
    for statement in [
        "INSERT INTO sms_config (id, provider, api_key, api_secret_enc, sender, enabled) \
         VALUES ('a', 'twilio', 'AC123', 'enc-token', '+15550100', 1)",
        "INSERT INTO sms_config (id, provider, api_secret_enc, base_url, enabled) \
         VALUES ('b', 'webhook', NULL, 'https://hook.example', 0)",
    ] {
        sqlx::query(statement).execute(&env.pool).await.unwrap();
    }
}

async fn gateways(env: &ApplyEnv) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM sms_config")
        .fetch_one(&env.pool)
        .await
        .unwrap()
}

fn change(path: &str, before: Value) -> Change {
    Change {
        path: path.to_string(),
        before,
        after: Value::Null,
    }
}

#[tokio::test]
async fn sms_off_deletes_the_stored_gateway_then_changes_nothing() {
    let env = ApplyEnv::new().await;
    seed_gateway(&env).await;
    // A stored gateway is not "off": the export leaves the key out.
    let exported = env.export().await.unwrap().site.unwrap();
    assert_eq!(exported.sms, None);

    let doc = site_doc(r#"{"sms":"off"}"#);
    let first = env.apply(&doc, false).await.unwrap();
    assert_eq!(
        first.changed,
        [
            change("site.sms.provider", json!("twilio")),
            change("site.sms.api_key", json!("AC123")),
            change("site.sms.sender", json!("+15550100")),
            change("site.sms.api_secret", json!(apply::SECRET_SET)),
        ],
        "the enabled row, its secret only as set"
    );
    assert!(!first.restart, "the gateway is read on every send");
    assert_eq!(gateways(&env).await, 0, "every row, as the Clear button");

    let applied = env.snapshot().await;
    let second = env.apply(&doc, false).await.unwrap();
    assert_eq!(second.changed, []);
    assert_eq!(env.snapshot().await, applied);

    let exported = env.export().await.unwrap().site.unwrap();
    assert_eq!(exported.sms, Some(Off::Off));
}

#[tokio::test]
async fn sms_off_on_a_dry_run_reports_and_writes_nothing() {
    let env = ApplyEnv::new().await;
    seed_gateway(&env).await;
    let before = env.snapshot().await;
    let dry = env
        .apply(&site_doc(r#"{"sms":"off"}"#), true)
        .await
        .unwrap();
    assert!(dry.dry_run);
    assert_eq!(dry.changed.len(), 4);
    assert_eq!(env.snapshot().await, before);
    assert_eq!(gateways(&env).await, 2);
}

#[tokio::test]
async fn sms_takes_only_off() {
    let env = ApplyEnv::new().await;
    seed_gateway(&env).await;
    let before = env.snapshot().await;
    for site in [
        r#"{"sms":null}"#,
        r#"{"sms":"on"}"#,
        r#"{"sms":false}"#,
        r#"{"sms":{"provider":"twilio"}}"#,
    ] {
        let error = env.apply(&site_doc(site), false).await.unwrap_err();
        assert!(
            matches!(&error, ApplyError::Invalid { path, .. } if path == "(document)"),
            "{site}: {error}"
        );
        assert_eq!(error.exit_code(), 2);
        assert_eq!(env.snapshot().await, before, "{site} wrote something");
    }
}

#[tokio::test]
async fn sms_left_out_keeps_the_gateway() {
    let env = ApplyEnv::new().await;
    seed_gateway(&env).await;
    let report = env
        .apply(&site_doc(r#"{"sms_allow_all_users":true}"#), false)
        .await
        .unwrap();
    assert_eq!(report.changed.len(), 1);
    assert_eq!(gateways(&env).await, 2);
}

/// The webhook row alone: no api_key, no sender, no secret; the report
/// names what is there and nothing else.
#[tokio::test]
async fn sms_off_reports_only_the_columns_that_are_set() {
    let env = ApplyEnv::new().await;
    sqlx::query(
        "INSERT INTO sms_config (id, provider, base_url, enabled) \
         VALUES ('w', 'webhook', 'https://hook.example', 1)",
    )
    .execute(&env.pool)
    .await
    .unwrap();
    let report = env
        .apply(&site_doc(r#"{"sms":"off"}"#), false)
        .await
        .unwrap();
    assert_eq!(
        report.changed,
        [
            change("site.sms.provider", json!("webhook")),
            change("site.sms.base_url", json!("https://hook.example")),
        ]
    );
    assert_eq!(gateways(&env).await, 0);
}
