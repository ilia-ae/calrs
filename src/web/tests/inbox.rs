//! ilia fork: owner — system — client (`crate::inbox`). The poller against
//! the in-process fake IMAP server (`inbox::fake`) and a real database:
//! matching by token, the filters, the text, dedupe, the folder state, the
//! done folder, the back-off and the one reused connection; the relay to the
//! host's Telegram (`telegram::fake`) and the dashboard thread; a Telegram
//! reply going back by email from the system; and the sentinels: the owner's
//! own address reaches no guest mail, `.ics` or page, and inbound text and
//! addresses never reach the log.

use super::messages::{
    booking, link_host, open_invoice, texts, thread_rows, LogCapture, HOST_CHAT,
};
use super::*;
use crate::inbox::fake::{self, PASSWORD, USER};
use crate::inbox::filter::Ours;
use crate::inbox::imap::Client;
use crate::inbox::poller::{self, Ctx, Policy, Stats};
use crate::inbox::{reply, test_mode, Config, TestGuard};
use crate::telegram::fake as tg_fake;

const INBOX: &str = "booking@ilia.ae";

fn relay(plus: bool) -> TestGuard {
    test_mode(true, Some(Config::for_tests(INBOX, plus)))
}

fn ctx(pool: &SqlitePool, config: Config) -> Ctx {
    Ctx::new(
        pool.clone(),
        std::sync::Arc::new(config),
        crate::inbox::key().unwrap(),
        Ours::new(INBOX, Some("noreply@test.com")),
    )
}

fn token_of(booking_id: &str) -> String {
    reply::token(&format!("uid-{booking_id}")).unwrap()
}

/// A plain-text message.
fn mail(from: &str, to: &str, subject: &str, extra: &str, body: &str) -> String {
    format!(
        "From: Guest <{from}>\r\nTo: {to}\r\nSubject: {subject}\r\nMessage-ID: <{}@mail.example>\r\n\
         Date: Mon, 10 Nov 2026 10:00:00 +0000\r\n{extra}Content-Type: text/plain; charset=utf-8\r\n\r\n{body}\r\n",
        uuid::Uuid::new_v4()
    )
}

/// One session: log in, select, poll once, log out.
async fn poll(ctx: &Ctx, port: u16) -> Stats {
    let s = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    let mut c = Client::new(s);
    c.greeting().await.unwrap();
    c.capability().await.unwrap();
    c.login(USER, PASSWORD).await.unwrap();
    c.capability().await.unwrap();
    let sel = c.select(&ctx.config.folder).await.unwrap();
    let stats = poller::poll_once(&mut c, ctx, &Policy::default(), sel)
        .await
        .unwrap();
    c.logout().await.unwrap();
    stats
}

fn count(stats: &Stats, what: &str) -> u32 {
    stats.0.get(what).copied().unwrap_or(0)
}

fn seen(state: &fake::Shared, uid: u32) -> bool {
    state
        .lock()
        .unwrap()
        .flags_of(uid)
        .is_some_and(|f| f.iter().any(|f| f == "\\Seen"))
}

#[tokio::test]
async fn a_plus_addressed_reply_is_stored_relayed_and_marked_seen() {
    let tg = tg_fake::install_fake("ilia_test_bot").await;
    let _relay = relay(true);
    let (app, pool, session, et_id) = setup_test_app().await;
    link_host(&pool).await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let token = token_of(&id);
    let state = fake::shared();
    let uid = state.lock().unwrap().deliver(
        &mail(
            "guest@test.com",
            &format!("Booking <booking+{token}@ilia.ae>"),
            "Re: Message from Test User",
            "",
            "Yes, Tuesday works.\r\nhttps://evil.example/x\r\n\r\nOn Mon, 10 Nov 2026, Test User <booking@ilia.ae> wrote:\r\n> Can we move it?",
        ),
        false,
    );
    let port = fake::serve(state.clone()).await;
    let stats = poll(&ctx(&pool, Config::for_tests(INBOX, true)), port).await;
    assert_eq!(count(&stats, "stored"), 1, "{stats:?}");
    assert!(seen(&state, uid));
    tg.flush(&pool).await;

    let rows = thread_rows(&pool, &id).await;
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].0.as_str(), rows[0].1.as_str()), ("guest", "email"));
    assert_eq!(rows[0].2, "Yes, Tuesday works.\nhttps://evil.example/x");
    assert_eq!(rows[0].4, "sent");
    let to_host = texts(&tg, HOST_CHAT);
    assert_eq!(to_host.len(), 1, "{to_host:#?}");
    assert!(
        to_host[0].starts_with("Email from Guest (guest@test.com) about Test Meeting, "),
        "{}",
        to_host[0]
    );
    assert!(
        to_host[0].contains("\n\nYes, Tuesday works. https[:]//evil[.]example/x\n\n"),
        "{}",
        to_host[0]
    );

    // The dashboard thread shows it as an email, escaped.
    let page = body_string(
        app.oneshot(get_authed(
            &format!("/dashboard/bookings/{id}/messages"),
            &session,
        ))
        .await
        .unwrap(),
    )
    .await;
    assert!(page.contains("Yes, Tuesday works."), "{page}");
    assert!(page.contains("&middot; email"), "{page}");
    drop(tg);
}

#[tokio::test]
async fn references_and_subject_tags_match_and_strangers_do_not() {
    let _relay = relay(false);
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let ended = booking(&pool, &et_id, -24 * 9, "confirmed").await;
    let token = token_of(&id);
    let state = fake::shared();
    let (by_ref, stranger_tag, guest_tag, wrong, late) = {
        let mut st = state.lock().unwrap();
        (
            // A stranger replying in the thread: the Message-ID is proof.
            st.deliver(
                &mail(
                    "assistant@corp.example",
                    INBOX,
                    "Re: hi",
                    &format!("In-Reply-To: <calrs.{token}.00aa11bb22cc33dd@ilia.ae>\r\n"),
                    "On behalf of the guest: fine.",
                ),
                false,
            ),
            st.deliver(
                &mail(
                    "stranger@corp.example",
                    INBOX,
                    &format!("Re: [#{token}]"),
                    "",
                    "Spoofed",
                ),
                false,
            ),
            st.deliver(
                &mail(
                    "guest@test.com",
                    INBOX,
                    &format!("Re: Message from Test User [#{token}]"),
                    "",
                    "Tagged",
                ),
                false,
            ),
            st.deliver(
                &mail(
                    "guest@test.com",
                    "booking+aaaaaaaaaaaaaaaa@ilia.ae",
                    "Re: [#bbbbbbbbbbbbbbbb]",
                    "",
                    "Wrong",
                ),
                false,
            ),
            st.deliver(
                &mail(
                    "guest@test.com",
                    INBOX,
                    &format!("Re: [#{}]", token_of(&ended)),
                    "",
                    "Too late",
                ),
                false,
            ),
        )
    };
    let port = fake::serve(state.clone()).await;
    let stats = poll(&ctx(&pool, Config::for_tests(INBOX, false)), port).await;
    assert_eq!(count(&stats, "stored"), 2, "{stats:?}");
    assert_eq!(count(&stats, "unmatched"), 3, "{stats:?}");
    assert!(seen(&state, by_ref) && seen(&state, guest_tag));
    // Unmatched mail is left unread for a human.
    for uid in [stranger_tag, wrong, late] {
        assert!(!seen(&state, uid), "{uid}");
    }
    let bodies: Vec<String> = thread_rows(&pool, &id)
        .await
        .into_iter()
        .map(|r| r.2)
        .collect();
    assert_eq!(bodies, ["On behalf of the guest: fine.", "Tagged"]);
    assert!(thread_rows(&pool, &ended).await.is_empty());
}

#[tokio::test]
async fn automatic_mail_is_marked_handled_and_never_stored() {
    let _relay = relay(true);
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let to = format!("booking+{}@ilia.ae", token_of(&id));
    let state = fake::shared();
    let big = "x".repeat(300 * 1024);
    let uids: Vec<u32> = {
        let mut st = state.lock().unwrap();
        let mut d = |raw: String| st.deliver(&raw, false);
        vec![
            d(mail("guest@test.com", &to, "Out of office", "Auto-Submitted: auto-replied\r\n", "Away")),
            // Kerio's out-of-office reply.
            d(mail("guest@test.com", &to, "Abwesend", "Auto-Submitted: auto-replied (vacation)\r\nX-Auto-Response-Suppress: All\r\n", "Away")),
            d(mail("guest@test.com", &to, "News", "List-Id: <news.example>\r\n", "Buy")),
            d(mail(INBOX, &to, "Loop", "", "Ours")),
            d(format!(
                "From: MAILER-DAEMON@mx.example\r\nTo: {to}\r\nSubject: Undelivered\r\n\
                 Content-Type: multipart/report; report-type=delivery-status; boundary=\"b\"\r\n\r\n\
                 --b\r\nContent-Type: text/plain\r\n\r\nfailed\r\n--b--\r\n"
            )),
            d(format!(
                "From: guest@test.com\r\nTo: {to}\r\nSubject: Accepted\r\n\
                 Content-Type: multipart/mixed; boundary=\"b\"\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nok\r\n\
                 --b\r\nContent-Type: text/calendar; method=REPLY\r\n\r\nBEGIN:VCALENDAR\r\nMETHOD:REPLY\r\nEND:VCALENDAR\r\n--b--\r\n"
            )),
        ]
    };
    let oversize = state
        .lock()
        .unwrap()
        .deliver(&mail("guest@test.com", &to, "Big", "", &big), false);
    let port = fake::serve(state.clone()).await;
    let stats = poll(&ctx(&pool, Config::for_tests(INBOX, true)), port).await;
    for what in ["autoreply", "list", "loop", "report", "calendar"] {
        assert!(count(&stats, what) >= 1, "{what}: {stats:?}");
    }
    assert_eq!(count(&stats, "autoreply"), 2, "{stats:?}");
    assert_eq!(count(&stats, "oversize"), 1, "{stats:?}");
    for uid in uids {
        assert!(seen(&state, uid), "{uid}");
    }
    // The oversized body was never fetched, and it is left for a human.
    assert!(!seen(&state, oversize));
    let fetched_bodies = state
        .lock()
        .unwrap()
        .commands
        .iter()
        .filter(|c| c.contains("BODY.PEEK") && c.contains(&format!(" {oversize} ")))
        .count();
    assert_eq!(fetched_bodies, 0);
    assert!(thread_rows(&pool, &id).await.is_empty());
}

#[tokio::test]
async fn html_mail_is_text_long_mail_is_shortened_and_a_message_is_stored_once() {
    let _relay = relay(true);
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let to = format!("booking+{}@ilia.ae", token_of(&id));
    let state = fake::shared();
    let html = format!(
        "From: guest@test.com\r\nTo: {to}\r\nSubject: hi\r\nMessage-ID: <html-1@mail.example>\r\n\
         Content-Type: text/html; charset=utf-8\r\n\r\n<p>See <b>you</b></p><script>steal()</script>\r\n"
    );
    let long = mail("guest@test.com", &to, "long", "", &"word ".repeat(400));
    {
        let mut st = state.lock().unwrap();
        st.deliver(&html, false);
        st.deliver(&long, false);
    }
    let port = fake::serve(state.clone()).await;
    let c = ctx(&pool, Config::for_tests(INBOX, true));
    assert_eq!(count(&poll(&c, port).await, "stored"), 2);
    let rows = thread_rows(&pool, &id).await;
    assert!(
        rows[0].2.contains("See") && rows[0].2.contains("you"),
        "{}",
        rows[0].2
    );
    assert!(!rows[0].2.contains('<'), "{}", rows[0].2);
    assert_eq!(rows[1].2.chars().count(), 1000);
    assert!(rows[1].2.ends_with('…'));
    // The same message again (another alias, a re-delivery): stored once.
    state.lock().unwrap().deliver(&html, false);
    let stats = poll(&c, port).await;
    assert_eq!(count(&stats, "duplicate"), 1, "{stats:?}");
    assert_eq!(thread_rows(&pool, &id).await.len(), 2);
}

#[tokio::test]
async fn the_first_run_takes_unseen_mail_and_a_new_uidvalidity_starts_over() {
    let _relay = relay(true);
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let to = format!("booking+{}@ilia.ae", token_of(&id));
    let state = fake::shared();
    {
        let mut st = state.lock().unwrap();
        st.deliver(
            &mail("guest@test.com", &to, "old", "", "Old, already read"),
            true,
        );
        st.deliver(&mail("guest@test.com", &to, "new", "", "Unread"), false);
    }
    let port = fake::serve(state.clone()).await;
    let c = ctx(&pool, Config::for_tests(INBOX, true));
    assert_eq!(count(&poll(&c, port).await, "stored"), 1);
    let saved = crate::inbox::store::state(&pool, "INBOX").await.unwrap();
    assert_eq!((saved.uidvalidity, saved.last_uid), (1000, 2));
    // A message read by someone in webmail is still taken once it is new.
    state.lock().unwrap().deliver(
        &mail("guest@test.com", &to, "next", "", "Read in webmail"),
        true,
    );
    assert_eq!(count(&poll(&c, port).await, "stored"), 1);
    // The mailbox was rebuilt: UIDs mean nothing any more.
    {
        let mut st = state.lock().unwrap();
        st.uidvalidity = 2000;
        st.deliver(
            &mail("guest@test.com", &to, "before", "", "Before the reset"),
            false,
        );
    }
    assert!(poll(&c, port).await.0.is_empty());
    state.lock().unwrap().deliver(
        &mail("guest@test.com", &to, "after", "", "After the reset"),
        false,
    );
    assert_eq!(count(&poll(&c, port).await, "stored"), 1);
    let bodies: Vec<String> = thread_rows(&pool, &id)
        .await
        .into_iter()
        .map(|r| r.2)
        .collect();
    assert_eq!(bodies, ["Unread", "Read in webmail", "After the reset"]);
}

#[tokio::test]
async fn processed_mail_is_moved_or_copied_and_expunged_by_capability() {
    let _relay = relay(true);
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let to = format!("booking+{}@ilia.ae", token_of(&id));
    let mut config = Config::for_tests(INBOX, true);
    config.done_folder = Some("calrs".into());
    for caps in [vec!["MOVE"], vec!["UIDPLUS"], vec![]] {
        let state = fake::shared();
        state.lock().unwrap().caps = caps.clone();
        let uid = state.lock().unwrap().deliver(
            &mail("guest@test.com", &to, "hi", "", &format!("via {caps:?}")),
            false,
        );
        let port = fake::serve(state.clone()).await;
        sqlx::query("DELETE FROM inbox_state")
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            count(&poll(&ctx(&pool, config.clone()), port).await, "stored"),
            1
        );
        let st = state.lock().unwrap();
        match caps.first().copied() {
            Some("MOVE") => assert_eq!(st.moved, vec![(uid, "calrs".to_string())]),
            Some("UIDPLUS") => {
                assert_eq!(st.copied, vec![(uid, "calrs".to_string())]);
                assert_eq!(st.expunged, vec![uid]);
            }
            _ => {
                assert!(st.moved.is_empty() && st.copied.is_empty());
                assert!(st.flags_of(uid).unwrap().contains(&"\\Seen".to_string()));
            }
        }
    }
}

#[tokio::test]
async fn an_unpaid_booking_takes_no_email_until_the_host_wrote() {
    let _relay = relay(true);
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "pending").await;
    open_invoice(&pool, &id, &et_id).await;
    let to = format!("booking+{}@ilia.ae", token_of(&id));
    let state = fake::shared();
    state.lock().unwrap().deliver(
        &mail("guest@test.com", &to, "hi", "", "Before paying"),
        false,
    );
    let port = fake::serve(state.clone()).await;
    let c = ctx(&pool, Config::for_tests(INBOX, true));
    assert_eq!(count(&poll(&c, port).await, "unpaid"), 1);
    assert!(thread_rows(&pool, &id).await.is_empty());
    // Not relayed, not stored, but left unread for a person to see.
    assert!(!seen(&state, 1));
    sqlx::query(
        "INSERT INTO booking_messages (id, booking_id, direction, via, body, created_at)
         VALUES ('h1', ?, 'host', 'dashboard', 'Any questions?', datetime('now'))",
    )
    .bind(&id)
    .execute(&pool)
    .await
    .unwrap();
    state.lock().unwrap().deliver(
        &mail("guest@test.com", &to, "Re", "", "One question"),
        false,
    );
    assert_eq!(count(&poll(&c, port).await, "stored"), 1);
}

fn fast(poll_ms: u64) -> Policy {
    Policy {
        poll: Some(std::time::Duration::from_millis(poll_ms)),
        backoff_start: std::time::Duration::from_millis(300),
        backoff_max: std::time::Duration::from_secs(5),
        auth_backoff: std::time::Duration::from_secs(60),
        batch: 50,
    }
}

async fn run_for(c: Ctx, connect: crate::inbox::tls::Connector, policy: Policy, ms: u64) {
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let task = tokio::spawn(poller::run(
        std::sync::Arc::new(c),
        connect,
        policy,
        stop_rx,
    ));
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    stop_tx.send(true).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn one_connection_serves_every_poll_and_failures_back_off() {
    let _relay = relay(true);
    let (_app, pool, _session, _et_id) = setup_test_app().await;
    // Several polls, one connection, one login, a LOGOUT at the end.
    let state = fake::shared();
    let port = fake::serve(state.clone()).await;
    let c = ctx(&pool, Config::for_tests(INBOX, true));
    run_for(c, fake::plain_connector(port), fast(40), 400).await;
    {
        let st = state.lock().unwrap();
        assert_eq!((st.connections, st.logins), (1, 1));
        assert!(st.count("NOOP") >= 3, "{:?}", st.commands);
        assert_eq!(st.count("LOGOUT"), 1);
    }
    // A refused login waits (an hour in production): one attempt only.
    let state = fake::shared();
    state.lock().unwrap().accept_login = false;
    let port = fake::serve(state.clone()).await;
    let c = ctx(&pool, Config::for_tests(INBOX, true));
    run_for(c, fake::plain_connector(port), fast(40), 500).await;
    assert_eq!(state.lock().unwrap().logins, 1);
    // A dropped connection reconnects after the back-off, never at once.
    let state = fake::shared();
    state.lock().unwrap().drop_at = Some("UID SEARCH");
    let port = fake::serve(state.clone()).await;
    let c = std::sync::Arc::new(ctx(&pool, Config::for_tests(INBOX, true)));
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let task = tokio::spawn(poller::run(
        c,
        fake::plain_connector(port),
        fast(40),
        stop_rx,
    ));
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    assert_eq!(state.lock().unwrap().connections, 1);
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    assert_eq!(state.lock().unwrap().connections, 2);
    stop_tx.send(true).unwrap();
    task.await.unwrap();
    // An unreachable server is a transport failure: logged, retried later.
    let log = LogCapture::default();
    let _guard = log.install();
    let c = ctx(&pool, Config::for_tests(INBOX, true));
    run_for(c, fake::dead_connector(), fast(40), 100).await;
    assert!(
        log.all().contains("inbox server unreachable (transport)"),
        "{}",
        log.all()
    );
}

#[tokio::test]
async fn a_telegram_reply_to_an_email_guest_goes_by_email_from_the_system() {
    let tg = tg_fake::install_fake("ilia_test_bot").await;
    let _ = tg.rt.secret_key.set([0u8; 32]);
    let _relay = relay(true);
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let (mta, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, mta).await;
    link_host(&pool).await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let token = token_of(&id);
    let state = fake::shared();
    state.lock().unwrap().deliver(
        &mail(
            "guest@test.com",
            &format!("booking+{token}@ilia.ae"),
            "Question",
            "",
            "SECRET-GUEST-TEXT: is there parking?",
        ),
        false,
    );
    let port = fake::serve(state.clone()).await;
    let log = LogCapture::default();
    let guard = log.install();
    assert_eq!(
        count(
            &poll(&ctx(&pool, Config::for_tests(INBOX, true)), port).await,
            "stored"
        ),
        1
    );
    tg.flush(&pool).await;
    // Inbound only: the outbound SMTP path logs its recipient at debug.
    drop(guard);
    // The host answers the relayed message in Telegram.
    let relayed = tg.api.last_id_to(HOST_CHAT);
    let update = tg_fake::private_message(1, HOST_CHAT, "Yes, behind the building.", Some(relayed));
    crate::telegram::commands::handle_update(&tg.rt, &pool, &update).await;
    tg.flush(&pool).await;
    assert_eq!(texts(&tg, HOST_CHAT).last().unwrap(), "Sent to the guest.");
    let sent = mail_to(&mails, "guest@test.com").await;
    drain_emails(std::time::Duration::from_secs(10)).await;
    assert!(sent.contains("Yes, behind the building."), "{sent}");
    assert_eq!(
        mail_header(&sent, "Reply-To"),
        Some(format!("booking+{token}@ilia.ae").as_str()),
        "{sent}"
    );
    let message_id = mail_header(&sent, "Message-ID").unwrap();
    assert!(
        message_id.starts_with(&format!("<calrs.{token}.")),
        "{message_id}"
    );
    assert!(mail_header(&sent, "From")
        .unwrap()
        .contains("<noreply@test.com>"));
    assert_eq!(mail_header(&sent, "X-Calrs-Origin"), Some("1"));
    assert!(!sent.contains("test@example.com"), "{sent}");
    let rows = thread_rows(&pool, &id).await;
    assert_eq!(
        (rows[1].0.as_str(), rows[1].1.as_str(), rows[1].3.as_str()),
        ("host", "telegram", "sent")
    );
    // Neither the guest's text nor an address reached the log.
    let all = log.all();
    assert!(!all.contains("SECRET-GUEST-TEXT"), "{all}");
    assert!(!all.contains("guest@test.com"), "{all}");
    assert!(!all.contains(PASSWORD), "{all}");
    drop(tg);
}

#[tokio::test]
async fn no_guest_mail_ics_or_page_names_the_owners_own_address() {
    let _relay = relay(false);
    let (app, pool, session, et_id) = setup_test_app().await;
    sqlx::query("UPDATE users SET booking_email = 'owner.private@personal.example' WHERE email = 'test@example.com'")
        .execute(&pool)
        .await
        .unwrap();
    let (mta, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, mta).await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let token = token_of(&id);
    // A host message: system From, Reply-To the bare mailbox, the tag.
    let response = app
        .clone()
        .oneshot(post_form(
            &format!("/dashboard/bookings/{id}/messages"),
            &session,
            "msg-csrf",
            "_csrf=msg-csrf&text=See+you",
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    let sent = mail_to(&mails, "guest@test.com").await;
    drain_emails(std::time::Duration::from_secs(10)).await;
    assert_eq!(mail_header(&sent, "Reply-To"), Some(INBOX), "{sent}");
    // The subject's tag (folded onto the header's next line).
    assert!(sent.contains(&format!(" [#{token}]")), "{sent}");
    for personal in ["owner.private@personal.example", "test@example.com"] {
        assert!(!sent.contains(personal), "{personal}: {sent}");
    }
    // A guest invite names the system as ORGANIZER; the host's own copies
    // (CalDAV write-back) keep the host.
    let details = crate::email::BookingDetails {
        uid: format!("uid-{id}"),
        host_name: "Test User".into(),
        host_email: "owner.private@personal.example".into(),
        guest_name: "Guest".into(),
        guest_email: "guest@test.com".into(),
        utc_times: Some(("20301010T100000Z".into(), "20301010T103000Z".into())),
        ..Default::default()
    };
    let guest_ics = crate::email::generate_ics(&details, "REQUEST");
    assert!(
        guest_ics.contains("ORGANIZER;CN=Test User:mailto:booking@ilia.ae"),
        "{guest_ics}"
    );
    assert!(!guest_ics.contains("personal.example"), "{guest_ics}");
    let caldav_ics = crate::email::generate_ics_caldav(&details);
    assert!(
        caldav_ics.contains("mailto:owner.private@personal.example"),
        "{caldav_ics}"
    );
    // The "too late to cancel" page offers the system mailbox, tagged.
    let cancel: String = sqlx::query_scalar("SELECT cancel_token FROM bookings WHERE id = ?")
        .bind(&id)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE event_types SET cancel_notice_min = 5000 WHERE id = ?")
        .bind(&et_id)
        .execute(&pool)
        .await
        .unwrap();
    let page = body_string(
        app.clone()
            .oneshot(get(&format!("/booking/cancel/{cancel}")))
            .await
            .unwrap(),
    )
    .await;
    assert!(
        page.contains(&format!("mailto:booking@ilia.ae?subject=%5B%23{token}%5D")),
        "{page}"
    );
    assert!(
        !page.contains("personal.example") && !page.contains("test@example.com"),
        "{page}"
    );
    // With no mailbox read, the page offers no address at all.
    drop(_relay);
    let _no_inbox = test_mode(true, None);
    let page = body_string(
        app.oneshot(get(&format!("/booking/cancel/{cancel}")))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains("let the host know another way"), "{page}");
    assert!(!page.contains("mailto:"), "{page}");
}

// --- Review fixes ---

async fn sender_of(pool: &SqlitePool, booking_id: &str) -> Vec<(Option<String>, Option<String>)> {
    sqlx::query_as(
        "SELECT sender, sender_email FROM booking_messages WHERE booking_id = ? ORDER BY created_at, rowid",
    )
    .bind(booking_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// The token in a Message-ID or a plus address proves the sender got our
/// mail, not that it is the guest: a forward's recipient replying is
/// stored as "another address", with that address, and said to be one in
/// Telegram, on the dashboard and in MCP; an attendee as an attendee.
#[tokio::test]
async fn mail_from_another_address_is_never_shown_as_the_guest() {
    let tg = tg_fake::install_fake("ilia_test_bot").await;
    let _relay = relay(false);
    let (app, pool, session, et_id) = setup_test_app().await;
    link_host(&pool).await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    sqlx::query("INSERT INTO booking_attendees (id, booking_id, email) VALUES ('att-1', ?, 'friend@test.com')")
        .bind(&id)
        .execute(&pool)
        .await
        .unwrap();
    let token = token_of(&id);
    let state = fake::shared();
    {
        let mut st = state.lock().unwrap();
        for (from, body) in [
            ("guest@test.com", "From the guest."),
            ("friend@test.com", "From the attendee."),
            ("scammer@evil.example", "Please pay to a new wallet."),
        ] {
            st.deliver(
                &mail(
                    from,
                    INBOX,
                    "Re: hi",
                    &format!("References: <calrs.{token}.00aa11bb22cc33dd@ilia.ae>\r\n"),
                    body,
                ),
                false,
            );
        }
    }
    let port = fake::serve(state.clone()).await;
    let stats = poll(&ctx(&pool, Config::for_tests(INBOX, false)), port).await;
    assert_eq!(count(&stats, "stored"), 3, "{stats:?}");
    assert_eq!(
        sender_of(&pool, &id).await,
        vec![
            (Some("guest".into()), None),
            (Some("attendee".into()), Some("friend@test.com".into())),
            (Some("other".into()), Some("scammer@evil.example".into())),
        ]
    );
    tg.flush(&pool).await;
    let to_host = texts(&tg, HOST_CHAT);
    assert_eq!(to_host.len(), 3, "{to_host:#?}");
    assert!(
        to_host[0].starts_with("Email from Guest (guest@test.com) about "),
        "{}",
        to_host[0]
    );
    assert!(
        to_host[1].starts_with("Email from friend@test.com, an additional attendee of the booking with Guest (guest@test.com), about "),
        "{}",
        to_host[1]
    );
    assert!(
        to_host[2].starts_with("Email from another address, not the guest's: scammer@evil.example, in the thread of the booking with Guest (guest@test.com) about "),
        "{}",
        to_host[2]
    );
    // The dashboard marks it.
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
        page.contains("Another address, not the guest&#x27;s: scammer@evil.example")
            || page.contains("Another address, not the guest's: scammer@evil.example"),
        "{page}"
    );
    assert!(page.contains("Attendee: friend@test.com"), "{page}");
    assert!(page.contains("msg-not-guest"), "{page}");
    // MCP's get_booking thread.
    let thread = crate::web::agent::ops_messages::thread_of(&pool, &id, chrono_tz::Tz::UTC)
        .await
        .unwrap();
    let from: Vec<&str> = thread.iter().map(|m| m.from.as_str()).collect();
    assert_eq!(from, ["guest", "attendee", "other"]);
    let json = serde_json::to_value(&thread[2]).unwrap();
    assert_eq!(
        json["guest_supplied"]["sender_email"],
        "scammer@evil.example"
    );
    assert!(serde_json::to_value(&thread[0]).unwrap()["guest_supplied"]
        .get("sender_email")
        .is_none());
    drop(tg);
}

/// A backlog far past what one `* SEARCH` line may hold (64 KiB, ~8 bytes a
/// UID here) is searched a span at a time, and the poll goes on.
#[tokio::test]
async fn a_large_backlog_is_searched_in_bounded_spans() {
    let _relay = relay(true);
    let (_app, pool, _session, _et_id) = setup_test_app().await;
    let state = fake::shared();
    {
        let mut st = state.lock().unwrap();
        st.uidnext = 1_000_000;
        let junk = "From: x@spam.example\r\nTo: booking@ilia.ae\r\nSubject: buy\r\n\r\nbuy\r\n";
        for _ in 0..10_000 {
            st.deliver(junk, false);
        }
    }
    crate::inbox::store::save_state(
        &pool,
        "INBOX",
        crate::inbox::store::State {
            uidvalidity: 1000,
            last_uid: 999_999,
        },
    )
    .await;
    let port = fake::serve(state.clone()).await;
    let stats = poll(&ctx(&pool, Config::for_tests(INBOX, true)), port).await;
    assert_eq!(count(&stats, "unmatched"), 50, "{stats:?}");
    assert_eq!(count(&stats, "deferred"), 950, "{stats:?}");
    let saved = crate::inbox::store::state(&pool, "INBOX").await.unwrap();
    assert_eq!(saved.last_uid, 1_000_049);
    let searches: Vec<String> = state
        .lock()
        .unwrap()
        .commands
        .iter()
        .filter(|c| c.contains("SEARCH"))
        .cloned()
        .collect();
    assert!(searches.iter().all(|c| !c.contains(":*")), "{searches:?}");

    // Sparse UIDs (mail moved away): empty spans are skipped in one poll.
    let state = fake::shared();
    state.lock().unwrap().uidnext = 4_500;
    let to = format!("booking+{}@ilia.ae", "x");
    let uid = state
        .lock()
        .unwrap()
        .deliver(&mail("guest@test.com", &to, "hi", "", "late"), false);
    sqlx::query("DELETE FROM inbox_state")
        .execute(&pool)
        .await
        .unwrap();
    crate::inbox::store::save_state(
        &pool,
        "INBOX",
        crate::inbox::store::State {
            uidvalidity: 1000,
            last_uid: 10,
        },
    )
    .await;
    let port = fake::serve(state.clone()).await;
    let stats = poll(&ctx(&pool, Config::for_tests(INBOX, true)), port).await;
    assert_eq!(count(&stats, "unmatched"), 1, "{stats:?}");
    let saved = crate::inbox::store::state(&pool, "INBOX").await.unwrap();
    assert_eq!(saved.last_uid, uid);
}

/// The per-sender limit counts matched mail of one booking only: tokenless
/// mail with the guest's From never uses up the guest's quota, and what is
/// over the limit stays unread for a person.
#[tokio::test]
async fn the_sender_limit_counts_matched_mail_and_leaves_the_rest_unread() {
    let _relay = relay(true);
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let to = format!("booking+{}@ilia.ae", token_of(&id));
    let state = fake::shared();
    let (junk, real) = {
        let mut st = state.lock().unwrap();
        let junk: Vec<u32> = (0..25)
            .map(|n| {
                st.deliver(
                    &mail("guest@test.com", INBOX, "spam", "", &format!("junk {n}")),
                    false,
                )
            })
            .collect();
        let real: Vec<u32> = (0..21)
            .map(|n| {
                st.deliver(
                    &mail("guest@test.com", &to, "Re", "", &format!("reply {n}")),
                    false,
                )
            })
            .collect();
        (junk, real)
    };
    let port = fake::serve(state.clone()).await;
    let c = ctx(&pool, Config::for_tests(INBOX, true));
    let stats = poll(&c, port).await;
    assert_eq!(count(&stats, "unmatched"), 25, "{stats:?}");
    assert_eq!(count(&stats, "stored"), 20, "{stats:?}");
    assert_eq!(count(&stats, "sender_limit"), 1, "{stats:?}");
    assert_eq!(thread_rows(&pool, &id).await.len(), 20);
    assert!(junk.iter().all(|u| !seen(&state, *u)));
    assert!(real[..20].iter().all(|u| seen(&state, *u)));
    assert!(!seen(&state, real[20]), "over the limit: left unread");
}

/// A SELECT refused (a folder that does not exist) is the service's
/// problem, logged as such, with the normal back-off, never as a refused
/// login.
#[tokio::test]
async fn a_refused_select_is_not_a_refused_login() {
    let _relay = relay(true);
    let (_app, pool, _session, _et_id) = setup_test_app().await;
    let state = fake::shared();
    state.lock().unwrap().refuse_select = true;
    let port = fake::serve(state.clone()).await;
    let log = LogCapture::default();
    let _guard = log.install();
    let c = ctx(&pool, Config::for_tests(INBOX, true));
    run_for(c, fake::plain_connector(port), fast(40), 700).await;
    let all = log.all();
    assert!(
        all.contains("inbox folder could not be selected (service)"),
        "{all}"
    );
    assert!(!all.contains("inbox login refused"), "{all}");
    // The normal back-off (300 ms here), not the hour of a refused login.
    assert!(state.lock().unwrap().logins >= 2);
}

/// The first run saves its place only once all the unseen mail is done:
/// a session that breaks half-way takes the rest next time.
#[tokio::test]
async fn a_first_run_broken_half_way_takes_the_rest_next_time() {
    let _relay = relay(true);
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let to = format!("booking+{}@ilia.ae", token_of(&id));
    let state = fake::shared();
    let uids: Vec<u32> = {
        let mut st = state.lock().unwrap();
        (0..3)
            .map(|n| {
                st.deliver(
                    &mail("guest@test.com", &to, "Re", "", &format!("reply {n}")),
                    false,
                )
            })
            .collect()
    };
    // The connection drops when the first message is marked.
    state.lock().unwrap().drop_at = Some("UID STORE");
    let port = fake::serve(state.clone()).await;
    let c = ctx(&pool, Config::for_tests(INBOX, true));
    {
        let s = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .unwrap();
        let mut client = Client::new(s);
        client.greeting().await.unwrap();
        client.capability().await.unwrap();
        client.login(USER, PASSWORD).await.unwrap();
        let sel = client.select("INBOX").await.unwrap();
        assert!(poller::poll_once(&mut client, &c, &Policy::default(), sel)
            .await
            .is_err());
    }
    assert_eq!(crate::inbox::store::state(&pool, "INBOX").await, None);
    let stats = poll(&c, port).await;
    assert_eq!(count(&stats, "stored"), 2, "{stats:?}");
    assert_eq!(count(&stats, "duplicate"), 1, "{stats:?}");
    assert!(uids.iter().all(|u| seen(&state, *u)));
    assert_eq!(thread_rows(&pool, &id).await.len(), 3);
    let saved = crate::inbox::store::state(&pool, "INBOX").await.unwrap();
    assert_eq!(saved.last_uid, *uids.last().unwrap());
}

/// A booking whose first invite named the host (made before guest mail
/// went through the system) keeps the host as ORGANIZER in every later
/// guest invite and CANCEL; a new booking names the system.
#[tokio::test]
async fn a_booking_made_before_the_cutover_keeps_the_host_as_organizer() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let _relay = relay(true);
    let (app, pool, _session, et_id) = setup_test_app().await;
    let (mta, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, mta).await;
    let old = insert_v1_booking(
        &pool,
        &et_id,
        "old-guest@test.com",
        Utc::now() + Duration::days(3),
        30,
        "confirmed",
        &[("language", Some("en".into()))],
    )
    .await;
    let new = booking(&pool, &et_id, 96, "confirmed").await;
    // What the migration does to every booking it finds.
    sqlx::query("UPDATE bookings SET guest_organizer = 'host' WHERE id = ?")
        .bind(&old)
        .execute(&pool)
        .await
        .unwrap();
    crate::inbox::organizer::load(&pool).await;
    let details = |uid: String| crate::email::BookingDetails {
        uid,
        host_name: "Test User".into(),
        host_email: "test@example.com".into(),
        guest_name: "Guest".into(),
        guest_email: "guest@test.com".into(),
        utc_times: Some(("20301010T100000Z".into(), "20301010T103000Z".into())),
        ..Default::default()
    };
    let ics = crate::email::generate_ics(&details(format!("uid-{old}")), "REQUEST");
    assert!(
        ics.contains("ORGANIZER;CN=Test User:mailto:test@example.com"),
        "{ics}"
    );
    let ics = crate::email::generate_ics(&details(format!("uid-{new}")), "REQUEST");
    assert!(
        ics.contains(&format!(
            "ORGANIZER;CN=Test User:mailto:booking+{}@ilia.ae",
            token_of(&new)
        )),
        "{ics}"
    );
    // The guest's CANCEL of the old booking names the host too.
    let cancel: String = sqlx::query_scalar("SELECT cancel_token FROM bookings WHERE id = ?")
        .bind(&old)
        .fetch_one(&pool)
        .await
        .unwrap();
    let csrf = "csrf-cutover";
    let response = app
        .oneshot(post_form_unauthed(
            &format!("/booking/cancel/{cancel}"),
            csrf,
            &format!("_csrf={csrf}"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let sent = mail_to(&mails, "old-guest@test.com").await;
    drain_emails(std::time::Duration::from_secs(10)).await;
    assert!(sent.contains("METHOD:CANCEL"), "{sent}");
    assert!(
        sent.contains("ORGANIZER;CN=Test User:mailto:test@example.com"),
        "{sent}"
    );
}

/// Every message that reaches a guest or an attendee in a real build (relay
/// mode) — confirmation and attendee copy, the host's reschedule request,
/// the moved booking, the guest's and the host's cancellations, the TON
/// mails — comes from the system, answers to the booking's system address,
/// carries the booking's token in its Message-ID, and names neither of the
/// owner's own addresses anywhere, `.ics` included.
#[tokio::test]
async fn in_relay_mode_no_guest_mail_carries_the_owners_address() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let _relay = relay(true);
    let (app, pool, session, et_id) = setup_test_app().await;
    const PERSONAL: &str = "owner.private@personal.example";
    sqlx::query("UPDATE users SET booking_email = ? WHERE email = 'test@example.com'")
        .bind(PERSONAL)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE event_types SET max_additional_guests = 3 WHERE id = ?")
        .bind(&et_id)
        .execute(&pool)
        .await
        .unwrap();
    let (mta, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, mta).await;

    // (1) A booking with an attendee: confirmation and attendee copy.
    let csrf = "csrf-relay-book";
    let form = format!(
        "_csrf={csrf}&date={}&time=10%3A00&name=Guest&email=relay-guest%40test.com&notes=&additional_guests=relay-friend%40test.com",
        far_future_monday()
    );
    let response = app
        .clone()
        .oneshot(post_form_unauthed(
            "/u/testuser/test-meeting/book",
            csrf,
            &form,
        ))
        .await
        .unwrap();
    assert!(response.status() == 200 || response.status().is_redirection());
    mail_to(&mails, "relay-friend@test.com").await;
    let (id, uid): (String, String) =
        sqlx::query_as("SELECT id, uid FROM bookings WHERE guest_email = 'relay-guest@test.com'")
            .fetch_one(&pool)
            .await
            .unwrap();
    // (2) The host asks the guest to pick another time.
    let csrf = "csrf-relay-hresched";
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
    let location = response
        .headers()
        .get("location")
        .and_then(|l| l.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(!location.contains("error"), "{location}");
    drain_emails(std::time::Duration::from_secs(10)).await;
    // The request gave the booking a new reschedule token.
    let reschedule: String =
        sqlx::query_scalar("SELECT reschedule_token FROM bookings WHERE id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
    // (3) The guest moves it.
    let csrf = "csrf-relay-gresched";
    let response = app
        .clone()
        .oneshot(post_form_unauthed(
            &format!("/booking/reschedule/{reschedule}"),
            csrf,
            &format!(
                "_csrf={csrf}&date={}&time=11%3A00&tz=UTC",
                far_future_monday()
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    drain_emails(std::time::Duration::from_secs(10)).await;
    // (4) The guest cancels (the move gave the booking new tokens).
    let cancel: String = sqlx::query_scalar("SELECT cancel_token FROM bookings WHERE id = ?")
        .bind(&id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let csrf = "csrf-relay-gcancel";
    let response = app
        .clone()
        .oneshot(post_form_unauthed(
            &format!("/booking/cancel/{cancel}"),
            csrf,
            &format!("_csrf={csrf}"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    drain_emails(std::time::Duration::from_secs(10)).await;
    // (5) The host cancels another booking.
    let other = insert_v1_booking(
        &pool,
        &et_id,
        "relay-other@test.com",
        Utc::now() + Duration::days(5),
        30,
        "confirmed",
        &[("language", Some("en".into()))],
    )
    .await;
    let csrf = "csrf-relay-hcancel";
    let response = app
        .clone()
        .oneshot(post_form(
            &format!("/dashboard/bookings/{other}/cancel"),
            &session,
            csrf,
            &format!("_csrf={csrf}&reason=Sorry"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection() || response.status() == 200);
    drain_emails(std::time::Duration::from_secs(10)).await;
    // (6) The TON mails, as the payment flow sends them.
    let smtp = crate::email::load_smtp_config(&pool, &[0u8; 32])
        .await
        .unwrap()
        .unwrap();
    let ton = crate::email::TonGuestMail {
        lang: "en".into(),
        guest_name: "Guest".into(),
        guest_email: "relay-ton@test.com".into(),
        host_name: "Test User".into(),
        host_email: PERSONAL.into(),
        booking_uid: uid.clone(),
        event_title: "Test Meeting".into(),
        date: "Monday".into(),
        time: "10:00 (UTC)".into(),
        pay_by: "10:30 (UTC)".into(),
    };
    // The host's "pick a new time" email (the route sends it only with a
    // public base URL, which tests do not set), as the route builds it.
    let pick = crate::email::BookingDetails {
        uid: uid.clone(),
        event_title: "Test Meeting".into(),
        guest_name: "Guest".into(),
        guest_email: "relay-guest@test.com".into(),
        guest_timezone: "UTC".into(),
        host_name: "Test User".into(),
        host_email: PERSONAL.into(),
        guest_language: Some("en".into()),
        utc_times: Some(("20301010T100000Z".into(), "20301010T103000Z".into())),
        ..Default::default()
    };
    crate::email::send_guest_pick_new_time(&smtp, &pick, "https://book.example/r", None)
        .await
        .unwrap();
    crate::email::send_guest_ton_pay_link(&smtp, &ton, "https://pay.example/p", None, None)
        .await
        .unwrap();
    crate::email::send_guest_ton_renewed(&smtp, &ton, "https://pay.example/p", None, None)
        .await
        .unwrap();
    crate::email::send_guest_ton_released(
        &smtp,
        &ton,
        crate::email::TonReleased::Expired,
        None,
        false,
    )
    .await
    .unwrap();
    drain_emails(std::time::Duration::from_secs(10)).await;

    let token = reply::token(&uid).unwrap();
    let other_uid: String = sqlx::query_scalar("SELECT uid FROM bookings WHERE id = ?")
        .bind(&other)
        .fetch_one(&pool)
        .await
        .unwrap();
    let other_token = reply::token(&other_uid).unwrap();
    let captured: Vec<(Vec<String>, String)> = mails
        .lock()
        .unwrap()
        .iter()
        .map(|m| (m.rcpt.clone(), super::decode_quoted_printable(&m.data)))
        .collect();
    let guest_side = [
        "relay-guest@test.com",
        "relay-friend@test.com",
        "relay-other@test.com",
        "relay-ton@test.com",
    ];
    let mut checked = std::collections::BTreeMap::<&str, usize>::new();
    for (rcpt, data) in &captured {
        let Some(to) = guest_side
            .iter()
            .find(|g| rcpt.iter().any(|r| r.contains(&format!("<{g}>"))))
        else {
            continue;
        };
        *checked.entry(to).or_default() += 1;
        let t = if *to == "relay-other@test.com" {
            &other_token
        } else {
            &token
        };
        let reply_to = format!("booking+{t}@ilia.ae");
        assert_eq!(
            mail_header(data, "Reply-To"),
            Some(reply_to.as_str()),
            "{to}: {data}"
        );
        let message_id = mail_header(data, "Message-ID").unwrap_or_default();
        assert!(
            message_id.starts_with(&format!("<calrs.{t}.")),
            "{to}: {message_id}"
        );
        assert!(message_id.ends_with("@ilia.ae>"), "{to}: {message_id}");
        assert_eq!(
            mail_header(data, "X-Calrs-Origin"),
            Some("1"),
            "{to}: {data}"
        );
        assert!(
            mail_header(data, "From")
                .unwrap_or_default()
                .contains("<noreply@test.com>"),
            "{to}: {data}"
        );
        for personal in [PERSONAL, "test@example.com"] {
            assert!(!data.contains(personal), "{to} sees {personal}: {data}");
        }
        if data.contains("BEGIN:VCALENDAR") {
            assert!(
                data.contains(&format!("ORGANIZER;CN=Test User:mailto:{reply_to}")),
                "{to}: {data}"
            );
        }
    }
    // The guest: confirmation, moved, cancelled, "pick a new time"; the
    // attendee: copy, move, cancel; the host's cancel; three TON mails.
    let subjects: Vec<String> = captured
        .iter()
        .map(|(r, d)| format!("{r:?} {}", mail_header(d, "Subject").unwrap_or_default()))
        .collect();
    assert_eq!(
        checked.get("relay-guest@test.com").copied(),
        Some(4),
        "{checked:?} {subjects:#?}"
    );
    assert_eq!(
        checked.get("relay-friend@test.com").copied(),
        Some(3),
        "{checked:?} {subjects:#?}"
    );
    assert_eq!(
        checked.get("relay-other@test.com").copied(),
        Some(1),
        "{checked:?}"
    );
    assert_eq!(
        checked.get("relay-ton@test.com").copied(),
        Some(3),
        "{checked:?}"
    );
}

/// Shared mode: the system mailbox is also the owner's own (iCloud+
/// `mail@ilia.ae`, ilia-booking docs/decisions.md, 2026-11-11).
fn shared_config() -> Config {
    let mut config = Config::for_tests(INBOX, false);
    config.shared = true;
    config
}

/// The commands the fake recorded for `uid`: a full fetch, or any flag,
/// move or copy.
fn touched(state: &fake::Shared, uid: u32) -> Vec<String> {
    let id = format!(" {uid} ");
    state
        .lock()
        .unwrap()
        .commands
        .iter()
        .filter(|c| {
            let c = format!("{c} ");
            c.contains(&id)
                && (c.contains("BODY.PEEK[]")
                    || c.starts_with("UID STORE")
                    || c.starts_with("UID MOVE")
                    || c.starts_with("UID COPY")
                    || c.starts_with("UID EXPUNGE"))
        })
        .cloned()
        .collect()
}

#[tokio::test]
async fn a_shared_mailbox_reads_only_replies_to_calrs_mail() {
    let _relay = test_mode(true, Some(shared_config()));
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let token = token_of(&id);
    let reference = format!("In-Reply-To: <calrs.{token}.00aa11bb22cc33dd@ilia.ae>\r\n");
    let state = fake::shared();
    state.lock().unwrap().caps = vec!["MOVE"];
    // The owner's history, a guest's unread reply among it: never searched.
    let history: Vec<u32> = {
        let mut st = state.lock().unwrap();
        vec![
            st.deliver(
                &mail("bank@bank.example", INBOX, "Your OTP", "", "123456"),
                false,
            ),
            st.deliver(
                &mail("guest@test.com", INBOX, "Re: hi", &reference, "Old reply"),
                false,
            ),
        ]
    };
    let mut config = shared_config();
    config.done_folder = Some("calrs".into());
    let c = ctx(&pool, config);
    let port = fake::serve(state.clone()).await;
    assert!(poll(&c, port).await.0.is_empty());
    let saved = crate::inbox::store::state(&pool, "INBOX").await.unwrap();
    assert_eq!(saved.last_uid, 2);
    {
        let st = state.lock().unwrap();
        assert_eq!(st.count("UID SEARCH"), 0, "{:?}", st.commands);
        assert_eq!(st.count("UID FETCH"), 0, "{:?}", st.commands);
    }

    let (private, read, news, away, dsn, tag, reply, auto_reply) = {
        let mut st = state.lock().unwrap();
        let mut d = |raw: String, seen: bool| st.deliver(&raw, seen);
        (
            d(
                mail("friend@friend.example", INBOX, "Dinner", "", "PRIVATE-TEXT"),
                false,
            ),
            d(
                mail("friend@friend.example", INBOX, "Photos", "", "Already read"),
                true,
            ),
            d(
                mail(
                    "news@shop.example",
                    INBOX,
                    "Sale",
                    "List-Id: <news.shop.example>\r\nList-Unsubscribe: <https://shop.example/u>\r\n",
                    "Buy",
                ),
                false,
            ),
            d(
                mail(
                    "colleague@corp.example",
                    INBOX,
                    "Out of office",
                    "Auto-Submitted: auto-replied\r\n",
                    "Away",
                ),
                false,
            ),
            d(
                format!(
                    "From: MAILER-DAEMON@mx.example\r\nTo: {INBOX}\r\nSubject: Undelivered\r\n\
                     Content-Type: multipart/report; report-type=delivery-status; boundary=\"b\"\r\n\r\n\
                     --b\r\nContent-Type: text/plain\r\n\r\nfailed\r\n--b--\r\n"
                ),
                false,
            ),
            // A subject tag from a stranger: token-shaped, still not a match.
            d(
                mail("stranger@q.example", INBOX, &format!("Re: [#{token}]"), "", "Spoofed"),
                false,
            ),
            d(
                mail("guest@test.com", INBOX, "Re: hi", &reference, "See you Tuesday."),
                false,
            ),
            // A matched automatic reply: handled as today.
            d(
                mail(
                    "guest@test.com",
                    INBOX,
                    "Out of office",
                    &format!("{reference}Auto-Submitted: auto-replied\r\n"),
                    "Away",
                ),
                false,
            ),
        )
    };
    let log = LogCapture::default();
    let stats = {
        let _guard = log.install();
        poll(&c, port).await
    };
    assert_eq!(count(&stats, "stored"), 1, "{stats:?}");
    assert_eq!(count(&stats, "autoreply"), 1, "{stats:?}");
    assert_eq!(count(&stats, "unmatched"), 6, "{stats:?}");
    assert_eq!(
        thread_rows(&pool, &id)
            .await
            .into_iter()
            .map(|r| r.2)
            .collect::<Vec<_>>(),
        ["See you Tuesday."]
    );
    let st = state.lock().unwrap();
    // Only the two matched messages were moved to the done folder.
    assert_eq!(
        st.moved,
        vec![
            (reply, "calrs".to_string()),
            (auto_reply, "calrs".to_string())
        ]
    );
    drop(st);
    for uid in history
        .iter()
        .copied()
        .chain([private, read, news, away, dsn, tag])
    {
        assert!(
            touched(&state, uid).is_empty(),
            "{uid}: {:?}",
            touched(&state, uid)
        );
    }
    // Their flags are as the owner left them.
    for uid in history
        .iter()
        .copied()
        .chain([private, news, away, dsn, tag])
    {
        assert_eq!(state.lock().unwrap().flags_of(uid), Some(vec![]), "{uid}");
    }
    assert_eq!(
        state.lock().unwrap().flags_of(read),
        Some(vec!["\\Seen".to_string()])
    );
    // Each new message was asked for its size and the match headers only;
    // the full body only for the two that matched.
    let st = state.lock().unwrap();
    let fetches: Vec<&String> = st
        .commands
        .iter()
        .filter(|c| c.starts_with("UID FETCH"))
        .collect();
    let headers = fetches
        .iter()
        .filter(|c| c.contains("BODY.PEEK[HEADER.FIELDS (FROM TO CC "))
        .count();
    let full = fetches.iter().filter(|c| c.contains("BODY.PEEK[]")).count();
    assert_eq!((headers, full, fetches.len()), (8, 2, 10), "{fetches:#?}");
    drop(st);
    // Nothing of the private mail reached the log: no UID, no address, no
    // subject, no text.
    let all = log.all();
    for uid in [private, read, news, away, dsn, tag] {
        assert!(!all.contains(&format!("uid={uid} ")), "{all}");
    }
    for secret in [
        "friend@",
        "PRIVATE-TEXT",
        "Dinner",
        "news@shop",
        "Sale",
        "stranger@",
    ] {
        assert!(!all.contains(secret), "{secret}: {all}");
    }
}

#[tokio::test]
async fn a_shared_mailbox_starts_over_from_uidnext_on_a_new_uidvalidity() {
    let _relay = test_mode(true, Some(shared_config()));
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    let reference = format!(
        "References: <a@gmail.com>\r\n <calrs.{}.00aa@ilia.ae>\r\n",
        token_of(&id)
    );
    let state = fake::shared();
    let port = fake::serve(state.clone()).await;
    let c = ctx(&pool, shared_config());
    assert!(poll(&c, port).await.0.is_empty());
    // A folded References header is matched whole.
    let first = state.lock().unwrap().deliver(
        &mail("guest@test.com", INBOX, "Re: hi", &reference, "First"),
        false,
    );
    assert_eq!(count(&poll(&c, port).await, "stored"), 1);
    assert!(seen(&state, first));
    {
        let mut st = state.lock().unwrap();
        st.uidvalidity = 2000;
        st.deliver(
            &mail(
                "guest@test.com",
                INBOX,
                "Re: hi",
                &reference,
                "Before the reset",
            ),
            false,
        );
    }
    assert!(poll(&c, port).await.0.is_empty());
    state.lock().unwrap().deliver(
        &mail(
            "guest@test.com",
            INBOX,
            "Re: hi",
            &reference,
            "After the reset",
        ),
        false,
    );
    assert_eq!(count(&poll(&c, port).await, "stored"), 1);
    let bodies: Vec<String> = thread_rows(&pool, &id)
        .await
        .into_iter()
        .map(|r| r.2)
        .collect();
    assert_eq!(bodies, ["First", "After the reset"]);
}

/// Off (the default), the same unrelated mail is handled as before: the
/// first run takes the unseen history and the filters mark a newsletter.
#[tokio::test]
async fn without_shared_mode_unrelated_automatic_mail_is_still_marked() {
    let _relay = relay(false);
    let (_app, pool, _session, _et_id) = setup_test_app().await;
    let state = fake::shared();
    let (news, private) = {
        let mut st = state.lock().unwrap();
        (
            st.deliver(
                &mail(
                    "news@shop.example",
                    INBOX,
                    "Sale",
                    "List-Id: <news.shop.example>\r\n",
                    "Buy",
                ),
                false,
            ),
            st.deliver(
                &mail("friend@friend.example", INBOX, "Dinner", "", "Hi"),
                false,
            ),
        )
    };
    let port = fake::serve(state.clone()).await;
    let stats = poll(&ctx(&pool, Config::for_tests(INBOX, false)), port).await;
    assert_eq!(
        (count(&stats, "list"), count(&stats, "unmatched")),
        (1, 1),
        "{stats:?}"
    );
    assert!(seen(&state, news));
    assert!(!seen(&state, private));
    // Read in full, and no header-only fetch.
    let st = state.lock().unwrap();
    assert!(!st.commands.iter().any(|c| c.contains("HEADER.FIELDS")));
    assert_eq!(
        st.commands
            .iter()
            .filter(|c| c.contains("BODY.PEEK[]"))
            .count(),
        2
    );
}

/// Shared mode: the mailbox's own person writes from it too. A reply to
/// them from someone they copied in, or a forward, still carries our
/// Message-ID, and calrs's own mail can land in it: none of it is a guest
/// writing, and all of it stays as the person left it.
#[tokio::test]
async fn a_shared_mailbox_leaves_others_replies_and_its_own_mail_alone() {
    let _relay = test_mode(true, Some(shared_config()));
    let (_app, pool, _session, et_id) = setup_test_app().await;
    let id = booking(&pool, &et_id, 48, "confirmed").await;
    // A test booking whose guest is the mailbox itself.
    let own = booking(&pool, &et_id, 72, "confirmed").await;
    sqlx::query("UPDATE bookings SET guest_email = ? WHERE id = ?")
        .bind(INBOX)
        .bind(&own)
        .execute(&pool)
        .await
        .unwrap();
    let token = token_of(&id);
    let ours = format!("<calrs.{token}.00aa11bb22cc33dd@ilia.ae>");
    let state = fake::shared();
    state.lock().unwrap().caps = vec!["MOVE"];
    let mut config = shared_config();
    config.done_folder = Some("calrs".into());
    let c = ctx(&pool, config);
    let port = fake::serve(state.clone()).await;
    assert!(poll(&c, port).await.0.is_empty());

    let (partner, forward, copy, reply) = {
        let mut st = state.lock().unwrap();
        let mut d = |raw: String| st.deliver(&raw, false);
        (
            // The owner answered the guest and copied in a partner, who
            // pressed Reply: to the owner alone, our Message-ID inherited.
            d(mail(
                "partner@p.example",
                INBOX,
                "Re: Re: hi",
                &format!("In-Reply-To: <owner-answer@ilia.ae>\r\nReferences: {ours}\r\n <owner-answer@ilia.ae>\r\n"),
                "PRIVATE-NOTE for the owner",
            )),
            // A forward that keeps References.
            d(mail(
                "colleague@corp.example",
                INBOX,
                "Fwd: hi",
                &format!("References: {ours}\r\n"),
                "Forwarded",
            )),
            // calrs's own message to the guest that is the mailbox.
            d(mail(
                INBOX,
                INBOX,
                &format!("Message from Test User [#{}]", token_of(&own)),
                "X-Calrs-Origin: host-message\r\n",
                "Our own text",
            )),
            d(mail(
                "guest@test.com",
                INBOX,
                "Re: hi",
                &format!("In-Reply-To: {ours}\r\n"),
                "See you Tuesday.",
            )),
        )
    };
    let log = LogCapture::default();
    let stats = {
        let _guard = log.install();
        poll(&c, port).await
    };
    assert_eq!(
        (
            count(&stats, "stored"),
            count(&stats, "unmatched"),
            count(&stats, "loop")
        ),
        (1, 2, 1),
        "{stats:?}"
    );
    let bodies = |rows: Vec<(String, String, String, String, String)>| {
        rows.into_iter().map(|r| r.2).collect::<Vec<_>>()
    };
    assert_eq!(bodies(thread_rows(&pool, &id).await), ["See you Tuesday."]);
    assert!(bodies(thread_rows(&pool, &own).await).is_empty());
    // The other people's mail is not even read in full.
    for uid in [partner, forward] {
        assert!(
            touched(&state, uid).is_empty(),
            "{uid}: {:?}",
            touched(&state, uid)
        );
    }
    // Our own mail is read (it matched), but neither flagged nor moved.
    assert!(touched(&state, copy)
        .iter()
        .all(|c| !c.starts_with("UID STORE") && !c.starts_with("UID MOVE")));
    for uid in [partner, forward, copy] {
        assert_eq!(state.lock().unwrap().flags_of(uid), Some(vec![]), "{uid}");
    }
    assert_eq!(
        state.lock().unwrap().moved,
        vec![(reply, "calrs".to_string())]
    );
    let all = log.all();
    for secret in ["partner@", "PRIVATE-NOTE", "colleague@", "Our own text"] {
        assert!(!all.contains(secret), "{secret}: {all}");
    }
}
