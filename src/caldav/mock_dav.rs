//! ilia fork, tests only: an in-process CalDAV server in Kerio Connect's
//! style (`a:` DAV prefix, `.eml` hrefs under a UUID folder, quoted etags,
//! `watermark:` sync tokens, bare-CR data) for the windowed-sync tests and
//! the byte measurement (`bytes_tests.rs`).
//!
//! It honours what the windowed sync asks for, each switchable off to
//! exercise the fallbacks: `calendar-query` with a time range (one-off
//! events by overlap, recurring ones by expansion), partial retrieval
//! (`comp`/`prop`, VTIMEZONE whole, `limit-recurrence-set`),
//! `calendar-multiget`, and `sync-collection` with or without data. Every
//! request and the size of every answer are logged.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use axum::http::{Method, StatusCode, Uri};
use chrono::NaiveDateTime;

/// Where the one calendar lives.
pub const CAL: &str = "/full-calendars/ilia.ae/owner/0f6c8b3e-6b1d-4c3a-9d1e-2a7c5b9e1f00/";

/// One request the server got.
#[derive(Clone, Debug)]
pub struct Logged {
    pub method: String,
    pub path: String,
    pub body: String,
    pub status: u16,
    pub bytes: usize,
}

/// What the server does.
#[derive(Clone, Debug)]
pub struct Behaviour {
    /// Apply partial retrieval when asked (else send whole objects, legal).
    pub honour_partial: bool,
    /// Answer 400 to a body that asks for partial retrieval.
    pub refuse_partial: bool,
    /// Answer 501 to `calendar-multiget`.
    pub refuse_multiget: bool,
    /// Answer 400 to a `calendar-query` with a time range.
    pub refuse_time_range: bool,
    /// Answer 403 to an etag-only `sync-collection`.
    pub refuse_etag_sync: bool,
    /// Report no sync token (no delta at all).
    pub no_sync: bool,
    /// Leave the sync token out of PROPFIND, while `sync-collection` works
    /// (an empty token then lists every member).
    pub token_in_report_only: bool,
    /// Answer a body asking for partial retrieval with a 207 whose
    /// `calendar-data` sits in a 404 propstat (refused in the body).
    pub refuse_partial_in_body: bool,
    /// Leave these hrefs out of a `calendar-multiget` answer altogether.
    pub multiget_omits: Vec<String>,
}

impl Default for Behaviour {
    fn default() -> Self {
        Self {
            honour_partial: true,
            refuse_partial: false,
            refuse_multiget: false,
            refuse_time_range: false,
            refuse_etag_sync: false,
            no_sync: false,
            token_in_report_only: false,
            refuse_partial_in_body: false,
            multiget_omits: Vec::new(),
        }
    }
}

#[derive(Default)]
struct State {
    /// href → (etag, data with `\n` line ends).
    resources: BTreeMap<String, (String, String)>,
    /// Changes in order: (version, href).
    changes: Vec<(u64, String)>,
    version: u64,
    log: Vec<Logged>,
}

/// A running mock server.
#[derive(Clone)]
pub struct MockDav {
    state: Arc<Mutex<State>>,
    pub behaviour: Arc<Mutex<Behaviour>>,
    pub base: String,
}

impl MockDav {
    /// Starts a server holding these resources (file name → iCal data).
    pub async fn start(resources: Vec<(String, String)>, behaviour: Behaviour) -> MockDav {
        let dav = MockDav {
            state: Arc::default(),
            behaviour: Arc::new(Mutex::new(behaviour)),
            base: String::new(),
        };
        for (name, data) in resources {
            dav.put(&name, &data);
        }
        dav.state.lock().unwrap().changes.clear();
        let served = dav.clone();
        let app = axum::Router::new().fallback(move |method: Method, uri: Uri, body: String| {
            let dav = served.clone();
            async move { dav.answer(method, uri, body) }
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        MockDav {
            base: format!("http://127.0.0.1:{port}/caldav/"),
            ..dav
        }
    }

    pub fn href(name: &str) -> String {
        format!("{CAL}{name}")
    }

    /// Creates or changes a resource.
    pub fn put(&self, name: &str, data: &str) {
        let mut st = self.state.lock().unwrap();
        st.version += 1;
        let v = st.version;
        let href = Self::href(name);
        st.resources.insert(
            href.clone(),
            (
                format!("\"{v}\""),
                data.replace("\r\n", "\n").replace('\r', "\n"),
            ),
        );
        st.changes.push((v, href));
    }

    /// Deletes a resource.
    pub fn delete(&self, name: &str) {
        let mut st = self.state.lock().unwrap();
        st.version += 1;
        let v = st.version;
        let href = Self::href(name);
        st.resources.remove(&href);
        st.changes.push((v, href));
    }

    pub fn log(&self) -> Vec<Logged> {
        self.state.lock().unwrap().log.clone()
    }

    pub fn clear_log(&self) {
        self.state.lock().unwrap().log.clear();
    }

    /// Bytes of every answer since the log was cleared.
    pub fn bytes(&self) -> usize {
        self.log().iter().map(|l| l.bytes).sum()
    }

    fn answer(&self, method: Method, uri: Uri, body: String) -> (StatusCode, String) {
        let path = uri.path().to_string();
        let (status, text) = self.route(method.as_str(), &path, &body);
        let mut st = self.state.lock().unwrap();
        st.log.push(Logged {
            method: method.to_string(),
            path,
            body,
            status: status.as_u16(),
            bytes: text.len(),
        });
        (status, text)
    }

    fn route(&self, method: &str, path: &str, body: &str) -> (StatusCode, String) {
        let ms = |inner: &str| {
            format!(
                "<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<a:multistatus xmlns:a=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\" xmlns:cs=\"http://calendarserver.org/ns/\">{inner}</a:multistatus>"
            )
        };
        let behaviour = self.behaviour.lock().unwrap().clone();
        let cal = CAL.trim_end_matches('/');
        match (method, path.trim_end_matches('/')) {
            ("PROPFIND", "/caldav") => (
                StatusCode::MULTI_STATUS,
                ms("<a:response><a:href>/caldav/</a:href><a:propstat><a:prop><a:current-user-principal><a:href>/caldav/users/ilia.ae/owner/</a:href></a:current-user-principal></a:prop><a:status>HTTP/1.1 200 OK</a:status></a:propstat></a:response>"),
            ),
            ("PROPFIND", "/caldav/users/ilia.ae/owner") => (
                StatusCode::MULTI_STATUS,
                ms("<a:response><a:href>/caldav/users/ilia.ae/owner/</a:href><a:propstat><a:prop><c:calendar-home-set><a:href>/full-calendars/ilia.ae/owner/</a:href></c:calendar-home-set></a:prop><a:status>HTTP/1.1 200 OK</a:status></a:propstat></a:response>"),
            ),
            ("PROPFIND", "/full-calendars/ilia.ae/owner") => {
                let v = self.state.lock().unwrap().version;
                let token = if behaviour.no_sync || behaviour.token_in_report_only {
                    String::new()
                } else {
                    format!("<a:sync-token>watermark:{v}</a:sync-token>")
                };
                (
                    StatusCode::MULTI_STATUS,
                    ms(&format!(
                        "<a:response><a:href>{CAL}</a:href><a:propstat><a:prop><a:displayname>Calendar</a:displayname><a:resourcetype><a:collection/><c:calendar/></a:resourcetype><cs:getctag>{v}</cs:getctag>{token}</a:prop><a:status>HTTP/1.1 200 OK</a:status></a:propstat></a:response>"
                    )),
                )
            }
            ("REPORT", p) if p == cal => self.report(body, &behaviour, &ms),
            _ => (StatusCode::NOT_FOUND, String::new()),
        }
    }

    fn report(
        &self,
        body: &str,
        behaviour: &Behaviour,
        ms: &dyn Fn(&str) -> String,
    ) -> (StatusCode, String) {
        let wants_data = body.contains("calendar-data");
        let partial = body.contains("<c:comp name=\"VCALENDAR\">");
        if partial && behaviour.refuse_partial {
            return (StatusCode::BAD_REQUEST, String::new());
        }
        let limit_from = attr(body, "limit-recurrence-set", "start");
        // The VEVENT properties asked for, and whether VTIMEZONE is.
        let props: Vec<String> = body
            .match_indices("<c:prop name=\"")
            .filter_map(|(i, m)| {
                let from = i + m.len();
                Some(body[from..from + body[from..].find('"')?].to_string())
            })
            .collect();
        let zones = body.contains("<c:comp name=\"VTIMEZONE\">");
        let shape = |data: &str| -> String {
            if partial && behaviour.honour_partial {
                boundaries(data, limit_from.as_deref(), &props, zones)
            } else {
                data.to_string()
            }
        };
        let refused_in_body = partial && behaviour.refuse_partial_in_body;
        let st = self.state.lock().unwrap();
        let entry = |href: &str, etag: &str, data: Option<&str>| -> String {
            if refused_in_body && data.is_some() {
                return format!("<a:response><a:href>{href}</a:href><a:propstat><a:prop><a:getetag>{etag}</a:getetag></a:prop><a:status>HTTP/1.1 200 OK</a:status></a:propstat><a:propstat><a:prop><c:calendar-data/></a:prop><a:status>HTTP/1.1 404 Not Found</a:status></a:propstat></a:response>");
            }
            let data = data
                .map(|d| {
                    format!(
                        "<c:calendar-data>{}</c:calendar-data>",
                        xml(&shape(d).replace('\n', "\r"))
                    )
                })
                .unwrap_or_default();
            format!("<a:response><a:href>{href}</a:href><a:propstat><a:prop><a:getetag>{etag}</a:getetag>{data}</a:prop><a:status>HTTP/1.1 200 OK</a:status></a:propstat></a:response>")
        };
        if body.contains("calendar-multiget") {
            if behaviour.refuse_multiget {
                return (StatusCode::NOT_IMPLEMENTED, String::new());
            }
            let mut out = String::new();
            for href in tags(body, "d:href") {
                if behaviour.multiget_omits.contains(&href) {
                    continue;
                }
                match st.resources.get(&href) {
                    Some((etag, data)) => out.push_str(&entry(&href, etag, wants_data.then_some(data.as_str()))),
                    None => out.push_str(&format!("<a:response><a:href>{href}</a:href><a:status>HTTP/1.1 404 Not Found</a:status></a:response>")),
                }
            }
            return (StatusCode::MULTI_STATUS, ms(&out));
        }
        if body.contains("sync-collection") {
            if behaviour.no_sync {
                return (StatusCode::FORBIDDEN, String::new());
            }
            if !wants_data && behaviour.refuse_etag_sync {
                return (StatusCode::FORBIDDEN, String::new());
            }
            let from: Option<u64> = tags(body, "d:sync-token")
                .first()
                .and_then(|t| t.strip_prefix("watermark:").and_then(|n| n.parse().ok()));
            // An empty token (RFC 6578 §3.8): every member.
            let mut hrefs: Vec<&String> = match from {
                Some(from) => st
                    .changes
                    .iter()
                    .filter(|(v, _)| *v > from)
                    .map(|(_, h)| h)
                    .collect(),
                None => st.resources.keys().collect(),
            };
            hrefs.sort();
            hrefs.dedup();
            let mut out = String::new();
            for href in hrefs {
                match st.resources.get(href) {
                    Some((etag, data)) => out.push_str(&entry(href, etag, wants_data.then_some(data.as_str()))),
                    None => out.push_str(&format!("<a:response><a:href>{href}</a:href><a:status>HTTP/1.1 404 Not Found</a:status></a:response>")),
                }
            }
            out.push_str(&format!(
                "<a:sync-token>watermark:{}</a:sync-token>",
                st.version
            ));
            return (StatusCode::MULTI_STATUS, ms(&out));
        }
        // calendar-query
        let start = attr(body, "time-range", "start");
        let end = attr(body, "time-range", "end");
        if start.is_some() && behaviour.refuse_time_range {
            return (StatusCode::BAD_REQUEST, String::new());
        }
        // `<c:prop-filter name="RECURRENCE-ID"/>`: resources with one.
        let needs_rid = body.contains("<c:prop-filter name=\"RECURRENCE-ID\"/>");
        let mut out = String::new();
        for (href, (etag, data)) in &st.resources {
            if needs_rid && !data.contains("\nRECURRENCE-ID") {
                continue;
            }
            if overlaps(data, start.as_deref(), end.as_deref()) {
                out.push_str(&entry(href, etag, wants_data.then_some(data.as_str())));
            }
        }
        (StatusCode::MULTI_STATUS, ms(&out))
    }
}

fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The value of `name="…"` on the first `<…element` tag.
fn attr(body: &str, element: &str, name: &str) -> Option<String> {
    let at = body.find(&format!(":{element} "))?;
    let tag = &body[at..at + body[at..].find('>')?];
    let key = format!("{name}=\"");
    let from = tag.find(&key)? + key.len();
    Some(tag[from..from + tag[from..].find('"')?].to_string())
}

/// The text of every `<tag>…</tag>`.
fn tags(body: &str, tag: &str) -> Vec<String> {
    let (open, close) = (format!("<{tag}>"), format!("</{tag}>"));
    body.match_indices(&open)
        .filter_map(|(i, _)| {
            let from = i + open.len();
            let to = from + body[from..].find(&close)?;
            Some(body[from..to].replace("&amp;", "&"))
        })
        .collect()
}

fn utc(value: &str) -> Option<NaiveDateTime> {
    crate::utils::parse_ical_datetime(value.trim_end_matches('Z'))
}

/// Whether any VEVENT of a resource overlaps `[start, end)` (RFC 4791 §9.9,
/// every recurrence instance considered; times read as UTC).
fn overlaps(data: &str, start: Option<&str>, end: Option<&str>) -> bool {
    let Some(start) = start.and_then(utc) else {
        return true;
    };
    let end = end.and_then(utc).unwrap_or(NaiveDateTime::MAX);
    crate::utils::split_vevents(&data.replace('\n', "\r\n"))
        .iter()
        .any(|v| {
            let f = crate::utils::unfold_ical(v);
            let Some(s) = crate::utils::extract_vevent_field(&f, "DTSTART").and_then(|s| utc(&s))
            else {
                return true;
            };
            let e = crate::utils::extract_vevent_end(&f, None)
                .and_then(|e| utc(&e))
                .unwrap_or(s);
            match crate::utils::extract_vevent_field(&f, "RRULE") {
                Some(rule) => !crate::rrule::expand_rrule(s, e, &rule, &[], start, end).is_empty(),
                None => s < end && e > start,
            }
        })
}

/// Partial retrieval as the windowed sync asks for it: the `props` of each
/// VEVENT, VTIMEZONE whole when `zones`, VALARM and everything else left
/// out; overrides wholly before `limit_from` dropped.
fn boundaries(data: &str, limit_from: Option<&str>, props: &[String], zones: bool) -> String {
    let limit = limit_from.and_then(utc);
    let mut out = String::new();
    let mut block: Vec<&str> = Vec::new();
    let (mut in_event, mut in_alarm, mut in_tz) = (false, false, false);
    for line in data.lines() {
        let name = line
            .split([':', ';'])
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();
        match line {
            "BEGIN:VEVENT" => {
                in_event = true;
                block = vec![line];
                continue;
            }
            "END:VEVENT" => {
                in_event = false;
                block.push(line);
                let rid = block
                    .iter()
                    .find(|l| l.starts_with("RECURRENCE-ID"))
                    .and_then(|l| l.rsplit(':').next())
                    .and_then(utc);
                let start = block
                    .iter()
                    .find(|l| l.starts_with("DTSTART"))
                    .and_then(|l| l.rsplit(':').next())
                    .and_then(utc);
                let old = matches!((limit, rid, start), (Some(lim), Some(r), Some(s)) if r < lim && s < lim);
                if !old {
                    for l in &block {
                        out.push_str(l);
                        out.push('\n');
                    }
                }
                continue;
            }
            "BEGIN:VTIMEZONE" => in_tz = true,
            "END:VTIMEZONE" => {
                in_tz = false;
                if zones {
                    out.push_str(line);
                    out.push('\n');
                }
                continue;
            }
            "BEGIN:VALARM" => {
                in_alarm = true;
                continue;
            }
            "END:VALARM" => {
                in_alarm = false;
                continue;
            }
            _ => {}
        }
        if in_alarm {
            continue;
        }
        if in_event {
            if props.contains(&name) {
                block.push(line);
            }
        } else if (in_tz && zones)
            || (!in_tz && matches!(name.as_str(), "BEGIN" | "END" | "VERSION"))
        {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries_keep_times_and_zones_only() {
        let data = "BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:x\nBEGIN:VTIMEZONE\nTZID:Europe/Moscow\nBEGIN:STANDARD\nTZOFFSETTO:+0300\nEND:STANDARD\nEND:VTIMEZONE\nBEGIN:VEVENT\nUID:a\nSUMMARY:secret\nDTSTART:20261110T100000Z\nDTEND:20261110T110000Z\nATTENDEE:mailto:x@y\nBEGIN:VALARM\nTRIGGER:-PT5M\nEND:VALARM\nEND:VEVENT\nBEGIN:VEVENT\nUID:a\nRECURRENCE-ID:20200101T100000Z\nDTSTART:20200101T120000Z\nDTEND:20200101T130000Z\nEND:VEVENT\nEND:VCALENDAR\n";
        let props: Vec<String> = crate::caldav::partial::VEVENT_PROPS
            .iter()
            .map(|p| p.to_string())
            .collect();
        let out = boundaries(data, Some("20261108T000000Z"), &props, true);
        assert!(
            !out.contains("secret")
                && !out.contains("ATTENDEE")
                && !out.contains("VALARM")
                && !out.contains("PRODID")
        );
        assert!(out.contains("TZOFFSETTO:+0300") && out.contains("DTSTART:20261110T100000Z"));
        assert!(!out.contains("20200101"), "{out}");
        assert!(overlaps(
            data,
            Some("20261110T103000Z"),
            Some("20261111T000000Z")
        ));
        assert!(!overlaps(
            data,
            Some("20261111T000000Z"),
            Some("20261112T000000Z")
        ));
    }
}
