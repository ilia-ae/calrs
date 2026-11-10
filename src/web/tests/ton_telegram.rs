//! ilia fork: TON payments told through Telegram (the seam is
//! `crate::telegram::TelegramEvent`, the bridge `web::ton_pay::telegram`).
//! A child of the TON tests, for their chain, rates and booking helpers; the
//! bot is `crate::telegram::fake::FakeApi`: nothing reaches Telegram.

use super::*;
use crate::telegram::api::Outgoing;
use crate::telegram::commands::handle_update;
use crate::telegram::fake::{self, Installed};
use crate::web::ton_pay::telegram::{link_token, TelegramSink};

const BASE: &str = "https://book.example.test";
const GUEST_CHAT: i64 = 987654321;
const HOST_CHAT: i64 = 555000111;

/// Telegram on with a fake bot, an HTTPS base URL (buttons), the event
/// type offering Telegram to guests, and the host's chat linked.
struct Tg {
    tg: Installed,
    e: Env,
    /// Alerts already handed to the Telegram sink.
    relayed: std::sync::atomic::AtomicUsize,
    _env: crate::test_support::EnvGuard,
    _lock: tokio::sync::MutexGuard<'static, ()>,
}

async fn tg_env() -> Tg {
    tg_env_with(&[], None).await
}

/// [`tg_env`] with TON settings and, with `ton`, a price in TON.
async fn tg_env_with(extra: &[(&'static str, &str)], ton: Option<u128>) -> Tg {
    let tg = fake::install_fake("ilia_test_bot").await;
    let lock = crate::test_support::ENV_LOCK.lock().await;
    let guard = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", BASE);
    let e = env_priced(extra, ton).await;
    crate::telegram::store::save_event_type_mode(&e.pool, &e.et_id, "optional")
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO telegram_user_chats (user_id, chat_id, telegram_username)
         SELECT id, ?, 'host_tg' FROM users WHERE email = 'test@example.com'",
    )
    .bind(HOST_CHAT)
    .execute(&e.pool)
    .await
    .unwrap();
    Tg {
        tg,
        e,
        relayed: std::sync::atomic::AtomicUsize::new(0),
        _env: guard,
        _lock: lock,
    }
}

impl Tg {
    /// A watcher step, then the alerts it claimed through the Telegram sink
    /// (`calrs serve` fans each one out to email and Telegram), then wait
    /// for the outbox.
    async fn step(&self, now: chrono::DateTime<Utc>) -> watcher::StepReport {
        let r = self.e.step(now).await;
        self.relay().await;
        r
    }

    async fn relay(&self) {
        let all = self.e.sink.all();
        let from = self
            .relayed
            .swap(all.len(), std::sync::atomic::Ordering::SeqCst);
        let sink = TelegramSink::new(self.e.pool.clone());
        for alert in all.into_iter().skip(from) {
            sink.deliver(alert).await;
        }
        self.tg.flush(&self.e.pool).await;
    }

    /// Book through the web form with a Telegram username: (booking id,
    /// pay token).
    async fn book(&self, time: &str) -> (String, String) {
        let body = format!(
            "_csrf={CSRF}&date={}&time={time}&name=Guest+Name&email=guest%40test.com&tz=UTC&telegram=guest_tg",
            far_future_monday()
        );
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/u/testuser/test-meeting/book")
            .header("cookie", format!("__Host-calrs_csrf={CSRF}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .header("x-forwarded-for", "198.51.100.7")
            .body(Body::from(body))
            .unwrap();
        let resp = self.e.app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
        let token = location(&resp)
            .strip_prefix("/booking/pay/")
            .unwrap()
            .to_string();
        let bid: String =
            sqlx::query_scalar("SELECT booking_id FROM ton_invoices WHERE pay_token = ?")
                .bind(&token)
                .fetch_one(&self.e.pool)
                .await
                .unwrap();
        self.tg.flush(&self.e.pool).await;
        (bid, token)
    }

    /// `/start <token>` from the guest's private chat.
    async fn start(&self, token: &str) {
        let update =
            fake::private_text(1, GUEST_CHAT, Some("guest_tg"), &format!("/start {token}"));
        handle_update(&self.tg.rt, &self.e.pool, &update).await;
        self.tg.flush(&self.e.pool).await;
    }

    fn to(&self, chat: i64) -> Vec<Outgoing> {
        self.tg
            .sent()
            .into_iter()
            .filter(|(c, _)| *c == chat)
            .map(|(_, m)| m)
            .collect()
    }
}

/// The deep link the payment page shows.
fn deep_link_in(page: &str) -> String {
    let marker = "https://t.me/ilia_test_bot?start=b_";
    let at = page
        .find(marker)
        .unwrap_or_else(|| panic!("no deep link in the page"));
    page[at..at + marker.len() - 2 + 45].to_string()
}

fn labels(m: &Outgoing) -> Vec<&str> {
    m.buttons.iter().map(|b| b.label.as_str()).collect()
}

/// No host message ever carries the payment link, its token or the
/// comment code.
fn assert_host_safe(host: &[Outgoing], pay_token: &str, reference: &str) {
    for m in host {
        let all = format!("{} {:?}", m.text, m.buttons);
        assert!(!all.contains(pay_token), "{all}");
        assert!(!all.contains("/booking/pay/"), "{all}");
        assert!(!all.contains(reference), "{all}");
        assert!(!all.contains(&code(reference)), "{all}");
        for b in &m.buttons {
            assert_eq!(b.url, format!("{BASE}/dashboard/bookings"));
        }
    }
}

#[tokio::test]
async fn the_payment_page_offers_telegram_and_the_guest_hears_each_payment_step() {
    let t = tg_env().await;
    let e = &t.e;
    let (bid, pay_token) = t.book("10:00").await;
    // Nothing to approve: no host message at creation.
    assert!(t.to(HOST_CHAT).is_empty(), "{:#?}", t.tg.sent());

    // The payment page shows the booking's deep link, the same on every
    // load (it refreshes itself), derived from the payment link.
    let (_, _, page) = e.get_page(&format!("/booking/pay/{pay_token}")).await;
    let link = deep_link_in(&page);
    let (_, _, again) = e.get_page(&format!("/booking/pay/{pay_token}")).await;
    assert_eq!(deep_link_in(&again), link);
    let token = link.rsplit('=').next().unwrap().to_string();
    assert_eq!(token, link_token(&[0u8; 32], &pay_token));
    let stored: String =
        sqlx::query_scalar("SELECT link_token_hash FROM telegram_guest_links WHERE booking_id = ?")
            .bind(&bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!(stored, crate::telegram::tokens::hash(&token));
    assert!(!page.contains(&stored));

    // Linking: the welcome says what to pay, by when, with the payment page.
    t.start(&token).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 1, "{guest:#?}");
    let due = &guest[0];
    assert!(due.text.starts_with("Done: notifications about this booking will come here.\n\nYour booking awaits payment: Test Meeting, "), "{}", due.text);
    assert!(
        due.text.contains("Amount: 20.00 TON (≈ $40.00). Pay by "),
        "{}",
        due.text
    );
    assert_eq!(labels(due), ["Pay", "Cancel booking"]);
    assert_eq!(
        due.buttons[0].url,
        format!("{BASE}/booking/pay/{pay_token}")
    );
    // The hosts hear nothing about an unpaid booking, not even the link.
    assert!(t.to(HOST_CHAT).is_empty(), "{:#?}", t.to(HOST_CHAT));

    // Part of it: what is left, and by when.
    let reference = e.inv(&bid).await.reference;
    e.pay(AMOUNT / 4, &code(&reference));
    t.step(e.now).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 2, "{guest:#?}");
    assert!(
        guest[1]
            .text
            .starts_with("Part of the payment for Test Meeting, "),
        "{}",
        guest[1].text
    );
    assert!(
        guest[1]
            .text
            .contains("has arrived: 5.00 of 20.00 TON. Pay the remaining 15.00 TON by "),
        "{}",
        guest[1].text
    );
    assert_eq!(labels(&guest[1]), ["Pay", "Cancel booking"]);
    assert!(t.to(HOST_CHAT).is_empty(), "{:#?}", t.to(HOST_CHAT));

    // The rest: one "payment received, confirmed" to the guest (not the
    // ordinary "approved" as well), "paid" to the host.
    e.pay(AMOUNT - AMOUNT / 4, &code(&reference));
    let r = t.step(e.now).await;
    assert_eq!(r.confirmed, 1);
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 3, "{guest:#?}");
    assert!(
        guest[2]
            .text
            .starts_with("Payment received, your booking is confirmed: Test Meeting, "),
        "{}",
        guest[2].text
    );
    assert_eq!(
        labels(&guest[2]),
        ["Add to calendar", "Reschedule", "Cancel booking"]
    );
    let host = t.to(HOST_CHAT);
    assert_eq!(host.len(), 1, "{host:#?}");
    assert!(
        host[0].text.starts_with("Paid: Test Meeting, "),
        "{}",
        host[0].text
    );
    assert!(
        host[0]
            .text
            .contains("Guest: Guest Name, guest@test.com. Received 20.00 TON (≈ $40.00); the booking is confirmed."),
        "{}",
        host[0].text
    );
    // The hosts were not told when the guest linked (it awaited payment):
    // who holds the booking's chat comes with "paid".
    assert!(
        host[0].text.ends_with("\nGuest's Telegram: @guest_tg."),
        "{}",
        host[0].text
    );
    assert_host_safe(&host, &pay_token, &reference);

    // Once paid and linked, the page still offers the link (the meeting
    // has not happened), but a later watcher step says nothing again.
    t.step(e.now).await;
    assert_eq!(t.to(GUEST_CHAT).len(), 3);
    assert_eq!(t.to(HOST_CHAT).len(), 1);
}

#[tokio::test]
async fn a_late_payment_and_the_release_are_told_and_the_host_hears_the_problem() {
    let t = tg_env().await;
    let e = &t.e;
    let (bid, pay_token) = t.book("10:00").await;
    t.start(&link_token(&[0u8; 32], &pay_token)).await;
    let inv = e.inv(&bid).await;
    let settle_by = inv.settle_by;

    // A transfer after the window: not credited, the guest and the host
    // both hear it (the host: refund it).
    let late = e.pay_to(
        OUR,
        settle_by.timestamp() + 1,
        AMOUNT,
        &code(&inv.reference),
    );
    e.chain.push(late);
    e.chain
        .height((settle_by + Duration::minutes(5)).timestamp() as u64);
    t.step(settle_by + Duration::minutes(5)).await;
    assert_eq!(e.inv(&bid).await.status, "expired");
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 2, "{guest:#?}");
    assert!(
        guest[1]
            .text
            .starts_with("A payment of 20.00 TON for Test Meeting, "),
        "{}",
        guest[1].text
    );
    assert!(guest[1]
        .text
        .contains("arrived after the deadline, so it did not confirm the booking."));
    let host = t.to(HOST_CHAT);
    assert_eq!(host.len(), 1, "{host:#?}");
    assert!(
        host[0]
            .text
            .starts_with("Payment needs attention: Test Meeting, "),
        "{}",
        host[0].text
    );
    assert!(
        host[0].text.contains("\nTransfer: 20.00 TON.\n"),
        "{}",
        host[0].text
    );
    assert!(host[0]
        .text
        .ends_with(&crate::i18n::translate("en", "ton-alert-late-body", None)));

    // calrs's sweep declines it: the guest hears why (not upstream's "the
    // host did not reply"), the hosts nothing more.
    sweep_pending_requests(&e.pool, &[0u8; 32], settle_by + Duration::minutes(6)).await;
    t.tg.flush(&e.pool).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "declined");
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 3, "{guest:#?}");
    assert!(
        guest[2]
            .text
            .starts_with("The time to pay for Test Meeting, "),
        "{}",
        guest[2].text
    );
    assert!(guest[2]
        .text
        .ends_with("has run out, so the time is released. You can book another time."));
    assert_eq!(labels(&guest[2]), ["Book another time"]);
    let host = t.to(HOST_CHAT);
    assert_eq!(host.len(), 1, "{host:#?}");
    assert_host_safe(&host, &pay_token, &inv.reference);
    // The link ended with the booking: the page no longer offers it.
    let (_, _, page) = e.get_page(&format!("/booking/pay/{pay_token}")).await;
    assert!(!page.contains("https://t.me/"));

    // The code is on chain now: a stranger's transfer carrying it is late
    // too, but the guest is not told "your payment" arrived (the hosts
    // still hear it: there is money to return).
    let later = settle_by + Duration::hours(2);
    let mut stranger = e.pay_to(OUR, later.timestamp(), AMOUNT / 2, &code(&inv.reference));
    stranger["in_msg"]["source"] = serde_json::json!(SPAMMER);
    e.chain.push(stranger);
    e.chain
        .height((later + Duration::minutes(1)).timestamp() as u64);
    t.step(later + Duration::minutes(1)).await;
    assert_eq!(t.to(GUEST_CHAT).len(), 3, "{:#?}", t.to(GUEST_CHAT));
    let host = t.to(HOST_CHAT);
    assert_eq!(host.len(), 2, "{host:#?}");
    assert!(
        host[1].text.contains("\nTransfer: 10.00 TON.\n"),
        "{}",
        host[1].text
    );
    // Another one from the wallet that paid first: the guest's own.
    let again = settle_by + Duration::hours(4);
    let own = e.pay_to(OUR, again.timestamp(), AMOUNT / 4, &code(&inv.reference));
    e.chain.push(own);
    e.chain
        .height((again + Duration::minutes(1)).timestamp() as u64);
    t.step(again + Duration::minutes(1)).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 4, "{guest:#?}");
    assert!(
        guest[3]
            .text
            .starts_with("A payment of 5.00 TON for Test Meeting, "),
        "{}",
        guest[3].text
    );
    assert_eq!(t.to(HOST_CHAT).len(), 3);
}

#[tokio::test]
async fn an_underpaid_booking_tells_the_guest_of_the_refund_and_the_host_of_the_problem() {
    let t = tg_env().await;
    let e = &t.e;
    let (bid, pay_token) = t.book("10:00").await;
    t.start(&link_token(&[0u8; 32], &pay_token)).await;
    let inv = e.inv(&bid).await;
    e.pay(AMOUNT / 2, &code(&inv.reference));
    t.step(e.now).await;
    let settle_by = inv.settle_by;
    e.chain
        .height((settle_by + Duration::minutes(3)).timestamp() as u64);
    let at = settle_by + Duration::minutes(4);
    assert_eq!(t.step(at).await.expired, 1);
    // The owner's "partial" problem, to the host.
    let host = t.to(HOST_CHAT);
    assert_eq!(host.len(), 1, "{host:#?}");
    assert!(
        host[0].text.contains("\nReceived 10.00 of 20.00 TON.\n"),
        "{}",
        host[0].text
    );
    assert!(host[0]
        .text
        .ends_with(&crate::i18n::translate("en", "ton-problem-partial", None)));
    assert_host_safe(&host, &pay_token, &inv.reference);
    sweep_pending_requests(&e.pool, &[0u8; 32], at + Duration::seconds(30)).await;
    t.tg.flush(&e.pool).await;
    let guest = t.to(GUEST_CHAT);
    // Welcome, partial, released.
    assert_eq!(guest.len(), 3, "{guest:#?}");
    assert!(
        guest[2].text.ends_with(
            "Part of the payment had already arrived: Test User will contact you about a refund."
        ),
        "{}",
        guest[2].text
    );
    assert_eq!(t.to(HOST_CHAT).len(), 1);
}

#[tokio::test]
async fn a_guest_cancelling_an_unpaid_booking_tells_no_host() {
    let t = tg_env().await;
    let e = &t.e;
    let (bid, pay_token) = t.book("10:00").await;
    t.start(&link_token(&[0u8; 32], &pay_token)).await;
    let cancel = e.booking_col(&bid, "cancel_token").await.unwrap();
    let resp = e.post_guest(&format!("/booking/cancel/{cancel}"), "").await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    t.tg.flush(&e.pool).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 2, "{guest:#?}");
    assert!(guest[1]
        .text
        .starts_with("You cancelled your booking: Test Meeting, "));
    assert!(t.to(HOST_CHAT).is_empty(), "{:#?}", t.to(HOST_CHAT));
}

#[tokio::test]
async fn telegram_off_offers_no_link_and_alerts_reach_no_chat() {
    let t = tg_env().await;
    let e = &t.e;
    // A bot that would record any send, then Telegram off.
    crate::telegram::install(None);
    assert!(!crate::telegram::enabled());
    let (bid, pay_token) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    let (_, _, page) = e.get_page(&format!("/booking/pay/{pay_token}")).await;
    assert!(!page.contains("https://t.me/"));
    let links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_guest_links")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(links, 0);
    // The alerts, through the sink `calrs serve` installs: no chat hears.
    let inv = e.inv(&bid).await;
    e.pay(AMOUNT / 2, &code(&inv.reference));
    e.step(e.now).await;
    e.pay(AMOUNT / 2, &code(&inv.reference));
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    let alerts = e.sink.all();
    assert!(alerts.len() >= 2, "{alerts:?}");
    let sink = crate::web::ton_pay::telegram::serve_sink(&e.state);
    for alert in alerts {
        sink.deliver(alert).await;
    }
    t.tg.flush(&e.pool).await;
    assert!(t.tg.sent().is_empty(), "{:#?}", t.tg.sent());
}

/// `calrs serve`'s alert sink (the one `watcher::spawn` installs): each
/// alert reaches the owner's email and the booking's Telegram chats.
#[tokio::test]
async fn the_serve_sink_emails_the_owner_and_tells_the_chats() {
    let t = tg_env().await;
    let e = &t.e;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (bid, pay_token) = t.book("10:00").await;
    t.start(&link_token(&[0u8; 32], &pay_token)).await;
    let inv = e.inv(&bid).await;
    e.pay(AMOUNT, &code(&inv.reference));
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    let sink = crate::web::ton_pay::telegram::serve_sink(&e.state);
    for alert in e.sink.all() {
        sink.deliver(alert).await;
    }
    t.tg.flush(&e.pool).await;
    let owner = settled_mails_to(&mails, "test@example.com").await;
    assert!(
        owner.iter().any(|m| m.contains("Payment received")),
        "{owner:?}"
    );
    let host = t.to(HOST_CHAT);
    assert_eq!(host.len(), 1, "{host:#?}");
    assert!(host[0].text.starts_with("Paid: Test Meeting, "));
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 2, "{guest:#?}");
    assert!(guest[1]
        .text
        .starts_with("Payment received, your booking is confirmed: Test Meeting, "));
}

/// Mark the invoice of `bid` paid in full and held (the host was busy).
async fn paid_and_held(e: &Env, bid: &str) {
    sqlx::query(
        "UPDATE ton_invoices SET status = 'paid', paid_nano = amount_nano, problem = 'host_busy'
         WHERE booking_id = ?",
    )
    .bind(bid)
    .execute(&e.pool)
    .await
    .unwrap();
}

/// A host confirming a paid booking: with the watcher running, the guest
/// hears "payment received" (not "approved" as well); with payments off,
/// no payment notice will come, so the guest hears upstream's "approved".
#[tokio::test]
async fn a_paid_booking_the_host_confirms_is_told_once_with_payments_on_or_off() {
    let t = tg_env().await;
    let e = &t.e;
    let session = e.session().await;
    let (bid, pay_token) = t.book("10:00").await;
    t.start(&link_token(&[0u8; 32], &pay_token)).await;
    paid_and_held(e, &bid).await;
    let resp = e
        .post_authed(&format!("/dashboard/bookings/{bid}/confirm"), &session)
        .await;
    assert!(resp.status().is_redirection(), "{}", resp.status());
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    t.tg.flush(&e.pool).await;
    assert_eq!(t.to(GUEST_CHAT).len(), 1, "{:#?}", t.to(GUEST_CHAT));
    t.step(e.now).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 2, "{guest:#?}");
    assert!(guest[1]
        .text
        .starts_with("Payment received, your booking is confirmed: Test Meeting, "));

    // Payments off (unset or invalid configuration): no watcher, no notice.
    let (bid, pay_token) = t.book("11:00").await;
    t.start(&link_token(&[0u8; 32], &pay_token)).await;
    assert_eq!(t.to(GUEST_CHAT).len(), 3);
    paid_and_held(e, &bid).await;
    e.state.ton.uninstall();
    let resp = e
        .post_authed(&format!("/dashboard/bookings/{bid}/confirm"), &session)
        .await;
    assert!(resp.status().is_redirection(), "{}", resp.status());
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    t.tg.flush(&e.pool).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 4, "{guest:#?}");
    assert!(
        guest[3]
            .text
            .starts_with("Your request is confirmed: Test Meeting, "),
        "{}",
        guest[3].text
    );
}

/// The invoice expired unpaid, and calrs's sweep has not declined the
/// booking yet: a guest cancel is as quiet as before the expiry, by email
/// and in the hosts' chats. With a part paid, the hosts hear it on both.
#[tokio::test]
async fn a_guest_cancel_after_an_unpaid_expiry_tells_no_host_on_any_channel() {
    let t = tg_env().await;
    let e = &t.e;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (bid, pay_token) = t.book("10:00").await;
    t.start(&link_token(&[0u8; 32], &pay_token)).await;
    let settle_by = e.inv(&bid).await.settle_by;
    e.chain
        .height((settle_by + Duration::minutes(3)).timestamp() as u64);
    let at = settle_by + Duration::minutes(4);
    assert_eq!(t.step(at).await.expired, 1);
    assert_eq!(e.inv(&bid).await.status, "expired");
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    assert!(invoice::cancel_is_quiet(&e.pool, &bid).await);
    let cancel = e.booking_col(&bid, "cancel_token").await.unwrap();
    let resp = e.post_guest(&format!("/booking/cancel/{cancel}"), "").await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    t.tg.flush(&e.pool).await;
    assert!(t.to(HOST_CHAT).is_empty(), "{:#?}", t.to(HOST_CHAT));
    let host_mails = settled_mails_to(&mails, "test@example.com").await;
    assert!(host_mails.is_empty(), "{host_mails:?}");

    // A part paid (here, on an invoice that expires in the same step: the
    // chain is already past its deadline): the host has money to return
    // and hears the cancel, after the owner's "partial" problem.
    let (bid, pay_token) = t.book("11:00").await;
    t.start(&link_token(&[0u8; 32], &pay_token)).await;
    let inv = e.inv(&bid).await;
    // Past this invoice's own deadline too: the chain height left by the
    // first case is a second short of it whenever the second booking was
    // made in a later second than the first.
    e.chain
        .height((inv.settle_by + Duration::minutes(3)).timestamp() as u64);
    e.pay(AMOUNT / 2, &code(&inv.reference));
    t.step(e.now).await;
    assert!(!invoice::cancel_is_quiet(&e.pool, &bid).await);
    let cancel = e.booking_col(&bid, "cancel_token").await.unwrap();
    e.post_guest(&format!("/booking/cancel/{cancel}"), "").await;
    t.tg.flush(&e.pool).await;
    let host = t.to(HOST_CHAT);
    assert_eq!(host.len(), 2, "{host:#?}");
    assert!(host[0].text.starts_with("Payment needs attention: "));
    assert!(
        host[1]
            .text
            .starts_with("Guest Name cancelled: Test Meeting, "),
        "{}",
        host[1].text
    );
    let host_mails = settled_mails_to(&mails, "test@example.com").await;
    assert!(!host_mails.is_empty());
}

/// "Email me the link" carries the booking's Telegram link, the one the
/// payment page shows; a booking whose event type does not offer Telegram
/// gets the email without it.
#[tokio::test]
async fn the_payment_link_email_carries_the_telegram_link() {
    let t = tg_env().await;
    let e = &t.e;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (_, pay_token) = t.book("10:00").await;
    let (_, _, page) = e.get_page(&format!("/booking/pay/{pay_token}")).await;
    let link = deep_link_in(&page);
    let resp = e
        .post_guest(&format!("/booking/pay/{pay_token}/email"), "")
        .await;
    assert_eq!(location(&resp), format!("/booking/pay/{pay_token}?sent=1"));
    let mail = mail_to(&mails, "guest@test.com").await;
    let flat = mail.split_whitespace().collect::<String>();
    assert!(flat.contains(&link), "{mail}");
    let button = crate::i18n::translate("en", "tg-email-button", None);
    assert!(mail.contains(&button), "{mail}");
    assert!(mail.contains(&format!("{BASE}/booking/pay/{pay_token}")));
    // Sending the email linked nothing and sent nothing to Telegram.
    assert!(t.tg.sent().is_empty(), "{:#?}", t.tg.sent());

    // Telegram off for the event type: the same email, no Telegram.
    crate::telegram::store::save_event_type_mode(&e.pool, &e.et_id, "off")
        .await
        .unwrap();
    let (_, other) = e.book_http("other@test.com", "11:00", "198.51.100.8").await;
    let resp = e
        .post_guest(&format!("/booking/pay/{other}/email"), "")
        .await;
    assert_eq!(location(&resp), format!("/booking/pay/{other}?sent=1"));
    let mail = mail_to(&mails, "other@test.com").await;
    assert!(mail.contains(&format!("{BASE}/booking/pay/{other}")));
    assert!(!mail.contains("t.me/"), "{mail}");
    assert!(!mail.contains(&button), "{mail}");
}

/// A new `secret.key` (rotated, or restored from another backup) derives
/// another Telegram link from the same payment link. The payment page
/// re-keys the booking's row to the new one, so what it shows works; the
/// old link gets "open the payment page again" from the bot, not "invalid";
/// a chat linked before the change stays linked.
#[tokio::test]
async fn a_new_secret_key_replaces_the_payment_pages_telegram_link() {
    let t = tg_env().await;
    let e = &t.e;
    let (bid, pay_token) = t.book("10:00").await;
    // A second guest, linked before the key changes.
    let (linked_bid, linked_pay) = e
        .book_http("second@test.com", "11:00", "198.51.100.9")
        .await;
    const SECOND_CHAT: i64 = 123123123;
    let old_linked = link_token(&[0u8; 32], &linked_pay);
    let update = fake::private_text(1, SECOND_CHAT, None, &format!("/start {old_linked}"));
    handle_update(&t.tg.rt, &e.pool, &update).await;
    t.tg.flush(&e.pool).await;
    assert_eq!(t.to(SECOND_CHAT).len(), 1, "{:#?}", t.tg.sent());

    let old = link_token(&[0u8; 32], &pay_token);
    let (_, _, page) = e.get_page(&format!("/booking/pay/{pay_token}")).await;
    assert_eq!(deep_link_in(&page).rsplit('=').next().unwrap(), old);

    // calrs restarted with another key.
    let rotated = build_state(e.pool.clone(), e._dir.to_path_buf(), [7u8; 32]).await;
    rotated.ton.install(e.rt.clone());
    let app = || crate::web::router(rotated.clone()).layer(local_proxy_peer());
    let page_of = |pay: String| {
        let app = app();
        async move {
            let resp = app
                .oneshot(get(&format!("/booking/pay/{pay}")))
                .await
                .unwrap();
            unescape(&body_string(resp).await)
        }
    };
    let new = link_token(&[7u8; 32], &pay_token);
    assert_ne!(new, old);
    let page = page_of(pay_token.clone()).await;
    assert_eq!(deep_link_in(&page).rsplit('=').next().unwrap(), new);
    // The same on the next load.
    let page = page_of(pay_token.clone()).await;
    assert_eq!(deep_link_in(&page).rsplit('=').next().unwrap(), new);
    let (stored, replaced): (String, Option<String>) = sqlx::query_as(
        "SELECT link_token_hash, replaced_token_hash FROM telegram_guest_links WHERE booking_id = ?",
    )
    .bind(&bid)
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(stored, crate::telegram::tokens::hash(&new));
    assert_eq!(replaced, Some(crate::telegram::tokens::hash(&old)));

    // The old link: told where the new one is, linked to nothing.
    t.start(&old).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 1, "{guest:#?}");
    assert_eq!(
        guest[0].text,
        crate::i18n::translate("en", "tg-bot-link-replaced", None)
    );
    let chat: Option<i64> =
        sqlx::query_scalar("SELECT chat_id FROM telegram_guest_links WHERE booking_id = ?")
            .bind(&bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!(chat, None);
    // The new one links, with the payment welcome.
    t.start(&new).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 2, "{guest:#?}");
    assert!(
        guest[1]
            .text
            .starts_with("Done: notifications about this booking will come here."),
        "{}",
        guest[1].text
    );
    assert_eq!(
        guest[1].buttons[0].url,
        format!("{BASE}/booking/pay/{pay_token}")
    );

    // Linked before the change: re-keyed on the next page load, still
    // linked; its old link says "already on" in that chat.
    let page = page_of(linked_pay.clone()).await;
    assert_eq!(
        deep_link_in(&page).rsplit('=').next().unwrap(),
        link_token(&[7u8; 32], &linked_pay)
    );
    let chat: Option<i64> =
        sqlx::query_scalar("SELECT chat_id FROM telegram_guest_links WHERE booking_id = ?")
            .bind(&linked_bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!(chat, Some(SECOND_CHAT));
    let update = fake::private_text(2, SECOND_CHAT, None, &format!("/start {old_linked}"));
    handle_update(&t.tg.rt, &e.pool, &update).await;
    t.tg.flush(&e.pool).await;
    let second = t.to(SECOND_CHAT);
    assert_eq!(second.len(), 2, "{second:#?}");
    assert_eq!(
        second[1].text,
        crate::i18n::translate("en", "tg-bot-already-linked", None)
    );

    // Once the booking is cancelled, the old link is just invalid.
    let cancel = e.booking_col(&bid, "cancel_token").await.unwrap();
    e.post_guest(&format!("/booking/cancel/{cancel}"), "").await;
    t.tg.flush(&e.pool).await;
    let update = fake::private_text(3, 777000777, None, &format!("/start {old}"));
    handle_update(&t.tg.rt, &e.pool, &update).await;
    t.tg.flush(&e.pool).await;
    let stranger = t.to(777000777);
    assert_eq!(stranger.len(), 1, "{stranger:#?}");
    assert_eq!(
        stranger[0].text,
        crate::i18n::translate("en", "tg-bot-link-invalid", None)
    );
}

/// Two key changes while a booking stands (a restore, then a rotation):
/// the link from the original key, the one the guest saw when booking,
/// still gets "replaced"; the one from the key in between is invalid. Back
/// on a key it had, the row answers to it again.
#[tokio::test]
async fn after_two_key_changes_the_original_link_is_still_replaced() {
    let t = tg_env().await;
    let e = &t.e;
    let (bid, pay_token) = t.book("10:00").await;
    let link_on = |key: [u8; 32]| {
        let pay = pay_token.clone();
        let pool = e.pool.clone();
        let dir = e._dir.to_path_buf();
        let rt = e.rt.clone();
        async move {
            let state = build_state(pool, dir, key).await;
            state.ton.install(rt);
            let resp = crate::web::router(state)
                .layer(local_proxy_peer())
                .oneshot(get(&format!("/booking/pay/{pay}")))
                .await
                .unwrap();
            let page = unescape(&body_string(resp).await);
            deep_link_in(&page).rsplit('=').next().unwrap().to_string()
        }
    };
    let original = link_token(&[0u8; 32], &pay_token);
    let between = link_token(&[7u8; 32], &pay_token);
    let latest = link_token(&[9u8; 32], &pay_token);
    assert_eq!(link_on([7u8; 32]).await, between);
    assert_eq!(link_on([9u8; 32]).await, latest);
    let row = || async {
        sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT link_token_hash, replaced_token_hash FROM telegram_guest_links WHERE booking_id = ?",
        )
        .bind(&bid)
        .fetch_one(&e.pool)
        .await
        .unwrap()
    };
    let hash = crate::telegram::tokens::hash;
    assert_eq!(row().await, (hash(&latest), Some(hash(&original))));

    t.start(&original).await;
    t.start(&between).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 2, "{guest:#?}");
    assert_eq!(
        guest[0].text,
        crate::i18n::translate("en", "tg-bot-link-replaced", None)
    );
    assert_eq!(
        guest[1].text,
        crate::i18n::translate("en", "tg-bot-link-invalid", None)
    );

    // The original key restored: its link works again, the latest is the
    // replaced one.
    assert_eq!(link_on([0u8; 32]).await, original);
    assert_eq!(row().await, (hash(&original), Some(hash(&latest))));
}

// --- booking flow v2 (ilia fork): TON prices and renewals in Telegram ---

/// A TON price says no dollars; a renewal asks the linked guest again,
/// with what is due and the new deadline, and tells the hosts nothing.
#[tokio::test]
async fn a_renewal_asks_the_linked_guest_again() {
    let t = tg_env_with(&[("CALRS_TON_MAX_RENEWALS", "2")], Some(100_000_000_000)).await;
    let e = &t.e;
    let (bid, pay_token) = t.book("10:00").await;
    t.start(&link_token(&[0u8; 32], &pay_token)).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 1, "{guest:#?}");
    assert!(
        guest[0].text.contains("Amount: 100.00 TON. Pay by "),
        "{}",
        guest[0].text
    );
    assert!(!guest[0].text.contains('$'), "{}", guest[0].text);
    let expires: String =
        sqlx::query_scalar("SELECT expires_at FROM ton_invoices WHERE booking_id = ?")
            .bind(&bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    let expires = crate::web::ton_pay::parse_stamp(&expires).unwrap();
    assert_eq!(t.step(expires + Duration::seconds(5)).await.renewed, 1);
    // Not before the indexer has read past the renewal.
    assert_eq!(t.to(GUEST_CHAT).len(), 1);
    e.chain
        .height((expires + Duration::minutes(4)).timestamp() as u64);
    t.step(expires + Duration::minutes(4)).await;
    let guest = t.to(GUEST_CHAT);
    assert_eq!(guest.len(), 2, "{guest:#?}");
    let renewed = &guest[1];
    assert!(
        renewed.text.starts_with("The payment for Test Meeting, "),
        "{}",
        renewed.text
    );
    assert!(
        renewed.text.contains("Pay the 100.00 TON still due by "),
        "{}",
        renewed.text
    );
    let deadline = (expires + Duration::seconds(5) + Duration::hours(6))
        .format("%H:%M")
        .to_string();
    assert!(renewed.text.contains(&deadline), "{}", renewed.text);
    assert!(
        renewed
            .text
            .contains("If you have already paid, no action is needed"),
        "{}",
        renewed.text
    );
    assert_eq!(labels(renewed), ["Pay", "Cancel booking"]);
    assert_eq!(
        renewed.buttons[0].url,
        format!("{BASE}/booking/pay/{pay_token}")
    );
    let reference = e.inv(&bid).await.reference;
    assert!(!renewed.text.contains(&code(&reference)));
    assert!(t.to(HOST_CHAT).is_empty(), "{:#?}", t.to(HOST_CHAT));
    // Once.
    t.step(expires + Duration::minutes(5)).await;
    assert_eq!(t.to(GUEST_CHAT).len(), 2);
    // Paid in the new window: confirmed, the hosts hear it, in TON only.
    e.pay(100_000_000_000, &code(&reference));
    t.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    let host = t.to(HOST_CHAT);
    assert_eq!(host.len(), 1, "{host:#?}");
    assert!(
        host[0]
            .text
            .contains("Received 100.00 TON; the booking is confirmed."),
        "{}",
        host[0].text
    );
    assert!(!host[0].text.contains('$'));
}

/// With Telegram on, the price section offers the guest's Telegram,
/// showing the type's own mode, and saves the choice.
#[tokio::test]
async fn the_price_section_offers_the_guest_telegram_mode() {
    let t = tg_env().await;
    let e = &t.e;
    let session = e.session().await;
    let edit = "/dashboard/event-types/test-meeting/edit";
    let resp = e.app().oneshot(get_authed(edit, &session)).await.unwrap();
    let page = body_string(resp).await;
    assert!(page.contains("name=\"ton_guest_telegram\""), "{page}");
    assert!(page.contains("<option value=\"optional\" selected>"));
    let body = format!("_csrf={CSRF}&title=Test+Meeting&slug=test-meeting&duration_min=30&location_value=https%3A%2F%2Fmeet.example.com&avail_days=1,2,3,4,5&avail_start=09:00&avail_end=17:00&ton_price_enabled=on&ton_price_currency=USD&ton_price_amount=40&ton_guest_telegram=required");
    let resp = e
        .app()
        .oneshot(post_form(edit, &session, CSRF, &body))
        .await
        .unwrap();
    assert!(resp.status().is_redirection());
    assert_eq!(
        crate::telegram::store::event_type_mode(&e.pool, &e.et_id).await,
        "required"
    );
}
