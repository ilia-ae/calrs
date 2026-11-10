//! ilia fork: host <-> guest messages (`web::messages`, `telegram::relay`):
//! the dashboard thread, email with Reply-To the host, the guest's
//! Telegram chat, guest texts relayed to the host's chat and the host's
//! replies, the limits and what is never logged. The bot is
//! `crate::telegram::fake::FakeApi`: nothing reaches Telegram.

use super::*;
use crate::telegram::commands::handle_update;
use crate::telegram::fake::{self, Installed};

const GUEST_CHAT: i64 = 987650001;
const HOST_CHAT: i64 = 555000222;
const CSRF: &str = "msg-csrf";

/// A version 1 booking `hours` from now, in English, for guest@test.com.
async fn booking(pool: &SqlitePool, et_id: &str, hours: i64, status: &str) -> String {
    insert_v1_booking(
        pool,
        et_id,
        "guest@test.com",
        Utc::now() + Duration::hours(hours),
        30,
        status,
        &[("language", Some("en".into()))],
    )
    .await
}

/// Link `chat` to the booking as its guest's chat.
async fn link_guest(pool: &SqlitePool, booking_id: &str, chat: i64) {
    let end: String = sqlx::query_scalar("SELECT end_at FROM bookings WHERE id = ?")
        .bind(booking_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let end = chrono::DateTime::parse_from_rfc3339(&end)
        .unwrap()
        .with_timezone(&Utc);
    sqlx::query(
        "INSERT INTO telegram_guest_links (booking_id, link_expires_at, chat_id, token_chat_id,
                                           telegram_username, linked_at)
         VALUES (?, ?, ?, ?, 'guest_tg', datetime('now'))",
    )
    .bind(booking_id)
    .bind(crate::telegram::store::sql_time(end))
    .bind(chat)
    .bind(chat)
    .execute(pool)
    .await
    .unwrap();
}

/// The seeded admin's host chat.
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

fn post_message(booking_id: &str, session: &str, text: &str) -> axum::http::Request<Body> {
    post_form(
        &format!("/dashboard/bookings/{booking_id}/messages"),
        session,
        CSRF,
        &format!("_csrf={CSRF}&text={}", urlencoding::encode(text)),
    )
}

fn location(response: &axum::http::Response<Body>) -> String {
    response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

fn texts(tg: &Installed, chat: i64) -> Vec<String> {
    tg.sent()
        .into_iter()
        .filter(|(c, _)| *c == chat)
        .map(|(_, m)| m.text)
        .collect()
}

/// A text from `chat`, optionally replying to bot message `reply_to`.
async fn write(
    tg: &Installed,
    pool: &SqlitePool,
    update: i64,
    chat: i64,
    text: &str,
    reply_to: Option<i64>,
) {
    let update = fake::private_message(update, chat, text, reply_to);
    handle_update(&tg.rt, pool, &update).await;
    tg.flush(pool).await;
}

async fn thread_rows(
    pool: &SqlitePool,
    booking_id: &str,
) -> Vec<(String, String, String, String, String)> {
    sqlx::query_as(
        "SELECT direction, via, body, email_status, telegram_status FROM booking_messages
          WHERE booking_id = ? ORDER BY created_at, rowid",
    )
    .bind(booking_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// An open TON invoice for a booking: it awaits its payment.
async fn open_invoice(pool: &SqlitePool, booking_id: &str, et_id: &str) {
    let now = Utc::now();
    let stamp = |t: chrono::DateTime<Utc>| crate::web::ton_pay::stamp(t);
    sqlx::query(
        "INSERT INTO ton_invoices (id, booking_id, event_type_id, pay_token, reference, network,
             address_raw, event_title, start_at_utc, guest_email_norm, client_tag,
             price_usd_cents, rate_usd_micro, rate_sources, rate_at, amount_nano, paid_nano,
             status, created_at, expires_at, settle_by, hard_deadline)
         VALUES (?, ?, ?, ?, 'BREF000001', 'mainnet', '0:83DF', 'Test Meeting', ?,
                 'guest@test.com', 'tag', 0, 1, '', ?, 100000000000, 0, 'open', ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(booking_id)
    .bind(et_id)
    .bind(format!("paytoken-{booking_id}"))
    .bind(stamp(now + Duration::days(2)))
    .bind(stamp(now))
    .bind(stamp(now))
    .bind(stamp(now + Duration::minutes(30)))
    .bind(stamp(now + Duration::minutes(45)))
    .bind(stamp(now + Duration::hours(6)))
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn a_host_message_reaches_the_guest_by_email_and_telegram() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (app, pool, session, et_id) = setup_test_app().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, port).await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    link_guest(&pool, &id, GUEST_CHAT).await;

    let page = body_string(
        app.clone()
            .oneshot(get_authed(
                &format!("/dashboard/bookings/{id}/messages"),
                &session,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains("No messages yet."), "{page}");
    assert!(page.contains("The guest has linked Telegram"), "{page}");

    let response = app
        .clone()
        .oneshot(post_message(
            &id,
            &session,
            "Hello Guest,\r\nsee you <soon>.",
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    assert!(
        location(&response).ends_with("?sent=1"),
        "{}",
        location(&response)
    );
    tg.flush(&pool).await;

    // Telegram: the guest's own chat, the text with its line break.
    let to_guest = texts(&tg, GUEST_CHAT);
    assert_eq!(to_guest.len(), 1, "{to_guest:#?}");
    assert!(
        to_guest[0].starts_with("Message from Test User about Test Meeting, "),
        "{}",
        to_guest[0]
    );
    assert!(to_guest[0]
        .contains("\n\nHello Guest,\nsee you <soon>.\n\nTo answer, write your reply here."));
    // Mapped, so the guest's reply picks this booking.
    let mapped: (String, String) = sqlx::query_as(
        "SELECT booking_id, side FROM telegram_message_map WHERE chat_id = ? AND message_id = ?",
    )
    .bind(GUEST_CHAT)
    .bind(tg.api.last_id_to(GUEST_CHAT))
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(mapped, (id.clone(), "guest".to_string()));

    // Email: from the host, replies to the host.
    let mail = mail_to(&mails, "guest@test.com").await;
    assert_eq!(
        mail_header(&mail, "Reply-To"),
        Some("test@example.com"),
        "{mail}"
    );
    assert!(
        mail_header(&mail, "From").unwrap().contains("Test User"),
        "{mail}"
    );
    assert!(
        mail.contains("Hello Guest,") && mail.contains("see you <soon>."),
        "{mail}"
    );
    drain_emails(std::time::Duration::from_secs(10)).await;

    let rows = thread_rows(&pool, &id).await;
    assert_eq!(
        rows,
        [(
            "host".to_string(),
            "dashboard".to_string(),
            "Hello Guest,\nsee you <soon>.".to_string(),
            "sent".to_string(),
            "sent".to_string(),
        )]
    );
    let author: Option<String> =
        sqlx::query_scalar("SELECT author_user_id FROM booking_messages WHERE booking_id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(author.is_some());

    // The thread, escaped, with what each channel did; the bookings list
    // counts it.
    let page = body_string(
        app.clone()
            .oneshot(get_authed(
                &format!("/dashboard/bookings/{id}/messages?sent=1"),
                &session,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains("Message sent."), "{page}");
    assert!(page.contains("see you &lt;soon&gt;."), "{page}");
    assert!(!page.contains("<soon>"), "{page}");
    assert!(
        page.contains("email sent") && page.contains("sent to Telegram"),
        "{page}"
    );
    let list = body_string(
        app.oneshot(get_authed("/dashboard/bookings", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(
        list.contains(&format!("/dashboard/bookings/{id}/messages")),
        "{list}"
    );
    assert!(list.contains("Messages (1)"), "{list}");
    drop(tg);
}

#[tokio::test]
async fn the_thread_is_the_hosts_only_and_needs_the_csrf_token() {
    let _tg = fake::install_fake("ilia_test_bot").await;
    let (app, pool, session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    link_guest(&pool, &id, GUEST_CHAT).await;

    // Without the matching CSRF token nothing is stored.
    let forged = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/dashboard/bookings/{id}/messages"))
        .header(
            "cookie",
            format!("__Host-calrs_session={session}; __Host-calrs_csrf=a"),
        )
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("_csrf=b&text=Hi"))
        .unwrap();
    let response = app.clone().oneshot(forged).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
    assert!(thread_rows(&pool, &id).await.is_empty());

    // Another user sees and writes nothing.
    let (_, other) =
        insert_test_user_with_session(&pool, "Mallory", "m@example.com", "mallory").await;
    let response = app
        .clone()
        .oneshot(get_authed(
            &format!("/dashboard/bookings/{id}/messages"),
            &other,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    let response = app
        .clone()
        .oneshot(post_message(&id, &other, "Hi"))
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    assert!(thread_rows(&pool, &id).await.is_empty());

    // Empty, too long, and an ended booking are refused with a reason.
    for (text, says) in [
        ("   ", "Write a message first."),
        (&"x".repeat(2001)[..], "at most 2000 characters"),
    ] {
        let response = app
            .clone()
            .oneshot(post_message(&id, &session, text))
            .await
            .unwrap();
        assert_eq!(response.status(), 422, "{says}");
        let page = body_string(response).await;
        assert!(page.contains(says), "{says}: {page}");
    }
    let page = body_string(
        app.clone()
            .oneshot(get_authed(
                &format!("/dashboard/bookings/{id}/messages?error=too_long"),
                &session,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains("at most 2000 characters"), "{page}");
    let cancelled = booking(&pool, &et_id, 50, "cancelled").await;
    let response = app
        .oneshot(post_message(&cancelled, &session, "Hi"))
        .await
        .unwrap();
    assert_eq!(response.status(), 422);
    assert!(body_string(response)
        .await
        .contains("Messages can be sent only for pending or confirmed bookings"));
    assert!(thread_rows(&pool, &id).await.is_empty());
}

#[tokio::test]
async fn with_no_email_and_no_telegram_nothing_is_stored() {
    // Telegram off in this process for the test's length.
    let _off = fake::TELEGRAM_LOCK.lock().await;
    let (app, pool, session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let response = app
        .clone()
        .oneshot(post_message(&id, &session, "Hello"))
        .await
        .unwrap();
    // Shown at once, the typed text kept in the box.
    assert_eq!(response.status(), 422);
    let page = body_string(response).await;
    assert!(page.contains("The guest cannot be reached"), "{page}");
    assert!(page.contains(">Hello</textarea>"), "{page}");
    assert!(thread_rows(&pool, &id).await.is_empty());
    let page = body_string(
        app.oneshot(get_authed(
            &format!("/dashboard/bookings/{id}/messages?error=no_channel"),
            &session,
        ))
        .await
        .unwrap(),
    )
    .await;
    assert!(page.contains("The guest cannot be reached"), "{page}");
    assert!(
        page.contains("Email is not set up on this server."),
        "{page}"
    );
}

#[tokio::test]
async fn the_daily_limits_hold_email_to_five_and_the_booking_to_twenty() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (app, pool, session, et_id) = setup_test_app().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, port).await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    link_guest(&pool, &id, GUEST_CHAT).await;
    for n in 0..20 {
        let response = app
            .clone()
            .oneshot(post_message(&id, &session, &format!("Note {n}")))
            .await
            .unwrap();
        assert!(location(&response).ends_with("?sent=1"), "{n}");
    }
    let response = app
        .clone()
        .oneshot(post_message(&id, &session, "One too many"))
        .await
        .unwrap();
    assert_eq!(response.status(), 422);
    let page = body_string(response).await;
    assert!(page.contains("daily message limit"), "{page}");
    assert!(page.contains(">One too many</textarea>"), "{page}");
    tg.flush(&pool).await;
    assert_eq!(texts(&tg, GUEST_CHAT).len(), 20);
    let emailed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM booking_messages WHERE booking_id = ? AND email_status != 'none'",
    )
    .bind(&id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(emailed, 5);
    assert_eq!(settled_mails_to(&mails, "guest@test.com").await.len(), 5);

    // Without Telegram, past the email limit there is no channel left.
    let other = booking(&pool, &et_id, 72, "confirmed").await;
    for n in 0..5 {
        let response = app
            .clone()
            .oneshot(post_message(&other, &session, &format!("Mail {n}")))
            .await
            .unwrap();
        assert!(location(&response).ends_with("?sent=1"), "{n}");
    }
    let response = app
        .oneshot(post_message(&other, &session, "Sixth"))
        .await
        .unwrap();
    assert_eq!(response.status(), 422);
    assert_eq!(thread_rows(&pool, &other).await.len(), 5);
    drop(tg);
}

#[tokio::test]
async fn a_guest_text_reaches_the_host_and_the_hosts_reply_reaches_the_guest() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (app, pool, session, et_id) = setup_test_app().await;
    link_host(&pool).await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    link_guest(&pool, &id, GUEST_CHAT).await;

    // A guest text that tries to look like a link and a second line.
    write(
        &tg,
        &pool,
        1,
        GUEST_CHAT,
        "Pay at https://evil.example/x\nnow @admin",
        None,
    )
    .await;
    let to_host = texts(&tg, HOST_CHAT);
    assert_eq!(to_host.len(), 1, "{to_host:#?}");
    assert!(
        to_host[0].starts_with("Message from Guest (guest@test.com) about Test Meeting, "),
        "{}",
        to_host[0]
    );
    assert!(
        to_host[0].contains("\n\nPay at https[:]//evil[.]example/x now [@]admin\n\nReply to this message to answer the guest."),
        "{}",
        to_host[0]
    );
    assert_eq!(
        texts(&tg, GUEST_CHAT),
        ["Your message has been passed on to the host."]
    );
    let rows = thread_rows(&pool, &id).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "guest");
    assert_eq!(rows[0].1, "telegram");
    assert_eq!(rows[0].2, "Pay at https://evil.example/x now @admin");
    assert_eq!(rows[0].4, "sent");

    // The host replies to the relayed message in their own chat.
    let relayed = tg.api.last_id_to(HOST_CHAT);
    write(
        &tg,
        &pool,
        2,
        HOST_CHAT,
        "Thanks, see you then.",
        Some(relayed),
    )
    .await;
    let to_guest = texts(&tg, GUEST_CHAT);
    assert_eq!(to_guest.len(), 2, "{to_guest:#?}");
    assert!(
        to_guest[1].contains("\n\nThanks, see you then.\n\n"),
        "{}",
        to_guest[1]
    );
    assert_eq!(texts(&tg, HOST_CHAT).last().unwrap(), "Sent to the guest.");
    let rows = thread_rows(&pool, &id).await;
    assert_eq!(rows.len(), 2);
    assert_eq!(
        (rows[1].0.as_str(), rows[1].1.as_str(), rows[1].3.as_str()),
        ("host", "telegram", "none")
    );

    // The guest answers by replying to the host's message.
    let host_message = tg.api.last_id_to(GUEST_CHAT);
    write(&tg, &pool, 3, GUEST_CHAT, "Great", Some(host_message)).await;
    assert_eq!(thread_rows(&pool, &id).await.len(), 3);
    assert_eq!(texts(&tg, HOST_CHAT).len(), 3);

    // The dashboard shows the whole thread, the guest's text escaped.
    let page = body_string(
        app.oneshot(get_authed(
            &format!("/dashboard/bookings/{id}/messages"),
            &session,
        ))
        .await
        .unwrap(),
    )
    .await;
    assert!(
        page.contains("evil.example") && page.contains("now @admin"),
        "{page}"
    );
    assert!(page.contains("Thanks, see you then."), "{page}");
    assert!(page.contains("Great"), "{page}");

    // /stop forgets what the chat could reply to.
    write(&tg, &pool, 4, GUEST_CHAT, "/stop", None).await;
    let left: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM telegram_message_map WHERE chat_id = ?")
            .bind(GUEST_CHAT)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(left, 0);
    drop(tg);
}

#[tokio::test]
async fn several_bookings_need_a_reply_and_an_unpaid_one_takes_no_text() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (_app, pool, _session, et_id) = setup_test_app().await;
    link_host(&pool).await;
    let first = booking(&pool, &et_id, 48, "confirmed").await;
    let second = booking(&pool, &et_id, 72, "confirmed").await;
    link_guest(&pool, &first, GUEST_CHAT).await;
    link_guest(&pool, &second, GUEST_CHAT).await;

    write(&tg, &pool, 1, GUEST_CHAT, "Which one?", None).await;
    assert_eq!(
        texts(&tg, GUEST_CHAT),
        ["Several bookings are linked to this chat. Reply to a message about the booking you mean."]
    );
    assert!(texts(&tg, HOST_CHAT).is_empty());

    // A reply to a message about the second booking picks it.
    let msg = crate::telegram::api::Outgoing {
        text: "About the second".into(),
        buttons: Vec::new(),
    };
    assert!(
        crate::telegram::worker::deliver(&tg.rt, &pool, GUEST_CHAT, &msg, "test", None, 1).await
    );
    crate::telegram::store::map_message(
        &pool,
        GUEST_CHAT,
        tg.api.last_id_to(GUEST_CHAT),
        &second,
        "guest",
    )
    .await;
    write(
        &tg,
        &pool,
        2,
        GUEST_CHAT,
        "This one",
        Some(tg.api.last_id_to(GUEST_CHAT)),
    )
    .await;
    assert_eq!(thread_rows(&pool, &second).await.len(), 1);
    assert!(thread_rows(&pool, &first).await.is_empty());
    assert_eq!(texts(&tg, HOST_CHAT).len(), 1);

    // A priced booking that awaits its payment: the hosts never heard of
    // it, and it takes no text.
    let unpaid = booking(&pool, &et_id, 96, "pending").await;
    open_invoice(&pool, &unpaid, &et_id).await;
    const OTHER_CHAT: i64 = 987650002;
    link_guest(&pool, &unpaid, OTHER_CHAT).await;
    write(&tg, &pool, 3, OTHER_CHAT, "Hello?", None).await;
    assert_eq!(
        texts(&tg, OTHER_CHAT),
        ["Messages to the host open once the booking is paid."]
    );
    assert!(thread_rows(&pool, &unpaid).await.is_empty());
    assert_eq!(texts(&tg, HOST_CHAT).len(), 1);
    drop(tg);
}

#[tokio::test]
async fn guest_texts_are_rate_limited_and_never_logged() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (_app, pool, _session, et_id) = setup_test_app().await;
    link_host(&pool).await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    link_guest(&pool, &id, GUEST_CHAT).await;
    let log = LogCapture::default();
    let _guard = log.install();
    for n in 1..=7 {
        write(&tg, &pool, n, GUEST_CHAT, &format!("SECRET-TEXT-{n}"), None).await;
    }
    // Five relayed, one "too many", then silence.
    assert_eq!(texts(&tg, HOST_CHAT).len(), 5);
    let to_guest = texts(&tg, GUEST_CHAT);
    assert_eq!(to_guest.len(), 6, "{to_guest:#?}");
    assert_eq!(to_guest[5], "Too many messages. Please try again later.");
    assert_eq!(thread_rows(&pool, &id).await.len(), 5);
    // The host's reply is not logged either.
    write(
        &tg,
        &pool,
        8,
        HOST_CHAT,
        "SECRET-REPLY",
        Some(tg.api.last_id_to(HOST_CHAT)),
    )
    .await;
    assert_eq!(thread_rows(&pool, &id).await.len(), 6);
    let all = log.all();
    assert!(!all.contains("SECRET-"), "{all}");
    assert!(!all.contains(&GUEST_CHAT.to_string()), "{all}");
    drop(tg);
}

#[tokio::test]
async fn reply_mappings_are_purged_after_a_month_or_with_the_guests_link() {
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let linked = booking(&pool, &et_id, 48, "confirmed").await;
    let unlinked = booking(&pool, &et_id, 72, "confirmed").await;
    link_guest(&pool, &linked, GUEST_CHAT).await;
    let old = crate::telegram::store::sql_time(Utc::now() - Duration::days(31));
    let new = crate::telegram::store::sql_time(Utc::now());
    for (chat, message, booking_id, side, at) in [
        (GUEST_CHAT, 1, &linked, "guest", &new),
        (GUEST_CHAT, 2, &linked, "guest", &old),
        (GUEST_CHAT, 3, &unlinked, "guest", &new),
        (HOST_CHAT, 4, &unlinked, "host", &new),
        (HOST_CHAT, 5, &unlinked, "host", &old),
    ] {
        sqlx::query(
            "INSERT INTO telegram_message_map (chat_id, message_id, booking_id, side, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(chat)
        .bind(message)
        .bind(booking_id)
        .bind(side)
        .bind(at)
        .execute(&pool)
        .await
        .unwrap();
    }
    crate::telegram::store::purge(&pool, Utc::now()).await;
    let left: Vec<i64> =
        sqlx::query_scalar("SELECT message_id FROM telegram_message_map ORDER BY message_id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(left, [1, 4]);
}

// --- review fixes ------------------------------------------------------------

/// A database error is not a "daily limit": the host reads "try again",
/// and the typed text stays in the box.
#[tokio::test]
async fn a_database_error_says_try_again_and_keeps_the_text() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (app, pool, session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    link_guest(&pool, &id, GUEST_CHAT).await;
    sqlx::raw_sql(
        "CREATE TRIGGER busy BEFORE INSERT ON booking_messages BEGIN SELECT RAISE(ABORT, 'busy'); END;",
    )
    .execute(&pool)
    .await
    .unwrap();
    let response = app
        .oneshot(post_message(&id, &session, "Draft that must survive"))
        .await
        .unwrap();
    assert_eq!(response.status(), 422);
    let page = body_string(response).await;
    assert!(page.contains("could not be saved"), "{page}");
    assert!(!page.contains("daily message limit"), "{page}");
    assert!(
        page.contains(">Draft that must survive</textarea>"),
        "{page}"
    );
    drop(tg);
}

/// A new bot (another token) starts with message ids from 1 again, while a
/// private chat's id is the user's: the old bot's reply map is forgotten,
/// so a host's reply to a new bot message never reaches an old booking's
/// guest.
#[tokio::test]
async fn a_new_bot_forgets_the_old_bots_reply_map() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (_app, pool, _session, et_id) = setup_test_app().await;
    link_host(&pool).await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    link_guest(&pool, &id, GUEST_CHAT).await;
    assert!(!crate::telegram::store::claim_bot(&pool, 111111).await);
    crate::telegram::store::map_message(&pool, HOST_CHAT, 2, &id, "host").await;
    assert!(crate::telegram::store::mapped_message(&pool, HOST_CHAT, 2)
        .await
        .is_some());
    assert!(crate::telegram::store::claim_bot(&pool, 222222).await);
    assert_eq!(
        crate::telegram::store::mapped_message(&pool, HOST_CHAT, 2).await,
        None
    );
    // The host replies to the new bot's message 2: nothing reaches the guest.
    write(&tg, &pool, 1, HOST_CHAT, "Meant for someone else", Some(2)).await;
    assert!(thread_rows(&pool, &id).await.is_empty());
    assert!(texts(&tg, GUEST_CHAT).is_empty());
    drop(tg);
}

/// Refused texts (an unpaid booking) do not use up the hourly relay budget;
/// past the budget a text is still stored, marked not relayed, and shown
/// on the dashboard.
#[tokio::test]
async fn the_relay_budget_counts_only_relayed_texts() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (app, pool, session, et_id) = setup_test_app().await;
    link_host(&pool).await;
    let unpaid = booking(&pool, &et_id, 96, "pending").await;
    open_invoice(&pool, &unpaid, &et_id).await;
    const UNPAID_CHAT: i64 = 987650003;
    link_guest(&pool, &unpaid, UNPAID_CHAT).await;
    for n in 1..=4 {
        write(&tg, &pool, n, UNPAID_CHAT, "spam", None).await;
    }
    assert_eq!(
        tg.rt.limits.relays.hit(&()),
        1,
        "refused texts spent nothing"
    );
    for _ in 0..tg.rt.limits.relays.max() {
        tg.rt.limits.relays.hit(&());
    }
    let paid = booking(&pool, &et_id, 48, "confirmed").await;
    link_guest(&pool, &paid, GUEST_CHAT).await;
    write(&tg, &pool, 10, GUEST_CHAT, "Over the budget", None).await;
    let rows = thread_rows(&pool, &paid).await;
    assert_eq!(rows.len(), 1, "stored all the same");
    assert_eq!(rows[0].4, "failed");
    assert!(texts(&tg, HOST_CHAT).is_empty());
    let page = body_string(
        app.oneshot(get_authed(
            &format!("/dashboard/bookings/{paid}/messages"),
            &session,
        ))
        .await
        .unwrap(),
    )
    .await;
    assert!(page.contains("Over the budget"), "{page}");
    drop(tg);
}

/// A host who wrote to the guest of an unpaid booking hears the answer.
#[tokio::test]
async fn the_guest_of_an_unpaid_booking_may_answer_the_host() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (app, pool, session, et_id) = setup_test_app().await;
    link_host(&pool).await;
    let unpaid = booking(&pool, &et_id, 96, "pending").await;
    open_invoice(&pool, &unpaid, &et_id).await;
    link_guest(&pool, &unpaid, GUEST_CHAT).await;
    write(&tg, &pool, 1, GUEST_CHAT, "Before anyone wrote", None).await;
    assert_eq!(
        texts(&tg, GUEST_CHAT),
        ["Messages to the host open once the booking is paid."]
    );
    let response = app
        .oneshot(post_message(&unpaid, &session, "Is 10:00 still fine?"))
        .await
        .unwrap();
    assert!(location(&response).ends_with("?sent=1"));
    tg.flush(&pool).await;
    let host_message = tg.api.last_id_to(GUEST_CHAT);
    write(&tg, &pool, 2, GUEST_CHAT, "Yes, fine", Some(host_message)).await;
    let rows = thread_rows(&pool, &unpaid).await;
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(
        (rows[1].0.as_str(), rows[1].2.as_str()),
        ("guest", "Yes, fine")
    );
    assert_eq!(texts(&tg, HOST_CHAT).len(), 1);
    drop(tg);
}

/// A host reply over 2000 characters, or with nothing visible, is answered
/// instead of dropped; a guest text over 1000 characters is said to be
/// shortened.
#[tokio::test]
async fn long_or_empty_texts_are_answered() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (_app, pool, _session, et_id) = setup_test_app().await;
    link_host(&pool).await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    link_guest(&pool, &id, GUEST_CHAT).await;
    write(&tg, &pool, 1, GUEST_CHAT, &"long ".repeat(300), None).await;
    assert_eq!(
        texts(&tg, GUEST_CHAT).last().unwrap(),
        "Your message has been passed on to the host, shortened to 1000 characters."
    );
    let relayed = tg.api.last_id_to(HOST_CHAT);
    write(&tg, &pool, 2, HOST_CHAT, &"x".repeat(2500), Some(relayed)).await;
    assert_eq!(
        texts(&tg, HOST_CHAT).last().unwrap(),
        "The reply was not sent: a message can have at most 2000 characters."
    );
    write(&tg, &pool, 3, HOST_CHAT, "\u{200B}\u{2060}", Some(relayed)).await;
    assert_eq!(
        texts(&tg, HOST_CHAT).last().unwrap(),
        "The reply was not sent: it has no visible text."
    );
    assert_eq!(
        thread_rows(&pool, &id).await.len(),
        1,
        "only the guest text"
    );
    drop(tg);
}

/// A meeting that started leaves the upcoming list but still takes
/// messages for a week: the dashboard lists it under "Recent meetings",
/// with the guest's unanswered messages counted.
#[tokio::test]
async fn recent_meetings_keep_their_thread_reachable() {
    let tg = fake::install_fake("ilia_test_bot").await;
    let (app, pool, session, et_id) = setup_test_app().await;
    let past = insert_v1_booking(
        &pool,
        &et_id,
        "past@test.com",
        Utc::now() - Duration::hours(3),
        30,
        "confirmed",
        &[("language", Some("en".into()))],
    )
    .await;
    let old = insert_v1_booking(
        &pool,
        &et_id,
        "old@test.com",
        Utc::now() - Duration::days(9),
        30,
        "confirmed",
        &[("language", Some("en".into()))],
    )
    .await;
    link_guest(&pool, &past, GUEST_CHAT).await;
    write(
        &tg,
        &pool,
        1,
        GUEST_CHAT,
        "Could you send the slides?",
        None,
    )
    .await;
    assert_eq!(thread_rows(&pool, &past).await.len(), 1);
    let page = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/bookings", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains("Recent meetings"), "{page}");
    assert!(
        page.contains(&format!("/dashboard/bookings/{past}/messages")),
        "{page}"
    );
    assert!(page.contains("Messages (1 new)"), "{page}");
    assert!(
        !page.contains(&format!("/dashboard/bookings/{old}/messages")),
        "{page}"
    );
    // Answered: no longer new.
    let response = app
        .clone()
        .oneshot(post_message(&past, &session, "Here they are."))
        .await
        .unwrap();
    assert!(location(&response).ends_with("?sent=1"));
    let page = body_string(
        app.oneshot(get_authed("/dashboard/bookings", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains("Messages (2)"), "{page}");
    drop(tg);
}

/// Every log event's fields, as text.
#[derive(Clone, Default)]
struct LogCapture(Arc<std::sync::Mutex<Vec<String>>>);

impl LogCapture {
    fn install(&self) -> tracing::subscriber::DefaultGuard {
        use tracing_subscriber::layer::SubscriberExt;
        let guard =
            tracing::subscriber::set_default(tracing_subscriber::registry().with(self.clone()));
        tracing::callsite::rebuild_interest_cache();
        guard
    }

    fn all(&self) -> String {
        self.0.lock().unwrap().join("\n")
    }
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for LogCapture {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        struct Fields(String);
        impl tracing::field::Visit for Fields {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                use std::fmt::Write;
                let _ = write!(self.0, "{}={:?} ", field.name(), value);
            }
        }
        let mut fields = Fields(String::new());
        event.record(&mut fields);
        self.0.lock().unwrap().push(fields.0);
    }
}
