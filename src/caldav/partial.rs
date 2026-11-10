//! ilia fork: only the boundaries from CalDAV (owner's decision 2026-11-10,
//! "when you load from CalDAV, take only the boundaries, without the
//! content"; ilia-booking docs/decisions.md).
//!
//! The windowed sync (`commands::sync_window`) asks the server for the
//! minimum:
//!
//! - **partial retrieval** (RFC 4791 §9.6): `calendar-data` names the
//!   VEVENT properties availability reads ([`VEVENT_PROPS`]) and the whole
//!   VTIMEZONE (its STANDARD/DAYLIGHT rules resolve custom TZIDs). SUMMARY,
//!   DESCRIPTION, LOCATION, ATTENDEE, VALARM and the rest are never sent;
//! - a **time range** `[since, until)` (§9.9), so nothing past the horizon is
//!   read, and `limit-recurrence-set` (§9.6.6) from `since`, so years of past
//!   overrides of a recurring master stay on the server while every future
//!   one still comes;
//! - **etags first**: the daily resync lists `(href, etag)` for the window and
//!   fetches with `calendar-multiget` (§7.9) only what changed; delta sync
//!   (`sync-collection`, RFC 6578) asks for etags only and multigets the same
//!   way.
//!
//! A server MAY ignore partial retrieval and send the whole object; that is
//! legal, logged once per calendar ([`has_content`]), and the busy-only
//! storage still strips the content. A server that refuses the partial body
//! is asked again with a bare `calendar-data` and remembered for the process
//! ([`partial_refused`]): refused either with an error status, or with a
//! 207 whose `calendar-data` comes back empty or in a 404/403 propstat.
//! A multiget answer is checked against the hrefs asked for: one missing
//! from it (not reported gone) is asked again whole, and if it still does
//! not come it is reported as unknown ([`Multi::unknown`]), so the caller
//! keeps its cached rows instead of taking it for deleted.
//!
//! **Moved instances stored on their own** (Kerio: same UID, a
//! RECURRENCE-ID, a resource of its own) are found by a time-range query
//! only where they moved to. One moved out of the window from a slot inside
//! it would leave that slot busy, so a full fetch also asks, without a time
//! range, for the UID and RECURRENCE-ID of every resource that has one
//! ([`CaldavClient::query_overrides`]), and fetches those of the window's
//! recurring masters it does not have. Only while the server honours
//! partial retrieval for the calendar (otherwise that answer would carry
//! whole objects), and never again in the process once a server ignored
//! the filter.
//!
//! Not used, and why (decisions.md): `expand` (§9.6.5) snapshots the window
//! at fetch time, so every slide of the window would refetch every series,
//! and the answer's shape varies per server; `free-busy-query` (§7.10) has no
//! UIDs, so a booking's own calendar copy could not be told apart (reschedule)
//! and host-deleted bookings could not be detected.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use anyhow::{bail, Result};

use super::{
    extract_tag, parse_event_responses, split_responses, strip_propstat_blocks, xml_escape,
    CaldavClient, RawEvent,
};

/// The VEVENT properties availability needs: times, recurrence, status.
/// EXRULE is deprecated and not read; RDATE is asked for (cheap) although
/// `rrule.rs` does not expand it yet.
pub const VEVENT_PROPS: &[&str] = &[
    "UID",
    "DTSTART",
    "DTEND",
    "DURATION",
    "RRULE",
    "RDATE",
    "EXDATE",
    "RECURRENCE-ID",
    "STATUS",
    "TRANSP",
];

/// Width of `limit-recurrence-set`, from `since`: far enough that every
/// future override comes, so an unchanged master skipped by etag never lacks
/// one when it slides into the window.
const RECURRENCE_SET_DAYS: i64 = 3660;

/// Hrefs per `calendar-multiget`.
pub const MULTIGET_BATCH: usize = 100;

/// The VEVENT properties asked for when looking for moved instances.
const OVERRIDE_PROPS: &[&str] = &["UID", "RECURRENCE-ID", "RRULE"];

/// What a REPORT asks for in `DAV:prop`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Data {
    /// `getetag` only: a listing.
    EtagOnly,
    /// `getetag` and the boundaries-only `calendar-data`.
    Partial,
    /// `getetag` and a bare `calendar-data` (the whole object).
    Full,
}

/// The `calendar-data` element asking for the boundaries only, with the
/// recurrence set limited from `since` (`YYYYMMDDTHHMMSSZ`).
pub fn partial_calendar_data(since: &str) -> String {
    let props: String = VEVENT_PROPS
        .iter()
        .map(|p| format!("<c:prop name=\"{p}\"/>"))
        .collect();
    let limit_end = chrono::NaiveDateTime::parse_from_str(since, "%Y%m%dT%H%M%SZ")
        .map(|t| t + chrono::Duration::days(RECURRENCE_SET_DAYS))
        .map(|t| t.format("%Y%m%dT%H%M%SZ").to_string())
        .unwrap_or_else(|_| "20991231T000000Z".to_string());
    format!(
        "<c:calendar-data>\
<c:comp name=\"VCALENDAR\"><c:prop name=\"VERSION\"/>\
<c:comp name=\"VEVENT\">{props}</c:comp>\
<c:comp name=\"VTIMEZONE\"><c:allprop/><c:allcomp/></c:comp>\
</c:comp>\
<c:limit-recurrence-set start=\"{since}\" end=\"{limit_end}\"/>\
</c:calendar-data>"
    )
}

fn prop(data: Data, since: &str) -> String {
    match data {
        Data::EtagOnly => "<d:getetag/>".to_string(),
        Data::Partial => format!("<d:getetag/>{}", partial_calendar_data(since)),
        Data::Full => "<d:getetag/><c:calendar-data/>".to_string(),
    }
}

/// `calendar-query` for the VEVENTs overlapping `[since, until)`.
pub fn query_body(since: &str, until: &str, data: Data) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
<d:prop>{prop}</d:prop>
<c:filter><c:comp-filter name="VCALENDAR"><c:comp-filter name="VEVENT">
<c:time-range start="{since}" end="{until}"/>
</c:comp-filter></c:comp-filter></c:filter>
</c:calendar-query>"#,
        prop = prop(data, since),
    )
}

/// `calendar-query` for the resources holding a VEVENT with a RECURRENCE-ID,
/// whenever it is, asking for their UID, RECURRENCE-ID and RRULE only.
pub fn overrides_body(since: &str) -> String {
    let props: String = OVERRIDE_PROPS
        .iter()
        .map(|p| format!("<c:prop name=\"{p}\"/>"))
        .collect();
    let data = partial_calendar_data(since);
    let limit = &data[data.find("<c:limit-recurrence-set").unwrap_or(data.len())..];
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
<d:prop><d:getetag/><c:calendar-data><c:comp name="VCALENDAR"><c:comp name="VEVENT">{props}</c:comp></c:comp>{limit}</d:prop>
<c:filter><c:comp-filter name="VCALENDAR"><c:comp-filter name="VEVENT">
<c:prop-filter name="RECURRENCE-ID"/>
</c:comp-filter></c:comp-filter></c:filter>
</c:calendar-query>"#
    )
}

/// `calendar-multiget` of these hrefs.
pub fn multiget_body(hrefs: &[String], data: Data, since: &str) -> String {
    let hrefs: String = hrefs
        .iter()
        .map(|h| format!("<d:href>{}</d:href>", xml_escape(h)))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<c:calendar-multiget xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
<d:prop>{prop}</d:prop>
{hrefs}
</c:calendar-multiget>"#,
        prop = prop(data, since),
    )
}

/// `sync-collection` asking for etags only.
pub fn sync_etags_body(token: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<d:sync-collection xmlns:d="DAV:">
<d:sync-token>{}</d:sync-token>
<d:sync-level>1</d:sync-level>
<d:prop><d:getetag/></d:prop>
</d:sync-collection>"#,
        xml_escape(token)
    )
}

/// One resource of a listing: its href and etag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listed {
    pub href: String,
    pub etag: Option<String>,
}

/// The answer to an etag-only `sync-collection`.
#[derive(Clone, Debug, Default)]
pub struct EtagDelta {
    pub new_sync_token: Option<String>,
    pub changed: Vec<Listed>,
    pub deleted_hrefs: Vec<String>,
}

/// Whether a response block's resource-level status says it is gone. Only
/// the status outside `propstat` counts (see `parse_sync_response`).
fn resource_gone(block: &str) -> bool {
    extract_tag(&strip_propstat_blocks(block), "d:status")
        .is_some_and(|status| status.contains("404") || status.contains("410"))
}

/// `(href, etag)` of every resource of a listing; the collection itself (an
/// href ending in `/`) and gone resources are left out.
pub fn parse_listing(xml: &str) -> Vec<Listed> {
    split_responses(xml)
        .into_iter()
        .filter(|block| !resource_gone(block))
        .filter_map(|block| {
            let href = extract_tag(block, "d:href")?;
            (!href.ends_with('/')).then(|| Listed {
                etag: extract_tag(block, "d:getetag"),
                href,
            })
        })
        .collect()
}

/// An etag-only `sync-collection` answer.
pub fn parse_sync_etags(xml: &str) -> EtagDelta {
    let mut delta = EtagDelta {
        new_sync_token: extract_tag(xml, "d:sync-token"),
        ..Default::default()
    };
    for block in split_responses(xml) {
        let Some(href) = extract_tag(block, "d:href") else {
            continue;
        };
        if resource_gone(block) {
            delta.deleted_hrefs.push(href);
        } else if !href.ends_with('/') {
            delta.changed.push(Listed {
                etag: extract_tag(block, "d:getetag"),
                href,
            });
        }
    }
    delta
}

/// The path of an href, for comparing what was asked with what came back
/// (a server may answer with an absolute URL).
fn href_path(href: &str) -> &str {
    let rest = href
        .strip_prefix("https://")
        .or_else(|| href.strip_prefix("http://"));
    match rest {
        Some(rest) => rest.find('/').map_or("/", |i| &rest[i..]),
        None => href,
    }
}

/// The hrefs an answer reports gone (resource-level 404 or 410).
fn gone_hrefs(xml: &str) -> HashSet<String> {
    split_responses(xml)
        .into_iter()
        .filter(|block| resource_gone(block))
        .filter_map(|block| extract_tag(block, "d:href"))
        .map(|h| href_path(&h).to_string())
        .collect()
}

/// Whether a 207 left out calendar data it was asked for: a live resource
/// came back without any (in a 403/404/501 propstat, or empty). Some
/// servers refuse partial retrieval this way instead of with an error.
fn data_missing(xml: &str, events: &[RawEvent]) -> bool {
    let with_data: HashSet<&str> = events.iter().map(|e| href_path(&e.href)).collect();
    split_responses(xml).into_iter().any(|block| {
        !resource_gone(block)
            && extract_tag(block, "d:href")
                .is_some_and(|h| !h.ends_with('/') && !with_data.contains(href_path(&h)))
    })
}

/// What a `calendar-multiget` got.
#[derive(Clone, Debug, Default)]
pub struct Multi {
    pub events: Vec<RawEvent>,
    /// Hrefs asked for that came back neither with data nor as gone, even
    /// asked for whole: present on the server as far as anyone knows.
    pub unknown: Vec<String>,
}

/// Whether returned calendar data carries content partial retrieval leaves
/// out: the server ignored it. Only the property names are looked at.
pub fn has_content(ical: &str) -> bool {
    ical.lines().any(|line| {
        let name = line
            .split([':', ';'])
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_uppercase();
        matches!(
            name.as_str(),
            "SUMMARY" | "DESCRIPTION" | "LOCATION" | "ATTENDEE" | "ORGANIZER"
        ) || line.trim().eq_ignore_ascii_case("BEGIN:VALARM")
    })
}

fn registry(cell: &'static OnceLock<Mutex<HashSet<String>>>) -> &'static Mutex<HashSet<String>> {
    cell.get_or_init(Mutex::default)
}

static NO_PARTIAL: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static IGNORED_LOGGED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static NO_OVERRIDE_QUERY: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn registered(cell: &'static OnceLock<Mutex<HashSet<String>>>, url: &str) -> bool {
    registry(cell).lock().is_ok_and(|set| set.contains(url))
}

/// Whether the server sent whole objects to a partial request for this
/// calendar URL in this process.
pub fn partial_ignored(url: &str) -> bool {
    registered(&IGNORED_LOGGED, url)
}

/// Whether the server refused the partial body for this calendar URL in this
/// process.
pub fn partial_refused(url: &str) -> bool {
    registered(&NO_PARTIAL, url)
}

fn note(cell: &'static OnceLock<Mutex<HashSet<String>>>, url: &str, message: &'static str) {
    if let Ok(mut set) = registry(cell).lock() {
        if set.insert(url.to_string()) {
            tracing::info!("{message}");
        }
    }
}

fn note_partial_refused(url: &str) {
    if let Ok(mut set) = registry(&NO_PARTIAL).lock() {
        if set.insert(url.to_string()) {
            tracing::info!(
                "CalDAV server refused partial retrieval; asking this calendar for whole objects (content is still stripped)"
            );
        }
    }
}

/// Logs once per calendar URL that the server ignored partial retrieval.
fn note_if_ignored(url: &str, events: &[RawEvent]) {
    if events.iter().any(|e| has_content(&e.ical_data)) {
        if let Ok(mut set) = registry(&IGNORED_LOGGED).lock() {
            if set.insert(url.to_string()) {
                tracing::info!(
                    "CalDAV server ignored partial retrieval and sent whole events; content is stripped before storage"
                );
            }
        }
    }
}

impl CaldavClient {
    /// One REPORT; `Ok(None)` when the server answered with a non-2xx status
    /// (it refused this body), `Err` when the request failed.
    async fn report(&self, url: &str, depth: Option<&str>, body: String) -> Result<Option<String>> {
        let req = self.request(reqwest::Method::from_bytes(b"REPORT")?, url)?;
        let mut req = self
            .apply_auth(req)
            .header("Content-Type", "application/xml; charset=utf-8")
            .timeout(Duration::from_secs(60))
            .body(body);
        if let Some(depth) = depth {
            req = req.header("Depth", depth);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        Ok(status.is_success().then_some(text))
    }

    /// Whether to ask this calendar for the boundaries only.
    pub fn wants_partial(&self, calendar_href: &str) -> bool {
        !partial_refused(&self.resolve_url(calendar_href))
    }

    /// The events overlapping `[since, until)`, boundaries only when the
    /// server allows it. `Ok(None)` when the server refused the time range
    /// itself (the caller falls back to the unfiltered REPORT).
    pub async fn query_window(
        &self,
        calendar_href: &str,
        since: &str,
        until: &str,
    ) -> Result<Option<Vec<RawEvent>>> {
        let url = self.resolve_url(calendar_href);
        let partial = self.wants_partial(calendar_href);
        if partial {
            if let Some(xml) = self
                .report(&url, Some("1"), query_body(since, until, Data::Partial))
                .await?
            {
                let events = parse_event_responses(&xml);
                if !data_missing(&xml, &events) {
                    note_if_ignored(&url, &events);
                    return Ok(Some(events));
                }
                // A 207 without the data: refused in the answer's body.
            }
        }
        let whole = self
            .report(&url, Some("1"), query_body(since, until, Data::Full))
            .await?;
        // Remembered only when the whole-object query worked: otherwise it
        // was the time range that was refused.
        if partial && whole.is_some() {
            note_partial_refused(&url);
        }
        // `None` (refused whole too) sends the caller to the unfiltered
        // REPORT, whose answer is complete.
        Ok(whole.map(|xml| parse_event_responses(&xml)))
    }

    /// `(href, etag)` of the events overlapping `[since, until)`. `Ok(None)`
    /// when the server refused the listing.
    pub async fn list_window(
        &self,
        calendar_href: &str,
        since: &str,
        until: &str,
    ) -> Result<Option<Vec<Listed>>> {
        let url = self.resolve_url(calendar_href);
        Ok(self
            .report(&url, Some("1"), query_body(since, until, Data::EtagOnly))
            .await?
            .map(|xml| parse_listing(&xml)))
    }

    /// These resources by `calendar-multiget`, in batches, boundaries only
    /// when the server allows it. `Ok(None)` when the server refused it.
    /// Every href asked for comes back with data, is reported gone (left
    /// out: it was deleted), or is in [`Multi::unknown`]; one missing from a
    /// partial answer is asked for again whole first.
    pub async fn multiget(
        &self,
        calendar_href: &str,
        hrefs: &[String],
        since: &str,
    ) -> Result<Option<Multi>> {
        let url = self.resolve_url(calendar_href);
        let mut out = Multi::default();
        for batch in hrefs.chunks(MULTIGET_BATCH) {
            let mut answer = None;
            let partial = self.wants_partial(calendar_href);
            if partial {
                answer = self
                    .report(&url, Some("1"), multiget_body(batch, Data::Partial, since))
                    .await?;
            }
            let mut asked_whole = false;
            if answer.is_none() {
                asked_whole = true;
                answer = self
                    .report(&url, Some("1"), multiget_body(batch, Data::Full, since))
                    .await?;
                if partial && answer.is_some() {
                    note_partial_refused(&url);
                }
            }
            let Some(xml) = answer else {
                return Ok(None);
            };
            let mut events = parse_event_responses(&xml);
            let mut gone = gone_hrefs(&xml);
            let missing = |events: &[RawEvent], gone: &HashSet<String>| -> Vec<String> {
                let got: HashSet<&str> = events.iter().map(|e| href_path(&e.href)).collect();
                batch
                    .iter()
                    .filter(|h| {
                        let path = href_path(h);
                        !got.contains(path) && !gone.contains(path)
                    })
                    .cloned()
                    .collect()
            };
            let mut lacking = missing(&events, &gone);
            if !lacking.is_empty() && !asked_whole {
                // Refused in the body (a 404/403 propstat for the data):
                // those once more, whole.
                if let Some(xml) = self
                    .report(&url, Some("1"), multiget_body(&lacking, Data::Full, since))
                    .await?
                {
                    let whole = parse_event_responses(&xml);
                    if !whole.is_empty() {
                        note_partial_refused(&url);
                    }
                    events.extend(whole);
                    gone.extend(gone_hrefs(&xml));
                    lacking = missing(&events, &gone);
                }
            }
            if !lacking.is_empty() {
                tracing::info!(
                    unknown = lacking.len(),
                    "CalDAV multiget left resources out; their cached events are kept"
                );
            }
            note_if_ignored(&url, &events);
            out.events.extend(events);
            out.unknown.extend(lacking);
        }
        Ok(Some(out))
    }

    /// The sync token of a collection, from an etag-only `sync-collection`
    /// with an empty token (RFC 6578 §3.8): the server lists every member's
    /// href and etag, and never its data. `Ok(None)` when it has none.
    pub async fn initial_sync_token(&self, calendar_href: &str) -> Result<Option<String>> {
        let url = self.resolve_url(calendar_href);
        Ok(self
            .report(&url, None, sync_etags_body(""))
            .await?
            .and_then(|xml| parse_sync_etags(&xml).new_sync_token))
    }

    /// The resources holding a moved instance (a VEVENT with RECURRENCE-ID),
    /// anywhere in time, with their UID, RECURRENCE-ID and RRULE only. `None`
    /// when not to ask: the server refused or ignored partial retrieval for
    /// this calendar, or ignored the filter before, or refused this query.
    pub async fn query_overrides(
        &self,
        calendar_href: &str,
        since: &str,
    ) -> Result<Option<Vec<RawEvent>>> {
        let url = self.resolve_url(calendar_href);
        if partial_refused(&url) || partial_ignored(&url) || registered(&NO_OVERRIDE_QUERY, &url) {
            return Ok(None);
        }
        let Some(xml) = self.report(&url, Some("1"), overrides_body(since)).await? else {
            note(
                &NO_OVERRIDE_QUERY,
                &url,
                "CalDAV server refused the query for moved instances; not asked again",
            );
            return Ok(None);
        };
        let events = parse_event_responses(&xml);
        let unfiltered = events
            .iter()
            .any(|e| !e.ical_data.contains("RECURRENCE-ID"));
        if unfiltered || events.iter().any(|e| has_content(&e.ical_data)) {
            // The filter or the partial body was ignored: this answer is
            // larger than it should be, and the next would be too.
            note(
                &NO_OVERRIDE_QUERY,
                &url,
                "CalDAV server ignored the query for moved instances; not asked again",
            );
        }
        Ok(Some(events))
    }

    /// An etag-only `sync-collection` (RFC 6578) from `token`.
    pub async fn sync_collection_etags(
        &self,
        calendar_href: &str,
        token: &str,
    ) -> Result<EtagDelta> {
        let url = self.resolve_url(calendar_href);
        match self.report(&url, None, sync_etags_body(token)).await? {
            Some(xml) => Ok(parse_sync_etags(&xml)),
            None => bail!("etag-only sync-collection refused"),
        }
    }
}

/// Tests only: a Kerio-style mock CalDAV server.
#[cfg(test)]
#[path = "mock_dav.rs"]
pub(crate) mod mock_dav;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A Kerio Connect answer, in the shape public reports show
    /// (twenty#25781, vandelay#48, Mozilla 456380): `a:` for DAV, `c:` for
    /// CalDAV, absolute `.eml` hrefs under a UUID folder, quoted etags, a
    /// `watermark:` sync token, bare-CR line endings. One weekly master in
    /// Moscow time with EXDATEs and an override in the same resource, one
    /// override of it stored as a separate resource with the same UID
    /// (calcom#8516), an all-day event, a transparent one, a cancelled one,
    /// and a Windows zone name with its VTIMEZONE. `full` adds what a server
    /// ignoring partial retrieval sends: titles, descriptions, attendees,
    /// alarms, X- properties.
    pub(crate) fn kerio_multistatus(full: bool) -> String {
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<a:multistatus xmlns:a=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\">",
        );
        for (n, (name, data)) in kerio_resources(full).iter().enumerate() {
            let id = &name[..2];
            xml.push_str(&format!(
                "<a:response><a:href>{}{name}</a:href><a:propstat><a:prop><a:getetag>\"{id}-{n}\"</a:getetag><c:calendar-data>{data}</c:calendar-data></a:prop><a:status>HTTP/1.1 200 OK</a:status></a:propstat></a:response>",
                super::mock_dav::CAL
            ));
        }
        xml.push_str("<a:sync-token>watermark:437165</a:sync-token></a:multistatus>");
        xml
    }

    /// The resources of [`kerio_multistatus`]: file name and bare-CR data.
    pub(crate) fn kerio_resources(full: bool) -> Vec<(String, String)> {
        let content = |title: &str| -> String {
            if !full {
                return String::new();
            }
            let mut s = format!(
                "SUMMARY:{title}\nDESCRIPTION:Agenda: {}\nLOCATION:Room 4\nORGANIZER;CN=Owner:mailto:owner@ilia.ae\n",
                "confidential notes ".repeat(10)
            );
            for n in 0..5 {
                s.push_str(&format!(
                    "ATTENDEE;CN=Guest {n};PARTSTAT=ACCEPTED:mailto:g{n}@example.com\n"
                ));
            }
            s.push_str("X-KERIO-PRIVATE:1\nBEGIN:VALARM\nACTION:DISPLAY\nTRIGGER:-PT15M\nDESCRIPTION:Reminder\nEND:VALARM\n");
            s
        };
        let moscow = "BEGIN:VTIMEZONE\nTZID:Europe/Moscow\nBEGIN:STANDARD\nDTSTART:19700101T000000\nTZOFFSETFROM:+0300\nTZOFFSETTO:+0300\nEND:STANDARD\nEND:VTIMEZONE\n";
        let windows = "BEGIN:VTIMEZONE\nTZID:Russian Standard Time\nBEGIN:STANDARD\nDTSTART:16010101T000000\nTZOFFSETFROM:+0300\nTZOFFSETTO:+0300\nEND:STANDARD\nEND:VTIMEZONE\n";
        let cal = |tz: &str, body: String| -> String {
            format!("BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//Kerio Technologies//Kerio Connect//EN\n{tz}{body}END:VCALENDAR\n")
                .replace('\n', "\r")
        };
        let resources = [
            (
                "a1",
                cal(
                    moscow,
                    format!(
                        "BEGIN:VEVENT\nUID:kerio-weekly\nDTSTAMP:20261101T000000Z\nDTSTART;TZID=Europe/Moscow:20261012T100000\nDTEND;TZID=Europe/Moscow:20261012T110000\nRRULE:FREQ=WEEKLY;BYDAY=MO\nEXDATE;TZID=Europe/Moscow:20261130T100000\n{}END:VEVENT\nBEGIN:VEVENT\nUID:kerio-weekly\nRECURRENCE-ID;TZID=Europe/Moscow:20261123T100000\nDTSTART;TZID=Europe/Moscow:20261123T150000\nDTEND;TZID=Europe/Moscow:20261123T160000\n{}END:VEVENT\n",
                        content("Weekly sync"),
                        content("Weekly sync (moved)")
                    ),
                ),
            ),
            (
                "a2",
                cal(
                    moscow,
                    format!(
                        "BEGIN:VEVENT\nUID:kerio-weekly\nRECURRENCE-ID;TZID=Europe/Moscow:20261116T100000\nDTSTART;TZID=Europe/Moscow:20261116T170000\nDTEND;TZID=Europe/Moscow:20261116T180000\n{}END:VEVENT\n",
                        content("Weekly sync (moved, own resource)")
                    ),
                ),
            ),
            (
                "a3",
                cal(
                    "",
                    format!(
                        "BEGIN:VEVENT\nUID:kerio-allday\nDTSTART;VALUE=DATE:20261118\nDTEND;VALUE=DATE:20261119\n{}END:VEVENT\n",
                        content("Day off")
                    ),
                ),
            ),
            (
                "a4",
                cal(
                    "",
                    format!(
                        "BEGIN:VEVENT\nUID:kerio-free\nDTSTART:20261117T090000Z\nDTEND:20261117T100000Z\nTRANSP:TRANSPARENT\n{}END:VEVENT\n",
                        content("Focus")
                    ),
                ),
            ),
            (
                "a5",
                cal(
                    "",
                    format!(
                        "BEGIN:VEVENT\nUID:kerio-cancelled\nDTSTART:20261119T090000Z\nDTEND:20261119T100000Z\nSTATUS:CANCELLED\n{}END:VEVENT\n",
                        content("Cancelled")
                    ),
                ),
            ),
            (
                "a6",
                cal(
                    windows,
                    format!(
                        "BEGIN:VEVENT\nUID:kerio-windows\nDTSTART;TZID=Russian Standard Time:20261120T120000\nDTEND;TZID=Russian Standard Time:20261120T130000\n{}END:VEVENT\n",
                        content("Windows zone")
                    ),
                ),
            ),
        ];
        resources
            .into_iter()
            .enumerate()
            .map(|(n, (id, data))| {
                (
                    format!("{id}5d0e-7c1f-4f7a-9b2e-0c9d{n}-00000001.eml"),
                    data,
                )
            })
            .collect()
    }

    #[test]
    fn the_partial_body_names_only_the_boundaries() {
        let body = query_body("20261108T120000Z", "20261127T120000Z", Data::Partial);
        for p in VEVENT_PROPS {
            assert!(body.contains(&format!("<c:prop name=\"{p}\"/>")), "{p}");
        }
        for absent in ["SUMMARY", "DESCRIPTION", "LOCATION", "ATTENDEE", "VALARM"] {
            assert!(!body.contains(&format!("\"{absent}\"")), "{absent}");
        }
        assert!(body.contains("<c:comp name=\"VTIMEZONE\"><c:allprop/><c:allcomp/></c:comp>"));
        assert!(
            body.contains("<c:time-range start=\"20261108T120000Z\" end=\"20261127T120000Z\"/>")
        );
        assert!(body.contains(
            "<c:limit-recurrence-set start=\"20261108T120000Z\" end=\"20361115T120000Z\"/>"
        ));
        assert!(!query_body("a", "b", Data::EtagOnly).contains("calendar-data"));
        assert!(query_body("a", "b", Data::Full).contains("<c:calendar-data/>"));
    }

    #[test]
    fn multiget_escapes_hrefs() {
        let body = multiget_body(
            &["/c/a&b.eml".to_string()],
            Data::Partial,
            "20261108T120000Z",
        );
        assert!(body.contains("<d:href>/c/a&amp;b.eml</d:href>"));
        assert!(body.contains("calendar-multiget"));
        assert!(body.contains("<c:comp name=\"VEVENT\">"));
    }

    #[test]
    fn kerio_answers_parse_with_etags_in_both_variants() {
        for full in [false, true] {
            let events = parse_event_responses(&kerio_multistatus(full));
            assert_eq!(events.len(), 6);
            assert!(events[0].href.ends_with("-00000001.eml"));
            assert_eq!(events[0].etag.as_deref(), Some("\"a1-0\""));
            assert!(events[0]
                .ical_data
                .contains("\r\nRRULE:FREQ=WEEKLY;BYDAY=MO\r\n"));
            assert_eq!(events.iter().any(|e| has_content(&e.ical_data)), full);
        }
        let listing = parse_listing(&kerio_multistatus(false));
        assert_eq!(listing.len(), 6);
        assert_eq!(listing[5].etag.as_deref(), Some("\"a6-5\""));
    }

    #[test]
    fn etag_sync_answers_split_changes_and_deletions() {
        let xml = "<a:multistatus xmlns:a=\"DAV:\">\
            <a:response><a:href>/full-calendars/d/u/f/x-00000001.eml</a:href><a:propstat><a:prop><a:getetag>\"9\"</a:getetag></a:prop><a:status>HTTP/1.1 200 OK</a:status></a:propstat></a:response>\
            <a:response><a:href>/full-calendars/d/u/f/y-00000001.eml</a:href><a:status>HTTP/1.1 404 Not Found</a:status></a:response>\
            <a:response><a:href>/full-calendars/d/u/f/z-00000001.eml</a:href><a:propstat><a:prop><a:getetag/></a:prop><a:status>HTTP/1.1 404 Not Found</a:status></a:propstat></a:response>\
            <a:sync-token>watermark:437170</a:sync-token></a:multistatus>";
        let delta = parse_sync_etags(xml);
        assert_eq!(delta.new_sync_token.as_deref(), Some("watermark:437170"));
        assert_eq!(
            delta.deleted_hrefs,
            vec!["/full-calendars/d/u/f/y-00000001.eml"]
        );
        // A property-level 404 is a missing etag on a live resource.
        assert_eq!(
            delta.changed,
            vec![
                Listed {
                    href: "/full-calendars/d/u/f/x-00000001.eml".into(),
                    etag: Some("\"9\"".into())
                },
                Listed {
                    href: "/full-calendars/d/u/f/z-00000001.eml".into(),
                    etag: None
                },
            ]
        );
    }

    #[test]
    fn content_detection_reads_property_names_only() {
        assert!(!has_content(
            "BEGIN:VEVENT\r\nUID:x\r\nDTSTART:20261110T100000Z\r\nEND:VEVENT"
        ));
        assert!(has_content("BEGIN:VEVENT\nSUMMARY:x\nEND:VEVENT"));
        assert!(has_content(
            "BEGIN:VEVENT\nATTENDEE;CN=a:mailto:a@b\nEND:VEVENT"
        ));
        assert!(has_content("BEGIN:VALARM\nEND:VALARM"));
        assert!(!has_content("UID:SUMMARY-like-uid"));
    }
}
