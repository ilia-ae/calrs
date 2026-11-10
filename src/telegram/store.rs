//! Every SQL statement of the Telegram feature (tables: migration
//! `ilia_002_telegram.sql`).
//!
//! `chat_id` is read through `CASE WHEN typeof(chat_id) = 'integer'`, so a
//! value that is not a chat id (a test's planted sentinel) reads as "not
//! linked" rather than failing the whole row.

use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Tz;
use sqlx::SqlitePool;

use super::username::mode;

/// SQLite's `datetime('now')` format, used for every Telegram timestamp.
pub(crate) const SQL_TIME: &str = "%Y-%m-%d %H:%M:%S";

pub(crate) fn sql_time(t: DateTime<Utc>) -> String {
    t.format(SQL_TIME).to_string()
}

fn log_err<T>(what: &'static str) -> impl FnOnce(sqlx::Error) -> Option<T> {
    move |e| {
        tracing::warn!(target: "calrs::telegram", error = %e, what, "telegram store query failed");
        None
    }
}

// --- Event type mode ---

/// The guest Telegram mode of an event type (`off` when none is stored).
pub(crate) async fn event_type_mode(pool: &SqlitePool, et_id: &str) -> &'static str {
    let stored: Option<String> =
        sqlx::query_scalar("SELECT guest_mode FROM telegram_event_types WHERE event_type_id = ?")
            .bind(et_id)
            .fetch_optional(pool)
            .await
            .unwrap_or_else(log_err("read event type mode"));
    mode::parse(stored.as_deref().unwrap_or(mode::OFF))
}

pub(crate) async fn save_event_type_mode(
    pool: &SqlitePool,
    et_id: &str,
    guest_mode: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO telegram_event_types (event_type_id, guest_mode) VALUES (?, ?)
         ON CONFLICT(event_type_id) DO UPDATE SET guest_mode = excluded.guest_mode,
             updated_at = datetime('now')
         WHERE guest_mode != excluded.guest_mode",
    )
    .bind(et_id)
    .bind(mode::parse(guest_mode))
    .execute(pool)
    .await
    .map(|_| ())
}

// --- Poller state ---

pub(crate) async fn cached_bot_username(pool: &SqlitePool) -> Option<String> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT bot_username FROM telegram_state WHERE id = 'singleton'",
    )
    .fetch_optional(pool)
    .await
    .unwrap_or_else(log_err("read bot username"))
    .flatten()
}

pub(crate) async fn save_bot_username(pool: &SqlitePool, username: &str) {
    if let Err(e) = sqlx::query(
        "UPDATE telegram_state SET bot_username = ?, updated_at = datetime('now') WHERE id = 'singleton'",
    )
    .bind(username)
    .execute(pool)
    .await
    {
        tracing::warn!(target: "calrs::telegram", error = %e, "could not store the bot username");
    }
}

/// Make the stored poller state belong to bot `bot_id` (the token's numeric
/// prefix). Update ids are a per-bot sequence and Telegram drops every update
/// below the offset, so an offset kept from another bot could silence the new
/// one for good; its cached username would name the wrong bot in deep links.
/// Returns whether the state was reset.
pub(crate) async fn claim_bot(pool: &SqlitePool, bot_id: i64) -> bool {
    let stored: Option<Option<i64>> =
        sqlx::query_scalar("SELECT bot_id FROM telegram_state WHERE id = 'singleton'")
            .fetch_optional(pool)
            .await
            .unwrap_or_else(log_err("read bot id"));
    let reset = match stored.flatten() {
        Some(id) if id == bot_id => return false,
        Some(_) => true,
        // First start with a token: whatever is stored came with it.
        None => false,
    };
    let sql = if reset {
        "UPDATE telegram_state SET bot_id = ?, update_offset = 0, bot_username = NULL,
                updated_at = datetime('now') WHERE id = 'singleton'"
    } else {
        "UPDATE telegram_state SET bot_id = ?, updated_at = datetime('now') WHERE id = 'singleton'"
    };
    if let Err(e) = sqlx::query(sql).bind(bot_id).execute(pool).await {
        tracing::warn!(target: "calrs::telegram", error = %e, "could not store the bot id");
        return false;
    }
    if reset {
        tracing::info!(target: "calrs::telegram", "telegram bot changed; update offset and bot username reset");
    }
    reset
}

pub(crate) async fn load_offset(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT update_offset FROM telegram_state WHERE id = 'singleton'")
        .fetch_optional(pool)
        .await
        .unwrap_or_else(log_err("read update offset"))
        .unwrap_or(0)
}

pub(crate) async fn save_offset(pool: &SqlitePool, offset: i64) {
    if let Err(e) = sqlx::query(
        "UPDATE telegram_state SET update_offset = ?, updated_at = datetime('now') WHERE id = 'singleton'",
    )
    .bind(offset)
    .execute(pool)
    .await
    {
        tracing::warn!(target: "calrs::telegram", error = %e, "could not store the update offset");
    }
}

// --- Guest links ---

/// The booking's event type mode, its end as a UTC instant, and its status.
pub(crate) async fn booking_mode_and_end(
    pool: &SqlitePool,
    booking_id: &str,
) -> Option<(&'static str, DateTime<Utc>)> {
    let (et_id, end_at): (String, String) = sqlx::query_as(&format!(
        "SELECT event_type_id, {END_AT} FROM bookings b WHERE id = ?"
    ))
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(log_err("read booking"))?;
    let tz = crate::booking_time::event_timezone(pool, &et_id)
        .await
        .unwrap_or(Tz::UTC);
    let end = utc_instant(&end_at, tz)?;
    Some((event_type_mode(pool, &et_id).await, end))
}

/// A booking's start and end as upstream reads them: version 1 rows are RFC
/// 3339; legacy rows (`time_version` 0) hold a wall clock in the event type's
/// zone and may carry a stray `Z`, which is not UTC and is dropped.
const START_AT: &str = "CASE b.time_version WHEN 1 THEN b.start_at ELSE rtrim(b.start_at, 'Z') END";
const END_AT: &str = "CASE b.time_version WHEN 1 THEN b.end_at ELSE rtrim(b.end_at, 'Z') END";

/// A stored booking time as a UTC instant: version 1 rows are RFC 3339,
/// legacy rows (read through [`START_AT`]/[`END_AT`]) a wall clock in the
/// event type's zone.
pub(crate) fn utc_instant(stored: &str, legacy_tz: Tz) -> Option<DateTime<Utc>> {
    crate::booking_time::local(stored, legacy_tz, Tz::UTC).map(|n| n.and_utc())
}

pub(crate) async fn create_guest_link(
    pool: &SqlitePool,
    booking_id: &str,
    entered: Option<&str>,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> bool {
    sqlx::query(
        "INSERT INTO telegram_guest_links (booking_id, entered_username, link_token_hash, link_expires_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(booking_id)
    .bind(entered)
    .bind(token_hash)
    .bind(sql_time(expires_at))
    .execute(pool)
    .await
    .map_err(|e| tracing::warn!(target: "calrs::telegram", error = %e, booking_id, "could not store the Telegram link"))
    .is_ok()
}

/// The booking's link row was made with the token hashing to `token_hash`,
/// is still unexpired, and the booking is pending or confirmed.
pub(crate) async fn link_offered(pool: &SqlitePool, booking_id: &str, token_hash: &str) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM telegram_guest_links g JOIN bookings b ON b.id = g.booking_id
                         WHERE g.booking_id = ? AND g.link_token_hash = ?
                           AND g.link_expires_at > datetime('now')
                           AND b.status IN ('pending', 'confirmed'))",
    )
    .bind(booking_id)
    .bind(token_hash)
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
    .unwrap_or_else(|e| {
        tracing::warn!(target: "calrs::telegram", error = %e, "could not read a Telegram link");
        false
    })
}

/// The token hash a booking's link row answers to, while the row is
/// unexpired and the booking pending or confirmed; `None` otherwise
/// (no row: the event type offers no Telegram).
pub(crate) async fn live_link_hash(pool: &SqlitePool, booking_id: &str) -> Option<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT g.link_token_hash FROM telegram_guest_links g JOIN bookings b ON b.id = g.booking_id
          WHERE g.booking_id = ? AND g.link_token_hash IS NOT NULL
            AND g.link_expires_at > datetime('now')
            AND b.status IN ('pending', 'confirmed')",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(target: "calrs::telegram", error = %e, "could not read a Telegram link");
        None
    })
}

/// A priced booking's link row was made with another token than
/// `token_hash` (its token is derived from the payment link and the secret
/// key, and the key changed since): from now on the row answers to
/// `token_hash`, and a `/start` with the old token learns that its link was
/// replaced ([`GuestLink::Replaced`]). Only while the booking stands and
/// before the meeting ends; whichever chat used the old token keeps the row
/// (`token_chat_id`), so it can link again with the new one. `true` when the
/// row was re-keyed.
///
/// One replaced token is kept, the first: after a second key change the
/// link from the original key (the one the guest saw when booking, and
/// usually the one emailed) still gets that answer; a link from a key in
/// between is just invalid. Back to the original key (a restore), the
/// replaced one is the key it replaces again.
pub(crate) async fn rekey_guest_link(
    pool: &SqlitePool,
    booking_id: &str,
    token_hash: &str,
) -> bool {
    sqlx::query(
        "UPDATE telegram_guest_links
            SET replaced_token_hash = CASE WHEN replaced_token_hash IS NULL OR replaced_token_hash = ?2
                                           THEN link_token_hash ELSE replaced_token_hash END,
                link_token_hash = ?2
          WHERE booking_id = ?1
            AND link_token_hash IS NOT NULL AND link_token_hash <> ?2
            AND link_expires_at > datetime('now')
            AND EXISTS (SELECT 1 FROM bookings b
                         WHERE b.id = telegram_guest_links.booking_id
                           AND b.status IN ('pending', 'confirmed'))",
    )
    .bind(booking_id)
    .bind(token_hash)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or_else(|e| {
        tracing::warn!(target: "calrs::telegram", error = %e, booking_id, "could not re-key a Telegram link");
        false
    })
}

/// What a booking token did for a chat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuestLink {
    /// The chat is now linked to this booking.
    Linked(String),
    /// This chat already is: the guest opened the link a second time.
    AlreadyLinked,
    /// A priced booking's link from before the secret key changed: the
    /// payment page shows its new one ([`rekey_guest_link`]).
    Replaced,
    /// Unknown, used by another chat, expired, or the booking ended.
    Refused,
}

/// Link the chat to the booking whose unexpired token hashes to
/// `token_hash`, if that booking is still pending or confirmed.
///
/// A token belongs to the first chat that uses it (`token_chat_id`): that
/// chat may link again after `/stop` or a block, with the same link, until
/// the meeting ends; no other chat ever can. One guarded UPDATE, so two chats
/// racing the same token cannot both win.
pub(crate) async fn consume_guest_token(
    pool: &SqlitePool,
    token_hash: &str,
    chat_id: i64,
    username: Option<&str>,
) -> GuestLink {
    let linked: Option<String> = sqlx::query_scalar(
        "UPDATE telegram_guest_links
            SET chat_id = ?1, token_chat_id = ?1, telegram_username = ?2,
                linked_at = datetime('now')
          WHERE link_token_hash = ?3
            AND link_expires_at > datetime('now')
            AND chat_id IS NULL
            AND (token_chat_id IS NULL OR token_chat_id = ?1)
            AND EXISTS (SELECT 1 FROM bookings b
                         WHERE b.id = telegram_guest_links.booking_id
                           AND b.status IN ('pending', 'confirmed'))
          RETURNING booking_id",
    )
    .bind(chat_id)
    .bind(username)
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(log_err("consume booking token"));
    if let Some(booking_id) = linked {
        return GuestLink::Linked(booking_id);
    }
    let already: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM telegram_guest_links g JOIN bookings b ON b.id = g.booking_id
          WHERE g.link_token_hash = ? AND g.chat_id = ?
            AND g.link_expires_at > datetime('now')
            AND b.status IN ('pending', 'confirmed')",
    )
    .bind(token_hash)
    .bind(chat_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(log_err("read booking token"));
    if already.is_some() {
        return GuestLink::AlreadyLinked;
    }
    // A link replaced after the secret key changed, while its booking
    // stands: already on for the chat that holds it, otherwise "open the
    // payment page again" (whoever has that page gets the new link).
    let replaced: Option<Option<i64>> = sqlx::query_scalar(
        "SELECT g.chat_id FROM telegram_guest_links g JOIN bookings b ON b.id = g.booking_id
          WHERE g.replaced_token_hash = ? AND g.link_token_hash IS NOT NULL
            AND g.link_expires_at > datetime('now')
            AND b.status IN ('pending', 'confirmed')",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(log_err("read a replaced booking token"));
    match replaced {
        Some(Some(linked)) if linked == chat_id => GuestLink::AlreadyLinked,
        Some(_) => GuestLink::Replaced,
        None => GuestLink::Refused,
    }
}

/// A recomputed link expiry after the meeting moved.
pub(crate) async fn move_link_expiry(pool: &SqlitePool, booking_id: &str, end: DateTime<Utc>) {
    let _ = sqlx::query("UPDATE telegram_guest_links SET link_expires_at = ? WHERE booking_id = ?")
        .bind(sql_time(end))
        .bind(booking_id)
        .execute(pool)
        .await
        .map_err(|e| tracing::warn!(target: "calrs::telegram", error = %e, "could not move the link expiry"));
}

/// The booking ended before its time (cancelled, declined, expired): its link
/// expires now, and the purge drops it a day later.
pub(crate) async fn end_link(pool: &SqlitePool, booking_id: &str, now: DateTime<Utc>) {
    let _ = sqlx::query(
        "UPDATE telegram_guest_links SET link_expires_at = ?, link_token_hash = NULL, replaced_token_hash = NULL
          WHERE booking_id = ? AND link_expires_at > ?",
    )
    .bind(sql_time(now))
    .bind(booking_id)
    .bind(sql_time(now))
    .execute(pool)
    .await
    .map_err(|e| tracing::warn!(target: "calrs::telegram", error = %e, "could not end the link"));
}

/// The chat linked to a booking that is still pending or confirmed.
pub(crate) async fn linked_guest_chat(pool: &SqlitePool, booking_id: &str) -> Option<i64> {
    sqlx::query_scalar::<_, Option<i64>>(
        "SELECT CASE WHEN typeof(g.chat_id) = 'integer' THEN g.chat_id END
           FROM telegram_guest_links g JOIN bookings b ON b.id = g.booking_id
          WHERE g.booking_id = ? AND b.status IN ('pending', 'confirmed')",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(log_err("read guest chat"))
    .flatten()
}

/// Remove every link of a chat (`/stop`, or the user blocked the bot).
/// Returns how many links there were. A guest link keeps the chat that used
/// its token (`token_chat_id`), so the same link turns it on again.
pub(crate) async fn unlink_chat(pool: &SqlitePool, chat_id: i64) -> u64 {
    let guests = sqlx::query(
        "UPDATE telegram_guest_links SET chat_id = NULL, telegram_username = NULL
          WHERE chat_id = ?",
    )
    .bind(chat_id)
    .execute(pool)
    .await
    .map(|r| r.rows_affected())
    .unwrap_or_else(|e| {
        tracing::warn!(target: "calrs::telegram", error = %e, "could not unlink guest chats");
        0
    });
    let hosts = sqlx::query("DELETE FROM telegram_user_chats WHERE chat_id = ?")
        .bind(chat_id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or_else(|e| {
            tracing::warn!(target: "calrs::telegram", error = %e, "could not unlink host chats");
            0
        });
    guests + hosts
}

/// For the bookings dashboard: (booking id, entered username, linked
/// username, linked) of each of `ids` that has a link row.
pub(crate) async fn guest_labels(
    pool: &SqlitePool,
    ids: &[String],
) -> Vec<(String, Option<String>, Option<String>, bool)> {
    let mut out = Vec::new();
    for chunk in ids.chunks(200) {
        let marks = vec!["?"; chunk.len()].join(",");
        let sql = format!(
            "SELECT booking_id, entered_username, telegram_username,
                    typeof(chat_id) = 'integer'
               FROM telegram_guest_links WHERE booking_id IN ({marks})"
        );
        let mut query = sqlx::query_as(&sql);
        for id in chunk {
            query = query.bind(id);
        }
        let rows: Vec<(String, Option<String>, Option<String>, bool)> =
            query.fetch_all(pool).await.unwrap_or_else(|e| {
                tracing::warn!(target: "calrs::telegram", error = %e, "could not read guest links");
                Vec::new()
            });
        out.extend(rows);
    }
    out
}

// --- Host links ---

/// A new single-use host token for `user_id`, valid 15 minutes; any earlier
/// one stops working.
pub(crate) async fn new_user_token(
    pool: &SqlitePool,
    user_id: &str,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM telegram_user_link_tokens WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO telegram_user_link_tokens (token_hash, user_id, expires_at)
         VALUES (?, ?, datetime('now', '+15 minutes'))",
    )
    .bind(token_hash)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

/// Use a host token: link the chat to its (enabled) user, replacing an
/// earlier chat. Returns the user's id, name and saved language.
pub(crate) async fn consume_user_token(
    pool: &SqlitePool,
    token_hash: &str,
    chat_id: i64,
    username: Option<&str>,
) -> Option<(String, String, Option<String>)> {
    let result: Result<Option<(String, String, Option<String>)>, sqlx::Error> = async {
        let mut tx = pool.begin().await?;
        let user_id: Option<String> = sqlx::query_scalar(
            "DELETE FROM telegram_user_link_tokens
              WHERE token_hash = ? AND expires_at > datetime('now')
              RETURNING user_id",
        )
        .bind(token_hash)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(user_id) = user_id else {
            return Ok(None);
        };
        let user: Option<(String, Option<String>)> =
            sqlx::query_as("SELECT name, language FROM users WHERE id = ? AND enabled = 1")
                .bind(&user_id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some((name, language)) = user else {
            tx.commit().await?;
            return Ok(None);
        };
        sqlx::query(
            "INSERT INTO telegram_user_chats (user_id, chat_id, telegram_username)
             VALUES (?, ?, ?)
             ON CONFLICT(user_id) DO UPDATE SET chat_id = excluded.chat_id,
                 telegram_username = excluded.telegram_username, linked_at = datetime('now')",
        )
        .bind(&user_id)
        .bind(chat_id)
        .bind(username)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(Some((user_id, name, language)))
    }
    .await;
    result.unwrap_or_else(log_err("consume user token"))
}

/// A user's linked chat, as (Telegram username, linked at).
pub(crate) async fn user_chat(
    pool: &SqlitePool,
    user_id: &str,
) -> Option<(Option<String>, String)> {
    sqlx::query_as("SELECT telegram_username, linked_at FROM telegram_user_chats WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .unwrap_or_else(log_err("read user chat"))
}

pub(crate) async fn unlink_user(pool: &SqlitePool, user_id: &str) {
    for sql in [
        "DELETE FROM telegram_user_chats WHERE user_id = ?",
        "DELETE FROM telegram_user_link_tokens WHERE user_id = ?",
    ] {
        let _ =
            sqlx::query(sql).bind(user_id).execute(pool).await.map_err(
                |e| tracing::warn!(target: "calrs::telegram", error = %e, "unlink failed"),
            );
    }
}

// --- What a message needs ---

/// One booking, as the worker composes messages about it.
#[derive(Debug, Clone)]
pub(crate) struct BookingRow {
    pub(crate) id: String,
    pub(crate) uid: String,
    pub(crate) status: String,
    pub(crate) start: DateTime<Utc>,
    pub(crate) end: DateTime<Utc>,
    pub(crate) guest_name: String,
    pub(crate) guest_email: String,
    pub(crate) guest_timezone: String,
    pub(crate) language: Option<String>,
    pub(crate) cancel_token: Option<String>,
    pub(crate) reschedule_token: Option<String>,
    pub(crate) event_type_id: String,
    pub(crate) title: String,
    /// The meeting link or place: the booking's own meeting URL, else the
    /// event type's location. For confirmed bookings only (pending emails
    /// do not reveal it either).
    pub(crate) location: Option<String>,
    pub(crate) guest_chat: Option<i64>,
    pub(crate) entered_username: Option<String>,
    pub(crate) telegram_username: Option<String>,
}

pub(crate) async fn booking_row(pool: &SqlitePool, booking_id: &str) -> Option<BookingRow> {
    #[allow(clippy::type_complexity)]
    let row: (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
        Option<String>,
    ) = sqlx::query_as(&format!(
        "SELECT b.id, b.uid, b.status, {START_AT}, {END_AT}, b.guest_name, b.guest_email,
                COALESCE(NULLIF(b.guest_timezone, ''), 'UTC'), b.language, b.cancel_token,
                b.reschedule_token, et.id, et.title,
                NULLIF(COALESCE(NULLIF(b.meeting_url, ''), et.location_value), '')
           FROM bookings b
           JOIN event_types et ON et.id = b.event_type_id
          WHERE b.id = ?"
    ))
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(log_err("read booking"))?;
    let link: Option<(Option<i64>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT CASE WHEN typeof(chat_id) = 'integer' THEN chat_id END,
                entered_username, telegram_username
           FROM telegram_guest_links WHERE booking_id = ?",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(log_err("read guest link"));
    let (guest_chat, entered_username, telegram_username) = link.unwrap_or_default();
    let (
        id,
        uid,
        status,
        start_at,
        end_at,
        guest_name,
        guest_email,
        guest_timezone,
        language,
        cancel_token,
        reschedule_token,
        event_type_id,
        title,
        location,
    ) = row;
    let tz = crate::booking_time::event_timezone(pool, &event_type_id)
        .await
        .unwrap_or(Tz::UTC);
    Some(BookingRow {
        id,
        uid,
        status,
        start: utc_instant(&start_at, tz)?,
        end: utc_instant(&end_at, tz)?,
        guest_name,
        guest_email,
        guest_timezone,
        language,
        cancel_token,
        reschedule_token,
        event_type_id,
        title,
        location,
        guest_chat,
        entered_username,
        telegram_username,
    })
}

/// A host's linked chat, with what their messages are written in.
#[derive(Debug, Clone)]
pub(crate) struct HostChat {
    pub(crate) chat_id: i64,
    pub(crate) language: Option<String>,
    pub(crate) timezone: String,
}

/// Who hosts a booking: the assigned member; else, for a collective team
/// event type, every eligible member; else the event type's owner. Only
/// enabled users with a linked chat are returned.
pub(crate) async fn host_chats(pool: &SqlitePool, booking_id: &str) -> Vec<HostChat> {
    let row: Option<(
        Option<String>,
        Option<String>,
        String,
        Option<String>,
        String,
    )> = sqlx::query_as(
        "SELECT b.assigned_user_id, et.team_id, et.scheduling_mode, a.user_id, et.id
               FROM bookings b
               JOIN event_types et ON et.id = b.event_type_id
               LEFT JOIN accounts a ON a.id = et.account_id
              WHERE b.id = ?",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(log_err("read booking hosts"));
    let Some((assigned, team_id, scheduling, owner, et_id)) = row else {
        return Vec::new();
    };
    let ids: Vec<String> = match (assigned, team_id) {
        (Some(member), _) => vec![member],
        (None, Some(team)) if scheduling == "collective" => sqlx::query_scalar(
            "SELECT u.id FROM users u
               JOIN team_members tm ON tm.user_id = u.id
               LEFT JOIN event_type_member_weights w
                 ON w.user_id = u.id AND w.event_type_id = ?
              WHERE tm.team_id = ? AND u.enabled = 1 AND COALESCE(w.weight, 1) > 0",
        )
        .bind(&et_id)
        .bind(&team)
        .fetch_all(pool)
        .await
        .unwrap_or_default(),
        _ => owner.into_iter().collect(),
    };
    let mut out = Vec::new();
    for id in ids {
        let chat: Option<(Option<i64>, Option<String>, String)> = sqlx::query_as(
            "SELECT CASE WHEN typeof(c.chat_id) = 'integer' THEN c.chat_id END, u.language,
                    COALESCE(NULLIF(u.timezone, ''), 'UTC')
               FROM telegram_user_chats c JOIN users u ON u.id = c.user_id
              WHERE c.user_id = ? AND u.enabled = 1",
        )
        .bind(&id)
        .fetch_optional(pool)
        .await
        .unwrap_or_else(log_err("read host chat"));
        if let Some((Some(chat_id), language, timezone)) = chat {
            if !out.iter().any(|h: &HostChat| h.chat_id == chat_id) {
                out.push(HostChat {
                    chat_id,
                    language,
                    timezone,
                });
            }
        }
    }
    out
}

/// The public page to book this event type again, as a path, when it is
/// public and enabled (private teams are left out: their link needs the
/// team's invite token).
pub(crate) async fn rebook_path(pool: &SqlitePool, et_id: &str) -> Option<String> {
    #[allow(clippy::type_complexity)]
    let (slug, visibility, enabled, team_slug, team_visibility, username): (
        String,
        String,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = sqlx::query_as(
        "SELECT et.slug, et.visibility, et.enabled, t.slug, t.visibility, u.username
           FROM event_types et
           LEFT JOIN accounts a ON a.id = et.account_id
           LEFT JOIN users u ON u.id = a.user_id
           LEFT JOIN teams t ON t.id = et.team_id
          WHERE et.id = ?",
    )
    .bind(et_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(log_err("read event type"))?;
    if visibility != "public" || enabled == 0 {
        return None;
    }
    let enc = |s: &str| urlencoding::encode(s).into_owned();
    match (team_slug, username) {
        (Some(_), _) if team_visibility.as_deref() == Some("private") => None,
        (Some(team), _) => Some(format!("/team/{}/{}", enc(&team), enc(&slug))),
        (None, Some(user)) => Some(format!("/u/{}/{}", enc(&user), enc(&slug))),
        (None, None) => None,
    }
}

// --- Retention ---

/// Drop what is no longer needed: guest links a week after the meeting, or a
/// day after the booking ended early (cancelled, declined or expired: the
/// worker sets `link_expires_at` to that moment, [`end_link`]); expired host
/// tokens. Runs whether Telegram is on or not (`telegram::spawn`), so a
/// switched-off feature still forgets its guests' chats. A booking that ended
/// while Telegram was off, so that no worker called [`end_link`], is ended
/// here first and dropped a day later.
pub(crate) async fn purge(pool: &SqlitePool, now: DateTime<Utc>) {
    let week_ago = sql_time(now - chrono::Duration::days(7));
    let day_ago = sql_time(now - chrono::Duration::days(1));
    let now_s = sql_time(now);
    if let Err(e) = sqlx::query(
        "UPDATE telegram_guest_links SET link_expires_at = ?1, link_token_hash = NULL, replaced_token_hash = NULL
          WHERE link_expires_at > ?1
            AND booking_id IN (SELECT id FROM bookings WHERE status IN ('cancelled', 'declined'))",
    )
    .bind(&now_s)
    .execute(pool)
    .await
    {
        tracing::warn!(target: "calrs::telegram", error = %e, "telegram purge failed");
    }
    let removed = sqlx::query(
        "DELETE FROM telegram_guest_links
          WHERE link_expires_at < ?1
             OR (link_expires_at < ?2
                 AND booking_id IN (SELECT id FROM bookings
                                     WHERE status IN ('cancelled', 'declined')))",
    )
    .bind(&week_ago)
    .bind(&day_ago)
    .execute(pool)
    .await
    .map(|r| r.rows_affected());
    let tokens = sqlx::query("DELETE FROM telegram_user_link_tokens WHERE expires_at < ?")
        .bind(&now_s)
        .execute(pool)
        .await
        .map(|r| r.rows_affected());
    match (removed, tokens) {
        (Ok(links), Ok(tokens)) if links + tokens > 0 => {
            tracing::info!(target: "calrs::telegram", links, tokens, "telegram links purged")
        }
        (Ok(_), Ok(_)) => {}
        (Err(e), _) | (_, Err(e)) => {
            tracing::warn!(target: "calrs::telegram", error = %e, "telegram purge failed")
        }
    }
}

/// A naive UTC time as stored by this module, back as an instant.
pub(crate) fn parse_sql_time(value: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(value, SQL_TIME)
        .ok()
        .map(|n| n.and_utc())
}
