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
    // A plain card on the TON page: the hazard frame is only on the test page itself.
    assert!(
        page.contains(r#"class="card" id="debug-block""#),
        "a plain card"
    );
    assert!(
        !block.contains("dbg-hazard") && !block.contains("dbg-banner"),
        "{block}"
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
            "(except `debug_booking` and `debug_booking_telegram`, which are read on every request and take effect at once)"
        ),
        "{para}"
    );

    let ru = crate::i18n::translate("ru", "dbg-no-slots", None);
    assert_eq!(ru, "В ближайший 31 день нет свободного времени.");
    assert!(!ru.contains("ближайшие 31 день"));
}

// --- the prefilled guest Telegram username ------------------------------------

/// The username's row: (value, owner).
async fn telegram_row(e: &Env) -> Option<(String, String)> {
    sqlx::query_as(
        "SELECT value, owner FROM ton_settings WHERE key = 'CALRS_DEBUG_BOOKING_TELEGRAM'",
    )
    .fetch_optional(&e.pool)
    .await
    .unwrap()
}

fn telegram_value(page: &str) -> &str {
    page.split(r#"name="telegram" value=""#)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("the Telegram field")
}

fn telegram_from(page: &str) -> &str {
    page.split(r#"id="dbg-telegram-from" data-from=""#)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("where the username came from")
}

/// The owner's "I asked you to fill it in": the guest Telegram username is
/// the linked chat's username, else the stored setting (the Debug block of
/// the TON page, audited; `config apply` locks it), else empty.
#[tokio::test]
async fn the_guest_telegram_username_is_the_chats_then_the_settings() {
    use crate::web::debug_booking::TELEGRAM_PATH;
    let e = env_on().await;
    crate::telegram::store::save_event_type_mode(&e.pool, &e.et_id, "required")
        .await
        .unwrap();
    let admin = e.session().await;
    // Nothing known: empty, and the page says where to set one.
    let (_, page) = e.get_as(PATH, &admin).await;
    assert_eq!(telegram_value(&page), "");
    assert_eq!(telegram_from(&page), "");
    assert!(page.contains(&crate::i18n::translate("en", "dbg-telegram-none", None)));
    assert!(page.contains(r#"name="telegram" value="" maxlength="64" autocomplete="off""#));

    // Set in the Debug block, as the booking form takes it.
    let resp = e
        .post_as(
            TELEGRAM_PATH,
            &admin,
            &format!("_csrf={CSRF}&debug_booking_telegram=%40getmonolith"),
        )
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    assert_eq!(
        location(&resp),
        format!("{TON_PAGE}?debug=tg-saved#debug-block")
    );
    assert_eq!(
        telegram_row(&e).await,
        Some(("getmonolith".into(), "panel".into()))
    );
    let audited: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ton_settings_audit WHERE key = 'CALRS_DEBUG_BOOKING_TELEGRAM' AND source = 'panel'",
    )
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(audited, 1);
    let (_, page) = e.get_as(PATH, &admin).await;
    assert_eq!(telegram_value(&page), "getmonolith");
    assert_eq!(telegram_from(&page), "setting");
    let (_, ton) = e.get_as(TON_PAGE, &admin).await;
    let block = debug_block(&ton);
    assert!(
        block.contains(r#"name="debug_booking_telegram" value="getmonolith""#),
        "{block}"
    );
    // The same again, or not a username: nothing changes.
    let same = e
        .post_as(
            TELEGRAM_PATH,
            &admin,
            &format!("_csrf={CSRF}&debug_booking_telegram=getmonolith"),
        )
        .await;
    assert!(location(&same).contains("debug=tg-same"));
    for bad in ["abc", "get+monolith", "1getmonolith"] {
        let resp = e
            .post_as(
                TELEGRAM_PATH,
                &admin,
                &format!("_csrf={CSRF}&debug_booking_telegram={bad}"),
            )
            .await;
        assert!(location(&resp).contains("debug=tg-invalid"), "{bad}");
    }
    assert_eq!(telegram_row(&e).await.unwrap().0, "getmonolith");

    // A linked chat's username comes first.
    sqlx::query(
        "INSERT INTO telegram_user_chats (user_id, chat_id, telegram_username)
         SELECT id, ?, 'host_tg' FROM users WHERE username = 'testuser'",
    )
    .bind(HOST_CHAT)
    .execute(&e.pool)
    .await
    .unwrap();
    let (_, page) = e.get_as(PATH, &admin).await;
    assert_eq!(telegram_value(&page), "host_tg");
    assert_eq!(telegram_from(&page), "chat");
    // A chat without a username: the setting again.
    sqlx::query("UPDATE telegram_user_chats SET telegram_username = NULL")
        .execute(&e.pool)
        .await
        .unwrap();
    let (_, page) = e.get_as(PATH, &admin).await;
    assert_eq!(telegram_value(&page), "getmonolith");
    // What was posted wins on a refused POST.
    let resp = e
        .post_as(
            PATH,
            &admin,
            &form("&telegram=typed_name").replace("et=test-meeting", "et=nope"),
        )
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let page = unescape(&body_string(resp).await);
    assert_eq!(telegram_value(&page), "typed_name");

    // Owned by config apply: locked on the TON page, refused by its form.
    sqlx::query(
        "UPDATE ton_settings SET owner = 'apply' WHERE key = 'CALRS_DEBUG_BOOKING_TELEGRAM'",
    )
    .execute(&e.pool)
    .await
    .unwrap();
    let resp = e
        .post_as(
            TELEGRAM_PATH,
            &admin,
            &format!("_csrf={CSRF}&debug_booking_telegram="),
        )
        .await;
    assert!(location(&resp).contains("debug=tg-locked"));
    assert_eq!(
        telegram_row(&e).await,
        Some(("getmonolith".into(), "apply".into()))
    );
    let (_, ton) = e.get_as(TON_PAGE, &admin).await;
    let block = debug_block(&ton);
    assert!(block.contains(r#"id="debug-telegram-locked""#), "{block}");
    assert!(block.contains("site.ton.debug_booking_telegram"));
    let (_, page) = e.get_as(PATH, &admin).await;
    assert_eq!(telegram_value(&page), "getmonolith");

    // Blank on the page's own row: removed (none again).
    sqlx::query(
        "UPDATE ton_settings SET owner = 'panel' WHERE key = 'CALRS_DEBUG_BOOKING_TELEGRAM'",
    )
    .execute(&e.pool)
    .await
    .unwrap();
    let resp = e
        .post_as(
            TELEGRAM_PATH,
            &admin,
            &format!("_csrf={CSRF}&debug_booking_telegram=+"),
        )
        .await;
    assert!(location(&resp).contains("debug=tg-saved"));
    assert_eq!(telegram_row(&e).await, None);
    // CSRF, and never while impersonating: the same rules as the switch.
    let resp = e
        .app()
        .oneshot(post_form(
            TELEGRAM_PATH,
            &admin,
            CSRF,
            "_csrf=wrong&debug_booking_telegram=getmonolith",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    assert_eq!(telegram_row(&e).await, None);
    let audited = telegram_audit_count(&e).await;
    let (member_id, _) = other_user(&e.pool, "imp@example.com", "user", "imp").await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(TELEGRAM_PATH)
        .header(
            "cookie",
            format!("__Host-calrs_session={admin}; __Host-calrs_csrf={CSRF}; __Host-calrs_impersonate={member_id}"),
        )
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "_csrf={CSRF}&debug_booking_telegram=getmonolith"
        )))
        .unwrap();
    let resp = e.app().oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::FORBIDDEN,
        "impersonating"
    );
    assert_eq!(telegram_row(&e).await, None, "nothing stored");
    assert_eq!(telegram_audit_count(&e).await, audited, "nothing audited");
}

async fn telegram_audit_count(e: &Env) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM ton_settings_audit WHERE key = 'CALRS_DEBUG_BOOKING_TELEGRAM'",
    )
    .fetch_one(&e.pool)
    .await
    .unwrap()
}

/// "That should basically always be on": the Telegram box is ticked
/// whenever it can be (Telegram on and a chat linked), and can be unticked.
#[tokio::test]
async fn the_own_chat_box_is_ticked_by_default() {
    let _tg = fake::install_fake("ilia_test_bot").await;
    let _lock = crate::test_support::ENV_LOCK.lock().await;
    let _guard = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", BASE);
    let e = env_on().await;
    let admin = e.session().await;
    // No chat: not ticked, and disabled.
    let (_, page) = e.get_as(PATH, &admin).await;
    assert!(page.contains(r#"id="dbg-own-chat" disabled"#), "no chat");
    sqlx::query(
        "INSERT INTO telegram_user_chats (user_id, chat_id, telegram_username)
         SELECT id, ?, 'host_tg' FROM users WHERE username = 'testuser'",
    )
    .bind(HOST_CHAT)
    .execute(&e.pool)
    .await
    .unwrap();
    let (_, page) = e.get_as(PATH, &admin).await;
    assert!(
        page.contains(r#"name="own_chat" value="1" id="dbg-own-chat" checked>"#),
        "ticked by default"
    );
    // Unticked on a refused POST: shown unticked again.
    let resp = e
        .post_as(
            PATH,
            &admin,
            &form("").replace("et=test-meeting", "et=nope"),
        )
        .await;
    let page = unescape(&body_string(resp).await);
    assert!(page.contains(r#"name="own_chat" value="1" id="dbg-own-chat">"#));
    // Ticked on a refused POST: ticked again.
    let resp = e
        .post_as(
            PATH,
            &admin,
            &form("&own_chat=1").replace("et=test-meeting", "et=nope"),
        )
        .await;
    let page = unescape(&body_string(resp).await);
    assert!(page.contains(r#"id="dbg-own-chat" checked>"#));
}

// --- the timing of a test booking ---------------------------------------------

/// (debug_timing, created_at, expires_at, settle_by, hard_deadline,
/// renewals, renewals_max) of a booking's invoice.
type Timing = (
    Option<String>,
    chrono::DateTime<Utc>,
    chrono::DateTime<Utc>,
    chrono::DateTime<Utc>,
    chrono::DateTime<Utc>,
    i64,
    i64,
);

async fn timing_of(e: &Env, bid: &str) -> Timing {
    let (t, c, x, s, h, r, m): (Option<String>, String, String, String, String, i64, i64) =
        sqlx::query_as(
            "SELECT debug_timing, created_at, expires_at, settle_by, hard_deadline, renewals, renewals_max
             FROM ton_invoices WHERE booking_id = ?",
        )
        .bind(bid)
        .fetch_one(&e.pool)
        .await
        .unwrap();
    let p = |s: &str| crate::web::ton_pay::parse_stamp(s).unwrap();
    (t, p(&c), p(&x), p(&s), p(&h), r, m)
}

/// The debug page list's row for one booking.
fn list_row<'a>(page: &'a str, bid: &str) -> &'a str {
    page.split(&format!("data-booking=\"{bid}\""))
        .nth(1)
        .expect("the booking's row")
        .split("</tr>")
        .next()
        .unwrap()
}

fn renewed_alerts(e: &Env) -> usize {
    e.sink
        .all()
        .iter()
        .filter(|a| matches!(a, TonAlert::Renewed { .. }))
        .count()
}

/// The owner's test, in minutes: a "Fast test" booking's window ends, it
/// is renewed once (the guest is asked to pay again), ends again, the
/// invoice expires and calrs releases the booking; never paid by itself.
/// A guest's booking made meanwhile keeps the site's timing throughout.
#[tokio::test]
async fn a_fast_test_booking_is_renewed_then_cancelled_unpaid() {
    use crate::ton::deadlines::DebugTiming;
    use crate::web::ton_pay::stamp;
    let e = env_on().await;
    let (guest, _) = e.book_http("guest@test.com", "11:00", "198.51.100.9").await;
    let admin = e.session().await;
    let resp = e.post_as(PATH, &admin, &form("&timing=fast")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let (bid, _, _) = e.debug_rows().await.pop().unwrap();
    let (text, created, expires, settle, hard, renewals, max) = timing_of(&e, &bid).await;
    assert_eq!(text.as_deref(), Some(DebugTiming::FAST.text().as_str()));
    assert_eq!(expires - created, Duration::minutes(3));
    assert_eq!(settle - expires, Duration::minutes(1));
    assert_eq!(hard - settle, Duration::minutes(10));
    assert_eq!((renewals, max), (0, 1), "the site allows none here");
    assert_eq!(pending_expires_at(&e.pool, &bid).await, Some(stamp(hard)));
    let g = timing_of(&e, &guest).await;
    assert_eq!(g.0, None);
    assert_eq!(g.2 - g.1, Duration::minutes(30), "the site's window");

    // The page shows the timing and the live deadline.
    let (_, page) = e.get_as(PATH, &admin).await;
    let row = list_row(&page, &bid);
    assert!(row.contains(r#"data-live="1""#), "{row}");
    assert!(row.contains(&format!("data-expires=\"{}\"", expires.timestamp())));
    assert!(row.contains(&format!("data-settle=\"{}\"", settle.timestamp())));
    assert!(row.contains("test: 3 min, renewals 1 × 3 min"), "{row}");
    assert!(row.contains("renewed 0 of 1"), "{row}");

    // The window ends: renewed for 3 more minutes, at the next step.
    assert_eq!(e.step(expires - Duration::seconds(1)).await.renewed, 0);
    let at = expires + Duration::seconds(5);
    e.chain.height(at.timestamp() as u64);
    let r = e.step(at).await;
    assert_eq!((r.renewed, r.expired, r.confirmed), (1, 0, 0));
    let (_, _, expires2, settle2, hard2, renewals, _) = timing_of(&e, &bid).await;
    assert_eq!(expires2, at + Duration::minutes(3));
    assert_eq!(settle2, at + Duration::minutes(4));
    assert_eq!(hard2, at + Duration::minutes(14));
    assert_eq!(renewals, 1);
    assert_eq!(pending_expires_at(&e.pool, &bid).await, Some(stamp(hard2)));
    // Asked to pay again once the indexer has read a minute past it (the
    // debug margin), not the site's three.
    assert_eq!(renewed_alerts(&e), 0);
    let later = at + Duration::seconds(61);
    e.chain.height(later.timestamp() as u64);
    e.step(later).await;
    assert_eq!(renewed_alerts(&e), 1);

    // It ends again: no renewal left; expired a minute after settle_by.
    let over = expires2 + Duration::seconds(5);
    e.chain.height(over.timestamp() as u64);
    let r = e.step(over).await;
    assert_eq!((r.renewed, r.expired), (0, 0));
    let end = settle2 + Duration::minutes(1);
    e.chain.height(end.timestamp() as u64);
    let r = e.step(end).await;
    assert_eq!((r.expired, r.confirmed), (1, 0));
    assert_eq!(e.inv(&bid).await.status, "expired");
    assert_eq!(pending_expires_at(&e.pool, &bid).await, Some(stamp(end)));
    // Expired but still pending until the sweep: the list keeps refreshing,
    // so the owner sees the booking declined without reloading.
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    let (_, page) = e.get_as(PATH, &admin).await;
    let row = list_row(&page, &bid);
    assert!(row.contains(r#"data-live="1""#), "{row}");
    assert!(
        !row.contains("data-expires="),
        "no countdown once expired: {row}"
    );
    // calrs's own sweep cancels it (declined) and frees the time.
    sweep_pending_requests(&e.pool, &[0u8; 32], end + Duration::seconds(30)).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "declined");
    let (_, page) = e.get_as(PATH, &admin).await;
    let row = list_row(&page, &bid);
    assert!(row.contains(r#"data-live="0""#), "settled: {row}");
    let resp = e.post_as(PATH, &admin, &form("&timing=fast")).await;
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::SEE_OTHER,
        "the time is free again"
    );

    // The guest's booking: untouched, the site's window still running.
    assert_eq!(booking_status(&e.pool, &guest).await, "pending");
    let g2 = timing_of(&e, &guest).await;
    assert_eq!((g2.2, g2.5), (g.2, 0));
    assert_eq!(e.inv(&guest).await.status, "open");
}

/// Custom minutes are checked against their bounds; "Normal" (or nothing)
/// is the site's timing, stored as none.
#[tokio::test]
async fn custom_timing_is_bounded_and_normal_is_the_sites() {
    use crate::ton::deadlines::DebugTiming;
    let e = env_on().await;
    let admin = e.session().await;
    for bad in [
        "&timing=custom&t_window=1",
        "&timing=custom&t_renew_window=1441",
        "&timing=custom&t_renewals=6",
        "&timing=custom&t_window=ten",
        "&timing=slow",
    ] {
        let resp = e.post_as(PATH, &admin, &form(bad)).await;
        assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST, "{bad}");
        let page = unescape(&body_string(resp).await);
        assert!(page.contains(&crate::i18n::translate("en", "dbg-err-timing", None)));
        assert!(e.debug_rows().await.is_empty(), "{bad}");
    }
    let resp = e
        .post_as(
            PATH,
            &admin,
            &form("&timing=custom&t_window=5&t_renew_window=4&t_renewals=2"),
        )
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let (bid, _, _) = e.debug_rows().await.pop().unwrap();
    let (text, created, expires, _, _, _, max) = timing_of(&e, &bid).await;
    let want = DebugTiming::custom(5, 4, 2).unwrap();
    assert_eq!(text.as_deref(), Some(want.text().as_str()));
    assert_eq!((expires - created, max), (Duration::minutes(5), 2));

    // Normal: the site's 30 minutes and no renewal (as configured here).
    sqlx::query("UPDATE ton_invoices SET status = 'closed'")
        .execute(&e.pool)
        .await
        .unwrap();
    let body = form("&timing=normal").replace("T10%3A00", "T11%3A00");
    let resp = e.post_as(PATH, &admin, &body).await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let rows = e.debug_rows().await;
    let normal = rows.iter().find(|r| r.0 != bid).unwrap().0.clone();
    let (text, created, expires, _, _, _, max) = timing_of(&e, &normal).await;
    assert_eq!(text, None);
    assert_eq!((expires - created, max), (Duration::minutes(30), 0));
    let (_, page) = e.get_as(PATH, &admin).await;
    assert!(page.contains(r#"id="dbg-timing-normal" checked"#));
    assert!(page.contains("site settings"));
}

/// Nothing a guest posts sets a timing, and a timing on an invoice whose
/// booking is not a marked test booking (a hand edit) is never honoured.
#[tokio::test]
async fn a_guest_cannot_get_a_debug_timing() {
    use crate::ton::deadlines::DebugTiming;
    use crate::web::ton_pay::store;
    let e = env_on().await;
    let fast = urlencoding::encode(&DebugTiming::FAST.text()).into_owned();
    let resp = e
        .post_guest(
            "/u/testuser/test-meeting/book",
            &format!(
                "&date={}&time=10:00&name=Guest&email=guest%40test.com&tz=UTC&timing=fast&t_window=3&debug_timing={fast}",
                far_future_monday()
            ),
        )
        .await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let bid: String = sqlx::query_scalar("SELECT booking_id FROM ton_invoices")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    let (text, created, expires, _, _, _, max) = timing_of(&e, &bid).await;
    assert_eq!(text, None);
    assert_eq!((expires - created, max), (Duration::minutes(30), 0));

    // Written by hand on an unmarked booking: read as none.
    sqlx::query("UPDATE ton_invoices SET debug_timing = ?")
        .bind(DebugTiming::FAST.text())
        .execute(&e.pool)
        .await
        .unwrap();
    let row = store::invoice_for_booking(&e.pool, &bid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.debug_timing, None);
    assert!(!store::open_debug_invoice(&e.pool).await);
    let r = e
        .step(created + Duration::minutes(3) + Duration::seconds(5))
        .await;
    assert_eq!((r.renewed, r.expired), (0, 0));
    assert_eq!(timing_of(&e, &bid).await.2, expires);
}

/// While a test booking's own timing runs, the watcher steps at least every
/// 10 seconds whatever `CALRS_TON_POLL_SECS` says; otherwise as before.
#[tokio::test]
async fn the_watcher_keeps_up_with_a_fast_test_booking() {
    let e = env_with(&[("CALRS_TON_POLL_SECS", "30")]).await;
    on(&e).await;
    let caught = watcher::StepReport {
        caught_up: true,
        ..Default::default()
    };
    let admin = e.session().await;
    let (_guest, _) = e.book_http("guest@test.com", "11:00", "198.51.100.9").await;
    let d = watcher::next_delay(&e.pool, &e.rt, &caught, e.now).await;
    assert_eq!(d, std::time::Duration::from_secs(30));
    let resp = e.post_as(PATH, &admin, &form("&timing=fast")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let d = watcher::next_delay(&e.pool, &e.rt, &caught, e.now).await;
    assert_eq!(d, watcher::DEBUG_POLL);
    // Transient indexer errors do not stretch the step past DEBUG_POLL;
    // a 429, a rejected key or the wrong network keep their backoff.
    e.chain.edit(|c| {
        c.fail_next = vec![
            IndexerError::Server(502),
            IndexerError::Network("timeout".into()),
            IndexerError::Server(503),
            IndexerError::Server(500),
        ]
    });
    let mut failed = None;
    for _ in 0..4 {
        let r = e.step(e.now).await;
        assert!(r.error.is_some(), "{r:?}");
        failed = Some(r);
    }
    let failed = failed.unwrap();
    assert!(e.rt.health().failures >= 4);
    let d = watcher::next_delay(&e.pool, &e.rt, &failed, e.now).await;
    assert_eq!(
        d,
        watcher::DEBUG_POLL,
        "capped while a test invoice is open"
    );
    let limited = watcher::StepReport {
        error: Some(watcher::StepError::Indexer(IndexerError::RateLimited)),
        ..Default::default()
    };
    let d = watcher::next_delay(&e.pool, &e.rt, &limited, e.now).await;
    assert!(d > watcher::DEBUG_POLL, "{d:?}");
    let auth = watcher::StepReport {
        error: Some(watcher::StepError::Indexer(IndexerError::Auth(401))),
        ..Default::default()
    };
    let d = watcher::next_delay(&e.pool, &e.rt, &auth, e.now).await;
    assert_eq!(d, std::time::Duration::from_secs(300));
    sqlx::query(
        "UPDATE ton_invoices SET status = 'expired' WHERE booking_id IN (SELECT booking_id FROM debug_bookings)",
    )
    .execute(&e.pool)
    .await
    .unwrap();
    let d = watcher::next_delay(&e.pool, &e.rt, &caught, e.now).await;
    assert_eq!(d, std::time::Duration::from_secs(30));
    // No open test invoice: a guest's invoice keeps the usual backoff.
    let d = watcher::next_delay(&e.pool, &e.rt, &failed, e.now).await;
    assert!(d > watcher::DEBUG_POLL, "{d:?}");
}

// --- the guest language ---------------------------------------------------

/// The language select of the page, as `(code, selected)`.
fn guest_lang_options(page: &str) -> Vec<(String, bool)> {
    let select = page
        .split(r#"<select id="dbg-guest-lang" name="guest_lang""#)
        .nth(1)
        .expect("the guest language select")
        .split("</select>")
        .next()
        .unwrap();
    select
        .split("<option value=\"")
        .skip(1)
        .map(|o| {
            let code = o.split('"').next().unwrap().to_string();
            let head = o.split('>').next().unwrap();
            (code, head.contains(" selected"))
        })
        .collect()
}

fn selected_guest_lang(page: &str) -> Option<String> {
    let all = guest_lang_options(page);
    let chosen: Vec<_> = all.iter().filter(|(_, s)| *s).collect();
    assert!(chosen.len() <= 1, "{all:?}");
    chosen.first().map(|(c, _)| c.clone())
}

#[tokio::test]
async fn the_guest_language_is_the_admins_own_by_default() {
    let e = env_on().await;
    let admin = e.session().await;
    // No saved language: the page's (here English, no Accept-Language).
    let (_, page) = e.get_as(PATH, &admin).await;
    let codes: Vec<String> = guest_lang_options(&page)
        .into_iter()
        .map(|(c, _)| c)
        .collect();
    let shipped: Vec<String> = crate::i18n::supported_with_labels()
        .map(|(c, _)| c.to_string())
        .collect();
    assert_eq!(codes, shipped, "every shipped language, in order");
    assert!(page.contains(r#"<option value="ru" lang="ru">Русский</option>"#));
    assert_eq!(selected_guest_lang(&page).as_deref(), Some("en"));
    // The page's language is the browser's while nothing is saved.
    let req = axum::http::Request::builder()
        .uri(PATH)
        .header("cookie", format!("__Host-calrs_session={admin}"))
        .header("accept-language", "it-IT,it;q=0.9")
        .body(Body::empty())
        .unwrap();
    let page = unescape(&body_string(e.app().oneshot(req).await.unwrap()).await);
    assert_eq!(selected_guest_lang(&page).as_deref(), Some("it"));
    // The admin's saved interface language wins over the browser.
    set_user_language(&e.pool, "test@example.com", "ru").await;
    let (_, page) = e.get_as(PATH, &admin).await;
    assert_eq!(selected_guest_lang(&page).as_deref(), Some("ru"));
    assert!(page.contains(&crate::i18n::translate("ru", "dbg-guest-lang", None)));
}

/// The test type, free: the booking is confirmed at once and the guest gets
/// the confirmation email.
async fn free_env_on() -> Env {
    let e = env_on().await;
    sqlx::query("UPDATE ton_prices SET enabled = 0")
        .execute(&e.pool)
        .await
        .unwrap();
    e
}

async fn newest_booking(e: &Env) -> String {
    sqlx::query_scalar("SELECT id FROM bookings ORDER BY created_at DESC, rowid DESC LIMIT 1")
        .fetch_one(&e.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn the_chosen_guest_language_is_stored_and_the_guest_emails_follow_it() {
    let e = free_env_on().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    // The owner's case: their interface in Russian, a browser in English.
    set_user_language(&e.pool, "test@example.com", "ru").await;
    let admin = e.session().await;
    let req = post_form(PATH, &admin, CSRF, &form("&guest_lang=fr"));
    let (mut parts, body) = req.into_parts();
    parts
        .headers
        .insert("accept-language", "en-US,en;q=0.9".parse().unwrap());
    let resp = e
        .app()
        .oneshot(axum::http::Request::from_parts(parts, body))
        .await
        .unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let bid = newest_booking(&e).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    assert_eq!(e.booking_col(&bid, "language").await.as_deref(), Some("fr"));
    assert_eq!(e.debug_rows().await.len(), 1);
    // The confirmation page the guest sees is in the guest's language.
    let page = body_string(resp).await;
    assert!(page.contains(r#"lang="fr""#), "{page}");

    // The guest's email in French, the host's in the host's Russian (the
    // admin is both, at the same address).
    let all = settled_mails_to(&mails, "test@example.com").await;
    let guest: Vec<&String> = all
        .iter()
        .filter(|m| {
            m.contains(&crate::i18n::translate(
                "fr",
                "email-confirm-headline",
                None,
            ))
        })
        .collect();
    assert_eq!(guest.len(), 1, "one guest confirmation in French: {all:#?}");
    assert!(guest[0].contains(r#"<html lang="fr">"#), "{}", guest[0]);
    assert!(!all.iter().any(|m| m.contains(&crate::i18n::translate(
        "en",
        "email-confirm-headline",
        None
    ))));
    assert!(
        all.iter().any(|m| m.contains(&crate::i18n::translate(
            "ru",
            "email-host-new-heading",
            None
        ))),
        "{all:#?}"
    );
    // Read back later (a cancellation, a reminder): the stored language.
    assert_eq!(
        crate::email::stored_booking_language(&e.pool, &bid)
            .await
            .as_deref(),
        Some("fr")
    );

    // The list shows it.
    let (_, page) = e.get_as(PATH, &admin).await;
    let row = list_row(&page, &bid);
    assert!(row.contains(r#"data-lang="fr""#), "{row}");
    assert!(row.contains("Français"), "{row}");

    // No field posted: the admin's own language, not the browser's.
    let body = form("").replace("T10%3A00", "T11%3A00");
    let req = post_form(PATH, &admin, CSRF, &body);
    let (mut parts, b) = req.into_parts();
    parts
        .headers
        .insert("accept-language", "en-US,en;q=0.9".parse().unwrap());
    let resp = e
        .app()
        .oneshot(axum::http::Request::from_parts(parts, b))
        .await
        .unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let second = newest_booking(&e).await;
    assert_ne!(second, bid);
    assert_eq!(
        e.booking_col(&second, "language").await.as_deref(),
        Some("ru")
    );

    // A visitor's own switcher pick on the admin's browser does not leak
    // into the test booking either: the posted choice is what counts.
    let body = form("&guest_lang=de-formal").replace("T10%3A00", "T12%3A00");
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(PATH)
        .header(
            "cookie",
            format!("calrs_lang=it; __Host-calrs_session={admin}; __Host-calrs_csrf={CSRF}; calrs_lang=es"),
        )
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    let resp = e.app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let third = newest_booking(&e).await;
    assert_eq!(
        e.booking_col(&third, "language").await.as_deref(),
        Some("de-formal")
    );
}

#[tokio::test]
async fn an_unknown_guest_language_is_refused_before_booking() {
    let e = env_on().await;
    let admin = e.session().await;
    for bad in ["xx", "fr-FR", "en%3B+calrs_lang%3Dru"] {
        let resp = e
            .post_as(PATH, &admin, &form(&format!("&guest_lang={bad}")))
            .await;
        assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST, "{bad}");
        let page = unescape(&body_string(resp).await);
        assert!(
            page.contains(&crate::i18n::translate("en", "dbg-err-lang", None)),
            "{bad}"
        );
    }
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 0);
}

/// A real guest's booking keeps the language of the guest's own page, with
/// the debug page on and an admin's test booking in another language.
#[tokio::test]
async fn a_real_guests_booking_keeps_its_own_language() {
    let e = free_env_on().await;
    let admin = e.session().await;
    set_user_language(&e.pool, "test@example.com", "ru").await;
    let resp = e.post_as(PATH, &admin, &form("&guest_lang=fr")).await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    assert_eq!(
        e.booking_col(&newest_booking(&e).await, "language")
            .await
            .as_deref(),
        Some("fr")
    );
    let guest = |time: &str, cookie: Option<&str>| {
        let b = axum::http::Request::builder()
            .method("POST")
            .uri("/u/testuser/test-meeting/book")
            .header("content-type", "application/x-www-form-urlencoded")
            .header("accept-language", "de-DE,de;q=0.9")
            .header(
                "cookie",
                format!("__Host-calrs_csrf={CSRF}{}", cookie.unwrap_or("")),
            );
        b.body(Body::from(format!(
            "_csrf={CSRF}&date={}&time={time}&name=Guest&email=guest%40test.com&tz=UTC&guest_lang=fr",
            far_future_monday()
        )))
        .unwrap()
    };
    // The browser's German; a `guest_lang` a guest posts is ignored.
    let resp = e.app().oneshot(guest("13:00", None)).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let bid = newest_booking(&e).await;
    assert_eq!(e.booking_col(&bid, "language").await.as_deref(), Some("de"));
    assert!(!e.debug_rows().await.iter().any(|(b, _, _)| *b == bid));
    // The guest's own switcher pick, as before.
    let resp = e
        .app()
        .oneshot(guest("14:00", Some("; calrs_lang=pl")))
        .await
        .unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let bid = newest_booking(&e).await;
    assert_eq!(e.booking_col(&bid, "language").await.as_deref(), Some("pl"));
}

#[tokio::test]
async fn without_a_linked_chat_the_page_says_so_at_the_top() {
    let _tg = fake::install_fake("ilia_test_bot").await;
    let _lock = crate::test_support::ENV_LOCK.lock().await;
    let _guard = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", BASE);
    let e = env_on().await;
    let admin = e.session().await;
    let (_, page) = e.get_as(PATH, &admin).await;
    let notice = page
        .split(r#"id="debug-booking-no-chat""#)
        .nth(1)
        .expect("the notice")
        .split("</div>")
        .next()
        .unwrap();
    assert!(notice.contains(&crate::i18n::translate("en", "dbg-no-chat-title", None)));
    assert!(
        notice.contains(r#"href="/dashboard/settings/telegram""#),
        "{notice}"
    );
    // Above the form.
    let at = |needle: &str| page.find(needle).unwrap();
    assert!(at(r#"id="debug-booking-no-chat""#) < at(r#"id="debug-booking-form""#));
    assert!(page.contains(r#"id="dbg-own-chat" disabled"#));

    // Linked: no notice.
    sqlx::query(
        "INSERT INTO telegram_user_chats (user_id, chat_id, telegram_username)
         SELECT id, ?, 'host_tg' FROM users WHERE username = 'testuser'",
    )
    .bind(HOST_CHAT)
    .execute(&e.pool)
    .await
    .unwrap();
    let (_, page) = e.get_as(PATH, &admin).await;
    assert!(!page.contains(r#"id="debug-booking-no-chat""#));
}
