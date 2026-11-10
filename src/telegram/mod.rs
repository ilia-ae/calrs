//! Telegram notifications (ilia fork; ilia-booking docs/operations.md,
//! «Уведомления в Telegram»).
//!
//! The guest types their Telegram username on the booking form (an event type
//! option, like the SMS phone field), and the confirmation page and email
//! offer a deep link `https://t.me/<bot>?start=<token>`. A bot cannot write to
//! a username, so the link is what connects: when the guest presses Start, the
//! bot receives `/start <token>` from their private chat and links that chat
//! to that one booking. From then on the guest hears about approval, decline,
//! cancellation, reschedule and the reminder there. Hosts link their own chat
//! from Profile & Settings the same way and hear about new bookings, requests
//! to approve, guest cancellations and moves.
//!
//! **Off unless `CALRS_TELEGRAM_BOT_TOKEN` is set.** Then `calrs serve` starts
//! a poller (`getUpdates` long polling: no webhook, no listening port) and an
//! outbox worker. Nothing else ever waits on Telegram: [`notify`] only queues,
//! and every failure is logged and dropped, as with SMS.
//!
//! Threat model, in short:
//! - the token is a secret that appears in every request URL: it is read from
//!   the environment only and redacted from every error (`config::redact`);
//! - deep-link tokens are 256-bit, single-use, stored as SHA-256 hashes, and a
//!   booking's expires with the meeting; a link is per booking and never by
//!   email address (an unproven address would hand someone else's later
//!   bookings, cancel links included, to whoever typed it);
//! - updates are untrusted: only private messages from people are read, the
//!   text is matched against fixed commands and never echoed into a message;
//! - messages are plain text (no `parse_mode`), and booking-token links go
//!   only to the guest's own linked chat;
//! - chat ids and tokens are never logged, shown on the dashboard or given to
//!   the MCP agent.
//!
//! The seam for other events: add a [`TelegramEvent`] variant and call
//! [`notify`]; the compiler lists the matches to fill in. TON payments use it
//! (`crate::web::ton_pay::telegram`): the guest of a priced booking links a
//! chat from the payment page and hears what to pay, a partial or late
//! payment, the confirmation and the release; hosts hear nothing until it is
//! paid, then "paid" and every payment problem.

pub(crate) mod api;
pub(crate) mod commands;
pub(crate) mod config;
pub(crate) mod events;
pub(crate) mod message;
pub(crate) mod poller;
pub(crate) mod store;
pub(crate) mod text;
pub(crate) mod tokens;
pub(crate) mod username;
pub(crate) mod worker;

#[cfg(test)]
pub(crate) mod fake;
#[cfg(test)]
mod tests;

use std::sync::{Arc, RwLock};
use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

pub(crate) use events::{PaymentRelease, TelegramEvent};

use api::BotApi;
use worker::Job;

/// The running feature: the API, what the bot is called, the outbox and the
/// bot's rate limits.
pub(crate) struct Runtime {
    pub(crate) api: Arc<dyn BotApi>,
    bot_username: RwLock<Option<String>>,
    outbox: mpsc::Sender<Job>,
    pub(crate) limits: commands::Limits,
    pub(crate) retry: worker::RetryPolicy,
}

impl Runtime {
    /// A runtime and the receiving end of its outbox (for [`worker::run`]).
    pub(crate) fn new(
        api: Arc<dyn BotApi>,
        bot_username: Option<String>,
        retry: worker::RetryPolicy,
    ) -> (Arc<Self>, mpsc::Receiver<Job>) {
        let (outbox, rx) = mpsc::channel(worker::QUEUE_CAPACITY);
        let rt = Arc::new(Self {
            api,
            bot_username: RwLock::new(bot_username.filter(|u| poller::valid_bot_username(u))),
            outbox,
            limits: commands::Limits::default(),
            retry,
        });
        (rt, rx)
    }

    pub(crate) fn bot_username(&self) -> Option<String> {
        self.bot_username
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub(crate) fn set_bot_username(&self, name: Option<String>) {
        *self.bot_username.write().unwrap_or_else(|e| e.into_inner()) = name;
    }

    /// Queue a job without waiting. A full queue drops the job (logged).
    /// Returns whether it was queued.
    pub(crate) fn enqueue(&self, job: Job) -> bool {
        match self.outbox.try_send(job) {
            Ok(()) => true,
            Err(mpsc::error::TrySendError::Full(_)) => {
                tracing::warn!(target: "calrs::telegram", "telegram outbox full; notification dropped");
                false
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                tracing::debug!(target: "calrs::telegram", "telegram outbox closed; notification dropped");
                false
            }
        }
    }

    /// Queue a reply to someone who wrote to the bot. Replies are untrusted
    /// traffic: they never take the last [`worker::REPLY_RESERVE`] places of
    /// the outbox (kept for booking notifications), and all chats together
    /// get at most [`commands::Limits::replies`] a minute. Returns whether it
    /// was queued.
    pub(crate) fn enqueue_reply(&self, job: Job) -> bool {
        if self.outbox.capacity() <= worker::REPLY_RESERVE {
            tracing::debug!(target: "calrs::telegram", "telegram outbox busy; bot reply dropped");
            return false;
        }
        if self.limits.replies.hit(&()) > self.limits.replies.max() {
            tracing::debug!(target: "calrs::telegram", "telegram reply budget used up; bot reply dropped");
            return false;
        }
        self.enqueue(job)
    }

    /// The deep link for a token, when the bot's username is known.
    pub(crate) fn deep_link(&self, token: &str) -> Option<String> {
        self.bot_username()
            .map(|bot| format!("https://t.me/{bot}?start={token}"))
    }
}

/// The installed runtime, when Telegram is on. Every entry point below reads
/// it; the functions they call take the runtime explicitly.
static RUNTIME: RwLock<Option<Arc<Runtime>>> = RwLock::new(None);

pub(crate) fn current() -> Option<Arc<Runtime>> {
    RUNTIME.read().unwrap_or_else(|e| e.into_inner()).clone()
}

pub(crate) fn install(rt: Option<Arc<Runtime>>) {
    *RUNTIME.write().unwrap_or_else(|e| e.into_inner()) = rt;
}

/// Whether Telegram is on in this process.
pub(crate) fn enabled() -> bool {
    current().is_some()
}

/// The bot's username, when known.
pub(crate) fn bot_username() -> Option<String> {
    current().and_then(|rt| rt.bot_username())
}

/// Tell whoever this event is for. Never waits on Telegram, never fails.
pub(crate) fn notify(pool: &SqlitePool, booking_id: &str, event: TelegramEvent) {
    if let Some(rt) = current() {
        rt.enqueue(Job::Event {
            pool: pool.clone(),
            booking_id: booking_id.to_string(),
            event,
        });
    }
}

/// Queue the reminder when the booking's guest has a linked chat. Returns
/// whether it was queued, so the reminder loop counts Telegram as a channel.
pub(crate) async fn notify_reminder(pool: &SqlitePool, booking_id: &str) -> bool {
    let Some(rt) = current() else {
        return false;
    };
    if store::linked_guest_chat(pool, booking_id).await.is_none() {
        return false;
    }
    rt.enqueue(Job::Event {
        pool: pool.clone(),
        booking_id: booking_id.to_string(),
        event: TelegramEvent::Reminder,
    })
}

/// A booking was just committed. Hosts are told; when the event type offers
/// Telegram to guests, a link row is stored (with the username they typed)
/// and the deep link for the confirmation page and email is returned.
pub(crate) async fn booking_created(
    pool: &SqlitePool,
    booking_id: &str,
    entered_username: Option<&str>,
    pending: bool,
) -> Option<String> {
    let rt = current()?;
    rt.enqueue(Job::Event {
        pool: pool.clone(),
        booking_id: booking_id.to_string(),
        event: TelegramEvent::BookingCreated { pending },
    });
    guest_link(&rt, pool, booking_id, entered_username).await
}

/// A priced booking (TON, ilia-booking docs/ton.md) was just committed. It
/// awaits its payment, so hosts hear nothing yet (they are told once it is
/// paid). When the event type offers Telegram, the link row is stored under
/// `token`, which the payment page derives again on every load
/// ([`offered_priced_link`]).
pub(crate) async fn priced_booking_created(
    pool: &SqlitePool,
    booking_id: &str,
    entered_username: Option<&str>,
    token: &str,
) {
    if let Some(rt) = current() {
        link_with_token(&rt, pool, booking_id, entered_username, token).await;
    }
}

/// The deep link of a priced booking while its link row stands (unexpired,
/// the booking pending or confirmed). Its token is derived from the payment
/// link and the server's secret key (TON,
/// `web::ton_pay::telegram::link_token`). When the row was made with
/// another token, the key changed since (a rotated or restored
/// `secret.key`): the row is re-keyed to `token`, so the payment page and
/// its email offer a link that works, and the old link gets "open the
/// payment page again" from the bot ([`store::GuestLink::Replaced`]).
///
/// Read first, write only to re-key: most loads find no row (the event
/// type offers no Telegram, the booking ended) or the row already on
/// `token`, and an UPDATE would take SQLite's write lock for nothing.
pub(crate) async fn offered_priced_link(
    pool: &SqlitePool,
    booking_id: &str,
    token: &str,
) -> Option<String> {
    let rt = current()?;
    let hash = tokens::hash(token);
    if store::live_link_hash(pool, booking_id).await? == hash {
        return rt.deep_link(token);
    }
    if store::rekey_guest_link(pool, booking_id, &hash).await {
        tracing::info!(target: "calrs::telegram", booking_id, "telegram link of a priced booking re-derived (the secret key changed); its old link now points to the payment page");
        return rt.deep_link(token);
    }
    // A concurrent load (another tab, the "email me the link" POST)
    // re-keyed it to this same token between the read and the UPDATE.
    store::link_offered(pool, booking_id, &hash)
        .await
        .then(|| rt.deep_link(token))
        .flatten()
}

/// The link row and deep link of a new booking (see [`booking_created`]).
pub(crate) async fn guest_link(
    rt: &Runtime,
    pool: &SqlitePool,
    booking_id: &str,
    entered_username: Option<&str>,
) -> Option<String> {
    let (token, _) = tokens::new_token(tokens::Kind::Booking);
    link_with_token(rt, pool, booking_id, entered_username, &token).await
}

async fn link_with_token(
    rt: &Runtime,
    pool: &SqlitePool,
    booking_id: &str,
    entered_username: Option<&str>,
    token: &str,
) -> Option<String> {
    let (guest_mode, end) = store::booking_mode_and_end(pool, booking_id).await?;
    if guest_mode == username::mode::OFF || end <= chrono::Utc::now() {
        return None;
    }
    let entered = entered_username.filter(|u| username::valid(u));
    if !store::create_guest_link(pool, booking_id, entered, &tokens::hash(token), end).await {
        return None;
    }
    rt.deep_link(token)
}

/// The background tasks of `calrs serve`.
pub(crate) struct TelegramHandle {
    rt: Option<Arc<Runtime>>,
    tasks: Vec<JoinHandle<()>>,
}

impl TelegramHandle {
    fn off() -> Self {
        Self {
            rt: None,
            tasks: Vec::new(),
        }
    }

    /// Stop, after the shutdown flag is set and the web server, the agent
    /// and the TON watcher have finished (they may still queue
    /// notifications): new notifications
    /// are refused, what is queued is sent, for at most `grace`. A task still
    /// running at the deadline is aborted.
    pub(crate) async fn finish(self, grace: Duration) {
        let deadline = tokio::time::Instant::now() + grace;
        if let Some(rt) = &self.rt {
            {
                let mut installed = RUNTIME.write().unwrap_or_else(|e| e.into_inner());
                if installed.as_ref().is_some_and(|i| Arc::ptr_eq(i, rt)) {
                    *installed = None;
                }
            }
            let _ = tokio::time::timeout_at(deadline, rt.outbox.send(Job::Shutdown)).await;
        }
        for mut task in self.tasks {
            if tokio::time::timeout_at(deadline, &mut task).await.is_err() {
                task.abort();
                tracing::warn!(target: "calrs::telegram", "telegram tasks still running at shutdown");
            }
        }
    }
}

/// Start the bot when `CALRS_TELEGRAM_BOT_TOKEN` is set (`calrs serve` only).
/// The retention purge runs either way: what was stored while Telegram was
/// on is forgotten on schedule after it is switched off.
pub(crate) async fn spawn(pool: SqlitePool, stop: watch::Receiver<bool>) -> TelegramHandle {
    let purge = tokio::spawn(worker::purge_loop(pool.clone(), stop.clone()));
    let Some(config) = config::from_env() else {
        let mut off = TelegramHandle::off();
        off.tasks.push(purge);
        return off;
    };
    // The stored offset and username belong to one bot: a token for
    // another bot starts over (no network call needed, the id is in it).
    if let Some(bot_id) = config.bot_id() {
        store::claim_bot(&pool, bot_id).await;
    }
    let cached = store::cached_bot_username(&pool).await;
    let api: Arc<dyn BotApi> = Arc::new(api::HttpApi::new(config));
    let (rt, rx) = Runtime::new(api, cached, worker::RetryPolicy::default());
    install(Some(rt.clone()));
    tracing::info!(target: "calrs::telegram", "telegram notifications on");
    let worker = tokio::spawn(worker::run(rt.clone(), rx));
    let poller = tokio::spawn(poller::run(
        rt.clone(),
        pool,
        stop,
        poller::PollPolicy::default(),
    ));
    TelegramHandle {
        rt: Some(rt),
        tasks: vec![worker, poller, purge],
    }
}
