//! ilia fork: `config apply`, the `site.ton` section (docs/apply-schema.md,
//! apply/ton.rs): a document without it changes nothing, its keys become
//! owned by `apply` (read-only on the TON page), keys it owned and leaves
//! out go back to the page, values are checked and canonical, the export
//! round-trips, and the environment still wins over what apply stored.
//! Shared fixtures: `apply_support`.

use super::apply_support::*;
use crate::ton::config::TonConfig;
use crate::ton::settings::{DbValues, Owner};
use crate::web::apply::{self, ApplyError, Change};
use serde_json::{json, Value};

fn ton_doc(ton: Value) -> String {
    json!({"format": apply::FORMAT, "user": ADMIN_EMAIL, "site": {"ton": ton}}).to_string()
}

async fn rows(env: &ApplyEnv) -> DbValues {
    let mut conn = env.pool.acquire().await.unwrap();
    crate::web::ton_pay::store::load_settings(&mut conn)
        .await
        .unwrap()
}

fn change(path: &str, before: Value, after: Value) -> Change {
    Change {
        path: path.to_string(),
        before,
        after,
    }
}

#[tokio::test]
async fn a_document_without_ton_changes_nothing() {
    let env = ApplyEnv::new().await;
    sqlx::query(
        "INSERT INTO ton_settings (key, value, owner, updated_at) VALUES
            ('CALRS_TON_WINDOW_MIN', '45', 'apply', '2026-11-01T00:00:00Z'),
            ('CALRS_TON_GRACE_MIN', '20', 'panel', '2026-11-01T00:00:00Z')",
    )
    .execute(&env.pool)
    .await
    .unwrap();
    let before = env.snapshot().await;
    for doc in [
        empty_doc(),
        json!({"format": apply::FORMAT, "user": ADMIN_EMAIL, "site": {}}).to_string(),
    ] {
        let report = env.apply(&doc, false).await.unwrap();
        assert_eq!(report.changed, [], "{doc}");
        assert!(!report.restart);
    }
    assert_eq!(env.snapshot().await, before);
}

#[tokio::test]
async fn the_section_owns_its_keys_round_trips_and_hands_back() {
    let env = ApplyEnv::new().await;
    sqlx::query(
        "INSERT INTO ton_settings (key, value, owner, updated_at) VALUES
            ('CALRS_TON_GRACE_MIN', '20', 'panel', '2026-11-01T00:00:00Z')",
    )
    .execute(&env.pool)
    .await
    .unwrap();
    let doc = ton_doc(json!({
        "window_min": 45,
        "grace_min": null,
        "underpay_tolerance_ton": "0.01",
        "rate_min_usd": "0.3",
        "rate_max_usd": "5",
        "ladder": "free,20,40",
        "alert_email": "ops@example.com",
    }));

    // A dry run reports exactly what the run does, and writes nothing.
    let snapshot = env.snapshot().await;
    let dry = env.apply(&doc, true).await.unwrap();
    assert_eq!(env.snapshot().await, snapshot);
    let report = env.apply(&doc, false).await.unwrap();
    assert_eq!(dry.changed, report.changed);
    assert!(report.restart, "calrs serve reads the settings at startup");
    assert!(report
        .changed
        .contains(&change("site.ton.window_min", Value::Null, json!(45))));
    assert!(report.changed.contains(&change(
        "site.ton.window_min.owner",
        Value::Null,
        json!("apply")
    )));
    assert!(report
        .changed
        .contains(&change("site.ton.grace_min", json!(20), Value::Null)));
    assert!(report.changed.contains(&change(
        "site.ton.grace_min.owner",
        json!("panel"),
        json!("apply")
    )));
    let stored = rows(&env).await;
    assert_eq!(stored.value("CALRS_TON_WINDOW_MIN"), Some("45"));
    assert_eq!(stored.value("CALRS_TON_GRACE_MIN"), Some(""));
    assert_eq!(stored.owner("CALRS_TON_GRACE_MIN"), Some(Owner::Apply));
    assert_eq!(
        stored.value("CALRS_TON_UNDERPAY_TOLERANCE_NANO"),
        Some("10000000")
    );
    let audit: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ton_settings_audit WHERE source = 'apply'")
            .fetch_one(&env.pool)
            .await
            .unwrap();
    assert_eq!(audit, 7);

    // Again: no change. The export prints the section back, and applies
    // as no change.
    let again = env.apply(&doc, false).await.unwrap();
    assert_eq!(again.changed, []);
    assert!(!again.restart);
    let exported = env.export().await.unwrap();
    let ton = exported.site.as_ref().unwrap().ton.clone().unwrap();
    let wanted: Value = serde_json::from_str::<Value>(&doc).unwrap()["site"]["ton"].clone();
    assert_eq!(serde_json::to_value(&ton).unwrap(), wanted);
    let report = env
        .apply(&serde_json::to_string(&exported).unwrap(), false)
        .await
        .unwrap();
    assert_eq!(report.changed, []);

    // The stored values are what `calrs serve` resolves: the environment
    // still wins, then these, then the defaults.
    let resolved = |env_vars: &[(&str, &str)]| {
        let env_vars: Vec<(String, String)> = env_vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k: &str| {
            env_vars
                .iter()
                .find(|(n, _)| n == k)
                .map(|(_, v)| v.clone())
        }
    };
    let base = [
        ("CALRS_TON_NETWORK", "mainnet"),
        (
            "CALRS_TON_ADDRESS",
            "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI",
        ),
    ];
    let cfg = TonConfig::resolve(&resolved(&base), &stored)
        .unwrap()
        .unwrap();
    assert_eq!((cfg.window_min, cfg.grace_min), (45, 15));
    assert_eq!(cfg.ladder.text(), "free,20,40");
    let mut with_env = base.to_vec();
    with_env.push(("CALRS_TON_WINDOW_MIN", "60"));
    let cfg = TonConfig::resolve(&resolved(&with_env), &stored)
        .unwrap()
        .unwrap();
    assert_eq!(cfg.window_min, 60, "the environment wins");

    // Keys left out of a present section go back to the page, value kept;
    // a null one is removed. Only the owner changes: no restart.
    let report = env
        .apply(
            &ton_doc(json!({
                "underpay_tolerance_ton": "0.01",
                "rate_min_usd": "0.3",
                "rate_max_usd": "5",
                "ladder": "free,20,40",
                "alert_email": "ops@example.com",
            })),
            false,
        )
        .await
        .unwrap();
    assert_eq!(
        report.changed,
        [
            change("site.ton.window_min.owner", json!("apply"), json!("panel")),
            change("site.ton.grace_min.owner", json!("apply"), Value::Null),
        ]
    );
    assert!(!report.restart);
    let stored = rows(&env).await;
    assert_eq!(stored.value("CALRS_TON_WINDOW_MIN"), Some("45"));
    assert_eq!(stored.owner("CALRS_TON_WINDOW_MIN"), Some(Owner::Panel));
    assert_eq!(stored.value("CALRS_TON_GRACE_MIN"), None);
    let exported = env.export().await.unwrap();
    let ton = exported.site.unwrap().ton.unwrap();
    assert!(!ton.contains_key("window_min"), "only apply's keys export");

    // An empty section hands everything back; the export then has none.
    env.apply(&ton_doc(json!({})), false).await.unwrap();
    assert!(env.export().await.unwrap().site.unwrap().ton.is_none());
}

#[tokio::test]
async fn invalid_or_unknown_values_write_nothing() {
    let env = ApplyEnv::new().await;
    let before = env.snapshot().await;
    for (ton, path, says) in [
        (
            json!({"network": "mainnet"}),
            "site.ton.network",
            "unknown key",
        ),
        (json!({"address": "UQ…"}), "site.ton.address", "unknown key"),
        (
            json!({"window_min": 9}),
            "site.ton.window_min",
            "between 10 and 240",
        ),
        (
            json!({"window_min": "45"}),
            "site.ton.window_min",
            "whole number",
        ),
        (
            json!({"underpay_tolerance_ton": "0.010"}),
            "site.ton.underpay_tolerance_ton",
            "spell it \"0.01\"",
        ),
        (
            json!({"ladder": "20,free"}),
            "site.ton.ladder",
            "free may only come first",
        ),
        (
            json!({"rate_min_usd": "0.3"}),
            "site.ton.rate_min_usd",
            "go together",
        ),
        (
            json!({"alert_email": "nobody"}),
            "site.ton.alert_email",
            "not an email",
        ),
    ] {
        let err = env.apply(&ton_doc(ton.clone()), false).await.unwrap_err();
        assert_eq!(err.exit_code(), 2, "{ton}");
        let ApplyError::Invalid { path: at, message } = &err else {
            panic!("{ton}: {err}");
        };
        assert_eq!(at, path, "{ton}: {err}");
        assert!(message.contains(says), "{ton}: {err}");
    }
    assert_eq!(env.snapshot().await, before);
}

/// The `calrs::audit` event of a `site.ton` change is logged only once the
/// run commits: a `--dry-run` (rolled back) logs nothing, a real run logs
/// each change. The durable trail is the `ton_settings_audit` row.
#[test]
fn the_audit_event_is_logged_only_after_the_commit() {
    if super::security::under_tarpaulin() {
        return;
    }
    let doc = ton_doc(json!({"window_min": 45}));
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let env = rt.block_on(ApplyEnv::new());
    let dry = super::security::capture_tracing(|| {
        rt.block_on(async {
            env.apply(&doc, true).await.unwrap();
        })
    });
    assert!(!dry.contains("ton setting changed"), "{dry}");
    let audited: i64 = rt.block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM ton_settings_audit")
            .fetch_one(&env.pool)
            .await
            .unwrap()
    });
    assert_eq!(audited, 0, "the dry run stored no audit row either");
    let real = super::security::capture_tracing(|| {
        rt.block_on(async {
            env.apply(&doc, false).await.unwrap();
        })
    });
    assert!(real.contains("ton setting changed"), "{real}");
    assert!(real.contains("CALRS_TON_WINDOW_MIN"), "{real}");
}
