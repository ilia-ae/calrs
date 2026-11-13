//! ilia fork: host <-> guest messages, one thread per booking (ilia-booking
//! docs/operations.md, «Сообщения гостям»; migration
//! `ilia_006_booking_messages.sql`).
//!
//! A host writes to the guest of one booking, from the booking in the
//! dashboard (`/dashboard/bookings/{id}/messages`), through the MCP agent
//! (`send_guest_message`) or by replying in Telegram (`telegram::relay`).
//! The guest gets it by email from the system address under the host's
//! name, Reply-To the system mailbox (`email::send_guest_host_message`,
//! `crate::inbox`), and in the Telegram chat linked to the booking. A
//! Telegram reply goes by email too when the guest has no linked chat or
//! last wrote by email. The guest's texts in their linked chat, and their
//! email replies read from the system mailbox (`via = 'email'`), are stored
//! here and relayed to the hosts' chats.
//!
//! No bulk messaging: one booking per call, no list or broadcast endpoint,
//! and persistent limits counted from the thread itself: 20 host messages
//! per booking a day, of which 5 by email (past that, Telegram only), 50
//! per user a day across all bookings, and 30 guest texts per booking a
//! day. Each limit is checked inside the INSERT that stores the message,
//! so two concurrent sends cannot both pass it. Logs carry the booking id,
//! the length and the channels, never the text.

use super::*;
use crate::telegram::store::{self as tg_store, sql_time, BookingRow};
use crate::telegram::text::{clean_lines, HOST_TEXT_MAX};
use chrono::DateTime;

/// Host messages per booking in any 24 hours.
pub(crate) const HOST_PER_BOOKING_DAY: i64 = 20;
/// Of those, sent by email.
pub(crate) const EMAIL_PER_BOOKING_DAY: i64 = 5;
/// Host messages per user in any 24 hours, over all bookings.
pub(crate) const HOST_PER_USER_DAY: i64 = 50;
/// Guest texts per booking in any 24 hours.
pub(crate) const GUEST_PER_BOOKING_DAY: i64 = 30;
/// How long after the meeting's end the host can still write.
const OPEN_AFTER_END_DAYS: i64 = 7;
/// Messages a thread shows (the newest).
pub(crate) const THREAD_MAX: i64 = 50;

/// Where a host message was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Via {
    Dashboard,
    Mcp,
    Telegram,
}

impl Via {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Via::Dashboard => "dashboard",
            Via::Mcp => "mcp",
            Via::Telegram => "telegram",
        }
    }
}

/// What happened on one channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Channel {
    /// Handed to the mail queue or the Telegram outbox.
    Queued,
    /// Not available: no SMTP, Telegram off, or a Telegram reply (email).
    Off,
    /// Email only: this booking's daily email limit is reached.
    Limit,
    /// Telegram only: the guest has not linked a chat.
    NotLinked,
}

impl Channel {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Channel::Queued => "queued",
            Channel::Off => "off",
            Channel::Limit => "limit",
            Channel::NotLinked => "not_linked",
        }
    }
}

/// A host message that was stored and handed on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Sent {
    pub(crate) id: String,
    pub(crate) email: Channel,
    pub(crate) telegram: Channel,
}

/// Why a host message was not sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// No such booking, or not this user's.
    NotFound,
    /// Not pending or confirmed, or ended more than a week ago.
    Closed,
    Empty,
    TooLong,
    /// A daily limit is reached.
    Limit,
    /// Neither email nor Telegram can reach the guest.
    NoChannel,
    /// A database error: nothing was stored, try again.
    Internal,
}

impl Refusal {
    /// The `?error=` code of the dashboard page.
    pub(crate) fn code(self) -> &'static str {
        match self {
            Refusal::NotFound => "not_found",
            Refusal::Closed => "closed",
            Refusal::Empty => "empty",
            Refusal::TooLong => "too_long",
            Refusal::Limit => "limit",
            Refusal::NoChannel => "no_channel",
            Refusal::Internal => "internal",
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        [
            Refusal::Closed,
            Refusal::Empty,
            Refusal::TooLong,
            Refusal::Limit,
            Refusal::NoChannel,
            Refusal::Internal,
        ]
        .into_iter()
        .find(|r| r.code() == code)
    }

    fn message_key(self) -> &'static str {
        match self {
            Refusal::NotFound | Refusal::Closed => "msg-error-closed",
            Refusal::Empty => "msg-error-empty",
            Refusal::TooLong => "msg-error-too-long",
            Refusal::Limit => "msg-error-limit",
            Refusal::NoChannel => "msg-error-no-channel",
            Refusal::Internal => "msg-error-internal",
        }
    }
}

/// Who writes a host message: their name, and the address the guest's
/// replies go to (the booking address, else the login one).
#[derive(Debug, Clone)]
pub(crate) struct Author {
    pub(crate) user_id: String,
    pub(crate) name: String,
    pub(crate) email: String,
}

/// An enabled user as an author.
pub(crate) async fn author(pool: &SqlitePool, user_id: &str) -> Option<Author> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT name, COALESCE(NULLIF(booking_email, ''), email) FROM users
          WHERE id = ? AND enabled = 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);
    row.map(|(name, email)| Author {
        user_id: user_id.to_string(),
        name,
        email,
    })
}

/// Whether `user_id` may write about `booking_id`: the owner of its event
/// type (the dashboard's rule for cancelling) or its assigned member.
pub(crate) async fn may_write(pool: &SqlitePool, user_id: &str, booking_id: &str) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM bookings b
                          JOIN event_types et ON et.id = b.event_type_id
                          LEFT JOIN accounts a ON a.id = et.account_id
                         WHERE b.id = ?1 AND (a.user_id = ?2 OR b.assigned_user_id = ?2))",
    )
    .bind(booking_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
    .unwrap_or(false)
}

/// The booking still takes host messages.
fn open(booking: &BookingRow, now: DateTime<Utc>) -> bool {
    matches!(booking.status.as_str(), "pending" | "confirmed")
        && booking.end > now - Duration::days(OPEN_AFTER_END_DAYS)
}

/// Store a host message about `booking_id` and hand it to email and the
/// guest's Telegram chat. `smtp`: the mail server, when one is set up and
/// the message should go by email (a Telegram reply passes it only when the
/// guest has no linked chat or last wrote by email, `telegram::relay`).
pub(crate) async fn send_host_message(
    pool: &SqlitePool,
    smtp: Option<crate::email::SmtpConfig>,
    author: &Author,
    booking_id: &str,
    via: Via,
    text: &str,
) -> Result<Sent, Refusal> {
    let raw = text.trim();
    if raw.chars().count() > HOST_TEXT_MAX {
        return Err(Refusal::TooLong);
    }
    let body = clean_lines(raw, HOST_TEXT_MAX);
    if body.is_empty() {
        return Err(Refusal::Empty);
    }
    if !may_write(pool, &author.user_id, booking_id).await {
        return Err(Refusal::NotFound);
    }
    let Some(booking) = tg_store::booking_row(pool, booking_id).await else {
        return Err(Refusal::NotFound);
    };
    let now = Utc::now();
    if !open(&booking, now) {
        return Err(Refusal::Closed);
    }
    let telegram = crate::telegram::enabled()
        && tg_store::linked_guest_chat(pool, booking_id)
            .await
            .is_some();
    if smtp.is_none() && !telegram {
        return Err(Refusal::NoChannel);
    }
    let id = uuid::Uuid::new_v4().to_string();
    let day_ago = sql_time(now - Duration::days(1));
    // One statement: the limits are counted and the row stored together.
    let stored: Option<(String, String)> = sqlx::query_as(
        "INSERT INTO booking_messages
                (id, booking_id, direction, author_user_id, via, body, email_status,
                 telegram_status, created_at)
         SELECT ?1, ?2, 'host', ?3, ?4, ?5,
                CASE WHEN ?6 AND (SELECT COUNT(*) FROM booking_messages
                                   WHERE booking_id = ?2 AND direction = 'host'
                                     AND email_status != 'none' AND created_at > ?8) < ?9
                     THEN 'queued' ELSE 'none' END,
                CASE WHEN ?7 THEN 'queued' ELSE 'none' END,
                ?10
          WHERE (SELECT COUNT(*) FROM booking_messages
                  WHERE booking_id = ?2 AND direction = 'host' AND created_at > ?8) < ?11
            AND (SELECT COUNT(*) FROM booking_messages
                  WHERE author_user_id = ?3 AND created_at > ?8) < ?12
         RETURNING email_status, telegram_status",
    )
    .bind(&id)
    .bind(booking_id)
    .bind(&author.user_id)
    .bind(via.as_str())
    .bind(&body)
    .bind(smtp.is_some())
    .bind(telegram)
    .bind(&day_ago)
    .bind(EMAIL_PER_BOOKING_DAY)
    .bind(sql_time(now))
    .bind(HOST_PER_BOOKING_DAY)
    .bind(HOST_PER_USER_DAY)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, booking_id, "could not store a host message");
        Refusal::Internal
    })?;
    let Some((email_status, telegram_status)) = stored else {
        tracing::info!(booking_id, "host message refused: daily limit");
        return Err(Refusal::Limit);
    };
    let email = match (&smtp, email_status.as_str()) {
        (Some(_), "queued") => Channel::Queued,
        (Some(_), _) => Channel::Limit,
        (None, _) => Channel::Off,
    };
    let telegram = match (telegram, telegram_status.as_str()) {
        (true, "queued") => Channel::Queued,
        _ if crate::telegram::enabled() => Channel::NotLinked,
        _ => Channel::Off,
    };
    if email != Channel::Queued && telegram != Channel::Queued {
        // Over the email limit with no chat to fall back on.
        let _ = sqlx::query("DELETE FROM booking_messages WHERE id = ?")
            .bind(&id)
            .execute(pool)
            .await;
        tracing::info!(
            booking_id,
            "host message refused: email limit and no telegram"
        );
        return Err(Refusal::Limit);
    }
    tracing::info!(
        booking_id,
        via = via.as_str(),
        chars = body.chars().count(),
        email = email.as_str(),
        telegram = telegram.as_str(),
        "host message sent to the guest"
    );
    if let (Channel::Queued, Some(smtp)) = (email, smtp) {
        let mut mail = host_message_mail(&booking, author, &body);
        // The event as the guest's pages name it (`event_texts`).
        mail.event_title = super::event_texts::title_in(
            pool,
            &booking.event_type_id,
            &mail.lang,
            mail.event_title,
        )
        .await;
        // And its author's name (`user_texts`).
        mail.host_name =
            super::user_texts::name_in(pool, &author.user_id, &mail.lang, mail.host_name).await;
        let (pool, id, booking_id) = (pool.clone(), id.clone(), booking_id.to_string());
        spawn_emails(async move {
            let result = crate::email::send_guest_host_message(&smtp, &mail).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            log_email_result(result, "guest_host_message", &booking_id);
            let _ = sqlx::query("UPDATE booking_messages SET email_status = ? WHERE id = ?")
                .bind(status)
                .bind(&id)
                .execute(&pool)
                .await;
        });
    }
    if telegram == Channel::Queued {
        let queued = crate::telegram::notify_queued(
            pool,
            booking_id,
            crate::telegram::TelegramEvent::HostMessage {
                id: id.clone(),
                text: body,
            },
        );
        if !queued {
            tg_store::set_message_telegram_status(pool, &id, "failed").await;
        }
    }
    Ok(Sent {
        id,
        email,
        telegram,
    })
}

/// The email of a host message, in the guest's language and zone.
fn host_message_mail(
    booking: &BookingRow,
    author: &Author,
    body: &str,
) -> crate::email::HostMessageMail {
    let lang = crate::email::supported_language(booking.language.clone())
        .unwrap_or_else(|| "en".to_string());
    let tz: chrono_tz::Tz = booking.guest_timezone.parse().unwrap_or(chrono_tz::Tz::UTC);
    let local = booking.start.with_timezone(&tz);
    crate::email::HostMessageMail {
        date: crate::i18n::format_long_date(local.date_naive(), &lang),
        time: format!("{} ({})", local.format("%H:%M"), tz.name()),
        lang,
        guest_name: booking.guest_name.clone(),
        guest_email: booking.guest_email.clone(),
        host_name: author.name.clone(),
        host_email: author.email.clone(),
        booking_uid: booking.uid.clone(),
        event_title: booking.title.clone(),
        body: body.to_string(),
    }
}

/// ilia fork (inbox): what an email reply adds to a guest-side row.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FromEmail<'a> {
    /// The Message-ID hash, so a message polled twice is stored once.
    pub(crate) hash: &'a str,
    pub(crate) sender: crate::inbox::matcher::Sender,
    /// The From address; stored only for an attendee or another sender.
    pub(crate) address: &'a str,
}

/// Store a guest-side text (already cleaned), unless the booking has had
/// `GUEST_PER_BOOKING_DAY` of them in the last day. `relayed`: it is queued
/// for a host chat; `email`: it came by email (`via = 'email'`, with who
/// sent it), else by the guest's Telegram chat. Returns the row id; `None`
/// over the limit, for a duplicate email, or on a database error.
pub(crate) async fn store_guest_message(
    pool: &SqlitePool,
    booking_id: &str,
    body: &str,
    relayed: bool,
    email: Option<FromEmail<'_>>,
) -> Option<String> {
    use crate::inbox::matcher::Sender;
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();
    let via = if email.is_some() { "email" } else { "telegram" };
    let sender_email = email
        .filter(|e| e.sender != Sender::Guest)
        .map(|e| crate::telegram::text::clean(e.address, crate::telegram::text::EMAIL_MAX))
        .filter(|a| !a.is_empty());
    sqlx::query_scalar(
        "INSERT OR IGNORE INTO booking_messages
                (id, booking_id, direction, via, body, telegram_status, created_at, inbound_hash,
                 sender, sender_email)
         SELECT ?1, ?2, 'guest', ?8, ?3, CASE WHEN ?4 THEN 'queued' ELSE 'none' END, ?5, ?9,
                ?10, ?11
          WHERE (SELECT COUNT(*) FROM booking_messages
                  WHERE booking_id = ?2 AND direction = 'guest' AND created_at > ?6) < ?7
         RETURNING id",
    )
    .bind(&id)
    .bind(booking_id)
    .bind(body)
    .bind(relayed)
    .bind(sql_time(now))
    .bind(sql_time(now - Duration::days(1)))
    .bind(GUEST_PER_BOOKING_DAY)
    .bind(via)
    .bind(email.map(|e| e.hash))
    .bind(email.map(|e| e.sender.as_str()))
    .bind(sender_email)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, booking_id, "could not store a guest message");
        None
    })
}

/// The guest's latest message on `booking_id` came by email (a Telegram
/// reply of the host then goes by email too).
pub(crate) async fn guest_wrote_last_by_email(pool: &SqlitePool, booking_id: &str) -> bool {
    sqlx::query_scalar::<_, String>(
        "SELECT via FROM booking_messages WHERE booking_id = ? AND direction = 'guest'
          ORDER BY created_at DESC, rowid DESC LIMIT 1",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .is_some_and(|via| via == "email")
}

/// A host already wrote to the guest of `booking_id` (the guest of an
/// unpaid booking may then answer).
pub(crate) async fn host_wrote(pool: &SqlitePool, booking_id: &str) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS (SELECT 1 FROM booking_messages WHERE booking_id = ? AND direction = 'host')",
    )
    .bind(booking_id)
    .fetch_one(pool)
    .await
    .map(|v| v != 0)
    .unwrap_or(false)
}

/// One message of a thread, as stored.
#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct StoredMessage {
    pub(crate) id: String,
    pub(crate) direction: String,
    pub(crate) via: String,
    pub(crate) body: String,
    pub(crate) email_status: String,
    pub(crate) telegram_status: String,
    pub(crate) created_at: String,
    /// ilia fork (inbox): an email's sender kind (`guest`, `attendee`,
    /// `other`) and, for the last two, the address it came from.
    pub(crate) sender: Option<String>,
    pub(crate) sender_email: Option<String>,
}

impl StoredMessage {
    /// Who wrote it: `host`, `guest`, `attendee` (an additional attendee's
    /// email) or `other` (an email from an address that is neither).
    pub(crate) fn author(&self) -> &str {
        match (self.direction.as_str(), self.sender.as_deref()) {
            ("host", _) => "host",
            (_, Some("attendee")) => "attendee",
            (_, Some("other")) => "other",
            _ => "guest",
        }
    }
}

/// The newest `THREAD_MAX` messages of a booking, oldest first.
pub(crate) async fn thread(
    pool: &SqlitePool,
    booking_id: &str,
) -> Result<Vec<StoredMessage>, sqlx::Error> {
    let mut rows: Vec<StoredMessage> = sqlx::query_as(
        "SELECT id, direction, via, body, email_status, telegram_status, created_at,
                sender, sender_email
           FROM booking_messages WHERE booking_id = ?
          ORDER BY created_at DESC, rowid DESC LIMIT ?",
    )
    .bind(booking_id)
    .bind(THREAD_MAX)
    .fetch_all(pool)
    .await?;
    rows.reverse();
    Ok(rows)
}

/// Per booking of `ids` with any message, for the bookings dashboard:
/// `total` messages, and `new`: guest messages after the host's last one
/// (unanswered). Ids with none are left out.
pub(crate) async fn counts(pool: &SqlitePool, ids: &[String]) -> HashMap<String, minijinja::Value> {
    let mut out = HashMap::new();
    for chunk in ids.chunks(200) {
        let marks = vec!["?"; chunk.len()].join(",");
        let sql = format!(
            "SELECT m.booking_id, COUNT(*),
                    SUM(m.direction = 'guest' AND m.created_at > COALESCE(
                        (SELECT MAX(h.created_at) FROM booking_messages h
                          WHERE h.booking_id = m.booking_id AND h.direction = 'host'), ''))
               FROM booking_messages m
              WHERE m.booking_id IN ({marks}) GROUP BY m.booking_id"
        );
        let mut query = sqlx::query_as::<_, (String, i64, i64)>(&sql);
        for id in chunk {
            query = query.bind(id);
        }
        match query.fetch_all(pool).await {
            Ok(rows) => out.extend(
                rows.into_iter()
                    .map(|(id, total, new)| (id, context! { total => total, new => new })),
            ),
            Err(e) => tracing::warn!(error = %e, "could not count booking messages"),
        }
    }
    out
}

/// The bookings dashboard's "recent meetings": this user's pending or
/// confirmed bookings that already started and ended at most
/// `OPEN_AFTER_END_DAYS` ago, so they leave the upcoming list but still
/// take messages. Those with unanswered guest messages first, then the
/// latest; at most 50. Each: id, event_title, guest_name, guest_email,
/// start_at (in the host's zone and language).
pub(crate) async fn recent_bookings(
    pool: &SqlitePool,
    user_id: &str,
    lang: &str,
    host_tz: &str,
) -> Vec<(String, minijinja::Value)> {
    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT b.id, et.title, b.guest_name, b.guest_email, b.start_at
           FROM bookings b
           JOIN event_types et ON et.id = b.event_type_id
           LEFT JOIN accounts a ON a.id = et.account_id
          WHERE (a.user_id = ?1 OR b.assigned_user_id = ?1)
            AND b.status IN ('pending', 'confirmed')
            AND datetime(b.start_at) < datetime('now')
            AND datetime(b.end_at) > datetime('now', ?2)
          ORDER BY EXISTS (SELECT 1 FROM booking_messages g
                            WHERE g.booking_id = b.id AND g.direction = 'guest'
                              AND g.created_at > COALESCE(
                                  (SELECT MAX(h.created_at) FROM booking_messages h
                                    WHERE h.booking_id = b.id AND h.direction = 'host'), '')) DESC,
                   b.start_at DESC
          LIMIT 50",
    )
    .bind(user_id)
    .bind(format!("-{OPEN_AFTER_END_DAYS} days"))
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "could not list recent bookings");
        Vec::new()
    });
    let tz: chrono_tz::Tz = host_tz.parse().unwrap_or(chrono_tz::Tz::UTC);
    rows.into_iter()
        .map(|(id, title, guest_name, guest_email, start)| {
            let start_at = tg_store::utc_instant(&start, chrono_tz::Tz::UTC)
                .map(|at| written_at(lang, tz, at))
                .unwrap_or(start);
            let value = context! {
                id => id.clone(),
                event_title => title,
                guest_name => guest_name,
                guest_email => guest_email,
                start_at => start_at,
            };
            (id, value)
        })
        .collect()
}

/// The bookings dashboard's fork context: the recent meetings, and the
/// message counts of those and of the listed `ids`.
pub(crate) async fn dashboard_ctx(
    pool: &SqlitePool,
    user_id: &str,
    lang: &str,
    host_tz: &str,
    ids: &[String],
) -> (Vec<minijinja::Value>, HashMap<String, minijinja::Value>) {
    let recent = recent_bookings(pool, user_id, lang, host_tz).await;
    let mut all: Vec<String> = ids.to_vec();
    all.extend(recent.iter().map(|(id, _)| id.clone()));
    let counts = counts(pool, &all).await;
    (recent.into_iter().map(|(_, v)| v).collect(), counts)
}

// --- The dashboard page ---

pub(super) fn router() -> Router<Arc<AppState>> {
    Router::new().route(
        "/dashboard/bookings/{id}/messages",
        get(thread_page).post(post_message),
    )
}

#[derive(Deserialize)]
struct PageQuery {
    #[serde(default)]
    sent: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

/// A stored UTC time, written out in `tz` and `lang`.
fn written_at(lang: &str, tz: chrono_tz::Tz, at: DateTime<Utc>) -> String {
    let local = at.with_timezone(&tz);
    crate::telegram::message::when(
        lang,
        local.date_naive(),
        &local.format("%H:%M").to_string(),
        tz.name(),
    )
}

/// The author line of a thread message: the host, the guest, or, for an
/// email from another address, that address and what it is.
fn who(lang: &str, m: &StoredMessage) -> String {
    let address = crate::telegram::text::guest_email(m.sender_email.as_deref().unwrap_or(""));
    let key = match m.author() {
        "host" => "msg-from-host",
        "attendee" => "msg-from-attendee",
        "other" => "msg-from-other",
        _ => "msg-from-guest",
    };
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("address", address);
    crate::i18n::translate(lang, key, Some(&args))
}

fn status_label(lang: &str, channel: &str, status: &str) -> Option<String> {
    matches!(status, "queued" | "sent" | "failed")
        .then(|| crate::i18n::translate(lang, &format!("msg-status-{channel}-{status}"), None))
}

async fn thread_page(
    State(state): State<Arc<AppState>>,
    auth_user: crate::auth::AuthUser,
    Path(booking_id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    let error = query.error.as_deref().and_then(Refusal::from_code);
    render_thread(
        &state,
        &auth_user,
        &booking_id,
        query.sent.is_some(),
        error,
        "",
    )
    .await
}

/// The thread page; a refused send shows its reason with the typed text
/// kept in the box.
async fn render_thread(
    state: &AppState,
    auth_user: &crate::auth::AuthUser,
    booking_id: &str,
    sent: bool,
    error: Option<Refusal>,
    draft: &str,
) -> Response {
    let booking_id = booking_id.to_string();
    let user = &auth_user.user;
    let lang = auth_user.lang;
    if !may_write(&state.pool, &user.id, &booking_id).await {
        return (axum::http::StatusCode::NOT_FOUND, "").into_response();
    }
    let Some(booking) = tg_store::booking_row(&state.pool, &booking_id).await else {
        return (axum::http::StatusCode::NOT_FOUND, "").into_response();
    };
    let tz: chrono_tz::Tz = user.timezone.parse().unwrap_or(chrono_tz::Tz::UTC);
    let stored = match thread(&state.pool, &booking_id).await {
        Ok(rows) => rows,
        Err(e) => return internal_error_response("booking messages", &e),
    };
    let messages: Vec<minijinja::Value> = stored
        .iter()
        .map(|m| {
            let from_host = m.direction == "host";
            context! {
                from_host => from_host,
                who => who(lang, m),
                // ilia fork (inbox): an email from someone other than the
                // guest is marked, so a forward is never read as the guest.
                not_guest => matches!(m.author(), "attendee" | "other"),
                via => crate::i18n::translate(lang, &format!("msg-via-{}", m.via), None),
                at => tg_store::parse_sql_time(&m.created_at)
                    .map(|at| written_at(lang, tz, at))
                    .unwrap_or_default(),
                email => status_label(lang, "email", &m.email_status),
                telegram => status_label(lang, "telegram", &m.telegram_status),
                text => m.body.clone(),
            }
        })
        .collect();
    let email_on = matches!(
        crate::email::load_smtp_config(&state.pool, &state.secret_key).await,
        Ok(Some(_))
    );
    let telegram_on = crate::telegram::enabled();
    let telegram_linked = telegram_on
        && tg_store::linked_guest_chat(&state.pool, &booking_id)
            .await
            .is_some();
    let error = error.map(|r| crate::i18n::translate(lang, r.message_key(), None));
    let (impersonating, impersonating_name, _) = impersonation_ctx(auth_user);
    let tmpl = match state.templates.get_template("ilia/booking_messages.html") {
        Ok(t) => t,
        Err(e) => return internal_error_response("template", &e),
    };
    let rendered = tmpl.render(context! {
        lang => lang,
        sidebar => sidebar_context(auth_user, "bookings"),
        impersonating => impersonating,
        impersonating_name => impersonating_name,
        booking_id => booking_id,
        event => booking.title,
        when => written_at(lang, tz, booking.start),
        guest_name => booking.guest_name,
        guest_email => booking.guest_email,
        messages => messages,
        open => open(&booking, Utc::now()),
        sent => sent,
        error => error,
        draft => draft,
        email_on => email_on,
        inbox_on => crate::inbox::enabled(),
        telegram_on => telegram_on,
        telegram_linked => telegram_linked,
        max_chars => HOST_TEXT_MAX,
    });
    match rendered {
        Ok(html) => Html(html).into_response(),
        Err(e) => internal_error_response("template render", &e),
    }
}

#[derive(Deserialize)]
struct MessageForm {
    _csrf: Option<String>,
    #[serde(default)]
    text: String,
}

async fn post_message(
    State(state): State<Arc<AppState>>,
    auth_user: crate::auth::AuthUser,
    headers: HeaderMap,
    Path(booking_id): Path<String>,
    Form(form): Form<MessageForm>,
) -> Response {
    if let Err(resp) = verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    // An admin looking through someone's eyes does not write to their
    // guests in their name.
    if auth_user.impersonation.is_some() {
        return axum::http::StatusCode::FORBIDDEN.into_response();
    }
    let Some(author) = author(&state.pool, &auth_user.user.id).await else {
        return axum::http::StatusCode::FORBIDDEN.into_response();
    };
    let smtp = crate::email::load_smtp_config(&state.pool, &state.secret_key)
        .await
        .ok()
        .flatten();
    let page = format!(
        "/dashboard/bookings/{}/messages",
        urlencoding::encode(&booking_id)
    );
    match send_host_message(
        &state.pool,
        smtp,
        &author,
        &booking_id,
        Via::Dashboard,
        &form.text,
    )
    .await
    {
        Ok(_) => Redirect::to(&format!("{page}?sent=1")).into_response(),
        Err(Refusal::NotFound) => (axum::http::StatusCode::NOT_FOUND, "").into_response(),
        // Shown at once with the typed text kept, so nothing is lost.
        Err(refusal) => {
            let draft: String = form.text.chars().take(HOST_TEXT_MAX * 2).collect();
            let mut resp = render_thread(
                &state,
                &auth_user,
                &booking_id,
                false,
                Some(refusal),
                &draft,
            )
            .await;
            if resp.status().is_success() {
                *resp.status_mut() = axum::http::StatusCode::UNPROCESSABLE_ENTITY;
            }
            resp
        }
    }
}
