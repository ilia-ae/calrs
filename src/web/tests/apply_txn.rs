//! ilia fork: the transaction spike of `config apply`, and its pipeline.
//!
//! The question (docs/apply-schema.md, "Transaction"): can the real web
//! handlers run inside one `BEGIN IMMEDIATE` issued on a pool of one
//! connection, so a run commits or rolls back as a whole? These tests post
//! real forms to `settings_save`, `admin_update_accent`,
//! `create_event_type` and `update_event_type` inside one `apply::Txn`.

use super::apply_support::*;
use super::*;
use crate::web::apply::{self, ApplyError, Ctx, Txn};
use axum::extract::{Path, State};
use axum::response::IntoResponse;

/// The admin as the run loads it.
async fn admin(pool: &SqlitePool) -> crate::models::User {
    apply::admin(pool, Some(ADMIN_EMAIL)).await.unwrap()
}

fn fields(pairs: &[(&'static str, &str)]) -> Vec<(&'static str, String)> {
    pairs.iter().map(|(k, v)| (*k, v.to_string())).collect()
}

/// Settings → Profile, as a browser posts it.
async fn post_profile(ctx: &Ctx) -> axum::response::Response {
    let (headers, form) = apply::form(&fields(&[
        ("name", "Spike Name"),
        ("username", "spike"),
        ("title", "Fractional CTO"),
        ("timezone", "Asia/Almaty"),
        ("language", "ru"),
        ("public_page_language", "visitor"),
        ("week_start", ""),
        ("avail_schedule", "1:10:00-19:00;2:10:00-19:00"),
    ]))
    .await
    .unwrap();
    crate::web::dashboard::settings_save(State(ctx.state.clone()), ctx.auth_user(), headers, form)
        .await
        .into_response()
}

/// Admin → Theme: the custom theme.
async fn post_theme(ctx: &Ctx) -> axum::response::Response {
    let (headers, form) = apply::form(&fields(&[
        ("theme", "custom"),
        ("custom_accent", "#313131"),
        ("custom_accent_hover", "#111111"),
        ("custom_bg", "#f6f6f6"),
        ("custom_surface", "#ffffff"),
        ("custom_text", "#313131"),
    ]))
    .await
    .unwrap();
    crate::web::admin::admin_update_accent(
        State(ctx.state.clone()),
        ctx.admin_user(),
        headers,
        form,
    )
    .await
    .into_response()
}

/// A new event type form for `slug`, `duration` minutes.
async fn event_type_form(
    slug: &str,
    duration: &str,
) -> (
    HeaderMap,
    axum::extract::Form<crate::web::event_types::EventTypeForm>,
) {
    apply::form(&fields(&[
        ("title", "Spike call"),
        ("slug", slug),
        ("duration_min", duration),
        ("location_type", "custom"),
        ("location_value", "The link comes by email"),
        ("visibility", "public"),
        ("avail_schedule", "1:10:00-19:00"),
        ("min_notice_min", "240"),
        ("buffer_before", "0"),
        ("buffer_after", "15"),
        ("reminder_minutes", "60"),
        ("max_additional_guests", "1"),
        ("default_calendar_view", "month"),
    ]))
    .await
    .unwrap()
}

async fn post_new_event_type(ctx: &Ctx) -> axum::response::Response {
    let (headers, form) = event_type_form("spike", "15").await;
    crate::web::event_types::create_event_type(
        State(ctx.state.clone()),
        ctx.auth_user(),
        headers,
        form,
    )
    .await
    .into_response()
}

/// An edit the web form refuses: a duration of 0.
async fn post_refused_edit(ctx: &Ctx) -> axum::response::Response {
    let (headers, form) = event_type_form("spike", "0").await;
    crate::web::event_types::update_event_type(
        State(ctx.state.clone()),
        ctx.auth_user(),
        headers,
        Path("spike".to_string()),
        form,
    )
    .await
    .into_response()
}

/// Three sections' handlers inside one transaction on the one connection,
/// as `apply::run` builds it.
async fn begin_with_three_sections(env: &ApplyEnv) -> (SqlitePool, Txn, Ctx) {
    let pool = env.single(false).await;
    let state = crate::web::build_state(pool.clone(), env.dir.clone(), [0u8; 32]).await;
    let txn = Txn::begin(&pool).await.unwrap();
    let mut ctx = Ctx::new(state, admin(&pool).await, false);
    apply::answered("profile", post_profile(&ctx).await)
        .await
        .unwrap();
    ctx.reload_me().await.unwrap();
    txn.check(&pool).await.unwrap();
    apply::answered("site", post_theme(&ctx).await)
        .await
        .unwrap();
    txn.check(&pool).await.unwrap();
    apply::answered("event_types[spike]", post_new_event_type(&ctx).await)
        .await
        .unwrap();
    txn.check(&pool).await.unwrap();
    (pool, txn, ctx)
}

async fn stored_name(pool: &SqlitePool, admin_id: &str) -> String {
    sqlx::query_scalar("SELECT name FROM users WHERE id = ?")
        .bind(admin_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn theme(pool: &SqlitePool) -> String {
    sqlx::query_scalar("SELECT theme FROM auth_config WHERE id = 'singleton'")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn event_type_count(pool: &SqlitePool, account_id: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM event_types WHERE account_id = ?")
        .bind(account_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// The spike: a refusal in the last section rolls back every handler's
/// write before it, byte for byte; until then another connection (a running
/// `calrs serve`) saw none of them, while the run's own connection saw all.
#[tokio::test]
async fn a_refusal_in_the_last_section_leaves_the_database_unchanged() {
    let env = ApplyEnv::new().await;
    let before = env.snapshot().await;

    let (pool, txn, ctx) = begin_with_three_sections(&env).await;
    assert_eq!(stored_name(&pool, &env.admin_id).await, "Spike Name");
    assert_eq!(theme(&pool).await, "custom");
    assert_eq!(event_type_count(&pool, &env.account_id).await, 1);
    // Another process reads the last commit (WAL): nothing of the run.
    assert_eq!(stored_name(&env.pool, &env.admin_id).await, ADMIN_EMAIL);
    assert_eq!(event_type_count(&env.pool, &env.account_id).await, 0);

    let Err(error) = apply::answered("event_types[spike]", post_refused_edit(&ctx).await).await
    else {
        panic!("a duration of 0 is refused");
    };
    assert!(matches!(error, ApplyError::Refused { .. }), "{error}");
    assert!(
        error.to_string().contains("from 1 to 1440"),
        "the form's own message: {error}"
    );

    txn.rollback(&pool).await.unwrap();
    pool.close().await;
    assert_eq!(env.snapshot().await, before);
}

/// The same handlers, committed: every write is kept, and is what another
/// connection reads afterwards.
#[tokio::test]
async fn a_commit_keeps_every_section() {
    let env = ApplyEnv::new().await;
    let (pool, txn, _ctx) = begin_with_three_sections(&env).await;
    txn.commit(&pool).await.unwrap();
    pool.close().await;

    assert_eq!(stored_name(&env.pool, &env.admin_id).await, "Spike Name");
    assert_eq!(theme(&env.pool).await, "custom");
    assert_eq!(event_type_count(&env.pool, &env.account_id).await, 1);
    let rules: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_availability_rules WHERE user_id = ?")
            .bind(&env.admin_id)
            .fetch_one(&env.pool)
            .await
            .unwrap();
    assert_eq!(rules, 2, "the profile's default hours");
}

/// A handler that begins a transaction of its own fails at once instead of
/// waiting for the one connection: SQLite refuses a nested BEGIN. The run's
/// transaction stays open and still rolls back.
#[tokio::test]
async fn a_nested_begin_fails_fast_and_keeps_the_run_transaction() {
    let env = ApplyEnv::new().await;
    let before = env.snapshot().await;
    let pool = env.single(false).await;
    let txn = Txn::begin(&pool).await.unwrap();
    sqlx::query("UPDATE users SET name = 'Inside' WHERE id = ?")
        .bind(&env.admin_id)
        .execute(&pool)
        .await
        .unwrap();

    let nested = tokio::time::timeout(std::time::Duration::from_secs(5), pool.begin())
        .await
        .expect("no wait for the connection");
    let Err(error) = nested else {
        panic!("a nested BEGIN is refused");
    };
    let error = error.to_string();
    assert!(error.contains("within a transaction"), "{error}");

    txn.check(&pool).await.unwrap();
    assert_eq!(stored_name(&pool, &env.admin_id).await, "Inside");
    txn.rollback(&pool).await.unwrap();
    pool.close().await;
    assert_eq!(env.snapshot().await, before);
}

/// A transaction that ended under the run (SQLite rolls back on its own
/// after a disk-full or I/O error; here a stray ROLLBACK) is caught by
/// `Txn::check`, so the run stops instead of writing outside it. `run`
/// checks after every section and before the commit.
#[tokio::test]
async fn a_lost_transaction_is_detected() {
    let env = ApplyEnv::new().await;
    let pool = env.single(false).await;
    let txn = Txn::begin(&pool).await.unwrap();
    sqlx::raw_sql("ROLLBACK").execute(&pool).await.unwrap();
    let error = txn.check(&pool).await.unwrap_err();
    assert!(matches!(error, ApplyError::Internal(_)), "{error}");
    assert!(txn.commit(&pool).await.is_err(), "nothing to commit");
    pool.close().await;
}

/// A second writer (a running `calrs serve`) waits while the run holds its
/// `BEGIN IMMEDIATE`, and writes once it ends: no lost update either way.
#[tokio::test]
async fn another_writer_waits_for_the_run() {
    let env = ApplyEnv::new().await;
    let pool = env.single(false).await;
    let txn = Txn::begin(&pool).await.unwrap();
    let other = env.pool.clone();
    let admin_id = env.admin_id.clone();
    let writer = tokio::spawn(async move {
        sqlx::query("UPDATE users SET title = 'From serve' WHERE id = ?")
            .bind(&admin_id)
            .execute(&other)
            .await
    });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert!(!writer.is_finished(), "the other writer waits for the lock");
    sqlx::query("UPDATE users SET name = 'From apply' WHERE id = ?")
        .bind(&env.admin_id)
        .execute(&pool)
        .await
        .unwrap();
    txn.commit(&pool).await.unwrap();
    writer.await.unwrap().unwrap();
    pool.close().await;
    let (name, title): (String, Option<String>) =
        sqlx::query_as("SELECT name, title FROM users WHERE id = ?")
            .bind(&env.admin_id)
            .fetch_one(&env.pool)
            .await
            .unwrap();
    assert_eq!(
        (name.as_str(), title.as_deref()),
        ("From apply", Some("From serve"))
    );
}

/// `mode=ro` refuses writes, so `--dry-run`, which runs the handlers, uses
/// a writable connection and always rolls back; `--export` reads only and
/// opens `mode=ro`.
#[tokio::test]
async fn a_read_only_pool_refuses_writes() {
    let env = ApplyEnv::new().await;
    let pool = env.single(true).await;
    let write = sqlx::query("UPDATE users SET name = 'x' WHERE id = ?")
        .bind(&env.admin_id)
        .execute(&pool)
        .await;
    assert!(write.is_err(), "read-only");
    assert_eq!(stored_name(&pool, &env.admin_id).await, ADMIN_EMAIL);
    pool.close().await;
}

/// `Txn` refuses a pool that is not the run's single connection: on a
/// larger one the handlers' statements would run outside the transaction.
#[tokio::test]
async fn a_pool_of_several_connections_is_refused() {
    let env = ApplyEnv::new().await;
    let Err(error) = Txn::begin(&env.pool).await else {
        panic!("refused");
    };
    assert!(
        error.to_string().contains("exactly one connection"),
        "{error}"
    );
}

// --- The pipeline ---

#[tokio::test]
async fn an_empty_document_changes_nothing() {
    let env = ApplyEnv::new().await;
    let before = env.snapshot().await;
    for dry_run in [true, false] {
        let report = env.apply(&empty_doc(), dry_run).await.unwrap();
        assert_eq!(report.format, apply::FORMAT);
        assert_eq!(report.dry_run, dry_run);
        assert!(report.changed.is_empty());
        assert!(!report.restart);
        assert!(report.unmanaged.event_types.is_empty());
    }
    assert_eq!(env.snapshot().await, before);
}

/// The report is one line of JSON with exactly the documented keys.
#[test]
fn the_report_is_one_json_line() {
    let report = apply::Report {
        format: apply::FORMAT.to_string(),
        dry_run: false,
        changed: vec![apply::Change {
            path: "site.theme".to_string(),
            before: serde_json::json!("default"),
            after: serde_json::json!("custom"),
        }],
        restart: true,
        unmanaged: apply::Unmanaged::default(),
    };
    let line = serde_json::to_string(&report).unwrap();
    assert!(!line.contains('\n'));
    assert_eq!(
        line,
        r#"{"format":"calrs-apply-v1","dry_run":false,"changed":[{"path":"site.theme","before":"default","after":"custom"}],"restart":true,"unmanaged":{"event_types":[]}}"#
    );
}

fn invalid_path(input: &str) -> String {
    match apply::parse(input) {
        Err(ApplyError::Invalid { path, .. }) => path,
        other => panic!("not refused as invalid: {other:?}"),
    }
}

#[test]
fn the_format_is_checked_first() {
    assert_eq!(invalid_path(r#"{"user":"a@b"}"#), "format");
    assert_eq!(
        invalid_path(r#"{"format":"calrs-apply-v2","user":"a@b","new":1}"#),
        "format"
    );
    assert_eq!(invalid_path("not json"), "(document)");
}

#[test]
fn unknown_keys_are_refused_at_every_level() {
    let f = apply::FORMAT;
    for doc in [
        format!(r#"{{"format":"{f}","user":"a@b","extra":1}}"#),
        format!(r#"{{"format":"{f}","user":"a@b","profile":{{"nickname":"x"}}}}"#),
        // Environment-owned settings are not keys of the document.
        format!(r#"{{"format":"{f}","user":"a@b","site":{{"base_url":"https://x"}}}}"#),
        format!(
            r##"{{"format":"{f}","user":"a@b","site":{{"custom":{{"accent":"#000000","accent_hover":"#000000","bg":"#000000","surface":"#000000","text":"#000000","link":"#000000"}}}}}}"##
        ),
        format!(
            r#"{{"format":"{f}","user":"a@b","site":{{"jitsi":{{"base_url":"https://meet.jit.si","pattern":null,"display_name":null,"x":1}}}}}}"#
        ),
        format!(
            r#"{{"format":"{f}","user":"a@b","sources":[{{"url":"u","username":"n","password":"p"}}]}}"#
        ),
        format!(
            r#"{{"format":"{f}","user":"a@b","event_types":{{"items":[{{"slug":"a","price":1}}]}}}}"#
        ),
        format!(r#"{{"format":"{f}","user":"a@b","event_types":{{"items":[],"other":1}}}}"#),
    ] {
        assert_eq!(invalid_path(&doc), "(document)", "{doc}");
    }
}

#[test]
fn null_is_refused_where_nothing_can_be_cleared() {
    let f = apply::FORMAT;
    for doc in [
        format!(r#"{{"format":"{f}","user":"a@b","profile":null}}"#),
        format!(r#"{{"format":"{f}","user":"a@b","profile":{{"name":null}}}}"#),
        format!(r#"{{"format":"{f}","user":"a@b","profile":{{"timezone":null}}}}"#),
        format!(r#"{{"format":"{f}","user":"a@b","site":{{"theme":null}}}}"#),
        format!(
            r#"{{"format":"{f}","user":"a@b","sources":[{{"url":"u","username":"n","sync_calendars":null}}]}}"#
        ),
        format!(r#"{{"format":"{f}","user":"a@b","event_types":{{"prune":null,"items":[]}}}}"#),
        format!(
            r#"{{"format":"{f}","user":"a@b","event_types":{{"items":[{{"slug":"a","duration_min":null}}]}}}}"#
        ),
        // Jitsi's keys must all be given.
        format!(
            r#"{{"format":"{f}","user":"a@b","site":{{"jitsi":{{"base_url":"https://meet.jit.si"}}}}}}"#
        ),
        // Keywords are closed sets.
        format!(r#"{{"format":"{f}","user":"a@b","site":{{"theme":"Custom"}}}}"#),
        format!(r#"{{"format":"{f}","user":"a@b","site":{{"oidc":"on"}}}}"#),
    ] {
        assert_eq!(invalid_path(&doc), "(document)", "{doc}");
    }
}

/// Left out leaves as is, null clears: the two read differently.
#[test]
fn left_out_and_null_differ() {
    let f = apply::FORMAT;
    let doc = apply::parse(&format!(
        r#"{{"format":"{f}","user":"a@b","profile":{{"title":null}},"site":{{"jitsi":"off","company_link":"https://ilia.ae"}},"event_types":{{"items":[{{"slug":"15min","timezone":null,"weekly_hours":null,"week_start":"mon"}}]}}}}"#
    ))
    .unwrap();
    let profile = doc.profile.as_ref().unwrap();
    assert_eq!(profile.title, Some(None));
    assert_eq!(profile.bio, None);
    let site = doc.site.as_ref().unwrap();
    assert_eq!(site.jitsi, Some(apply::Jitsi::Off(apply::Off::Off)));
    assert_eq!(site.company_link, Some(Some("https://ilia.ae".to_string())));
    let events = doc.event_types.as_ref().unwrap();
    assert_eq!(events.prune(), apply::Prune::None);
    let item = &events.items[0];
    assert_eq!(item.timezone, Some(None));
    assert_eq!(item.weekly_hours, Some(None));
    assert_eq!(item.week_start, Some(Some(apply::Day::Mon)));
    assert_eq!(item.title, None);
    // Written back, the same keys and nulls come out.
    let again = apply::parse(&serde_json::to_string(&doc).unwrap()).unwrap();
    assert_eq!(again, doc);
}

#[tokio::test]
async fn the_user_must_be_an_enabled_admin() {
    let env = ApplyEnv::new().await;
    insert_role_user(&env.pool, "member@example.com", "user").await;
    let f = apply::FORMAT;
    for user in [
        "nobody@example.com",
        "member@example.com",
        " admin@example.com",
    ] {
        let doc = format!(r#"{{"format":"{f}","user":"{user}"}}"#);
        match env.apply(&doc, false).await {
            Err(ApplyError::Invalid { path, .. }) => assert_eq!(path, "user"),
            other => panic!("{user}: {other:?}"),
        }
    }
}

#[test]
fn the_document_is_at_most_a_mebibyte() {
    let big = vec![b' '; apply::DOCUMENT_MAX_BYTES as usize + 1];
    assert!(apply::read_document(&big[..]).is_err());
    assert_eq!(apply::read_document(&b"{}"[..]).unwrap(), "{}");
}

/// The export is the stored state in the document's own format.
#[tokio::test]
async fn the_export_names_the_format_and_the_admin() {
    let env = ApplyEnv::new().await;
    let before = env.snapshot().await;
    let doc = env.export().await.unwrap();
    assert_eq!(doc.format, apply::FORMAT);
    assert_eq!(doc.user, ADMIN_EMAIL);
    assert_eq!(env.snapshot().await, before);
}

#[test]
fn canonical_text() {
    assert!(apply::check_text("p", "ok", Some(2)).is_ok());
    assert!(apply::check_text("p", "", None).is_err());
    assert!(apply::check_text("p", " x", None).is_err());
    assert!(apply::check_text("p", "x\n", None).is_err());
    assert!(apply::check_text("p", "abc", Some(2)).is_err());
    assert!(apply::check_opt_text("p", None, None).is_ok());
    assert!(apply::check_timezone("p", "Asia/Almaty").is_ok());
    assert!(apply::check_timezone("p", "Asia/Almty").is_err());
}
