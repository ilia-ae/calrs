//! ilia fork: busy-only calendar storage. A calendar is read for one thing,
//! when the host is busy, so the event cache keeps only what availability
//! needs: the times, the all-day flag, the timezone, the RRULE, its EXDATEs
//! and the RECURRENCE-IDs of its moved instances, STATUS and TRANSP, the UID.
//! Titles, descriptions, locations, attendees and the raw iCal are not kept.
//!
//! On by default; `CALRS_BUSY_ONLY=0` (or `false`, `no`) restores upstream's
//! full cache, which the next daily full resync fills again. Anything else
//! keeps it on (fail private).
//!
//! `raw_ical` is read by one function only, [`crate::rrule::extract_exdates`],
//! and only for rows with an RRULE. [`reduce`] keeps exactly the lines that
//! function reads, in a shape it reads the same way, so the busy times are
//! unchanged (tests below and in `src/web/tests/busy_only.rs`).

use std::cell::Cell;
use std::sync::OnceLock;

use sqlx::SqlitePool;

/// First line of a reduced `raw_ical`, so the startup scrub can tell a row
/// it already reduced.
pub const MARKER: &str = "X-CALRS-BUSY:1";

/// Rows reduced per write transaction at startup.
const BATCH: i64 = 500;

thread_local! {
    /// Tests only: the mode for this thread. `#[tokio::test]` runs on one
    /// thread, so the override covers the whole test without a global.
    static OVERRIDE: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Parses `CALRS_BUSY_ONLY`. `Err` carries nothing: the caller warns.
fn parse(value: Option<&str>) -> Result<bool, ()> {
    match value.map(|v| v.trim().to_ascii_lowercase()) {
        None => Ok(true),
        Some(v) => match v.as_str() {
            "" | "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => Err(()),
        },
    }
}

/// Whether the event cache is busy-only. Read once per process.
pub fn enabled() -> bool {
    if let Some(mode) = OVERRIDE.with(Cell::get) {
        return mode;
    }
    static MODE: OnceLock<bool> = OnceLock::new();
    *MODE.get_or_init(|| {
        let raw = std::env::var("CALRS_BUSY_ONLY").ok();
        parse(raw.as_deref()).unwrap_or_else(|()| {
            tracing::warn!(
                "CALRS_BUSY_ONLY is not 0/1/true/false/yes/no; busy-only storage stays on"
            );
            true
        })
    })
}

/// Tests only: sets this thread's mode until the guard drops.
#[cfg(test)]
pub fn test_mode(on: bool) -> ModeGuard {
    let old = OVERRIDE.with(|c| c.replace(Some(on)));
    ModeGuard(old)
}

#[cfg(test)]
pub struct ModeGuard(Option<bool>);

#[cfg(test)]
impl Drop for ModeGuard {
    fn drop(&mut self) {
        OVERRIDE.with(|c| c.set(self.0));
    }
}

/// A content field (SUMMARY, DESCRIPTION, LOCATION): dropped when busy-only.
pub fn keep(value: Option<String>) -> Option<String> {
    if enabled() {
        None
    } else {
        value
    }
}

/// The lines of `raw` that [`crate::rrule::extract_exdates`] reads, or `None`
/// when there are none (a NULL `raw_ical` gives the same empty list). It
/// scans exactly as that function does: the EXDATE lines of the region from
/// the first `BEGIN:VEVENT` to the first `END:VEVENT` after it, and every
/// RECURRENCE-ID line of the whole text, after unfolding. The output keeps
/// them verbatim (parameters included) as
/// `X-CALRS-BUSY:1`, `BEGIN:VEVENT`, the EXDATEs, `END:VEVENT`, the
/// RECURRENCE-IDs, and is a fixed point: `reduce(reduce(x)) == reduce(x)`.
pub fn reduce(raw: &str) -> Option<String> {
    let unfolded = crate::utils::unfold_ical(raw);
    let text: &str = &unfolded;
    let named = |line: &str, name: &str| {
        crate::utils::parse_content_line(line).is_some_and(|p| p.name.eq_ignore_ascii_case(name))
    };

    let mut exdates: Vec<&str> = Vec::new();
    if let Some(start) = text.find("BEGIN:VEVENT") {
        let end = text[start..]
            .find("END:VEVENT")
            .map(|i| start + i)
            .unwrap_or(text.len());
        exdates.extend(
            text[start..end]
                .lines()
                .map(str::trim)
                .filter(|l| named(l, "EXDATE")),
        );
    }
    let recurrence_ids: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| named(l, "RECURRENCE-ID"))
        .collect();
    if exdates.is_empty() && recurrence_ids.is_empty() {
        return None;
    }

    let mut out = String::with_capacity(
        64 + exdates
            .iter()
            .chain(&recurrence_ids)
            .map(|l| l.len() + 2)
            .sum::<usize>(),
    );
    out.push_str(MARKER);
    out.push_str("\r\nBEGIN:VEVENT\r\n");
    for line in exdates {
        out.push_str(line);
        out.push_str("\r\n");
    }
    out.push_str("END:VEVENT\r\n");
    for line in recurrence_ids {
        out.push_str(line);
        out.push_str("\r\n");
    }
    Some(out)
}

/// The `raw_ical` to store for the rows of one fetched resource (or one feed
/// VEVENT). Busy-only: nothing for a row without RRULE (no reader looks), the
/// [`reduce`]d text otherwise, computed once per resource. Off: the resource
/// as fetched, as upstream stores it.
pub struct StoredIcal<'a> {
    raw: &'a str,
    on: bool,
    reduced: OnceLock<Option<String>>,
}

impl<'a> StoredIcal<'a> {
    pub fn new(raw: &'a str) -> Self {
        Self {
            raw,
            on: enabled(),
            reduced: OnceLock::new(),
        }
    }

    pub fn for_row(&self, rrule: Option<&str>) -> Option<&str> {
        if !self.on {
            return Some(self.raw);
        }
        if rrule.is_none_or(|r| r.trim().is_empty()) {
            return None;
        }
        self.reduced.get_or_init(|| reduce(self.raw)).as_deref()
    }
}

/// Startup scrub (from `db::migrate`, when busy-only): drops content a run
/// with `CALRS_BUSY_ONLY=0` cached, and reduces recurring rows not reduced
/// yet, in batches. Idempotent; once done every statement matches nothing.
/// Returns the number of rows changed.
pub async fn scrub_stored(pool: &SqlitePool) -> anyhow::Result<u64> {
    let mut changed = 0u64;
    for sql in [
        "UPDATE events SET summary = NULL, description = NULL, location = NULL
          WHERE summary IS NOT NULL OR description IS NOT NULL OR location IS NOT NULL",
        "UPDATE events SET raw_ical = NULL
          WHERE raw_ical IS NOT NULL AND (rrule IS NULL OR rrule = '')",
        "UPDATE resource_events SET summary = NULL WHERE summary IS NOT NULL",
        "UPDATE resource_events SET raw_ical = NULL
          WHERE raw_ical IS NOT NULL AND (rrule IS NULL OR rrule = '')",
    ] {
        changed += sqlx::query(sql).execute(pool).await?.rows_affected();
    }
    for table in ["events", "resource_events"] {
        loop {
            let rows: Vec<(String, String)> = sqlx::query_as(&format!(
                "SELECT id, raw_ical FROM {table}
                  WHERE rrule IS NOT NULL AND rrule != '' AND raw_ical IS NOT NULL
                    AND substr(raw_ical, 1, {len}) != '{MARKER}'
                  LIMIT {BATCH}",
                len = MARKER.len()
            ))
            .fetch_all(pool)
            .await?;
            if rows.is_empty() {
                break;
            }
            let mut tx = pool.begin().await?;
            for (id, raw) in &rows {
                sqlx::query(&format!("UPDATE {table} SET raw_ical = ? WHERE id = ?"))
                    .bind(reduce(raw))
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await?;
            changed += rows.len() as u64;
        }
    }
    if changed > 0 {
        tracing::info!(rows = changed, "busy-only: calendar event content removed");
    }
    Ok(changed)
}

/// After a scrub that changed rows: rebuild the file (`VACUUM`), so the
/// removed content is not left readable in free pages or in freed cells
/// (where every later backup would copy it), then empty the WAL. The pool
/// also runs with `secure_delete` on (`db::connect`). Once per scrub that
/// changed anything; a no-op scrub costs nothing.
pub async fn wipe_free_pages(pool: &SqlitePool) -> anyhow::Result<()> {
    let started = std::time::Instant::now();
    sqlx::raw_sql("VACUUM").execute(pool).await?;
    sqlx::raw_sql("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(pool)
        .await?;
    tracing::info!(
        ms = started.elapsed().as_millis() as u64,
        "busy-only: database file rebuilt, removed content wiped"
    );
    Ok(())
}

/// Tests only: a BlueMind-style resource with content to drop: a VTIMEZONE
/// under a custom TZID, a weekly master with three EXDATEs, two moved
/// instances and a cancelled one.
#[cfg(test)]
pub(crate) const BLUEMIND_FIXTURE: &str = "BEGIN:VCALENDAR\r\nPRODID:-//BlueMind//BlueMind Calendar//FR\r\nBEGIN:VTIMEZONE\r\nTZID:Customized Time Zone\r\nBEGIN:STANDARD\r\nDTSTART:16010101T000000\r\nTZOFFSETFROM:+0400\r\nTZOFFSETTO:+0400\r\nEND:STANDARD\r\nEND:VTIMEZONE\r\nBEGIN:VEVENT\r\nUID:weekly-1\r\nDTSTART;TZID=Customized Time Zone:20300107T100000\r\nDTEND;TZID=Customized Time Zone:20300107T110000\r\nRRULE:FREQ=WEEKLY;BYDAY=MO\r\nEXDATE;TZID=Customized Time Zone:20300114T100000\r\nEXDATE;TZID=Customized Time Zone:20300121T100000,20300128T100000\r\nSUMMARY:Secret plans\r\nDESCRIPTION:Quarterly numbers\\, do not share\r\nLOCATION:Board room 7\r\nORGANIZER;CN=Boss:mailto:boss@corp.example\r\nATTENDEE;CN=Alice:mailto:alice@corp.example\r\nBEGIN:VALARM\r\nTRIGGER:-PT15M\r\nACTION:DISPLAY\r\nDESCRIPTION:Reminder\r\nEND:VALARM\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:weekly-1\r\nRECURRENCE-ID;TZID=Customized Time Zone:20300204T100000\r\nDTSTART;TZID=Customized Time Zone:20300204T150000\r\nDTEND;TZID=Customized Time Zone:20300204T160000\r\nSUMMARY:Secret plans (moved)\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:weekly-1\r\nRECURRENCE-ID;TZID=Customized Time Zone:20300211T100000\r\nDTSTART;TZID=Customized Time Zone:20300212T090000\r\nDTEND;TZID=Customized Time Zone:20300212T100000\r\nSUMMARY:Secret plans (moved again)\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:weekly-1\r\nRECURRENCE-ID;TZID=Customized Time Zone:20300218T100000\r\nDTSTART;TZID=Customized Time Zone:20300218T100000\r\nDTEND;TZID=Customized Time Zone:20300218T110000\r\nSTATUS:CANCELLED\r\nSUMMARY:Secret plans (cancelled)\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rrule::extract_exdates;

    #[test]
    fn parse_values() {
        assert_eq!(parse(None), Ok(true));
        for on in ["1", "true", "YES", " on ", ""] {
            assert_eq!(parse(Some(on)), Ok(true), "{on}");
        }
        for off in ["0", "false", "No", "off"] {
            assert_eq!(parse(Some(off)), Ok(false), "{off}");
        }
        assert_eq!(parse(Some("maybe")), Err(()));
    }

    #[test]
    fn keep_follows_the_mode() {
        let _on = test_mode(true);
        assert_eq!(keep(Some("Board meeting".into())), None);
        {
            let _off = test_mode(false);
            assert_eq!(
                keep(Some("Board meeting".into())).as_deref(),
                Some("Board meeting")
            );
        }
        assert_eq!(keep(Some("x".into())), None);
    }

    /// Fixtures from `src/rrule.rs`, `src/commands/sync.rs` and
    /// `src/caldav/mod.rs` tests, and edge cases of the scan.
    fn corpus() -> Vec<&'static str> {
        vec![
            // rrule.rs
            "BEGIN:VEVENT\nDTSTART:20260302T100000\nRRULE:FREQ=WEEKLY;BYDAY=MO\nEXDATE:20260309T100000\nEXDATE:20260316T100000\nEND:VEVENT",
            "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:abc\nDTSTART:20260302T100000\nDTEND:20260302T110000\nRRULE:FREQ=WEEKLY;BYDAY=MO\nEND:VEVENT\nBEGIN:VEVENT\nUID:abc\nRECURRENCE-ID:20260309T100000\nDTSTART:20260309T140000\nDTEND:20260309T150000\nEND:VEVENT\nEND:VCALENDAR",
            "BEGIN:VEVENT\nDTSTART:20260302T100000\nRRULE:FREQ=WEEKLY\nEXDATE;TZID=Europe/Paris:20260309T100000\nEND:VEVENT",
            "BEGIN:VEVENT\nDTSTART:20260302T100000\nRRULE:FREQ=WEEKLY\nEXDATE;TZID=\"(UTC+01:00) Paris\":20260309T100000\nEND:VEVENT\nBEGIN:VEVENT\nRECURRENCE-ID;TZID=\"(UTC+01:00) Paris\":20260316T100000\nEND:VEVENT",
            "BEGIN:VEVENT\nDTSTART;TZID=Europe/Paris:20260302T003000\nRRULE:FREQ=WEEKLY\nEXDATE:20260308T233000Z\nEND:VEVENT",
            "BEGIN:VEVENT\nDTSTART;TZID=Europe/Paris:20260302T160000\nRRULE:FREQ=WEEKLY\nEND:VEVENT\nBEGIN:VEVENT\nRECURRENCE-ID;TZID=\"Eastern Standard Time\":20260309T110000\nEND:VEVENT",
            "BEGIN:VEVENT\nDTSTART;TZID=Customized Time Zone:20260302T100000\nRRULE:FREQ=WEEKLY\nEXDATE;TZID=Customized Time Zone:20260309T100000\nEND:VEVENT",
            "BEGIN:VEVENT\nDTSTART:20260302T100000\nRRULE:FREQ=WEEKLY\nEXDATE:20260309T100000Z\nEND:VEVENT",
            "BEGIN:VEVENT\nEXDATE:20260309T100000,20260316T100000,20260323T100000\nEND:VEVENT",
            "BEGIN:VEVENT\r\nEXDATE:20260309T100000,2026031\r\n 6T100000,20260323T100000\r\nEND:VEVENT",
            // A BlueMind-style resource: VTIMEZONE, master, moved and cancelled instances.
            BLUEMIND_FIXTURE,
            // Overrides first in the resource: the master's EXDATEs are not read.
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:o\r\nRECURRENCE-ID:20260309T100000\r\nDTSTART:20260309T120000\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:o\r\nRRULE:FREQ=DAILY\r\nEXDATE:20260310T100000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
            // Several EXDATE lines, a VALUE=DATE one, RANGE=THISANDFUTURE.
            "BEGIN:VEVENT\r\nRRULE:FREQ=DAILY\r\nEXDATE:20260309T100000\r\nEXDATE;VALUE=DATE:20260310\r\nEXDATE;TZID=Asia/Dubai:20260311T100000,20260312T100000\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nRECURRENCE-ID;RANGE=THISANDFUTURE;TZID=Europe/Paris:20260320T100000\r\nEND:VEVENT\r\n",
            // A VALARM inside the master, lowercase names, LF only.
            "BEGIN:VEVENT\nrrule:FREQ=WEEKLY\nexdate:20260309T100000\nBEGIN:VALARM\nTRIGGER:-PT15M\nEND:VALARM\nEXDATE:20260316T100000\nEND:VEVENT\nBEGIN:VEVENT\nrecurrence-id:20260323T100000\nEND:VEVENT\n",
            // BEGIN:VEVENT / END:VEVENT inside an escaped DESCRIPTION.
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDESCRIPTION:see END:VEVENT and BEGIN:VEVENT here\r\nEXDATE:20260309T100000\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nRECURRENCE-ID:20260316T100000\r\nEND:VEVENT\r\n",
            "X-FOO:BEGIN:VEVENT\r\nEXDATE:20260309T100000\r\nEND:VEVENT\r\n",
            // An EXDATE cut by an END:VEVENT on the same line.
            "BEGIN:VEVENT\r\nEXDATE:20260309T100000,20260316T100000END:VEVENT\r\n",
            // No VEVENT at all; EXDATE only in the second VEVENT.
            "RECURRENCE-ID:20260309T100000\r\nEXDATE:20260316T100000\r\n",
            "BEGIN:VEVENT\r\nRRULE:FREQ=DAILY\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nEXDATE:20260316T100000\r\nEND:VEVENT\r\n",
            // Unparseable values are kept and stay unparseable.
            "BEGIN:VEVENT\r\nEXDATE:garbage\r\nEXDATE\r\nEND:VEVENT\r\n",
            // Nothing to keep.
            "BEGIN:VEVENT\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Board meeting\r\nEND:VEVENT\r\n",
            "",
        ]
    }

    #[test]
    fn reduce_is_equivalent_for_extract_exdates() {
        for raw in corpus() {
            let reduced = reduce(raw);
            for tz in [
                None,
                Some("Europe/Paris"),
                Some("Asia/Dubai"),
                Some("America/New_York"),
            ] {
                assert_eq!(
                    extract_exdates(reduced.as_deref().unwrap_or(""), tz),
                    extract_exdates(raw, tz),
                    "{raw:?} -> {reduced:?} ({tz:?})"
                );
            }
            if let Some(r) = &reduced {
                assert!(r.starts_with(MARKER));
                assert_eq!(
                    reduce(r).as_deref(),
                    Some(r.as_str()),
                    "fixed point: {raw:?}"
                );
            }
        }
    }

    #[test]
    fn reduce_drops_the_content() {
        let reduced = reduce(BLUEMIND_FIXTURE).unwrap();
        for word in [
            "Secret",
            "Quarterly",
            "Board",
            "boss@",
            "alice@",
            "Customized Time Zone:20300107T1",
        ] {
            assert!(!reduced.contains(word), "{word} in {reduced}");
        }
        assert_eq!(reduced.lines().count(), 8, "{reduced}");
        assert!(extract_exdates(&reduced, Some("Asia/Dubai")).len() == 6);
        assert_eq!(
            reduce("BEGIN:VEVENT\r\nRRULE:FREQ=DAILY\r\nEND:VEVENT"),
            None
        );
    }

    #[test]
    fn stored_ical_per_mode() {
        {
            let _on = test_mode(true);
            let s = StoredIcal::new(BLUEMIND_FIXTURE);
            assert_eq!(s.for_row(None), None);
            assert_eq!(s.for_row(Some("")), None);
            assert!(s.for_row(Some("FREQ=WEEKLY")).unwrap().starts_with(MARKER));
        }
        let _off = test_mode(false);
        let s = StoredIcal::new(BLUEMIND_FIXTURE);
        assert_eq!(s.for_row(None), Some(BLUEMIND_FIXTURE));
        assert_eq!(s.for_row(Some("FREQ=WEEKLY")), Some(BLUEMIND_FIXTURE));
    }
}
