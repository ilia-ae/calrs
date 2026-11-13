//! ilia fork: tests for the third overnight review round, group `ton`.
//!
//! The TON admin page's status in the admin's language (the watcher's
//! "not ready" reasons and the unset network; the English texts stay for
//! the log), the ladder editor's row label without a grammatical number
//! and its accessible names, and the debug page's disabled Telegram box.

use super::*;
use crate::ton::config::TonConfig;
use crate::web::ton_pay::{EnvSnapshot, TonRuntime};

const TON_PAGE: &str = "/dashboard/admin/ton";

struct Page {
    pool: SqlitePool,
    state: Arc<AppState>,
    session: String,
    _dir: TestDataDir,
}

/// An admin (`testuser`) and the app, payments off unless `ton` (the
/// test network) is asked for: then a runtime that has checked nothing yet.
async fn page_env(ton: bool) -> (Page, Option<Arc<TonRuntime>>) {
    let pool = setup_test_db().await;
    seed_test_data(&pool).await;
    let dir = TestDataDir::new();
    let state = build_state(pool.clone(), dir.to_path_buf(), [0u8; 32]).await;
    let rt = ton.then(|| {
        let vars = crate::ton::config::tests::base();
        let pairs: Vec<(&str, &str)> = vars.iter().map(|(k, v)| (*k, v.as_str())).collect();
        state.ton.set_env(EnvSnapshot::of(&pairs));
        let cfg = TonConfig::from_vars(|k| vars.get(k).cloned())
            .unwrap()
            .unwrap();
        let rt = Arc::new(TonRuntime::http(cfg));
        state.ton.install(rt.clone());
        rt
    });
    if !ton {
        state.ton.set_env(EnvSnapshot::default());
    }
    let session = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, expires_at)
         SELECT ?, id, ? FROM users WHERE username = 'testuser'",
    )
    .bind(&session)
    .bind(
        (Utc::now() + Duration::days(1))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    )
    .execute(&pool)
    .await
    .unwrap();
    (
        Page {
            pool,
            state,
            session,
            _dir: dir,
        },
        rt,
    )
}

impl Page {
    async fn get(&self, uri: &str) -> String {
        let resp = crate::web::router(self.state.clone())
            .layer(local_proxy_peer())
            .oneshot(get_authed(uri, &self.session))
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK, "{uri}");
        body_string(resp)
            .await
            .replace("&#x2f;", "/")
            .replace("&quot;", "\"")
            .replace("&#x27;", "'")
    }
}

#[tokio::test]
async fn the_unset_network_is_said_in_the_admins_language() {
    let (p, _) = page_env(false).await;
    set_user_language(&p.pool, "test@example.com", "ru").await;
    let page = p.get(TON_PAGE).await;
    assert!(
        page.contains("Оплата выключена: не задана переменная CALRS_TON_NETWORK"),
        "{page}"
    );
    assert!(!page.contains("is not set"), "{page}");
    // The log and the agent API keep the English reason.
    assert_eq!(
        p.state.ton.disabled_reason().as_deref(),
        Some("CALRS_TON_NETWORK is not set")
    );

    set_user_language(&p.pool, "test@example.com", "en").await;
    let page = p.get(TON_PAGE).await;
    assert!(
        page.contains("Payments are off: CALRS_TON_NETWORK is not set"),
        "{page}"
    );
}

#[tokio::test]
async fn the_watchers_not_ready_reasons_are_translated() {
    let (p, rt) = page_env(true).await;
    let rt = rt.unwrap();
    set_user_language(&p.pool, "test@example.com", "ru").await;

    // After a restart, before the first check.
    let page = p.get(TON_PAGE).await;
    assert!(
        page.contains("не готов: кошелёк-получатель не активен (или ещё не проверен)"),
        "{page}"
    );
    assert!(!page.contains("receiving wallet"), "{page}");

    rt.with_health(|h| {
        h.account_status = Some("active".into());
        h.account_checked_at = Some(Utc::now());
    });
    let page = p.get(TON_PAGE).await;
    assert!(
        page.contains("не готов: индексатор в другой сети (или ещё не проверен)"),
        "{page}"
    );
    assert!(!page.contains("another network"), "{page}");

    rt.with_health(|h| h.network_ok = Some(true));
    let page = p.get(TON_PAGE).await;
    assert!(
        page.contains("не готов: наблюдатель не делал шагов последние 2 минуты"),
        "{page}"
    );
    assert!(!page.contains("has not completed"), "{page}");

    // The English Display stays for the log.
    assert_eq!(
        rt.ready_on_demand(Utc::now()).unwrap_err().to_string(),
        "the watcher has not completed a step in the last 2 minutes"
    );

    rt.with_health(|h| h.last_ok = Some(Utc::now()));
    let page = p.get(TON_PAGE).await;
    assert!(page.contains("Наблюдатель</td><td>готов"), "{page}");
}

#[test]
fn every_not_ready_reason_has_its_own_text() {
    for lang in ["en", "de", "es", "et", "fr", "it", "nl", "pl", "pt", "ru"] {
        let texts: std::collections::HashSet<String> = [
            "ton-admin-notready-wallet",
            "ton-admin-notready-network",
            "ton-admin-notready-stale",
            "ton-admin-off-unset",
        ]
        .iter()
        .map(|k| {
            let t = crate::i18n::translate(lang, k, None);
            assert!(!t.is_empty() && t != *k, "{lang} {k}");
            t
        })
        .collect();
        assert_eq!(texts.len(), 4, "{lang}");
    }
}

#[tokio::test]
async fn the_ladder_rows_have_no_wrong_number_and_named_fields() {
    let (p, _) = page_env(true).await;
    set_user_language(&p.pool, "test@example.com", "ru").await;
    let page = p.get(TON_PAGE).await;
    assert!(page.contains(r#"data-row="Бронирований: %1""#), "{page}");
    // The editor's fields and buttons are named by their row.
    assert!(page.contains("field.setAttribute('aria-label', label.textContent);"));
    assert!(page.contains(
        "remove.setAttribute('aria-label', remove.textContent + ' ' + label.textContent);"
    ));

    set_user_language(&p.pool, "test@example.com", "en").await;
    let page = p.get(TON_PAGE).await;
    assert!(page.contains(r#"data-row="Bookings: %1""#), "{page}");
    for lang in ["en", "de", "es", "et", "fr", "it", "nl", "pl", "pt", "ru"] {
        let row = crate::i18n::translate(lang, "ton-admin-ladder-row", None);
        assert!(row.contains("%1"), "{lang}: {row}");
    }
}

#[tokio::test]
async fn the_disabled_telegram_box_has_a_muted_label() {
    let (p, _) = page_env(true).await;
    sqlx::query(
        "INSERT INTO ton_settings (key, value, owner, updated_at) VALUES ('CALRS_DEBUG_BOOKING', '1', 'panel', '2026-11-12T00:00:00Z')",
    )
    .execute(&p.pool)
    .await
    .unwrap();
    let page = p.get(crate::web::debug_booking::PATH).await;
    assert!(
        page.contains("#dbg-own-chat:disabled + span { color: var(--text-muted); }"),
        "{page}"
    );
    assert!(page.contains(
        "#dbg-own-chat-group label:has(#dbg-own-chat:disabled) { cursor: not-allowed; }"
    ));
    assert!(page.contains(r#"id="dbg-own-chat" disabled"#) || crate::telegram::enabled());
}
