//! ilia fork: the work behind `send_guest_message` and the thread that
//! `get_booking` shows (`web::messages`).
//!
//! The scope is the acting user's personal event types, as everywhere in
//! the agent. The message goes through `web::messages::send_host_message`,
//! the dashboard's own path, with its per-booking and per-user limits, plus
//! `MESSAGES_PER_HOUR` for the agent. The audit line names the booking, the
//! length and the channels, never the text.

use super::dto::*;
use super::error::{db, ToolError};
use super::ops_bookings::{check_booking_id, state_of};
use super::ops_write::Writer;
use super::sanitize::{is_invisible, owner_text, untrusted_text};
use crate::telegram::text::{GUEST_TEXT_MAX, HOST_TEXT_MAX};
use crate::web::messages::{self, Refusal, Via};
use crate::web::RateLimiter;
use chrono_tz::Tz;
use serde_json::{Map, Value};
use sqlx::SqlitePool;
use std::sync::OnceLock;

/// Messages the agent sends per hour for one account.
pub(crate) const MESSAGES_PER_HOUR: u32 = 30;

fn limiter() -> &'static RateLimiter {
    static LIMITER: OnceLock<RateLimiter> = OnceLock::new();
    LIMITER.get_or_init(|| RateLimiter::new(MESSAGES_PER_HOUR, 3600))
}

/// The text of a message: line breaks allowed, no other control or
/// invisible character, 1 to 2000 characters.
fn check_text(text: &str) -> Result<&str, ToolError> {
    let text = text.trim();
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(ToolError::invalid(
            "text",
            "must not contain control characters other than line breaks",
        ));
    }
    if text.chars().any(is_invisible) {
        return Err(ToolError::invalid(
            "text",
            "must not contain invisible formatting characters",
        ));
    }
    if text.is_empty() {
        return Err(ToolError::invalid("text", "must not be empty"));
    }
    if text.chars().count() > HOST_TEXT_MAX {
        return Err(ToolError::invalid(
            "text",
            format!("at most {HOST_TEXT_MAX} characters"),
        ));
    }
    Ok(text)
}

pub(crate) async fn send_guest_message(
    w: &Writer<'_>,
    args: SendGuestMessageArgs,
) -> Result<GuestMessageSent, ToolError> {
    let booking_id = check_booking_id(&args.booking_id)?;
    let text = check_text(&args.text)?;
    let pool = w.pool();
    let before = state_of(pool, w.me, booking_id)
        .await?
        .ok_or_else(|| ToolError::not_found("booking"))?;
    if limiter().check_limited(&w.me.id).await {
        return Err(ToolError::capped("guest messages", MESSAGES_PER_HOUR));
    }
    let author = messages::author(pool, &w.me.id)
        .await
        .ok_or_else(|| ToolError::unavailable("acting admin account unavailable"))?;
    let smtp = crate::email::load_smtp_config(pool, &w.ctx.state.secret_key)
        .await
        .ok()
        .flatten();
    let sent = messages::send_host_message(pool, smtp, &author, booking_id, Via::Mcp, text)
        .await
        .map_err(|refusal| match refusal {
            Refusal::NotFound => ToolError::not_found("booking"),
            Refusal::Closed => ToolError::unavailable(
                "the booking takes messages only while pending or confirmed, and up to a week \
                 after the meeting: nothing was sent",
            ),
            Refusal::Empty => ToolError::invalid("text", "must not be empty"),
            Refusal::TooLong => {
                ToolError::invalid("text", format!("at most {HOST_TEXT_MAX} characters"))
            }
            Refusal::Limit => ToolError::unavailable(
                "the daily message limit of this booking or account is reached \
                 (20 per booking, 5 of them by email, 50 per account): nothing was sent",
            ),
            Refusal::NoChannel => ToolError::unavailable(
                "the guest cannot be reached: email is not set up on this server and the \
                 guest has not linked Telegram; nothing was sent",
            ),
            Refusal::Internal => ToolError::unavailable(
                "the message could not be stored (a database error): nothing was sent; \
                 try again",
            ),
        })?;
    let mut audit_before = Map::new();
    audit_before.insert("status".into(), Value::from(before.status.as_str()));
    let mut after = Map::new();
    after.insert("chars".into(), Value::from(text.chars().count()));
    after.insert("email".into(), Value::from(sent.email.as_str()));
    after.insert("telegram".into(), Value::from(sent.telegram.as_str()));
    w.audit("send_guest_message", booking_id, audit_before, after);
    Ok(GuestMessageSent {
        message_id: sent.id,
        email: sent.email.as_str().to_string(),
        telegram: sent.telegram.as_str().to_string(),
    })
}

/// A booking's thread for `get_booking`: host text as written, guest text
/// cleaned and under `guest_supplied`.
pub(crate) async fn thread_of(
    pool: &SqlitePool,
    booking_id: &str,
    tz: Tz,
) -> Result<Vec<MessageOut>, ToolError> {
    let rows = messages::thread(pool, booking_id)
        .await
        .map_err(db("read the booking's messages"))?;
    Ok(rows
        .into_iter()
        .map(|m| {
            let from_host = m.direction == "host";
            MessageOut {
                at: crate::telegram::store::parse_sql_time(&m.created_at)
                    .map(|at| at.with_timezone(&tz).to_rfc3339())
                    .unwrap_or_default(),
                from: if from_host { "host" } else { "guest" }.to_string(),
                via: m.via,
                email: m.email_status,
                telegram: m.telegram_status,
                text: from_host.then(|| owner_text(&m.body, HOST_TEXT_MAX).0),
                guest_supplied: (!from_host).then(|| GuestMessageText {
                    text: untrusted_text(&m.body, GUEST_TEXT_MAX),
                }),
            }
        })
        .collect())
}
