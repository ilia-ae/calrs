//! ilia fork: 3CX video meetings (`web::threecx`), against a mock PBX that
//! answers like the owner's 3CX did on 2026-11-13.

use super::*;
use crate::web::threecx::{self, ApiError, ThreeCxConfig};
use serde_json::{json, Value};
use std::collections::HashMap;

const KEY: &str = "test-video-conference-key-0123456789";
/// The key the deployment sets (`CALRS_THREECX_API_KEY`), another one.
const ENV_KEY: &str = "deployment-video-conference-key-9876543210";
const KEY32: [u8; 32] = [0u8; 32];

#[derive(Default)]
struct Pbx {
    /// (method, path, the key it was given, the JSON body).
    calls: Vec<(String, String, Option<String>, Value)>,
    meetings: HashMap<String, Value>,
    next: u32,
    /// Answer every call as a refused key.
    refuse_key: bool,
    /// Hand out an organizer link on another host.
    foreign_link: bool,
    /// Answer every call with a 502 and an HTML page.
    down: bool,
    /// The key it accepts, when not [`KEY`].
    accept: Option<String>,
}

type SharedPbx = Arc<std::sync::Mutex<Pbx>>;

/// A mock 3CX on 127.0.0.1: its base URL and its state.
async fn spawn_pbx() -> (String, SharedPbx) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let pbx: SharedPbx = Arc::default();
    let state = (base.clone(), pbx.clone());
    let app = axum::Router::new().fallback(
        move |method: axum::http::Method,
              uri: axum::http::Uri,
              headers: HeaderMap,
              body: axum::body::Bytes| {
            let (base, pbx) = state.clone();
            async move { answer(&base, &pbx, method, uri, headers, body) }
        },
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (base, pbx)
}

fn answer(
    base: &str,
    pbx: &SharedPbx,
    method: axum::http::Method,
    uri: axum::http::Uri,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let mut p = pbx.lock().unwrap();
    let json_body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    let key = headers
        .get("3CX-ApiKey")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    p.calls.push((
        method.to_string(),
        uri.path().to_string(),
        key.clone(),
        json_body.clone(),
    ));
    let reply = |status: u16, v: Value| {
        (
            axum::http::StatusCode::from_u16(status).unwrap(),
            axum::Json(v),
        )
            .into_response()
    };
    let error = |status: u16, info: &str| {
        reply(status, json!({"status": "error", "errorInformation": info}))
    };
    if p.down {
        return (
            axum::http::StatusCode::BAD_GATEWAY,
            "<html>Bad gateway</html>",
        )
            .into_response();
    }
    if p.refuse_key || key.as_deref() != Some(p.accept.as_deref().unwrap_or(KEY)) {
        return error(401, "API Authentication Failed");
    }
    let path = uri.path().trim_start_matches("/webmeeting/api/v1");
    let id_of = |path: &str| path.trim_start_matches("/scheduled/").to_string();
    match (method.as_str(), path) {
        ("POST", "/scheduled") => {
            p.next += 1;
            let n = p.next;
            let id = format!("2026111312000{n}abcdef");
            let host = if p.foreign_link {
                format!("https://evil.example/meet/HOST{n}")
            } else {
                format!("{base}/meet/HOSTLINK{n}")
            };
            let mut meeting = json_body.clone();
            meeting["openlink"] = json!(format!("{base}/meet/OPENLINK{n}"));
            meeting["organizer"] =
                json!({"join": host.clone(), "extension": json_body["extension"]});
            p.meetings.insert(id.clone(), meeting);
            reply(
                200,
                json!({"status": "success", "result": {"meetingid": id, "participants": [],
                       "organizer": {"name": "Ilia A", "email": "3cx@ilia.ae", "join": host, "extension": "100"}},
                       "errorInformation": ""}),
            )
        }
        ("GET", "/meetings/list") => {
            if p.meetings.is_empty() {
                error(404, "No Meetings")
            } else {
                reply(
                    200,
                    json!({"status": "success", "result": {"scheduledMeetings": []}}),
                )
            }
        }
        ("GET", p_) if p_.starts_with("/scheduled/") => match p.meetings.get(&id_of(p_)) {
            Some(m) => reply(
                200,
                json!({"status": "success", "result": {"meetingid": id_of(p_), "openlink": m["openlink"],
                       "organizer": m["organizer"], "participants": []}, "errorInformation": ""}),
            ),
            None => error(404, "Meeting Not Found"),
        },
        ("PATCH", p_) if p_.starts_with("/scheduled/") => {
            let id = id_of(p_);
            match p.meetings.get_mut(&id) {
                Some(m) => {
                    m["dateTime"] = json_body["dateTime"].clone();
                    m["duration"] = json_body["duration"].clone();
                    reply(200, json!({"status": "success", "errorInformation": ""}))
                }
                None => error(404, "Meeting Not Found"),
            }
        }
        ("DELETE", p_) if p_.starts_with("/scheduled/") => match p.meetings.remove(&id_of(p_)) {
            Some(_) => reply(200, json!({"status": "success", "errorInformation": ""})),
            None => error(404, "Meeting Not Found"),
        },
        _ => error(400, "Invalid Parameter"),
    }
}

fn cfg(base: &str) -> ThreeCxConfig {
    ThreeCxConfig {
        base_url: base.to_string(),
        api_key: KEY.to_string(),
        extension: "100".to_string(),
        display_name: Some("3CX".to_string()),
    }
}

fn calls(pbx: &SharedPbx) -> Vec<(String, String)> {
    pbx.lock()
        .unwrap()
        .calls
        .iter()
        .map(|(m, p, _, _)| (m.clone(), p.clone()))
        .collect()
}

async fn store_settings(pool: &SqlitePool, base: &str) {
    threecx::store(
        pool,
        &KEY32,
        threecx::Update {
            base_url: Some(base.to_string()),
            extension: Some("100".to_string()),
            display_name: Some("3CX".to_string()),
            api_key: Some(KEY.to_string()),
            clear: false,
        },
    )
    .await
    .unwrap();
}

// --- Checks ---

#[test]
fn settings_are_checked_and_canonical() {
    assert_eq!(
        threecx::check_base_url("https://SIP.ilia.ae/").unwrap(),
        "https://sip.ilia.ae"
    );
    assert_eq!(
        threecx::check_base_url("https://sip.ilia.ae:5001").unwrap(),
        "https://sip.ilia.ae:5001"
    );
    assert_eq!(
        threecx::check_base_url("http://127.0.0.1:8080").unwrap(),
        "http://127.0.0.1:8080"
    );
    for bad in [
        "http://sip.ilia.ae",
        "https://sip.ilia.ae/webmeeting",
        "https://user:pw@sip.ilia.ae",
        "https://sip.ilia.ae/?a=1",
        "javascript:alert(1)",
        "sip.ilia.ae",
        "",
    ] {
        assert!(threecx::check_base_url(bad).is_err(), "{bad}");
    }
    assert_eq!(threecx::check_extension(" 100 ").unwrap(), "100");
    for bad in ["", "10a", "12345678901", "-1"] {
        assert!(threecx::check_extension(bad).is_err(), "{bad}");
    }
    assert!(threecx::check_api_key(KEY).is_ok());
    for bad in ["short", "with space inside the key 0123", &"x".repeat(257)] {
        assert!(threecx::check_api_key(bad).is_err(), "{bad}");
    }
    assert_eq!(threecx::check_display_name("  ").unwrap(), None);
    assert!(threecx::check_display_name(&"x".repeat(101)).is_err());
    assert!(threecx::check_display_name("a\u{7}b").is_err());
}

#[test]
fn the_deployment_key_is_read_as_a_key() {
    use threecx::{parse_env_key, EnvKey};
    assert_eq!(parse_env_key(None), EnvKey::Unset);
    assert_eq!(parse_env_key(Some("")), EnvKey::Unset);
    assert_eq!(parse_env_key(Some("  \n")), EnvKey::Unset, "empty is unset");
    assert_eq!(
        parse_env_key(Some(&format!("{ENV_KEY}\n"))),
        EnvKey::Set(ENV_KEY.to_string())
    );
    assert_eq!(parse_env_key(Some("short")), EnvKey::Malformed);
    assert_eq!(
        parse_env_key(Some("has a space in it 0123")),
        EnvKey::Malformed
    );
    assert!(parse_env_key(Some("short")).is_set());
    let shown = format!("{:?}", parse_env_key(Some(ENV_KEY)));
    assert!(!shown.contains(ENV_KEY), "{shown}");
    assert!(
        !threecx::env_key().is_set(),
        "tests never read the process environment"
    );
}

#[test]
fn the_key_is_never_printed() {
    let shown = format!("{:?}", cfg("https://sip.ilia.ae"));
    assert!(!shown.contains(KEY), "{shown}");
    assert!(shown.contains("<redacted>"));
}

/// The answers the owner's PBX gave on 2026-11-13, and the shapes around them.
#[test]
fn answers_are_read_as_the_pbx_writes_them() {
    let ok = br#"{"status":"success","result":{"meetingid":"1"},"errorInformation":""}"#;
    assert_eq!(
        threecx::parse_answer(200, ok).unwrap(),
        json!({"meetingid": "1"})
    );
    let none = br#"{"status":"error","errorInformation":"No Meetings"}"#;
    assert_eq!(threecx::parse_answer(404, none), Err(ApiError::NotFound));
    let gone = br#"{"status":"error","errorInformation":"Meeting Not Found"}"#;
    assert_eq!(threecx::parse_answer(404, gone), Err(ApiError::NotFound));
    let auth = br#"{"status":"error","errorInformation":"API Authentication Failed"}"#;
    assert_eq!(threecx::parse_answer(200, auth), Err(ApiError::Auth));
    assert_eq!(
        threecx::parse_answer(403, b"Forbidden"),
        Err(ApiError::Auth)
    );
    let param = br#"{"status":"error","errorInformation":"Invalid Parameter: participants[0]"}"#;
    assert_eq!(
        threecx::parse_answer(400, param),
        Err(ApiError::Service(
            "Invalid Parameter: participants[0]".to_string()
        ))
    );
    assert_eq!(
        threecx::parse_answer(502, b"<html>bad gateway</html>"),
        Err(ApiError::Transport)
    );
    // "success" with an error status is not a success.
    assert!(threecx::parse_answer(500, ok).is_err());
}

#[test]
fn only_links_on_the_pbx_are_handed_out() {
    let c = cfg("https://sip.ilia.ae");
    assert!(threecx::check_link(&c, "https://sip.ilia.ae/meet/XngQp06UzA2vTf").is_ok());
    for bad in [
        "https://evil.example/meet/abc",
        "http://sip.ilia.ae/meet/abc",
        "https://sip.ilia.ae:8443/meet/abc",
        "https://sip.ilia.ae/other/abc",
        "https://sip.ilia.ae/meet/",
        "javascript:alert(1)",
        "https://sip.ilia.ae/meet/a b",
        "https://user@sip.ilia.ae/meet/abc",
    ] {
        assert!(threecx::check_link(&c, bad).is_err(), "{bad}");
    }
}

// --- The API against the mock ---

#[tokio::test]
async fn a_meeting_is_made_without_participants_and_read_back() {
    let (base, pbx) = spawn_pbx().await;
    let c = cfg(&base);
    let start = chrono::DateTime::parse_from_rfc3339("2026-12-31T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let made = threecx::create(&c, start, 15, "CTO Consultation — Guest")
        .await
        .unwrap();
    assert!(made.open_link.starts_with(&format!("{base}/meet/OPENLINK")));
    assert!(made.host_link.starts_with(&format!("{base}/meet/HOSTLINK")));
    let p = pbx.lock().unwrap();
    let (method, path, key, body) = &p.calls[0];
    assert_eq!(
        (method.as_str(), path.as_str()),
        ("POST", "/webmeeting/api/v1/scheduled")
    );
    assert_eq!(key.as_deref(), Some(KEY));
    // No guest email reaches 3CX, so 3CX never writes to the guest.
    assert_eq!(body["participants"], json!([]));
    assert_eq!(body["extension"], json!("100"));
    assert_eq!(body["dateTime"], json!("2026-12-31T10:00:00Z"));
    assert_eq!(body["duration"], json!(15));
    assert_eq!(body["hostJoinFirst"], json!(false));
    assert_eq!(p.calls[1].0, "GET");
}

#[tokio::test]
async fn a_meeting_with_a_foreign_link_is_deleted_again() {
    let (base, pbx) = spawn_pbx().await;
    pbx.lock().unwrap().foreign_link = true;
    let start = Utc::now() + chrono::Duration::days(3);
    let made = threecx::create(&cfg(&base), start, 15, "x").await;
    assert_eq!(made, Err(ApiError::BadLink));
    assert!(pbx.lock().unwrap().meetings.is_empty());
    assert_eq!(calls(&pbx).last().unwrap().0, "DELETE");
}

#[tokio::test]
async fn a_refused_key_pauses_the_calls_until_a_test() {
    let (base, pbx) = spawn_pbx().await;
    pbx.lock().unwrap().refuse_key = true;
    let c = cfg(&base);
    let start = Utc::now() + chrono::Duration::days(3);
    assert_eq!(
        threecx::create(&c, start, 15, "x").await,
        Err(ApiError::Auth)
    );
    assert!(threecx::auth_blocked(&base));
    // Paused: nothing more reaches the PBX.
    assert_eq!(
        threecx::create(&c, start, 15, "x").await,
        Err(ApiError::Auth)
    );
    assert_eq!(threecx::delete(&c, "1").await, Err(ApiError::Auth));
    assert_eq!(calls(&pbx).len(), 1);
    // The admin's test always calls, and a working key ends the pause.
    pbx.lock().unwrap().refuse_key = false;
    threecx::check(&c).await.unwrap();
    assert!(!threecx::auth_blocked(&base));
}

#[tokio::test]
async fn a_pbx_that_is_down_is_a_transport_error() {
    let (base, pbx) = spawn_pbx().await;
    pbx.lock().unwrap().down = true;
    assert_eq!(threecx::check(&cfg(&base)).await, Err(ApiError::Transport));
    assert!(!threecx::auth_blocked(&base));
}

// --- Bookings ---

async fn threecx_app() -> (Router, SqlitePool, String, String, String, SharedPbx) {
    let (app, pool, session, et_id) = setup_test_app().await;
    let (base, pbx) = spawn_pbx().await;
    sqlx::query(
        "UPDATE event_types SET location_type = 'threecx_auto', location_value = NULL WHERE id = ?",
    )
    .bind(&et_id)
    .execute(&pool)
    .await
    .unwrap();
    // Through the admin card, so the router's cached settings follow.
    let csrf = "csrf-3cx";
    let body = format!(
        "_csrf={csrf}&action=save&threecx_base_url={}&threecx_extension=100&threecx_display_name=3CX&threecx_api_key={KEY}",
        urlencoding::encode(&base)
    );
    let response = app
        .clone()
        .oneshot(post_form("/dashboard/admin/threecx", &session, csrf, &body))
        .await
        .unwrap();
    assert_eq!(response.status(), 303);
    assert_eq!(
        response.headers().get("location").unwrap(),
        "/dashboard/admin?threecx=saved#threecx"
    );
    (app, pool, session, et_id, base, pbx)
}

#[tokio::test]
async fn a_booking_gets_its_meeting_the_guest_the_open_link_the_host_the_organizer_link() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, _session, _et_id, base, pbx) = threecx_app().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, port).await;

    let page = body_string(
        app.clone()
            .oneshot(get("/u/testuser/test-meeting"))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains(">3CX<"), "the label is shown");

    let csrf = "csrf-3cx-book";
    let response = app
        .clone()
        .oneshot(post_form_unauthed(
            "/u/testuser/test-meeting/book",
            csrf,
            &book_body(csrf),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let confirmed = body_string(response).await;

    let (booking_id, meeting_url): (String, Option<String>) =
        sqlx::query_as("SELECT id, meeting_url FROM bookings WHERE guest_email = 'guest@test.com'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let open = format!("{base}/meet/OPENLINK1");
    let host = format!("{base}/meet/HOSTLINK1");
    assert_eq!(meeting_url.as_deref(), Some(open.as_str()));
    // Escaped by the template (`/` is `&#x2f;`): look for the link's token.
    assert!(
        confirmed.contains("OPENLINK1"),
        "the guest's page links the meeting"
    );
    assert!(!confirmed.contains("HOSTLINK"));

    let (meeting_id, host_enc, start_at, minutes): (String, String, String, i64) = sqlx::query_as(
        "SELECT meeting_id, host_link_enc, start_at, duration_min FROM ilia_threecx_meetings WHERE booking_id = ?",
    )
    .bind(&booking_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(pbx.lock().unwrap().meetings.contains_key(&meeting_id));
    assert!(
        !host_enc.contains("HOSTLINK"),
        "the organizer link is stored encrypted"
    );
    assert_eq!(start_at, format!("{}T10:00:00Z", far_future_monday()));
    assert_eq!(minutes, 30);
    let body = pbx.lock().unwrap().calls[0].3.clone();
    assert_eq!(body["participants"], json!([]));
    assert!(body["subject"].as_str().unwrap().contains("Test Meeting"));

    let guest = mail_to(&mails, "guest@test.com").await;
    assert!(guest.contains("OPENLINK1"), "{guest}");
    assert!(
        !guest.contains("HOSTLINK"),
        "the guest never sees the organizer link"
    );
    let host_mail = mail_to(&mails, "test@example.com").await;
    assert!(host_mail.contains("HOSTLINK1"), "{host_mail}");
    assert!(!host_mail.contains("OPENLINK"), "{host_mail}");
    assert!(host.ends_with("HOSTLINK1"));
}

#[tokio::test]
async fn a_failed_meeting_does_not_stop_the_booking() {
    let (app, pool, _session, _et_id, _base, pbx) = threecx_app().await;
    pbx.lock().unwrap().down = true;
    let csrf = "csrf-3cx-down";
    let response = app
        .oneshot(post_form_unauthed(
            "/u/testuser/test-meeting/book",
            csrf,
            &book_body(csrf),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let (status, url): (String, Option<String>) = sqlx::query_as(
        "SELECT status, meeting_url FROM bookings WHERE guest_email = 'guest@test.com'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "confirmed");
    assert_eq!(url, None);
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ilia_threecx_meetings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
}

/// A booking with a meeting on the mock, made the way a booking makes it:
/// one a day from 2030-03-04 at 10:00, 30 minutes.
async fn booking_with_meeting(pool: &SqlitePool, et_id: &str, c: &ThreeCxConfig) -> String {
    let day: i64 = sqlx::query_scalar("SELECT COUNT(*) + 4 FROM bookings")
        .fetch_one(pool)
        .await
        .unwrap();
    let (booking_id, _, _) = insert_test_booking(
        pool,
        et_id,
        &format!("2030-03-{day:02}T10:00:00"),
        &format!("2030-03-{day:02}T10:30:00"),
        "confirmed",
    )
    .await;
    let link = threecx::create_for_booking(pool, &KEY32, c, &booking_id).await;
    assert!(link.is_some());
    booking_id
}

#[tokio::test]
async fn the_sweep_moves_and_deletes_meetings_with_their_bookings() {
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let (base, pbx) = spawn_pbx().await;
    let c = cfg(&base);
    let moved = booking_with_meeting(&pool, &et_id, &c).await;
    let cancelled = booking_with_meeting(&pool, &et_id, &c).await;
    let deleted = booking_with_meeting(&pool, &et_id, &c).await;
    let unchanged = booking_with_meeting(&pool, &et_id, &c).await;
    assert_eq!(pbx.lock().unwrap().meetings.len(), 4);

    sqlx::query("UPDATE bookings SET start_at = '2030-03-25T11:00:00', end_at = '2030-03-25T11:45:00' WHERE id = ?")
        .bind(&moved)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
        .bind(&cancelled)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM bookings WHERE id = ?")
        .bind(&deleted)
        .execute(&pool)
        .await
        .unwrap();
    let before = calls(&pbx).len();
    let swept = threecx::sweep(&pool, Some(&c), Utc::now()).await;
    assert_eq!(
        swept,
        threecx::Swept {
            moved: 1,
            deleted: 2,
            forgotten: 0,
            failed: 0
        }
    );
    let made: Vec<_> = calls(&pbx)[before..].to_vec();
    assert_eq!(made.iter().filter(|(m, _)| m == "PATCH").count(), 1);
    assert_eq!(made.iter().filter(|(m, _)| m == "DELETE").count(), 2);
    let (left_on_pbx, patch) = {
        let p = pbx.lock().unwrap();
        let patch = p.calls.iter().find(|c| c.0 == "PATCH").unwrap().3.clone();
        (p.meetings.len(), patch)
    };
    assert_eq!(left_on_pbx, 2);
    assert_eq!(patch["dateTime"], json!("2030-03-25T11:00:00Z"));
    assert_eq!(patch["duration"], json!(45));
    let (start, minutes): (String, i64) = sqlx::query_as(
        "SELECT start_at, duration_min FROM ilia_threecx_meetings WHERE booking_id = ?",
    )
    .bind(&moved)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((start.as_str(), minutes), ("2030-03-25T11:00:00Z", 45));
    let left: Vec<String> =
        sqlx::query_scalar("SELECT booking_id FROM ilia_threecx_meetings ORDER BY booking_id")
            .fetch_all(&pool)
            .await
            .unwrap();
    let mut want = vec![moved.clone(), unchanged.clone()];
    want.sort();
    assert_eq!(left, want);

    // A second pass has nothing to do and calls nothing.
    let before = calls(&pbx).len();
    assert_eq!(
        threecx::sweep(&pool, Some(&c), Utc::now()).await,
        threecx::Swept::default()
    );
    assert_eq!(calls(&pbx).len(), before);
}

#[tokio::test]
async fn a_failed_sweep_backs_off_and_a_gone_meeting_counts_as_deleted() {
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let (base, pbx) = spawn_pbx().await;
    let c = cfg(&base);
    let booking = booking_with_meeting(&pool, &et_id, &c).await;
    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
        .bind(&booking)
        .execute(&pool)
        .await
        .unwrap();
    pbx.lock().unwrap().down = true;
    let now = Utc::now();
    assert_eq!(threecx::sweep(&pool, Some(&c), now).await.failed, 1);
    let (attempts, next, error): (i64, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT attempts, next_attempt_at, last_error FROM ilia_threecx_meetings WHERE booking_id = ?",
    )
    .bind(&booking)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(attempts, 1);
    assert!(next.is_some());
    assert_eq!(error.as_deref(), Some("transport"));
    // Not due yet: no call.
    let before = calls(&pbx).len();
    assert_eq!(
        threecx::sweep(&pool, Some(&c), now).await,
        threecx::Swept::default()
    );
    assert_eq!(calls(&pbx).len(), before);
    // Later, the PBX up and the meeting already gone there: forgotten.
    {
        let mut p = pbx.lock().unwrap();
        p.down = false;
        p.meetings.clear();
    }
    let later = now + chrono::Duration::minutes(5);
    assert_eq!(threecx::sweep(&pool, Some(&c), later).await.deleted, 1);
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ilia_threecx_meetings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
}

#[tokio::test]
async fn the_sweep_does_nothing_without_settings() {
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let (base, pbx) = spawn_pbx().await;
    let booking = booking_with_meeting(&pool, &et_id, &cfg(&base)).await;
    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
        .bind(&booking)
        .execute(&pool)
        .await
        .unwrap();
    let before = calls(&pbx).len();
    assert_eq!(
        threecx::sweep(&pool, None, Utc::now()).await,
        threecx::Swept::default()
    );
    assert_eq!(calls(&pbx).len(), before);
}

#[tokio::test]
async fn host_details_carry_the_organizer_link() {
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let (base, _pbx) = spawn_pbx().await;
    let booking = booking_with_meeting(&pool, &et_id, &cfg(&base)).await;
    let uid: String = sqlx::query_scalar("SELECT uid FROM bookings WHERE id = ?")
        .bind(&booking)
        .fetch_one(&pool)
        .await
        .unwrap();
    let details = crate::email::BookingDetails {
        uid: uid.clone(),
        location: Some(format!("{base}/meet/OPENLINK1")),
        ..Default::default()
    };
    let host = threecx::for_host(&pool, &KEY32, &details).await;
    assert_eq!(host.location, Some(format!("{base}/meet/HOSTLINK1")));
    let other = crate::email::BookingDetails {
        uid: "someone-else@calrs".to_string(),
        location: Some("https://meet.example/x".to_string()),
        ..Default::default()
    };
    assert_eq!(
        threecx::for_host(&pool, &KEY32, &other).await.location,
        other.location
    );
}

// --- The admin card ---

#[tokio::test]
async fn the_admin_card_saves_tests_and_removes_without_showing_the_key() {
    let (app, pool, session, _et_id, base, pbx) = threecx_app().await;
    let page = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/admin", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains("id=\"threecx\""));
    assert!(page.contains("127.0.0.1"));
    assert!(!page.contains(KEY), "the key is never shown");
    let stored: String =
        sqlx::query_scalar("SELECT threecx_api_key_enc FROM auth_config WHERE id = 'singleton'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!stored.contains(KEY), "the key is stored encrypted");

    let post = |body: String| {
        let app = app.clone();
        let session = session.clone();
        async move {
            let r = app
                .oneshot(post_form(
                    "/dashboard/admin/threecx",
                    &session,
                    "c",
                    &format!("_csrf=c&{body}"),
                ))
                .await
                .unwrap();
            r.headers()
                .get("location")
                .map(|v| v.to_str().unwrap().to_string())
                .unwrap_or_default()
        }
    };
    // Saving with an empty key keeps the stored one.
    let enc = |base: &str| format!("threecx_base_url={}", urlencoding::encode(base));
    assert!(post(format!(
        "action=save&{}&threecx_extension=100&threecx_api_key=",
        enc(&base)
    ))
    .await
    .ends_with("threecx=saved#threecx"));
    assert!(threecx::load(&pool, &KEY32).await.is_some());
    assert!(post("action=test".to_string())
        .await
        .ends_with("threecx=ok#threecx"));
    assert!(calls(&pbx)
        .iter()
        .any(|(m, p)| m == "GET" && p.ends_with("/meetings/list")));
    // A refused key says so; refused values are not stored.
    pbx.lock().unwrap().refuse_key = true;
    assert!(post("action=test".to_string())
        .await
        .ends_with("threecx=auth#threecx"));
    pbx.lock().unwrap().refuse_key = false;
    assert!(
        post(format!("action=save&{}&threecx_extension=abc", enc(&base)))
            .await
            .ends_with("threecx=bad-extension#threecx")
    );
    assert!(post(
        "action=save&threecx_base_url=http%3A%2F%2Fsip.example&threecx_extension=100".to_string()
    )
    .await
    .ends_with("threecx=bad-base-url#threecx"));
    assert!(
        post(format!("action=save&{}&threecx_extension=", enc(&base)))
            .await
            .ends_with("threecx=bad-pair#threecx")
    );
    assert_eq!(threecx::shown(&pool).await.extension, "100");
    // Removed, the key with it.
    assert!(post("action=clear".to_string())
        .await
        .ends_with("threecx=cleared#threecx"));
    assert_eq!(threecx::shown(&pool).await, threecx::Shown::default());
    assert!(threecx::load(&pool, &KEY32).await.is_none());
    assert!(post("action=test".to_string())
        .await
        .ends_with("threecx=not-set#threecx"));
    // A notice that is not one of the card's words is not shown.
    let page = body_string(
        app.clone()
            .oneshot(get_authed(
                "/dashboard/admin?threecx=%3Cscript%3E",
                &session,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert!(!page.contains("admin-threecx-notice"));
    assert!(!page.contains("<script>alert"));
}

async fn stored_enc(pool: &SqlitePool) -> Option<String> {
    sqlx::query_scalar("SELECT threecx_api_key_enc FROM auth_config WHERE id = 'singleton'")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn the_deployment_key_wins_and_locks_the_card() {
    let (app, pool, session, _et_id, base, pbx) = threecx_app().await;
    let _env = threecx::test_env_key(Some(ENV_KEY));
    // The deployment's key is the key, the stored one is never used.
    assert_eq!(threecx::load(&pool, &KEY32).await.unwrap().api_key, ENV_KEY);
    let shown = threecx::shown(&pool).await;
    assert!(shown.key_set && shown.key_from_env && !shown.key_env_bad);

    let page = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/admin", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains("Set by the deployment (CALRS_THREECX_API_KEY)"));
    assert!(
        !page.contains("name=\"threecx_api_key\""),
        "the key field posts nothing"
    );
    assert!(page.contains(
        "id=\"threecx_api_key\" value=\"\" placeholder=\"Set by the deployment\" disabled"
    ));
    assert!(!page.contains(KEY) && !page.contains(ENV_KEY));

    let post = |body: String| {
        let app = app.clone();
        let session = session.clone();
        async move {
            let r = app
                .oneshot(post_form(
                    "/dashboard/admin/threecx",
                    &session,
                    "c",
                    &format!("_csrf=c&{body}"),
                ))
                .await
                .unwrap();
            r.headers()
                .get("location")
                .map(|v| v.to_str().unwrap().to_string())
                .unwrap_or_default()
        }
    };
    let enc = format!("threecx_base_url={}", urlencoding::encode(&base));
    pbx.lock().unwrap().accept = Some(ENV_KEY.to_string());
    // The test call carries the deployment's key, even with one typed.
    assert!(post(format!(
        "action=test&{enc}&threecx_extension=100&threecx_api_key={KEY}"
    ))
    .await
    .ends_with("threecx=ok#threecx"));
    let sent = pbx.lock().unwrap().calls.last().unwrap().2.clone();
    assert_eq!(sent.as_deref(), Some(ENV_KEY));
    // A typed key is refused, nothing written.
    let before = stored_enc(&pool).await;
    assert!(before.is_some());
    assert!(post(format!(
        "action=save&{enc}&threecx_extension=100&threecx_api_key={KEY}"
    ))
    .await
    .ends_with("threecx=key-env#threecx"));
    assert_eq!(stored_enc(&pool).await, before);
    let page = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/admin?threecx=key-env", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains("Not saved: the key is set by the deployment"));
    // A save without a key removes the stored one, unused from now on.
    assert!(post(format!("action=save&{enc}&threecx_extension=100"))
        .await
        .ends_with("threecx=saved#threecx"));
    assert_eq!(stored_enc(&pool).await, None);
    assert_eq!(threecx::load(&pool, &KEY32).await.unwrap().api_key, ENV_KEY);

    // A malformed variable keeps 3CX off and says so, without a fallback.
    drop(_env);
    let _bad = threecx::test_env_key(Some("short"));
    assert!(threecx::load(&pool, &KEY32).await.is_none());
    let shown = threecx::shown(&pool).await;
    assert!(!shown.key_set && shown.key_from_env && shown.key_env_bad);
    let page = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/admin", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains("CALRS_THREECX_API_KEY is set but is not a key"));
    assert!(post("action=test".to_string())
        .await
        .ends_with("threecx=not-set#threecx"));
    drop(_bad);
    // Without the variable there is no key left: the stored one went.
    assert!(threecx::load(&pool, &KEY32).await.is_none());
    assert!(!threecx::shown(&pool).await.key_set);
}

#[tokio::test]
async fn the_admin_card_needs_an_admin_and_csrf() {
    let (app, pool, _session, _et_id) = setup_test_app().await;
    let member = insert_role_user(&pool, "member@example.com", "user").await;
    let member_session = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, datetime('now', '+1 day'))",
    )
    .bind(&member_session)
    .bind(&member)
    .execute(&pool)
    .await
    .unwrap();
    let body = format!("_csrf=c&action=save&threecx_base_url=https%3A%2F%2Fsip.ilia.ae&threecx_extension=100&threecx_api_key={KEY}");
    let r = app
        .clone()
        .oneshot(post_form(
            "/dashboard/admin/threecx",
            &member_session,
            "c",
            &body,
        ))
        .await
        .unwrap();
    assert_eq!(r.status(), 403);
    // The admin's session without the CSRF cookie: refused, nothing stored.
    let admin_session: String = sqlx::query_scalar(
        "SELECT s.id FROM sessions s JOIN users u ON u.id = s.user_id WHERE u.email = 'test@example.com'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let r = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/dashboard/admin/threecx")
                .header("cookie", format!("__Host-calrs_session={admin_session}"))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let location = r
        .headers()
        .get("location")
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    assert!(!location.contains("threecx=saved"), "{location}");
    assert_eq!(threecx::shown(&pool).await, threecx::Shown::default());
}

// --- config apply: site.threecx ---

mod apply_section {
    use super::super::apply_support::*;
    use super::*;
    use crate::web::apply::{self, ApplyError};

    fn doc(threecx: &str) -> String {
        format!(
            r#"{{"format":"{}","user":"{ADMIN_EMAIL}","site":{{"threecx":{threecx}}}}}"#,
            apply::FORMAT
        )
    }

    async fn stored_key(env: &ApplyEnv) -> Option<String> {
        let enc: Option<String> = sqlx::query_scalar(
            "SELECT threecx_api_key_enc FROM auth_config WHERE id = 'singleton'",
        )
        .fetch_one(&env.pool)
        .await
        .unwrap();
        enc.map(|e| crate::crypto::decrypt_value(&KEY32, &e).unwrap())
    }

    #[tokio::test]
    async fn the_settings_are_written_the_key_kept_or_set_and_never_exported() {
        let env = ApplyEnv::new().await;
        let on = format!(
            r#"{{"base_url":"https://sip.ilia.ae","extension":"100","display_name":"3CX","api_key":"{KEY}"}}"#
        );
        let report = env.apply(&doc(&on), false).await.unwrap();
        assert_eq!(report.changed.len(), 1);
        assert_eq!(report.changed[0].path, "site.threecx");
        assert_eq!(report.changed[0].before, json!("off"));
        assert_eq!(
            report.changed[0].after,
            json!({"base_url": "https://sip.ilia.ae", "extension": "100", "display_name": "3CX",
                   "api_key": apply::SECRET_SET})
        );
        assert!(report.restart, "calrs serve caches the settings");
        assert_eq!(stored_key(&env).await.as_deref(), Some(KEY));
        let raw = serde_json::to_string(&report.changed).unwrap();
        assert!(!raw.contains(KEY), "the report never carries the key");

        // A second run changes nothing.
        assert!(env
            .apply(&doc(&on), false)
            .await
            .unwrap()
            .changed
            .is_empty());

        // A null key keeps the stored one (the admin page may own it).
        let page_owned = r#"{"base_url":"https://sip.ilia.ae","extension":"100","display_name":null,"api_key":null}"#;
        let report = env.apply(&doc(page_owned), false).await.unwrap();
        assert_eq!(report.changed.len(), 1);
        assert_eq!(stored_key(&env).await.as_deref(), Some(KEY));
        assert!(env
            .apply(&doc(page_owned), false)
            .await
            .unwrap()
            .changed
            .is_empty());

        // The export names no key, and applying it changes nothing.
        let exported = env.export().await.unwrap();
        let text = serde_json::to_string(&exported).unwrap();
        assert!(!text.contains(KEY), "{text}");
        assert!(text.contains(r#""threecx":{"base_url":"https://sip.ilia.ae","extension":"100","display_name":null}"#), "{text}");

        // Off removes everything, the key with it.
        let report = env.apply(&doc(r#""off""#), false).await.unwrap();
        assert_eq!(report.changed[0].after, json!("off"));
        assert_eq!(stored_key(&env).await, None);
        assert_eq!(threecx::shown(&env.pool).await, threecx::Shown::default());
    }

    #[tokio::test]
    async fn the_deployment_key_is_not_the_documents() {
        let env = ApplyEnv::new().await;
        let _env = threecx::test_env_key(Some(ENV_KEY));
        let on = r#"{"base_url":"https://sip.ilia.ae","extension":"100","display_name":"3CX"}"#;
        let report = env.apply(&doc(on), false).await.unwrap();
        assert_eq!(report.changed.len(), 1);
        assert_eq!(
            report.changed[0].after,
            json!({"base_url": "https://sip.ilia.ae", "extension": "100", "display_name": "3CX",
                   "api_key": apply::SECRET_SET})
        );
        assert_eq!(
            stored_key(&env).await,
            None,
            "nothing stored: the key is the deployment's"
        );
        assert!(env.apply(&doc(on), false).await.unwrap().changed.is_empty());

        // A key in the document is refused, in a dry run too, nothing written.
        let before = env.snapshot().await;
        let with_key = format!(
            r#"{{"base_url":"https://sip.ilia.ae","extension":"100","display_name":"3CX","api_key":"{KEY}"}}"#
        );
        for dry_run in [true, false] {
            match env.apply(&doc(&with_key), dry_run).await {
                Err(ApplyError::Invalid { path, message }) => {
                    assert_eq!(path, "site.threecx.api_key");
                    assert!(message.contains("CALRS_THREECX_API_KEY"), "{message}");
                    assert!(!message.contains(KEY));
                }
                other => panic!("{other:?}"),
            }
        }
        assert_eq!(env.snapshot().await, before);

        // The export names no key; off and off again change nothing twice.
        let text = serde_json::to_string(&env.export().await.unwrap()).unwrap();
        assert!(!text.contains(ENV_KEY));
        assert!(text.contains(r#""threecx":{"base_url":"https://sip.ilia.ae","extension":"100","display_name":"3CX"}"#), "{text}");
        assert_eq!(
            env.apply(&doc(r#""off""#), false)
                .await
                .unwrap()
                .changed
                .len(),
            1
        );
        assert!(env
            .apply(&doc(r#""off""#), false)
            .await
            .unwrap()
            .changed
            .is_empty());
    }

    #[tokio::test]
    async fn values_must_be_written_as_stored() {
        let env = ApplyEnv::new().await;
        let before = env.snapshot().await;
        for (value, path) in [
            (
                r#"{"base_url":"http://sip.ilia.ae","extension":"100","display_name":null}"#,
                "site.threecx.base_url",
            ),
            (
                r#"{"base_url":"https://SIP.ilia.ae/","extension":"100","display_name":null}"#,
                "site.threecx.base_url",
            ),
            (
                r#"{"base_url":"https://sip.ilia.ae","extension":"1 00","display_name":null}"#,
                "site.threecx.extension",
            ),
            (
                r#"{"base_url":"https://sip.ilia.ae","extension":"100","display_name":" 3CX"}"#,
                "site.threecx.display_name",
            ),
            (
                r#"{"base_url":"https://sip.ilia.ae","extension":"100","display_name":null,"api_key":"short"}"#,
                "site.threecx.api_key",
            ),
        ] {
            match env.apply(&doc(value), false).await {
                Err(ApplyError::Invalid { path: p, .. }) => assert_eq!(p, path, "{value}"),
                other => panic!("{value}: {other:?}"),
            }
        }
        assert_eq!(env.snapshot().await, before, "nothing written");
    }
}
