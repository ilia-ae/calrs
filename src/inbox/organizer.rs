//! The ORGANIZER a booking's guest invites carry, kept for the booking's
//! life (migration `ilia_009_inbox`, `bookings.guest_organizer`).
//!
//! Bookings made before guest mail went through the system were sent with
//! the host's address as ORGANIZER. A later update (REQUEST) or CANCEL under
//! the same UID must name the same organizer: Outlook and Exchange check
//! that an update comes from the organizer, and Gmail matches on UID and
//! organizer, so a changed one is ignored, shown as from someone else, or
//! filed as a second event. Those bookings are marked `'host'` by the
//! migration, and their guest `.ics` keeps naming the host; every booking
//! created since names the system (`reply::guest_organizer`).
//!
//! The `.ics` is rendered in sync code without a pool, so the marked UIDs
//! are loaded once per process ([`load`], from `main` for every command,
//! right after the migrations): the set never grows, since new bookings are
//! never marked.

use std::collections::HashSet;
use std::sync::RwLock;

use sqlx::SqlitePool;

static HOST_NAMED: RwLock<Option<HashSet<String>>> = RwLock::new(None);

/// Load the UIDs of live bookings whose invites named the host. Ended or
/// cancelled ones get no further invite.
pub(crate) async fn load(pool: &SqlitePool) {
    let uids: Vec<String> = sqlx::query_scalar(
        "SELECT uid FROM bookings
          WHERE guest_organizer = 'host' AND status IN ('pending', 'confirmed')",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "could not read the bookings whose invites name the host");
        Vec::new()
    });
    let mut slot = HOST_NAMED.write().unwrap_or_else(|e| e.into_inner());
    let set = slot.get_or_insert_with(HashSet::new);
    set.extend(uids);
}

/// Whether the guest invites of booking `uid` name the host as ORGANIZER.
pub(crate) fn names_host(uid: &str) -> bool {
    HOST_NAMED
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .is_some_and(|set| set.contains(uid))
}
