//! Free text in a private chat (ilia fork; the thread is `web::messages`).
//!
//! A guest's text in the chat linked to their booking goes to the booking's
//! hosts, with the booking named, the text cleaned to one line, defanged and
//! capped (`message::host`), and a "reply to answer" line. A host who
//! replies to a message about a booking, in their own linked chat, writes to
//! that booking's guest: the reply goes to the guest's Telegram, and by
//! email from the system address when the guest has no linked chat or last
//! wrote by email (ilia fork, `crate::inbox`).
//!
//! Everything a Telegram user sends is untrusted. The text is stored in the
//! booking's thread and relayed, never echoed back to its writer and never
//! logged (logs carry the booking id and the length). Limits: per chat 5
//! guest texts per 10 minutes, then one "too many", then silence; 30 guest
//! texts per booking a day (counted in the database); 100 relays an hour
//! across all chats, counted only for texts that are stored and have a
//! host chat to go to (past it, the text is still stored and shown on the
//! dashboard). A booking that awaits its TON payment takes no guest text
//! (the hosts have not heard of it yet, and an unpaid booking must not
//! become a way to write to them) unless a host already wrote to its guest.

use sqlx::SqlitePool;

use super::commands::{quiet_reply, reply};
use super::message::{bot_reply, Reply};
use super::store;
use super::worker::Job;
use super::{Runtime, TelegramEvent};
use crate::web::messages::{self, Refusal, Via};

/// Handle text that is not a command. `false` when there is nothing to
/// relay (a chat with no booking): the caller answers with the help.
pub(crate) async fn handle_text(
    rt: &Runtime,
    pool: &SqlitePool,
    chat_id: i64,
    reply_to: Option<i64>,
    text: &str,
    lang: &'static str,
) -> bool {
    let mapped = match reply_to {
        Some(message_id) => store::mapped_message(pool, chat_id, message_id).await,
        None => None,
    };
    if let Some((booking_id, side)) = &mapped {
        if side == "host" {
            host_reply(rt, pool, chat_id, booking_id, text, lang).await;
            return true;
        }
    }
    let live = store::guest_bookings_of_chat(pool, chat_id).await;
    let booking_id = match mapped {
        // A reply to a message about one of this chat's bookings.
        Some((booking_id, _)) if live.contains(&booking_id) => booking_id,
        Some(_) => {
            quiet_reply(
                rt,
                pool,
                chat_id,
                bot_reply(Reply::MessageClosed, lang, ""),
                "message_closed",
            );
            return true;
        }
        None => match live.as_slice() {
            [] => return false,
            [only] => only.clone(),
            _ => {
                let msg = bot_reply(Reply::MessagePickBooking, lang, "");
                quiet_reply(rt, pool, chat_id, msg, "message_pick_booking");
                return true;
            }
        },
    };
    guest_text(rt, pool, chat_id, &booking_id, text, lang).await;
    true
}

/// A guest's text about `booking_id`, from its linked chat.
async fn guest_text(
    rt: &Runtime,
    pool: &SqlitePool,
    chat_id: i64,
    booking_id: &str,
    text: &str,
    lang: &'static str,
) {
    let limits = &rt.limits;
    let n = limits.guest_texts.hit(&chat_id);
    if n > limits.guest_texts.max() {
        if n == limits.guest_texts.max() + 1 {
            reply(
                rt,
                pool,
                chat_id,
                bot_reply(Reply::MessageLimit, lang, ""),
                "message_limit",
            );
        }
        return;
    }
    let Some(booking) = store::booking_row(pool, booking_id).await else {
        return;
    };
    // A host who wrote to the guest of an unpaid booking is aware of it:
    // the guest may answer.
    let unpaid = crate::web::ton_pay::telegram::facts(pool, booking_id)
        .await
        .is_some_and(|p| p.hosts_unaware(&booking.status))
        && !messages::host_wrote(pool, booking_id).await;
    if unpaid {
        let msg = bot_reply(Reply::MessageAfterPayment, lang, "");
        quiet_reply(rt, pool, chat_id, msg, "message_after_payment");
        return;
    }
    let body = super::text::clean(text, super::text::GUEST_TEXT_MAX);
    if body.is_empty() {
        return;
    }
    let shortened =
        super::text::clean(text, text.len()).chars().count() > super::text::GUEST_TEXT_MAX;
    let hosts = !store::host_chats(pool, booking_id).await.is_empty();
    let Some(id) = messages::store_guest_message(pool, booking_id, &body, hosts, None).await else {
        // Over the booking's daily limit (or the store failed).
        let msg = bot_reply(Reply::MessageLimit, lang, "");
        quiet_reply(rt, pool, chat_id, msg, "message_limit");
        return;
    };
    tracing::info!(target: "calrs::telegram", booking_id, chars = body.chars().count(), hosts, "guest message received");
    if hosts && limits.relays.hit(&()) > limits.relays.max() {
        // Stored all the same: the hosts see it on the dashboard.
        tracing::warn!(target: "calrs::telegram", booking_id, "guest message relays used up this hour; stored, not relayed");
        store::set_message_telegram_status(pool, &id, "failed").await;
    } else if hosts {
        let queued = rt.enqueue(Job::Event {
            pool: pool.clone(),
            booking_id: booking_id.to_string(),
            event: TelegramEvent::GuestMessage {
                id: id.clone(),
                text: body,
                email: None,
            },
        });
        if !queued {
            store::set_message_telegram_status(pool, &id, "failed").await;
        }
    }
    // Stored either way: the hosts see the thread on the dashboard.
    let sent = if shortened {
        Reply::MessageSentShortened
    } else {
        Reply::MessageSent
    };
    reply(rt, pool, chat_id, bot_reply(sent, lang, ""), "message_sent");
}

/// A host's reply, in their own chat, to a message about `booking_id`.
async fn host_reply(
    rt: &Runtime,
    pool: &SqlitePool,
    chat_id: i64,
    booking_id: &str,
    text: &str,
    lang: &'static str,
) {
    let author = match store::host_user_of_chat(pool, chat_id).await {
        Some(user_id) => messages::author(pool, &user_id).await,
        None => None,
    };
    let Some(author) = author else {
        let msg = bot_reply(Reply::HostReplyNotAllowed, lang, "");
        quiet_reply(rt, pool, chat_id, msg, "reply_not_allowed");
        return;
    };
    // By email too when the guest cannot read Telegram here, or answered
    // by email last (the system address; `crate::inbox`).
    let by_email = store::linked_guest_chat(pool, booking_id).await.is_none()
        || messages::guest_wrote_last_by_email(pool, booking_id).await;
    let smtp = match rt.secret_key.get() {
        Some(key) if by_email => crate::email::load_smtp_config(pool, key)
            .await
            .ok()
            .flatten(),
        _ => None,
    };
    let answer =
        match messages::send_host_message(pool, smtp, &author, booking_id, Via::Telegram, text)
            .await
        {
            Ok(_) => Reply::HostReplySent,
            Err(Refusal::NoChannel) => Reply::HostReplyNotLinked,
            Err(Refusal::Limit) => Reply::MessageLimit,
            Err(Refusal::TooLong) => Reply::HostReplyTooLong,
            Err(Refusal::Empty) => Reply::HostReplyEmpty,
            Err(Refusal::Internal) => Reply::HostReplyFailed,
            Err(Refusal::NotFound | Refusal::Closed) => Reply::HostReplyNotAllowed,
        };
    let what = match answer {
        Reply::HostReplySent => "reply_sent",
        Reply::HostReplyNotLinked => "reply_not_linked",
        Reply::MessageLimit => "message_limit",
        Reply::HostReplyTooLong => "reply_too_long",
        Reply::HostReplyEmpty => "reply_empty",
        Reply::HostReplyFailed => "reply_failed",
        _ => "reply_not_allowed",
    };
    reply(rt, pool, chat_id, bot_reply(answer, lang, ""), what);
}

/// ilia fork (inbox): an email reply, stored as row `id`, to the hosts'
/// chats, within the relay budget shared with Telegram texts. `from` says
/// whether the guest, an attendee or another address sent it.
pub(crate) async fn relay_guest_email(
    pool: &SqlitePool,
    booking_id: &str,
    id: &str,
    body: &str,
    from: super::MailFrom,
) {
    let Some(rt) = super::current() else {
        store::set_message_telegram_status(pool, id, "failed").await;
        return;
    };
    if rt.limits.relays.hit(&()) > rt.limits.relays.max() {
        tracing::warn!(target: "calrs::telegram", booking_id, "guest message relays used up this hour; stored, not relayed");
        store::set_message_telegram_status(pool, id, "failed").await;
        return;
    }
    let queued = rt.enqueue(Job::Event {
        pool: pool.clone(),
        booking_id: booking_id.to_string(),
        event: TelegramEvent::GuestMessage {
            id: id.to_string(),
            text: body.to_string(),
            email: Some(from),
        },
    });
    if !queued {
        store::set_message_telegram_status(pool, id, "failed").await;
    }
}
