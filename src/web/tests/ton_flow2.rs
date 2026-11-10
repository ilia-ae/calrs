//! ilia fork, booking flow v2 (ilia-booking docs/ton.md): prices in TON by
//! default (no rate at all), Telegram asked for by default on a priced
//! type, and the renewal of an unpaid window while the meeting is far away.
//! A child of the TON tests, for their chain, rates and booking helpers.

use super::*;
use crate::ton::amount::NANO_PER_TON;
use crate::web::ton_pay::stamp;

/// The owner's price.
const HUNDRED: u128 = 100 * NANO_PER_TON;

/// Renewals on (2), as in production.
const RENEW: [(&str, &str); 1] = [("CALRS_TON_MAX_RENEWALS", "2")];

async fn ton_env(extra: &[(&'static str, &str)]) -> Env {
    env_priced(extra, Some(HUNDRED)).await
}

/// (currency, price_usd_cents, rate_usd_micro, rate_sources, reference,
/// pay_token, created_at, expires_at, renewals, renewals_max).
type Snapshot = (
    String,
    i64,
    i64,
    String,
    String,
    String,
    String,
    String,
    i64,
    i64,
);

async fn snapshot(e: &Env, bid: &str) -> Snapshot {
    sqlx::query_as(
        "SELECT currency, price_usd_cents, rate_usd_micro, rate_sources, reference, pay_token,
                created_at, expires_at, renewals, renewals_max
         FROM ton_invoices WHERE booking_id = ?",
    )
    .bind(bid)
    .fetch_one(&e.pool)
    .await
    .unwrap()
}

fn renewed_alerts(e: &Env) -> usize {
    e.sink
        .all()
        .iter()
        .filter(|a| matches!(a, TonAlert::Renewed { .. }))
        .count()
}

async fn held(pool: &SqlitePool) -> i64 {
    let user: String = sqlx::query_scalar("SELECT id FROM users LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM bookings b JOIN event_types et ON et.id = b.event_type_id
         JOIN accounts a ON a.id = et.account_id WHERE {}",
        crate::booking_time::user_busy_bookings(true)
    ))
    .bind(&user)
    .bind(&user)
    .bind(&user)
    .fetch_one(pool)
    .await
    .unwrap()
}

// --- (b) prices in TON ---------------------------------------------------------

/// A TON price is the amount asked, exactly: no rate source is ever asked,
/// not by the booking, the pages, nor the watcher's steps.
#[tokio::test]
async fn a_ton_price_is_invoiced_exactly_without_any_rate() {
    let e = ton_env(&[]).await;
    assert_eq!(e.rates.calls(), 0, "the first step fetched no rate");
    let (bid, opened) = e.booking("guest@example.com").await;
    assert_eq!(opened.amount_nano as u128, HUNDRED);
    let s = snapshot(&e, &bid).await;
    assert_eq!(
        (s.0.as_str(), s.1, s.2, s.3.as_str()),
        ("TON", 0, 1, ""),
        "neutral rate columns"
    );
    // The web booking, its page and its status, a few steps.
    let (web, token) = e.book_http("web@test.com", "10:00", "198.51.100.1").await;
    assert_eq!(e.inv(&web).await.amount as u128, HUNDRED);
    let (status, _, page) = e.get_page(&format!("/booking/pay/{token}")).await;
    assert_eq!(status, 200);
    assert!(page.contains("100.00 TON"), "{page}");
    assert!(!page.contains('$'), "no dollars for a TON price");
    let (_, _, json) = e.get_page(&format!("/booking/pay/{token}/status")).await;
    assert!(json.contains("\"remaining_ton\":\"100.00\""), "{json}");
    for minutes in [1, 6, 12] {
        e.step(e.now + Duration::minutes(minutes)).await;
    }
    // The slot page shows the price as it is.
    let (_, _, slots) = e.get_page("/u/testuser/test-meeting").await;
    assert!(slots.contains("Price: 100.00 TON"), "{slots}");
    assert!(!slots.contains("(≈"), "no approximation");
    assert_eq!(e.rates.calls(), 0);

    // A full payment confirms it.
    e.pay(HUNDRED, &opened.reference);
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    // The owner's email has no dollars.
    let id: String = sqlx::query_scalar("SELECT id FROM ton_invoices WHERE booking_id = ?")
        .bind(&bid)
        .fetch_one(&e.pool)
        .await
        .unwrap();
    let mail = crate::web::ton_pay::alerts::invoice_mail(&e.pool, &id)
        .await
        .unwrap();
    assert_eq!((mail.price_usd.as_str(), mail.rate_usd.as_str()), ("", ""));
    assert_eq!(mail.amount_ton, "100.00");
    assert_eq!(e.rates.calls(), 0);
}

/// A USD price still takes the rate (the existing tests price in USD).
#[tokio::test]
async fn a_usd_price_still_uses_the_rate() {
    let e = env().await;
    let before = e.rates.calls();
    assert!(before > 0, "the watcher keeps a USD rate warm");
    let (bid, opened) = e.booking("guest@example.com").await;
    assert_eq!(opened.amount_nano as u128, AMOUNT);
    let s = snapshot(&e, &bid).await;
    assert_eq!((s.0.as_str(), s.1, s.2), ("USD", 4000, 2_000_000));
}

/// Without a corridor (neither variable set) payments work for TON prices
/// and nothing is fetched; a USD price is refused, never free.
#[tokio::test]
async fn without_a_corridor_ton_prices_work_and_usd_ones_are_refused() {
    let no_corridor = [
        ("CALRS_TON_RATE_MIN_USD", ""),
        ("CALRS_TON_RATE_MAX_USD", ""),
    ];
    let e = ton_env(&no_corridor).await;
    assert!(e.rt.cfg.rate_policy.is_none());
    let (_, opened) = e.booking("guest@example.com").await;
    assert_eq!(opened.amount_nano as u128, HUNDRED);

    set_price(&e.pool, &e.et_id, 4000).await;
    e.step(e.now).await;
    assert_eq!(e.rates.calls(), 0, "no corridor: no rate, even for USD");
    let r = e
        .booking_at("usd@example.com", e.now + Duration::days(4), "198.51.100.9")
        .await;
    match r {
        Err(Refusal::Unavailable(why)) => assert!(why.contains("USD prices need"), "{why}"),
        other => panic!("{other:?}"),
    }
    let resp = e
        .post_booking(
            "/u/testuser/test-meeting/book",
            "web@test.com",
            "10:00",
            "198.51.100.1",
        )
        .await;
    assert_ne!(resp.status(), axum::http::StatusCode::SEE_OTHER);
    let page = body_string(resp).await;
    assert!(page.contains(&crate::i18n::translate(
        "en",
        "ton-bae-body-unavailable",
        None
    )));
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ton_invoices")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(n, 1, "only the TON booking");
    // The form warns the owner.
    let session = e.session().await;
    let resp = e
        .app()
        .oneshot(get_authed(
            "/dashboard/event-types/test-meeting/edit",
            &session,
        ))
        .await
        .unwrap();
    let form = body_string(resp).await;
    assert!(form.contains(&crate::i18n::translate("en", "ton-etf-usd-no-rates", None)));
    assert_eq!(e.rates.calls(), 0);
}

/// The form: TON by default (blank and switched on is 100 TON), a TON
/// amount kept as typed, and a duplicate keeps currency and amount.
#[tokio::test]
async fn the_form_prices_in_ton_by_default() {
    let e = env().await;
    let session = e.session().await;
    sqlx::query("DELETE FROM ton_prices")
        .execute(&e.pool)
        .await
        .unwrap();
    let edit = "/dashboard/event-types/test-meeting/edit";
    let post = |extra: &str| {
        format!("_csrf={CSRF}&title=Test+Meeting&slug=test-meeting&duration_min=30&location_value=https%3A%2F%2Fmeet.example.com&avail_days=1,2,3,4,5&avail_start=09:00&avail_end=17:00{extra}")
    };
    let price = || async {
        sqlx::query_as::<_, (String, Option<i64>, Option<i64>, i64)>(
            "SELECT currency, price_nano, price_usd_cents, enabled FROM ton_prices",
        )
        .fetch_optional(&e.pool)
        .await
        .unwrap()
    };
    // Unchecked and blank: no price row (a new type is not priced).
    e.app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_amount=&ton_price_currency=TON"),
        ))
        .await
        .unwrap();
    assert_eq!(price().await, None);
    // Switched on, blank: the default, 100 TON.
    e.app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_enabled=on&ton_price_amount="),
        ))
        .await
        .unwrap();
    assert_eq!(
        price().await,
        Some(("TON".into(), Some(100_000_000_000), None, 1))
    );
    // A TON amount, as typed.
    e.app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_enabled=on&ton_price_currency=TON&ton_price_amount=150.5"),
        ))
        .await
        .unwrap();
    assert_eq!(
        price().await,
        Some(("TON".into(), Some(150_500_000_000), None, 1))
    );
    let resp = e.app().oneshot(get_authed(edit, &session)).await.unwrap();
    assert!(body_string(resp).await.contains("value=\"150.50\""));
    // Invalid TON: refused with the TON message, nothing changed.
    let resp = e
        .app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_enabled=on&ton_price_amount=0.5"),
        ))
        .await
        .unwrap();
    let page = body_string(resp).await;
    assert!(page.contains(&crate::i18n::translate(
        "en",
        "ton-form-error-price-ton",
        None
    )));
    assert_eq!(
        price().await,
        Some(("TON".into(), Some(150_500_000_000), None, 1))
    );
    // A duplicate keeps the currency and the amount.
    let resp = e
        .post_authed("/dashboard/event-types/test-meeting/duplicate", &session)
        .await;
    assert!(resp.status().is_redirection());
    let copies: Vec<(String, Option<i64>)> =
        sqlx::query_as("SELECT currency, price_nano FROM ton_prices WHERE event_type_id != ?")
            .bind(&e.et_id)
            .fetch_all(&e.pool)
            .await
            .unwrap();
    assert_eq!(copies, vec![("TON".to_string(), Some(150_500_000_000))]);
}

/// The fork migration: an existing USD price survives as USD with its
/// cents, an invoice made before it reads as USD, and the priced type asks
/// for the guest's Telegram unless its mode was chosen.
#[tokio::test]
async fn existing_usd_prices_survive_the_currency_migration() {
    let pool = setup_test_db().await;
    let (_, _, et_id) = seed_test_data(&pool).await;
    // Back to the shape before ilia_004.
    sqlx::raw_sql(
        "DROP TABLE ton_prices;
         CREATE TABLE ton_prices (
             event_type_id   TEXT PRIMARY KEY REFERENCES event_types(id) ON DELETE CASCADE,
             price_usd_cents INTEGER NOT NULL CHECK (price_usd_cents BETWEEN 100 AND 1000000),
             window_min      INTEGER CHECK (window_min IS NULL OR window_min BETWEEN 10 AND 240),
             enabled         INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
             updated_at      TEXT NOT NULL);
         ALTER TABLE ton_invoices DROP COLUMN currency;
         DELETE FROM _migrations WHERE name = 'ilia_004_ton_price_currency';",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO ton_prices (event_type_id, price_usd_cents, window_min, enabled, updated_at)
         VALUES (?, 4050, 20, 1, '2026-11-01T00:00:00Z')",
    )
    .bind(&et_id)
    .execute(&pool)
    .await
    .unwrap();
    crate::db::migrate(&pool).await.unwrap();
    let row: (String, Option<i64>, Option<i64>, Option<i64>, i64) = sqlx::query_as(
        "SELECT currency, price_nano, price_usd_cents, window_min, enabled FROM ton_prices",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row, ("USD".to_string(), None, Some(4050), Some(20), 1));
    assert_eq!(
        crate::telegram::store::event_type_mode(&pool, &et_id).await,
        "required"
    );
    let default: String = sqlx::query_scalar(
        "SELECT dflt_value FROM pragma_table_info('ton_invoices') WHERE name = 'currency'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(default, "'USD'");
}

// --- (c) Telegram by default for a priced type ----------------------------------

#[tokio::test]
async fn a_new_price_asks_for_telegram_and_a_lowered_mode_stays_lowered() {
    let e = env().await;
    let session = e.session().await;
    sqlx::query("DELETE FROM ton_prices")
        .execute(&e.pool)
        .await
        .unwrap();
    let mode = || crate::telegram::store::event_type_mode(&e.pool, &e.et_id);
    assert_eq!(mode().await, "off");
    let edit = "/dashboard/event-types/test-meeting/edit";
    let post = |extra: &str| {
        format!("_csrf={CSRF}&title=Test+Meeting&slug=test-meeting&duration_min=30&location_value=https%3A%2F%2Fmeet.example.com&avail_days=1,2,3,4,5&avail_start=09:00&avail_end=17:00{extra}")
    };
    // A price kept switched off asks for nothing.
    e.app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_amount=100"),
        ))
        .await
        .unwrap();
    assert_eq!(mode().await, "off");
    // Switched on: required.
    let on = post("&ton_price_enabled=on&ton_price_amount=100");
    e.app()
        .oneshot(post_form(edit, &session, CSRF, &on))
        .await
        .unwrap();
    assert_eq!(mode().await, "required");
    // The owner lowers it; saving the price again does not raise it.
    crate::telegram::store::save_event_type_mode(&e.pool, &e.et_id, "optional")
        .await
        .unwrap();
    e.app()
        .oneshot(post_form(edit, &session, CSRF, &on))
        .await
        .unwrap();
    assert_eq!(mode().await, "optional");
    // Removed and priced again: still the owner's choice.
    e.app()
        .oneshot(post_form(edit, &session, CSRF, &post("&ton_price_amount=")))
        .await
        .unwrap();
    e.app()
        .oneshot(post_form(edit, &session, CSRF, &on))
        .await
        .unwrap();
    assert_eq!(mode().await, "optional");
    // The section's own choice, when posted, is saved (it is only shown
    // while Telegram is on).
    e.app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_enabled=on&ton_price_amount=100&ton_guest_telegram=off"),
        ))
        .await
        .unwrap();
    assert_eq!(mode().await, "off");
    let resp = e
        .app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_enabled=on&ton_price_amount=100&ton_guest_telegram=always"),
        ))
        .await
        .unwrap();
    let page = body_string(resp).await;
    assert!(page.contains(&crate::i18n::translate(
        "en",
        "ton-form-error-telegram",
        None
    )));
    assert_eq!(mode().await, "off");
}

// --- (d) renewals -----------------------------------------------------------------

/// A far meeting whose window ends unpaid keeps its time: a new, longer
/// window from now (`CALRS_TON_RENEW_WINDOW_MIN`, 6 hours), the same
/// comment code, payment link and creation time, the booking's hard
/// deadline moved, one notice once the indexer has read past the renewal,
/// none again after a restart.
#[tokio::test]
async fn an_unpaid_window_of_a_far_meeting_is_renewed() {
    let e = ton_env(&RENEW).await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let before = snapshot(&e, &bid).await;
    assert_eq!((before.8, before.9), (0, 2));
    // Not before the window ends.
    let r = e.step(opened.expires_at - Duration::seconds(1)).await;
    assert_eq!(r.renewed, 0);

    let at = opened.expires_at + Duration::seconds(5);
    let r = e.step(at).await;
    assert_eq!((r.renewed, r.expired), (1, 0));
    let after = snapshot(&e, &bid).await;
    assert_eq!(after.4, before.4, "the same comment code");
    assert_eq!(after.5, before.5, "the same payment link");
    assert_eq!(after.6, before.6, "the same creation time");
    assert_eq!(after.7, stamp(at + Duration::hours(6)), "a new window");
    assert_eq!(after.8, 1);
    let inv = e.inv(&bid).await;
    assert_eq!(inv.status, "open");
    assert_eq!(
        inv.settle_by,
        at + Duration::hours(6) + Duration::minutes(15)
    );
    assert_eq!(
        pending_expires_at(&e.pool, &bid).await,
        Some(inv.hard_deadline)
    );
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    assert_eq!(held(&e.pool).await, 1, "the slot stays held");
    // The indexer has not read past the renewal yet: no notice.
    assert_eq!(renewed_alerts(&e), 0);
    e.chain
        .height((at + Duration::minutes(4)).timestamp() as u64);
    e.step(at + Duration::minutes(4)).await;
    assert_eq!(renewed_alerts(&e), 1);
    // Once per renewal, also after a restart.
    e.step(at + Duration::minutes(5)).await;
    assert_eq!(renewed_alerts(&e), 1);
    let sink = Arc::new(CaptureSink::default());
    let rt = runtime(
        &vars(&RENEW),
        e.chain.clone(),
        e.rates.clone(),
        sink.clone(),
    );
    watcher::step(&e.state, &rt, at + Duration::minutes(6)).await;
    assert!(sink
        .all()
        .iter()
        .all(|a| !matches!(a, TonAlert::Renewed { .. })));

    // The page says so; the dashboard and the waive form too.
    let (_, _, page) = e.get_page(&format!("/booking/pay/{}", after.5)).await;
    assert!(
        page.contains("The deadline was extended (1 of 2)."),
        "{page}"
    );
    let session = e.session().await;
    let resp = e
        .app()
        .oneshot(get_authed("/dashboard/bookings", &session))
        .await
        .unwrap();
    assert!(body_string(resp).await.contains("deadline extended 1 of 2"));
    let resp = e
        .app()
        .oneshot(get_authed(&format!("/dashboard/ton/waive/{bid}"), &session))
        .await
        .unwrap();
    assert!(body_string(resp).await.contains("deadline extended 1 of 2"));
}

/// The guest is asked again by email: the payment page, the new deadline
/// and the cancel link, from the host, with no amount, address or coin.
#[tokio::test]
async fn a_renewal_emails_the_guest_the_new_deadline() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let _guard = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", "https://cal.example.com");
    let e = ton_env(&RENEW).await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (bid, token) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    // The guest asked for the link by email: the address is proven.
    prove_address(&e, &bid).await;
    let expires = parse_stamp_of(&e, &bid).await;
    renew_and_notify(&e, expires).await;
    for alert in e.sink.all() {
        if matches!(alert, TonAlert::Renewed { .. }) {
            crate::web::ton_pay::alerts::deliver_now(&e.pool, &[0u8; 32], alert).await;
        }
    }
    let mail = mail_to(&mails, "guest@test.com").await;
    assert!(
        mail.contains("Your time is still held: Test Meeting"),
        "{mail}"
    );
    assert!(
        mail.contains("We have not received the payment for Test Meeting"),
        "{mail}"
    );
    assert!(
        mail.contains("If you have already paid, no action is needed"),
        "{mail}"
    );
    assert!(mail.contains(&format!("https://cal.example.com/booking/pay/{token}")));
    assert!(mail.contains("https://cal.example.com/booking/cancel/"));
    let new_deadline = (expires + Duration::seconds(5) + Duration::hours(6))
        .format("%H:%M")
        .to_string();
    assert!(mail.contains(&new_deadline), "{new_deadline} in {mail}");
    let reference = e.inv(&bid).await.reference;
    for word in ["100.00", "UQCD", "TON", &code(&reference), &reference] {
        assert!(!mail.contains(word), "{word} in {mail}");
    }
    assert_eq!(settled_mails_to(&mails, "guest@test.com").await.len(), 1);
    assert!(settled_mails_to(&mails, "test@example.com")
        .await
        .is_empty());
}

/// The guest asked for the payment link by email (`link_emailed_at`).
async fn prove_address(e: &Env, bid: &str) {
    sqlx::query("UPDATE ton_invoices SET link_emailed_at = ? WHERE booking_id = ?")
        .bind(stamp(e.now))
        .bind(bid)
        .execute(&e.pool)
        .await
        .unwrap();
}

/// Renew the window that ended at `expires`, then let the indexer read
/// past the renewal so its notice goes out.
async fn renew_and_notify(e: &Env, expires: chrono::DateTime<Utc>) {
    assert_eq!(e.step(expires + Duration::seconds(5)).await.renewed, 1);
    let later = expires + Duration::minutes(4);
    e.chain.height(later.timestamp() as u64);
    e.step(later).await;
}

async fn parse_stamp_of(e: &Env, bid: &str) -> chrono::DateTime<Utc> {
    let s: String = sqlx::query_scalar("SELECT expires_at FROM ton_invoices WHERE booking_id = ?")
        .bind(bid)
        .fetch_one(&e.pool)
        .await
        .unwrap();
    crate::web::ton_pay::parse_stamp(&s).unwrap()
}

/// A transfer sent in the first window that the indexer only shows after
/// the renewal still counts: its time is before every later settle_by.
#[tokio::test]
async fn a_first_window_transfer_seen_after_the_renewal_confirms() {
    let e = ton_env(&RENEW).await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let at = opened.expires_at + Duration::seconds(5);
    assert_eq!(e.step(at).await.renewed, 1);
    let tx = e.pay_to(
        OUR,
        (opened.expires_at - Duration::minutes(1)).timestamp(),
        HUNDRED,
        &opened.reference,
    );
    e.chain.push(tx);
    e.chain
        .height((at + Duration::minutes(2)).timestamp() as u64);
    let r = e.step(at + Duration::minutes(2)).await;
    assert_eq!(r.credited, 1);
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
}

/// After an outage past the old settle_by: the renewal runs before any
/// transfer is read, so one sent between the old and the new settle_by,
/// shown in that same step, is credited, not late.
#[tokio::test]
async fn the_renewal_runs_before_transfers_are_read() {
    let e = ton_env(&RENEW).await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let old_settle_by = e.inv(&bid).await.settle_by;
    let at = old_settle_by + Duration::minutes(10);
    let tx = e.pay_to(
        OUR,
        (old_settle_by + Duration::minutes(2)).timestamp(),
        HUNDRED,
        &opened.reference,
    );
    e.chain.push(tx);
    e.chain.height(at.timestamp() as u64);
    let r = e.step(at).await;
    assert_eq!((r.renewed, r.credited, r.expired), (1, 1, 0));
    assert_eq!(e.payments().await[0].0, "credited");
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
}

/// A part in the first window, the rest in the second: confirmed.
#[tokio::test]
async fn a_partial_payment_carries_over_the_renewal() {
    let e = ton_env(&RENEW).await;
    let (bid, opened) = e.booking("guest@example.com").await;
    e.pay(HUNDRED / 4, &opened.reference);
    e.step(e.now).await;
    assert_eq!(e.inv(&bid).await.paid as u128, HUNDRED / 4);
    let at = opened.expires_at + Duration::seconds(5);
    assert_eq!(e.step(at).await.renewed, 1);
    assert_eq!(e.inv(&bid).await.paid as u128, HUNDRED / 4, "kept");
    let rest = e.pay_to(OUR, at.timestamp() + 60, HUNDRED * 3 / 4, &opened.reference);
    e.chain.push(rest);
    e.chain
        .height((at + Duration::minutes(2)).timestamp() as u64);
    e.step(at + Duration::minutes(2)).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
}

/// A meeting nearer than the threshold expires as before.
#[tokio::test]
async fn a_near_meeting_is_released_as_before() {
    let e = ton_env(&RENEW).await;
    let (bid, opened) = e
        .booking_at(
            "near@example.com",
            e.now + Duration::hours(23),
            "198.51.100.7",
        )
        .await
        .unwrap();
    let r = e.step(opened.expires_at + Duration::seconds(5)).await;
    assert_eq!(r.renewed, 0);
    let settle_by = e.inv(&bid).await.settle_by;
    e.chain
        .height((settle_by + Duration::minutes(3)).timestamp() as u64);
    let r = e.step(settle_by + Duration::minutes(4)).await;
    assert_eq!((r.renewed, r.expired), (0, 1));
    assert_eq!(e.inv(&bid).await.status, "expired");
    assert_eq!(renewed_alerts(&e), 0);
}

/// Renewals used up: released after the last window, and the guest, who
/// was asked to pay again, hears that it was released.
#[tokio::test]
async fn used_up_renewals_release_and_tell_the_guest() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let _guard = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", "https://cal.example.com");
    let e = ton_env(&[("CALRS_TON_MAX_RENEWALS", "1")]).await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (bid, _) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    prove_address(&e, &bid).await;
    let first = parse_stamp_of(&e, &bid).await;
    assert_eq!(e.step(first + Duration::seconds(5)).await.renewed, 1);
    let second = parse_stamp_of(&e, &bid).await;
    assert_eq!(e.step(second + Duration::seconds(5)).await.renewed, 0);
    let settle_by = e.inv(&bid).await.settle_by;
    e.chain
        .height((settle_by + Duration::minutes(3)).timestamp() as u64);
    let at = settle_by + Duration::minutes(4);
    assert_eq!(e.step(at).await.expired, 1);
    sweep_pending_requests(&e.pool, &[0u8; 32], at + Duration::seconds(30)).await;
    drain_emails(std::time::Duration::from_secs(5)).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "declined");
    let mails = settled_mails_to(&mails, "guest@test.com").await;
    assert_eq!(
        mails.len(),
        1,
        "the release (renewal emails go by the sink)"
    );
    assert!(mails[0].contains("did not arrive in time"), "{}", mails[0]);
}

/// No renewal when the booking already ended (the sweep won), under
/// capacity pressure, or for a type whose price was switched off.
#[tokio::test]
async fn no_renewal_for_an_ended_booking_under_pressure_or_without_a_price() {
    let e = ton_env(&RENEW).await;
    let (bid, opened) = e.booking("a@example.com").await;
    let at = opened.expires_at + Duration::seconds(5);
    sqlx::query("UPDATE bookings SET status = 'declined' WHERE id = ?")
        .bind(&bid)
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(e.step(at).await.renewed, 0);
    assert_eq!(snapshot(&e, &bid).await.8, 0);

    let (bid, opened) = e
        .booking_at("b@example.com", e.now + Duration::days(4), "198.51.100.3")
        .await
        .unwrap();
    let at = opened.expires_at + Duration::seconds(5);
    e.rt.note_capacity_refusal(at - Duration::minutes(10));
    assert_eq!(e.step(at).await.renewed, 0);
    // An hour after the last refusal: renewed again.
    let at = at + Duration::minutes(51);
    sqlx::query("UPDATE ton_prices SET enabled = 0")
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(e.step(at).await.renewed, 0);
    sqlx::query("UPDATE ton_prices SET enabled = 1")
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(e.step(at).await.renewed, 1);
    assert_eq!(snapshot(&e, &bid).await.8, 1);
}

/// `CALRS_TON_MAX_RENEWALS=0` (as every other TON test runs): the first
/// deadline releases, exactly as before.
#[tokio::test]
async fn renewals_off_keeps_the_first_deadline() {
    let e = ton_env(&[]).await;
    let (bid, opened) = e.booking("guest@example.com").await;
    assert_eq!(snapshot(&e, &bid).await.9, 0);
    assert_eq!(
        e.step(opened.expires_at + Duration::seconds(5))
            .await
            .renewed,
        0
    );
    let settle_by = e.inv(&bid).await.settle_by;
    e.chain
        .height((settle_by + Duration::minutes(3)).timestamp() as u64);
    assert_eq!(e.step(settle_by + Duration::minutes(4)).await.expired, 1);
}

// --- review fixes -----------------------------------------------------------------

/// The guest pays 30 seconds before the window ends and the indexer is a
/// minute behind: the window is renewed at once (so the late-indexed
/// transfer still counts), but "pay again" goes out only once the indexer
/// has read past the renewal, and not at all once the payment is seen.
#[tokio::test]
async fn no_pay_again_notice_while_the_indexer_lags_behind_a_payment() {
    let e = ton_env(&RENEW).await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let at = opened.expires_at + Duration::seconds(5);
    // The chain stands at the window's end: the transfer is not indexed.
    e.chain
        .height((opened.expires_at - Duration::seconds(40)).timestamp() as u64);
    assert_eq!(e.step(at).await.renewed, 1);
    assert_eq!(renewed_alerts(&e), 0, "no notice in the renewal step");
    e.step(at + Duration::seconds(10)).await;
    assert_eq!(renewed_alerts(&e), 0, "the indexer is still behind");
    // A minute later the transfer shows up.
    let tx = e.pay_to(
        OUR,
        (opened.expires_at - Duration::seconds(30)).timestamp(),
        HUNDRED,
        &opened.reference,
    );
    e.chain.push(tx);
    e.chain
        .height((at + Duration::minutes(5)).timestamp() as u64);
    let r = e.step(at + Duration::minutes(5)).await;
    assert_eq!(r.credited, 1);
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
    e.step(at + Duration::minutes(6)).await;
    assert_eq!(renewed_alerts(&e), 0, "paid: never asked to pay again");
}

/// The indexer fails in the renewal step: the deadlines move, the guest
/// is not told until a later step reads the address past the renewal.
#[tokio::test]
async fn no_pay_again_notice_from_a_step_whose_indexer_failed() {
    let e = ton_env(&RENEW).await;
    let (_bid, opened) = e.booking("guest@example.com").await;
    let at = opened.expires_at + Duration::seconds(5);
    e.chain
        .height((at + Duration::minutes(4)).timestamp() as u64);
    e.chain.edit(|c| {
        c.fail_next = vec![IndexerError::Server(502)];
    });
    let r = e.step(at).await;
    assert!(r.error.is_some());
    assert_eq!(r.renewed, 1);
    assert_eq!(renewed_alerts(&e), 0);
    // Still failing: nothing.
    e.chain.edit(|c| {
        c.fail_next = vec![IndexerError::Server(502)];
    });
    e.step(at + Duration::seconds(20)).await;
    assert_eq!(renewed_alerts(&e), 0);
    // Back, and past the renewal: told once.
    e.step(at + Duration::minutes(4)).await;
    assert_eq!(renewed_alerts(&e), 1);
}

/// Every capacity refusal pauses renewals for an hour, not only the one
/// whose alert was emailed (claimed once an hour).
#[tokio::test]
async fn every_capacity_refusal_pauses_renewals() {
    let e = ton_env(&[
        ("CALRS_TON_MAX_RENEWALS", "2"),
        ("CALRS_TON_MAX_OPEN_TOTAL", "1"),
    ])
    .await;
    let (_bid, opened) = e.booking("held@example.com").await;
    let start = e.now + Duration::days(4);
    let refuse = |now: chrono::DateTime<Utc>| {
        invoice::prepare_invoice(
            &e.state,
            &e.et_id,
            start,
            0,
            "other@example.com",
            "198.51.100.9",
            now,
        )
    };
    let end = opened.expires_at + Duration::seconds(5);
    // Refused at T-80 min (alert claimed) and at T-30 min (no new alert).
    assert_eq!(
        refuse(end - Duration::minutes(80)).await.unwrap_err(),
        Refusal::Capacity
    );
    assert_eq!(
        refuse(end - Duration::minutes(30)).await.unwrap_err(),
        Refusal::Capacity
    );
    let claimed: String =
        sqlx::query_scalar("SELECT last_sent_at FROM ton_alerts WHERE kind = 'capacity'")
            .fetch_one(&e.pool)
            .await
            .unwrap();
    assert_eq!(claimed, stamp(end - Duration::minutes(80)), "one alert");
    assert!(e.rt.capacity_pressed(end));
    assert_eq!(e.step(end).await.renewed, 0, "the T-30 refusal counts");
    let later = end + Duration::minutes(31);
    assert!(!e.rt.capacity_pressed(later));
    assert_eq!(e.step(later).await.renewed, 1);
}

/// A guest whose typed address is not proven (no "email me the link", no
/// payment) gets no renewal email and no release email: anyone can type a
/// stranger's address into the form. Their linked Telegram is still told.
#[tokio::test]
async fn an_unproven_address_gets_no_renewal_or_release_email() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let _guard = crate::test_support::EnvGuard::new();
    std::env::set_var("CALRS_BASE_URL", "https://cal.example.com");
    let e = ton_env(&[("CALRS_TON_MAX_RENEWALS", "1")]).await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&e.pool, port).await;
    let (bid, _) = e
        .book_http("victim@test.com", "10:00", "198.51.100.1")
        .await;
    let first = parse_stamp_of(&e, &bid).await;
    renew_and_notify(&e, first).await;
    assert_eq!(
        renewed_alerts(&e),
        1,
        "the alert itself goes out (Telegram)"
    );
    for alert in e.sink.all() {
        if matches!(alert, TonAlert::Renewed { .. }) {
            crate::web::ton_pay::alerts::deliver_now(&e.pool, &[0u8; 32], alert).await;
        }
    }
    let second = parse_stamp_of(&e, &bid).await;
    assert_eq!(e.step(second + Duration::seconds(5)).await.renewed, 0);
    let settle_by = e.inv(&bid).await.settle_by;
    e.chain
        .height((settle_by + Duration::minutes(3)).timestamp() as u64);
    let at = settle_by + Duration::minutes(4);
    assert_eq!(e.step(at).await.expired, 1);
    sweep_pending_requests(&e.pool, &[0u8; 32], at + Duration::seconds(30)).await;
    drain_emails(std::time::Duration::from_secs(5)).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "declined");
    assert!(settled_mails_to(&mails, "victim@test.com").await.is_empty());
}

/// A renewal lasts hours, not the first window's 30 minutes: both
/// renewals do not run out within the hour.
#[tokio::test]
async fn a_renewed_window_lasts_the_renewal_length() {
    let e = ton_env(&[
        ("CALRS_TON_MAX_RENEWALS", "2"),
        ("CALRS_TON_RENEW_WINDOW_MIN", "180"),
    ])
    .await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let at = opened.expires_at + Duration::seconds(5);
    assert_eq!(e.step(at).await.renewed, 1);
    assert_eq!(snapshot(&e, &bid).await.7, stamp(at + Duration::hours(3)));
    assert_eq!(e.step(at + Duration::minutes(90)).await.renewed, 0);
}

/// The payment page reloads when the deadline moves (a renewal keeps the
/// state "waiting"): /status carries the deadline and the renewal count,
/// and the page compares both.
#[tokio::test]
async fn the_payment_page_notices_a_renewal() {
    let e = ton_env(&RENEW).await;
    let (bid, opened) = e.booking("guest@example.com").await;
    let token = snapshot(&e, &bid).await.5;
    let (_, _, page) = e.get_page(&format!("/booking/pay/{token}")).await;
    let before = stamp(opened.expires_at);
    assert!(
        page.contains(&format!(
            "var shownExpiry = \"{before}\", shownRenewals = 0;"
        )),
        "{page}"
    );
    assert!(page.contains("j.expires_at !== shownExpiry"), "{page}");
    e.step(opened.expires_at + Duration::seconds(5)).await;
    let (_, _, status) = e.get_page(&format!("/booking/pay/{token}/status")).await;
    let j: serde_json::Value = serde_json::from_str(&status).unwrap();
    assert_eq!(j["state"], "waiting");
    assert_eq!(j["renewals"], 1);
    assert_ne!(j["expires_at"].as_str().unwrap(), before);
}

/// A switched-off price makes no type require Telegram: not through the
/// migration, and not through the form when an amount is typed with the
/// price unchecked.
#[tokio::test]
async fn a_price_that_is_off_never_requires_telegram() {
    let pool = setup_test_db().await;
    let (_, _, et_id) = seed_test_data(&pool).await;
    sqlx::raw_sql(
        "DELETE FROM _migrations WHERE name = 'ilia_004_ton_price_currency';
         DROP TABLE ton_prices;
         CREATE TABLE ton_prices (
             event_type_id   TEXT PRIMARY KEY REFERENCES event_types(id) ON DELETE CASCADE,
             price_usd_cents INTEGER NOT NULL CHECK (price_usd_cents BETWEEN 100 AND 1000000),
             window_min      INTEGER CHECK (window_min IS NULL OR window_min BETWEEN 10 AND 240),
             enabled         INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
             updated_at      TEXT NOT NULL);
         ALTER TABLE ton_invoices DROP COLUMN currency;",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO ton_prices (event_type_id, price_usd_cents, enabled, updated_at)
         VALUES (?, 4000, 0, '2026-11-01T00:00:00Z')",
    )
    .bind(&et_id)
    .execute(&pool)
    .await
    .unwrap();
    crate::db::migrate(&pool).await.unwrap();
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_event_types")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0, "a disabled price row");

    // The form: an amount typed, the price left unchecked.
    let e = ton_env(&[]).await;
    sqlx::query("DELETE FROM ton_prices")
        .execute(&e.pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM telegram_event_types")
        .execute(&e.pool)
        .await
        .unwrap();
    let session = e.session().await;
    let edit = "/dashboard/event-types/test-meeting/edit";
    let body = format!("_csrf={CSRF}&title=Test+Meeting&slug=test-meeting&duration_min=30&location_value=https%3A%2F%2Fmeet.example.com&avail_days=1,2,3,4,5&avail_start=09:00&avail_end=17:00&ton_price_currency=TON&ton_price_amount=50&ton_guest_telegram=required");
    let resp = e
        .app()
        .oneshot(post_form(edit, &session, CSRF, &body))
        .await
        .unwrap();
    assert!(resp.status().is_redirection());
    let enabled: i64 = sqlx::query_scalar("SELECT enabled FROM ton_prices")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(enabled, 0, "the amount is kept, switched off");
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_event_types")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(rows, 0, "a free type does not start requiring Telegram");
    // A mode the owner already chose can still be lowered with the price off.
    sqlx::query("INSERT INTO telegram_event_types (event_type_id, guest_mode, updated_at) VALUES (?, 'required', datetime('now'))")
        .bind(&e.et_id)
        .execute(&e.pool)
        .await
        .unwrap();
    let body = body.replace("ton_guest_telegram=required", "ton_guest_telegram=optional");
    e.app()
        .oneshot(post_form(edit, &session, CSRF, &body))
        .await
        .unwrap();
    assert_eq!(
        crate::telegram::store::event_type_mode(&e.pool, &e.et_id).await,
        "optional"
    );
}
