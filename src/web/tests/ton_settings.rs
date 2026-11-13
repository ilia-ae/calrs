//! ilia fork: the TON settings page (`/dashboard/admin/ton`): admin only
//! with CSRF, fields owned by the environment or `config apply` locked and
//! a forced POST of them ignored, the whole result validated before
//! anything is stored, the audit trail, the ladder preview, the hot apply
//! (a new runtime, the watcher's generation bumped, what the watcher
//! learned kept), the alert email through the resolver, and env > DB >
//! default end to end. A child of the TON tests, for their runtime.

use super::*;
use crate::ton::amount::NANO_PER_TON;
use crate::ton::ladder::Step;
use crate::ton::settings::{DbValues, Owner};
use crate::web::ton_pay::{alerts, store, EnvSnapshot};

const PAGE: &str = "/dashboard/admin/ton";

/// The environment `calrs serve` would have for the test runtime, with
/// `drop` left out and `extra` on top.
fn snapshot(extra: &[(&'static str, &str)], drop: &[&str]) -> EnvSnapshot {
    let v = vars(extra);
    let pairs: Vec<(&str, &str)> = v
        .iter()
        .filter(|(k, _)| !drop.contains(k))
        .map(|(k, v)| (*k, v.as_str()))
        .collect();
    EnvSnapshot::of(&pairs)
}

/// A TON environment whose slot resolves against the runtime's own env.
async fn settings_env(extra: &[(&'static str, &str)]) -> Env {
    let e = env_priced(extra, Some(100 * NANO_PER_TON)).await;
    e.state.ton.set_env(snapshot(extra, &[]));
    e
}

impl Env {
    async fn get_authed_page(&self, session: &str) -> (axum::http::StatusCode, String) {
        let resp = self.app().oneshot(get_authed(PAGE, session)).await.unwrap();
        let status = resp.status();
        (status, unescape(&body_string(resp).await))
    }

    async fn post_settings(&self, session: &str, body: &str) -> axum::http::Response<Body> {
        self.app()
            .oneshot(post_form(
                PAGE,
                session,
                CSRF,
                &format!("_csrf={CSRF}&{body}"),
            ))
            .await
            .unwrap()
    }

    async fn rows(&self) -> DbValues {
        let mut conn = self.pool.acquire().await.unwrap();
        store::load_settings(&mut conn).await.unwrap()
    }
}

/// The `<input>` of a field, as rendered.
fn input_of<'a>(page: &'a str, field: &str) -> &'a str {
    let start = page
        .find(&format!(r#"id="ton-{field}""#))
        .unwrap_or_else(|| panic!("no field {field}"));
    let start = page[..start].rfind("<input").unwrap();
    let end = start + page[start..].find('>').unwrap();
    &page[start..end]
}

#[tokio::test]
async fn only_an_admin_sees_and_saves_it_with_csrf() {
    let e = settings_env(&[]).await;
    let anon = e.app().oneshot(get(PAGE)).await.unwrap();
    assert_eq!(anon.status(), axum::http::StatusCode::SEE_OTHER);
    let (_uid, user_session) =
        insert_test_user_with_session(&e.pool, "Member", "member@example.com", "member").await;
    let (status, _) = e.get_authed_page(&user_session).await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
    let resp = e.post_settings(&user_session, "window_min=45").await;
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);

    let admin = e.session().await;
    let (status, page) = e.get_authed_page(&admin).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(page.contains("Payments are on"), "{page}");
    // The wallet in both forms, read-only, and the key only as set or not.
    assert!(page.contains(&e.rt.cfg.address_friendly));
    assert!(page.contains(&e.rt.cfg.address_raw));
    assert!(page.contains(r#"data-secret="unset""#));
    assert!(!page.contains(r#"name="network""#) && !page.contains(r#"name="address""#));
    // The nav link.
    assert!(page.contains(r#"href="/dashboard/admin/ton""#));

    // A POST without the token is refused and stores nothing.
    let resp = e
        .app()
        .oneshot(post_form(PAGE, &admin, CSRF, "window_min=45"))
        .await
        .unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    assert_eq!(e.rows().await, DbValues::default());
}

#[tokio::test]
async fn a_save_is_validated_stored_audited_and_applied_at_once() {
    let e = settings_env(&[]).await;
    let admin = e.session().await;
    let mut generation = e.state.ton.generation();
    generation.borrow_and_update();
    let before = e.state.ton.get().unwrap();

    // Invalid: nothing stored, the error next to its field, values kept.
    let resp = e
        .post_settings(&admin, "window_min=45&ladder=20%2Cfree&max_renewals=1")
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let page = unescape(&body_string(resp).await);
    assert!(page.contains("Nothing was saved"), "{page}");
    assert!(page.contains("free may only come first"), "{page}");
    assert!(input_of(&page, "window_min").contains(r#"value="45""#));
    assert_eq!(e.rows().await, DbValues::default());
    assert!(!generation.has_changed().unwrap());

    // Valid.
    let resp = e
        .post_settings(
            &admin,
            "window_min=45&ladder=free%2C+10%2C20&underpay_tolerance_ton=0.01&grace_min=&action=save",
        )
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    assert_eq!(location(&resp), format!("{PAGE}?saved=live"));
    let rows = e.rows().await;
    assert_eq!(rows.value("CALRS_TON_WINDOW_MIN"), Some("45"));
    assert_eq!(rows.owner("CALRS_TON_WINDOW_MIN"), Some(Owner::Panel));
    assert_eq!(rows.value("CALRS_TON_LADDER"), Some("free,10,20"));
    assert_eq!(
        rows.value("CALRS_TON_UNDERPAY_TOLERANCE_NANO"),
        Some("10000000")
    );
    assert_eq!(rows.value("CALRS_TON_GRACE_MIN"), None, "blank: default");
    let audited: Vec<(
        String,
        Option<String>,
        Option<String>,
        String,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT key, before, after, source, actor_user_id FROM ton_settings_audit ORDER BY id",
    )
    .fetch_all(&e.pool)
    .await
    .unwrap();
    let keys: Vec<&str> = audited.iter().map(|a| a.0.as_str()).collect();
    assert_eq!(
        keys,
        [
            "CALRS_TON_WINDOW_MIN",
            "CALRS_TON_UNDERPAY_TOLERANCE_NANO",
            "CALRS_TON_LADDER"
        ]
    );
    assert!(audited
        .iter()
        .all(|a| a.1.is_none() && a.3 == "panel" && a.4.is_some()));

    // In effect now: a new runtime, the watcher told, its health kept.
    assert!(generation.has_changed().unwrap());
    let after = e.state.ton.get().unwrap();
    assert!(!Arc::ptr_eq(&before, &after));
    assert_eq!(after.cfg.window_min, 45);
    assert_eq!(after.cfg.ladder.text(), "free,10,20");
    assert_eq!(after.cfg.underpay_tolerance_nano, 10_000_000);
    assert!(
        after.ready(e.now).is_ok(),
        "what the watcher learned is kept"
    );

    // The page shows the values, their source, the preview and the audit.
    let (_, page) = e.get_authed_page(&admin).await;
    assert!(input_of(&page, "window_min").contains(r#"value="45""#));
    assert!(input_of(&page, "underpay_tolerance_ton").contains(r#"value="0.01""#));
    assert!(page.contains(r#"data-field="window_min" data-source="panel""#));
    assert!(page.contains(r#"data-field="grace_min" data-source="default""#));
    let preview = &page[page.find(r#"id="ton-ladder-preview""#).unwrap()..];
    let preview = &preview[..preview.find("</table>").unwrap()];
    assert!(preview.contains("<td>free</td>") && preview.contains("<td>10 TON</td>"));
    assert!(preview.contains("2 or more"), "{preview}");
    assert!(page.contains("Your bookings ahead now: 0. Your ladder price now: free."));
    assert!(page.contains("free,10,20"));
    assert!(
        page.contains("test@example.com"),
        "the audit names the admin"
    );

    // Saving the same again changes nothing.
    let resp = e
        .post_settings(
            &admin,
            "window_min=45&ladder=free%2C10%2C20&underpay_tolerance_ton=0.01",
        )
        .await;
    assert_eq!(location(&resp), format!("{PAGE}?saved=none"));
}

#[tokio::test]
async fn fields_the_environment_or_apply_own_are_locked() {
    let e = settings_env(&[("CALRS_TON_GRACE_MIN", "20")]).await;
    let mut db = DbValues::default();
    db.set("CALRS_TON_FUSE_HOURS", "8".into(), Owner::Apply);
    {
        let mut conn = e.pool.acquire().await.unwrap();
        let changes = crate::web::ton_pay::settings::diff(&DbValues::default(), &db);
        crate::web::ton_pay::settings::write(&mut conn, &changes, None, Owner::Apply, e.now)
            .await
            .unwrap();
    }
    let admin = e.session().await;
    let (_, page) = e.get_authed_page(&admin).await;
    let grace = input_of(&page, "grace_min");
    assert!(
        grace.contains("disabled") && grace.contains(r#"value="20""#),
        "{grace}"
    );
    assert!(page.contains("set by the environment (CALRS_TON_GRACE_MIN)"));
    let fuse = input_of(&page, "fuse_hours");
    assert!(
        fuse.contains("disabled") && fuse.contains(r#"value="8""#),
        "{fuse}"
    );
    assert!(page.contains("set by config apply"));
    assert!(!input_of(&page, "window_min").contains("disabled"));

    // A forced POST of the locked fields is ignored.
    let resp = e
        .post_settings(&admin, "grace_min=30&fuse_hours=12&renew_window_min=400")
        .await;
    assert_eq!(location(&resp), format!("{PAGE}?saved=live"));
    let rows = e.rows().await;
    assert_eq!(rows.value("CALRS_TON_GRACE_MIN"), None);
    assert_eq!(rows.value("CALRS_TON_FUSE_HOURS"), Some("8"));
    assert_eq!(rows.owner("CALRS_TON_FUSE_HOURS"), Some(Owner::Apply));
    assert_eq!(rows.value("CALRS_TON_RENEW_WINDOW_MIN"), Some("400"));
    let rt = e.state.ton.get().unwrap();
    assert_eq!(
        (rt.cfg.grace_min, rt.cfg.fuse_hours, rt.cfg.renew_window_min),
        (20, 8, 400),
        "env > DB > default"
    );
}

/// The whole merged result is checked: half a corridor is refused when
/// the environment sets none, and nothing is stored.
#[tokio::test]
async fn the_merged_result_is_checked_before_anything_is_stored() {
    let e = settings_env(&[]).await;
    e.state.ton.set_env(snapshot(
        &[],
        &["CALRS_TON_RATE_MIN_USD", "CALRS_TON_RATE_MAX_USD"],
    ));
    let admin = e.session().await;
    let resp = e.post_settings(&admin, "rate_min_usd=0.3").await;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let page = unescape(&body_string(resp).await);
    let field = &page[page.find(r#"data-field="rate_min_usd""#).unwrap()..];
    let field = &field[..field.find("</div>").unwrap()];
    assert!(field.contains("go together"), "{field}");
    assert_eq!(e.rows().await, DbValues::default());

    let resp = e
        .post_settings(&admin, "rate_min_usd=0.30&rate_max_usd=5")
        .await;
    assert_eq!(location(&resp), format!("{PAGE}?saved=live"));
    let rows = e.rows().await;
    assert_eq!(rows.value("CALRS_TON_RATE_MIN_USD"), Some("0.3"));
    let policy = e.state.ton.get().unwrap().cfg.rate_policy.clone().unwrap();
    assert_eq!(policy.min_usd_micro, 300_000);
}

#[tokio::test]
async fn a_preview_shows_the_ladder_and_stores_nothing() {
    let e = settings_env(&[]).await;
    let admin = e.session().await;
    let resp = e
        .post_settings(&admin, "ladder=5%2C7.5&action=preview")
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let page = unescape(&body_string(resp).await);
    assert!(page.contains(r#"id="ton-previewing""#));
    let preview = &page[page.find(r#"id="ton-ladder-preview""#).unwrap()..];
    let preview = &preview[..preview.find("</table>").unwrap()];
    assert!(preview.contains("<td>5 TON</td>") && preview.contains("<td>7.5 TON</td>"));
    assert!(preview.contains("1 or more"));
    assert!(input_of(&page, "ladder").contains(r#"value="5,7.5""#));
    assert_eq!(e.rows().await, DbValues::default());
    assert_eq!(
        e.state.ton.get().unwrap().cfg.ladder,
        crate::ton::ladder::Ladder::default()
    );
}

/// A ladder saved on the page prices the next booking without a restart;
/// with payments off it is stored and the event type form reads it.
#[tokio::test]
async fn the_saved_ladder_prices_the_next_booking() {
    let e = settings_env(&[]).await;
    sqlx::query(
        "UPDATE ton_prices SET mode = 'ladder', currency = 'TON', price_nano = NULL,
             price_usd_cents = NULL",
    )
    .execute(&e.pool)
    .await
    .unwrap();
    let admin = e.session().await;
    let resp = e.post_settings(&admin, "ladder=30%2C40").await;
    assert_eq!(location(&resp), format!("{PAGE}?saved=live"));
    let ask = crate::web::ton_pay::invoice::BookingAsk {
        event_type_id: &e.et_id,
        start_utc: e.now + Duration::days(3),
        min_notice_min: 0,
        guest_email: "g@example.com",
        client_ip: "203.0.113.9",
        telegram: None,
        quoted: Some(Step::Ton(30 * NANO_PER_TON)),
    };
    let prepared = invoice::prepare(&e.state, &ask, e.now).await.unwrap();
    let crate::web::ton_pay::invoice::Prepared::Invoice(inv) = prepared else {
        panic!("a priced step opens an invoice");
    };
    assert_eq!(inv.amount_nano, (30 * NANO_PER_TON) as i64);

    // Payments off: stored for later, read from the database.
    e.state.ton.set_env(EnvSnapshot::default());
    let resp = e.post_settings(&admin, "ladder=free%2C50").await;
    assert_eq!(location(&resp), format!("{PAGE}?saved=off"));
    e.state.ton.uninstall();
    let ladder = crate::web::ton_pay::pricing::current_ladder(&e.state)
        .await
        .unwrap();
    assert_eq!(ladder.text(), "free,50");
}

/// The alert email comes from the resolver: the environment's, else the
/// stored one, else none (the admins get the alert).
#[tokio::test]
async fn the_alert_email_goes_through_the_resolver() {
    let e = settings_env(&[]).await;
    let none = |_: &str| None;
    assert_eq!(alerts::alert_email_with(&e.pool, &none).await, None);
    let admin = e.session().await;
    let resp = e
        .post_settings(&admin, "alert_email=ops%40example.com")
        .await;
    assert_eq!(location(&resp), format!("{PAGE}?saved=live"));
    assert_eq!(
        alerts::alert_email_with(&e.pool, &none).await.as_deref(),
        Some("ops@example.com")
    );
    let env = |k: &str| (k == "CALRS_TON_ALERT_EMAIL").then(|| "env@example.com".to_string());
    assert_eq!(
        alerts::alert_email_with(&e.pool, &env).await.as_deref(),
        Some("env@example.com")
    );
    assert_eq!(
        e.state.ton.get().unwrap().cfg.alert_email.as_deref(),
        Some("ops@example.com")
    );
}

/// A hot apply while the watcher's loop runs: until the supervisor starts
/// the new runtime, it counts as running (a booking in that gap wakes it
/// and is taken, not refused as "not ready"); and what the old loop's last
/// step learned after the swap (health, payments seen) is carried over
/// when the supervisor hands over. Without a running loop nothing is
/// pretended.
#[tokio::test]
async fn a_hot_apply_keeps_the_watcher_ready_and_its_last_step() {
    use std::sync::atomic::Ordering;
    let e = settings_env(&[]).await;
    let old = e.state.ton.get().unwrap();
    old.looping.store(true, Ordering::Release);
    // Idle: the heartbeat is older than a new invoice needs.
    old.with_health(|h| h.last_ok = Some(e.now - Duration::minutes(10)));
    assert!(old.ready(e.now).is_err());
    assert!(old.ready_on_demand(e.now).is_ok());

    let next = old.retuned(old.cfg.clone()).expect("the same wallet");
    assert!(next.looping.load(Ordering::Acquire), "counts as running");
    assert!(next.idle_but_well(e.now));
    assert!(next.ready_on_demand(e.now).is_ok());

    // The old loop's step ends after the swap: it saw a payment and
    // refreshed the heartbeat on the old runtime.
    old.with_health(|h| h.last_ok = Some(e.now));
    old.seen
        .lock()
        .unwrap()
        .insert("inv-seen".into(), std::time::Instant::now());
    assert!(next.ready(e.now).is_err(), "not seen by the new one yet");
    next.take_over(&old);
    assert!(next.ready(e.now).is_ok(), "the last heartbeat carried over");
    assert!(next.seen.lock().unwrap().contains_key("inv-seen"));

    old.looping.store(false, Ordering::Release);
    let idle = old.retuned(old.cfg.clone()).unwrap();
    assert!(!idle.looping.load(Ordering::Acquire));
}

/// The supervisor end to end: a save while the loop runs restarts it on
/// the new runtime, which is running from the swap on and steps on its own.
#[tokio::test]
async fn the_supervisor_restarts_on_the_new_runtime() {
    use std::sync::atomic::Ordering;
    let e = settings_env(&[]).await;
    let old = e.state.ton.get().unwrap();
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let handle = tokio::spawn(crate::web::ton_pay::watcher::supervise(
        e.state.clone(),
        stop_rx,
        std::time::Duration::from_millis(5),
    ));
    let mut steps = old.steps.subscribe();
    tokio::time::timeout(std::time::Duration::from_secs(5), steps.changed())
        .await
        .expect("a first step")
        .unwrap();
    let mut db = DbValues::default();
    db.set("CALRS_TON_WINDOW_MIN", "45".into(), Owner::Panel);
    assert!(matches!(
        e.state.ton.apply_settings(&db).await,
        crate::web::ton_pay::Applied::Live
    ));
    let next = e.state.ton.get().unwrap();
    assert!(!Arc::ptr_eq(&old, &next));
    assert!(
        next.looping.load(Ordering::Acquire),
        "running from the swap"
    );
    let mut next_steps = next.steps.subscribe();
    next.wake.notify_one();
    tokio::time::timeout(std::time::Duration::from_secs(5), next_steps.changed())
        .await
        .expect("the new runtime steps")
        .unwrap();
    assert!(!old.looping.load(Ordering::Acquire), "the old loop ended");
    assert!(next.looping.load(Ordering::Acquire));
    stop_tx.send(true).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), handle)
        .await
        .expect("the supervisor stops")
        .unwrap();
}

/// The page says a key comes from systemd credentials only when
/// `load_into_env` really loaded it: a variable already set in calrs.env
/// or the unit wins, its credential file is left unread, and the page
/// says "env".
#[test]
fn a_key_is_from_credentials_only_when_loaded_from_them() {
    use crate::credentials::{Outcome, Report};
    let report = Report {
        missing_dir: false,
        names: vec![
            ("CALRS_TON_API_KEY", Outcome::AlreadySet),
            ("CALRS_TON_COINGECKO_KEY", Outcome::Loaded),
        ],
    };
    assert_eq!(report.loaded(), ["CALRS_TON_COINGECKO_KEY"]);
    let env = EnvSnapshot::of(&[
        ("CALRS_TON_API_KEY", "from-calrs-env"),
        ("CALRS_TON_COINGECKO_KEY", "from-credential"),
    ])
    .loaded_from(&report.loaded());
    assert_eq!(env.secret_state("CALRS_TON_API_KEY"), "env");
    assert_eq!(env.secret_state("CALRS_TON_COINGECKO_KEY"), "credentials");
    let nothing = EnvSnapshot::of(&[]).loaded_from(&report.loaded());
    assert_eq!(nothing.secret_state("CALRS_TON_API_KEY"), "unset");
}

#[tokio::test]
async fn the_watchers_last_step_is_shown_in_the_admins_zone() {
    let e = settings_env(&[]).await;
    sqlx::query("UPDATE users SET timezone = 'Asia/Dubai' WHERE email = 'test@example.com'")
        .execute(&e.pool)
        .await
        .unwrap();
    let at = chrono::DateTime::parse_from_rfc3339("2026-11-12T19:36:27Z")
        .unwrap()
        .with_timezone(&Utc);
    e.rt.with_health(|h| h.last_ok = Some(at));
    let admin = e.session().await;
    let (status, page) = e.get_authed_page(&admin).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    // The stored UTC stamp only in the attribute; the text in Dubai (+4).
    assert!(
        page.contains(r#"<time datetime="2026-11-12T19:36:27Z">2026-11-12 23:36:27</time>"#),
        "{page}"
    );
}
