//! ilia fork: owner — system — client (ilia-booking docs/decisions.md,
//! 2026-11-10; docs/operations.md, «Почтовый ящик системы»).
//!
//! The system writes to the guest itself, and the owner writes to the guest
//! through the system: no guest-facing mail, `.ics` or page carries the
//! owner's personal address.
//!
//! - **Outgoing** ([`reply`]): every guest email goes out from the SMTP
//!   address (with the host's display name, upstream #221). Its Reply-To is
//!   the system mailbox: `booking+<token>@domain` in plus mode, else the bare
//!   address; with no mailbox configured there is no Reply-To at all, so a
//!   reply goes to the system From. Its Message-ID carries the booking's
//!   token (`<calrs.<token>.<random>@domain>`), so In-Reply-To and
//!   References identify the booking whatever the guest's client does with
//!   Reply-To. A guest `.ics` names the system address as ORGANIZER; the
//!   host's own copies (CalDAV write-back, host emails) keep the host, and
//!   so do a booking's guest invites when its first one named the host
//!   (bookings made before this, [`organizer`]).
//! - **Inbound** ([`poller`]): off unless `CALRS_INBOX_IMAP_HOST`, `_USER`
//!   and `_PASSWORD` are set ([`config`]). `calrs serve` then keeps one IMAPS
//!   connection ([`imap`], [`tls`]) and polls the folder every few minutes.
//!   A reply is parsed ([`mime`]), filtered ([`filter`]: autoreplies,
//!   bounces, lists, iTIP answers, loops), matched to a booking by its token
//!   ([`matcher`]), cut to what the guest wrote ([`clean`]) and stored in the
//!   booking's thread as `via = 'email'` ([`store`]). The hosts' Telegram
//!   chats hear about it like a Telegram text. HTML is never rendered,
//!   stored or forwarded; text, addresses and the password are never logged.
//!
//! **Tokens** are `base32(HMAC-SHA256(inbox_key, "calrs-reply-v1\0" || uid))`
//! cut to 16 characters (80 bits), with `inbox_key` derived from the server's
//! secret key ([`init_key`]). No table: matching recomputes them over the
//! live bookings.
//!
//! **Relay mode** ([`relay`]) is always on in a real build. In tests it is
//! off unless a test sets [`test_mode`], so upstream's tests (which expect
//! Reply-To the host, upstream #221) hold, and the fork's tests switch it on.

pub(crate) mod clean;
pub(crate) mod config;
pub(crate) mod filter;
pub(crate) mod imap;
pub(crate) mod matcher;
pub(crate) mod mime;
pub(crate) mod organizer;
pub(crate) mod poller;
pub(crate) mod reply;
pub(crate) mod store;
pub(crate) mod tls;

#[cfg(test)]
pub(crate) mod fake;
#[cfg(test)]
mod tests;

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::SqlitePool;
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub(crate) use config::Config;

/// The key tokens are made with, derived once from the server's secret key.
static KEY: OnceLock<[u8; 32]> = OnceLock::new();

fn derive_key(secret_key: &[u8; 32]) -> [u8; 32] {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret_key)
        .expect("HMAC accepts a key of any length");
    mac.update(b"calrs-inbox-key-v1");
    mac.finalize().into_bytes().into()
}

/// Called by `main` right after the secret key is loaded, for every command:
/// any of them may send a guest email.
pub(crate) fn init_key(secret_key: &[u8; 32]) {
    let _ = KEY.set(derive_key(secret_key));
}

/// The token key. In tests, the one derived from the test secret key (all
/// zeroes) unless `init_key` ran.
pub(crate) fn key() -> Option<[u8; 32]> {
    #[cfg(test)]
    {
        Some(KEY.get().copied().unwrap_or_else(|| derive_key(&[0u8; 32])))
    }
    #[cfg(not(test))]
    {
        KEY.get().copied()
    }
}

#[cfg(test)]
thread_local! {
    /// Tests only: relay mode and the inbox configuration of this thread
    /// (`#[tokio::test]` runs on one thread, spawned email tasks included).
    static TEST: std::cell::RefCell<(bool, Option<Arc<Config>>)> =
        const { std::cell::RefCell::new((false, None)) };
}

/// Whether guest mail goes through the system: always in a real build.
pub(crate) fn relay() -> bool {
    #[cfg(test)]
    {
        TEST.with(|t| t.borrow().0)
    }
    #[cfg(not(test))]
    {
        true
    }
}

/// The inbox configuration, read once from the environment; `None`: off.
pub(crate) fn config() -> Option<Arc<Config>> {
    #[cfg(test)]
    {
        TEST.with(|t| t.borrow().1.clone())
    }
    #[cfg(not(test))]
    {
        static CONFIG: OnceLock<Option<Arc<Config>>> = OnceLock::new();
        CONFIG
            .get_or_init(|| config::from_env().map(Arc::new))
            .clone()
    }
}

/// Whether the system mailbox is read (replies reach the thread).
pub(crate) fn enabled() -> bool {
    relay() && config().is_some()
}

/// Tests only: relay mode and an inbox configuration for this thread, until
/// the guard drops.
#[cfg(test)]
pub(crate) fn test_mode(relay: bool, config: Option<Config>) -> TestGuard {
    let previous = TEST.with(|t| t.replace((relay, config.map(Arc::new))));
    TestGuard(Some(previous))
}

#[cfg(test)]
pub(crate) struct TestGuard(Option<(bool, Option<Arc<Config>>)>);

#[cfg(test)]
impl Drop for TestGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.0.take() {
            TEST.with(|t| *t.borrow_mut() = previous);
        }
    }
}

/// The inbox task of `calrs serve`.
pub(crate) struct InboxHandle {
    task: Option<JoinHandle<()>>,
}

impl InboxHandle {
    /// Stop after the shutdown flag is set: the poller logs out at its next
    /// await point. A task still running after `grace` is aborted.
    pub(crate) async fn finish(self, grace: Duration) {
        if let Some(mut task) = self.task {
            if tokio::time::timeout(grace, &mut task).await.is_err() {
                task.abort();
                tracing::warn!(target: "calrs::inbox", "inbox poller still running at shutdown");
            }
        }
    }
}

/// Start the poller when the inbox is configured (`calrs serve` only).
/// Either way the SMTP From is loaded once, for guest invites' ORGANIZER.
pub(crate) async fn spawn(
    pool: SqlitePool,
    secret_key: [u8; 32],
    stop: watch::Receiver<bool>,
) -> InboxHandle {
    // The SMTP From is the ORGANIZER of guest invites when no mailbox is
    // read (remembered by `load_smtp_config`, so the ICS download has it
    // before any email went out) and one of our own addresses (loops).
    let smtp_from = crate::email::load_smtp_config(&pool, &secret_key)
        .await
        .ok()
        .flatten()
        .map(|smtp| smtp.from_email);
    let Some(config) = config() else {
        return InboxHandle { task: None };
    };
    let Some(key) = key() else {
        tracing::warn!(target: "calrs::inbox", "inbox key missing; inbox off");
        return InboxHandle { task: None };
    };
    let ours = filter::Ours::new(&config.address, smtp_from.as_deref());
    tracing::info!(
        target: "calrs::inbox",
        port = config.port,
        poll_min = config.poll.as_secs() / 60,
        plus = config.plus,
        moves = config.done_folder.is_some(),
        "inbox on"
    );
    let ctx = Arc::new(poller::Ctx::new(pool, config, key, ours));
    let connect = tls::connector(ctx.config.clone());
    let task = tokio::spawn(poller::run(ctx, connect, poller::Policy::default(), stop));
    InboxHandle { task: Some(task) }
}
