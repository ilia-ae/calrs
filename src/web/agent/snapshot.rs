//! A snapshot of everything that configures one event type: its row and its
//! rows in the child tables, with child row ids left out. Its hash is the
//! version token `get_event_type` returns, so a write can refuse to apply on
//! top of a change made in between (from the web, say), and a test can prove
//! that an empty edit changes nothing.
//!
//! Columns are read from the schema at run time, so a column upstream adds is
//! covered without a change here. A new child table is not: the guard test in
//! `web::tests::agent` fails until it is listed below.

use sha2::{Digest, Sha256};
use sqlx::{SqliteConnection, SqlitePool};

/// Tables with an `event_type_id` column that hold the event type's
/// configuration.
pub(crate) const CHILD_TABLES: &[&str] = &[
    "availability_rules",
    "availability_overrides",
    "event_type_calendars",
    "booking_frequency_limits",
    "event_type_resources",
    "event_type_watchers",
    "event_type_member_weights",
    // ilia fork: the price (docs/ton.md); setting it changes the version.
    "ton_prices",
    // ilia fork: the guest Telegram field (migration ilia_002_telegram).
    "telegram_event_types",
    // ilia fork: the titles and descriptions per language (`event_texts`).
    "event_type_texts",
];

/// Tables with an `event_type_id` column that are not configuration: what
/// guests and users did with the event type.
pub(crate) const NOT_CONFIG_TABLES: &[&str] = &[
    "bookings",
    "booking_invites",
    "event_type_favorites",
    // ilia fork: money records of bookings, not configuration.
    "ton_invoices",
    // ilia fork: free demand-ladder steps taken (the per-guest limit).
    "ton_free_bookings",
];

/// The configuration of one event type, as canonical text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Snapshot {
    /// The `event_types` row.
    pub(crate) event_type: String,
    /// Per child table (in `CHILD_TABLES` order), its rows, sorted.
    pub(crate) children: Vec<(&'static str, Vec<String>)>,
}

/// Separates values in a row (ASCII unit separator).
const UNIT: &str = "char(31)";

/// Read the snapshot of event type `et_id`. `None` when there is no such row.
pub(crate) async fn snapshot(
    pool: &SqlitePool,
    et_id: &str,
) -> Result<Option<Snapshot>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    snapshot_on(&mut conn, et_id).await
}

/// `snapshot` on one connection, so a write can check the version inside the
/// transaction that then changes the rows.
pub(crate) async fn snapshot_on(
    conn: &mut SqliteConnection,
    et_id: &str,
) -> Result<Option<Snapshot>, sqlx::Error> {
    let Some(event_type) = rows(conn, "event_types", "id", et_id, false)
        .await?
        .into_iter()
        .next()
    else {
        return Ok(None);
    };
    let mut children = Vec::with_capacity(CHILD_TABLES.len());
    for table in CHILD_TABLES {
        let mut found = rows(conn, table, "event_type_id", et_id, true).await?;
        found.sort();
        children.push((*table, found));
    }
    Ok(Some(Snapshot {
        event_type,
        children,
    }))
}

/// The rows of `table` where `key = value`, each as its columns' SQL literals
/// (`quote()`) joined by the unit separator. `skip_id` leaves the `id` column
/// out. `table` and `key` come from the constants above, never from input.
async fn rows(
    conn: &mut SqliteConnection,
    table: &str,
    key: &str,
    value: &str,
    skip_id: bool,
) -> Result<Vec<String>, sqlx::Error> {
    let columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info(?) ORDER BY cid")
            .bind(table)
            .fetch_all(&mut *conn)
            .await?;
    let expr = columns
        .iter()
        .filter(|c| !(skip_id && c.as_str() == "id"))
        .map(|c| format!("quote(\"{}\")", c.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(&format!(" || {UNIT} || "));
    if expr.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_scalar(&format!(
        "SELECT {expr} FROM \"{table}\" WHERE \"{key}\" = ?"
    ))
    .bind(value)
    .fetch_all(&mut *conn)
    .await
}

/// `v1-` and the first 16 hex digits of the snapshot's SHA-256.
pub(crate) fn version_token(snapshot: &Snapshot) -> String {
    let mut hash = Sha256::new();
    hash.update(b"event_types\x1e");
    hash.update(snapshot.event_type.as_bytes());
    for (table, rows) in &snapshot.children {
        hash.update(b"\x1d");
        hash.update(table.as_bytes());
        for row in rows {
            hash.update(b"\x1e");
            hash.update(row.as_bytes());
        }
    }
    let digest = hex::encode(hash.finalize());
    format!("v1-{}", &digest[..16])
}
