//! What the bot does with an incoming update.
//!
//! Everything a Telegram user sends is untrusted. Only private text messages
//! from a person (not a bot) to the bot are read; groups, channels, edited
//! messages and member updates are ignored without a reply. The text is never
//! put into any message calrs sends: a command is matched, a payload is
//! checked against the token shape and looked up by hash, and the reply is a
//! fixed sentence.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use sqlx::SqlitePool;

use super::api::{Outgoing, Update};
use super::message::{self, Reply};
use super::store::GuestLink;
use super::tokens::{self, Kind};
use super::worker::Job;
use super::{Runtime, TelegramEvent};

/// Longest text read from a message.
const MAX_TEXT: usize = 4096;

/// A fixed-window counter per key.
pub(crate) struct Window<K> {
    max: u32,
    span: Duration,
    entries: Mutex<HashMap<K, (u32, Instant)>>,
}

impl<K: Eq + Hash + Clone> Window<K> {
    pub(crate) fn new(max: u32, span: Duration) -> Self {
        Self {
            max,
            span,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// The count in the current window.
    pub(crate) fn count(&self, key: &K) -> u32 {
        let now = Instant::now();
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        match entries.get(key) {
            Some((n, start)) if now.duration_since(*start) < self.span => *n,
            _ => 0,
        }
    }

    /// Count one more; returns the new count.
    pub(crate) fn hit(&self, key: &K) -> u32 {
        let now = Instant::now();
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if entries.len() > 10_000 {
            let span = self.span;
            entries.retain(|_, (_, start)| now.duration_since(*start) < span);
        }
        let entry = entries.entry(key.clone()).or_insert((0, now));
        if now.duration_since(entry.1) >= self.span {
            *entry = (0, now);
        }
        entry.0 = entry.0.saturating_add(1);
        entry.0
    }

    /// Whether the window is used up.
    pub(crate) fn full(&self, key: &K) -> bool {
        self.count(key) >= self.max
    }

    pub(crate) fn max(&self) -> u32 {
        self.max
    }
}

/// The bot's own limits. With 256-bit tokens guessing is hopeless anyway;
/// these keep a noisy chat from spending the bot's sending quota and the
/// database's time. Only per-chat windows ever refuse a `/start`: a shared
/// window would let a few strangers switch linking off for everyone.
pub(crate) struct Limits {
    /// Failed `/start` payloads per chat: 5 per 15 minutes.
    pub(crate) failures: Window<i64>,
    /// Any command per chat: 30 a minute.
    pub(crate) commands: Window<i64>,
    /// Token-shaped payloads that matched nothing, across all chats: 60 an
    /// hour, then one warning in the log. Never refuses anything.
    pub(crate) global_failures: Window<()>,
    /// Replies that say nothing new (help, "not linked", "already on"): once
    /// per chat per 10 minutes.
    pub(crate) help: Window<i64>,
    /// Bot replies across all chats: 60 a minute ([`Runtime::enqueue_reply`]).
    pub(crate) replies: Window<()>,
    /// ilia fork (messages): guest texts per chat, 5 per 10 minutes.
    pub(crate) guest_texts: Window<i64>,
    /// Guest texts relayed to hosts across all chats: 100 an hour.
    pub(crate) relays: Window<()>,
    global_warned: Mutex<Option<Instant>>,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            failures: Window::new(5, Duration::from_secs(15 * 60)),
            commands: Window::new(30, Duration::from_secs(60)),
            global_failures: Window::new(60, Duration::from_secs(3600)),
            help: Window::new(1, Duration::from_secs(10 * 60)),
            replies: Window::new(60, Duration::from_secs(60)),
            guest_texts: Window::new(5, Duration::from_secs(10 * 60)),
            relays: Window::new(100, Duration::from_secs(3600)),
            global_warned: Mutex::new(None),
        }
    }
}

impl Limits {
    /// Count a token-shaped payload that matched nothing; warn once an hour
    /// when there are many. A flood shows in the log; linking goes on.
    fn note_unknown_token(&self) {
        if self.global_failures.hit(&()) < self.global_failures.max() {
            return;
        }
        let mut warned = self.global_warned.lock().unwrap_or_else(|e| e.into_inner());
        let due = match *warned {
            Some(at) => at.elapsed() > Duration::from_secs(3600),
            None => true,
        };
        if due {
            tracing::warn!(
                target: "calrs::telegram",
                "many unknown Telegram links this hour (someone guessing or replaying old links?)"
            );
            *warned = Some(Instant::now());
        }
    }
}

/// The language of a reply to someone calrs knows nothing about yet: their
/// Telegram client's, mapped onto the shipped locales.
fn client_lang(code: Option<&str>) -> &'static str {
    crate::i18n::detect_from_accept_language(code)
}

/// How this calrs names itself in the help text: its host name.
pub(crate) fn site_name() -> String {
    crate::settings::base_url()
        .and_then(|base| {
            let rest = base.split_once("://").map(|(_, r)| r).unwrap_or(&base);
            rest.split(['/', ':']).next().map(str::to_string)
        })
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| "calrs".to_string())
}

/// A command, split from its arguments: `/start@our_bot payload`.
/// `None` when the text is not a command, or is addressed to another bot.
fn parse_command<'a>(text: &'a str, bot: Option<&str>) -> Option<(&'a str, &'a str)> {
    let text = text.strip_prefix('/')?;
    let (head, rest) = match text.split_once(char::is_whitespace) {
        Some((head, rest)) => (head, rest.trim()),
        None => (text, ""),
    };
    let command = match head.split_once('@') {
        Some((command, addressed)) => {
            if !bot.is_some_and(|b| b.eq_ignore_ascii_case(addressed)) {
                return None;
            }
            command
        }
        None => head,
    };
    Some((command, rest))
}

/// A reply to whoever wrote: one attempt, behind the outbox reserve and the
/// global reply budget.
pub(crate) fn reply(
    rt: &Runtime,
    pool: &SqlitePool,
    chat_id: i64,
    msg: Outgoing,
    what: &'static str,
) {
    rt.enqueue_reply(Job::Reply {
        pool: pool.clone(),
        chat_id,
        msg,
        what,
        attempts: 1,
    });
}

/// A reply that says nothing new: at most once per chat per 10 minutes.
pub(crate) fn quiet_reply(
    rt: &Runtime,
    pool: &SqlitePool,
    chat_id: i64,
    msg: Outgoing,
    what: &'static str,
) {
    if rt.limits.help.hit(&chat_id) <= rt.limits.help.max() {
        reply(rt, pool, chat_id, msg, what);
    }
}

/// Handle one raw update from `getUpdates`.
pub(crate) async fn handle_update(rt: &Runtime, pool: &SqlitePool, raw: &serde_json::Value) {
    let Ok(update) = serde_json::from_value::<Update>(raw.clone()) else {
        return;
    };
    let Some(message) = update.message else {
        return;
    };
    let Some(from) = message.from else {
        return;
    };
    if message.chat.kind != "private" || from.is_bot || message.chat.id != from.id {
        return;
    }
    let Some(text) = message.text else {
        return;
    };
    let reply_to = message.reply_to_message.map(|r| r.message_id);
    let text: String = text.trim().chars().take(MAX_TEXT).collect();
    let chat_id = message.chat.id;
    let lang = client_lang(from.language_code.as_deref());
    let username = from
        .username
        .as_deref()
        .filter(|u| super::username::valid(u));

    if rt.limits.commands.hit(&chat_id) > rt.limits.commands.max() {
        return;
    }
    let bot = rt.bot_username();
    let Some((command, payload)) = parse_command(&text, bot.as_deref()) else {
        // A command for another bot, or free text: a message about a
        // booking (ilia fork, `relay`), else the help, now and then.
        if !text.starts_with('/') {
            if super::relay::handle_text(rt, pool, chat_id, reply_to, &text, lang).await {
                return;
            }
            let help = message::bot_reply(Reply::Help, lang, &site_name());
            quiet_reply(rt, pool, chat_id, help, "help");
        }
        return;
    };
    match command.to_ascii_lowercase().as_str() {
        "start" if !payload.is_empty() => start(rt, pool, chat_id, payload, username, lang).await,
        "stop" => {
            let removed = super::store::unlink_chat(pool, chat_id).await;
            if removed > 0 {
                // Something changed: always say so.
                tracing::info!(target: "calrs::telegram", removed, "telegram chat unlinked by /stop");
                let msg = message::bot_reply(Reply::Stopped, lang, "");
                reply(rt, pool, chat_id, msg, "stop");
            } else {
                let msg = message::bot_reply(Reply::NotLinked, lang, "");
                quiet_reply(rt, pool, chat_id, msg, "stop");
            }
        }
        // A bare /start, /help, or a command the bot does not know.
        _ => {
            let help = message::bot_reply(Reply::Help, lang, &site_name());
            quiet_reply(rt, pool, chat_id, help, "help");
        }
    }
}

/// `/start <payload>`: link this chat to a booking or to a user.
async fn start(
    rt: &Runtime,
    pool: &SqlitePool,
    chat_id: i64,
    payload: &str,
    username: Option<&str>,
    lang: &'static str,
) {
    let limits = &rt.limits;
    // Over the per-chat limit: one "too many attempts", then silence, and
    // even a good token waits for the window to pass.
    if limits.failures.full(&chat_id) {
        if limits.failures.hit(&chat_id) == limits.failures.max() + 1 {
            reply(
                rt,
                pool,
                chat_id,
                message::bot_reply(Reply::RateLimited, lang, ""),
                "rate_limited",
            );
        }
        return;
    }
    let parsed = tokens::parse_payload(payload);
    let refused = match &parsed {
        Some((Kind::Booking, hash)) => {
            match super::store::consume_guest_token(pool, hash, chat_id, username).await {
                GuestLink::Linked(booking_id) => {
                    tracing::info!(target: "calrs::telegram", booking_id = %booking_id, "guest linked a telegram chat");
                    // The welcome, with the booking's state, goes to this chat
                    // as the booking's guest message; the hosts hear who linked.
                    rt.enqueue(Job::Event {
                        pool: pool.clone(),
                        booking_id,
                        event: TelegramEvent::GuestLinked,
                    });
                    None
                }
                // Opened the link again: not a failure, and nothing changed.
                GuestLink::AlreadyLinked => {
                    let msg = message::bot_reply(Reply::AlreadyLinked, lang, "");
                    quiet_reply(rt, pool, chat_id, msg, "already_linked");
                    None
                }
                // The booking stands, but its payment page shows a new
                // link now (the secret key changed): say where it is. Not
                // a guess, so not counted against the chat.
                GuestLink::Replaced => {
                    tracing::info!(target: "calrs::telegram", "a payment page's Telegram link from before a key change was opened");
                    let msg = message::bot_reply(Reply::LinkReplaced, lang, "");
                    reply(rt, pool, chat_id, msg, "link_replaced");
                    None
                }
                GuestLink::Refused => Some(Reply::LinkInvalid),
            }
        }
        Some((Kind::User, hash)) => {
            match super::store::consume_user_token(pool, hash, chat_id, username).await {
                Some((user_id, name, language)) => {
                    tracing::info!(target: "calrs::telegram", user_id = %user_id, "host linked a telegram chat");
                    let lang = crate::email::supported_language(language)
                        .and_then(|l| crate::i18n::supported_code(&l))
                        .unwrap_or("en");
                    // A proven token: retried like a notification.
                    rt.enqueue(Job::Reply {
                        pool: pool.clone(),
                        chat_id,
                        msg: message::host_linked(lang, &name),
                        what: "host_linked",
                        attempts: 3,
                    });
                    None
                }
                None => Some(Reply::HostLinkInvalid),
            }
        }
        None => Some(Reply::LinkInvalid),
    };
    if let Some(answer) = refused {
        limits.failures.hit(&chat_id);
        // Only token-shaped payloads say anything about guessing; "/start x"
        // from a shared t.me link is noise.
        if parsed.is_some() {
            limits.note_unknown_token();
        }
        reply(
            rt,
            pool,
            chat_id,
            message::bot_reply(answer, lang, ""),
            "link_invalid",
        );
    }
}
