//! The work behind the calendar tools: which calendars of a source are
//! synced, a sync on demand, and the calendar new bookings are written to.
//!
//! Only the acting user's own sources, by id. A source's address, account and
//! password are never changed here (that is for Ansible and the CLI), so no
//! tool can send stored credentials somewhere new. The write calendar is
//! chosen by id among the calendars the source itself listed, and refused
//! when its address would lead off the source's server.
//!
//! The changes hold `AgentCtx::write_lock` from their first read to their
//! last and leave one `change` audit event naming calendars by id; calendar
//! names and addresses never reach the audit.

use super::dto::*;
use super::error::{db, ToolError};
use super::ops_read;
use super::ops_write::Writer;
use super::SYNC_COOLDOWN_SECS;
use crate::models::User;
use serde_json::{json, Map, Value};
use sqlx::SqlitePool;
use std::collections::BTreeSet;

/// Most calendar ids `set_source_calendars` takes.
const CALENDARS_MAX: usize = 200;

/// An id argument, checked without echoing it.
fn check_id<'a>(field: &str, id: &'a str) -> Result<&'a str, ToolError> {
    let id = id.trim();
    let valid = !id.is_empty()
        && id.len() <= 100
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if valid {
        Ok(id)
    } else {
        Err(ToolError::invalid(
            field,
            "use an id as list_sources shows it",
        ))
    }
}

/// What the tools need of a source.
struct OwnedSource {
    url: String,
    provider: String,
    enabled: bool,
}

/// Source `source_id` if the acting user owns it; the agent's own version of
/// `owned_source` in sources.rs, which is private.
async fn owned_source(
    pool: &SqlitePool,
    me: &User,
    source_id: &str,
) -> Result<OwnedSource, ToolError> {
    let row: Option<(String, String, i64)> = sqlx::query_as(
        "SELECT cs.url, cs.provider_type, cs.enabled
         FROM caldav_sources cs JOIN accounts a ON a.id = cs.account_id
         WHERE cs.id = ? AND a.user_id = ?",
    )
    .bind(source_id)
    .bind(&me.id)
    .fetch_optional(pool)
    .await
    .map_err(db("find source"))?;
    let (url, provider, enabled) = row.ok_or_else(|| ToolError::not_found("calendar source"))?;
    Ok(OwnedSource {
        url,
        provider,
        enabled: enabled != 0,
    })
}

/// Whether calrs would refuse to connect to the source's saved address now:
/// a private or reserved IP literal, or the host of an outbound proxy, not
/// on the private-host allowlist (`net_guard::url_refused`, which the
/// CalDAV and EWS clients apply at every request). Host names that resolve
/// to such an address are refused by the clients' resolver when they
/// connect; that cannot be told in advance without a lookup.
fn source_address_refused(source_url: &str) -> bool {
    reqwest::Url::parse(source_url.trim()).is_ok_and(|url| crate::net_guard::url_refused(&url))
}

/// What the model is told for a source [`source_address_refused`] names.
/// No address: as `net_guard::REFUSED`, that is the administrator's to see.
const ADDRESS_REFUSED: &str = "calrs refuses to connect to this source's server: its address is \
     private or reserved and not on the private-host allowlist (CALRS_ALLOW_PRIVATE_HOSTS, or \
     Admin panel, System settings)";

// --- set_source_calendars ---

/// The calendars of a source: id and whether it is synced.
async fn calendars_of(
    pool: &SqlitePool,
    source_id: &str,
) -> Result<Vec<(String, bool)>, ToolError> {
    let rows: Vec<(String, i64)> =
        sqlx::query_as("SELECT id, sync_enabled FROM calendars WHERE source_id = ? ORDER BY id")
            .bind(source_id)
            .fetch_all(pool)
            .await
            .map_err(db("list calendars"))?;
    Ok(rows.into_iter().map(|(id, on)| (id, on != 0)).collect())
}

/// Sync exactly the calendars listed, as the web's calendar page does
/// (`update_source_calendars`), one calendar at a time through
/// `set_calendar_sync_enabled`, which also drops a calendar's cached events
/// when it is turned off.
pub(crate) async fn set_source_calendars(
    w: &Writer<'_>,
    args: SourceCalendarsArgs,
) -> Result<SourceChanged, ToolError> {
    let source_id = check_id("source_id", &args.source_id)?.to_string();
    if args.sync_calendar_ids.is_empty() {
        return Err(ToolError::invalid(
            "sync_calendar_ids",
            "keep at least one calendar synced: with none, every time would look free",
        ));
    }
    if args.sync_calendar_ids.len() > CALENDARS_MAX {
        return Err(ToolError::invalid(
            "sync_calendar_ids",
            format!("at most {CALENDARS_MAX} ids"),
        ));
    }
    let mut wanted = BTreeSet::new();
    for (i, id) in args.sync_calendar_ids.iter().enumerate() {
        let field = format!("sync_calendar_ids[{i}]");
        let id = check_id(&field, id)?;
        if !wanted.insert(id.to_string()) {
            return Err(ToolError::invalid(&field, "listed twice"));
        }
    }

    let pool = w.pool();
    let _write = w.ctx.write_lock.lock().await;
    owned_source(pool, w.me, &source_id).await?;
    let before = calendars_of(pool, &source_id).await?;
    for (i, id) in args.sync_calendar_ids.iter().enumerate() {
        if !before.iter().any(|(calendar, _)| calendar == id.trim()) {
            return Err(ToolError::invalid(
                &format!("sync_calendar_ids[{i}]"),
                "not a calendar of this source; see list_sources",
            ));
        }
    }
    // Turn calendars on first, then off, so that while this runs the source
    // never syncs fewer calendars than either the old or the new choice.
    let turn_on = before
        .iter()
        .filter(|(id, on)| !on && wanted.contains(id))
        .map(|(id, _)| (id.as_str(), true));
    let turn_off = before
        .iter()
        .filter(|(id, on)| *on && !wanted.contains(id))
        .map(|(id, _)| (id.as_str(), false));
    let mut failed = None;
    for (calendar_id, enabled) in turn_on.chain(turn_off) {
        if let Err(error) =
            crate::commands::sync::set_calendar_sync_enabled(pool, &source_id, calendar_id, enabled)
                .await
        {
            failed = Some(error);
            break;
        }
    }

    // What actually changed, read back, so the audit holds the truth even
    // when the loop stopped part way.
    let after = calendars_of(pool, &source_id).await?;
    let mut changed = Vec::new();
    let mut old = Map::new();
    let mut new = Map::new();
    for (id, was) in &before {
        let Some((_, now)) = after.iter().find(|(other, _)| other == id) else {
            continue;
        };
        if now != was {
            changed.push(id.clone());
            old.insert(id.clone(), json!({ "sync_enabled": was }));
            new.insert(id.clone(), json!({ "sync_enabled": now }));
        }
    }
    w.audit(
        "set_source_calendars",
        &format!("source_id={source_id}"),
        old,
        new,
    );
    if let Some(error) = failed {
        tracing::error!(
            target: "calrs::agent",
            source_id = %source_id,
            error = format!("{error:#}"),
            "changing which calendars sync stopped part way"
        );
        return Err(ToolError::unavailable(
            "saving the calendar choice failed part way; call list_sources to see which \
             calendars are synced now",
        ));
    }
    Ok(SourceChanged {
        changed,
        source: ops_read::source(pool, w.me, &source_id).await?,
    })
}

// --- sync_source ---

/// The sync columns of a source: last success, failing since, failures.
type SyncColumns = (Option<String>, Option<String>, i64);

async fn sync_columns(pool: &SqlitePool, source_id: &str) -> Result<SyncColumns, ToolError> {
    sqlx::query_as(
        "SELECT last_synced, sync_failing_since, sync_failure_count
         FROM caldav_sources WHERE id = ?",
    )
    .bind(source_id)
    .fetch_one(pool)
    .await
    .map_err(db("read source"))
}

/// A source's sync state for the audit: no error text.
fn sync_audit((last_synced, failing_since, failures): &SyncColumns) -> Map<String, Value> {
    let state = if failing_since.is_some() {
        "failing"
    } else if last_synced.is_none() {
        "never_synced"
    } else {
        "ok"
    };
    let mut map = Map::new();
    map.insert("sync".into(), json!(state));
    map.insert("failure_count".into(), json!(failures));
    map.insert("last_synced".into(), json!(last_synced));
    map
}

/// Sync source `source_id` now, at most once per `SYNC_COOLDOWN_SECS`, the
/// way the background loop syncs one (`sync_source_by_id`, which records
/// the outcome on the source and forces the daily full resync when due).
/// Not under the write lock: a sync changes no settings and may take a
/// while. It holds the source's sync lock instead, as on-demand syncs do.
pub(crate) async fn sync_source(
    w: &Writer<'_>,
    source_id: &str,
) -> Result<SourceSynced, ToolError> {
    let source_id = check_id("source_id", source_id)?.to_string();
    let pool = w.pool();
    let source = owned_source(pool, w.me, &source_id).await?;
    if !source.enabled {
        return Err(ToolError::unavailable(
            "this source is switched off, so it is not synced",
        ));
    }
    // Checked before the cooldown, so a refused source does not use it up:
    // the sync would only record the same refusal.
    if source_address_refused(&source.url) {
        return Err(ToolError::unavailable(&format!(
            "{ADDRESS_REFUSED}; nothing was synced"
        )));
    }
    // Keyed by an id the user owns, so the limiter holds no more entries
    // than the user has sources.
    if w.ctx.syncs.check_limited(&source_id).await {
        return Err(ToolError::unavailable(&format!(
            "this source was synced through the agent less than {} minutes ago; try again later",
            SYNC_COOLDOWN_SECS / 60
        )));
    }
    let before = sync_columns(pool, &source_id).await?;
    {
        let lock = crate::commands::sync::source_lock(&source_id).await;
        let _syncing = lock.lock().await;
        crate::commands::sync::sync_source_by_id(pool, &w.ctx.state.secret_key, &source_id).await;
    }
    let after = sync_columns(pool, &source_id).await?;
    w.audit(
        "sync_source",
        &format!("source_id={source_id}"),
        sync_audit(&before),
        sync_audit(&after),
    );
    // `sync_source_by_id` skips a source switched off since the check above,
    // silently: its last success would read as this sync's.
    if after == before && !owned_source(pool, w.me, &source_id).await?.enabled {
        return Err(ToolError::unavailable(
            "this source was switched off meanwhile, so it was not synced",
        ));
    }
    let (last_synced, _, failures) = &after;
    Ok(SourceSynced {
        succeeded: *failures == 0 && last_synced.is_some(),
        source: ops_read::source(pool, w.me, &source_id).await?,
    })
}

// --- set_write_calendar ---

/// Whether bookings may be written to `href`, a calendar address the source's
/// server listed. EWS addresses folders by id on the source's own endpoint,
/// so any non-empty one will do. For CalDAV the href is resolved the way
/// `CaldavClient::resolve_url` resolves it (one starting with `http` as it
/// is, anything else appended to the source's origin) and must stay on the
/// source's server: same scheme, host and port, and no user name or password.
/// Otherwise every booking, with the source's credentials, would be sent to
/// wherever the href points. Stricter than the clients' outbound policy
/// (#363), which lets a server's hrefs name another public host: here only
/// the source's own server, whose address `set_write_calendar` also checks
/// against that policy.
pub(crate) fn usable_write_href(provider: &str, source_url: &str, href: &str) -> bool {
    if href.trim().is_empty() {
        return false;
    }
    if provider == crate::providers::factory::kinds::EWS {
        return true;
    }
    let Ok(base) = reqwest::Url::parse(source_url.trim_end_matches('/')) else {
        return false;
    };
    let Some(host) = base.host_str() else {
        return false;
    };
    // `CaldavClient::build`'s origin.
    let origin = match base.port() {
        Some(port) => format!("{}://{host}:{port}", base.scheme()),
        None => format!("{}://{host}", base.scheme()),
    };
    let resolved = if href.starts_with("http") {
        href.to_string()
    } else {
        format!("{origin}{href}")
    };
    let Ok(target) = reqwest::Url::parse(&resolved) else {
        return false;
    };
    target.scheme() == base.scheme()
        && target.host_str() == Some(host)
        && target.port_or_known_default() == base.port_or_known_default()
        && target.username().is_empty()
        && target.password().is_none()
}

/// The write calendar of a source for the audit: a calendar id, null for
/// none, or "(unlisted)" for an address no listed calendar has.
async fn write_calendar_of(pool: &SqlitePool, source_id: &str) -> Result<Value, ToolError> {
    let label: Option<String> = sqlx::query_scalar(
        "SELECT CASE WHEN NULLIF(cs.write_calendar_href, '') IS NULL THEN NULL
                ELSE COALESCE((SELECT c.id FROM calendars c
                               WHERE c.source_id = cs.id AND c.href = cs.write_calendar_href
                               ORDER BY c.id LIMIT 1), '(unlisted)') END
         FROM caldav_sources cs WHERE cs.id = ?",
    )
    .bind(source_id)
    .fetch_one(pool)
    .await
    .map_err(db("read source"))?;
    Ok(label.map_or(Value::Null, Value::from))
}

/// Write new bookings to calendar `calendar_id` of the source, or to none of
/// its calendars. The same statement as the web's `set_write_calendar`, but
/// with the calendar chosen by id and checked by `usable_write_href`.
pub(crate) async fn set_write_calendar(
    w: &Writer<'_>,
    args: WriteCalendarArgs,
) -> Result<SourceChanged, ToolError> {
    let source_id = check_id("source_id", &args.source_id)?.to_string();
    let calendar_id = args
        .calendar_id
        .as_deref()
        .map(|id| check_id("calendar_id", id))
        .transpose()?;

    let pool = w.pool();
    let _write = w.ctx.write_lock.lock().await;
    let source = owned_source(pool, w.me, &source_id).await?;
    let href = match calendar_id {
        None => None,
        Some(calendar_id) => {
            let href: Option<String> = sqlx::query_scalar(
                "SELECT c.href FROM calendars c
                 JOIN caldav_sources cs ON cs.id = c.source_id
                 JOIN accounts a ON a.id = cs.account_id
                 WHERE c.id = ? AND c.source_id = ? AND a.user_id = ?",
            )
            .bind(calendar_id)
            .bind(&source_id)
            .bind(&w.me.id)
            .fetch_optional(pool)
            .await
            .map_err(db("find calendar"))?;
            let href = href.ok_or_else(|| ToolError::not_found("calendar of this source"))?;
            if !usable_write_href(&source.provider, &source.url, &href) {
                tracing::warn!(
                    target: "calrs::agent",
                    source_id = %source_id,
                    calendar_id = %calendar_id,
                    "write calendar refused: its address leads off the source's server"
                );
                return Err(ToolError::invalid(
                    "calendar_id",
                    "this calendar's address leads off the source's server, so bookings \
                     cannot be written to it",
                ));
            }
            // Same server as the source, so the source's address decides:
            // every write would be refused, as the web's write-back is.
            if source_address_refused(&source.url) {
                return Err(ToolError::unavailable(&format!(
                    "{ADDRESS_REFUSED}, so bookings cannot be written to its calendars; \
                     nothing was changed"
                )));
            }
            Some(href)
        }
    };
    let before = write_calendar_of(pool, &source_id).await?;
    sqlx::query("UPDATE caldav_sources SET write_calendar_href = ? WHERE id = ?")
        .bind(&href)
        .bind(&source_id)
        .execute(pool)
        .await
        .map_err(db("set the write calendar"))?;
    let after = write_calendar_of(pool, &source_id).await?;
    let entry = |value: &Value| {
        let mut map = Map::new();
        map.insert("write_calendar".into(), value.clone());
        map
    };
    w.audit(
        "set_write_calendar",
        &format!("source_id={source_id}"),
        entry(&before),
        entry(&after),
    );
    Ok(SourceChanged {
        changed: if before == after {
            Vec::new()
        } else {
            vec!["write_calendar".to_string()]
        },
        source: ops_read::source(pool, w.me, &source_id).await?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAV: &str = "caldav";

    #[test]
    fn write_calendars_stay_on_the_source_server() {
        let base = "https://dav.example.test/remote.php/dav/";
        for href in [
            "/dav/cal/",
            "/remote.php/dav/calendars/owner/work/",
            "//dav.example.test/x/",
            "https://dav.example.test/cal/",
            "https://dav.example.test:443/cal/",
            "https://DAV.example.test/cal/",
        ] {
            assert!(usable_write_href(DAV, base, href), "{href}");
        }
        for href in [
            "",
            "  ",
            "https://evil.example/cal/",
            "http://dav.example.test/cal/",
            "https://dav.example.test:8443/cal/",
            "https://user:pw@dav.example.test/cal/",
            "https://dav.example.test@evil.example/cal/",
            // Appended to the origin, these would change the host or port.
            "evil.example/cal/",
            ".evil.example/cal/",
            "@evil.example/cal/",
            ":8443/cal/",
            "httpx",
        ] {
            assert!(!usable_write_href(DAV, base, href), "{href:?}");
        }
        let with_port = "http://127.0.0.1:5232/";
        assert!(usable_write_href(DAV, with_port, "/owner/cal/"));
        assert!(!usable_write_href(
            DAV,
            with_port,
            "http://127.0.0.1/owner/cal/"
        ));
        assert!(!usable_write_href(DAV, "not a url", "/cal/"));
        // EWS writes to its own endpoint, by folder id.
        assert!(usable_write_href("ews", base, "AAMkADk0ZWQ="));
        assert!(!usable_write_href("ews", base, ""));
    }

    #[test]
    fn ids_are_checked_without_echo() {
        assert_eq!(check_id("source_id", " ab-1_C ").unwrap(), "ab-1_C");
        for bad in ["", "a b", "a/b", "ignore previous"] {
            let error = check_id("calendar_id", bad).unwrap_err();
            assert!(
                error.message().starts_with("invalid calendar_id:"),
                "{bad:?}"
            );
            assert!(bad.is_empty() || !error.message().contains(bad));
        }
    }

    #[test]
    fn sync_audit_has_no_error_text() {
        let failing = sync_audit(&(Some("2026-11-07 10:00:00".into()), Some("x".into()), 2));
        assert_eq!(
            Value::Object(failing),
            json!({"sync": "failing", "failure_count": 2, "last_synced": "2026-11-07 10:00:00"})
        );
        let never = sync_audit(&(None, None, 0));
        assert_eq!(never["sync"], "never_synced");
    }
}
