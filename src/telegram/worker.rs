//! The outbox: one task that turns queued events into messages and sends
//! them. Callers never wait on Telegram (`notify` only queues), and a failure
//! here is logged and dropped: a booking never depends on Telegram.
//!
//! Logs name the event, the booking and the outcome, never a chat id, a token
//! or a message text.

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use chrono_tz::Tz;
use sqlx::SqlitePool;
use tokio::sync::mpsc;

use super::api::{Outgoing, TgError};
use super::events::TelegramEvent;
use super::message::{self, GuestLinks, GuestPayment, GuestView, HostPayment, HostView};
use super::store::{self, BookingRow};
use super::text;
use super::Runtime;

/// Jobs waiting for the worker. Bounded: when Telegram is slow and the queue
/// is full, a notification is dropped rather than held in memory.
pub(crate) const QUEUE_CAPACITY: usize = 1000;

/// Places of the outbox bot replies never take: a flood of `/help` from
/// strangers cannot push out a booking's notifications.
pub(crate) const REPLY_RESERVE: usize = QUEUE_CAPACITY / 2;

/// One unit of work. Each carries the pool of the database it came from.
pub(crate) enum Job {
    Event {
        pool: SqlitePool,
        booking_id: String,
        event: TelegramEvent,
    },
    /// A bot reply. `attempts`: 1 for replies to anyone who writes to the
    /// bot (never worth a wait that holds up booking notifications), 3 for
    /// the reply to a proven host token.
    Reply {
        pool: SqlitePool,
        chat_id: i64,
        msg: Outgoing,
        what: &'static str,
        attempts: u8,
    },
    /// Everything queued before this has been handled: stop.
    Shutdown,
}

/// How hard to try.
#[derive(Debug, Clone)]
pub(crate) struct RetryPolicy {
    /// Waits before the second and the third attempt.
    pub(crate) delays: [Duration; 2],
    /// One second of Telegram's `retry_after`, in real time (shorter in tests).
    pub(crate) retry_after_unit: Duration,
    /// Longest `retry_after` honoured.
    pub(crate) max_retry_after: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            delays: [Duration::from_secs(2), Duration::from_secs(10)],
            retry_after_unit: Duration::from_secs(1),
            max_retry_after: Duration::from_secs(60),
        }
    }
}

/// Run until [`Job::Shutdown`] (or every sender is gone).
pub(crate) async fn run(rt: Arc<Runtime>, mut rx: mpsc::Receiver<Job>) {
    while let Some(job) = rx.recv().await {
        if matches!(job, Job::Shutdown) {
            break;
        }
        process(&rt, job).await;
    }
    tracing::debug!(target: "calrs::telegram", "telegram outbox stopped");
}

/// The hourly retention purge ([`store::purge`]), until the shutdown flag
/// is set. Started by `telegram::spawn` whether or not Telegram is on: links
/// stored while it was on must still be forgotten after it is switched off.
pub(crate) async fn purge_loop(pool: SqlitePool, mut stop: tokio::sync::watch::Receiver<bool>) {
    let mut tick = tokio::time::interval(Duration::from_secs(3600));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        if *stop.borrow() {
            return;
        }
        tokio::select! {
            _ = tick.tick() => store::purge(&pool, Utc::now()).await,
            _ = stop.changed() => return,
        }
    }
}

/// Handle one job to its end.
pub(crate) async fn process(rt: &Runtime, job: Job) {
    match job {
        Job::Event {
            pool,
            booking_id,
            event,
        } => process_event(rt, &pool, &booking_id, &event).await,
        Job::Reply {
            pool,
            chat_id,
            msg,
            what,
            attempts,
        } => {
            deliver(rt, &pool, chat_id, &msg, what, None, attempts).await;
        }
        Job::Shutdown => {}
    }
}

async fn process_event(rt: &Runtime, pool: &SqlitePool, booking_id: &str, event: &TelegramEvent) {
    let Some(booking) = store::booking_row(pool, booking_id).await else {
        tracing::debug!(target: "calrs::telegram", booking_id, "booking gone; nothing to send");
        return;
    };
    // A moved meeting moves the link's expiry with it.
    if matches!(
        event,
        TelegramEvent::Rescheduled { .. } | TelegramEvent::Approved { reschedule: true }
    ) {
        store::move_link_expiry(pool, booking_id, booking.end).await;
    }
    // TON (ilia-booking docs/ton.md): the invoice as it is now. A guest who
    // links a chat to a booking that awaits its payment is told what to pay;
    // hosts hear about such a booking only through payment events.
    let payment = crate::web::ton_pay::telegram::facts(pool, booking_id).await;
    let event = match event {
        TelegramEvent::GuestLinked if payment.as_ref().is_some_and(|p| p.open()) => {
            &TelegramEvent::PaymentDue
        }
        other => other,
    };
    let mut audience = event.audience();
    if !event.is_payment()
        && payment
            .as_ref()
            .is_some_and(|p| p.hosts_unaware(&booking.status))
    {
        audience.hosts = false;
    }
    let base = crate::settings::base_url().map(|b| b.trim_end_matches('/').to_string());
    if audience.guest {
        if let Some(chat_id) = booking.guest_chat {
            let lang = crate::email::supported_language(booking.language.clone())
                .unwrap_or_else(|| "en".to_string());
            let mut view = guest_view(pool, &booking, base.as_deref(), &lang).await;
            view.payment = payment
                .as_ref()
                .map(|p| guest_payment(p, &booking.guest_timezone, base.as_deref(), &lang));
            if let Some(msg) = message::guest(event, &view, &lang) {
                deliver(rt, pool, chat_id, &msg, event.kind(), Some(booking_id), 3).await;
            }
        }
    }
    if audience.hosts {
        for host in store::host_chats(pool, booking_id).await {
            let lang = crate::email::supported_language(host.language.clone())
                .unwrap_or_else(|| "en".to_string());
            let mut view = host_view(&booking, &host.timezone, base.as_deref(), &lang);
            view.payment = payment.as_ref().map(host_payment);
            if let Some(msg) = message::host(event, &view, &lang) {
                deliver(
                    rt,
                    pool,
                    host.chat_id,
                    &msg,
                    event.kind(),
                    Some(booking_id),
                    3,
                )
                .await;
            }
        }
    }
    if matches!(
        event,
        TelegramEvent::Cancelled { .. }
            | TelegramEvent::Declined { .. }
            | TelegramEvent::Expired
            | TelegramEvent::PaymentExpired { .. }
    ) {
        store::end_link(pool, booking_id, Utc::now()).await;
    }
}

/// The invoice as the guest reads it: the deadline in their zone, and the
/// payment page (this view only ever reaches the guest's own chat).
fn guest_payment(
    p: &crate::web::ton_pay::telegram::PaymentFacts,
    tz_name: &str,
    base: Option<&str>,
    lang: &str,
) -> GuestPayment {
    let tz: Tz = tz_name.parse().unwrap_or(Tz::UTC);
    let deadline = p
        .expires()
        .map(|t| {
            let local = t.with_timezone(&tz);
            message::when(
                lang,
                local.date_naive(),
                &local.format("%H:%M").to_string(),
                tz.name(),
            )
        })
        .unwrap_or_default();
    GuestPayment {
        amount: p.amount(),
        paid: p.paid(),
        rest: p.rest(),
        usd: p.usd(),
        deadline,
        open: p.open(),
        partly_paid: p.paid_nano > 0,
        pay_url: p.pay_url(base),
    }
}

/// The invoice as a host reads it: amounts only.
fn host_payment(p: &crate::web::ton_pay::telegram::PaymentFacts) -> HostPayment {
    HostPayment {
        amount: p.amount(),
        paid: p.paid(),
        usd: p.usd(),
    }
}

/// The meeting's start in `tz`, written out in `lang`.
fn when_in(booking: &BookingRow, tz_name: &str, lang: &str) -> String {
    let tz: Tz = tz_name.parse().unwrap_or(Tz::UTC);
    let local = booking.start.with_timezone(&tz);
    message::when(
        lang,
        local.date_naive(),
        &local.format("%H:%M").to_string(),
        tz.name(),
    )
}

async fn guest_view(
    pool: &SqlitePool,
    booking: &BookingRow,
    base: Option<&str>,
    lang: &str,
) -> GuestView {
    let host = crate::web::booking_host_identity(pool, &booking.uid)
        .await
        .map(|(name, _, _)| name)
        .unwrap_or_default();
    let pending = booking.status == "pending";
    let link = |path: String| base.map(|b| format!("{b}{path}"));
    let rebook = match base {
        Some(_) => store::rebook_path(pool, &booking.event_type_id)
            .await
            .and_then(link),
        None => None,
    };
    let links = GuestLinks {
        cancel: booking
            .cancel_token
            .as_ref()
            .and_then(|t| link(format!("/booking/cancel/{t}"))),
        reschedule: booking
            .reschedule_token
            .as_ref()
            .and_then(|t| link(format!("/booking/reschedule/{t}"))),
        calendar: booking
            .cancel_token
            .as_ref()
            .filter(|_| booking.status == "confirmed")
            .and_then(|t| link(format!("/booking/ics/{t}"))),
        pick_time: booking
            .reschedule_token
            .as_ref()
            .and_then(|t| link(format!("/booking/reschedule/{t}"))),
        rebook,
    };
    GuestView {
        event: text::clean(&booking.title, text::TITLE_MAX),
        when: when_in(booking, &booking.guest_timezone, lang),
        host: text::clean(&host, text::NAME_MAX),
        pending,
        minutes_to_start: (booking.start - Utc::now()).num_minutes(),
        location: booking
            .location
            .as_deref()
            .filter(|_| booking.status == "confirmed")
            .map(|l| text::clean(l, text::LOCATION_MAX))
            .filter(|l| !l.is_empty()),
        links,
        payment: None,
    }
}

fn host_view(booking: &BookingRow, host_tz: &str, base: Option<&str>, lang: &str) -> HostView {
    let username = |u: &Option<String>| {
        u.as_deref()
            .filter(|u| super::username::valid(u))
            .map(str::to_string)
    };
    HostView {
        event: text::clean(&booking.title, text::TITLE_MAX),
        when: when_in(booking, host_tz, lang),
        guest: text::defang(&text::clean(&booking.guest_name, text::GUEST_NAME_MAX)),
        email: text::guest_email(&booking.guest_email),
        guest_linked: booking.guest_chat.is_some(),
        linked_as: username(&booking.telegram_username),
        entered: username(&booking.entered_username),
        bookings_url: base.map(|b| format!("{b}/dashboard/bookings")),
        payment: None,
    }
}

/// Send with up to `attempts` attempts (at most three). `true` when Telegram
/// accepted it.
pub(crate) async fn deliver(
    rt: &Runtime,
    pool: &SqlitePool,
    chat_id: i64,
    msg: &Outgoing,
    what: &'static str,
    booking_id: Option<&str>,
    attempts: u8,
) -> bool {
    let booking_id = booking_id.unwrap_or("");
    let attempts = usize::from(attempts.clamp(1, 3));
    for attempt in 0..attempts {
        let wait = match rt.api.send_message(chat_id, msg).await {
            Ok(()) => {
                tracing::info!(target: "calrs::telegram", what, booking_id, "telegram message sent");
                return true;
            }
            Err(TgError::Forbidden(_)) => {
                let removed = store::unlink_chat(pool, chat_id).await;
                tracing::warn!(
                    target: "calrs::telegram",
                    what,
                    booking_id,
                    removed,
                    "telegram chat blocked the bot; its links were removed"
                );
                return false;
            }
            Err(e @ (TgError::BadRequest(_) | TgError::Unauthorized | TgError::Conflict(_))) => {
                tracing::warn!(
                    target: "calrs::telegram",
                    what,
                    booking_id,
                    kind = e.kind(),
                    error = %e,
                    "telegram message refused"
                );
                return false;
            }
            Err(TgError::RateLimited { retry_after }) => rt
                .retry
                .retry_after_unit
                .saturating_mul(u32::try_from(retry_after).unwrap_or(u32::MAX))
                .min(rt.retry.max_retry_after),
            Err(e) => {
                tracing::warn!(
                    target: "calrs::telegram",
                    what,
                    booking_id,
                    attempt = attempt + 1,
                    kind = e.kind(),
                    error = %e,
                    "telegram send failed"
                );
                rt.retry.delays.get(attempt).copied().unwrap_or_default()
            }
        };
        if attempt + 1 < attempts {
            tokio::time::sleep(wait).await;
        }
    }
    tracing::warn!(
        target: "calrs::telegram",
        what,
        booking_id,
        attempts,
        "telegram message not delivered"
    );
    false
}
