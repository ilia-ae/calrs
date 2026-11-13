//! ilia fork: busy-only calendar storage (`src/busy_only.rs`). The same
//! calendar synced with and without it gives the same busy times and the
//! same slots, and busy-only keeps none of the content.

use super::*;
use crate::busy_only::{test_mode, BLUEMIND_FIXTURE, MARKER};
use crate::caldav::RawEvent;

/// Strings of the fixtures that must never be stored in busy-only mode.
const CONTENT: [&str; 9] = [
    "Secret",
    "Quarterly",
    "Board room",
    "boss@corp",
    "alice@corp",
    "Reminder",
    "lunch",
    "Offsite",
    "Dinner",
];

/// The fixtures are written for Monday 2030-01-07; the slot grid only
/// reaches a year ahead, so they are moved to [`base`] (a Monday too).
const DAYS: [(&str, i64); 15] = [
    ("20300107", 0),
    ("20300108", 1),
    ("20300109", 2),
    ("20300110", 3),
    ("20300114", 7),
    ("20300115", 8),
    ("20300116", 9),
    ("20300120", 13),
    ("20300121", 14),
    ("20300122", 15),
    ("20300125", 18),
    ("20300128", 21),
    ("20300204", 28),
    ("20300210", 34),
    ("20300211", 35),
];

fn base() -> NaiveDate {
    let mut d = Utc::now().date_naive() + Duration::days(21);
    while d.weekday() != chrono::Weekday::Mon {
        d += Duration::days(1);
    }
    d
}

/// `base() + n` days at `h:00`.
fn at(n: i64, h: u32) -> NaiveDateTime {
    (base() + Duration::days(n)).and_hms_opt(h, 0, 0).unwrap()
}

fn shift(text: &str) -> String {
    let mut out = text
        .replace("20300212", &day(36))
        .replace("20300218", &day(42));
    for (old, n) in DAYS {
        out = out.replace(old, &day(n));
    }
    out
}

fn day(n: i64) -> String {
    (base() + Duration::days(n)).format("%Y%m%d").to_string()
}

fn resources() -> Vec<RawEvent> {
    let ics = |uid: &str, body: &str| RawEvent {
        href: format!("/cal/{uid}.ics"),
        ical_data: shift(&format!(
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:{uid}\r\n{body}END:VEVENT\r\nEND:VCALENDAR\r\n"
        )),
        etag: None,
    };
    vec![
        RawEvent {
            href: "/cal/weekly-1.ics".into(),
            ical_data: shift(BLUEMIND_FIXTURE),
            etag: None,
        },
        ics(
            "transparent-1",
            "DTSTART:20300108T100000Z\r\nDTEND:20300108T120000Z\r\nTRANSP:TRANSPARENT\r\nSUMMARY:Secret lunch\r\n",
        ),
        ics(
            "allday-1",
            "DTSTART;VALUE=DATE:20300109\r\nDTEND;VALUE=DATE:20300110\r\nSUMMARY:Offsite\r\nLOCATION:Board room 7\r\n",
        ),
        ics(
            "timed-1",
            "DTSTART;TZID=Europe/Paris:20300115T090000\r\nDTEND;TZID=Europe/Paris:20300115T100000\r\nSUMMARY:Dinner prep\r\nDESCRIPTION:Quarterly\r\nATTENDEE;CN=Alice:mailto:alice@corp.example\r\n",
        ),
        ics(
            "daily-1",
            "DTSTART;TZID=Asia/Tokyo:20300120T160000\r\nDTEND;TZID=Asia/Tokyo:20300120T170000\r\nRRULE:FREQ=DAILY;UNTIL=20300210T000000Z\r\nEXDATE:20300122T070000Z\r\nEXDATE;TZID=Asia/Tokyo:20300125T160000\r\nSUMMARY:Secret standup\r\n",
        ),
        ics(
            "cancelled-1",
            "DTSTART:20300116T100000Z\r\nDTEND:20300116T110000Z\r\nSTATUS:CANCELLED\r\nSUMMARY:Secret call\r\n",
        ),
    ]
}

/// Which write path a calendar is synced through.
#[derive(Clone, Copy, Debug)]
enum Path {
    /// CalDAV, full and delta (`upsert_raw_events`).
    Caldav,
    /// Google, Microsoft and the other providers (`upsert_provider_events`).
    Provider,
}

/// A database with the fixtures synced through the real upsert, in `on`
/// mode. Returns the pool, the user, the event type and the calendar.
async fn synced(on: bool) -> (SqlitePool, String, String, String) {
    synced_by(on, Path::Caldav).await
}

async fn synced_by(on: bool, path: Path) -> (SqlitePool, String, String, String) {
    let _mode = test_mode(on);
    let pool = setup_test_db().await;
    let (user_id, account_id, et_id) = seed_test_data(&pool).await;
    let source_id = uuid::Uuid::new_v4().to_string();
    let cal_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO caldav_sources (id, account_id, name, url, username) VALUES (?, ?, 'Src', 'https://x/dav', 'u')")
        .bind(&source_id).bind(&account_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO calendars (id, source_id, href, display_name, is_busy) VALUES (?, ?, '/cal/', 'Cal', 1)")
        .bind(&cal_id).bind(&source_id).execute(&pool).await.unwrap();
    let count = match path {
        Path::Caldav => {
            crate::commands::sync::upsert_raw_events(&pool, &cal_id, &resources()).await
        }
        Path::Provider => {
            let raw: Vec<crate::providers::RawEvent> = resources()
                .into_iter()
                .map(|r| crate::providers::RawEvent {
                    remote_id: r.href,
                    ical: r.ical_data,
                })
                .collect();
            crate::commands::sync::upsert_provider_events(&pool, &cal_id, &raw).await
        }
    };
    assert_eq!(count, 9, "{path:?}");
    (pool, user_id, et_id, cal_id)
}

/// The fixtures as one resource feed (a room's published ICS): its
/// VTIMEZONE and every VEVENT in one calendar.
fn feed() -> String {
    let mut body = String::from("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n");
    for r in resources() {
        body.push_str(
            &r.ical_data
                .replace("BEGIN:VCALENDAR\r\n", "")
                .replace("END:VCALENDAR\r\n", ""),
        );
    }
    body.push_str("END:VCALENDAR\r\n");
    body
}

/// A database with the fixtures stored as a resource's feed, in `on`
/// mode. Returns the pool and the resource.
async fn synced_feed(on: bool) -> (SqlitePool, String) {
    let _mode = test_mode(on);
    let pool = setup_test_db().await;
    let rid = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO resources (id, name, feed_url) VALUES (?, 'Room', 'https://x/room.ics')",
    )
    .bind(&rid)
    .execute(&pool)
    .await
    .unwrap();
    let count = crate::resources::store_feed(&pool, &rid, &feed()).await;
    assert_eq!(count, 9);
    (pool, rid)
}

fn dubai() -> Tz {
    "Asia/Dubai".parse().unwrap()
}

async fn busy(pool: &SqlitePool, user_id: &str) -> Vec<(NaiveDateTime, NaiveDateTime)> {
    let mut busy =
        fetch_busy_times_for_user(pool, user_id, at(-1, 0), at(59, 0), dubai(), None).await;
    busy.sort();
    busy
}

async fn slots(pool: &SqlitePool, et_id: &str, user_id: &str) -> Vec<(String, Vec<String>)> {
    let today = Utc::now().with_timezone(&dubai()).date_naive();
    let offset = (base() - today).num_days() as i32 - 1;
    let days = compute_slots(
        pool,
        et_id,
        30,
        0,
        0,
        0,
        offset,
        60,
        BookingWindow::default(),
        dubai(),
        dubai(),
        BusySource::Individual(busy(pool, user_id).await),
        None,
    )
    .await;
    days.into_iter()
        .map(|d| (d.date, d.slots.into_iter().map(|s| s.start).collect()))
        .collect()
}

/// Every column of every row of `table`, as text.
async fn dump(pool: &SqlitePool, table: &str) -> String {
    let columns: Vec<String> =
        sqlx::query_scalar(&format!("SELECT name FROM pragma_table_info('{table}')"))
            .fetch_all(pool)
            .await
            .unwrap();
    let expr = columns
        .iter()
        .map(|c| format!("COALESCE(CAST({c} AS TEXT), '')"))
        .collect::<Vec<_>>()
        .join(" || char(31) || ");
    let rows: Vec<String> = sqlx::query_scalar(&format!("SELECT {expr} FROM {table}"))
        .fetch_all(pool)
        .await
        .unwrap();
    rows.join("\n")
}

#[tokio::test]
async fn busy_only_keeps_availability_and_drops_the_content() {
    let (full, full_user, full_et, _) = synced(false).await;
    let (lean, lean_user, lean_et, _) = synced(true).await;

    let full_busy = busy(&full, &full_user).await;
    assert_eq!(busy(&lean, &lean_user).await, full_busy);
    // The fixture does block: the weekly 10:00 (Dubai) master on its first
    // Monday, not on its excluded ones, the moved instance at 15:00, the
    // all-day event, not the transparent or the cancelled one.
    assert!(full_busy.contains(&(at(0, 10), at(0, 11))));
    assert!(!full_busy.iter().any(|b| b.0 == at(7, 10)));
    assert!(full_busy.contains(&(at(28, 15), at(28, 16))));
    assert!(!full_busy.iter().any(|b| b.0 == at(28, 10)));
    assert!(!full_busy.iter().any(|b| b.0 == at(1, 14)));
    assert!(!full_busy.iter().any(|b| b.0 == at(9, 14)));
    assert!(full_busy.contains(&(at(2, 0), at(3, 0))));
    // Daily 16:00 Tokyo = 11:00 Dubai, two days excluded.
    assert!(full_busy.contains(&(at(14, 11), at(14, 12))));
    assert!(!full_busy.iter().any(|b| b.0 == at(15, 11)));
    assert!(!full_busy.iter().any(|b| b.0 == at(18, 11)));

    let full_slots = slots(&full, &full_et, &full_user).await;
    assert!(!full_slots.is_empty());
    assert_eq!(slots(&lean, &lean_et, &lean_user).await, full_slots);

    let stored = dump(&lean, "events").await;
    for word in CONTENT {
        assert!(!stored.contains(word), "{word} stored: {stored}");
    }
    assert!(!stored.contains("BEGIN:VCALENDAR"), "{stored}");
    let raws: Vec<(String, Option<String>)> =
        sqlx::query_as("SELECT uid, raw_ical FROM events WHERE raw_ical IS NOT NULL ORDER BY uid")
            .fetch_all(&lean)
            .await
            .unwrap();
    assert_eq!(
        raws.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        ["daily-1", "weekly-1"]
    );
    assert!(raws
        .iter()
        .all(|r| r.1.as_deref().unwrap().starts_with(MARKER)));

    // Off: upstream's cache, the title and the whole resource.
    let (summary, raw): (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT summary, raw_ical FROM events WHERE uid = 'weekly-1' AND recurrence_id IS NULL",
    )
    .fetch_one(&full)
    .await
    .unwrap();
    assert_eq!(summary.as_deref(), Some("Secret plans"));
    assert_eq!(raw, Some(shift(BLUEMIND_FIXTURE)));
}

/// Rows cached before the fork migration (or by a run with the switch off)
/// are scrubbed at startup, and the busy times do not change.
#[tokio::test]
async fn migration_scrubs_cached_content() {
    let (pool, user_id, _, _) = synced(false).await;
    let before = busy(&pool, &user_id).await;
    assert!(dump(&pool, "events").await.contains("Secret"));

    let _on = test_mode(true);
    sqlx::query("DELETE FROM _migrations WHERE name = 'ilia_003_busy_only'")
        .execute(&pool)
        .await
        .unwrap();
    // The SQL part alone: content gone, recurring rows not reduced yet.
    sqlx::raw_sql(include_str!("../../../migrations/ilia_003_busy_only.sql"))
        .execute(&pool)
        .await
        .unwrap();
    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM events WHERE raw_ical IS NOT NULL AND raw_ical NOT LIKE 'X-CALRS-BUSY:1%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(pending, 2);
    assert_eq!(busy(&pool, &user_id).await, before);

    // The startup step reduces them.
    crate::db::migrate(&pool).await.unwrap();
    let stored = dump(&pool, "events").await;
    for word in CONTENT {
        assert!(!stored.contains(word), "{word} stored: {stored}");
    }
    let nonrecurring_raw: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM events WHERE raw_ical IS NOT NULL AND (rrule IS NULL OR rrule = '')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(nonrecurring_raw, 0);
    let reduced: Vec<String> =
        sqlx::query_scalar("SELECT raw_ical FROM events WHERE raw_ical IS NOT NULL")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(reduced.len(), 2);
    assert!(reduced.iter().all(|r| r.starts_with(MARKER)));
    assert_eq!(busy(&pool, &user_id).await, before);

    // Idempotent.
    assert_eq!(crate::busy_only::scrub_stored(&pool).await.unwrap(), 0);
    crate::db::migrate(&pool).await.unwrap();
    assert_eq!(busy(&pool, &user_id).await, before);
}

async fn troubleshoot_label(lang: Option<&str>) -> String {
    let (app, pool, session, _) = setup_test_app().await;
    if let Some(lang) = lang {
        sqlx::query("UPDATE users SET language = ? WHERE email = 'test@example.com'")
            .bind(lang)
            .execute(&pool)
            .await
            .unwrap();
    }
    let account_id: String = sqlx::query_scalar("SELECT id FROM accounts LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let day = far_future_monday();
    let compact = day.replace('-', "");
    seed_synced_event(
        &pool,
        &account_id,
        &format!("{compact}T100000"),
        &format!("{compact}T110000"),
        "UTC",
        "OPAQUE",
    )
    .await;
    sqlx::query("UPDATE events SET summary = NULL")
        .execute(&pool)
        .await
        .unwrap();
    // Fresh, so the view does not sync it (no network in tests).
    sqlx::query("UPDATE caldav_sources SET last_synced = datetime('now')")
        .execute(&pool)
        .await
        .unwrap();
    let response = app
        .oneshot(get_authed(
            &format!("/dashboard/troubleshoot?date={day}&event_type=test-meeting"),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    body_string(response).await
}

#[tokio::test]
async fn troubleshoot_shows_busy_for_an_event_without_title() {
    let body = troubleshoot_label(None).await;
    assert!(body.contains("Calendar event: Busy"), "{body}");
    assert!(body.contains("Monday"), "{body}");
    let body = troubleshoot_label(Some("ru")).await;
    assert!(body.contains("Занято"), "{body}");
    assert!(body.contains("понедельник"), "{body}");
}

/// The provider path (Google, Microsoft, …) keeps the same busy times and
/// slots as with the switch off, and none of the content, raw iCal
/// included.
#[tokio::test]
async fn the_provider_path_is_busy_only_too() {
    let (full, full_user, full_et, _) = synced_by(false, Path::Provider).await;
    let (lean, lean_user, lean_et, _) = synced_by(true, Path::Provider).await;
    let full_busy = busy(&full, &full_user).await;
    assert!(full_busy.contains(&(at(0, 10), at(0, 11))));
    assert!(full_busy.contains(&(at(28, 15), at(28, 16))));
    assert_eq!(busy(&lean, &lean_user).await, full_busy);
    // And the same as the CalDAV path.
    let (caldav, caldav_user, _, _) = synced_by(true, Path::Caldav).await;
    assert_eq!(busy(&caldav, &caldav_user).await, full_busy);
    assert_eq!(
        slots(&lean, &lean_et, &lean_user).await,
        slots(&full, &full_et, &full_user).await
    );
    let stored = dump(&lean, "events").await;
    for word in CONTENT {
        assert!(!stored.contains(word), "{word} stored: {stored}");
    }
    assert!(!stored.contains("BEGIN:VCALENDAR"), "{stored}");
    assert!(dump(&full, "events").await.contains("Secret plans"));
}

/// A resource feed (a room's ICS) stores each VEVENT on its own: the same
/// busy times on and off, and no title or iCal content on.
#[tokio::test]
async fn the_resource_feed_path_is_busy_only_too() {
    let (full, full_rid) = synced_feed(false).await;
    let (lean, lean_rid) = synced_feed(true).await;
    let window = |pool: SqlitePool, rid: String| async move {
        let mut b =
            crate::resources::busy_for_resource(&pool, &rid, at(-1, 0), at(59, 0), dubai(), None)
                .await;
        b.sort();
        b
    };
    let full_busy = window(full.clone(), full_rid).await;
    assert!(full_busy.contains(&(at(0, 10), at(0, 11))), "{full_busy:?}");
    assert!(!full_busy.iter().any(|b| b.0 == at(7, 10)));
    assert!(full_busy.contains(&(at(14, 11), at(14, 12))));
    assert!(!full_busy.iter().any(|b| b.0 == at(15, 11)));
    assert_eq!(window(lean.clone(), lean_rid).await, full_busy);
    let stored = dump(&lean, "resource_events").await;
    for word in CONTENT {
        assert!(!stored.contains(word), "{word} stored: {stored}");
    }
    assert!(!stored.contains("BEGIN:VEVENT\r\nUID"), "{stored}");
    let raws: Vec<String> = sqlx::query_scalar(
        "SELECT raw_ical FROM resource_events WHERE raw_ical IS NOT NULL ORDER BY uid",
    )
    .fetch_all(&lean)
    .await
    .unwrap();
    assert!(!raws.is_empty());
    assert!(raws.iter().all(|r| r.starts_with(MARKER)), "{raws:?}");
    assert!(dump(&full, "resource_events").await.contains("Secret"));
}

/// After a scrub that changed rows, the removed content is not left in the
/// database file's free pages or in its WAL (backups copy every page): the
/// file is rebuilt and the WAL emptied. On a real file, through the same
/// connect as `calrs serve` (secure_delete on).
#[tokio::test]
async fn the_scrub_leaves_no_content_in_the_file() {
    let dir = TestDataDir::new();
    let pool = crate::db::connect(&dir).await.unwrap();
    let _off = test_mode(false);
    crate::db::migrate(&pool).await.unwrap();
    let secure: i64 = sqlx::query_scalar("PRAGMA secure_delete")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(secure, 1);
    // Content cached with the switch off: big enough for overflow pages.
    let (_, account_id, _) = seed_test_data(&pool).await;
    let source_id = uuid::Uuid::new_v4().to_string();
    let cal_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO caldav_sources (id, account_id, name, url, username) VALUES (?, ?, 'Src', 'https://x/dav', 'u')")
        .bind(&source_id).bind(&account_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO calendars (id, source_id, href, display_name, is_busy) VALUES (?, ?, '/cal/', 'Cal', 1)")
        .bind(&cal_id).bind(&source_id).execute(&pool).await.unwrap();
    let filler = "Secret agenda line. ".repeat(400);
    let raw: Vec<RawEvent> = (0..40)
        .map(|n| RawEvent {
            href: format!("/cal/e{n}.ics"),
            ical_data: format!(
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:e{n}\r\nDTSTART:20300107T100000Z\r\nDTEND:20300107T110000Z\r\nSUMMARY:ZZSECRETTITLE{n}\r\nDESCRIPTION:{filler}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
            ),
            etag: None,
        })
        .collect();
    crate::commands::sync::upsert_raw_events(&pool, &cal_id, &raw).await;
    sqlx::raw_sql("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&pool)
        .await
        .unwrap();
    let file = dir.to_path_buf().join("calrs.db");
    let read = || {
        let mut bytes = std::fs::read(&file).unwrap();
        if let Ok(wal) = std::fs::read(file.with_extension("db-wal")) {
            bytes.extend(wal);
        }
        bytes
    };
    let has =
        |bytes: &[u8], needle: &str| bytes.windows(needle.len()).any(|w| w == needle.as_bytes());
    assert!(has(&read(), "ZZSECRETTITLE"));

    // The switch back on: the startup scrub changes rows, then wipes.
    let _on = test_mode(true);
    crate::db::migrate(&pool).await.unwrap();
    let bytes = read();
    assert!(!has(&bytes, "ZZSECRETTITLE"));
    assert!(!has(&bytes, "Secret agenda"));
    // Nothing to scrub the next time: no rebuild either.
    assert_eq!(crate::busy_only::scrub_stored(&pool).await.unwrap(), 0);
    pool.close().await;
}
