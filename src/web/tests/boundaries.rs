//! ilia fork, Boundaries stage: the fork-wide horizon on every booking path
//! and the windowed, boundaries-only calendar sync, end to end against the
//! Kerio-style mock DAV server (`caldav::partial::mock_dav`).

use super::*;
use crate::caldav::partial::mock_dav::{Behaviour, MockDav, CAL};
use crate::caldav::CaldavClient;
use crate::horizon::test_limit;

const KEY: [u8; 32] = [0u8; 32];

/// Bytes transferred before and after, on a realistic calendar (`#[ignore]`,
/// run alone: `cargo test --locked perf_caldav_bytes -- --ignored --nocapture
/// --test-threads 1`).
#[path = "boundaries_bytes.rs"]
mod bytes;

// --- Fixtures ---

/// Whole weeks from the fixtures' anchor (2026-11-10) to the current week,
/// so a shifted fixture keeps its weekdays.
fn shift_days() -> i64 {
    let anchor = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
    let days = (Utc::now().date_naive() - anchor).num_days();
    days.div_euclid(7) * 7
}

/// `text` with every `2026MMDD` date moved by [`shift_days`].
fn shift(text: &str) -> String {
    let delta = Duration::days(shift_days());
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let candidate = text.get(i..i + 8).filter(|c| {
            c.starts_with("2026")
                && c.bytes().all(|b| b.is_ascii_digit())
                && (i == 0 || !bytes[i - 1].is_ascii_digit())
        });
        match candidate.and_then(|c| NaiveDate::parse_from_str(c, "%Y%m%d").ok()) {
            Some(date) => {
                out.push_str(&(date + delta).format("%Y%m%d").to_string());
                i += 8;
            }
            None => {
                let ch = text[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    out
}

/// The Kerio fixture, moved to this week.
fn kerio(full: bool) -> Vec<(String, String)> {
    crate::caldav::partial::tests::kerio_resources(full)
        .into_iter()
        .map(|(name, data)| (name, shift(&data)))
        .collect()
}

/// A one-off event `days` from now at 10:00 UTC.
fn single(uid: &str, days: i64, full: bool) -> (String, String) {
    let day = (Utc::now() + Duration::days(days)).format("%Y%m%d");
    let content = if full {
        "SUMMARY:confidential title\nDESCRIPTION:confidential\n"
    } else {
        ""
    };
    (
        format!("{uid}-00000001.eml"),
        format!(
            "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nUID:{uid}\nDTSTART:{day}T100000Z\nDTEND:{day}T110000Z\n{content}END:VEVENT\nEND:VCALENDAR\n"
        ),
    )
}

/// A database with the test user, and a CalDAV source of theirs pointing at
/// `dav`, writing to its calendar. Returns the pool, user, event type and
/// source.
async fn seeded(dav: &MockDav) -> (SqlitePool, String, String, String) {
    let pool = setup_test_db().await;
    let (user_id, account_id, et_id) = seed_test_data(&pool).await;
    let source_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO caldav_sources (id, account_id, name, url, username, password_enc, enabled, write_calendar_href) \
         VALUES (?, ?, 'Kerio', ?, 'owner@ilia.ae', ?, 1, ?)",
    )
    .bind(&source_id)
    .bind(&account_id)
    .bind(&dav.base)
    .bind(crate::crypto::encrypt_password(&KEY, "p").unwrap())
    .bind(CAL)
    .execute(&pool)
    .await
    .unwrap();
    (pool, user_id, et_id, source_id)
}

async fn sync(pool: &SqlitePool, dav: &MockDav, source_id: &str) {
    let client = CaldavClient::new(&dav.base, "owner@ilia.ae", "p");
    crate::commands::sync::sync_source(pool, &KEY, &client, source_id)
        .await
        .unwrap();
}

/// What the daily resync does first: forget ctag and token.
async fn force_full(pool: &SqlitePool) {
    sqlx::query("UPDATE calendars SET ctag = NULL, sync_token = NULL")
        .execute(pool)
        .await
        .unwrap();
}

fn reports(dav: &MockDav) -> Vec<crate::caldav::partial::mock_dav::Logged> {
    dav.log()
        .into_iter()
        .filter(|l| l.method == "REPORT")
        .collect()
}

async fn uids(pool: &SqlitePool) -> Vec<String> {
    sqlx::query_scalar("SELECT uid || '/' || COALESCE(recurrence_id, '') FROM events ORDER BY 1")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn insert_confirmed_booking(pool: &SqlitePool, et_id: &str, uid: &str, days: i64) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let start = Utc::now() + Duration::days(days);
    sqlx::query(
        "INSERT INTO bookings (id, event_type_id, uid, guest_name, guest_email, guest_timezone, start_at, end_at, status, cancel_token, reschedule_token, caldav_calendar_href, time_version) \
         VALUES (?, ?, ?, 'Guest', 'guest@test.com', 'UTC', ?, ?, 'confirmed', ?, ?, ?, 1)",
    )
    .bind(&id)
    .bind(et_id)
    .bind(uid)
    .bind(utc_stamp(start))
    .bind(utc_stamp(start + Duration::minutes(30)))
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(CAL)
    .execute(pool)
    .await
    .unwrap();
    id
}

// --- The windowed sync ---

#[tokio::test]
async fn the_sync_asks_for_the_boundaries_inside_the_window_only() {
    let _cap = test_limit(14);
    let mut resources = kerio(true);
    resources.push(single("inside", 3, true));
    resources.push(single("far", 40, true));
    resources.push(single("past", -30, true));
    let dav = MockDav::start(resources, Behaviour::default()).await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;

    let first = &reports(&dav)[0];
    assert!(first.body.contains("calendar-query"), "{}", first.body);
    assert!(first.body.contains("<c:comp name=\"VEVENT\">"));
    assert!(first.body.contains("<c:prop name=\"DTSTART\"/>"));
    assert!(!first.body.contains("SUMMARY"));
    let end = first
        .body
        .split("<c:time-range ")
        .nth(1)
        .and_then(|s| s.split("end=\"").nth(1))
        .and_then(|s| s.get(..16))
        .unwrap()
        .to_string();
    let latest = (Utc::now() + Duration::days(17) + Duration::minutes(5))
        .format("%Y%m%dT%H%M%SZ")
        .to_string();
    assert!(end <= latest, "time range ends {end}, past {latest}");

    let stored = uids(&pool).await;
    assert!(stored.contains(&"inside/".to_string()), "{stored:?}");
    assert!(stored.contains(&"kerio-weekly/".to_string()), "{stored:?}");
    assert!(
        !stored
            .iter()
            .any(|u| u.starts_with("far/") || u.starts_with("past/")),
        "{stored:?}"
    );

    // Rows carry their resource's href and etag; no content is kept, and
    // partial retrieval sent none (the mock honours it).
    let (href, etag): (String, String) =
        sqlx::query_as("SELECT href, etag FROM events WHERE uid = 'inside'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(href, MockDav::href("inside-00000001.eml"));
    assert!(etag.starts_with('"'));
    assert!(!reports(&dav)[0].body.is_empty());
    let answers: String = dav.log().iter().map(|l| l.bytes.to_string()).collect();
    assert!(!answers.is_empty());
    let content: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM events WHERE summary IS NOT NULL OR description IS NOT NULL OR location IS NOT NULL OR raw_ical LIKE '%confidential%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(content, 0);
    let synced_until: Option<String> = sqlx::query_scalar("SELECT synced_until FROM calendars")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(synced_until.is_some_and(|u| u == end));
}

#[tokio::test]
async fn an_unchanged_calendar_is_skipped_until_its_cache_falls_short() {
    let _cap = test_limit(14);
    let dav = MockDav::start(
        vec![single("a", 2, false), single("b", 5, false)],
        Behaviour::default(),
    )
    .await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;

    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    assert!(
        reports(&dav).is_empty(),
        "ctag unchanged and covered: skipped"
    );

    // The window slid: the cache no longer reaches the horizon plus a day.
    let short = (Utc::now() + Duration::days(10))
        .format("%Y%m%dT%H%M%SZ")
        .to_string();
    sqlx::query("UPDATE calendars SET synced_until = ?")
        .bind(&short)
        .execute(&pool)
        .await
        .unwrap();
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    let seen = reports(&dav);
    assert_eq!(
        seen.len(),
        1,
        "one etag listing, nothing to fetch: {seen:?}"
    );
    assert!(seen[0].body.contains("calendar-query") && !seen[0].body.contains("calendar-data"));
    let until: String = sqlx::query_scalar("SELECT synced_until FROM calendars")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(until > short);
}

#[tokio::test]
async fn the_daily_resync_fetches_only_changed_resources() {
    let _cap = test_limit(14);
    let dav = MockDav::start(
        vec![
            single("a", 2, false),
            single("b", 5, false),
            single("c", 7, false),
        ],
        Behaviour::default(),
    )
    .await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;

    let (_, changed) = single("b", 6, false);
    dav.put("b-00000001.eml", &changed);
    force_full(&pool).await;
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    let seen = reports(&dav);
    assert_eq!(seen.len(), 2, "{seen:?}");
    assert!(
        !seen[0].body.contains("calendar-data"),
        "etag listing first"
    );
    assert!(seen[1].body.contains("calendar-multiget"));
    assert_eq!(
        seen[1].body.matches("<d:href>").count(),
        1,
        "{}",
        seen[1].body
    );
    assert!(seen[1].body.contains("b-00000001.eml"));
    let day6 = (Utc::now() + Duration::days(6))
        .format("%Y%m%d")
        .to_string();
    let b_start: String = sqlx::query_scalar("SELECT start_at FROM events WHERE uid = 'b'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(b_start.starts_with(&day6));
    assert_eq!(uids(&pool).await, vec!["a/", "b/", "c/"]);
}

#[tokio::test]
async fn a_delta_lists_etags_and_multigets_what_changed() {
    let _cap = test_limit(14);
    let dav = MockDav::start(
        vec![
            single("a", 2, false),
            single("b", 5, false),
            single("c", 7, false),
        ],
        Behaviour::default(),
    )
    .await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;

    let (_, moved) = single("a", 3, false);
    dav.put("a-00000001.eml", &moved);
    dav.delete("c-00000001.eml");
    let (far_name, far) = single("far", 60, false);
    dav.put(&far_name, &far);
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    let seen = reports(&dav);
    assert!(seen[0].body.contains("sync-collection"), "{seen:?}");
    assert!(!seen[0].body.contains("calendar-data"), "etags only");
    assert!(seen[1].body.contains("calendar-multiget"));
    assert_eq!(seen.len(), 2, "{seen:?}");
    // The deletion matched by href (a Kerio `.eml`, not `{uid}.ics`); the
    // far change was fetched, then pruned.
    assert_eq!(uids(&pool).await, vec!["a/", "b/"]);
}

#[tokio::test]
async fn a_deleted_eml_resource_cancels_only_its_own_booking() {
    let _cap = test_limit(14);
    let dav = MockDav::start(
        vec![single("bk-1", 3, false), single("bk-2", 4, false)],
        Behaviour::default(),
    )
    .await;
    let (pool, _, et_id, source_id) = seeded(&dav).await;
    let gone = insert_confirmed_booking(&pool, &et_id, "bk-1", 3).await;
    let kept = insert_confirmed_booking(&pool, &et_id, "bk-2", 4).await;
    sync(&pool, &dav, &source_id).await;

    dav.delete("bk-1-00000001.eml");
    sync(&pool, &dav, &source_id).await;
    assert_eq!(booking_status(&pool, &gone).await, "cancelled");
    assert_eq!(booking_status(&pool, &kept).await, "confirmed");
    // Confirmed with the server first (HEAD on the booking's own resource).
    assert!(dav
        .log()
        .iter()
        .any(|l| l.method == "HEAD" && l.path.ends_with("/bk-1.ics")));
}

#[tokio::test]
async fn the_first_windowed_sync_prunes_old_and_far_rows_without_cancelling() {
    let _cap = test_limit(14);
    let dav = MockDav::start(vec![single("a", 2, false)], Behaviour::default()).await;
    let (pool, _, et_id, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    let cal_id: String = sqlx::query_scalar("SELECT id FROM calendars")
        .fetch_one(&pool)
        .await
        .unwrap();
    // What upstream's 90-day, two-year cache left, and a booking beyond the
    // year the window can ever reach, whose event is not cached.
    let day = |n: i64| {
        (Utc::now() + Duration::days(n))
            .format("%Y%m%dT100000")
            .to_string()
    };
    for (uid, n) in [("old", -60), ("far", 200), ("ended-weekly", -400)] {
        sqlx::query("INSERT INTO events (id, calendar_id, uid, start_at, end_at, rrule) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&cal_id)
            .bind(uid)
            .bind(day(n))
            .bind(day(n))
            .bind((uid == "ended-weekly").then(|| format!("FREQ=WEEKLY;UNTIL={}", day(-100))))
            .execute(&pool)
            .await
            .unwrap();
    }
    let beyond = insert_confirmed_booking(&pool, &et_id, "beyond-a-year", 400).await;

    force_full(&pool).await;
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    assert_eq!(uids(&pool).await, vec!["a/"]);
    assert_eq!(booking_status(&pool, &beyond).await, "confirmed");
    assert!(
        !dav.log()
            .iter()
            .any(|l| l.method == "HEAD" || l.path.ends_with(".ics")),
        "no booking was checked: {:?}",
        dav.log()
    );
}

#[tokio::test]
async fn a_recurring_master_missing_from_the_window_is_dropped_plainly() {
    let _cap = test_limit(14);
    let dav = MockDav::start(vec![single("a", 2, false)], Behaviour::default()).await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    sqlx::query(
        "INSERT INTO events (id, calendar_id, uid, start_at, end_at, rrule, href, etag) \
         SELECT 'gone', id, 'gone-master', '20240101T100000', '20240101T110000', 'FREQ=WEEKLY', ?, '\"1\"' FROM calendars",
    )
    .bind(MockDav::href("gone-00000001.eml"))
    .execute(&pool)
    .await
    .unwrap();
    force_full(&pool).await;
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    assert_eq!(uids(&pool).await, vec!["a/"]);
    assert!(!dav.log().iter().any(|l| l.method == "HEAD"));
}

#[tokio::test]
async fn a_refused_partial_body_is_retried_whole_and_remembered() {
    let _cap = test_limit(14);
    let dav = MockDav::start(
        vec![single("a", 2, true)],
        Behaviour {
            refuse_partial: true,
            ..Default::default()
        },
    )
    .await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    let seen = reports(&dav);
    assert_eq!(seen[0].status, 400);
    assert!(seen[1].body.contains("<c:calendar-data/>"));
    assert_eq!(uids(&pool).await, vec!["a/"]);
    let content: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE summary IS NOT NULL")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(content, 0, "whole objects are still stripped");

    // Remembered: the next fetch of this calendar asks for whole objects.
    let (_, changed) = single("a", 3, true);
    dav.put("a-00000001.eml", &changed);
    force_full(&pool).await;
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    assert!(
        !reports(&dav).iter().any(|l| l.body.contains("<c:comp ")),
        "{:?}",
        reports(&dav)
    );
}

#[tokio::test]
async fn a_refused_time_range_falls_back_to_the_unfiltered_report() {
    let _cap = test_limit(14);
    let dav = MockDav::start(
        vec![single("a", 2, false), single("far", 60, false)],
        Behaviour {
            refuse_time_range: true,
            ..Default::default()
        },
    )
    .await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    assert!(reports(&dav)
        .iter()
        .any(|l| l.status == 207 && !l.body.contains("time-range")));
    // Complete, then pruned to the window.
    assert_eq!(uids(&pool).await, vec!["a/"]);
}

#[tokio::test]
async fn a_refused_etag_delta_or_multiget_falls_back_to_inline_data() {
    for behaviour in [
        Behaviour {
            refuse_etag_sync: true,
            ..Default::default()
        },
        Behaviour {
            refuse_multiget: true,
            ..Default::default()
        },
    ] {
        let _cap = test_limit(14);
        let dav = MockDav::start(vec![single("a", 2, false)], behaviour.clone()).await;
        let (pool, _, _, source_id) = seeded(&dav).await;
        sync(&pool, &dav, &source_id).await;
        let (name, b) = single("b", 4, false);
        dav.put(&name, &b);
        dav.clear_log();
        sync(&pool, &dav, &source_id).await;
        assert!(
            reports(&dav)
                .iter()
                .any(|l| l.body.contains("sync-collection") && l.body.contains("calendar-data")),
            "{behaviour:?}: {:?}",
            reports(&dav)
        );
        assert_eq!(uids(&pool).await, vec!["a/", "b/"], "{behaviour:?}");
    }
}

// --- Kerio: the same busy times however the server answers ---

/// The Kerio fixture synced, in one of three ways.
#[derive(Clone, Copy, Debug)]
enum Way {
    /// The windowed sync, partial retrieval honoured.
    Boundaries,
    /// The windowed sync, partial retrieval ignored (whole objects).
    Ignored,
    /// Upstream's flow (no horizon in tests): whole objects from −90 days.
    Upstream,
}

async fn kerio_synced(way: Way) -> (SqlitePool, String, String) {
    let _cap = match way {
        Way::Upstream => None,
        _ => Some(test_limit(14)),
    };
    let behaviour = Behaviour {
        honour_partial: matches!(way, Way::Boundaries),
        ..Default::default()
    };
    let dav = MockDav::start(kerio(!matches!(way, Way::Boundaries)), behaviour).await;
    let (pool, user_id, et_id, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    (pool, user_id, et_id)
}

async fn kerio_busy(pool: &SqlitePool, user_id: &str) -> Vec<(NaiveDateTime, NaiveDateTime)> {
    let today = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
    let mut busy = fetch_busy_times_for_user(
        pool,
        user_id,
        today,
        today + Duration::days(14),
        Tz::UTC,
        None,
    )
    .await;
    busy.sort();
    busy
}

#[tokio::test]
async fn kerio_busy_times_and_slots_match_however_the_server_answers() {
    let mut seen = Vec::new();
    for way in [Way::Boundaries, Way::Ignored, Way::Upstream] {
        let (pool, user_id, et_id) = kerio_synced(way).await;
        let busy = kerio_busy(&pool, &user_id).await;
        let slots = compute_slots(
            &pool,
            &et_id,
            30,
            0,
            0,
            0,
            0,
            14,
            BookingWindow::default(),
            Tz::UTC,
            Tz::UTC,
            BusySource::Individual(busy.clone()),
            None,
        )
        .await;
        let slots: Vec<(String, Vec<String>)> = slots
            .into_iter()
            .map(|d| (d.date, d.slots.into_iter().map(|s| s.start).collect()))
            .collect();
        let content: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE summary IS NOT NULL OR raw_ical LIKE '%confidential%' OR raw_ical LIKE '%ATTENDEE%'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(content, 0, "{way:?}");
        seen.push((way, busy, slots));
    }
    let (_, busy, slots) = &seen[0];
    assert!(!busy.is_empty());
    for (way, other_busy, other_slots) in &seen[1..] {
        assert_eq!(busy, other_busy, "{way:?}");
        assert_eq!(slots, other_slots, "{way:?}");
    }
}

/// A moved instance Kerio stores as a resource of its own (same UID, a
/// RECURRENCE-ID) blocks its new time, and no longer its original one.
#[tokio::test]
async fn a_separately_stored_override_frees_its_original_time() {
    let (pool, user_id, _) = kerio_synced(Way::Boundaries).await;
    let busy = kerio_busy(&pool, &user_id).await;
    let monday = NaiveDate::from_ymd_opt(2026, 11, 16).unwrap() + Duration::days(shift_days());
    // Moscow is UTC+3: 10:00 there is 07:00 UTC, 17:00 is 14:00.
    let original = monday.and_hms_opt(7, 0, 0).unwrap();
    let moved = monday.and_hms_opt(14, 0, 0).unwrap();
    if monday < Utc::now().date_naive() {
        return; // the fixture's Monday is behind us this week
    }
    assert!(busy.iter().any(|(s, _)| *s == moved), "{busy:?}");
    assert!(!busy.iter().any(|(s, _)| *s == original), "{busy:?}");
}

// --- Review fixes: the windowed sync ---

/// A PROPFIND without a sync token (the token only in REPORT answers): the
/// initial token comes from an etag-only `sync-collection`, never one with
/// the whole calendar's data.
#[tokio::test]
async fn a_full_fetch_without_a_propfind_token_asks_for_one_without_data() {
    let _cap = test_limit(14);
    let dav = MockDav::start(
        vec![single("a", 2, true), single("old", -300, true)],
        Behaviour {
            token_in_report_only: true,
            ..Default::default()
        },
    )
    .await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    let seen = reports(&dav);
    assert!(
        !seen
            .iter()
            .any(|l| l.body.contains("sync-collection") && l.body.contains("calendar-data")),
        "{seen:?}"
    );
    let initial = seen
        .iter()
        .find(|l| l.body.contains("sync-collection"))
        .expect("an initial token request");
    assert!(initial.body.contains("<d:sync-token></d:sync-token>"));
    assert!(!initial.body.contains("confidential"));
    let token: Option<String> = sqlx::query_scalar("SELECT sync_token FROM calendars")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(token.is_some_and(|t| t.starts_with("watermark:")));
    // The token works: the next change comes as an etag delta.
    let (name, b) = single("b", 3, false);
    dav.put(&name, &b);
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    assert!(reports(&dav)[0].body.contains("sync-collection"));
    assert_eq!(uids(&pool).await, vec!["a/", "b/"]);
}

/// A 207 whose `calendar-data` comes back in a 404 propstat refuses partial
/// retrieval as surely as a 400: the query and the multiget ask again for
/// whole objects, and nothing cached is taken for deleted.
#[tokio::test]
async fn partial_retrieval_refused_inside_a_207_is_asked_again_whole() {
    let _cap = test_limit(14);
    let dav = MockDav::start(
        vec![single("a", 2, false), single("b", 4, false)],
        Behaviour::default(),
    )
    .await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    assert_eq!(uids(&pool).await, vec!["a/", "b/"]);

    // From now on the server refuses partial bodies inside its answers.
    dav.behaviour.lock().unwrap().refuse_partial_in_body = true;
    let (_, moved) = single("b", 5, false);
    dav.put("b-00000001.eml", &moved);
    force_full(&pool).await;
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    let seen = reports(&dav);
    assert!(
        seen.iter()
            .any(|l| l.body.contains("calendar-multiget") && l.body.contains("<c:calendar-data/>")),
        "asked again whole: {seen:?}"
    );
    assert_eq!(uids(&pool).await, vec!["a/", "b/"]);
    let day5 = (Utc::now() + Duration::days(5))
        .format("%Y%m%d")
        .to_string();
    let b_start: String = sqlx::query_scalar("SELECT start_at FROM events WHERE uid = 'b'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(b_start.starts_with(&day5), "{b_start}");

    // A fresh calendar on such a server: the window query is retried whole.
    let dav = MockDav::start(
        vec![single("c", 2, false)],
        Behaviour {
            refuse_partial_in_body: true,
            ..Default::default()
        },
    )
    .await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    assert_eq!(uids(&pool).await, vec!["c/"]);
}

/// A resource the multiget leaves out (neither data nor reported gone) is
/// unknown, not deleted: its cached rows stay, in a full fetch and in a
/// delta (which then hands over to the full fetch without advancing the
/// token).
#[tokio::test]
async fn a_resource_missing_from_a_multiget_keeps_its_cached_rows() {
    let _cap = test_limit(14);
    let dav = MockDav::start(
        vec![single("a", 2, false), single("b", 4, false)],
        Behaviour::default(),
    )
    .await;
    let (pool, _, et_id, source_id) = seeded(&dav).await;
    let kept = insert_confirmed_booking(&pool, &et_id, "b", 4).await;
    sync(&pool, &dav, &source_id).await;
    dav.behaviour.lock().unwrap().multiget_omits = vec![MockDav::href("b-00000001.eml")];
    let (_, moved) = single("b", 5, false);
    dav.put("b-00000001.eml", &moved);
    // The delta.
    sync(&pool, &dav, &source_id).await;
    assert_eq!(uids(&pool).await, vec!["a/", "b/"]);
    // The daily resync.
    force_full(&pool).await;
    sync(&pool, &dav, &source_id).await;
    assert_eq!(uids(&pool).await, vec!["a/", "b/"]);
    assert_eq!(booking_status(&pool, &kept).await, "confirmed");
}

/// A running multi-day event (started before the window) deleted on the
/// server goes with the next full fetch: the orphan check scopes one-off
/// rows by overlap, as the time-range query returns them.
#[tokio::test]
async fn a_running_multi_day_event_deleted_on_the_server_is_removed() {
    let _cap = test_limit(14);
    let start = (Utc::now() - Duration::days(5)).format("%Y%m%d");
    let end = (Utc::now() + Duration::days(16)).format("%Y%m%d");
    let ooo = (
        "ooo-00000001.eml".to_string(),
        format!("BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nUID:ooo\nDTSTART;VALUE=DATE:{start}\nDTEND;VALUE=DATE:{end}\nEND:VEVENT\nEND:VCALENDAR\n"),
    );
    let dav = MockDav::start(vec![single("a", 2, false), ooo], Behaviour::default()).await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    assert_eq!(uids(&pool).await, vec!["a/", "ooo/"]);
    dav.delete("ooo-00000001.eml");
    // As after a delta that missed the deletion (BlueMind) or a reset token.
    force_full(&pool).await;
    sync(&pool, &dav, &source_id).await;
    assert_eq!(uids(&pool).await, vec!["a/"]);
}

/// Stored times written with dashes (a feed's `2026-11-20T10:00:00`) are
/// compared in compact form: a row inside the window stays, one ended
/// before it goes.
#[tokio::test]
async fn the_prune_reads_dashed_times_in_compact_form() {
    let pool = setup_test_db().await;
    let (_, account_id, _) = seed_test_data(&pool).await;
    let source_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO caldav_sources (id, account_id, name, url, username, password_enc) VALUES (?, ?, 's', 'http://x', 'u', 'p')")
        .bind(&source_id)
        .bind(&account_id)
        .execute(&pool)
        .await
        .unwrap();
    let cal_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO calendars (id, source_id, href, display_name) VALUES (?, ?, '/c/', 'c')",
    )
    .bind(&cal_id)
    .bind(&source_id)
    .execute(&pool)
    .await
    .unwrap();
    let dashed = |days: i64| {
        (Utc::now() + Duration::days(days))
            .format("%Y-%m-%dT10:00:00")
            .to_string()
    };
    for (uid, days) in [("inside", 3), ("ended", -10), ("far", 40)] {
        sqlx::query(
            "INSERT INTO events (id, calendar_id, uid, start_at, end_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&cal_id)
        .bind(uid)
        .bind(dashed(days))
        .bind(dashed(days))
        .execute(&pool)
        .await
        .unwrap();
    }
    let window = crate::horizon::window_for(Utc::now(), 14, None);
    crate::commands::sync_window::prune(&pool, &cal_id, &window).await;
    assert_eq!(uids(&pool).await, vec!["inside/"]);
}

/// The next Monday at least two days ahead.
fn next_monday() -> NaiveDate {
    let mut d = Utc::now().date_naive() + Duration::days(2);
    while d.weekday() != chrono::Weekday::Mon {
        d += Duration::days(1);
    }
    d
}

/// A weekly Monday 10:00 Moscow master from four weeks before `monday`,
/// and one of its instances on `monday` moved to `moved_to`, stored as a
/// resource of its own (Kerio), its RECURRENCE-ID under `rid_tz` at
/// `rid_time`.
fn kerio_series(
    monday: NaiveDate,
    moved_to: NaiveDateTime,
    rid_tz: &str,
    rid_time: &str,
) -> Vec<(String, String)> {
    let first = monday - Duration::weeks(4);
    let moscow = "BEGIN:VTIMEZONE\nTZID:Europe/Moscow\nBEGIN:STANDARD\nDTSTART:19700101T000000\nTZOFFSETFROM:+0300\nTZOFFSETTO:+0300\nEND:STANDARD\nEND:VTIMEZONE\n";
    vec![
        (
            "m1-00000001.eml".into(),
            format!(
                "BEGIN:VCALENDAR\nVERSION:2.0\n{moscow}BEGIN:VEVENT\nUID:series\nDTSTART;TZID=Europe/Moscow:{}T100000\nDTEND;TZID=Europe/Moscow:{}T110000\nRRULE:FREQ=WEEKLY;BYDAY=MO\nSUMMARY:secret\nEND:VEVENT\nEND:VCALENDAR\n",
                first.format("%Y%m%d"),
                first.format("%Y%m%d")
            ),
        ),
        (
            "m2-00000001.eml".into(),
            format!(
                "BEGIN:VCALENDAR\nVERSION:2.0\n{moscow}BEGIN:VEVENT\nUID:series\nRECURRENCE-ID;TZID={rid_tz}:{}T{rid_time}\nDTSTART;TZID=Europe/Moscow:{}\nDTEND;TZID=Europe/Moscow:{}\nSUMMARY:secret\nEND:VEVENT\nEND:VCALENDAR\n",
                monday.format("%Y%m%d"),
                moved_to.format("%Y%m%dT%H%M%S"),
                (moved_to + Duration::hours(1)).format("%Y%m%dT%H%M%S")
            ),
        ),
    ]
}

/// Kerio stores a moved instance as a resource of its own; moved out of the
/// window from a slot inside it, the time-range query never lists it. The
/// full fetch asks for moved instances by RECURRENCE-ID, so the original
/// slot is free; a server that ignores partial retrieval is not asked.
#[tokio::test]
async fn an_instance_moved_past_the_window_frees_its_original_slot() {
    let _cap = test_limit(14);
    let monday = next_monday();
    let far = (monday + Duration::days(42)).and_hms_opt(10, 0, 0).unwrap();
    let dav = MockDav::start(
        kerio_series(monday, far, "Europe/Moscow", "100000"),
        Behaviour::default(),
    )
    .await;
    let (pool, user_id, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    let query = reports(&dav)
        .into_iter()
        .find(|l| l.body.contains("<c:prop-filter name=\"RECURRENCE-ID\"/>"))
        .expect("the query for moved instances");
    assert!(!query.body.contains("time-range"));
    assert!(!query.body.contains("\"SUMMARY\""));
    assert!(
        uids(&pool)
            .await
            .iter()
            .any(|u| u.starts_with("series/") && u.len() > 7),
        "{:?}",
        uids(&pool).await
    );
    let busy = kerio_busy(&pool, &user_id).await;
    // Moscow is UTC+3: 10:00 there is 07:00 UTC.
    let original = monday.and_hms_opt(7, 0, 0).unwrap();
    assert!(!busy.iter().any(|(s, _)| *s == original), "{busy:?}");
    let week_after = (monday + Duration::days(7)).and_hms_opt(7, 0, 0).unwrap();
    if week_after < (Utc::now() + Duration::days(14)).naive_utc() {
        assert!(busy.iter().any(|(s, _)| *s == week_after), "{busy:?}");
    }
    // The daily resync does not fetch it again (same etag).
    force_full(&pool).await;
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    assert!(
        !reports(&dav)
            .iter()
            .any(|l| l.body.contains("calendar-multiget") && l.body.contains("m2-")),
        "{:?}",
        reports(&dav)
    );
    let busy = kerio_busy(&pool, &user_id).await;
    assert!(!busy.iter().any(|(s, _)| *s == original), "{busy:?}");

    // Partial retrieval ignored: whole objects would come, so not asked.
    let dav = MockDav::start(
        kerio_series(monday, far, "Europe/Moscow", "100000"),
        Behaviour {
            honour_partial: false,
            ..Default::default()
        },
    )
    .await;
    let (pool, _, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    assert!(!reports(&dav).iter().any(|l| l.body.contains("prop-filter")));
}

/// A moved instance's RECURRENCE-ID written under another zone than its
/// master's (Asia/Dubai 11:00 is Moscow 10:00) still frees the original
/// slot: its TZID is kept with the row.
#[tokio::test]
async fn a_sibling_recurrence_id_keeps_its_tzid() {
    let _cap = test_limit(14);
    let monday = next_monday();
    let moved = monday.and_hms_opt(17, 0, 0).unwrap();
    let dav = MockDav::start(
        kerio_series(monday, moved, "Asia/Dubai", "110000"),
        Behaviour::default(),
    )
    .await;
    let (pool, user_id, _, source_id) = seeded(&dav).await;
    sync(&pool, &dav, &source_id).await;
    let tzid: Option<String> = sqlx::query_scalar(
        "SELECT recurrence_tzid FROM events WHERE uid = 'series' AND recurrence_id IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(tzid.as_deref(), Some("Asia/Dubai"));
    let busy = kerio_busy(&pool, &user_id).await;
    let original = monday.and_hms_opt(7, 0, 0).unwrap();
    let moved_utc = monday.and_hms_opt(14, 0, 0).unwrap();
    assert!(!busy.iter().any(|(s, _)| *s == original), "{busy:?}");
    assert!(busy.iter().any(|(s, _)| *s == moved_utc), "{busy:?}");
}

// --- The horizon on the booking paths ---

#[tokio::test]
async fn the_slot_page_offers_nothing_past_the_horizon() {
    let _cap = test_limit(14);
    let (app, _pool, _, _) = setup_test_app().await;
    let today = host_today();
    let last = today + Duration::days(14);
    let mut offered = Vec::new();
    for month in [today, today + Duration::days(31)] {
        let body = body_string(
            app.clone()
                .oneshot(get(&format!(
                    "/u/testuser/test-meeting?month={}",
                    month.format("%Y-%m")
                )))
                .await
                .unwrap(),
        )
        .await;
        let dates = body
            .split("\"availableDates\": {")
            .nth(1)
            .and_then(|s| s.split('}').next())
            .unwrap_or("")
            .to_string();
        offered.extend(
            dates
                .split(',')
                .filter_map(|e| e.trim().trim_matches('"').get(..10).map(str::to_string))
                .filter_map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
        );
    }
    assert!(!offered.is_empty());
    assert!(offered.iter().all(|d| *d <= last), "{offered:?}");
}

#[tokio::test]
async fn bookings_and_reschedules_past_the_horizon_are_refused() {
    let _cap = test_limit(14);
    let (app, pool, _, et_id) = setup_test_app().await;
    // A Monday about 28 days out, with no horizon of its own.
    let csrf = "csrf-cap";
    let response = app
        .clone()
        .oneshot(post_form_unauthed(
            "/u/testuser/test-meeting/book",
            csrf,
            &book_body(csrf),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(super::booking::booking_count(&pool, &et_id).await, 0);

    let resched = uuid::Uuid::new_v4().to_string();
    let id = insert_booking_at(
        &pool,
        &et_id,
        &uuid::Uuid::new_v4().to_string(),
        &resched,
        Duration::hours(48),
    )
    .await;
    let before: String = sqlx::query_scalar("SELECT start_at FROM bookings WHERE id = ?")
        .bind(&id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let body = format!(
        "_csrf={csrf}&date={}&time=10:00&tz=UTC",
        far_future_monday()
    );
    app.oneshot(post_form_unauthed(
        &format!("/booking/reschedule/{resched}"),
        csrf,
        &body,
    ))
    .await
    .unwrap();
    let after: String = sqlx::query_scalar("SELECT start_at FROM bookings WHERE id = ?")
        .bind(&id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(before, after);
}

#[tokio::test]
async fn troubleshoot_names_the_cap_and_the_unsynced_days() {
    let _cap = test_limit(14);
    let (app, _pool, session, _) = setup_test_app().await;
    let day = host_today() + Duration::days(20);
    let body = body_string(
        app.clone()
            .oneshot(get_authed(
                &format!(
                    "/dashboard/troubleshoot?date={}&event_type=test-meeting",
                    day.format("%Y-%m-%d")
                ),
                &session,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert!(body.contains("Beyond booking horizon (14 days)"), "{body}");
    assert!(body.contains("troubleshoot-not-synced"), "{body}");
    assert!(body.contains("are read only from"), "{body}");

    let near = host_today() + Duration::days(3);
    let body = body_string(
        app.oneshot(get_authed(
            &format!(
                "/dashboard/troubleshoot?date={}&event_type=test-meeting",
                near.format("%Y-%m-%d")
            ),
            &session,
        ))
        .await
        .unwrap(),
    )
    .await;
    assert!(!body.contains("troubleshoot-not-synced"));
}

#[tokio::test]
async fn the_event_type_form_says_the_horizon_is_capped() {
    let _cap = test_limit(14);
    let pool = setup_test_db().await;
    let (_, _, et_id) = seed_test_data(&pool).await;
    let window = get_booking_window(&pool, &et_id).await;
    let warnings = booking_window_warnings(&pool, &et_id, &window, "ru").await;
    let text = format!("{warnings:?}");
    assert!(text.contains("не более чем на 14 дней"), "{text}");
    // Information only: saving does not land back on the form for it.
    let response = after_event_type_save(&pool, &et_id, "/edit", "en").await;
    assert_eq!(
        response.headers()["location"],
        "/dashboard/event-types",
        "{:?}",
        response.headers()
    );
    set_horizon(&pool, &et_id, 7).await;
    let window = get_booking_window(&pool, &et_id).await;
    assert!(booking_window_warnings(&pool, &et_id, &window, "en")
        .await
        .is_empty());
}

// --- EWS: the CalendarView spans the window ---

struct WindowProvider {
    asked: std::sync::Mutex<Vec<(String, String)>>,
    day: String,
}

#[async_trait::async_trait]
impl crate::providers::CalendarProvider for WindowProvider {
    async fn check_connection(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
    async fn list_calendars(&self) -> anyhow::Result<Vec<crate::providers::RemoteCalendar>> {
        Ok(vec![crate::providers::RemoteCalendar {
            id: "folder-1".into(),
            display_name: Some("Calendar".into()),
            color: None,
            change_marker: None,
            sync_state: None,
        }])
    }
    async fn fetch_events(&self, _: &str) -> anyhow::Result<Vec<crate::providers::RawEvent>> {
        unreachable!("the windowed sync never reads the whole folder")
    }
    async fn fetch_events_since(
        &self,
        _: &str,
        _: &str,
    ) -> anyhow::Result<Vec<crate::providers::RawEvent>> {
        unreachable!("the windowed sync passes its end")
    }
    async fn fetch_events_window(
        &self,
        _: &str,
        since: &str,
        until: &str,
    ) -> anyhow::Result<Vec<crate::providers::RawEvent>> {
        self.asked
            .lock()
            .unwrap()
            .push((since.to_string(), until.to_string()));
        Ok(vec![crate::providers::RawEvent {
            remote_id: "item-1".into(),
            ical: format!(
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:ews-1\r\nDTSTART:{d}T100000Z\r\nDTEND:{d}T110000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                d = self.day
            ),
        }])
    }
    async fn sync_delta(
        &self,
        _: &str,
        _: Option<&str>,
    ) -> anyhow::Result<crate::providers::DeltaResult> {
        Ok(Default::default())
    }
    async fn put_event(&self, _: &str, _: &str, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete_event(&self, _: &str, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn event_exists(&self, _: &str, _: &str) -> anyhow::Result<bool> {
        panic!("no booking may be checked")
    }
}

#[tokio::test]
async fn ews_reads_the_window_and_prunes_beyond_it() {
    let _cap = test_limit(14);
    let pool = setup_test_db().await;
    let (_, account_id, _) = seed_test_data(&pool).await;
    let source_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO caldav_sources (id, account_id, name, url, username, provider_type) VALUES (?, ?, 'EWS', 'https://ews.example.com/EWS/Exchange.asmx', 'u', 'ews')")
        .bind(&source_id)
        .bind(&account_id)
        .execute(&pool)
        .await
        .unwrap();
    let provider = WindowProvider {
        asked: Default::default(),
        day: (Utc::now() + Duration::days(2))
            .format("%Y%m%d")
            .to_string(),
    };
    crate::commands::sync::sync_ews_source(&pool, &KEY, &provider, &source_id)
        .await
        .unwrap();
    let cal_id: String = sqlx::query_scalar("SELECT id FROM calendars")
        .fetch_one(&pool)
        .await
        .unwrap();
    let far = (Utc::now() + Duration::days(200))
        .format("%Y%m%dT100000Z")
        .to_string();
    sqlx::query("INSERT INTO events (id, calendar_id, uid, start_at, end_at) VALUES ('far', ?, 'far', ?, ?)")
        .bind(&cal_id)
        .bind(&far)
        .bind(&far)
        .execute(&pool)
        .await
        .unwrap();
    crate::commands::sync::sync_ews_source(&pool, &KEY, &provider, &source_id)
        .await
        .unwrap();

    let asked = provider.asked.lock().unwrap().clone();
    let until = chrono::DateTime::parse_from_rfc3339(&asked[0].1).unwrap();
    let hours = (until.with_timezone(&Utc) - Utc::now()).num_hours();
    assert!((16 * 24..=17 * 24).contains(&hours), "{asked:?}");
    assert_eq!(uids(&pool).await, vec!["ews-1/"]);
}
