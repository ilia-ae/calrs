//! ilia fork: the TON price by demand (the ladder; ilia-booking pricing
//! design §1 with its critique). The count `n`, the quote before the
//! booking transaction and the recount inside it, the free step that waits
//! for the owner's approval, the per-guest limit, the price fixed in the
//! invoice, both booking handlers, two guests at once, the event type
//! form. A child of the TON tests, for their chain, rates and helpers.

use super::*;
use crate::ton::amount::NANO_PER_TON;
use crate::ton::ladder::Step;
use crate::web::ton_pay::invoice::{BookingAsk, Prepared};
use crate::web::ton_pay::store;

const T: u128 = NANO_PER_TON;

/// The ladder of these tests: free, then 20, then 40 TON (the ceiling).
const LADDER: (&str, &str) = ("CALRS_TON_LADDER", "free,20,40");

async fn ladder_env(extra: &[(&'static str, &str)]) -> Env {
    ladder_env_in(extra, setup_test_db().await).await
}

/// The test type priced by the ladder (`extra` may set another one).
async fn ladder_env_in(extra: &[(&'static str, &str)], pool: SqlitePool) -> Env {
    let mut vars = vec![LADDER];
    vars.extend_from_slice(extra);
    let e = env_priced_in(&vars, Some(100 * T), pool).await;
    sqlx::query(
        "UPDATE ton_prices SET mode = 'ladder', currency = 'TON', price_nano = NULL,
             price_usd_cents = NULL",
    )
    .execute(&e.pool)
    .await
    .unwrap();
    e
}

/// A file database in WAL mode, as `calrs serve` opens it: two
/// transactions on separate connections, like two requests.
async fn file_pool(dir: &TestDataDir) -> SqlitePool {
    let pool = crate::db::connect(dir).await.unwrap();
    crate::db::migrate(&pool).await.unwrap();
    pool
}

async fn owner(e: &Env) -> String {
    sqlx::query_scalar("SELECT id FROM users WHERE username = 'testuser'")
        .fetch_one(&e.pool)
        .await
        .unwrap()
}

async fn demand(e: &Env, exclude: Option<&str>) -> i64 {
    let owner = owner(e).await;
    let mut conn = e.pool.acquire().await.unwrap();
    store::demand_n(&mut conn, &owner, e.now, exclude)
        .await
        .unwrap()
}

fn ask<'a>(
    e: &'a Env,
    email: &'a str,
    ip: &'a str,
    start: chrono::DateTime<Utc>,
    quoted: Option<Step>,
) -> BookingAsk<'a> {
    BookingAsk {
        event_type_id: &e.et_id,
        start_utc: start,
        min_notice_min: 0,
        guest_email: email,
        client_ip: ip,
        telegram: None,
        quoted,
    }
}

/// `invoice::prepare` at the environment's clock.
async fn prep(e: &Env, a: BookingAsk<'_>) -> Result<Prepared, Refusal> {
    invoice::prepare(&e.state, &a, e.now).await
}

/// The booking handlers' INSERT, on the transaction's connection.
async fn insert_on(
    conn: &mut sqlx::SqliteConnection,
    et_id: &str,
    email: &str,
    start: chrono::DateTime<Utc>,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO bookings (time_version, id, event_type_id, uid, guest_name, guest_email,
             guest_timezone, start_at, end_at, status, cancel_token, reschedule_token, confirm_token)
         VALUES (1, ?, ?, ?, 'Guest', ?, 'UTC', ?, ?, 'pending', ?, ?, ?)",
    )
    .bind(&id)
    .bind(et_id)
    .bind(format!("uid-{id}"))
    .bind(email)
    .bind(utc_stamp(start))
    .bind(utc_stamp(start + Duration::minutes(30)))
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(format!("confirm-{id}"))
    .execute(&mut *conn)
    .await
    .unwrap();
    id
}

/// Book `prepared` as the handlers do: BEGIN, INSERT, `invoice::open` on
/// the same connection, COMMIT (or ROLLBACK on a refusal).
async fn book(
    e: &Env,
    prepared: &Prepared,
    email: &str,
    start: chrono::DateTime<Utc>,
) -> Result<(String, Option<OpenedInvoice>), Refusal> {
    let mut tx = e.pool.begin().await.unwrap();
    let id = insert_on(&mut tx, &e.et_id, email, start).await;
    match invoice::open(&mut tx, &id, prepared).await {
        Ok(opened) => {
            tx.commit().await.unwrap();
            Ok((id, opened))
        }
        Err(r) => {
            tx.rollback().await.unwrap();
            Err(r)
        }
    }
}

async fn count(e: &Env, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(&e.pool).await.unwrap()
}

/// The demand counts the owner's live bookings ahead: confirmed ones,
/// free requests awaiting approval, paid or waived ones still pending;
/// never a payment hold, an ended booking, the past, another user's, the
/// booking being made, or one beyond the horizon.
#[tokio::test]
async fn the_demand_counts_live_bookings_ahead_only() {
    let e = ladder_env(&[]).await;
    let day = |d: i64| e.now + Duration::days(d);
    assert_eq!(demand(&e, None).await, 0);
    let confirmed =
        insert_v1_booking(&e.pool, &e.et_id, "c@x.io", day(2), 30, "confirmed", &[]).await;
    assert_eq!(demand(&e, None).await, 1);
    assert_eq!(demand(&e, Some(&confirmed)).await, 0, "the booking made");
    insert_v1_booking(&e.pool, &e.et_id, "p@x.io", day(3), 30, "pending", &[]).await;
    assert_eq!(
        demand(&e, None).await,
        2,
        "a free request awaiting approval"
    );

    // A payment hold does not count, a paid or waived one does.
    let p = invoice::prepare(
        &e.state,
        &ask(
            &e,
            "hold@x.io",
            "198.51.100.1",
            day(4),
            Some(Step::Ton(40 * T)),
        ),
        e.now,
    )
    .await
    .unwrap();
    let (held, opened) = book(&e, &p, "hold@x.io", day(4)).await.unwrap();
    assert_eq!(opened.unwrap().amount_nano as u128, 40 * T);
    assert_eq!(demand(&e, None).await, 2, "an open invoice is a hold");
    for (status, n) in [("paid", 3), ("waived", 3), ("expired", 2), ("closed", 2)] {
        sqlx::query("UPDATE ton_invoices SET status = ? WHERE booking_id = ?")
            .bind(status)
            .bind(&held)
            .execute(&e.pool)
            .await
            .unwrap();
        assert_eq!(demand(&e, None).await, n, "{status}");
    }

    for status in ["cancelled", "declined"] {
        insert_v1_booking(&e.pool, &e.et_id, "x@x.io", day(5), 30, status, &[]).await;
    }
    insert_v1_booking(&e.pool, &e.et_id, "old@x.io", day(-1), 30, "confirmed", &[]).await;
    // Another user's booking.
    let other = insert_role_user(&e.pool, "other@example.com", "user").await;
    let account = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO accounts (id, name, email, timezone, user_id)
         VALUES (?, 'Other', 'other@example.com', 'UTC', ?)",
    )
    .bind(&account)
    .bind(&other)
    .execute(&e.pool)
    .await
    .unwrap();
    let other_et = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO event_types (id, account_id, slug, title, duration_min, buffer_before,
             buffer_after, min_notice_min, enabled)
         VALUES (?, ?, 'other-meeting', 'Other', 30, 0, 0, 0, 1)",
    )
    .bind(&other_et)
    .bind(&account)
    .execute(&e.pool)
    .await
    .unwrap();
    insert_v1_booking(&e.pool, &other_et, "y@x.io", day(2), 30, "confirmed", &[]).await;
    assert_eq!(demand(&e, None).await, 2);

    // Beyond the booking horizon: not demand.
    insert_v1_booking(&e.pool, &e.et_id, "far@x.io", day(20), 30, "confirmed", &[]).await;
    assert_eq!(demand(&e, None).await, 3, "no horizon in tests by default");
    let _horizon = crate::horizon::test_limit(14);
    assert_eq!(demand(&e, None).await, 2);
}

/// Inside the transaction the price is recounted: a lower price is charged
/// without asking; a higher one, or a switch between free and priced, is
/// asked again and nothing is booked. The invoice keeps the step, `n` and
/// the quote; a later ladder change never touches it.
#[tokio::test]
async fn the_price_is_recounted_in_the_transaction_and_fixed_in_the_invoice() {
    let e = ladder_env(&[]).await;
    let start = e.now + Duration::days(3);
    let ip = "198.51.100.1";
    // n = 0: free. A priced quote, or none, is asked again.
    let p = |email: &'static str, ip: &'static str, start, quoted| {
        prep(&e, ask(&e, email, ip, start, quoted))
    };
    assert_eq!(
        p("a@x.io", ip, start, Some(Step::Ton(20 * T))).await.err(),
        Some(Refusal::Requote {
            quoted: Some(Step::Ton(20 * T)),
            now: Step::Free,
            free_used: false,
        })
    );
    assert_eq!(
        p("a@x.io", ip, start, None).await.err(),
        Some(Refusal::Requote {
            quoted: None,
            now: Step::Free,
            free_used: false,
        })
    );
    insert_v1_booking(
        &e.pool,
        &e.et_id,
        "c1@x.io",
        start + Duration::days(1),
        30,
        "confirmed",
        &[],
    )
    .await;
    // n = 1: 20 TON. Quoted 40 (shown earlier, when n was 2): 20 is charged.
    let low = p("low@x.io", ip, start, Some(Step::Ton(40 * T)))
        .await
        .unwrap();
    let (bid, opened) = book(&e, &low, "low@x.io", start).await.unwrap();
    assert_eq!(opened.unwrap().amount_nano as u128, 20 * T);
    let row: (String, Option<i64>, Option<i64>, i64, String) = sqlx::query_as(
        "SELECT pricing_mode, demand_n, quoted_nano, amount_nano, currency
         FROM ton_invoices WHERE booking_id = ?",
    )
    .bind(&bid)
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(
        row,
        (
            "ladder".into(),
            Some(1),
            Some((40 * T) as i64),
            (20 * T) as i64,
            "TON".into()
        )
    );
    assert_eq!(e.rates.calls(), 0, "a ladder price asks no rate");

    // Quoted 20 before the transaction; another booking is confirmed
    // meanwhile, so inside it the price is 40: asked again, nothing booked.
    let later = start + Duration::hours(1);
    let up = p("up@x.io", "198.51.101.1", later, Some(Step::Ton(20 * T)))
        .await
        .unwrap();
    insert_v1_booking(
        &e.pool,
        &e.et_id,
        "c2@x.io",
        start + Duration::days(2),
        30,
        "confirmed",
        &[],
    )
    .await;
    let bookings = count(&e, "SELECT COUNT(*) FROM bookings").await;
    assert_eq!(
        book(&e, &up, "up@x.io", later).await.err(),
        Some(Refusal::Requote {
            quoted: Some(Step::Ton(20 * T)),
            now: Step::Ton(40 * T),
            free_used: false,
        })
    );
    assert_eq!(count(&e, "SELECT COUNT(*) FROM bookings").await, bookings);
    // Past the last step the price stays at the ceiling.
    for d in 3..6 {
        insert_v1_booking(
            &e.pool,
            &e.et_id,
            "cx@x.io",
            start + Duration::days(d),
            30,
            "confirmed",
            &[],
        )
        .await;
    }
    let top = p("top@x.io", "198.51.102.1", later, Some(Step::Ton(40 * T)))
        .await
        .unwrap();
    let (_, opened) = book(&e, &top, "top@x.io", later).await.unwrap();
    assert_eq!(opened.unwrap().amount_nano as u128, 40 * T);

    // A new ladder applies to new bookings only: the open invoice keeps
    // its 20 TON, and paying them confirms it.
    let rt = runtime(
        &vars(&[("CALRS_TON_LADDER", "free,90,99")]),
        e.chain.clone(),
        e.rates.clone(),
        e.sink.clone(),
    );
    e.state.ton.install(rt);
    assert_eq!(e.inv(&bid).await.amount as u128, 20 * T);
    let reference = e.inv(&bid).await.reference;
    e.pay(20 * T, &code(&reference));
    e.step(e.now).await;
    assert_eq!(booking_status(&e.pool, &bid).await, "confirmed");
}

/// The free step: no invoice, the booking waits for the owner's approval
/// and holds the time; one per guest (email, Telegram username or client
/// network) per `CALRS_TON_FREE_PER_GUEST_DAYS`, a cancelled one included,
/// checked before the transaction and inside it.
#[tokio::test]
async fn the_free_step_waits_for_approval_and_once_per_guest() {
    let e = ladder_env(&[("CALRS_TON_LADDER", "free,free,20")]).await;
    let start = e.now + Duration::days(3);
    let free = Some(Step::Free);
    let mut first = ask(&e, "free@x.io", "198.51.100.7", start, free);
    first.telegram = Some("@Guest_One");
    let p = invoice::prepare(&e.state, &first, e.now).await.unwrap();
    assert!(matches!(p, Prepared::Free(_)) && p.holds());
    let (bid, opened) = book(&e, &p, "free@x.io", start).await.unwrap();
    assert_eq!(opened, None, "no invoice");
    assert_eq!(booking_status(&e.pool, &bid).await, "pending");
    assert_eq!(
        count(&e, "SELECT COUNT(*) FROM ton_invoices").await,
        0,
        "nothing to pay"
    );
    let (n, tg): (i64, Option<String>) = sqlx::query_as(
        "SELECT demand_n, telegram_norm FROM ton_free_bookings WHERE booking_id = ?",
    )
    .bind(&bid)
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!((n, tg.as_deref()), (0, Some("guest_one")));
    // The owner may approve it (no unpaid invoice), and it holds the time
    // although the type's flags say a request does not.
    assert!(!invoice::approval_refused(&e.pool, &bid).await);
    let user = owner(&e).await;
    let held: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM bookings b JOIN event_types et ON et.id = b.event_type_id
         JOIN accounts a ON a.id = et.account_id WHERE b.id = ? AND {}",
        crate::booking_time::user_busy_bookings(true)
    ))
    .bind(&bid)
    .bind(&user)
    .bind(&user)
    .bind(&user)
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(held, 1);
    assert_eq!(demand(&e, None).await, 1);

    // Cancelled: the step is free again, but not for this guest: for them
    // it is the first price (asked, never a lockout).
    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
        .bind(&bid)
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(demand(&e, None).await, 0);
    let later = start + Duration::hours(1);
    let prepare = |a| prep(&e, a);
    let first_price = Some(Refusal::Requote {
        quoted: free,
        now: Step::Ton(20 * T),
        free_used: true,
    });
    assert_eq!(
        prepare(ask(&e, " FREE@x.io ", "203.0.113.1", later, free))
            .await
            .err(),
        first_price,
        "by email"
    );
    let mut by_tg = ask(&e, "new@x.io", "203.0.113.2", later, free);
    by_tg.telegram = Some("guest_one");
    assert_eq!(prepare(by_tg).await.err(), first_price, "by Telegram");
    assert_eq!(
        prepare(ask(&e, "new@x.io", "198.51.100.99", later, free))
            .await
            .err(),
        first_price,
        "by client network"
    );

    // Two requests of one new guest prepared at once: the second is asked
    // again at the first price inside its transaction, nothing is booked.
    let a = prepare(ask(&e, "twice@x.io", "203.0.113.3", later, free))
        .await
        .unwrap();
    let b = prepare(ask(
        &e,
        "twice@x.io",
        "203.0.113.3",
        later + Duration::hours(1),
        free,
    ))
    .await
    .unwrap();
    book(&e, &a, "twice@x.io", later).await.unwrap();
    let bookings = count(&e, "SELECT COUNT(*) FROM bookings").await;
    assert_eq!(
        book(&e, &b, "twice@x.io", later + Duration::hours(1))
            .await
            .err(),
        first_price
    );
    assert_eq!(count(&e, "SELECT COUNT(*) FROM bookings").await, bookings);

    // The limit switched off (0 days): only the ladder decides.
    let open = ladder_env(&[
        ("CALRS_TON_LADDER", "free,free,20"),
        ("CALRS_TON_FREE_PER_GUEST_DAYS", "0"),
    ])
    .await;
    for h in 0..2 {
        let at = start + Duration::hours(h);
        let p = invoice::prepare(
            &open.state,
            &ask(&open, "same@x.io", "198.51.100.7", at, free),
            open.now,
        )
        .await
        .unwrap();
        assert!(matches!(p, Prepared::Free(_)));
        book(&open, &p, "same@x.io", at).await.unwrap();
    }
}

/// A guest past the per-guest free limit, on a quiet calendar (n = 0, the
/// step free): never shut out. They are quoted the ladder's first price,
/// prepare and the in-transaction recount both book them at it, and only
/// a ladder without any price refuses them.
#[tokio::test]
async fn a_guest_past_the_free_limit_can_book_at_the_first_price() {
    let e = ladder_env(&[]).await;
    let start = e.now + Duration::days(3);
    let free = Some(Step::Free);
    let first = prep(&e, ask(&e, "back@x.io", "198.51.100.1", start, free))
        .await
        .unwrap();
    let (bid, _) = book(&e, &first, "back@x.io", start).await.unwrap();
    // The meeting is cancelled: n is 0 again, the step is free.
    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = ?")
        .bind(&bid)
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(demand(&e, None).await, 0);

    let later = start + Duration::hours(2);
    assert_eq!(
        prep(&e, ask(&e, "back@x.io", "203.0.113.5", later, free))
            .await
            .err(),
        Some(Refusal::Requote {
            quoted: free,
            now: Step::Ton(20 * T),
            free_used: true,
        }),
        "quoted the first price, not refused"
    );
    let twenty = Some(Step::Ton(20 * T));
    let paid = prep(&e, ask(&e, "back@x.io", "203.0.113.5", later, twenty))
        .await
        .unwrap();
    match &paid {
        Prepared::Invoice(p) => assert_eq!(p.amount_nano as u128, 20 * T),
        other => panic!("an invoice, not {other:?}"),
    }
    // Booked: the recount in the transaction still finds the step free,
    // and still prices it for this guest at the first price.
    let (_, opened) = book(&e, &paid, "back@x.io", later).await.unwrap();
    assert_eq!(opened.unwrap().amount_nano as u128, 20 * T);
    assert_eq!(
        count(&e, "SELECT COUNT(*) FROM ton_free_bookings").await,
        1,
        "no second free step"
    );

    // Someone else at the same network, while the step is free: the same.
    let neighbour = later + Duration::hours(1);
    assert_eq!(
        prep(&e, ask(&e, "nb@y.io", "198.51.100.77", neighbour, free))
            .await
            .err()
            .map(|r| matches!(
                r,
                Refusal::Requote {
                    free_used: true,
                    ..
                }
            )),
        Some(true)
    );

    // Only a ladder that is free all the way refuses: nothing to pay.
    let all_free = ladder_env(&[("CALRS_TON_LADDER", "free,free")]).await;
    let p = prep(
        &all_free,
        ask(&all_free, "once@x.io", "198.51.100.1", start, free),
    )
    .await
    .unwrap();
    book(&all_free, &p, "once@x.io", start).await.unwrap();
    assert_eq!(
        prep(
            &all_free,
            ask(&all_free, "once@x.io", "198.51.100.1", later, free)
        )
        .await
        .err(),
        Some(Refusal::FreeUsed)
    );
}

/// The quote page explains the change neutrally (it may go up or down,
/// after a booking, a cancellation or a ladder change), or that the
/// guest's free step is used; with no quote it asks to confirm.
#[test]
fn the_quote_page_reason_follows_the_cause() {
    use crate::web::ton_pay::flow::quote_why_key;
    let (free, twenty) = (Some(Step::Free), Some(Step::Ton(20 * T)));
    assert_eq!(quote_why_key(twenty, false), "ton-quote-why");
    assert_eq!(quote_why_key(free, false), "ton-quote-why");
    assert_eq!(quote_why_key(free, true), "ton-quote-why-free-used");
    assert_eq!(quote_why_key(None, false), "ton-quote-confirm-help");
    for (lang, _) in crate::i18n::supported_with_labels() {
        let why = crate::i18n::translate(lang, "ton-quote-why", None);
        for stale in ["Someone booked", "кто-то забронировал", "jemand gebucht"] {
            assert!(!why.contains(stale), "{lang}: {why}");
        }
    }
}

/// Two guests quoted the free step at the same moment, on two connections
/// of a WAL database: the second transaction's INSERT waits for the
/// first's commit, its recount sees that booking, and it is asked again at
/// 20 TON. Only one booking is made.
#[tokio::test]
async fn two_guests_at_once_only_one_gets_the_free_step() {
    let dir = TestDataDir::new();
    let e = ladder_env_in(&[], file_pool(&dir).await).await;
    let (start_a, start_b) = (
        e.now + Duration::days(3),
        e.now + Duration::days(3) + Duration::hours(1),
    );
    let free = Some(Step::Free);
    let pa = invoice::prepare(
        &e.state,
        &ask(&e, "a@x.io", "198.51.100.1", start_a, free),
        e.now,
    )
    .await
    .unwrap();
    let pb = invoice::prepare(
        &e.state,
        &ask(&e, "b@y.io", "203.0.113.9", start_b, free),
        e.now,
    )
    .await
    .unwrap();
    assert!(matches!(pa, Prepared::Free(_)) && matches!(pb, Prepared::Free(_)));

    let mut tx_a = e.pool.begin().await.unwrap();
    let a = insert_on(&mut tx_a, &e.et_id, "a@x.io", start_a).await;
    assert_eq!(invoice::open(&mut tx_a, &a, &pa).await, Ok(None));
    let (pool, et_id) = (e.pool.clone(), e.et_id.clone());
    let second = tokio::spawn(async move {
        let mut tx = pool.begin().await.unwrap();
        let b = insert_on(&mut tx, &et_id, "b@y.io", start_b).await;
        let r = invoice::open(&mut tx, &b, &pb).await;
        tx.rollback().await.unwrap();
        r
    });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert!(!second.is_finished(), "B waits for A's write lock");
    tx_a.commit().await.unwrap();
    assert_eq!(
        second.await.unwrap(),
        Err(Refusal::Requote {
            quoted: Some(Step::Free),
            now: Step::Ton(20 * T),
            free_used: false,
        })
    );
    assert_eq!(count(&e, "SELECT COUNT(*) FROM bookings").await, 1);
    assert_eq!(count(&e, "SELECT COUNT(*) FROM ton_free_bookings").await, 1);

    // Both quoted 20 TON at n = 1: both get it (holds do not count).
    for (email, ip, h) in [("c@x.io", "198.51.101.1", 2), ("d@x.io", "198.51.102.1", 3)] {
        let at = start_a + Duration::hours(h);
        let p = invoice::prepare(
            &e.state,
            &ask(&e, email, ip, at, Some(Step::Ton(20 * T))),
            e.now,
        )
        .await
        .unwrap();
        let (_, opened) = book(&e, &p, email, at).await.unwrap();
        assert_eq!(opened.unwrap().amount_nano as u128, 20 * T);
    }
}

/// A booking POST as the form sends it, with `extra` fields.
async fn post_book(
    e: &Env,
    uri: &str,
    email: &str,
    time: &str,
    ip: &str,
    extra: &str,
) -> (axum::http::StatusCode, Option<String>, String) {
    let body = format!(
        "_csrf={CSRF}&date={}&time={time}&name=Guest+Name&email={}&tz=UTC{extra}",
        far_future_monday(),
        urlencoding::encode(email)
    );
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(uri)
        .header("cookie", format!("__Host-calrs_csrf={CSRF}"))
        .header("content-type", "application/x-www-form-urlencoded")
        .header("x-forwarded-for", ip)
        .body(Body::from(body))
        .unwrap();
    let resp = e.app().oneshot(req).await.unwrap();
    let status = resp.status();
    let location = resp
        .headers()
        .get("location")
        .map(|v| v.to_str().unwrap().to_string());
    (status, location, unescape(&body_string(resp).await))
}

const QUOTE_TITLE: &str = "The price has changed";

/// Both booking handlers (`/u/{user}/{slug}/book` and the legacy
/// `/{slug}/book`): the pages show the price now and send it as the quote;
/// a free step is a request awaiting approval; a stale quote gets the
/// "price changed" page with the same form, and nothing is booked; the
/// confirmed quote opens an invoice at the price now.
#[tokio::test]
async fn both_booking_handlers_show_quote_and_lock_the_price() {
    let e = ladder_env(&[]).await;
    let (_, _, slots) = e.get_page("/u/testuser/test-meeting").await;
    assert!(
        slots.contains("Now: free, the host confirms the request"),
        "{slots}"
    );
    assert!(slots.contains("Free while the calendar is empty, then 20 to 40 TON as it fills up."));
    assert!(slots.contains("The network fee is paid by your wallet."));
    let form_uri = format!(
        "/u/testuser/test-meeting/book?date={}&time=10:00&tz=UTC",
        far_future_monday()
    );
    let (status, _, form) = e.get_page(&form_uri).await;
    assert_eq!(status, 200);
    assert!(
        form.contains(r#"<input type="hidden" name="quoted_nano" value="0">"#),
        "{form}"
    );

    // The free step through the user handler: pending, no invoice.
    let (status, location, _) = post_book(
        &e,
        "/u/testuser/test-meeting/book",
        "first@x.io",
        "10:00",
        "198.51.100.1",
        "&quoted_nano=0",
    )
    .await;
    assert_eq!((status, location), (axum::http::StatusCode::OK, None));
    let first: String = sqlx::query_scalar("SELECT id FROM bookings WHERE guest_email = ?")
        .bind("first@x.io")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(booking_status(&e.pool, &first).await, "pending");
    assert_eq!(count(&e, "SELECT COUNT(*) FROM ton_invoices").await, 0);
    assert_eq!(count(&e, "SELECT COUNT(*) FROM ton_free_bookings").await, 1);
    let (_, _, slots) = e.get_page("/test-meeting").await;
    assert!(slots.contains("Now: 20 TON"), "{slots}");

    // A stale free quote through the legacy handler: the quote page.
    let (status, _, page) = post_book(
        &e,
        "/test-meeting/book",
        "second@x.io",
        "11:00",
        "198.51.101.1",
        "&quoted_nano=0",
    )
    .await;
    assert_eq!(status, 200);
    assert!(page.contains(QUOTE_TITLE), "{page}");
    assert!(page.contains("You were shown a free booking."), "{page}");
    assert!(
        page.contains(&crate::i18n::translate("en", "ton-quote-why", None)),
        "a neutral reason: {page}"
    );
    assert!(!page.contains("Someone booked"), "{page}");
    assert!(page.contains("Book for 20 TON"), "{page}");
    assert!(page.contains(r#"action="/test-meeting/book""#), "{page}");
    assert!(
        page.contains(r#"name="email" value="second@x.io""#),
        "{page}"
    );
    assert!(
        page.contains(r#"name="quoted_nano" value="20000000000""#),
        "{page}"
    );
    assert_eq!(count(&e, "SELECT COUNT(*) FROM bookings").await, 1);
    // Confirmed at 20 TON: the invoice, at 20 TON.
    let (status, location, _) = post_book(
        &e,
        "/test-meeting/book",
        "second@x.io",
        "11:00",
        "198.51.101.1",
        "&quoted_nano=20000000000",
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::SEE_OTHER);
    assert!(location.unwrap().starts_with("/booking/pay/"));
    let row: (i64, String, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT i.amount_nano, i.pricing_mode, i.demand_n, i.quoted_nano
         FROM ton_invoices i JOIN bookings b ON b.id = i.booking_id
         WHERE b.guest_email = 'second@x.io'",
    )
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(
        row,
        (
            (20 * T) as i64,
            "ladder".into(),
            Some(1),
            Some((20 * T) as i64)
        )
    );

    // No quote at all (an old page): asked to confirm, then a higher quote
    // is charged the price now.
    let (status, _, page) = post_book(
        &e,
        "/u/testuser/test-meeting/book",
        "third@x.io",
        "12:00",
        "198.51.102.1",
        "",
    )
    .await;
    assert_eq!(status, 200);
    assert!(page.contains("Please confirm the price."), "{page}");
    assert!(page.contains(r#"action="/u/testuser/test-meeting/book""#));
    let (status, _, _) = post_book(
        &e,
        "/u/testuser/test-meeting/book",
        "third@x.io",
        "12:00",
        "198.51.102.1",
        "&quoted_nano=40000000000",
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::SEE_OTHER);
    let amount: i64 = sqlx::query_scalar(
        "SELECT i.amount_nano FROM ton_invoices i JOIN bookings b ON b.id = i.booking_id
         WHERE b.guest_email = 'third@x.io'",
    )
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(amount as u128, 20 * T);

    // The same guest again at the free step (after the owner declined):
    // past the per-guest limit, so not free for them, but never shut out:
    // the quote page offers the first price, and confirming it opens an
    // invoice at that price although the step is still free.
    sqlx::query("UPDATE bookings SET status = 'declined' WHERE id = ?")
        .bind(&first)
        .execute(&e.pool)
        .await
        .unwrap();
    assert_eq!(demand(&e, None).await, 0, "the holds do not count");
    let (status, _, page) = post_book(
        &e,
        "/u/testuser/test-meeting/book",
        "first@x.io",
        "13:00",
        "198.51.103.1",
        "&quoted_nano=0",
    )
    .await;
    assert_eq!(status, 200);
    assert!(page.contains(QUOTE_TITLE), "{page}");
    assert!(
        page.contains(&crate::i18n::translate(
            "en",
            "ton-quote-why-free-used",
            None
        )),
        "{page}"
    );
    assert!(page.contains("Book for 20 TON"), "{page}");
    let (status, location, _) = post_book(
        &e,
        "/u/testuser/test-meeting/book",
        "first@x.io",
        "13:00",
        "198.51.103.1",
        "&quoted_nano=20000000000",
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::SEE_OTHER);
    assert!(location.unwrap().starts_with("/booking/pay/"));
    let row: (i64, Option<i64>) = sqlx::query_as(
        "SELECT i.amount_nano, i.demand_n FROM ton_invoices i
         JOIN bookings b ON b.id = i.booking_id
         WHERE b.guest_email = 'first@x.io' AND b.status = 'pending'",
    )
    .fetch_one(&e.pool)
    .await
    .unwrap();
    assert_eq!(row, ((20 * T) as i64, Some(0)));
}

/// The two handlers at once, on a WAL database: one guest gets the free
/// step, the other the "price changed" page; one booking.
#[tokio::test]
async fn two_free_requests_at_once_through_both_handlers() {
    let dir = TestDataDir::new();
    let e = ladder_env_in(&[], file_pool(&dir).await).await;
    let (a, b) = tokio::join!(
        post_book(
            &e,
            "/u/testuser/test-meeting/book",
            "a@x.io",
            "10:00",
            "198.51.100.1",
            "&quoted_nano=0",
        ),
        post_book(
            &e,
            "/test-meeting/book",
            "b@y.io",
            "11:00",
            "203.0.113.9",
            "&quoted_nano=0",
        ),
    );
    assert_eq!(
        (a.0, b.0),
        (axum::http::StatusCode::OK, axum::http::StatusCode::OK)
    );
    let quoted = [&a.2, &b.2]
        .iter()
        .filter(|page| page.contains(QUOTE_TITLE))
        .count();
    assert_eq!(quoted, 1, "one guest is asked again");
    assert_eq!(count(&e, "SELECT COUNT(*) FROM bookings").await, 1);
    assert_eq!(count(&e, "SELECT COUNT(*) FROM ton_free_bookings").await, 1);
}

/// The event type form: the pricing mode select, the ladder shown
/// read-only, the mode stored, a junk mode refused.
#[tokio::test]
async fn the_event_type_form_sets_the_ladder_mode() {
    let e = env_priced(&[LADDER], Some(100 * T)).await;
    let session = e.session().await;
    let edit = "/dashboard/event-types/test-meeting/edit";
    let post = |extra: &str| {
        format!("_csrf={CSRF}&title=Test+Meeting&slug=test-meeting&duration_min=30&location_value=https%3A%2F%2Fmeet.example.com&avail_days=1,2,3,4,5&avail_start=09:00&avail_end=17:00&ton_price_enabled=on&ton_price_amount=&ton_price_currency=TON{extra}")
    };
    let mode = || async {
        sqlx::query_as::<_, (String, Option<i64>)>("SELECT mode, price_nano FROM ton_prices")
            .fetch_one(&e.pool)
            .await
            .unwrap()
    };
    let page = body_string(e.app().oneshot(get_authed(edit, &session)).await.unwrap()).await;
    assert!(page.contains(r#"name="ton_price_mode""#), "{page}");
    assert!(page.contains("free → 20 → 40 TON"), "{page}");
    // The word "free" is the page's language, as on the TON settings page.
    sqlx::query("UPDATE users SET language = 'ru' WHERE username = 'testuser'")
        .execute(&e.pool)
        .await
        .unwrap();
    let ru = body_string(e.app().oneshot(get_authed(edit, &session)).await.unwrap()).await;
    assert!(ru.contains("бесплатно → 20 → 40 TON"), "{ru}");
    assert!(!ru.contains("free → 20"), "{ru}");
    sqlx::query("UPDATE users SET language = 'en' WHERE username = 'testuser'")
        .execute(&e.pool)
        .await
        .unwrap();
    e.app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_mode=ladder"),
        ))
        .await
        .unwrap();
    assert_eq!(mode().await, ("ladder".into(), None));
    let page = body_string(e.app().oneshot(get_authed(edit, &session)).await.unwrap()).await;
    assert!(
        page.contains(r#"<option value="ladder" selected>"#),
        "{page}"
    );
    let page = body_string(
        e.app()
            .oneshot(post_form(
                edit,
                &session,
                CSRF,
                &post("&ton_price_mode=auction"),
            ))
            .await
            .unwrap(),
    )
    .await;
    assert!(page.contains(&crate::i18n::translate("en", "ton-form-error-mode", None)));
    assert_eq!(mode().await, ("ladder".into(), None));
    e.app()
        .oneshot(post_form(
            edit,
            &session,
            CSRF,
            &post("&ton_price_mode=fixed"),
        ))
        .await
        .unwrap();
    assert_eq!(mode().await, ("fixed".into(), Some((100 * T) as i64)));
}
