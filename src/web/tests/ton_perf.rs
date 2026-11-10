//! ilia fork, the Optimize stage: measurements on a seeded database. A
//! child of the TON tests, for their chain, rates and booking helpers.
//!
//! Every test here prints numbers and is `#[ignore]`, run by hand in the
//! builder container (one at a time, so the numbers are not shared with
//! the rest of the suite):
//!
//! ```text
//! cargo test --locked perf_ -- --ignored --nocapture --test-threads 1
//! ```
//!
//! Nothing leaves the process: the chain and the rates are the TON tests'
//! fakes, and the calendar source is marked synced far in the future, so no
//! page syncs it. The data, as the plan has it: one owner, one source with 3
//! calendars, 5000 events (18 months back, 6 ahead), 300 weekly masters with
//! 10 EXDATEs and 5 moved instances each, 2000 bookings, a TON-priced and a
//! USD-priced type, 200 invoices and 400 payments.

use super::*;
use crate::ton::amount::NANO_PER_TON;
use crate::web::ton_pay::stamp;
use std::time::Instant;

const EVENTS: i64 = 5000;
const MASTERS: i64 = 300;
const EXDATES: i64 = 10;
const OVERRIDES: i64 = 5;
const BOOKINGS: i64 = 2000;
const INVOICES: i64 = 200;
const ZONE: &str = "Europe/Berlin";

/// The seeded environment: the TON test `Env` plus what the pages need.
pub(super) struct Seeded {
    pub(super) e: Env,
    /// An open invoice's payment link (a booking 3 days out).
    pub(super) pay_token: String,
    pub(super) booking_id: String,
    pub(super) session: String,
    /// Bytes of the database before and after the busy-only scrub, each
    /// after VACUUM.
    pub(super) bytes_with_content: i64,
    pub(super) bytes_busy_only: i64,
}

fn ical(t: NaiveDateTime) -> String {
    t.format("%Y%m%dT%H%M%S").to_string()
}

/// An event as upstream stores it: title, description, location, people.
fn vevent(uid: &str, start: NaiveDateTime, end: NaiveDateTime, extra: &str, n: i64) -> String {
    format!(
        "BEGIN:VEVENT\r\nUID:{uid}\r\nDTSTAMP:20261101T090000Z\r\n\
         DTSTART;TZID={ZONE}:{s}\r\nDTEND;TZID={ZONE}:{e}\r\n{extra}\
         SUMMARY:Project sync {n} with the regional team\r\n\
         DESCRIPTION:Agenda: numbers for the quarter\\, hiring\\, the office move. \
         Dial-in details below. Please read the attached document before the call \
         and bring your questions.\r\n\
         LOCATION:Room {n}\\, 4th floor\\, Main building\r\n\
         ORGANIZER;CN=Boss:mailto:boss@corp.example\r\n\
         ATTENDEE;CN=Alice;PARTSTAT=ACCEPTED:mailto:alice@corp.example\r\n\
         ATTENDEE;CN=Bob;PARTSTAT=TENTATIVE:mailto:bob@corp.example\r\n\
         BEGIN:VALARM\r\nTRIGGER:-PT15M\r\nACTION:DISPLAY\r\nDESCRIPTION:Reminder\r\nEND:VALARM\r\n\
         END:VEVENT\r\n",
        s = ical(start),
        e = ical(end),
    )
}

fn calendar(body: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//perf//EN\r\n\
         BEGIN:VTIMEZONE\r\nTZID:{ZONE}\r\nBEGIN:STANDARD\r\nDTSTART:19701025T030000\r\n\
         TZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nRRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU\r\n\
         END:STANDARD\r\nBEGIN:DAYLIGHT\r\nDTSTART:19700329T020000\r\nTZOFFSETFROM:+0100\r\n\
         TZOFFSETTO:+0200\r\nRRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU\r\nEND:DAYLIGHT\r\n\
         END:VTIMEZONE\r\n{body}END:VCALENDAR\r\n"
    )
}

/// The database's size after VACUUM, in bytes.
async fn db_bytes(pool: &SqlitePool) -> i64 {
    sqlx::query("VACUUM").execute(pool).await.unwrap();
    let pages: i64 = sqlx::query_scalar("PRAGMA page_count")
        .fetch_one(pool)
        .await
        .unwrap();
    let size: i64 = sqlx::query_scalar("PRAGMA page_size")
        .fetch_one(pool)
        .await
        .unwrap();
    pages * size
}

/// Seed the plan's data set into a TON test environment (100 TON price),
/// in memory or in `pool`.
pub(super) async fn seed(pool: Option<SqlitePool>) -> Seeded {
    let pool = match pool {
        Some(p) => p,
        None => setup_test_db().await,
    };
    let e = env_priced_in(&[], Some(100 * NANO_PER_TON), pool).await;
    let pool = e.pool.clone();
    let user_id: String = sqlx::query_scalar("SELECT id FROM users WHERE username = 'testuser'")
        .fetch_one(&pool)
        .await
        .unwrap();
    // A source synced "in the future": no page and no loop syncs it.
    let source_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO caldav_sources (id, account_id, name, url, username, last_synced, last_full_sync)
         VALUES (?, ?, 'Work', 'https://caldav.invalid/dav/', 'u', '2999-01-01 00:00:00', '2999-01-01 00:00:00')",
    )
    .bind(&source_id)
    .bind(&e.account_id)
    .execute(&pool)
    .await
    .unwrap();
    let mut calendars = Vec::new();
    for n in 0..3 {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO calendars (id, source_id, href, display_name, is_busy) VALUES (?, ?, ?, ?, 1)",
        )
        .bind(&id)
        .bind(&source_id)
        .bind(format!("/cal/{n}/"))
        .bind(format!("Calendar {n}"))
        .execute(&pool)
        .await
        .unwrap();
        calendars.push(id);
    }

    let now = Utc::now().naive_utc();
    let base = now.date().and_hms_opt(0, 0, 0).unwrap() - Duration::days(548);
    let span_min = (548 + 183) * 24 * 60;
    let mut tx = pool.begin().await.unwrap();
    // Single events, spread evenly, 9:00-18:00 local, 30-90 minutes.
    for i in 0..EVENTS {
        let day = base + Duration::minutes(span_min * i / EVENTS);
        let start = day.date().and_hms_opt(9 + (i % 9) as u32, 0, 0).unwrap();
        let end = start + Duration::minutes(30 * (1 + i % 3));
        let uid = format!("single-{i}@perf");
        let raw = calendar(&vevent(&uid, start, end, "", i));
        insert_event(
            &mut tx,
            &calendars[(i % 3) as usize],
            &uid,
            None,
            start,
            end,
            None,
            &raw,
            i,
        )
        .await;
    }
    // Weekly masters with EXDATEs and moved instances.
    for m in 0..MASTERS {
        let start = (base + Duration::days(m % 400))
            .date()
            .and_hms_opt(8 + (m % 10) as u32, 30, 0)
            .unwrap();
        let end = start + Duration::minutes(45);
        let uid = format!("weekly-{m}@perf");
        let rrule = if m % 3 == 0 {
            "FREQ=WEEKLY;INTERVAL=2".to_string()
        } else {
            "FREQ=WEEKLY".to_string()
        };
        let mut extra = format!("RRULE:{rrule}\r\n");
        for x in 0..EXDATES {
            let ex = start + Duration::weeks(3 + 4 * x);
            extra.push_str(&format!("EXDATE;TZID={ZONE}:{}\r\n", ical(ex)));
        }
        let mut body = vevent(&uid, start, end, &extra, m);
        let mut moved = Vec::new();
        for o in 0..OVERRIDES {
            let original = start + Duration::weeks(5 + 4 * o);
            let new_start = original + Duration::hours(2);
            let rid = format!("RECURRENCE-ID;TZID={ZONE}:{}\r\n", ical(original));
            body.push_str(&vevent(
                &uid,
                new_start,
                new_start + Duration::minutes(45),
                &rid,
                m,
            ));
            moved.push((ical(original), new_start));
        }
        let raw = calendar(&body);
        insert_event(
            &mut tx,
            &calendars[(m % 3) as usize],
            &uid,
            None,
            start,
            end,
            Some(&rrule),
            &raw,
            m,
        )
        .await;
        for (rid, new_start) in moved {
            let one = calendar(&vevent(
                &uid,
                new_start,
                new_start + Duration::minutes(45),
                &format!("RECURRENCE-ID;TZID={ZONE}:{rid}\r\n"),
                m,
            ));
            insert_event(
                &mut tx,
                &calendars[(m % 3) as usize],
                &uid,
                Some(&rid),
                new_start,
                new_start + Duration::minutes(45),
                None,
                &one,
                m,
            )
            .await;
        }
    }
    // A free type and a USD-priced one beside the TON-priced test type.
    let free_et = uuid::Uuid::new_v4().to_string();
    let usd_et = uuid::Uuid::new_v4().to_string();
    for (id, slug) in [(&free_et, "intro"), (&usd_et, "usd-call")] {
        sqlx::query(
            "INSERT INTO event_types (id, account_id, slug, title, duration_min, buffer_before, buffer_after, min_notice_min, enabled)
             VALUES (?, ?, ?, ?, 30, 0, 0, 0, 1)",
        )
        .bind(id)
        .bind(&e.account_id)
        .bind(slug)
        .bind(slug)
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    // Bookings every 6 hours from 15 months back: the first INVOICES on the
    // TON type (each with an invoice, mostly paid), the rest free.
    let first = Utc::now() - Duration::days(450);
    for i in 0..BOOKINGS {
        let start = first + Duration::hours(6 * i) + Duration::minutes(7);
        let past = start < Utc::now();
        let (et, status) = match (i < INVOICES, past, i % 10) {
            (true, _, 0..=6) => (&e.et_id, "confirmed"),
            (true, _, _) => (&e.et_id, "declined"),
            (false, _, 9) => (&free_et, "cancelled"),
            (false, _, _) => (&free_et, "confirmed"),
        };
        let id = format!("perf-booking-{i}");
        sqlx::query(
            "INSERT INTO bookings (time_version, id, event_type_id, uid, guest_name, guest_email, guest_timezone, start_at, end_at, status, cancel_token, reschedule_token, confirm_token)
             VALUES (1, ?, ?, ?, 'Guest', ?, 'UTC', ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(et)
        .bind(format!("uid-{id}"))
        .bind(format!("guest{i}@example.com"))
        .bind(utc_stamp(start))
        .bind(utc_stamp(start + Duration::minutes(30)))
        .bind(status)
        .bind(format!("cancel-{id}"))
        .bind(format!("reschedule-{id}"))
        .bind(format!("confirm-{id}"))
        .execute(&mut *tx)
        .await
        .unwrap();
        if i < INVOICES {
            let paid = status == "confirmed";
            let created = start - Duration::days(2);
            let inv = format!("perf-invoice-{i}");
            sqlx::query(
                "INSERT INTO ton_invoices (id, booking_id, event_type_id, pay_token, reference, network, address_raw,
                     event_title, start_at_utc, guest_email_norm, price_usd_cents, rate_usd_micro, rate_sources, rate_at,
                     amount_nano, paid_nano, status, created_at, expires_at, settle_by, hard_deadline, paid_at,
                     confirmed_at, closed_at, paid_notice_sent_at, currency)
                 VALUES (?, ?, ?, ?, ?, 'mainnet', ?, 'Test Meeting', ?, ?, 0, 1, '', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'TON')",
            )
            .bind(&inv)
            .bind(&id)
            .bind(&e.et_id)
            .bind(format!("perf-pay-token-{i:06}"))
            .bind(format!("BPERF{i:06}"))
            .bind(OUR)
            .bind(stamp(start))
            .bind(format!("guest{i}@example.com"))
            .bind(stamp(created))
            .bind((100 * NANO_PER_TON) as i64)
            .bind(if paid { (100 * NANO_PER_TON) as i64 } else { 0 })
            .bind(if paid { "confirmed" } else { "expired" })
            .bind(stamp(created))
            .bind(stamp(created + Duration::minutes(30)))
            .bind(stamp(created + Duration::minutes(45)))
            .bind(stamp(created + Duration::hours(6)))
            .bind(paid.then(|| stamp(created + Duration::minutes(10))))
            .bind(paid.then(|| stamp(created + Duration::minutes(11))))
            .bind((!paid).then(|| stamp(created + Duration::hours(1))))
            .bind(paid.then(|| stamp(created + Duration::minutes(11))))
            .execute(&mut *tx)
            .await
            .unwrap();
            // Two transfers each: one credited, one too small to matter.
            for p in 0..2i64 {
                let n = i * 2 + p;
                sqlx::query(
                    "INSERT INTO ton_payments (tx_hex, network, address_raw, lt, utime, mc_block_seqno, source_raw,
                         value_nano, comment, invoice_id, outcome, reason, seen_at)
                     VALUES (?, 'mainnet', ?, ?, ?, 1, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(format!("{n:064x}"))
                .bind(OUR)
                .bind(10_000 + n)
                .bind((created + Duration::minutes(10)).timestamp())
                .bind(OTHER)
                .bind(if p == 0 { (100 * NANO_PER_TON) as i64 } else { 1000 })
                .bind(format!("BPERF{i:06}"))
                .bind((p == 0).then(|| inv.clone()))
                .bind(if p == 0 { "credited" } else { "ignored" })
                .bind((p == 1).then_some("below_threshold"))
                .bind(stamp(created + Duration::minutes(10)))
                .execute(&mut *tx)
                .await
                .unwrap();
            }
        }
    }
    tx.commit().await.unwrap();
    set_price(&pool, &usd_et, 4000).await;
    // The chain's cursor past the seeded transfers.
    sqlx::query("UPDATE ton_watch SET cursor_lt = 20000")
        .execute(&pool)
        .await
        .unwrap();

    let bytes_with_content = db_bytes(&pool).await;
    {
        let _mode = crate::busy_only::test_mode(true);
        crate::busy_only::scrub_stored(&pool).await.unwrap();
    }
    let bytes_busy_only = db_bytes(&pool).await;

    // One open invoice for the payment page, and a thread on it, at a time
    // the host is free (else the watcher releases it at once): 3 days out,
    // at night in Berlin, an hour clear of every seeded booking.
    let day = (Utc::now() + Duration::days(3)).date_naive();
    let mut free = None;
    for h in [2u32, 3, 4, 1, 5, 0] {
        let start = day.and_hms_opt(h, 0, 0).unwrap().and_utc();
        let near: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM bookings WHERE start_at > ? AND start_at < ?")
                .bind(utc_stamp(start - Duration::hours(1)))
                .bind(utc_stamp(start + Duration::hours(1)))
                .fetch_one(&pool)
                .await
                .unwrap();
        if near == 0 {
            free = Some(start);
            break;
        }
    }
    let (booking_id, opened) = e
        .booking_at(
            "payer@example.com",
            free.expect("a free hour"),
            "203.0.113.5",
        )
        .await
        .unwrap();
    let session = e.session().await;
    sqlx::query(
        "INSERT INTO booking_messages (id, booking_id, direction, author_user_id, via, body, created_at)
         VALUES ('perf-m1', ?, 'host', ?, 'dashboard', 'Hello', datetime('now'))",
    )
    .bind(&booking_id)
    .bind(&user_id)
    .execute(&pool)
    .await
    .unwrap();
    Seeded {
        e,
        pay_token: opened.pay_token,
        booking_id,
        session,
        bytes_with_content,
        bytes_busy_only,
    }
}

#[allow(clippy::too_many_arguments)]
async fn insert_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    calendar_id: &str,
    uid: &str,
    recurrence_id: Option<&str>,
    start: NaiveDateTime,
    end: NaiveDateTime,
    rrule: Option<&str>,
    raw: &str,
    n: i64,
) {
    sqlx::query(
        "INSERT INTO events (id, calendar_id, uid, summary, description, location, start_at, end_at, all_day,
             timezone, rrule, raw_ical, status, transp, recurrence_id, synced_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?, 'CONFIRMED', 'OPAQUE', ?, datetime('now'))",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(calendar_id)
    .bind(uid)
    .bind(format!("Project sync {n} with the regional team"))
    .bind("Agenda: numbers for the quarter, hiring, the office move.")
    .bind(format!("Room {n}, 4th floor, Main building"))
    .bind(ical(start))
    .bind(ical(end))
    .bind(ZONE)
    .bind(rrule)
    .bind(raw)
    .bind(recurrence_id)
    .execute(&mut **tx)
    .await
    .unwrap();
}

// --- statements on the hot paths, captured from sqlx's own log --------------

static CAPTURE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static STATEMENTS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

struct SqlCapture;

struct StatementVisitor(Option<String>, Option<String>);

impl tracing::field::Visit for StatementVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "db.statement" if !value.trim().is_empty() => self.0 = Some(value.to_string()),
            "summary" => self.1 = Some(value.to_string()),
            _ => {}
        }
    }
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "db.statement" || field.name() == "summary" {
            let s = format!("{value:?}");
            let s = s
                .trim_matches('"')
                .replace("\\n", "\n")
                .replace("\\\"", "\"");
            self.record_str(field, &s);
        }
    }
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for SqlCapture {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        if !CAPTURE.load(std::sync::atomic::Ordering::SeqCst)
            || event.metadata().target() != "sqlx::query"
        {
            return;
        }
        let mut v = StatementVisitor(None, None);
        event.record(&mut v);
        if let Some(sql) = v.0.or(v.1) {
            STATEMENTS.lock().unwrap().push(sql);
        }
    }
}

/// The SQLite worker threads log statements, so the capture is the global
/// subscriber (these tests run alone, `--test-threads 1`).
fn install_capture() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        use tracing_subscriber::layer::SubscriberExt;
        let _ = tracing::subscriber::set_global_default(
            tracing_subscriber::registry().with(SqlCapture),
        );
    });
}

fn one_line(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Tables that grow with use: a full SCAN of one is worth a look.
const GROWING: [&str; 8] = [
    "events",
    "bookings",
    "ton_invoices",
    "ton_payments",
    "booking_messages",
    "telegram_message_map",
    "sessions",
    "resource_events",
];

fn scans_growing(plan: &[String], sql: &str) -> Vec<String> {
    // Aliases: "FROM events e" → SCAN e.
    let words: Vec<&str> = sql.split_whitespace().collect();
    let mut aliases: Vec<(String, String)> = Vec::new();
    for w in words.windows(2) {
        let table = w[0].trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
        if GROWING.contains(&table) {
            let alias = w[1].trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
            aliases.push((alias.to_string(), table.to_string()));
            aliases.push((table.to_string(), table.to_string()));
        }
    }
    plan.iter()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("SCAN ")?;
            let name = rest.split_whitespace().next()?;
            let table = aliases
                .iter()
                .find(|(a, _)| a == name)
                .map(|(_, t)| t.clone())?;
            Some(format!("{table}: {line}"))
        })
        .collect()
}

async fn plan_of(pool: &SqlitePool, sql: &str) -> Result<Vec<String>, String> {
    let rows: Vec<(i64, i64, i64, String)> = sqlx::query_as(&format!("EXPLAIN QUERY PLAN {sql}"))
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|(_, _, _, d)| d).collect())
}

/// `EXPLAIN QUERY PLAN` of every statement the hot paths run: the slot
/// page, the booking form, the payment page and its status, the bookings
/// dashboard and a message thread, a watcher step with an open invoice,
/// the pending sweep and the background sync pick. Prints each plan and
/// a summary of the full scans of growing tables.
#[tokio::test]
#[ignore]
async fn perf_explain_hot_paths() {
    install_capture();
    let s = seed(None).await;
    let e = &s.e;
    let app = e.app();
    let paths = [
        ("slot page", "/u/testuser/test-meeting".to_string()),
        (
            "booking form",
            format!(
                "/u/testuser/test-meeting/book?date={}&time=10:00",
                far_future_monday()
            ),
        ),
        ("pay page", format!("/booking/pay/{}", s.pay_token)),
        ("pay status", format!("/booking/pay/{}/status", s.pay_token)),
    ];
    let mut by_path: Vec<(String, Vec<String>)> = Vec::new();
    let mut run = |name: &str| {
        let got = std::mem::take(&mut *STATEMENTS.lock().unwrap());
        by_path.push((name.to_string(), got));
    };
    CAPTURE.store(true, std::sync::atomic::Ordering::SeqCst);
    for (name, uri) in &paths {
        let resp = app.clone().oneshot(get(uri)).await.unwrap();
        assert!(resp.status().is_success(), "{name}: {}", resp.status());
        let _ = body_string(resp).await;
        run(name);
    }
    for (name, uri) in [
        ("dashboard bookings", "/dashboard/bookings".to_string()),
        (
            "message thread",
            format!("/dashboard/bookings/{}/messages", s.booking_id),
        ),
    ] {
        let resp = app
            .clone()
            .oneshot(get_authed(&uri, &s.session))
            .await
            .unwrap();
        assert!(resp.status().is_success(), "{name}: {}", resp.status());
        let _ = body_string(resp).await;
        run(name);
    }
    e.step(e.now).await;
    run("watcher step (open invoice)");
    sweep_pending_requests(&e.pool, &[0u8; 32], Utc::now()).await;
    run("pending sweep");
    crate::commands::sync::next_background_source(&e.pool, 300).await;
    run("background sync pick");
    CAPTURE.store(false, std::sync::atomic::Ordering::SeqCst);

    let mut seen = std::collections::HashSet::new();
    let mut scans: Vec<String> = Vec::new();
    for (name, statements) in &by_path {
        println!("\n=== {name}: {} statements", statements.len());
        for sql in statements {
            let line = one_line(sql);
            if !seen.insert(line.clone()) {
                continue;
            }
            let short: String = line.chars().take(150).collect();
            match plan_of(&e.pool, sql).await {
                Ok(plan) => {
                    let hits = scans_growing(&plan, &line);
                    println!("--- {short}");
                    for p in &plan {
                        println!("      {p}");
                    }
                    for h in hits {
                        scans.push(format!("[{name}] {h} <- {short}"));
                    }
                }
                Err(err) => println!("--- (no plan: {err}) {short}"),
            }
        }
    }
    println!("\n=== full scans of growing tables: {}", scans.len());
    for s in &scans {
        println!("SCAN {s}");
    }
}

fn percentile(sorted: &[u128], p: f64) -> u128 {
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// p50, p95 and max of 200 requests to each page (microseconds), plus the
/// database size with content and busy-only (after VACUUM).
#[tokio::test]
#[ignore]
async fn perf_latency_and_size() {
    let s = seed(None).await;
    let e = &s.e;
    println!(
        "db bytes: with content {} / busy-only {}",
        s.bytes_with_content, s.bytes_busy_only
    );
    // The busy times of the slot page's window (two months), alone.
    let user_id: String = sqlx::query_scalar("SELECT id FROM users WHERE username = 'testuser'")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    let from = Utc::now().naive_utc();
    let mut times = Vec::new();
    let mut found = 0;
    for i in 0..120 {
        let t = Instant::now();
        let busy = fetch_busy_times_for_user(
            &e.pool,
            &user_id,
            from,
            from + Duration::days(60),
            chrono_tz::UTC,
            Some(&e.et_id),
        )
        .await;
        if i >= 20 {
            times.push(t.elapsed().as_micros());
        }
        found = busy.len();
    }
    times.sort_unstable();
    println!(
        "busy times (60 days, {found} intervals): n={} p50={}us p95={}us max={}us",
        times.len(),
        percentile(&times, 0.5),
        percentile(&times, 0.95),
        times[times.len() - 1]
    );
    // A watcher step with the open invoice (no transfer).
    let mut times = Vec::new();
    for i in 0..60 {
        let t = Instant::now();
        e.step(e.now).await;
        if i >= 10 {
            times.push(t.elapsed().as_micros());
        }
    }
    times.sort_unstable();
    println!(
        "watcher step: n={} p50={}us p95={}us max={}us",
        times.len(),
        percentile(&times, 0.5),
        percentile(&times, 0.95),
        times[times.len() - 1]
    );
    let open: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ton_invoices WHERE status = 'open'")
        .fetch_one(&e.pool)
        .await
        .unwrap();
    assert_eq!(open, 1, "the open invoice is still open");
    let app = e.app();
    let pages = [
        (
            "slot page (month)",
            "/u/testuser/test-meeting".to_string(),
            false,
        ),
        (
            "booking form",
            format!(
                "/u/testuser/test-meeting/book?date={}&time=10:00",
                far_future_monday()
            ),
            false,
        ),
        ("pay page", format!("/booking/pay/{}", s.pay_token), false),
        (
            "pay status",
            format!("/booking/pay/{}/status", s.pay_token),
            false,
        ),
        (
            "dashboard bookings",
            "/dashboard/bookings".to_string(),
            true,
        ),
    ];
    for (name, uri, authed) in &pages {
        let mut times = Vec::new();
        for i in 0..220u32 {
            let mut req = if *authed {
                get_authed(uri, &s.session)
            } else {
                get(uri)
            };
            // A client address each: the payment page allows 120 a minute.
            req.headers_mut().insert(
                "x-forwarded-for",
                format!("198.51.{}.{}", i / 200, i % 200 + 1)
                    .parse()
                    .unwrap(),
            );
            let t = Instant::now();
            let resp = app.clone().oneshot(req).await.unwrap();
            let status = resp.status();
            let _ = body_string(resp).await;
            let us = t.elapsed().as_micros();
            assert!(status.is_success(), "{name}: {status}");
            // The first 20 warm the caches.
            if i >= 20 {
                times.push(us);
            }
        }
        times.sort_unstable();
        println!(
            "{name}: n={} p50={}us p95={}us max={}us",
            times.len(),
            percentile(&times, 0.5),
            percentile(&times, 0.95),
            times[times.len() - 1]
        );
    }
}

/// Write the seeded database (busy-only) for `calrs serve`, as three files
/// under `CALRS_PERF_DIR` (default `/build/target/flow2-perf`), with the
/// calendar source switched off (nothing to sync, no network):
/// `c/calrs.db` with the prices and one open invoice (its pay token in
/// `c/token`), `b/calrs.db` with the prices and no open invoice, and
/// `a/calrs.db` with no price at all. (An in-memory database cannot be
/// written out with VACUUM INTO, so this one is seeded in a file.)
#[tokio::test]
#[ignore]
async fn perf_seed_db_file() {
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    let dir = std::path::PathBuf::from(
        std::env::var("CALRS_PERF_DIR").unwrap_or_else(|_| "/build/target/flow2-perf".to_string()),
    );
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let work = dir.join("work.db");
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&work)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Delete)
                .foreign_keys(true),
        )
        .await
        .unwrap();
    crate::db::migrate(&pool).await.unwrap();
    let s = seed(Some(pool.clone())).await;
    sqlx::query("UPDATE caldav_sources SET enabled = 0")
        .execute(&pool)
        .await
        .unwrap();
    let copy = |name: &str| {
        let sub = dir.join(name);
        std::fs::create_dir_all(&sub).unwrap();
        let out = sub.join("calrs.db");
        std::fs::copy(&work, &out).unwrap();
        println!(
            "{} ({} bytes)",
            out.display(),
            std::fs::metadata(&out).unwrap().len()
        );
        sub
    };
    let c = copy("c");
    std::fs::write(c.join("token"), &s.pay_token).unwrap();
    sqlx::query("DELETE FROM ton_invoices WHERE booking_id = ?")
        .bind(&s.booking_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM bookings WHERE id = ?")
        .bind(&s.booking_id)
        .execute(&pool)
        .await
        .unwrap();
    copy("b");
    sqlx::query("DELETE FROM ton_prices")
        .execute(&pool)
        .await
        .unwrap();
    copy("a");
}

// --- watcher steps and external requests per hour ------------------------------

/// What one simulated hour of the watcher asked for.
#[derive(Debug, Default)]
pub(super) struct Hour {
    pub(super) steps: usize,
    pub(super) masterchain: usize,
    pub(super) pages: usize,
    pub(super) account: usize,
    pub(super) rates: usize,
}

/// Run the watcher's loop for `minutes` of simulated time: a step, then
/// the delay the watcher would sleep (no wakes), on the clock passed in.
pub(super) async fn simulate(e: &Env, start: chrono::DateTime<Utc>, minutes: i64) -> Hour {
    let before = e
        .chain
        .edit(|c| (c.mc_calls, c.pages_served, c.status_calls));
    let rates_before = e.rates.calls();
    let end = start + Duration::minutes(minutes);
    let mut t = start;
    let mut h = Hour::default();
    while t < end {
        let report = e.step(t).await;
        h.steps += 1;
        let delay = watcher::next_delay(&e.pool, &e.rt, &report, t).await;
        // The rate cache ages in real time: a simulated wait as long as it
        // lives empties it, as it would live.
        if delay >= crate::web::ton_pay::RATE_CACHE {
            e.rt.forget_rate();
        }
        t += Duration::from_std(delay).unwrap();
    }
    let after = e
        .chain
        .edit(|c| (c.mc_calls, c.pages_served, c.status_calls));
    h.masterchain = after.0 - before.0;
    h.pages = after.1 - before.1;
    h.account = after.2 - before.2;
    h.rates = e.rates.calls() - rates_before;
    h
}

/// Steps and requests per simulated hour: a TON price and no invoice, the
/// same with one open invoice, a USD price, and no price at all.
#[tokio::test]
#[ignore]
async fn perf_requests_per_hour() {
    let e = env_priced(&[], Some(100 * NANO_PER_TON)).await;
    let idle = simulate(&e, e.now + Duration::minutes(1), 60).await;
    println!("TON price, no invoice: {idle:?}");
    let e = env_priced(&[], Some(100 * NANO_PER_TON)).await;
    e.booking("guest@example.com").await;
    let open = simulate(&e, e.now + Duration::seconds(10), 60).await;
    println!("TON price, one open invoice: {open:?}");
    let e = env().await;
    let usd = simulate(&e, e.now + Duration::minutes(1), 60).await;
    println!("USD price, no invoice: {usd:?}");
    let e = env_priced(&[], Some(100 * NANO_PER_TON)).await;
    sqlx::query("DELETE FROM ton_prices")
        .execute(&e.pool)
        .await
        .unwrap();
    let none = simulate(&e, e.now + Duration::minutes(1), 60).await;
    println!("no price: {none:?}");
}
