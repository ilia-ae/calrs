//! ilia fork: the windowed calendar sync (owner's decisions 2026-11-10,
//! "only the boundaries" and "no further than 14 days"; ilia-booking
//! docs/decisions.md). `sync.rs` calls in here when
//! [`crate::horizon::sync_window`] gives a window, which is always in a real
//! build; without one (tests by default) it keeps upstream's flow.
//!
//! - **Full fetch** ([`full_fetch`]): a calendar that already has etags
//!   stored is listed by etag for the window and only changed or new
//!   resources are fetched (`calendar-multiget`); otherwise one
//!   `calendar-query` for the window. Both ask for the boundaries only
//!   (`caldav::partial`). A refused listing or multiget falls back to the
//!   query, a refused time range to the unfiltered REPORT. A resource the
//!   multiget leaves out (neither data nor gone) keeps its cached rows.
//!   Then the moved instances stored on their own whose master is cached
//!   but which moved out of the window ([`moved_out`]).
//! - **Delta** ([`delta`]): `sync-collection` for etags only, then the same
//!   multiget for the changed hrefs whose etag differs; a refusal falls back
//!   to upstream's inline-data `sync-collection`, and a change the multiget
//!   could not fetch to a full fetch (the token is not advanced past it).
//! - **Prune** ([`prune`]): plain deletes, no booking cancelled, of cached
//!   rows outside the window, after every upsert batch. Stored times are
//!   compared in compact form (`replace` of `-` and `:`), since a feed may
//!   write `2026-11-20T10:00:00`.

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use sqlx::SqlitePool;

use crate::caldav::{CaldavClient, RawEvent, SyncResult};
use crate::horizon::SyncWindow;

/// Rows deleted per statement by the recurring-row prune.
const PRUNE_BATCH: usize = 500;

/// `start_at` and `end_at` in compact form (`YYYYMMDD[THHMMSS]`), as the
/// window's prefixes are: a dashed `2026-11-20…` would sort below
/// `20261108` byte-wise.
pub(crate) const START_COMPACT: &str = "replace(replace(start_at, '-', ''), ':', '')";
pub(crate) const END_COMPACT: &str = "replace(replace(end_at, '-', ''), ':', '')";

/// A recurring master with the row's UID is cached in its calendar.
const MASTER_CACHED: &str = "EXISTS (SELECT 1 FROM events m
                                 WHERE m.calendar_id = events.calendar_id
                                   AND m.uid = events.uid
                                   AND m.rrule IS NOT NULL AND m.rrule != '')";

/// What a windowed full fetch got.
pub(crate) struct Fetched {
    /// Resources fetched (changed, new, or all of them).
    pub events: Vec<RawEvent>,
    /// `(uid, recurrence_id)` of cached rows whose resource is listed with an
    /// unchanged etag: present on the server, not refetched.
    pub unchanged: Vec<(String, String)>,
    /// How many resources the server listed for the window.
    pub listed: usize,
}

/// The `href → etag` of this calendar's cached resources.
async fn stored_etags(pool: &SqlitePool, cal_id: &str) -> HashMap<String, String> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT href, MAX(etag) FROM events
         WHERE calendar_id = ? AND href IS NOT NULL AND etag IS NOT NULL
         GROUP BY href",
    )
    .bind(cal_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .collect()
}

/// `(uid, recurrence_id)` of the cached rows of these resources.
async fn rows_of(
    pool: &SqlitePool,
    cal_id: &str,
    hrefs: &HashSet<String>,
) -> Vec<(String, String)> {
    if hrefs.is_empty() {
        return Vec::new();
    }
    sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT uid, recurrence_id, href FROM events WHERE calendar_id = ? AND href IS NOT NULL",
    )
    .bind(cal_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .filter(|(_, _, href)| href.as_ref().is_some_and(|h| hrefs.contains(h)))
    .map(|(uid, rid, _)| (uid, rid.unwrap_or_default()))
    .collect()
}

/// The hrefs to fetch from a listing: new ones and those whose etag changed
/// (a missing etag counts as changed).
fn changed_hrefs(
    listed: &[crate::caldav::partial::Listed],
    stored: &HashMap<String, String>,
) -> (Vec<String>, HashSet<String>) {
    let mut fetch = Vec::new();
    let mut unchanged = HashSet::new();
    for item in listed {
        match (&item.etag, stored.get(&item.href)) {
            (Some(new), Some(old)) if new == old => {
                unchanged.insert(item.href.clone());
            }
            _ => fetch.push(item.href.clone()),
        }
    }
    (fetch, unchanged)
}

/// A full fetch of the window (see the module docs).
pub(crate) async fn full_fetch(
    pool: &SqlitePool,
    client: &CaldavClient,
    cal_id: &str,
    cal_href: &str,
    window: &SyncWindow,
) -> Result<Fetched> {
    let (since, until) = (window.since_utc(), window.until_utc());
    let stored = stored_etags(pool, cal_id).await;
    if !stored.is_empty() {
        if let Some(listed) = client.list_window(cal_href, &since, &until).await? {
            let (fetch, mut unchanged) = changed_hrefs(&listed, &stored);
            let got = if fetch.is_empty() {
                Some(crate::caldav::partial::Multi::default())
            } else {
                client.multiget(cal_href, &fetch, &since).await?
            };
            if let Some(got) = got {
                tracing::debug!(
                    listed = listed.len(),
                    fetched = fetch.len(),
                    unknown = got.unknown.len(),
                    "windowed resync by etag"
                );
                // Not fetched, not gone: kept as they are.
                unchanged.extend(got.unknown);
                let mut fetched = Fetched {
                    events: got.events,
                    unchanged: Vec::new(),
                    listed: listed.len(),
                };
                moved_out(
                    pool,
                    client,
                    cal_id,
                    cal_href,
                    window,
                    &stored,
                    &mut fetched,
                    &mut unchanged,
                )
                .await?;
                fetched.unchanged = rows_of(pool, cal_id, &unchanged).await;
                return Ok(fetched);
            }
        }
    }
    let events = match client.query_window(cal_href, &since, &until).await? {
        Some(events) => events,
        // The time range itself was refused: upstream's unfiltered REPORT,
        // whose answer is complete.
        None => client.fetch_events(cal_href).await?,
    };
    let mut fetched = Fetched {
        listed: events.len(),
        events,
        unchanged: Vec::new(),
    };
    let mut unchanged = HashSet::new();
    moved_out(
        pool,
        client,
        cal_id,
        cal_href,
        window,
        &stored,
        &mut fetched,
        &mut unchanged,
    )
    .await?;
    fetched.unchanged = rows_of(pool, cal_id, &unchanged).await;
    Ok(fetched)
}

/// The UIDs of the recurring masters a window fetch may expand: those it
/// got, and those cached (an unchanged one is not refetched).
async fn master_uids(pool: &SqlitePool, cal_id: &str, events: &[RawEvent]) -> HashSet<String> {
    let mut uids: HashSet<String> = sqlx::query_scalar(
        "SELECT DISTINCT uid FROM events WHERE calendar_id = ? AND rrule IS NOT NULL AND rrule != ''",
    )
    .bind(cal_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .collect();
    for raw in events {
        for vevent in crate::utils::split_vevents(&raw.ical_data) {
            let fields = crate::utils::unfold_ical(&vevent);
            if crate::utils::extract_vevent_field(&fields, "RRULE").is_some_and(|r| !r.is_empty()) {
                if let Some(uid) = crate::utils::extract_vevent_field(&fields, "UID") {
                    uids.insert(uid);
                }
            }
        }
    }
    uids
}

/// Whether this calendar keeps a moved instance in a resource without its
/// series: a cached one, or one just fetched.
async fn stores_overrides_apart(pool: &SqlitePool, cal_id: &str, events: &[RawEvent]) -> bool {
    let fetched = events.iter().any(|raw| {
        let vevents = crate::utils::split_vevents(&raw.ical_data);
        !vevents.is_empty()
            && vevents.iter().all(|v| {
                let fields = crate::utils::unfold_ical(v);
                crate::utils::extract_vevent_field(&fields, "RECURRENCE-ID").is_some()
                    && crate::utils::extract_vevent_field(&fields, "RRULE").is_none()
            })
    });
    fetched
        || sqlx::query_scalar::<_, i64>(
            "SELECT EXISTS (SELECT 1 FROM events o
                             WHERE o.calendar_id = ?1 AND o.href IS NOT NULL
                               AND o.recurrence_id IS NOT NULL AND o.recurrence_id != ''
                               AND NOT EXISTS (SELECT 1 FROM events m
                                                WHERE m.calendar_id = o.calendar_id
                                                  AND m.href = o.href
                                                  AND m.rrule IS NOT NULL AND m.rrule != ''))",
        )
        .bind(cal_id)
        .fetch_one(pool)
        .await
        .is_ok_and(|v| v != 0)
}

/// Moved instances stored as resources of their own (Kerio) whose master
/// the window holds but which moved out of it: the time-range query never
/// lists them, and their master would block the original slot. Asked for
/// by RECURRENCE-ID without a time range (`CaldavClient::query_overrides`),
/// UID and RECURRENCE-ID only; those of a master here, with a
/// RECURRENCE-ID from the window's start on and not fetched already, are
/// added to `fetched` (multiget) or, unchanged by etag, to `unchanged`.
/// Asked on a calendar's first windowed fetch, then only while it keeps
/// moved instances apart from their series.
#[allow(clippy::too_many_arguments)]
async fn moved_out(
    pool: &SqlitePool,
    client: &CaldavClient,
    cal_id: &str,
    cal_href: &str,
    window: &SyncWindow,
    stored: &HashMap<String, String>,
    fetched: &mut Fetched,
    unchanged: &mut HashSet<String>,
) -> Result<()> {
    let since = window.since_utc();
    let masters = master_uids(pool, cal_id, &fetched.events).await;
    if masters.is_empty() {
        return Ok(());
    }
    // Only where it can matter: a calendar's first windowed fetch, or one
    // that does keep moved instances apart from their series (Kerio). A
    // server keeping them inside the series' resource (most) is not asked
    // again: the window query brings them with the series.
    let first = synced_until(pool, cal_id).await.is_none();
    if !first && !stores_overrides_apart(pool, cal_id, &fetched.events).await {
        return Ok(());
    }
    let Some(found) = client
        .query_overrides(cal_href, &since)
        .await
        .unwrap_or_else(|e| {
            tracing::debug!(error = %e, "query for moved instances failed");
            None
        })
    else {
        return Ok(());
    };
    let mut known: HashSet<&str> = fetched.events.iter().map(|e| e.href.as_str()).collect();
    known.extend(unchanged.iter().map(String::as_str));
    let since_prefix = window.since_prefix();
    let mut fetch = Vec::new();
    let mut same = Vec::new();
    for raw in &found {
        if known.contains(raw.href.as_str()) {
            continue;
        }
        let mut wanted = false;
        for vevent in crate::utils::split_vevents(&raw.ical_data) {
            let fields = crate::utils::unfold_ical(&vevent);
            let field = |name| crate::utils::extract_vevent_field(&fields, name);
            if field("RRULE").is_some_and(|r| !r.is_empty()) {
                // A series' own resource: the window query has it if it
                // matters.
                wanted = false;
                break;
            }
            let (Some(uid), Some(rid)) = (field("UID"), field("RECURRENCE-ID")) else {
                continue;
            };
            let rid = rid.replace(['-', ':'], "");
            wanted |= masters.contains(&uid) && rid.as_str() >= since_prefix.as_str();
        }
        if !wanted {
            continue;
        }
        match (&raw.etag, stored.get(&raw.href)) {
            (Some(new), Some(old)) if new == old => same.push(raw.href.clone()),
            _ => fetch.push(raw.href.clone()),
        }
    }
    unchanged.extend(same);
    if fetch.is_empty() {
        return Ok(());
    }
    tracing::debug!(
        count = fetch.len(),
        "fetching moved instances stored outside the window"
    );
    if let Some(got) = client.multiget(cal_href, &fetch, &since).await? {
        fetched.events.extend(got.events);
        unchanged.extend(got.unknown);
    }
    Ok(())
}

/// A delta from `token` (see the module docs), in upstream's result shape,
/// and whether the server reported any change at all: a change whose etag we
/// already have is not refetched, and must not read as BlueMind's empty
/// delta against a changed ctag (which falls back to a full fetch).
pub(crate) async fn delta(
    pool: &SqlitePool,
    client: &CaldavClient,
    cal_id: &str,
    cal_href: &str,
    token: &str,
    window: &SyncWindow,
) -> Result<(SyncResult, bool)> {
    let inline = |r: SyncResult| {
        let reported = !r.changed.is_empty() || !r.deleted_hrefs.is_empty();
        (r, reported)
    };
    let etags = match client.sync_collection_etags(cal_href, token).await {
        Ok(delta) => delta,
        Err(_) => {
            return client
                .sync_collection(cal_href, Some(token))
                .await
                .map(inline)
        }
    };
    let reported = !etags.changed.is_empty() || !etags.deleted_hrefs.is_empty();
    let stored = stored_etags(pool, cal_id).await;
    let (fetch, _) = changed_hrefs(&etags.changed, &stored);
    let changed = if fetch.is_empty() {
        Some(crate::caldav::partial::Multi::default())
    } else {
        client
            .multiget(cal_href, &fetch, &window.since_utc())
            .await?
    };
    match changed {
        // A change that could not be fetched: storing the new token would
        // skip it, so the full fetch takes over (which keeps such rows).
        Some(got) if !got.unknown.is_empty() => {
            anyhow::bail!(
                "{} changed resources could not be fetched",
                got.unknown.len()
            )
        }
        Some(got) => Ok((
            SyncResult {
                new_sync_token: etags.new_sync_token,
                changed: got.events,
                deleted_hrefs: etags.deleted_hrefs,
            },
            reported,
        )),
        // Multiget refused: upstream's inline-data sync-collection.
        None => client
            .sync_collection(cal_href, Some(token))
            .await
            .map(inline),
    }
}

/// The `UNTIL` of an RRULE, as compact `YYYYMMDD[THHMMSS]`, when it has one.
fn rrule_until(rrule: &str) -> Option<String> {
    rrule
        .split(';')
        .find_map(|part| part.strip_prefix("UNTIL="))
        .map(|v| v.trim_end_matches('Z').to_string())
}

/// Delete the cached rows of a calendar outside `window`, plainly (no
/// booking is cancelled: these events were not deleted on the server):
///
/// - one-off rows that ended before `since`, compared by end so a running
///   multi-day event stays, unless it is a moved instance of a recurring
///   master still cached whose original slot (RECURRENCE-ID) is from
///   `since` on: its row is what frees that slot;
/// - one-off rows starting the day after `until`'s date or later, except the moved instances of
///   a recurring master still cached (an unchanged master is not refetched,
///   so its future overrides must stay);
/// - recurring rows whose `UNTIL` is before `since`.
///
/// Returns how many rows went.
pub(crate) async fn prune(pool: &SqlitePool, cal_id: &str, window: &SyncWindow) -> u64 {
    // From the day after `until`'s date: a row starting on that date may be
    // before `until` in UTC, and so listed by the window's query; pruned, it
    // would be fetched again at every resync.
    let since = window.since_prefix();
    let until = (window.until + chrono::Duration::days(1))
        .format("%Y%m%d")
        .to_string();
    let sql = format!(
        "DELETE FROM events
         WHERE calendar_id = ?1 AND (rrule IS NULL OR rrule = '')
           AND (({END_COMPACT} < ?2
                 AND NOT (recurrence_id IS NOT NULL
                          AND replace(replace(recurrence_id, '-', ''), ':', '') >= ?2
                          AND {MASTER_CACHED}))
                OR ({START_COMPACT} >= ?3 AND NOT {MASTER_CACHED}))"
    );
    let mut removed = sqlx::query(&sql)
        .bind(cal_id)
        .bind(&since)
        .bind(&until)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or(0);

    let ended: Vec<String> = sqlx::query_as::<_, (String, String)>(
        "SELECT id, rrule FROM events WHERE calendar_id = ? AND rrule LIKE '%UNTIL=%'",
    )
    .bind(cal_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .filter(|(_, rrule)| {
        rrule_until(rrule).is_some_and(|u| u.replace(['-', ':'], "").as_str() < since.as_str())
    })
    .map(|(id, _)| id)
    .collect();
    for batch in ended.chunks(PRUNE_BATCH) {
        let marks = vec!["?"; batch.len()].join(",");
        let sql = format!("DELETE FROM events WHERE id IN ({marks})");
        let mut q = sqlx::query(&sql);
        for id in batch {
            q = q.bind(id);
        }
        removed += q
            .execute(pool)
            .await
            .map(|r| r.rows_affected())
            .unwrap_or(0);
    }
    if removed > 0 {
        tracing::debug!(calendar_id = %cal_id, removed, "pruned cached events outside the sync window");
    }
    removed
}

/// The `synced_until` of a calendar.
pub(crate) async fn synced_until(pool: &SqlitePool, cal_id: &str) -> Option<String> {
    sqlx::query_scalar("SELECT synced_until FROM calendars WHERE id = ?")
        .bind(cal_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .flatten()
}

/// Record that a full fetch covered the calendar up to `window.until`, unless
/// its sync setting changed meanwhile (the same guard as its ctag).
pub(crate) async fn set_synced_until(
    pool: &SqlitePool,
    cal_id: &str,
    generation: i64,
    window: &SyncWindow,
) {
    let _ = sqlx::query(
        "UPDATE calendars SET synced_until = ?
         WHERE id = ? AND sync_enabled = 1 AND sync_generation = ?",
    )
    .bind(window.until_utc())
    .bind(cal_id)
    .bind(generation)
    .execute(pool)
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caldav::partial::Listed;

    #[test]
    fn rrule_until_reads_the_compact_form() {
        assert_eq!(
            rrule_until("FREQ=WEEKLY;UNTIL=20261001T000000Z;BYDAY=MO").as_deref(),
            Some("20261001T000000")
        );
        assert_eq!(
            rrule_until("FREQ=DAILY;UNTIL=20261001").as_deref(),
            Some("20261001")
        );
        assert_eq!(rrule_until("FREQ=DAILY"), None);
    }

    #[test]
    fn only_new_and_changed_hrefs_are_fetched() {
        let stored: HashMap<String, String> = [("/a", "1"), ("/b", "1"), ("/c", "1")]
            .into_iter()
            .map(|(h, e)| (h.to_string(), e.to_string()))
            .collect();
        let l = |h: &str, e: Option<&str>| Listed {
            href: h.into(),
            etag: e.map(Into::into),
        };
        let (fetch, unchanged) = changed_hrefs(
            &[
                l("/a", Some("1")),
                l("/b", Some("2")),
                l("/c", None),
                l("/d", Some("1")),
            ],
            &stored,
        );
        assert_eq!(fetch, vec!["/b", "/c", "/d"]);
        assert_eq!(unchanged, HashSet::from(["/a".to_string()]));
    }
}
