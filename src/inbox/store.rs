//! The inbox's SQL: where the poller stopped, and the messages it stores
//! (`booking_messages`, via `web::messages::store_guest_message`).

use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

/// Where the poller stopped in a folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct State {
    pub(crate) uidvalidity: u32,
    pub(crate) last_uid: u32,
}

pub(crate) async fn state(pool: &SqlitePool, folder: &str) -> Option<State> {
    let row: Option<(i64, i64)> =
        sqlx::query_as("SELECT uidvalidity, last_uid FROM inbox_state WHERE folder = ?")
            .bind(folder)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    row.map(|(v, l)| State {
        uidvalidity: v as u32,
        last_uid: l as u32,
    })
}

pub(crate) async fn save_state(pool: &SqlitePool, folder: &str, state: State) -> bool {
    sqlx::query(
        "INSERT INTO inbox_state (folder, uidvalidity, last_uid, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT (folder) DO UPDATE SET uidvalidity = ?2, last_uid = ?3,
                                            updated_at = datetime('now')",
    )
    .bind(folder)
    .bind(i64::from(state.uidvalidity))
    .bind(i64::from(state.last_uid))
    .execute(pool)
    .await
    .is_ok()
}

/// The dedupe key of a message: its Message-ID, normalised; without one,
/// the raw message.
pub(crate) fn inbound_hash(message_id: Option<&str>, raw: &[u8]) -> String {
    let mut h = Sha256::new();
    match message_id
        .map(|id| id.trim().trim_matches(['<', '>']).to_ascii_lowercase())
        .filter(|id| !id.is_empty())
    {
        Some(id) => {
            h.update(b"id\0");
            h.update(id.as_bytes());
        }
        None => {
            h.update(b"raw\0");
            h.update(raw);
        }
    }
    hex::encode(h.finalize())
}

/// Whether this message is already in the booking's thread.
pub(crate) async fn seen(pool: &SqlitePool, booking_id: &str, hash: &str) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM booking_messages WHERE booking_id = ? AND inbound_hash = ?)",
    )
    .bind(booking_id)
    .bind(hash)
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
    .unwrap_or(false)
}
