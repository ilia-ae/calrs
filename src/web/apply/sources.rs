//! ilia fork: `config apply`, the `sources` section: which calendar of a
//! source bookings are written to, and which calendars are synced
//! (docs/apply-schema.md).
//!
//! A source must already exist among the admin's own (the role adds it with
//! `calrs source add`), matched on `url` and `username` exactly. Calendars
//! are named by their display name (their address when they have none), as
//! the local `calendars` rows list them: nothing here connects to the
//! calendar server, so no login is ever tried.
//!
//! The writes are the ones the web and MCP make:
//! - the name (ilia fork) is the edit form's column (`UPDATE caldav_sources
//!   SET name`), written directly: the form's handler would log in to the
//!   server to test the connection (Kerio locks an account after failed
//!   logins), and the name needs no connection;
//! - the write calendar is the web's statement (`set_write_calendar`,
//!   `UPDATE caldav_sources SET write_calendar_href`), with MCP's check that
//!   the calendar's address stays on the source's server
//!   (`ops_sources::usable_write_href`);
//! - the synced calendars go through `commands::sync::set_calendar_sync_enabled`
//!   one calendar at a time, on first, then off, as MCP's
//!   `set_source_calendars` does, which also drops a calendar's cached
//!   events when it is turned off.
//!
//! MCP also refuses a write calendar when calrs would refuse to connect to
//! the source's address (a private address not on the allowlist). That
//! check reads the allowlist of the running server (environment and
//! settings), which this command may not have, and the clients apply it
//! again at every request, so it is left to them.

use super::{check_text, ApplyError, Ctx, Source};
use serde_json::Value;
use std::collections::BTreeSet;

/// The write calendar's name in a diff when its address is not one of the
/// source's listed calendars (as MCP's audit says it).
const UNLISTED: &str = "(unlisted)";

/// The most bytes of a source's name (the column has no limit; the role and
/// the form agree on this one).
const NAME_MAX: usize = 255;

/// ilia fork: what a source's `name` is. The role greps for this literal in
/// the binary before it sends the key (an older calrs refuses it as an
/// unknown key): keep it a literal, and used at runtime (the refusal of a
/// name that is empty, padded or too long).
pub(crate) const SOURCE_NAME_SHAPE: &str = "sources[].name: 1..255 bytes, not padded";

fn at(index: usize, key: &str) -> String {
    format!("sources[{index}].{key}")
}

/// A source as stored.
struct Stored {
    id: String,
    name: String,
    url: String,
    provider: String,
    write_href: Option<String>,
    /// By name.
    calendars: Vec<Calendar>,
}

struct Calendar {
    id: String,
    name: String,
    href: String,
    synced: bool,
}

impl Stored {
    async fn load(ctx: &Ctx, id: &str) -> Result<Self, ApplyError> {
        let (name, url, provider, write_href): (String, String, String, Option<String>) =
            sqlx::query_as(
                "SELECT name, url, provider_type, NULLIF(write_calendar_href, '')
                 FROM caldav_sources WHERE id = ?",
            )
            .bind(id)
            .fetch_one(ctx.pool())
            .await?;
        let rows: Vec<(String, Option<String>, String, i64)> = sqlx::query_as(
            "SELECT id, display_name, href, sync_enabled FROM calendars
             WHERE source_id = ? ORDER BY id",
        )
        .bind(id)
        .fetch_all(ctx.pool())
        .await?;
        let mut calendars: Vec<Calendar> = rows
            .into_iter()
            .map(|(id, display_name, href, synced)| Calendar {
                id,
                name: display_name
                    .map(|n| n.trim().to_string())
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| href.clone()),
                href,
                synced: synced != 0,
            })
            .collect();
        calendars.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
        Ok(Self {
            id: id.to_string(),
            name,
            url,
            provider,
            write_href,
            calendars,
        })
    }

    /// The one calendar named `name`, or why there is none.
    fn named(&self, path: &str, name: &str) -> Result<&Calendar, ApplyError> {
        let found: Vec<&Calendar> = self.calendars.iter().filter(|c| c.name == name).collect();
        match found.as_slice() {
            [calendar] => Ok(calendar),
            [] => Err(ApplyError::invalid(
                path,
                format!("no calendar named {name:?}; {}", self.available()),
            )),
            several => Err(ApplyError::invalid(
                path,
                format!(
                    "{} calendars of this source are named {name:?}: rename one in the calendar \
                     server, then sync the source",
                    several.len()
                ),
            )),
        }
    }

    /// The names the source has, for an error.
    fn available(&self) -> String {
        if self.calendars.is_empty() {
            return "this source has listed no calendars yet: sync it once first".to_string();
        }
        let names: Vec<String> = self.names().iter().map(|n| format!("{n:?}")).collect();
        format!("this source has {}", names.join(", "))
    }

    /// Every calendar name, once each, sorted.
    fn names(&self) -> BTreeSet<&str> {
        self.calendars.iter().map(|c| c.name.as_str()).collect()
    }

    /// Whether a name belongs to more than one calendar.
    fn ambiguous(&self, name: &str) -> bool {
        self.calendars.iter().filter(|c| c.name == name).count() > 1
    }

    /// The names of the calendars synced, sorted.
    fn synced(&self) -> Vec<String> {
        self.calendars
            .iter()
            .filter(|c| c.synced)
            .map(|c| c.name.clone())
            .collect()
    }

    /// The write calendar as the diff shows it: its name, null for none,
    /// `UNLISTED` for an address no listed calendar has.
    fn write_name(&self) -> Value {
        match &self.write_href {
            None => Value::Null,
            Some(href) => self
                .calendars
                .iter()
                .find(|c| &c.href == href)
                .map_or(Value::from(UNLISTED), |c| Value::from(c.name.clone())),
        }
    }
}

/// A source's name as the edit form stores it: not empty, not padded, at
/// most `NAME_MAX` bytes.
fn check_name(index: usize, name: &str) -> Result<(), ApplyError> {
    if name.is_empty() || name.trim() != name || name.len() > NAME_MAX {
        return Err(ApplyError::invalid(at(index, "name"), SOURCE_NAME_SHAPE));
    }
    Ok(())
}

/// The admin's source `url` + `username`.
async fn find(ctx: &Ctx, index: usize, source: &Source) -> Result<String, ApplyError> {
    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT cs.id FROM caldav_sources cs JOIN accounts a ON a.id = cs.account_id
         WHERE a.user_id = ? AND cs.url = ? AND cs.username = ?
         ORDER BY cs.created_at, cs.id",
    )
    .bind(&ctx.me.id)
    .bind(&source.url)
    .bind(&source.username)
    .fetch_all(ctx.pool())
    .await?;
    match ids.as_slice() {
        [id] => Ok(id.clone()),
        [] => Err(ApplyError::invalid(
            format!("sources[{index}]"),
            format!(
                "{} has no calendar source with this url and username (add it first)",
                ctx.me.email
            ),
        )),
        several => Err(ApplyError::invalid(
            format!("sources[{index}]"),
            format!(
                "{} sources of {} have this url and username",
                several.len(),
                ctx.me.email
            ),
        )),
    }
}

/// The ids of the calendars `names` asks to sync.
fn wanted_synced(
    stored: &Stored,
    index: usize,
    names: &[String],
) -> Result<BTreeSet<String>, ApplyError> {
    names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            stored
                .named(&at(index, &format!("sync_calendars[{i}]")), name)
                .map(|c| c.id.clone())
        })
        .collect()
}

/// The address `write_calendar` asks for.
fn wanted_href(
    stored: &Stored,
    index: usize,
    name: Option<&str>,
) -> Result<Option<String>, ApplyError> {
    let Some(name) = name else {
        return Ok(None);
    };
    let path = at(index, "write_calendar");
    let calendar = stored.named(&path, name)?;
    if !crate::web::agent::ops_sources::usable_write_href(
        &stored.provider,
        &stored.url,
        &calendar.href,
    ) {
        return Err(ApplyError::invalid(
            path,
            "this calendar's address leads off the source's server, so bookings cannot be \
             written to it",
        ));
    }
    Ok(Some(calendar.href.clone()))
}

/// Every check of the section, reading only (pipeline step 1).
pub(super) async fn check(ctx: &Ctx, section: &[Source]) -> Result<(), ApplyError> {
    let mut seen = BTreeSet::new();
    for (index, source) in section.iter().enumerate() {
        check_text(&at(index, "url"), &source.url, None)?;
        check_text(&at(index, "username"), &source.username, None)?;
        if !seen.insert((source.url.as_str(), source.username.as_str())) {
            return Err(ApplyError::invalid(
                format!("sources[{index}]"),
                "listed twice",
            ));
        }
        if let Some(name) = &source.name {
            check_name(index, name)?;
        }
        let stored = Stored::load(ctx, &find(ctx, index, source).await?).await?;
        if let Some(name) = &source.write_calendar {
            if let Some(name) = name {
                check_text(&at(index, "write_calendar"), name, None)?;
            }
            wanted_href(&stored, index, name.as_deref())?;
        }
        if let Some(names) = &source.sync_calendars {
            if names.is_empty() {
                return Err(ApplyError::invalid(
                    at(index, "sync_calendars"),
                    "keep at least one calendar synced: with none, every time would look free",
                ));
            }
            let mut listed = BTreeSet::new();
            for (i, name) in names.iter().enumerate() {
                let path = at(index, &format!("sync_calendars[{i}]"));
                check_text(&path, name, None)?;
                if !listed.insert(name.as_str()) {
                    return Err(ApplyError::invalid(path, "listed twice"));
                }
            }
            wanted_synced(&stored, index, names)?;
        }
    }
    Ok(())
}

/// Write what differs and record it (pipeline step 3).
pub(super) async fn apply(ctx: &mut Ctx, section: &[Source]) -> Result<(), ApplyError> {
    for (index, source) in section.iter().enumerate() {
        let stored = Stored::load(ctx, &find(ctx, index, source).await?).await?;

        if let Some(name) = &source.name {
            if name != &stored.name {
                ctx.record(at(index, "name"), &stored.name, name);
                sqlx::query("UPDATE caldav_sources SET name = ? WHERE id = ?")
                    .bind(name)
                    .bind(&stored.id)
                    .execute(ctx.pool())
                    .await?;
            }
        }

        if let Some(name) = &source.write_calendar {
            let href = wanted_href(&stored, index, name.as_deref())?;
            if href != stored.write_href {
                ctx.record(at(index, "write_calendar"), stored.write_name(), name);
                sqlx::query("UPDATE caldav_sources SET write_calendar_href = ? WHERE id = ?")
                    .bind(&href)
                    .bind(&stored.id)
                    .execute(ctx.pool())
                    .await?;
            }
        }

        if let Some(names) = &source.sync_calendars {
            let wanted = wanted_synced(&stored, index, names)?;
            let now: BTreeSet<String> = stored
                .calendars
                .iter()
                .filter(|c| c.synced)
                .map(|c| c.id.clone())
                .collect();
            if wanted == now {
                continue;
            }
            let mut after: Vec<&str> = names.iter().map(String::as_str).collect();
            after.sort_unstable();
            ctx.record(at(index, "sync_calendars"), stored.synced(), after);
            // On first, then off: while this runs the source never syncs
            // fewer calendars than either choice.
            let turn_on = stored
                .calendars
                .iter()
                .filter(|c| !c.synced && wanted.contains(&c.id))
                .map(|c| (c.id.as_str(), true));
            let turn_off = stored
                .calendars
                .iter()
                .filter(|c| c.synced && !wanted.contains(&c.id))
                .map(|c| (c.id.as_str(), false));
            for (calendar_id, enabled) in turn_on.chain(turn_off) {
                crate::commands::sync::set_calendar_sync_enabled(
                    ctx.pool(),
                    &stored.id,
                    calendar_id,
                    enabled,
                )
                .await
                .map_err(|e| ApplyError::internal("set which calendars sync", format!("{e:#}")))?;
            }
        }
    }
    Ok(())
}

/// Read back and compare with the document (pipeline step 4).
pub(super) async fn verify(ctx: &Ctx, section: &[Source]) -> Result<(), ApplyError> {
    for (index, source) in section.iter().enumerate() {
        let stored = Stored::load(ctx, &find(ctx, index, source).await?).await?;
        if let Some(name) = &source.name {
            if name != &stored.name {
                return Err(ApplyError::stored_differs(
                    at(index, "name"),
                    &Value::from(name.clone()),
                    &Value::from(stored.name.clone()),
                ));
            }
        }
        if let Some(name) = &source.write_calendar {
            if wanted_href(&stored, index, name.as_deref())? != stored.write_href {
                return Err(ApplyError::stored_differs(
                    at(index, "write_calendar"),
                    &Value::from(name.clone()),
                    &stored.write_name(),
                ));
            }
        }
        if let Some(names) = &source.sync_calendars {
            let wanted = wanted_synced(&stored, index, names)?;
            let now: BTreeSet<String> = stored
                .calendars
                .iter()
                .filter(|c| c.synced)
                .map(|c| c.id.clone())
                .collect();
            if wanted != now {
                return Err(ApplyError::stored_differs(
                    at(index, "sync_calendars"),
                    &Value::from(names.clone()),
                    &Value::from(stored.synced()),
                ));
            }
        }
    }
    Ok(())
}

/// The stored state in the document's shape (`--export`), reading only:
/// every source of the admin, oldest first. A key is left out where the
/// document cannot say what is stored: a name the edit form would not
/// store (empty, padded, too long: a hand edit), a write calendar whose
/// address no listed calendar has, a name two calendars share, or a source
/// that syncs no calendar.
pub(super) async fn export(ctx: &Ctx) -> Result<Option<Vec<Source>>, ApplyError> {
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT cs.id, cs.url, cs.username FROM caldav_sources cs
         JOIN accounts a ON a.id = cs.account_id
         WHERE a.user_id = ? ORDER BY cs.created_at, cs.id",
    )
    .bind(&ctx.me.id)
    .fetch_all(ctx.pool())
    .await?;
    let mut sources = Vec::with_capacity(rows.len());
    for (id, url, username) in rows {
        let stored = Stored::load(ctx, &id).await?;
        let write_calendar = match stored.write_name() {
            Value::Null => Some(None),
            Value::String(name) if name != UNLISTED && !stored.ambiguous(&name) => Some(Some(name)),
            _ => None,
        };
        let synced = stored.synced();
        let sync_calendars =
            (!synced.is_empty() && !synced.iter().any(|n| stored.ambiguous(n))).then_some(synced);
        let name = check_name(0, &stored.name)
            .is_ok()
            .then(|| stored.name.clone());
        sources.push(Source {
            url,
            username,
            name,
            write_calendar,
            sync_calendars,
        });
    }
    Ok(Some(sources))
}
