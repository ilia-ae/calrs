//! ilia fork: writing the stored TON settings (`ton_settings`, read by
//! `store::load_settings`) and their audit trail (`ton_settings_audit`).
//! The rules (which keys, who wins, the one parser) are in
//! `crate::ton::settings`; the admin page is `settings_page`, `config apply`
//! is `crate::web::apply::ton`. Both write through [`write`], on a
//! connection the caller holds (the page's transaction, apply's run).

use chrono::{DateTime, Utc};
use sqlx::{SqliteConnection, SqlitePool};

use super::stamp;
use crate::ton::settings::{self, DbValues, Owner};

/// Audit rows are kept this long.
const AUDIT_DAYS: i64 = 366;

/// One key whose row changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Change {
    pub(crate) var: &'static str,
    /// The stored value before (`None`: no row).
    pub(crate) before: Option<String>,
    /// The stored value after (`None`: the row is removed).
    pub(crate) after: Option<String>,
    /// The owner after (`None` when removed).
    pub(crate) owner: Option<Owner>,
}

/// The rows that differ between `old` and `new` (value or owner), in the
/// order of `settings::KEYS`.
pub(crate) fn diff(old: &DbValues, new: &DbValues) -> Vec<Change> {
    settings::KEYS
        .iter()
        .filter(|var| old.0.get(**var) != new.0.get(**var))
        .map(|var| Change {
            var,
            before: old.value(var).map(str::to_string),
            after: new.value(var).map(str::to_string),
            owner: new.owner(var),
        })
        .collect()
}

/// Store `changes` and one audit row each. `actor` is the admin's user id.
/// Runs on the caller's connection, inside its transaction. The
/// `calrs::audit` log events are the caller's, once that transaction has
/// committed ([`log_changes`]): a rolled-back save or a `--dry-run` must
/// not log changes that never happened.
pub(crate) async fn write(
    conn: &mut SqliteConnection,
    changes: &[Change],
    actor: Option<&str>,
    source: Owner,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let at = stamp(now);
    for c in changes {
        match (&c.after, c.owner) {
            (Some(value), Some(owner)) => {
                sqlx::query(
                    "INSERT INTO ton_settings (key, value, owner, updated_at, updated_by)
                     VALUES (?, ?, ?, ?, ?)
                     ON CONFLICT (key) DO UPDATE SET value = excluded.value,
                         owner = excluded.owner, updated_at = excluded.updated_at,
                         updated_by = excluded.updated_by",
                )
                .bind(c.var)
                .bind(value)
                .bind(owner.as_str())
                .bind(&at)
                .bind(actor)
                .execute(&mut *conn)
                .await?;
            }
            _ => {
                sqlx::query("DELETE FROM ton_settings WHERE key = ?")
                    .bind(c.var)
                    .execute(&mut *conn)
                    .await?;
            }
        }
        sqlx::query(
            "INSERT INTO ton_settings_audit (at, actor_user_id, source, key, before, after, owner)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&at)
        .bind(actor)
        .bind(source.as_str())
        .bind(c.var)
        .bind(&c.before)
        .bind(&c.after)
        .bind(c.owner.map(Owner::as_str))
        .execute(&mut *conn)
        .await?;
    }
    sqlx::query("DELETE FROM ton_settings_audit WHERE at < ?")
        .bind(stamp(now - chrono::Duration::days(AUDIT_DAYS)))
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// One `calrs::audit` event per change, after the transaction that
/// [`write`] ran in has committed. No secret is ever a stored setting, so
/// the values can be logged.
pub(crate) fn log_changes(changes: &[Change], actor: Option<&str>, source: Owner) {
    for c in changes {
        tracing::info!(
            target: "calrs::audit",
            source = source.as_str(),
            actor = actor.unwrap_or(""),
            key = c.var,
            before = c.before.as_deref().unwrap_or("(default)"),
            after = c.after.as_deref().unwrap_or("(default)"),
            owner = c.owner.map(Owner::as_str).unwrap_or(""),
            "ton setting changed"
        );
    }
}

/// One audit row as the page shows it.
#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct AuditRow {
    pub(crate) at: String,
    /// The admin's email, or empty when the user is gone.
    pub(crate) actor: String,
    pub(crate) source: String,
    pub(crate) field: String,
    pub(crate) before: Option<String>,
    pub(crate) after: Option<String>,
    pub(crate) owner: Option<String>,
}

/// The newest `limit` changes, values in the page's units.
pub(crate) async fn recent(pool: &SqlitePool, limit: i64) -> Vec<AuditRow> {
    let rows: Vec<(
        String,
        Option<String>,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT a.at, u.email, a.source, a.key, a.before, a.after, a.owner
         FROM ton_settings_audit a LEFT JOIN users u ON u.id = a.actor_user_id
         ORDER BY a.id DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .map(|(at, actor, source, var, before, after, owner)| {
            let spec = settings::by_var(&var);
            let human = |v: Option<String>| match (spec, v) {
                (Some(k), Some(v)) => Some(settings::to_human(k, &v)),
                (_, v) => v,
            };
            AuditRow {
                at,
                actor: actor.unwrap_or_default(),
                source,
                field: spec.map_or(var.clone(), |k| k.field.to_string()),
                before: human(before),
                after: human(after),
                owner,
            }
        })
        .collect()
}
