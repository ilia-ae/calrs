//! Telegram notifications through the web layer (ilia fork): the booking
//! form field and deep link, linking a chat with `/start`, the lifecycle
//! messages, host linking from Profile & Settings, `/stop`, and the bookings
//! dashboard. The bot is `crate::telegram::fake::FakeApi`: nothing reaches
//! Telegram.

use super::*;
use crate::telegram::api::Outgoing;
use crate::telegram::commands::handle_update;
use crate::telegram::fake::{self, Installed};

const BASE: &str = "https://book.example.test";
const GUEST_CHAT: i64 = 987654321;
const HOST_CHAT: i64 = 555000111;

/// Telegram on with a fake bot, and `CALRS_BASE_URL` pointing at an HTTPS
/// site so messages carry buttons. Hold both for the whole test.
async fn telegram_on() -> (
    Installed,
    tokio::sync::MutexGuard<'static, ()>,
    crate::test_support::EnvGuard,
) {
    let tg = fake::install_fake("ilia_test_bot").await;
    let lock = crate::test_support::ENV_LOCK.lock().await;
    let env = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", BASE);
    (tg, lock, env)
}

fn post_book(csrf: &str, body: &str, accept_language: &str) -> axum::http::Request<Body> {
    axum::http::Request::builder()
        .method("POST")
        .uri("/u/testuser/test-meeting/book")
        .header("cookie", format!("__Host-calrs_csrf={csrf}"))
        .header("accept-language", accept_language)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// The booking form for the slot `book_at` posts.
fn form_uri() -> String {
    format!(
        "/u/testuser/test-meeting/book?date={}&time=10:00",
        far_future_monday()
    )
}

fn book_at(csrf: &str, time: &str, telegram: Option<&str>) -> String {
    let mut body = format!(
        "_csrf={csrf}&date={}&time={}&name=Guest+One&email=guest%40test.com&tz=UTC",
        far_future_monday(),
        time.replace(':', "%3A")
    );
    if let Some(t) = telegram {
        body.push_str(&format!("&telegram={}", urlencoding::encode(t)));
    }
    body
}

/// The `b_`/`u_` token of the first deep link in a page.
fn token_in(page: &str, kind: &str) -> String {
    let marker = format!("https://t.me/ilia_test_bot?start={kind}");
    let at = page
        .find(&marker)
        .unwrap_or_else(|| panic!("no {marker} in the page"));
    let start = at + marker.len() - 2;
    page[start..start + 45].to_string()
}

async fn set_mode(pool: &SqlitePool, et_id: &str, mode: &str) {
    crate::telegram::store::save_event_type_mode(pool, et_id, mode)
        .await
        .unwrap();
}

async fn link_count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM telegram_guest_links")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn start(tg: &Installed, pool: &SqlitePool, chat: i64, username: Option<&str>, token: &str) {
    let update = fake::private_text(1, chat, username, &format!("/start {token}"));
    handle_update(&tg.rt, pool, &update).await;
    tg.flush(pool).await;
}

/// What each chat has been sent since the last look: every lifecycle step
/// is checked for exactly the messages it should cause, so a duplicate
/// notification fails as surely as a missing one.
#[derive(Default)]
struct Inbox {
    seen: std::collections::HashMap<i64, usize>,
}

impl Inbox {
    /// The texts sent to `chat` since the last call for it.
    fn take(&mut self, tg: &Installed, chat: i64) -> Vec<String> {
        let all = texts(tg, chat);
        let seen = self.seen.entry(chat).or_default();
        let new = all[*seen..].to_vec();
        *seen = all.len();
        new
    }

    /// Exactly one new message to `chat`, starting with `prefix`.
    fn one(&mut self, tg: &Installed, chat: i64, prefix: &str) -> String {
        let got = self.take(tg, chat);
        assert_eq!(got.len(), 1, "to {chat}, want one {prefix:?}: {got:#?}");
        assert!(
            got[0].starts_with(prefix),
            "to {chat}, want {prefix:?}: {got:#?}"
        );
        got[0].clone()
    }

    /// Nothing new to `chat`.
    fn none(&mut self, tg: &Installed, chat: i64) {
        let got = self.take(tg, chat);
        assert!(got.is_empty(), "to {chat}, want nothing: {got:#?}");
    }
}

fn texts(tg: &Installed, chat: i64) -> Vec<String> {
    tg.sent()
        .into_iter()
        .filter(|(c, _)| *c == chat)
        .map(|(_, m)| m.text)
        .collect()
}

fn last_to(tg: &Installed, chat: i64) -> Outgoing {
    tg.sent()
        .into_iter()
        .filter(|(c, _)| *c == chat)
        .map(|(_, m)| m)
        .next_back()
        .unwrap_or_else(|| panic!("nothing sent to {chat}: {:#?}", tg.sent()))
}

fn en(key: &str) -> String {
    crate::i18n::translate("en", key, None)
}

/// The seeded admin's linked host chat.
async fn link_host(pool: &SqlitePool) {
    sqlx::query(
        "INSERT INTO telegram_user_chats (user_id, chat_id, telegram_username)
         SELECT id, ?, 'host_tg' FROM users WHERE email = 'test@example.com'",
    )
    .bind(HOST_CHAT)
    .execute(pool)
    .await
    .unwrap();
}

/// A version 1 booking `hours` from now with a deep link; returns
/// (booking id, token).
async fn booking_with_link(
    tg: &Installed,
    pool: &SqlitePool,
    et_id: &str,
    hours: i64,
    status: &str,
    entered: Option<&str>,
) -> (String, String) {
    let id = insert_v1_booking(
        pool,
        et_id,
        "guest@test.com",
        Utc::now() + Duration::hours(hours),
        30,
        status,
        &[("language", Some("en".into()))],
    )
    .await;
    let link = crate::telegram::guest_link(&tg.rt, pool, &id, entered)
        .await
        .expect("a deep link");
    let token = link.rsplit('=').next().unwrap().to_string();
    (id, token)
}

#[tokio::test]
async fn telegram_off_by_mode_shows_no_field_and_no_link() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, _, _) = setup_test_app().await;
    let page = body_string(app.clone().oneshot(get(&form_uri())).await.unwrap()).await;
    assert!(!page.contains("name=\"telegram\""));
    let csrf = "tg-off";
    let response = app
        .oneshot(post_book(
            csrf,
            &book_at(csrf, "10:00", Some("@Guest_Name")),
            "en",
        ))
        .await
        .unwrap();
    let page = body_string(response).await;
    assert!(page.contains("booked!"), "{page}");
    assert!(!page.contains("t.me/"), "{page}");
    assert_eq!(link_count(&pool).await, 0);
    drop(tg);
}

#[tokio::test]
async fn optional_mode_offers_the_link_and_keeps_only_a_hash() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "optional").await;
    let page = body_string(app.clone().oneshot(get(&form_uri())).await.unwrap()).await;
    assert!(page.contains("name=\"telegram\""), "{page}");
    let input = &page[page.find("id=\"telegram\"").unwrap()..];
    let input = &input[..input.find('>').unwrap()];
    assert!(!input.contains("required"), "{input}");

    let csrf = "tg-opt";
    let response = app
        .clone()
        .oneshot(post_book(csrf, &book_at(csrf, "10:00", None), "en"))
        .await
        .unwrap();
    let page = body_string(response).await;
    let first = token_in(&page, "b_");
    assert!(page.contains(&en("tg-confirmed-title")), "{page}");

    let response = app
        .oneshot(post_book(
            csrf,
            &book_at(csrf, "11:00", Some("@Guest_Name")),
            "en",
        ))
        .await
        .unwrap();
    let second = token_in(&body_string(response).await, "b_");
    let rows: Vec<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT entered_username, link_token_hash FROM telegram_guest_links ORDER BY created_at, rowid",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, None);
    assert_eq!(rows[1].0.as_deref(), Some("Guest_Name"));
    let hashes: Vec<String> = rows.into_iter().filter_map(|r| r.1).collect();
    assert_eq!(
        hashes,
        [
            crate::telegram::tokens::hash(&first),
            crate::telegram::tokens::hash(&second)
        ]
    );
    // The plain token is nowhere in the database.
    let dump: String = sqlx::query_scalar(
        "SELECT group_concat(quote(entered_username) || quote(link_token_hash) || quote(link_expires_at))
           FROM telegram_guest_links",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!dump.contains(&first[2..]) && !dump.contains(&second[2..]));
    drop(tg);
}

#[tokio::test]
async fn required_mode_refuses_a_missing_or_bad_username_in_the_page_language() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "required").await;
    let page = body_string(app.clone().oneshot(get(&form_uri())).await.unwrap()).await;
    assert!(page.contains("name=\"telegram\""));
    let csrf = "tg-req";
    let response = app
        .clone()
        .oneshot(post_book(csrf, &book_at(csrf, "10:00", Some("")), "en"))
        .await
        .unwrap();
    let page = body_string(response).await;
    assert!(page.contains(&en("tg-book-required")), "{page}");
    let response = app
        .oneshot(post_book(
            csrf,
            &book_at(csrf, "10:00", Some("no way")),
            "ru",
        ))
        .await
        .unwrap();
    let page = body_string(response).await;
    assert!(
        page.contains(&crate::i18n::translate("ru", "tg-book-invalid", None)),
        "{page}"
    );
    let bookings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(bookings, 0);
    assert_eq!(link_count(&pool).await, 0);
    drop(tg);
}

#[tokio::test]
async fn start_links_the_chat_and_the_host_sees_who() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "optional").await;
    link_host(&pool).await;
    let csrf = "tg-link";
    let response = app
        .clone()
        .oneshot(post_book(
            csrf,
            &book_at(csrf, "10:00", Some("@Guest_Name")),
            "ru-RU,ru;q=0.9",
        ))
        .await
        .unwrap();
    let token = token_in(&body_string(response).await, "b_");
    tg.flush(&pool).await;
    // The host heard about the booking at once.
    assert!(
        texts(&tg, HOST_CHAT)
            .iter()
            .any(|t| t.starts_with("New booking: Test Meeting")),
        "{:#?}",
        tg.sent()
    );

    start(&tg, &pool, GUEST_CHAT, Some("RealName"), &token).await;
    let (booking_id, cancel_token, language): (String, String, String) =
        sqlx::query_as("SELECT id, cancel_token, language FROM bookings")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(language, "ru");
    let (chat, username, token_chat): (Option<i64>, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT chat_id, telegram_username, token_chat_id FROM telegram_guest_links WHERE booking_id = ?",
    )
    .bind(&booking_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(chat, Some(GUEST_CHAT));
    assert_eq!(username.as_deref(), Some("RealName"));
    // The token now belongs to this chat (and only it may link again).
    assert_eq!(token_chat, Some(GUEST_CHAT));

    let welcome = last_to(&tg, GUEST_CHAT);
    assert!(
        welcome
            .text
            .starts_with("Готово: уведомления об этой записи будут приходить сюда.\n\n"),
        "{}",
        welcome.text
    );
    assert!(
        welcome.text.contains("Запись подтверждена"),
        "{}",
        welcome.text
    );
    let cancel = welcome
        .buttons
        .iter()
        .find(|b| b.label == "Отменить запись")
        .expect("a cancel button");
    assert_eq!(cancel.url, format!("{BASE}/booking/cancel/{cancel_token}"));

    let linked = last_to(&tg, HOST_CHAT);
    assert!(linked.text.contains("@RealName"), "{}", linked.text);
    assert!(
        linked.text.contains("Entered on the form: @Guest_Name."),
        "{}",
        linked.text
    );
    assert!(!linked.text.contains("/booking/"), "{}", linked.text);
    for b in &linked.buttons {
        assert_eq!(b.url, format!("{BASE}/dashboard/bookings"));
    }

    // The same token again, from anyone else: no.
    start(&tg, &pool, 1234567, None, &token).await;
    assert_eq!(last_to(&tg, 1234567).text, en("tg-bot-link-invalid"));
    drop(tg);
}

/// The guest taps the link on the confirmation page, later the one in the
/// email, then turns notifications off and on again: the same link serves,
/// in the same chat only, and a second tap is not a failure.
#[tokio::test]
async fn the_same_link_reopened_or_after_stop_works_in_its_own_chat() {
    let (tg, _lock, _env) = telegram_on().await;
    let (_app, pool, _, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "optional").await;
    link_host(&pool).await;
    let (id, token) = booking_with_link(&tg, &pool, &et_id, 48, "confirmed", None).await;
    let mut inbox = Inbox::default();
    start(&tg, &pool, GUEST_CHAT, Some("RealName"), &token).await;
    inbox.one(&tg, GUEST_CHAT, "Done: notifications about this booking");
    inbox.one(&tg, HOST_CHAT, "Guest will get Telegram notifications");

    // Reopened six times: "already on" once, never a failure, never the limit.
    for _ in 0..6 {
        start(&tg, &pool, GUEST_CHAT, Some("RealName"), &token).await;
    }
    assert_eq!(inbox.take(&tg, GUEST_CHAT), [en("tg-bot-already-linked")]);
    inbox.none(&tg, HOST_CHAT);
    assert_eq!(tg.rt.limits.failures.count(&GUEST_CHAT), 0);

    // /stop, then the same link turns it back on.
    let update = fake::private_text(2, GUEST_CHAT, None, "/stop");
    handle_update(&tg.rt, &pool, &update).await;
    tg.flush(&pool).await;
    assert_eq!(inbox.take(&tg, GUEST_CHAT), [en("tg-bot-stopped")]);
    assert!(crate::telegram::store::linked_guest_chat(&pool, &id)
        .await
        .is_none());
    // Another chat holding the link still cannot take it over.
    start(&tg, &pool, GUEST_CHAT + 9, None, &token).await;
    assert_eq!(inbox.take(&tg, GUEST_CHAT + 9), [en("tg-bot-link-invalid")]);
    start(&tg, &pool, GUEST_CHAT, Some("RealName"), &token).await;
    inbox.one(&tg, GUEST_CHAT, "Done: notifications about this booking");
    inbox.one(&tg, HOST_CHAT, "Guest will get Telegram notifications");
    assert_eq!(
        crate::telegram::store::linked_guest_chat(&pool, &id).await,
        Some(GUEST_CHAT)
    );

    // The same after the bot was blocked (403 unlinks), then unblocked.
    tg.api
        .script_send(vec![Err(crate::telegram::api::TgError::Forbidden(
            "Forbidden: bot was blocked by the user".into(),
        ))]);
    crate::telegram::notify(&pool, &id, crate::telegram::TelegramEvent::Reminder);
    tg.flush(&pool).await;
    inbox.none(&tg, GUEST_CHAT);
    assert!(crate::telegram::store::linked_guest_chat(&pool, &id)
        .await
        .is_none());
    start(&tg, &pool, GUEST_CHAT, None, &token).await;
    inbox.one(&tg, GUEST_CHAT, "Done: notifications about this booking");
    drop(tg);
}

#[tokio::test]
async fn tokens_of_ended_or_cancelled_bookings_are_refused() {
    let (tg, _lock, _env) = telegram_on().await;
    let (_app, pool, _, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "optional").await;
    let (cancelled, token) = booking_with_link(&tg, &pool, &et_id, 48, "confirmed", None).await;
    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
        .bind(&cancelled)
        .execute(&pool)
        .await
        .unwrap();
    start(&tg, &pool, GUEST_CHAT, None, &token).await;
    assert_eq!(last_to(&tg, GUEST_CHAT).text, en("tg-bot-link-invalid"));

    let (expired, token) = booking_with_link(&tg, &pool, &et_id, 72, "confirmed", None).await;
    sqlx::query("UPDATE telegram_guest_links SET link_expires_at = '2000-01-01 00:00:00' WHERE booking_id = ?")
        .bind(&expired)
        .execute(&pool)
        .await
        .unwrap();
    start(&tg, &pool, GUEST_CHAT + 1, None, &token).await;
    assert_eq!(last_to(&tg, GUEST_CHAT + 1).text, en("tg-bot-link-invalid"));
    let linked: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM telegram_guest_links WHERE chat_id IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(linked, 0);
    drop(tg);
}

/// Link a fresh chat to a fresh booking; the welcome and the host's "guest
/// linked" are taken from the inbox, so the step under test starts clean.
async fn linked_booking(
    tg: &Installed,
    pool: &SqlitePool,
    inbox: &mut Inbox,
    et_id: &str,
    hours: i64,
    status: &str,
    chat: i64,
) -> String {
    let (id, token) = booking_with_link(tg, pool, et_id, hours, status, None).await;
    start(tg, pool, chat, None, &token).await;
    inbox.one(tg, chat, "Done: notifications about this booking");
    inbox.one(tg, HOST_CHAT, "Guest will get Telegram notifications");
    id
}

async fn column(pool: &SqlitePool, column: &str, id: &str) -> String {
    sqlx::query_scalar(&format!("SELECT {column} FROM bookings WHERE id = ?"))
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Every lifecycle hook, end to end through the web layer: each step sends
/// exactly one message to the guest's chat and exactly what the host should
/// get (often nothing) to the host's.
#[tokio::test]
async fn the_linked_guest_hears_about_every_change_exactly_once() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, session, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "optional").await;
    link_host(&pool).await;
    let mut inbox = Inbox::default();
    let csrf = "tg-life";

    // Host cancels from the dashboard, with a reason.
    let chat = GUEST_CHAT;
    let id = linked_booking(&tg, &pool, &mut inbox, &et_id, 48, "confirmed", chat).await;
    let response = app
        .clone()
        .oneshot(post_form(
            &format!("/dashboard/bookings/{id}/cancel"),
            &session,
            csrf,
            &format!("_csrf={csrf}&reason=Away+that+day"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    tg.flush(&pool).await;
    let msg = inbox.one(&tg, chat, "The host cancelled: Test Meeting");
    assert!(msg.ends_with("Reason: Away that day"), "{msg}");
    inbox.none(&tg, HOST_CHAT);

    // The guest cancels: they are told, and so is the host.
    let chat = GUEST_CHAT + 1;
    let id = linked_booking(&tg, &pool, &mut inbox, &et_id, 50, "confirmed", chat).await;
    let cancel_token = column(&pool, "cancel_token", &id).await;
    app.clone()
        .oneshot(post_form_unauthed(
            &format!("/booking/cancel/{cancel_token}"),
            csrf,
            &format!("_csrf={csrf}&reason=Overlap"),
        ))
        .await
        .unwrap();
    tg.flush(&pool).await;
    inbox.one(&tg, chat, "You cancelled your booking: Test Meeting");
    let host = inbox.one(&tg, HOST_CHAT, "Guest cancelled: Test Meeting");
    assert!(host.ends_with("Reason: Overlap"), "{host}");

    // A request approved by token.
    let chat = GUEST_CHAT + 2;
    let id = linked_booking(&tg, &pool, &mut inbox, &et_id, 52, "pending", chat).await;
    let confirm = column(&pool, "confirm_token", &id).await;
    app.clone()
        .oneshot(post_bare(&format!("/booking/approve/{confirm}")))
        .await
        .unwrap();
    tg.flush(&pool).await;
    assert_eq!(booking_status(&pool, &id).await, "confirmed");
    inbox.one(&tg, chat, "Your request is confirmed: Test Meeting");
    inbox.none(&tg, HOST_CHAT);

    // A request declined by token, with the host's reason.
    let chat = GUEST_CHAT + 3;
    let id = linked_booking(&tg, &pool, &mut inbox, &et_id, 54, "pending", chat).await;
    let confirm = column(&pool, "confirm_token", &id).await;
    app.clone()
        .oneshot(post_form_unauthed(
            &format!("/booking/decline/{confirm}"),
            csrf,
            &format!("_csrf={csrf}&reason=Fully+booked"),
        ))
        .await
        .unwrap();
    tg.flush(&pool).await;
    let msg = inbox.one(&tg, chat, "Your request was declined");
    assert!(msg.ends_with("Reason: Fully booked"), "{msg}");
    inbox.none(&tg, HOST_CHAT);

    // A request confirmed from the dashboard (the agent's path too).
    let chat = GUEST_CHAT + 4;
    let id = linked_booking(&tg, &pool, &mut inbox, &et_id, 56, "pending", chat).await;
    let response = app
        .clone()
        .oneshot(post_form(
            &format!("/dashboard/bookings/{id}/confirm"),
            &session,
            csrf,
            &format!("_csrf={csrf}"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    tg.flush(&pool).await;
    assert_eq!(booking_status(&pool, &id).await, "confirmed");
    inbox.one(&tg, chat, "Your request is confirmed: Test Meeting");
    inbox.none(&tg, HOST_CHAT);

    // A request declined from the dashboard.
    let chat = GUEST_CHAT + 5;
    let id = linked_booking(&tg, &pool, &mut inbox, &et_id, 58, "pending", chat).await;
    let response = app
        .clone()
        .oneshot(post_form(
            &format!("/dashboard/bookings/{id}/cancel"),
            &session,
            csrf,
            &format!("_csrf={csrf}&reason=No+room"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    tg.flush(&pool).await;
    assert_eq!(booking_status(&pool, &id).await, "declined");
    let msg = inbox.one(&tg, chat, "Your request was declined");
    assert!(msg.ends_with("Reason: No room"), "{msg}");
    inbox.none(&tg, HOST_CHAT);

    // The host asks the guest to pick a new time.
    let chat = GUEST_CHAT + 6;
    let id = linked_booking(&tg, &pool, &mut inbox, &et_id, 60, "confirmed", chat).await;
    let response = app
        .clone()
        .oneshot(post_form(
            &format!("/dashboard/bookings/{id}/reschedule"),
            &session,
            csrf,
            &format!("_csrf={csrf}"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    tg.flush(&pool).await;
    inbox.one(
        &tg,
        chat,
        "The host asks you to pick a new time for Test Meeting",
    );
    let pick = last_to(&tg, chat);
    let reschedule_token = column(&pool, "reschedule_token", &id).await;
    assert!(
        pick.buttons
            .iter()
            .any(|b| b.url == format!("{BASE}/booking/reschedule/{reschedule_token}")),
        "{pick:?}"
    );
    inbox.none(&tg, HOST_CHAT);

    // The guest moves a booking: both hear, and the link follows the meeting.
    let chat = GUEST_CHAT + 7;
    let id = linked_booking(&tg, &pool, &mut inbox, &et_id, 62, "confirmed", chat).await;
    let reschedule_token = column(&pool, "reschedule_token", &id).await;
    let response = app
        .clone()
        .oneshot(post_form_unauthed(
            &format!("/booking/reschedule/{reschedule_token}"),
            csrf,
            &format!(
                "_csrf={csrf}&date={}&time=10%3A00&tz=UTC",
                far_future_monday()
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    tg.flush(&pool).await;
    inbox.one(&tg, chat, "Booking moved: Test Meeting is now");
    inbox.one(&tg, HOST_CHAT, "Guest moved Test Meeting to");
    let expires: String =
        sqlx::query_scalar("SELECT link_expires_at FROM telegram_guest_links WHERE booking_id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(expires, format!("{} 10:30:00", far_future_monday()));

    // The guest moves a booking that needs approval: "requested" and
    // "approve a new time".
    sqlx::query("UPDATE event_types SET requires_confirmation = 1 WHERE id = ?")
        .bind(&et_id)
        .execute(&pool)
        .await
        .unwrap();
    let chat = GUEST_CHAT + 8;
    let id = linked_booking(&tg, &pool, &mut inbox, &et_id, 64, "confirmed", chat).await;
    let reschedule_token = column(&pool, "reschedule_token", &id).await;
    let response = app
        .clone()
        .oneshot(post_form_unauthed(
            &format!("/booking/reschedule/{reschedule_token}"),
            csrf,
            &format!(
                "_csrf={csrf}&date={}&time=11%3A00&tz=UTC",
                far_future_monday()
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    tg.flush(&pool).await;
    assert_eq!(booking_status(&pool, &id).await, "pending");
    inbox.one(&tg, chat, "New time requested: Test Meeting");
    inbox.one(&tg, HOST_CHAT, "Approve a new time: Test Meeting");

    // A request that expires (the reminder loop's sweep).
    let chat = GUEST_CHAT + 10;
    let id = linked_booking(&tg, &pool, &mut inbox, &et_id, 66, "pending", chat).await;
    sqlx::query("UPDATE bookings SET pending_expires_at = ? WHERE id = ?")
        .bind(
            (Utc::now() - Duration::minutes(1))
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string(),
        )
        .bind(&id)
        .execute(&pool)
        .await
        .unwrap();
    let sweep = sweep_pending_requests(&pool, &[0u8; 32], Utc::now()).await;
    assert_eq!(sweep.expired, 1);
    tg.flush(&pool).await;
    inbox.one(&tg, chat, "Request expired: Test Meeting");
    inbox.none(&tg, HOST_CHAT);
    // The ended link is refused from now on.
    let still: Option<String> =
        sqlx::query_scalar("SELECT link_token_hash FROM telegram_guest_links WHERE booking_id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(still, None);
    drop(tg);
}

#[tokio::test]
async fn the_reminder_loop_reminds_through_telegram_alone() {
    let (tg, _lock, _env) = telegram_on().await;
    let (_app, pool, _, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "optional").await;
    sqlx::query("UPDATE event_types SET reminder_minutes = 60 WHERE id = ?")
        .bind(&et_id)
        .execute(&pool)
        .await
        .unwrap();
    let id = insert_v1_booking(
        &pool,
        &et_id,
        "guest@test.com",
        Utc::now() + Duration::minutes(30),
        30,
        "confirmed",
        &[],
    )
    .await;
    let token = crate::telegram::guest_link(&tg.rt, &pool, &id, None)
        .await
        .unwrap()
        .rsplit('=')
        .next()
        .unwrap()
        .to_string();
    start(&tg, &pool, GUEST_CHAT, None, &token).await;
    // No SMTP and no SMS configured.
    let mut inbox = Inbox::default();
    inbox.take(&tg, GUEST_CHAT);
    send_due_reminders(pool.clone(), [0u8; 32]).await;
    send_due_reminders(pool.clone(), [0u8; 32]).await;
    tg.flush(&pool).await;
    inbox.one(&tg, GUEST_CHAT, "Reminder: Test Meeting starts in");
    let stamped: Option<String> =
        sqlx::query_scalar("SELECT reminder_sent_at FROM bookings WHERE id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(stamped.is_some());
    drop(tg);
}

#[tokio::test]
async fn hosts_link_their_own_chat_from_settings() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, session, _) = setup_test_app().await;
    let response = app
        .clone()
        .oneshot(get_authed("/dashboard/settings/telegram", &session))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let page = body_string(response).await;
    assert!(page.contains(&en("tg-settings-not-linked")), "{page}");
    let settings = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/settings", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(settings.contains("/dashboard/settings/telegram"));

    // No CSRF, no link.
    let response = app
        .clone()
        .oneshot(post_form(
            "/dashboard/settings/telegram/link",
            &session,
            "c",
            "",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 403);

    let csrf = "tg-host";
    let link = |app: Router| {
        let session = session.clone();
        async move {
            body_string(
                app.oneshot(post_form(
                    "/dashboard/settings/telegram/link",
                    &session,
                    csrf,
                    &format!("_csrf={csrf}"),
                ))
                .await
                .unwrap(),
            )
            .await
        }
    };
    let page = link(app.clone()).await;
    let token = token_in(&page, "u_");
    assert!(page.contains(&en("tg-settings-link-expires")));
    start(&tg, &pool, HOST_CHAT, Some("host_tg"), &token).await;
    assert_eq!(
        last_to(&tg, HOST_CHAT).text,
        "Done: host notifications for Test User will come here."
    );
    let chat: i64 = sqlx::query_scalar("SELECT chat_id FROM telegram_user_chats")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(chat, HOST_CHAT);
    // Once only.
    start(&tg, &pool, HOST_CHAT + 1, None, &token).await;
    assert_eq!(
        last_to(&tg, HOST_CHAT + 1).text,
        en("tg-bot-link-invalid-host")
    );
    // Not after 15 minutes.
    let token = token_in(&link(app.clone()).await, "u_");
    sqlx::query("UPDATE telegram_user_link_tokens SET expires_at = datetime('now', '-1 minute')")
        .execute(&pool)
        .await
        .unwrap();
    start(&tg, &pool, HOST_CHAT + 2, None, &token).await;
    assert_eq!(
        last_to(&tg, HOST_CHAT + 2).text,
        en("tg-bot-link-invalid-host")
    );

    let page = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/settings/telegram", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(
        page.contains("Host notifications go to @host_tg. Linked: "),
        "{page}"
    );
    assert!(!page.contains(&HOST_CHAT.to_string()), "{page}");

    // Unlink.
    let response = app
        .clone()
        .oneshot(post_form(
            "/dashboard/settings/telegram/unlink",
            &session,
            csrf,
            &format!("_csrf={csrf}"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_user_chats")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(left, 0);

    // Five links an hour: two so far, three more, then no.
    for _ in 0..3 {
        assert!(link(app.clone()).await.contains("?start=u_"));
    }
    let page = link(app.clone()).await;
    assert!(!page.contains("?start=u_"), "{page}");
    assert!(page.contains(&en("tg-settings-link-limited")), "{page}");
    drop(tg);
}

#[tokio::test]
async fn modes_are_saved_only_for_manageable_event_types() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, admin_session, et_id) = setup_test_app().await;
    let (_, member_session) =
        insert_test_user_with_session(&pool, "Member", "member@example.com", "member").await;
    let csrf = "tg-modes";
    let response = app
        .clone()
        .oneshot(post_form(
            "/dashboard/settings/telegram/modes",
            &member_session,
            csrf,
            &format!("_csrf={csrf}&mode_{et_id}=required"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 403);
    assert_eq!(
        crate::telegram::store::event_type_mode(&pool, &et_id).await,
        "off"
    );
    let page = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/settings/telegram", &member_session))
            .await
            .unwrap(),
    )
    .await;
    assert!(!page.contains(&format!("mode_{et_id}")), "{page}");

    let response = app
        .clone()
        .oneshot(post_form(
            "/dashboard/settings/telegram/modes",
            &admin_session,
            csrf,
            &format!("_csrf={csrf}&mode_{et_id}=required"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    assert_eq!(
        crate::telegram::store::event_type_mode(&pool, &et_id).await,
        "required"
    );
    let page = body_string(
        app.oneshot(get_authed(
            "/dashboard/settings/telegram?saved=1",
            &admin_session,
        ))
        .await
        .unwrap(),
    )
    .await;
    assert!(page.contains(&en("tg-settings-saved")), "{page}");
    assert!(page.contains(&format!("name=\"mode_{et_id}\"")), "{page}");
    drop(tg);
}

#[tokio::test]
async fn stop_unlinks_guest_and_host_chats() {
    let (tg, _lock, _env) = telegram_on().await;
    let (_app, pool, _, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "optional").await;
    let (id, token) = booking_with_link(&tg, &pool, &et_id, 48, "confirmed", None).await;
    start(&tg, &pool, GUEST_CHAT, None, &token).await;
    sqlx::query(
        "INSERT INTO telegram_user_chats (user_id, chat_id)
         SELECT id, ? FROM users WHERE email = 'test@example.com'",
    )
    .bind(GUEST_CHAT)
    .execute(&pool)
    .await
    .unwrap();
    let update = fake::private_text(2, GUEST_CHAT, None, "/stop");
    handle_update(&tg.rt, &pool, &update).await;
    tg.flush(&pool).await;
    assert_eq!(last_to(&tg, GUEST_CHAT).text, en("tg-bot-stopped"));
    assert!(crate::telegram::store::linked_guest_chat(&pool, &id)
        .await
        .is_none());
    let hosts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_user_chats")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(hosts, 0);
    drop(tg);
}

#[tokio::test]
async fn the_bookings_dashboard_shows_the_linked_username_not_the_chat() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, session, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "optional").await;
    let (_, token) =
        booking_with_link(&tg, &pool, &et_id, 48, "confirmed", Some("Guest_Name")).await;
    start(&tg, &pool, GUEST_CHAT, Some("RealName"), &token).await;
    booking_with_link(&tg, &pool, &et_id, 26, "pending", Some("Waiting_One")).await;
    let page = body_string(
        app.oneshot(get_authed("/dashboard/bookings", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(
        page.contains("Telegram @RealName (entered @Guest_Name)"),
        "{page}"
    );
    assert!(
        page.contains("Telegram @Waiting_One (not linked yet)"),
        "{page}"
    );
    assert!(!page.contains(&GUEST_CHAT.to_string()), "{page}");
    drop(tg);
}

#[tokio::test]
async fn the_confirmation_and_pending_emails_carry_the_link() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "optional").await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, port).await;
    let csrf = "tg-mail";
    let response = app
        .clone()
        .oneshot(post_book(csrf, &book_at(csrf, "10:00", None), "en"))
        .await
        .unwrap();
    let token = token_in(&body_string(response).await, "b_");
    let mail = mail_to(&mails, "guest@test.com").await;
    let plain = mail.split_whitespace().collect::<String>();
    assert!(
        plain.contains(&format!("https://t.me/ilia_test_bot?start={token}")),
        "{mail}"
    );
    assert!(mail.contains(&en("tg-email-button")), "{mail}");
    // The Telegram button comes first, away from the red Cancel.
    let telegram_at = mail.find(&en("tg-email-button")).unwrap();
    let cancel_at = mail
        .find(&en("email-action-cancel-booking"))
        .expect("a cancel button");
    assert!(telegram_at < cancel_at, "{mail}");

    // A request: the pending notice has it too.
    sqlx::query("UPDATE event_types SET requires_confirmation = 1 WHERE id = ?")
        .bind(&et_id)
        .execute(&pool)
        .await
        .unwrap();
    let response = app
        .oneshot(post_book(
            csrf,
            &book_at(csrf, "11:00", None).replace("guest%40test.com", "second%40test.com"),
            "en",
        ))
        .await
        .unwrap();
    let token = token_in(&body_string(response).await, "b_");
    let mail = mail_to(&mails, "second@test.com").await;
    let plain = mail.split_whitespace().collect::<String>();
    assert!(
        plain.contains(&format!("https://t.me/ilia_test_bot?start={token}")),
        "{mail}"
    );
    drop(tg);
}

#[tokio::test]
async fn a_host_chat_without_username_reads_as_a_sentence_with_a_local_date() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, session, _) = setup_test_app().await;
    sqlx::query("UPDATE users SET timezone = 'Europe/Moscow' WHERE email = 'test@example.com'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO telegram_user_chats (user_id, chat_id, telegram_username, linked_at)
         SELECT id, ?, NULL, '2026-11-09 14:03:00' FROM users WHERE email = 'test@example.com'",
    )
    .bind(HOST_CHAT)
    .execute(&pool)
    .await
    .unwrap();
    let page = body_string(
        app.oneshot(get_authed("/dashboard/settings/telegram", &session))
            .await
            .unwrap(),
    )
    .await
    .replace("&#x2f;", "/");
    let when = crate::telegram::message::when(
        "en",
        chrono::NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
        "17:03",
        "Europe/Moscow",
    );
    let want = format!("Host notifications go to your Telegram chat. Linked: {when}.");
    assert!(page.contains(&want), "{want}\n{page}");
    assert!(!page.contains("go to no username"), "{page}");
    drop(tg);
}

#[tokio::test]
async fn a_duplicated_event_type_keeps_its_telegram_field() {
    let (tg, _lock, _env) = telegram_on().await;
    let (app, pool, session, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "required").await;
    let csrf = "tg-dup";
    let response = app
        .oneshot(post_form(
            "/dashboard/event-types/test-meeting/duplicate",
            &session,
            csrf,
            &format!("_csrf={csrf}"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    let copy: String =
        sqlx::query_scalar("SELECT id FROM event_types WHERE slug = 'test-meeting-copy'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        crate::telegram::store::event_type_mode(&pool, &copy).await,
        "required"
    );
    drop(tg);
}

/// A legacy row (`time_version` 0) holds a wall clock in the event type's
/// zone and may end in a stray `Z`: read as upstream reads it, not as UTC.
#[tokio::test]
async fn legacy_booking_times_are_read_in_the_event_timezone() {
    let (tg, _lock, _env) = telegram_on().await;
    let (_app, pool, _, et_id) = setup_test_app().await;
    set_mode(&pool, &et_id, "optional").await;
    sqlx::query("UPDATE event_types SET timezone = 'Europe/Moscow' WHERE id = ?")
        .bind(&et_id)
        .execute(&pool)
        .await
        .unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO bookings (time_version, id, event_type_id, uid, guest_name, guest_email,
                               guest_timezone, start_at, end_at, status, cancel_token, reschedule_token)
         VALUES (0, ?, ?, 'uid-legacy-z', 'Guest', 'guest@test.com', 'UTC',
                 '2030-06-15T10:00:00Z', '2030-06-15T10:30:00Z', 'confirmed', 'ct-legacy', 'rt-legacy')",
    )
    .bind(&id)
    .bind(&et_id)
    .execute(&pool)
    .await
    .unwrap();
    let row = crate::telegram::store::booking_row(&pool, &id)
        .await
        .unwrap();
    assert_eq!(row.start.to_rfc3339(), "2030-06-15T07:00:00+00:00");
    assert_eq!(row.end.to_rfc3339(), "2030-06-15T07:30:00+00:00");
    let (_, end) = crate::telegram::store::booking_mode_and_end(&pool, &id)
        .await
        .unwrap();
    assert_eq!(end.to_rfc3339(), "2030-06-15T07:30:00+00:00");
    drop(tg);
}

/// With Telegram switched off, what it stored is still forgotten on time:
/// `spawn` runs the purge without a token.
#[tokio::test]
async fn links_are_purged_even_with_telegram_off() {
    let _tg_lock = crate::telegram::fake::TELEGRAM_LOCK.lock().await;
    let _lock = crate::test_support::ENV_LOCK.lock().await;
    let _env = crate::test_support::EnvGuard::new();
    std::env::remove_var(crate::telegram::config::TOKEN_ENV);
    let (_app, pool, _, et_id) = setup_test_app().await;
    let old = insert_v1_booking(
        &pool,
        &et_id,
        "old@test.com",
        Utc::now() - Duration::days(9),
        30,
        "confirmed",
        &[],
    )
    .await;
    let cancelled = insert_v1_booking(
        &pool,
        &et_id,
        "gone@test.com",
        Utc::now() + Duration::days(9),
        30,
        "cancelled",
        &[],
    )
    .await;
    for (id, expires) in [
        (
            &old,
            (Utc::now() - Duration::days(9))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        ),
        (&cancelled, "2999-01-01 00:00:00".to_string()),
    ] {
        sqlx::query(
            "INSERT INTO telegram_guest_links (booking_id, link_token_hash, link_expires_at, chat_id, token_chat_id)
             VALUES (?, ?, ?, 4242, 4242)",
        )
        .bind(id)
        .bind(format!("hash-{id}"))
        .bind(expires)
        .execute(&pool)
        .await
        .unwrap();
    }
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let handle = crate::telegram::spawn(pool.clone(), stop_rx).await;
    assert!(!crate::telegram::enabled());
    let mut left: Vec<(String, Option<String>)> = Vec::new();
    for _ in 0..200 {
        left = sqlx::query_as("SELECT booking_id, link_token_hash FROM telegram_guest_links")
            .fetch_all(&pool)
            .await
            .unwrap();
        if left.len() == 1 && left[0].1.is_none() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    // The old link is gone; the cancelled booking's (never ended while
    // Telegram was off) is ended now, so it goes a day later.
    assert_eq!(left, [(cancelled.clone(), None)]);
    crate::telegram::store::purge(&pool, Utc::now() + Duration::days(2)).await;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_guest_links")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    stop_tx.send(true).unwrap();
    let started = std::time::Instant::now();
    handle.finish(std::time::Duration::from_secs(5)).await;
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
}

/// With Telegram off, a host can still make calrs forget the chat they
/// linked while it was on.
#[tokio::test]
async fn a_host_can_unlink_while_telegram_is_off() {
    let _tg_lock = crate::telegram::fake::TELEGRAM_LOCK.lock().await;
    assert!(!crate::telegram::enabled());
    let (app, pool, session, _) = setup_test_app().await;
    link_host(&pool).await;
    let page = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/settings/telegram", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains(&en("tg-settings-off")), "{page}");
    assert!(
        page.contains("/dashboard/settings/telegram/unlink"),
        "{page}"
    );
    let csrf = "tg-off-unlink";
    let response = app
        .oneshot(post_form(
            "/dashboard/settings/telegram/unlink",
            &session,
            csrf,
            &format!("_csrf={csrf}"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_user_chats")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(left, 0);
}
