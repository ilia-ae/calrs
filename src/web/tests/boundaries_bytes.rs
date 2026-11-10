//! ilia fork: the bytes a CalDAV sync transfers, before (upstream's flow:
//! whole objects from −90 days on, inline-data delta) and after (the
//! windowed, boundaries-only sync), against the Kerio-style mock server on a
//! realistic, seeded calendar. Numbers: ilia-booking docs/operations.md,
//! «Производительность».
//!
//! `cargo test --locked perf_caldav_bytes -- --ignored --nocapture --test-threads 1`

use super::*;

/// A tiny deterministic generator (no new crate).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> i64 {
        (self.next() % n) as i64
    }
}

const MOSCOW: &str = "BEGIN:VTIMEZONE\nTZID:Europe/Moscow\nX-LIC-LOCATION:Europe/Moscow\nBEGIN:STANDARD\nDTSTART:19700101T000000\nTZOFFSETFROM:+0300\nTZOFFSETTO:+0300\nTZNAME:MSK\nEND:STANDARD\nEND:VTIMEZONE\n";

/// What a real event carries besides its times: title, ~300 B of
/// description, place, organizer, five attendees, an alarm, X- properties.
fn content(rng: &mut Lcg, n: usize) -> String {
    let words = [
        "quarterly",
        "review",
        "budget",
        "roadmap",
        "client",
        "contract",
        "design",
        "sync",
        "hiring",
        "launch",
        "metrics",
        "partner",
        "legal",
        "offsite",
        "support",
    ];
    let mut description = String::new();
    while description.len() < 300 {
        description.push_str(words[rng.below(words.len() as u64) as usize]);
        description.push(' ');
    }
    let mut s = format!(
        "SUMMARY:{} {} #{n}\nDESCRIPTION:{}\nLOCATION:Meeting room {}, 4th floor\nORGANIZER;CN=Ilia Arestov:mailto:owner@ilia.ae\n",
        words[rng.below(15) as usize],
        words[rng.below(15) as usize],
        description.trim_end(),
        rng.below(12)
    );
    for a in 0..5 {
        s.push_str(&format!(
            "ATTENDEE;CN=Attendee {a};ROLE=REQ-PARTICIPANT;PARTSTAT=ACCEPTED;RSVP=TRUE:mailto:attendee{a}.{n}@example.com\n"
        ));
    }
    s.push_str(&format!(
        "DTSTAMP:20261101T000000Z\nCREATED:20260101T000000Z\nLAST-MODIFIED:20261101T000000Z\nSEQUENCE:{}\nCLASS:PUBLIC\nPRIORITY:5\nX-MICROSOFT-CDO-BUSYSTATUS:BUSY\nX-KERIO-PRIVATE:0\nBEGIN:VALARM\nACTION:DISPLAY\nTRIGGER:-PT15M\nDESCRIPTION:Reminder\nEND:VALARM\n",
        rng.below(4)
    ));
    s
}

fn stamp(date: NaiveDate, hour: i64, minute: i64) -> String {
    format!("{}T{:02}{:02}00", date.format("%Y%m%d"), hour, minute)
}

/// 1500 one-off events over 18 months back and 6 ahead, and 60 weekly
/// masters (10 of them ended) with 10 EXDATEs and 5 moved instances each;
/// a VTIMEZONE in every resource, Kerio-style names.
fn calendar() -> Vec<(String, String)> {
    let mut rng = Lcg(20261110);
    let today = Utc::now().date_naive();
    let mut out = Vec::new();
    for n in 0..1500 {
        let day = today + Duration::days(-548 + rng.below(730));
        let hour = 8 + rng.below(10);
        let minutes = [30, 60, 90][rng.below(3) as usize];
        let start = day.and_hms_opt(hour as u32, 0, 0).unwrap();
        let end = start + Duration::minutes(minutes);
        let body = content(&mut rng, n);
        out.push((
            format!("{n:08x}-5d0e-7c1f-4f7a-9b2e0c9d1f00-00000001.eml"),
            format!(
                "BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//Kerio Technologies//Kerio Connect//EN\n{MOSCOW}BEGIN:VEVENT\nUID:single-{n}@ilia.ae\nDTSTART;TZID=Europe/Moscow:{}\nDTEND;TZID=Europe/Moscow:{}\n{body}END:VEVENT\nEND:VCALENDAR\n",
                start.format("%Y%m%dT%H%M%S"),
                end.format("%Y%m%dT%H%M%S")
            ),
        ));
    }
    for m in 0..60 {
        let first = today + Duration::days(-548 + rng.below(540));
        let hour = 9 + rng.below(8);
        let until = if m < 10 {
            format!(
                ";UNTIL={}T000000Z",
                (today - Duration::days(30 + rng.below(200))).format("%Y%m%d")
            )
        } else {
            String::new()
        };
        let weeks = ((today - first).num_days() / 7).max(12);
        let mut master = format!(
            "BEGIN:VEVENT\nUID:weekly-{m}@ilia.ae\nDTSTART;TZID=Europe/Moscow:{}\nDTEND;TZID=Europe/Moscow:{}\nRRULE:FREQ=WEEKLY{until}\n",
            stamp(first, hour, 0),
            stamp(first, hour + 1, 0)
        );
        for _ in 0..10 {
            let k = 1 + rng.below(weeks as u64 + 2);
            master.push_str(&format!(
                "EXDATE;TZID=Europe/Moscow:{}\n",
                stamp(first + Duration::weeks(k), hour, 0)
            ));
        }
        master.push_str(&content(&mut rng, 10_000 + m));
        master.push_str("END:VEVENT\n");
        for o in 0..5 {
            // Spread from the series' start to a few weeks ahead.
            let k = (weeks + 4) * (o + 1) / 5;
            let original = first + Duration::weeks(k);
            master.push_str(&format!(
                "BEGIN:VEVENT\nUID:weekly-{m}@ilia.ae\nRECURRENCE-ID;TZID=Europe/Moscow:{}\nDTSTART;TZID=Europe/Moscow:{}\nDTEND;TZID=Europe/Moscow:{}\n{}END:VEVENT\n",
                stamp(original, hour, 0),
                stamp(original, hour, 30),
                stamp(original, hour + 1, 30),
                content(&mut rng, 20_000 + m * 5 + o as usize)
            ));
        }
        out.push((
            format!("{m:08x}-aaaa-7c1f-4f7a-9b2e0c9d1f00-00000001.eml"),
            format!(
                "BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//Kerio Technologies//Kerio Connect//EN\n{MOSCOW}{master}END:VCALENDAR\n"
            ),
        ));
    }
    out
}

/// Bytes and REPORTs since the log was cleared.
fn traffic(dav: &MockDav) -> (usize, usize) {
    let log = dav.log();
    (
        log.iter().map(|l| l.bytes).sum(),
        log.iter().filter(|l| l.method == "REPORT").count(),
    )
}

struct Run {
    first: (usize, usize),
    resync: (usize, usize),
    delta: (usize, usize),
    busy: Vec<(NaiveDateTime, NaiveDateTime)>,
    rows: i64,
}

async fn run(windowed: bool, honour_partial: bool) -> Run {
    run_with(
        windowed,
        Behaviour {
            honour_partial,
            ..Default::default()
        },
    )
    .await
}

async fn run_with(windowed: bool, behaviour: Behaviour) -> Run {
    let _cap = windowed.then(|| test_limit(14));
    let resources = calendar();
    let dav = MockDav::start(resources.clone(), behaviour).await;
    let (pool, user_id, _, source_id) = seeded(&dav).await;

    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    let first = traffic(&dav);

    // (2) The daily resync, nothing changed on the server.
    force_full(&pool).await;
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    let resync = traffic(&dav);

    // (3) A delta: 5 events moved inside the horizon, 1 far ahead, 1 deleted.
    let mut rng = Lcg(7);
    let today = Utc::now().date_naive();
    for n in 0..5 {
        let (name, data) = &resources[n * 100];
        let day = today + Duration::days(1 + rng.below(12));
        let moved = data
            .lines()
            .map(|l| {
                if l.starts_with("DTSTART;") {
                    format!("DTSTART;TZID=Europe/Moscow:{}", stamp(day, 10, 0))
                } else if l.starts_with("DTEND;") {
                    format!("DTEND;TZID=Europe/Moscow:{}", stamp(day, 11, 0))
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        dav.put(name, &moved);
    }
    let (far_name, far_data) = &resources[777];
    dav.put(
        far_name,
        &far_data
            .lines()
            .map(|l| {
                if l.starts_with("DTSTART;") {
                    format!(
                        "DTSTART;TZID=Europe/Moscow:{}",
                        stamp(today + Duration::days(120), 10, 0)
                    )
                } else if l.starts_with("DTEND;") {
                    format!(
                        "DTEND;TZID=Europe/Moscow:{}",
                        stamp(today + Duration::days(120), 11, 0)
                    )
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    );
    dav.delete(&resources[900].0);
    dav.clear_log();
    sync(&pool, &dav, &source_id).await;
    let delta = traffic(&dav);

    let busy = kerio_busy(&pool, &user_id).await;
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await
        .unwrap();
    Run {
        first,
        resync,
        delta,
        busy,
        rows,
    }
}

fn kb(bytes: usize) -> String {
    format!("{:.1} KB", bytes as f64 / 1024.0)
}

#[tokio::test]
#[ignore]
async fn perf_caldav_bytes() {
    let before = run(false, false).await;
    let after = run(true, true).await;
    let ignored = run(true, false).await;
    // A server whose PROPFIND carries no sync token (only its REPORTs do).
    let report_token = Behaviour {
        token_in_report_only: true,
        ..Default::default()
    };
    let before_rt = run_with(
        false,
        Behaviour {
            honour_partial: false,
            ..report_token.clone()
        },
    )
    .await;
    let after_rt = run_with(true, report_token).await;
    println!("\nCalDAV sync bytes (1500 one-off events, 60 weekly masters, Kerio-style mock)");
    println!(
        "{:<44} {:>14} {:>14} {:>16}",
        "scenario", "before", "after", "after, partial ignored"
    );
    for (label, b, a, i) in [
        (
            "(1) first full sync",
            before.first,
            after.first,
            ignored.first,
        ),
        (
            "(2) daily resync, nothing changed",
            before.resync,
            after.resync,
            ignored.resync,
        ),
        (
            "(3) delta: 5 moved, 1 far, 1 deleted",
            before.delta,
            after.delta,
            ignored.delta,
        ),
    ] {
        println!(
            "{label:<44} {:>9} ({} r) {:>9} ({} r) {:>11} ({} r)",
            kb(b.0),
            b.1,
            kb(a.0),
            a.1,
            kb(i.0),
            i.1
        );
    }
    println!("the same, sync token only in REPORT answers (initial token by sync-collection):");
    for (label, b, a) in [
        ("(1) first full sync", before_rt.first, after_rt.first),
        (
            "(2) daily resync, nothing changed",
            before_rt.resync,
            after_rt.resync,
        ),
        (
            "(3) delta: 5 moved, 1 far, 1 deleted",
            before_rt.delta,
            after_rt.delta,
        ),
    ] {
        println!(
            "{label:<44} {:>9} ({} r) {:>9} ({} r)",
            kb(b.0),
            b.1,
            kb(a.0),
            a.1
        );
    }
    println!(
        "cached rows: before {}, after {}, after with partial ignored {}",
        before.rows, after.rows, ignored.rows
    );
    assert_eq!(after_rt.busy, after.busy);
    assert!(after_rt.first.0 * 5 < before_rt.first.0);
    assert!(after_rt.resync.0 * 5 < before_rt.resync.0);
    // (4) The same busy times inside the horizon, whichever way.
    assert_eq!(before.busy, after.busy);
    assert_eq!(after.busy, ignored.busy);
    println!(
        "busy intervals in the next 14 days: {} (equal in all three)",
        after.busy.len()
    );
    assert!(
        after.first.0 * 5 < before.first.0,
        "the first sync is at least 5x smaller"
    );
    assert!(after.resync.0 * 5 < before.resync.0);
    assert!(after.delta.0 <= before.delta.0);
}
