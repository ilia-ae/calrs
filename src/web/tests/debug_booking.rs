//! ilia fork: the admin's debug booking page (`src/web/debug_booking.rs`)
//! and its switch at the top of the TON page (env `CALRS_DEBUG_BOOKING` >
//! `ton_settings` > off). A child of the TON tests, for their chain, rates
//! and `Env`: the page books through the public path, so these tests see a
//! real invoice, a real hold and real refusals.

use super::*;
use crate::telegram::fake;
use crate::web::debug_booking::{CANCEL_PATH, ENV, PATH, TOGGLE_PATH};
use crate::web::ton_pay::settings_page::PATH as TON_PAGE;

const HOST_CHAT: i64 = 555000111;
const BASE: &str = "https://book.example.test";

/// The page switched on for `e`, as the TON page's toggle stores it (read
/// per request: no new state, no restart).
async fn on(e: &Env) {
    sqlx::query(
        "INSERT INTO ton_settings (key, value, owner, updated_at) VALUES ('CALRS_DEBUG_BOOKING', '1', 'panel', '2026-11-12T00:00:00Z')
         ON CONFLICT (key) DO UPDATE SET value = '1', owner = 'panel'",
    )
    .execute(&e.pool)
    .await
    .unwrap();
}

async fn env_on() -> Env {
    let e = env().await;
    on(&e).await;
    e
}

/// The test type priced by the ladder `ladder`.
async fn ladder_env_on(ladder: &str) -> Env {
    let e = env_priced(&[("CALRS_TON_LADDER", ladder)], Some(100 * NANO)).await;
    sqlx::query(
        "UPDATE ton_prices SET mode = 'ladder', currency = 'TON', price_nano = NULL, price_usd_cents = NULL",
    )
    .execute(&e.pool)
    .await
    .unwrap();
    on(&e).await;
    e
}

const NANO: u128 = 1_000_000_000;

fn slot() -> String {
    format!("{}T10:00", far_future_monday())
}

fn form(extra: &str) -> String {
    format!(
        "_csrf={CSRF}&et=test-meeting&slot={}&name=Test+User&email=test%40example.com&notes=debug{extra}",
        urlencoding::encode(&slot())
    )
}

impl Env {
    async fn get_as(&self, uri: &str, session: &str) -> (axum::http::StatusCode, String) {
        let resp = self.app().oneshot(get_authed(uri, session)).await.unwrap();
        let status = resp.status();
        (status, unescape(&body_string(resp).await))
    }

    async fn post_as(&self, uri: &str, session: &str, body: &str) -> axum::http::Response<Body> {
        self.app()
            .oneshot(post_form(uri, session, CSRF, body))
            .await
            .unwrap()
    }

    async fn debug_rows(&self) -> Vec<(String, String, i64)> {
        sqlx::query_as("SELECT booking_id, user_id, telegram_own_chat FROM debug_bookings")
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }

    async fn admin_id(&self) -> String {
        sqlx::query_scalar("SELECT id FROM users WHERE username = 'testuser'")
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}

/// A session for a user with `role`, with an account and a free personal
/// type `slug` open Mon-Fri 9-17: (user id, session).
async fn other_user(pool: &SqlitePool, email: &str, role: &str, slug: &str) -> (String, String) {
    let user_id = insert_role_user(pool, email, role).await;
    sqlx::query(
        "INSERT INTO accounts (id, name, email, timezone, user_id) VALUES (?, ?, ?, 'UTC', ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(email)
    .bind(email)
    .bind(&user_id)
    .execute(pool)
    .await
    .unwrap();
    let et_id = insert_personal_et(pool, &user_id, slug).await;
    for day in 1..=5 {
        sqlx::query("INSERT INTO availability_rules (id, event_type_id, day_of_week, start_time, end_time) VALUES (?, ?, ?, '09:00', '17:00')")
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&et_id)
            .bind(day)
            .execute(pool)
            .await
            .unwrap();
    }
    let token = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, datetime('now', '+1 day'))",
    )
    .bind(&token)
    .bind(&user_id)
    .execute(pool)
    .await
    .unwrap();
    (user_id, token)
}

#[tokio::test]
async fn off_the_routes_are_404_and_nothing_links_to_them() {
    let e = env().await;
    assert!(!crate::web::debug_booking::resolve(&e.state).await.value);
    let admin = e.session().await;
    let (status, _) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);
    let resp = e
        .post_as(CANCEL_PATH, &admin, &format!("_csrf={CSRF}"))
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 0);
    // A member and a signed-out visitor get the same 404, not a 403 or
    // the login page: off, the page does not exist for anyone.
    let (_, member) = other_user(&e.pool, "member@example.com", "user", "m").await;
    let (status, _) = e.get_as(PATH, &member).await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    let resp = e.app().oneshot(get(PATH)).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);
    // The admin's pages carry no link.
    for uri in [TON_PAGE, "/dashboard/bookings", "/dashboard"] {
        let (status, page) = e.get_as(uri, &admin).await;
        assert_eq!(status, axum::http::StatusCode::OK, "{uri}");
        assert!(!page.contains(PATH), "{uri}");
    }
    // On, the sidebar links it, on every dashboard page.
    let e = env_on().await;
    let admin = e.session().await;
    for uri in [TON_PAGE, "/dashboard/bookings", "/dashboard"] {
        let (_, page) = e.get_as(uri, &admin).await;
        assert!(page.contains(&format!("href=\"{PATH}\"")), "{uri}");
    }
}

#[tokio::test]
async fn only_admins_not_impersonating_and_only_with_csrf() {
    let e = env_on().await;
    let (_, user) = other_user(&e.pool, "member@example.com", "user", "m").await;
    let (status, _) = e.get_as(PATH, &user).await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
    let resp = e.post_as(PATH, &user, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    let resp = e
        .post_as(CANCEL_PATH, &user, &format!("_csrf={CSRF}"))
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    // Signed out: the login page.
    let resp = e.app().oneshot(get(PATH)).await.unwrap();
    assert!(location(&resp).starts_with("/auth/login"));

    // CSRF: no cookie, or a token that does not match it.
    let admin = e.session().await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(PATH)
        .header("cookie", format!("__Host-calrs_session={admin}"))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form("")))
        .unwrap();
    let resp = e.app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    let resp = e
        .app()
        .oneshot(post_form(PATH, &admin, "another-token", &form("")))
        .await
        .unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    let resp = e
        .app()
        .oneshot(post_form(
            CANCEL_PATH,
            &admin,
            "another-token",
            &format!("_csrf={CSRF}"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 0, "nothing was booked");

    // Impersonating someone: refused, page and POST.
    let (member_id, _) = other_user(&e.pool, "other@example.com", "user", "o").await;
    let cookie = format!(
        "__Host-calrs_session={admin}; __Host-calrs_csrf={CSRF}; __Host-calrs_impersonate={member_id}"
    );
    for (method, uri, body) in [("GET", PATH, String::new()), ("POST", PATH, form(""))] {
        let req = axum::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("cookie", &cookie)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();
        let resp = e.app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN, "{method}");
    }
}

#[tokio::test]
async fn the_form_is_prefilled_from_the_admins_profile() {
    let e = env_on().await;
    sqlx::query("UPDATE users SET booking_email = 'book@example.com', timezone = 'Asia/Almaty' WHERE username = 'testuser'")
        .execute(&e.pool)
        .await
        .unwrap();
    let id = insert_personal_et(&e.pool, &e.admin_id().await, "aaa-free").await;
    assert!(!id.is_empty());
    let admin = e.session().await;
    let (status, page) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    // The test type first, though "aaa-free" sorts before it.
    assert!(
        page.contains(r#"<option value="test-meeting" selected>"#),
        "{page}"
    );
    assert!(page.contains(r#"<option value="aaa-free">"#));
    assert!(page.contains(r#"name="name" value="Test User""#));
    assert!(page.contains(r#"<option value="test@example.com">test@example.com</option>"#));
    assert!(page.contains(r#"<option value="book@example.com" selected>book@example.com</option>"#));
    assert!(page.contains("Asia/Almaty"));
    assert!(page.contains(&crate::i18n::translate("en", "dbg-notes-default", None)));
    assert!(page.contains("$40.00"));
    // The next free times, as the form posts them.
    assert!(page.contains(r#"<select id="dbg-slot" name="slot""#));
    let first = page.split(r#"<select id="dbg-slot""#).nth(1).unwrap();
    let value = first
        .split("value=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    assert!(
        crate::web::debug_booking::split_slot(value).is_some(),
        "{value}"
    );
    // Telegram off here: the box cannot be ticked.
    assert!(page.contains(r#"id="dbg-own-chat" disabled"#) || crate::telegram::enabled());
    // Another type by `?et=`.
    let (_, page) = e.get_as(&format!("{PATH}?et=aaa-free"), &admin).await;
    assert!(page.contains(r#"<option value="aaa-free" selected>"#));
}

#[tokio::test]
async fn a_booking_goes_through_the_public_path_to_its_payment_page() {
    let e = env_on().await;
    let admin = e.session().await;
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let token = location(&resp)
        .strip_prefix("/booking/pay/")
        .expect("the payment page")
        .to_string();
    let bid: String = sqlx::query_scalar("SELECT booking_id FROM ton_invoices WHERE pay_token = ?")
        .bind(&token)
        .fetch_one(&e.pool)
        .await
        .unwrap();
    // A real priced booking: pending, holding, an open invoice of $40.
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    let inv = e.inv(&bid).await;
    assert_eq!((inv.status.as_str(), inv.amount as u128), ("open", AMOUNT));
    assert_eq!(
        e.booking_col(&bid, "guest_email").await.as_deref(),
        Some("test@example.com")
    );
    assert_eq!(e.booking_col(&bid, "notes").await.as_deref(), Some("debug"));
    // Marked as this admin's test booking.
    assert_eq!(
        e.debug_rows().await,
        vec![(bid.clone(), e.admin_id().await, 0)]
    );
    // Never paid by itself.
    let r = e.step(e.now).await;
    assert_eq!(r.confirmed, 0);
    assert_eq!(e.inv(&bid).await.status, "open");

    // Again: the public path refuses it (the time is held, and the address
    // has an open invoice), and nothing is marked.
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    assert_eq!(e.debug_rows().await.len(), 1);

    // Someone else's address is refused before anything is booked.
    let body = form("").replace("test%40example.com", "victim%40example.org");
    let resp = e.post_as(PATH, &admin, &body).await;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let page = body_string(resp).await;
    assert!(page.contains(&crate::i18n::translate("en", "dbg-err-email", None)));
    // A type that is not the admin's, a slot not in the form's shape.
    let (_, _) = other_user(&e.pool, "b@example.com", "admin", "theirs").await;
    let body = form("").replace("et=test-meeting", "et=theirs");
    let resp = e.post_as(PATH, &admin, &body).await;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = form("").replace("T10%3A00", "+10%3A00");
    let resp = e.post_as(PATH, &admin, &body).await;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 1);

    // The page lists it with its payment page.
    let (_, page) = e.get_as(PATH, &admin).await;
    assert!(page.contains(&format!("data-booking=\"{bid}\"")));
    assert!(page.contains(&format!("/booking/pay/{token}")));
}

#[tokio::test]
async fn a_ladder_price_is_quoted_and_invoiced_like_a_guests() {
    // A priced first step: an invoice at it, `pricing_mode = 'ladder'`.
    let e = ladder_env_on("20,40").await;
    let admin = e.session().await;
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::SEE_OTHER,
        "{}",
        body_string(resp).await
    );
    let (bid, _, _) = e.debug_rows().await.pop().unwrap();
    let (amount, mode): (i64, String) =
        sqlx::query_as("SELECT amount_nano, pricing_mode FROM ton_invoices WHERE booking_id = ?")
            .bind(&bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!((amount as u128, mode.as_str()), (20 * NANO, "ladder"));

    // A free first step: no invoice, the request waits for the owner, and
    // the confirmation page is the guest's.
    let e = ladder_env_on("free,20,40").await;
    let admin = e.session().await;
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let rows = e.debug_rows().await;
    assert_eq!(rows.len(), 1, "the free booking is marked");
    assert_eq!(booking_status(&e.pool, &rows[0].0).await, "pending");
    let invoices: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ton_invoices")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(invoices, 0);
}

#[tokio::test]
async fn a_used_free_step_is_quoted_at_the_first_price_for_this_guest() {
    // Regression: the page quoted the owner-wide step ("free") while the
    // booking path priced this guest (free step used: the first price), so
    // the booking came back as "the price changed" and the quote page's
    // own button booked past this page (no mark, no own-chat routing).
    let e = ladder_env_on("free,20,40").await;
    let admin = e.session().await;
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let (first, _, _) = e.debug_rows().await.pop().expect("the free booking");
    assert_eq!(booking_status(&e.pool, &first).await, "pending");
    let (_, page) = e.get_as(PATH, &admin).await;
    assert!(!page.contains(r#"id="dbg-free-used""#), "nothing used yet");

    // Cancelled: the demand is 0 again, the step "free" for a new guest,
    // but this address has used its free step.
    let resp = e
        .post_as(CANCEL_PATH, &admin, &format!("_csrf={CSRF}"))
        .await;
    assert_eq!(location(&resp), format!("{PATH}?cancelled=1"));
    let (_, page) = e.get_as(PATH, &admin).await;
    assert!(page.contains(r#"id="dbg-free-used""#), "{page}");
    assert!(page.contains("20 TON now"), "the first price");

    // Booked again with the same address: the first price, an invoice, the
    // payment page, and the booking is marked as this admin's test.
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::SEE_OTHER,
        "{}",
        body_string(resp).await
    );
    assert!(location(&resp).starts_with("/booking/pay/"));
    let rows = e.debug_rows().await;
    assert_eq!(rows.len(), 2);
    let (second, _, _) = rows.into_iter().find(|r| r.0 != first).unwrap();
    let (amount, mode): (i64, String) =
        sqlx::query_as("SELECT amount_nano, pricing_mode FROM ton_invoices WHERE booking_id = ?")
            .bind(&second)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!((amount as u128, mode.as_str()), (20 * NANO, "ladder"));
    assert_eq!(e.inv(&second).await.status, "open");
    // Cancel-all reaches it.
    let resp = e
        .post_as(CANCEL_PATH, &admin, &format!("_csrf={CSRF}"))
        .await;
    assert_eq!(location(&resp), format!("{PATH}?cancelled=1"));
    assert_eq!(booking_status(&e.pool, &second).await, "declined");
}

#[tokio::test]
async fn a_private_type_gets_a_one_time_invite_for_the_admin() {
    let e = env_on().await;
    sqlx::query("UPDATE event_types SET visibility = 'private' WHERE id = ?")
        .bind(&e.et_id)
        .execute(&e.pool)
        .await
        .unwrap();
    let admin = e.session().await;
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let invites: Vec<(String, i64, i64)> =
        sqlx::query_as("SELECT guest_email, max_uses, used_count FROM booking_invites")
            .fetch_all(&e.pool)
            .await
            .unwrap();
    assert_eq!(invites, vec![("test@example.com".to_string(), 1, 1)]);
    // Refused (the time is taken now): the unused invite is removed.
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM booking_invites")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 1);
}

#[tokio::test]
async fn telegram_goes_to_the_admins_own_chat_only_when_ticked() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let _lock = crate::test_support::ENV_LOCK.lock().await;
    let _guard = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", BASE);
    let e = env_on().await;
    crate::telegram::store::save_event_type_mode(&e.pool, &e.et_id, "optional")
        .await
        .unwrap();
    let admin = e.session().await;

    // Ticked without a linked chat: refused, nothing booked.
    let resp = e.post_as(PATH, &admin, &form("&own_chat=1")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    assert!(e.debug_rows().await.is_empty());
    let (_, page) = e.get_as(PATH, &admin).await;
    assert!(page.contains(r#"id="dbg-own-chat" disabled"#));
    assert!(page.contains("/dashboard/settings/telegram"));

    sqlx::query(
        "INSERT INTO telegram_user_chats (user_id, chat_id, telegram_username)
         SELECT id, ?, 'host_tg' FROM users WHERE username = 'testuser'",
    )
    .bind(HOST_CHAT)
    .execute(&e.pool)
    .await
    .unwrap();
    let (_, page) = e.get_as(PATH, &admin).await;
    assert!(page.contains("@host_tg"));
    assert!(
        page.contains(r#"name="telegram" value="host_tg""#),
        "prefilled"
    );
    assert!(!page.contains(r#"id="dbg-own-chat" disabled"#));

    // Ticked: the guest link is the admin's chat, and what to pay arrives there.
    let resp = e
        .post_as(PATH, &admin, &form("&telegram=host_tg&own_chat=1"))
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    tg.flush(&e.pool).await;
    let (bid, _, own) = e.debug_rows().await.pop().unwrap();
    assert_eq!(own, 1);
    let link: (Option<i64>, Option<i64>, Option<String>) = sqlx::query_as(
        "SELECT chat_id, token_chat_id, telegram_username FROM telegram_guest_links WHERE booking_id = ?",
    )
    .bind(&bid)
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(
        link,
        (Some(HOST_CHAT), Some(HOST_CHAT), Some("host_tg".into()))
    );
    let texts = tg.api.texts_to(HOST_CHAT);
    assert!(
        texts.iter().any(|t| t.contains("TON")),
        "what to pay: {texts:?}"
    );

    // Not ticked: the link row waits for the guest's own /start.
    sqlx::query("UPDATE users SET booking_email = 'book@example.com' WHERE username = 'testuser'")
        .execute(&e.pool)
        .await
        .unwrap();
    let body = form("&telegram=host_tg")
        .replace("test%40example.com", "book%40example.com")
        .replace("T10%3A00", "T11%3A00");
    let resp = e.post_as(PATH, &admin, &body).await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let rows = e.debug_rows().await;
    let (bid2, _, own) = rows.iter().find(|r| r.0 != bid).unwrap().clone();
    assert_eq!(own, 0);
    let chat: Option<i64> =
        sqlx::query_scalar("SELECT chat_id FROM telegram_guest_links WHERE booking_id = ?")
            .bind(&bid2)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!(chat, None);

    // A type that offers no Telegram field: a row is made and linked.
    crate::telegram::store::save_event_type_mode(&e.pool, &e.et_id, "off")
        .await
        .unwrap();
    sqlx::query("UPDATE ton_invoices SET status = 'closed'")
        .execute(&e.pool)
        .await
        .unwrap();
    let body = form("&own_chat=1").replace("T10%3A00", "T12%3A00");
    let resp = e.post_as(PATH, &admin, &body).await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let linked: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM telegram_guest_links g JOIN debug_bookings d ON d.booking_id = g.booking_id
          WHERE d.telegram_own_chat = 1 AND g.chat_id = ?",
    )
    .bind(HOST_CHAT)
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(linked, 2);
}

#[tokio::test]
async fn a_failed_own_chat_routing_is_recorded_and_shown() {
    // Regression: the mark stored what was asked (own chat), not what was
    // done, so a routing that failed still listed "my chat".
    let _tg = fake::install_fake("ilia_test_bot").await;
    let _lock = crate::test_support::ENV_LOCK.lock().await;
    let _guard = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", BASE);
    let e = env_on().await;
    crate::telegram::store::save_event_type_mode(&e.pool, &e.et_id, "optional")
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO telegram_user_chats (user_id, chat_id, telegram_username)
         SELECT id, ?, 'host_tg' FROM users WHERE username = 'testuser'",
    )
    .bind(HOST_CHAT)
    .execute(&e.pool)
    .await
    .unwrap();
    // The chat is unlinked between the page's check and the routing (in
    // another tab, say): here, as the booking row is written.
    sqlx::query(
        "CREATE TRIGGER unlink_on_booking AFTER INSERT ON bookings
         BEGIN DELETE FROM telegram_user_chats; END",
    )
    .execute(&e.pool)
    .await
    .unwrap();
    let admin = e.session().await;
    let resp = e
        .post_as(PATH, &admin, &form("&telegram=host_tg&own_chat=1"))
        .await;
    // The booking stands, but the admin sees the warning, not the
    // payment page.
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let page = unescape(&body_string(resp).await);
    assert!(page.contains(r#"id="debug-booking-warning""#));
    assert!(page.contains(&crate::i18n::translate("en", "dbg-own-chat-failed", None)));
    let (bid, _, own) = e.debug_rows().await.pop().expect("marked as a test");
    assert_eq!(own, 0, "recorded as not routed");
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    assert_eq!(e.inv(&bid).await.status, "open");
    let chat: Option<i64> =
        sqlx::query_scalar("SELECT chat_id FROM telegram_guest_links WHERE booking_id = ?")
            .bind(&bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!(chat, None);
    assert!(!page.contains(&crate::i18n::translate("en", "dbg-own-chat-yes", None)));
    assert!(page.contains(&format!("data-booking=\"{bid}\"")));
}

#[tokio::test]
async fn cancel_all_cancels_only_this_admins_test_bookings() {
    let e = env_on().await;
    let admin = e.session().await;
    // A's test booking, through the page.
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let (mine, _, _) = e.debug_rows().await.pop().unwrap();
    // A guest's real booking of A's type.
    let (real, _) = e.book_http("guest@test.com", "11:00", "198.51.100.9").await;
    // Admin B and B's own test booking.
    let (b_id, b_session) = other_user(&e.pool, "b@example.com", "admin", "b-test").await;
    sqlx::query("UPDATE users SET name = 'B' WHERE id = ?")
        .bind(&b_id)
        .execute(&e.pool)
        .await
        .unwrap();
    let body = format!(
        "_csrf={CSRF}&et=b-test&slot={}&name=B&email=b%40example.com",
        urlencoding::encode(&slot())
    );
    let resp = e.post_as(PATH, &b_session, &body).await;
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::OK,
        "B's type is free"
    );
    let theirs = e
        .debug_rows()
        .await
        .into_iter()
        .find(|r| r.1 == b_id)
        .expect("B's test booking")
        .0;

    let resp = e
        .post_as(CANCEL_PATH, &admin, &format!("_csrf={CSRF}"))
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    assert_eq!(location(&resp), format!("{PATH}?cancelled=1"));
    assert_eq!(booking_status(&e.pool, &mine).await, "declined");
    assert_eq!(e.inv(&mine).await.status, "closed");
    assert_eq!(booking_status(&e.pool, &real).await, "pending");
    assert_eq!(e.inv(&real).await.status, "open");
    assert_eq!(booking_status(&e.pool, &theirs).await, "confirmed");
    // Nothing left: the button says 0 and a second run cancels nothing.
    let resp = e
        .post_as(CANCEL_PATH, &admin, &format!("_csrf={CSRF}"))
        .await;
    assert_eq!(location(&resp), format!("{PATH}?cancelled=0"));
    let (_, page) = e.get_as(&format!("{PATH}?cancelled=1"), &admin).await;
    assert!(page.contains("data-cancelled=\"1\""));
}

// --- the switch at the top of the TON page -----------------------------------

/// The switch's row: (value, owner).
async fn switch_row(e: &Env) -> Option<(String, String)> {
    sqlx::query_as("SELECT value, owner FROM ton_settings WHERE key = 'CALRS_DEBUG_BOOKING'")
        .fetch_optional(&e.pool)
        .await
        .unwrap()
}

/// The switch's audit rows: (actor, source, before, after, owner).
#[allow(clippy::type_complexity)]
async fn switch_audit(
    e: &Env,
) -> Vec<(
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
)> {
    sqlx::query_as(
        "SELECT actor_user_id, source, before, after, owner FROM ton_settings_audit
          WHERE key = 'CALRS_DEBUG_BOOKING' ORDER BY id",
    )
    .fetch_all(&e.pool)
    .await
    .unwrap()
}

impl Env {
    async fn toggle(&self, session: &str, on: bool) -> axum::http::Response<Body> {
        let on = if on { "1" } else { "0" };
        self.post_as(TOGGLE_PATH, session, &format!("_csrf={CSRF}&on={on}"))
            .await
    }
}

/// The Debug block of the TON page.
fn debug_block(page: &str) -> &str {
    let start = page.find(r#"id="debug-block""#).expect("the Debug block");
    let end = start + page[start..].find(r#"id="ton-status""#).unwrap();
    &page[start..end]
}

#[tokio::test]
async fn the_switch_on_the_ton_page_is_live_and_audited() {
    let e = env().await;
    let admin = e.session().await;
    let admin_id = e.admin_id().await;
    let (status, page) = e.get_as(TON_PAGE, &admin).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    // At the top: before the payments' status.
    let block = debug_block(&page);
    assert!(
        block.contains(r#"data-debug="off" data-source="default""#),
        "{block}"
    );
    assert!(block.contains(&format!(r#"action="{TOGGLE_PATH}""#)));
    assert!(block.contains(r#"name="on" value="1""#));
    assert!(block.contains(r#"role="switch" aria-checked="false""#));
    assert!(
        page.contains(r#"class="card dbg-hazard" id="debug-block""#),
        "styled as debug"
    );
    assert!(!block.contains("disabled"), "not locked");
    assert!(!block.contains(r#"id="debug-open""#));

    // On: live at once, no restart, no new state.
    let resp = e.toggle(&admin, true).await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    assert_eq!(location(&resp), format!("{TON_PAGE}?debug=on"));
    assert_eq!(switch_row(&e).await, Some(("1".into(), "panel".into())));
    let (_, page) = e.get_as(&format!("{TON_PAGE}?debug=on"), &admin).await;
    let block = debug_block(&page);
    assert!(block.contains(r#"data-debug="on" data-source="panel""#));
    assert!(block.contains(r#"data-notice="on""#));
    assert!(block.contains(&format!(
        r#"href="{PATH}" class="dbg-open" id="debug-open""#
    )));
    assert!(block.contains(&crate::i18n::translate("en", "ton-admin-debug-open", None)));
    assert!(block.contains(r#"name="on" value="0""#));
    let (status, page) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    // The debug look: the banner and the hazard frame.
    assert!(page.contains(r#"id="debug-banner""#));
    assert!(page.contains("DEBUG · test booking"));
    assert!(page.contains(r#"class="dbg-hazard" id="debug-zone""#));
    let (_, page) = e.get_as("/dashboard/bookings", &admin).await;
    assert!(
        page.contains(&format!("href=\"{PATH}\"")),
        "the sidebar link"
    );

    // Off: 404 again, no link, at once.
    let resp = e.toggle(&admin, false).await;
    assert_eq!(location(&resp), format!("{TON_PAGE}?debug=off"));
    assert_eq!(switch_row(&e).await, Some(("0".into(), "panel".into())));
    let (status, _) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);
    let (_, page) = e.get_as("/dashboard/bookings", &admin).await;
    assert!(!page.contains(PATH));

    // Audited like the TON keys: who, from where, before and after.
    let panel = || Some("panel".to_string());
    assert_eq!(
        switch_audit(&e).await,
        vec![
            (
                Some(admin_id.clone()),
                "panel".into(),
                None,
                Some("1".into()),
                panel()
            ),
            (
                Some(admin_id.clone()),
                "panel".into(),
                Some("1".into()),
                Some("0".into()),
                panel()
            ),
        ]
    );
    let (_, page) = e.get_as(TON_PAGE, &admin).await;
    assert!(page.contains("debug_booking"), "listed in the last changes");
    // The same again: nothing to record.
    e.toggle(&admin, false).await;
    assert_eq!(switch_audit(&e).await.len(), 2);
    // The rest of the TON page's form never carries it.
    let resp = e
        .post_as(TON_PAGE, &admin, &format!("_csrf={CSRF}&debug_booking=on"))
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    assert_eq!(switch_row(&e).await, Some(("0".into(), "panel".into())));
}

#[tokio::test]
async fn the_switch_takes_an_admin_with_csrf_and_not_impersonating() {
    let e = env().await;
    let (_, member) = other_user(&e.pool, "member@example.com", "user", "m").await;
    let resp = e.toggle(&member, true).await;
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    let admin = e.session().await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(TOGGLE_PATH)
        .header("cookie", format!("__Host-calrs_session={admin}"))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("on=1"))
        .unwrap();
    let resp = e.app().oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::FORBIDDEN,
        "no CSRF cookie"
    );
    let resp = e
        .app()
        .oneshot(post_form(
            TOGGLE_PATH,
            &admin,
            "another-token",
            "_csrf=x&on=1",
        ))
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::FORBIDDEN,
        "token mismatch"
    );
    let (member_id, _) = other_user(&e.pool, "other@example.com", "user", "o").await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(TOGGLE_PATH)
        .header(
            "cookie",
            format!("__Host-calrs_session={admin}; __Host-calrs_csrf={CSRF}; __Host-calrs_impersonate={member_id}"),
        )
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(format!("_csrf={CSRF}&on=1")))
        .unwrap();
    let resp = e.app().oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::FORBIDDEN,
        "impersonating"
    );
    assert_eq!(switch_row(&e).await, None, "nothing stored");
    assert!(switch_audit(&e).await.is_empty());
    let (status, _) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_environment_or_config_apply_locks_the_switch() {
    let e = env().await;
    let admin = e.session().await;
    let started = e.state.ton.env();

    // The environment on: the page is served, the toggle locked with why.
    e.state.ton.set_env(started.clone().with(ENV, "1"));
    let (status, _) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let (_, page) = e.get_as(TON_PAGE, &admin).await;
    let block = debug_block(&page);
    assert!(
        block.contains(r#"data-debug="on" data-source="env""#),
        "{block}"
    );
    assert!(block.contains(r#"aria-checked="true""#));
    assert!(block.contains(" disabled>"), "{block}");
    assert!(block.contains(r#"id="debug-locked""#));
    assert!(block.contains("CALRS_DEBUG_BOOKING=1"), "{block}");
    assert!(block.contains(r#"id="debug-open""#));
    // A forced POST changes nothing.
    let resp = e.toggle(&admin, false).await;
    assert_eq!(location(&resp), format!("{TON_PAGE}?debug=locked"));
    assert_eq!(switch_row(&e).await, None);
    assert!(switch_audit(&e).await.is_empty());
    let (status, _) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let (_, page) = e.get_as(&format!("{TON_PAGE}?debug=locked"), &admin).await;
    assert!(debug_block(&page).contains(r#"data-notice="locked""#));

    // The environment off wins over a stored on.
    on(&e).await;
    e.state.ton.set_env(started.clone().with(ENV, "0"));
    let (status, _) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    let (_, page) = e.get_as(TON_PAGE, &admin).await;
    assert!(debug_block(&page).contains(r#"data-debug="off" data-source="env""#));
    // A typo is off, and still locked.
    e.state.ton.set_env(started.clone().with(ENV, "enabled"));
    let (status, _) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    e.toggle(&admin, true).await;
    assert_eq!(switch_row(&e).await, Some(("1".into(), "panel".into())));
    assert!(switch_audit(&e).await.is_empty());

    // Owned by config apply: locked too, with its own reason.
    e.state.ton.set_env(started);
    sqlx::query("UPDATE ton_settings SET owner = 'apply' WHERE key = 'CALRS_DEBUG_BOOKING'")
        .execute(&e.pool)
        .await
        .unwrap();
    let (status, _) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let (_, page) = e.get_as(TON_PAGE, &admin).await;
    let block = debug_block(&page);
    assert!(block.contains(r#"data-debug="on" data-source="apply""#));
    assert!(block.contains(&crate::i18n::translate(
        "en",
        "ton-admin-debug-locked-apply",
        None
    )));
    let resp = e.toggle(&admin, false).await;
    assert_eq!(location(&resp), format!("{TON_PAGE}?debug=locked"));
    assert_eq!(switch_row(&e).await, Some(("1".into(), "apply".into())));
    // Without the row: off by default, the page's again.
    sqlx::query("DELETE FROM ton_settings WHERE key = 'CALRS_DEBUG_BOOKING'")
        .execute(&e.pool)
        .await
        .unwrap();
    let (status, _) = e.get_as(PATH, &admin).await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn only_test_bookings_carry_the_test_badge() {
    let e = env_on().await;
    let admin = e.session().await;
    let resp = e.post_as(PATH, &admin, &form("")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let test_pay = location(&resp);
    let test_id: String =
        sqlx::query_scalar("SELECT booking_id FROM ton_invoices WHERE pay_token = ?")
            .bind(test_pay.strip_prefix("/booking/pay/").unwrap())
            .fetch_one(&e.pool)
            .await
            .unwrap();
    // A guest's booking of the same type.
    let (guest_id, _) = e.booking("guest@example.org").await;
    let guest_token: String =
        sqlx::query_scalar("SELECT pay_token FROM ton_invoices WHERE booking_id = ?")
            .bind(&guest_id)
            .fetch_one(&e.pool)
            .await
            .unwrap();

    let (status, page) = e.get_as("/dashboard/bookings", &admin).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(
        page.contains(&format!("/dashboard/bookings/{guest_id}/")),
        "both are listed"
    );
    assert_eq!(page.matches("data-test-booking").count(), 1, "one badge");
    let at = page.find("data-test-booking").unwrap();
    let next = &page[at..];
    let row = &next[next.find("/dashboard/bookings/").unwrap()..];
    assert!(
        row.starts_with(&format!("/dashboard/bookings/{test_id}/")),
        "on the test booking"
    );
    assert!(page.contains(">TEST</span>"));

    // The payment page: the test one only (the owner is its guest).
    let resp = e.app().oneshot(get(&test_pay)).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    assert!(body_string(resp).await.contains("data-test-booking"));
    let resp = e
        .app()
        .oneshot(get(&format!("/booking/pay/{guest_token}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    assert!(!body_string(resp).await.contains("data-test-booking"));

    // The badge stays when the switch is off: it marks the booking.
    sqlx::query("DELETE FROM ton_settings WHERE key = 'CALRS_DEBUG_BOOKING'")
        .execute(&e.pool)
        .await
        .unwrap();
    let (_, page) = e.get_as("/dashboard/bookings", &admin).await;
    assert_eq!(page.matches("data-test-booking").count(), 1);
}

/// The words around the switch say what it does now: the migration's
/// comment names the switch (not only the variable), the TON docs say a
/// `debug_booking` change needs no restart, and the Russian "no free time"
/// line agrees with its numeral ("ближайший 31 день").
#[test]
fn the_texts_around_the_switch_say_what_it_does() {
    let migration = include_str!("../../../migrations/ilia_013_debug_bookings.sql");
    assert!(
        !migration.contains("on only while\n-- CALRS_DEBUG_BOOKING is set"),
        "the variable is no longer the only way on"
    );
    assert!(migration.contains("the Debug block of /dashboard/admin/ton"));

    let docs = include_str!("../../../docs/src/ton.md");
    let para = docs
        .lines()
        .find(|l| l.starts_with("`site.ton` sets the same keys"))
        .expect("the site.ton paragraph");
    assert!(
        para.contains(
            "(except `debug_booking`, which is read on every request and takes effect at once)"
        ),
        "{para}"
    );

    let ru = crate::i18n::translate("ru", "dbg-no-slots", None);
    assert_eq!(ru, "В ближайший 31 день нет свободного времени.");
    assert!(!ru.contains("ближайшие 31 день"));
}
