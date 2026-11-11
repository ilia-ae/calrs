//! The session loop of `calrs serve`: one IMAPS connection, reused across
//! polls (the working agreement: one connection per run, never one per
//! command).
//!
//! connect → greeting → CAPABILITY → LOGIN → SELECT, then every poll:
//! NOOP → `UID SEARCH UID <last+1>:<last+1000>` → per message the size,
//! then `BODY.PEEK[]<0.262144>` → store, relay, `\Seen` or move. LOGOUT on
//! stop.
//!
//! Searches are bounded to [`SEARCH_SPAN`] UIDs, so their one `* SEARCH`
//! line stays far below the client's 64 KiB line cap however large a
//! backlog grows (about 7 bytes a UID: an unbounded search failed every
//! poll from ~9,000 messages on). An empty range below the last UID is
//! skipped, and the state saved, within the same poll.
//!
//! Failures: a refused LOGIN waits an hour (Kerio locks accounts after
//! failed logins) and is never retried at once; a refused SELECT (a wrong
//! `CALRS_INBOX_FOLDER`) is logged as such, never as a login problem; any
//! other error drops the session and reconnects after 1, 2, 4 … 30
//! minutes. The log says whether the transport (connect, TLS) or the
//! service (NO, BAD, protocol) failed.
//!
//! First run in a folder (outside shared mode): only the unseen mail
//! among the last [`SEARCH_SPAN`] UIDs is taken, and the history before it
//! is skipped. The state is saved only once all of it was processed, so a
//! session that breaks half-way takes the rest next time. A UIDVALIDITY
//! change starts again from UIDNEXT. Later polls take at most
//! [`Policy::batch`] messages.
//!
//! Shared mode (`CALRS_INBOX_SHARED`, a mailbox that is also a person's
//! own): per new message only `RFC822.SIZE` and the headers the match needs
//! ([`MATCH_HEADERS`], at most [`HEADERS_MAX`] bytes) are fetched, and the
//! match runs on them alone, and only from the guest's or an attendee's
//! address (`matcher::find`, `known_only`). Only a message that matches a
//! booking is fetched in full, filtered, stored, flagged or moved; any
//! other one is counted as `unmatched` and nothing else: no full fetch, no
//! flag, no move, no log line of its own. A matched message that is our
//! own mail (`filter::Drop::Loop`) is counted as `loop` and left as it is.
//! The first run starts at UIDNEXT: the mailbox's history is never
//! searched.
//!
//! Logs carry counts by outcome, UIDs, sizes and booking ids; never an
//! address, a subject or the text.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::watch;

use super::imap::{self, Client, Selected};
use super::tls::{ConnectError, Connector};
use super::{clean, filter, matcher, mime, store, Config};
use crate::telegram::commands::Window;
use crate::telegram::store as tg_store;
use crate::web::messages;

/// The largest message read; a larger one is left for a human.
pub(crate) const MESSAGE_MAX: u32 = 256 * 1024;
/// Mails one sender may have stored per booking and hour; past it they are
/// left unread for a human.
pub(crate) const SENDER_PER_HOUR: u32 = 20;
/// Matched mails stored per hour over all bookings (a flood); past it they
/// are left unread for a human.
pub(crate) const MATCHED_PER_HOUR: u32 = 200;
/// UIDs one `UID SEARCH` covers.
pub(crate) const SEARCH_SPAN: u32 = 1000;
/// Empty spans one poll skips before it waits for the next.
const EMPTY_SPANS_PER_POLL: u32 = 100;
/// Shared mode: the headers fetched to match a new message (`matcher`
/// reads the recipients, In-Reply-To and References, Subject and From;
/// Message-ID only matters to a matched message, read in full).
pub(crate) const MATCH_HEADERS: &str =
    "FROM TO CC DELIVERED-TO X-ORIGINAL-TO ENVELOPE-TO IN-REPLY-TO REFERENCES SUBJECT";
/// Shared mode: the most of those headers read (a partial fetch, so one
/// crafted header can never exceed the client's literal cap and wedge the
/// session on its UID).
pub(crate) const HEADERS_MAX: u32 = 64 * 1024;

/// Timing of the loop.
#[derive(Debug, Clone)]
pub(crate) struct Policy {
    /// Overrides the configured poll interval (tests).
    pub(crate) poll: Option<Duration>,
    pub(crate) backoff_start: Duration,
    pub(crate) backoff_max: Duration,
    pub(crate) auth_backoff: Duration,
    pub(crate) batch: usize,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            poll: None,
            backoff_start: Duration::from_secs(60),
            backoff_max: Duration::from_secs(30 * 60),
            auth_backoff: Duration::from_secs(3600),
            batch: 50,
        }
    }
}

/// What the poller works with.
pub(crate) struct Ctx {
    pub(crate) pool: SqlitePool,
    pub(crate) config: Arc<Config>,
    pub(crate) key: [u8; 32],
    pub(crate) ours: filter::Ours,
    /// Per booking and From, counted only for matched mail.
    senders: Window<[u8; 32]>,
    matched: Window<()>,
}

impl Ctx {
    pub(crate) fn new(
        pool: SqlitePool,
        config: Arc<Config>,
        key: [u8; 32],
        ours: filter::Ours,
    ) -> Self {
        Self {
            pool,
            config,
            key,
            ours,
            senders: Window::new(SENDER_PER_HOUR, Duration::from_secs(3600)),
            matched: Window::new(MATCHED_PER_HOUR, Duration::from_secs(3600)),
        }
    }
}

/// Outcomes of one poll, by reason.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Stats(pub(crate) BTreeMap<&'static str, u32>);

impl Stats {
    fn add(&mut self, what: &'static str) {
        *self.0.entry(what).or_default() += 1;
    }
}

/// What to do with a message once read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Disposition {
    pub(crate) reason: &'static str,
    /// Mark it seen (or move it); otherwise it is left for a human.
    pub(crate) handled: bool,
}

const fn handled(reason: &'static str) -> Disposition {
    Disposition {
        reason,
        handled: true,
    }
}

const fn leave(reason: &'static str) -> Disposition {
    Disposition {
        reason,
        handled: false,
    }
}

/// Decide on one raw message: filter, match, clean, store, relay.
///
/// Mail that may matter to a person and was not stored is left unread:
/// unmatched, a pending TON booking's (the Telegram rule keeps it out of
/// the thread, not out of sight), and over a limit. Only what nobody needs
/// (automatic mail, empty, a duplicate) is marked handled without storing.
/// In shared mode the match comes first: the filters only ever mark mail
/// that is about a booking, and never our own mail.
pub(crate) async fn handle(ctx: &Ctx, raw: &[u8]) -> Disposition {
    let Some(m) = mime::parse(raw) else {
        return leave("unreadable");
    };
    let shared = ctx.config.shared;
    if !shared {
        if let Some(why) = filter::check(&m, &ctx.ours) {
            return handled(why.as_str());
        }
    }
    let pool = &ctx.pool;
    let candidates = matcher::candidates(pool).await;
    let Some(hit) = matcher::find(&m, &candidates, &ctx.key, ctx.ours.domain(), shared) else {
        return leave("unmatched");
    };
    if shared {
        match filter::check(&m, &ctx.ours) {
            // Our own mail in the person's mailbox (their copy of what
            // calrs sent, or the mailbox given as a guest's address): never
            // a guest writing, and theirs to read. Left as it is.
            Some(filter::Drop::Loop) => return leave("loop"),
            Some(why) => return handled(why.as_str()),
            None => {}
        }
    }
    let booking_id = hit.booking_id.as_str();
    let Some(booking) = tg_store::booking_row(pool, booking_id).await else {
        return leave("unmatched");
    };
    // The Telegram rule: an unpaid booking takes no guest text until a host
    // wrote to its guest (hosts never hear of unpaid bookings otherwise).
    // The mail stays unread in the system mailbox, for a person to see.
    let unpaid = crate::web::ton_pay::telegram::facts(pool, booking_id)
        .await
        .is_some_and(|p| p.hosts_unaware(&booking.status))
        && !messages::host_wrote(pool, booking_id).await;
    if unpaid {
        return leave("unpaid");
    }
    let (body, shortened) = clean::finish(m.text.as_deref().unwrap_or(""));
    if body.is_empty() {
        return handled("empty");
    }
    let hash = store::inbound_hash(m.message_id.as_deref(), raw);
    if store::seen(pool, booking_id, &hash).await {
        return handled("duplicate");
    }
    // Limits count matched mail only, per booking and From (unmatched junk
    // with a guest's From never uses up that guest's quota), and leave what
    // is over them unread.
    let from = m.from.as_deref().unwrap_or("");
    let mut key = Sha256::new();
    key.update(booking_id.as_bytes());
    key.update([0u8]);
    key.update(from.as_bytes());
    let key: [u8; 32] = key.finalize().into();
    if ctx.senders.hit(&key) > ctx.senders.max() {
        tracing::info!(target: "calrs::inbox", booking_id, "inbox sender over its hourly limit for this booking; left unread");
        return leave("sender_limit");
    }
    if ctx.matched.hit(&()) > ctx.matched.max() {
        tracing::warn!(target: "calrs::inbox", "inbox over its hourly limit of stored mail; left unread");
        return leave("flood");
    }
    let hosts =
        crate::telegram::enabled() && !tg_store::host_chats(pool, booking_id).await.is_empty();
    let email = messages::FromEmail {
        hash: &hash,
        sender: hit.sender,
        address: from,
    };
    let Some(id) = messages::store_guest_message(pool, booking_id, &body, hosts, Some(email)).await
    else {
        tracing::info!(target: "calrs::inbox", booking_id, "guest email not stored: daily limit; left unread");
        return leave("limit");
    };
    tracing::info!(
        target: "calrs::inbox",
        booking_id,
        chars = body.chars().count(),
        proof = hit.proof.as_str(),
        sender = hit.sender.as_str(),
        shortened,
        hosts,
        "guest email stored"
    );
    if hosts {
        let from = crate::telegram::MailFrom {
            sender: hit.sender,
            address: if hit.sender == matcher::Sender::Guest {
                String::new()
            } else {
                from.to_string()
            },
        };
        crate::telegram::relay::relay_guest_email(pool, booking_id, &id, &body, from).await;
    }
    handled("stored")
}

/// `\Seen`, then the done folder: MOVE, else COPY + `\Deleted` + UID
/// EXPUNGE (UIDPLUS), else it stays seen where it is. A refused move is
/// logged and the message stays seen.
async fn mark<S: AsyncRead + AsyncWrite + Unpin>(
    client: &mut Client<S>,
    config: &Config,
    uid: u32,
) -> imap::Result<()> {
    client.uid_add_flag(uid, "\\Seen").await?;
    let Some(folder) = &config.done_folder else {
        return Ok(());
    };
    let moved = if client.has("MOVE") {
        client.uid_move(uid, folder).await
    } else if client.has("UIDPLUS") {
        match client.uid_copy(uid, folder).await {
            Ok(()) => {
                client.uid_add_flag(uid, "\\Deleted").await?;
                client.uid_expunge(uid).await
            }
            Err(e) => Err(e),
        }
    } else {
        Ok(())
    };
    match moved {
        Err(imap::Error::No | imap::Error::Bad) => {
            tracing::warn!(target: "calrs::inbox", uid, "inbox could not move a message to the done folder; left seen");
            Ok(())
        }
        other => other,
    }
}

/// Shared mode: whether the headers of a new message match a booking.
async fn matches_on_headers(ctx: &Ctx, headers: &[u8]) -> bool {
    let Some(m) = mime::parse(headers) else {
        return false;
    };
    let domain = ctx.ours.domain();
    if !matcher::may_match(&m, domain) {
        return false;
    }
    let candidates = matcher::candidates(&ctx.pool).await;
    matcher::find(&m, &candidates, &ctx.key, domain, true).is_some()
}

/// One message: its size, then its body, then what to do. In shared mode
/// its size and match headers first, and the rest only when they match.
async fn process<S: AsyncRead + AsyncWrite + Unpin>(
    client: &mut Client<S>,
    ctx: &Ctx,
    uid: u32,
    stats: &mut Stats,
) -> imap::Result<()> {
    let set = uid.to_string();
    let size = if ctx.config.shared {
        let items = format!(
            "(UID RFC822.SIZE BODY.PEEK[HEADER.FIELDS ({MATCH_HEADERS})]<0.{HEADERS_MAX}>)"
        );
        let meta = client.uid_fetch(&set, &items).await?;
        let Some(meta) = meta.into_iter().find(|f| f.uid == uid) else {
            stats.add("gone");
            return Ok(());
        };
        // Not ours: counted, and nothing else.
        if !matches_on_headers(ctx, meta.body.as_deref().unwrap_or_default()).await {
            stats.add("unmatched");
            return Ok(());
        }
        meta.size
    } else {
        let meta = client.uid_fetch(&set, "(UID RFC822.SIZE FLAGS)").await?;
        let Some(meta) = meta.into_iter().find(|f| f.uid == uid) else {
            stats.add("gone");
            return Ok(());
        };
        meta.size
    };
    let size = size.unwrap_or(u32::MAX);
    if size > MESSAGE_MAX {
        tracing::info!(target: "calrs::inbox", uid, size, "inbox message over the size limit; left unread");
        stats.add("oversize");
        return Ok(());
    }
    let items = format!("(UID BODY.PEEK[]<0.{MESSAGE_MAX}>)");
    let fetched = client.uid_fetch(&set, &items).await?;
    let Some(raw) = fetched
        .into_iter()
        .find(|f| f.uid == uid)
        .and_then(|f| f.body)
    else {
        stats.add("gone");
        return Ok(());
    };
    let d = handle(ctx, &raw).await;
    stats.add(d.reason);
    if d.handled {
        mark(client, &ctx.config, uid).await?;
    }
    Ok(())
}

/// The UIDs above `last`, at most [`SEARCH_SPAN`] of them a search: an
/// empty span below the folder's last UID is skipped (and `skipped` set to
/// where it ends), up to [`EMPTY_SPANS_PER_POLL`] of them.
async fn next_uids<S: AsyncRead + AsyncWrite + Unpin>(
    client: &mut Client<S>,
    last: u32,
    skipped: &mut Option<u32>,
) -> imap::Result<Vec<u32>> {
    let mut from = last.saturating_add(1);
    let mut top = None;
    for _ in 0..EMPTY_SPANS_PER_POLL {
        let to = from.saturating_add(SEARCH_SPAN - 1);
        let found: Vec<u32> = client
            .uid_search(&format!("UID {from}:{to}"))
            .await?
            .into_iter()
            .filter(|u| (from..=to).contains(u))
            .collect();
        if !found.is_empty() {
            return Ok(found);
        }
        // Nothing in this span: is there anything above it?
        let top = match top {
            Some(t) => t,
            None => {
                let t = client.last_uid().await?.unwrap_or(0);
                top = Some(t);
                t
            }
        };
        if top <= to || to == u32::MAX {
            return Ok(Vec::new());
        }
        *skipped = Some(to);
        from = to + 1;
    }
    Ok(Vec::new())
}

/// One poll of the selected folder.
pub(crate) async fn poll_once<S: AsyncRead + AsyncWrite + Unpin>(
    client: &mut Client<S>,
    ctx: &Ctx,
    policy: &Policy,
    selected: Selected,
) -> imap::Result<Stats> {
    client.noop().await?;
    let folder = &ctx.config.folder;
    let pool = &ctx.pool;
    let mut stats = Stats::default();
    let uidvalidity = selected.uidvalidity;
    let save = |last_uid: u32| async move {
        store::save_state(
            pool,
            folder,
            store::State {
                uidvalidity,
                last_uid,
            },
        )
        .await;
    };
    match store::state(pool, folder).await {
        Some(s) if s.uidvalidity == uidvalidity => {
            let mut skipped = None;
            let todo = next_uids(client, s.last_uid, &mut skipped).await?;
            if let Some(to) = skipped {
                save(to).await;
            }
            if todo.len() > policy.batch {
                stats
                    .0
                    .insert("deferred", (todo.len() - policy.batch) as u32);
            }
            for uid in todo.into_iter().take(policy.batch) {
                process(client, ctx, uid, &mut stats).await?;
                save(uid).await;
            }
        }
        state => {
            let last = match selected.uidnext {
                Some(next) => next.saturating_sub(1),
                None => client.last_uid().await?.unwrap_or(0),
            };
            if state.is_some() {
                tracing::warn!(target: "calrs::inbox", "inbox UIDVALIDITY changed; starting from the next message");
            } else if ctx.config.shared {
                // A person's own mailbox: its history is theirs.
                tracing::info!(target: "calrs::inbox", "inbox shared mode: first run starts from the next message");
            } else if last > 0 {
                // The unseen mail of the last span; the state is saved
                // only once all of it is done, so a session that breaks
                // half-way takes the rest (already handled mail is seen by
                // then, stored mail deduplicated).
                let from = last.saturating_sub(SEARCH_SPAN - 1).max(1);
                let unseen = client
                    .uid_search(&format!("UNSEEN UID {from}:{last}"))
                    .await?;
                for uid in unseen.into_iter().filter(|u| (from..=last).contains(u)) {
                    process(client, ctx, uid, &mut stats).await?;
                }
            }
            save(last).await;
        }
    }
    if !stats.0.is_empty() {
        tracing::info!(target: "calrs::inbox", outcomes = ?stats.0, "inbox poll");
    }
    Ok(stats)
}

/// How a session ended.
#[derive(Debug)]
pub(crate) enum End {
    Stopped,
    /// LOGIN (or AUTHENTICATE) refused with NO or BAD.
    Auth,
    /// SELECT of the configured folder refused with NO or BAD.
    Select,
    /// The transport: connect or TLS.
    Transport(ConnectError),
    /// The service: an I/O error, a timeout, NO/BAD or a protocol error.
    Session(imap::Error),
}

async fn stopped(stop: &mut watch::Receiver<bool>) {
    if stop.wait_for(|v| *v).await.is_err() {
        // The sender is gone: never stop from here.
        std::future::pending::<()>().await;
    }
}

/// Wait `d`; `true` when stopped meanwhile.
async fn wait(stop: &mut watch::Receiver<bool>, d: Duration) -> bool {
    tokio::select! {
        _ = tokio::time::sleep(d) => *stop.borrow(),
        _ = stopped(stop) => true,
    }
}

/// One connection, polled until it fails or the server stops.
async fn session(
    ctx: &Ctx,
    connect: &Connector,
    policy: &Policy,
    stop: &mut watch::Receiver<bool>,
    polled: &mut bool,
) -> End {
    let stream = tokio::select! {
        r = connect() => r,
        _ = stopped(stop) => return End::Stopped,
    };
    let stream = match stream {
        Ok(s) => s,
        Err(e) => return End::Transport(e),
    };
    let mut client = Client::new(stream);
    // Only a refused LOGIN is a credential problem; a NO anywhere else is
    // the service's.
    let preauth = match client.greeting().await {
        Ok(p) => p,
        Err(e) => return End::Session(e),
    };
    if let Err(e) = client.capability().await {
        return End::Session(e);
    }
    if !preauth {
        match client.login(&ctx.config.user, &ctx.config.password).await {
            Ok(()) => {}
            Err(imap::Error::No | imap::Error::Bad) => return End::Auth,
            Err(e) => return End::Session(e),
        }
        if let Err(e) = client.capability().await {
            return End::Session(e);
        }
    }
    let selected = match client.select(&ctx.config.folder).await {
        Ok(s) => s,
        Err(imap::Error::No | imap::Error::Bad) => {
            let _ = client.logout().await;
            return End::Select;
        }
        Err(e) => return End::Session(e),
    };
    let interval = policy.poll.unwrap_or(ctx.config.poll);
    loop {
        match poll_once(&mut client, ctx, policy, selected).await {
            Ok(_) => *polled = true,
            Err(e) => return End::Session(e),
        }
        if wait(stop, interval).await {
            let _ = client.logout().await;
            return End::Stopped;
        }
    }
}

/// The poller: sessions with back-off, until `stop`.
pub(crate) async fn run(
    ctx: Arc<Ctx>,
    connect: Connector,
    policy: Policy,
    mut stop: watch::Receiver<bool>,
) {
    let mut backoff = policy.backoff_start;
    loop {
        if *stop.borrow() {
            return;
        }
        let mut polled = false;
        let end = session(&ctx, &connect, &policy, &mut stop, &mut polled).await;
        if polled {
            backoff = policy.backoff_start;
        }
        let pause = match end {
            End::Stopped => return,
            End::Auth => {
                tracing::warn!(target: "calrs::inbox", "inbox login refused; next attempt in an hour");
                policy.auth_backoff
            }
            End::Select => {
                tracing::warn!(target: "calrs::inbox", retry_secs = backoff.as_secs(), "inbox folder could not be selected (service); check CALRS_INBOX_FOLDER");
                let p = backoff;
                backoff = (backoff * 2).min(policy.backoff_max);
                p
            }
            End::Transport(e) => {
                tracing::warn!(target: "calrs::inbox", error = %e, retry_secs = backoff.as_secs(), "inbox server unreachable (transport)");
                let p = backoff;
                backoff = (backoff * 2).min(policy.backoff_max);
                p
            }
            End::Session(e) => {
                tracing::warn!(target: "calrs::inbox", error = %e, retry_secs = backoff.as_secs(), "inbox session failed (service)");
                let p = backoff;
                backoff = (backoff * 2).min(policy.backoff_max);
                p
            }
        };
        if wait(&mut stop, pause).await {
            return;
        }
    }
}
