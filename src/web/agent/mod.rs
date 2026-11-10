//! The calrs agent: an MCP server inside `calrs serve`, reachable only through
//! a local Unix socket (fork addition, see ilia-booking `docs/mcp.md`).
//!
//! Threat model in short. The socket is created by systemd (`calrs-admin.socket`,
//! group `calrs-mcp`, mode 0660) and passed in through `LISTEN_FDS`; the only way
//! to it is SSH with a hardware-backed key whose forced command is `calrs mcp`, a
//! byte pump (`src/commands/mcp.rs`). Before reading a byte, serve asks the kernel
//! who the peer is (`SO_PEERCRED`) and maps that uid to a role (read-only or write);
//! unknown uids are dropped. Every tool runs as one configured admin account, which
//! is re-checked on each call. Outputs are hand-written DTOs that never carry secrets,
//! tokens or links, and guest or calendar text only under `guest_supplied` /
//! `calendar_supplied` keys, cleaned and capped. Calls are rate limited, writes more
//! tightly, and guest emails (approvals, declines, cancellations), full booking
//! reads and source syncs each have their own cap; a session reads no further
//! while 16 of its requests are unanswered, sessions time out (also when the
//! peer stops reading), and every call is written to the journal
//! (`calrs::audit`), every change with its settings before and after.
//!
//! Configuration (environment of `calrs serve`):
//! - `CALRS_AGENT_USERS`: `calrs-mcp=write,calrs-mcp-ro=read`, system user names;
//! - `CALRS_AGENT_ADMIN_EMAIL`: the calrs admin account the agent acts as;
//! - `CALRS_AGENT_IDLE_TIMEOUT_SECS` (default 1800), `CALRS_AGENT_MAX_SESSION_SECS`
//!   (default 28800);
//! - `CALRS_AGENT_SOCKET`: a path serve binds itself, for development and tests,
//!   used only without `LISTEN_FDS`.
//!
//! Without a socket the agent stays off. With a socket but a broken configuration
//! it accepts and immediately closes every connection, so the pump fails fast
//! instead of hanging, and the public site keeps running.

#![deny(unsafe_code)]

use super::*;

pub(super) mod audit;
pub(super) mod config;
pub(super) mod dto;
pub(super) mod error;
pub(super) mod limit;
pub(super) mod listener;
pub(super) mod ops_bookings;
pub(super) mod ops_read;
pub(super) mod ops_sources;
pub(super) mod ops_ton;
pub(super) mod ops_write;
pub(super) mod sanitize;
pub(super) mod server;
pub(super) mod snapshot;
mod tools_bookings;
mod tools_read;
mod tools_sources;
mod tools_ton;
mod tools_write;
pub(super) mod transport;

use config::AgentConfig;
use limit::SlidingLimit;
use listener::InFlight;
use tokio::sync::{watch, Semaphore};
use tokio::task::JoinHandle;

/// Largest JSON-RPC message, in bytes without its newline, accepted in either
/// direction. The pump enforces it on stdin, serve on the socket.
pub(crate) const MAX_FRAME_BYTES: usize = 256 * 1024;
/// Sessions served at once; one more is refused, never queued.
pub(crate) const MAX_CONNECTIONS: usize = 2;
/// Time a new connection has to complete the MCP handshake.
pub(crate) const HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
/// A session whose peer has not read a waiting answer for this long is closed.
pub(crate) const WRITE_STALL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
/// Requests of one session that may be unanswered at once; past it, nothing
/// more is read from the socket until an answer has been written.
pub(crate) const MAX_UNANSWERED: usize = 16;
/// Messages one session may send: a burst, then a steady rate per second.
/// Past it the reading waits; the session stays open.
pub(crate) const MESSAGE_BURST: u32 = 100;
pub(crate) const MESSAGES_PER_SECOND: u32 = 20;
/// Tool calls per hour, all sessions together.
pub(crate) const CALLS_PER_HOUR: u32 = 1200;
/// Calls of write tools in any hour (a sliding window, `SlidingLimit`), all
/// sessions together, refused ones included; they count against
/// `CALLS_PER_HOUR` as well.
pub(crate) const WRITES_PER_HOUR: u32 = 60;
/// Approvals handed to the web handler in any hour, all sessions together;
/// they count against `WRITES_PER_HOUR` as well.
pub(crate) const APPROVALS_PER_HOUR: u32 = 20;
/// Declines and cancellations handed to the web handler in any hour,
/// together and for all sessions; they count against `WRITES_PER_HOUR` as
/// well.
pub(crate) const ENDINGS_PER_HOUR: u32 = 5;
/// `get_booking` calls in any hour, all sessions together: each shows one
/// guest's full details.
pub(crate) const BOOKING_READS_PER_HOUR: u32 = 60;
/// The window of the caps above.
const HOUR: std::time::Duration = std::time::Duration::from_secs(3600);
/// Seconds between two syncs of one calendar source through the agent.
pub(crate) const SYNC_COOLDOWN_SECS: u64 = 300;
/// Page size of list tools when the caller gives none, and the largest allowed.
pub(crate) const PAGE_DEFAULT: u32 = 20;
pub(crate) const PAGE_MAX: u32 = 50;
/// Highest cursor offset a list tool accepts.
pub(crate) const CURSOR_MAX: u32 = 10_000;
/// Caps on guest- and calendar-supplied text, in characters.
pub(crate) const GUEST_NAME_MAX: usize = 80;
pub(crate) const CAL_NAME_MAX: usize = 80;
/// Cap on owner-written text (titles, descriptions), in characters.
pub(crate) const OWNER_TEXT_MAX: usize = 4000;
/// Cap on a booking's notes as `get_booking` shows them, in characters.
pub(crate) const GUEST_NOTES_MAX: usize = 500;

/// Everything a connection needs, shared by all of them.
pub(crate) struct AgentCtx {
    pub(crate) state: Arc<AppState>,
    pub(crate) config: AgentConfig,
    /// One permit per session (`MAX_CONNECTIONS`).
    pub(crate) conns: Arc<Semaphore>,
    /// Tool calls running in any session, waited for at shutdown.
    pub(crate) in_flight: Arc<InFlight>,
    /// `CALLS_PER_HOUR`, keyed by "calls". Upstream's fixed window: a
    /// coarse cap, where twice the rate across a window's end does no harm.
    pub(crate) calls: RateLimiter,
    /// `WRITES_PER_HOUR`. The caps on changes, guest emails and full
    /// booking reads hold in any hour, not in a fixed one (`SlidingLimit`).
    pub(crate) writes: SlidingLimit,
    /// `APPROVALS_PER_HOUR`.
    pub(crate) approvals: SlidingLimit,
    /// `ENDINGS_PER_HOUR`.
    pub(crate) endings: SlidingLimit,
    /// `BOOKING_READS_PER_HOUR`.
    pub(crate) booking_reads: SlidingLimit,
    /// One sync per source and `SYNC_COOLDOWN_SECS`, keyed by source id.
    pub(crate) syncs: RateLimiter,
    /// Held by every write from its first read to its last, so two sessions
    /// never interleave changes. The web interface does not take it: a write
    /// that depends on what it read checks a version token instead, and a
    /// booking action relies on the web handler's own guard on the status. A
    /// source sync changes no setting and takes the source's sync lock instead.
    pub(crate) write_lock: tokio::sync::Mutex<()>,
    /// Flips to `true` when serve shuts down; a dropped sender means the same.
    pub(crate) shutdown: watch::Receiver<bool>,
}

impl AgentCtx {
    pub(crate) fn new(
        state: Arc<AppState>,
        config: AgentConfig,
        shutdown: watch::Receiver<bool>,
    ) -> Self {
        Self {
            state,
            config,
            conns: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
            in_flight: Arc::new(InFlight::default()),
            calls: RateLimiter::new(CALLS_PER_HOUR, 3600),
            writes: SlidingLimit::new(WRITES_PER_HOUR, HOUR),
            approvals: SlidingLimit::new(APPROVALS_PER_HOUR, HOUR),
            endings: SlidingLimit::new(ENDINGS_PER_HOUR, HOUR),
            booking_reads: SlidingLimit::new(BOOKING_READS_PER_HOUR, HOUR),
            syncs: RateLimiter::new(1, SYNC_COOLDOWN_SECS),
            write_lock: tokio::sync::Mutex::new(()),
            shutdown,
        }
    }

    /// Whether serve is shutting down: the flag is set or its sender is gone.
    pub(crate) fn shutting_down(&self) -> bool {
        listener::is_shutdown(&self.shutdown)
    }
}

/// Start the agent if an admin socket is configured. Called by `calrs serve`
/// once its state is built; `shutdown` flips when serve receives SIGTERM or
/// SIGINT. Never fails: problems are logged and leave the agent off (no socket)
/// or refusing (socket but invalid configuration).
pub(crate) fn spawn(state: Arc<AppState>, shutdown: watch::Receiver<bool>) -> AgentHandle {
    let env = |name: &str| std::env::var(name).ok();
    let source = match listener::listener_source(env, std::process::id()) {
        Ok(Some(source)) => source,
        Ok(None) => {
            tracing::info!(
                target: "calrs::agent",
                "no admin socket configured (LISTEN_FDS or CALRS_AGENT_SOCKET), MCP agent off"
            );
            return AgentHandle::off();
        }
        Err(reason) => {
            tracing::error!(
                target: "calrs::agent",
                %reason,
                "admin socket unusable, MCP agent off"
            );
            return AgentHandle::off();
        }
    };
    let listener = match listener::open(&source) {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(
                target: "calrs::agent",
                socket = %source,
                %error,
                "cannot open the admin socket, MCP agent off"
            );
            return AgentHandle::off();
        }
    };
    let passwd = std::fs::read_to_string("/etc/passwd").unwrap_or_default();
    let config = match AgentConfig::from_lookup(env, &passwd) {
        Ok(config) => config,
        Err(error) => {
            tracing::error!(
                target: "calrs::agent",
                socket = %source,
                %error,
                "MCP agent configuration invalid: every connection on the admin socket is refused"
            );
            let task = tokio::spawn(listener::refuse_loop(listener, shutdown));
            return AgentHandle {
                task: Some(task),
                in_flight: Arc::new(InFlight::default()),
                conns: None,
            };
        }
    };
    if tracing::enabled!(target: "rmcp", tracing::Level::DEBUG) {
        tracing::warn!(
            target: "calrs::agent",
            "debug logging is on for target rmcp, which logs tool results, guest data included"
        );
    }
    tracing::info!(
        target: "calrs::agent",
        socket = %source,
        users = %config.roles,
        "MCP agent listening on the admin socket"
    );
    let ctx = Arc::new(AgentCtx::new(state, config, shutdown));
    let handle = AgentHandle {
        task: None,
        in_flight: ctx.in_flight.clone(),
        conns: Some(ctx.conns.clone()),
    };
    AgentHandle {
        task: Some(tokio::spawn(listener::accept_loop(listener, ctx))),
        ..handle
    }
}

/// Handle on a running agent, for an orderly shutdown.
pub(crate) struct AgentHandle {
    task: Option<JoinHandle<()>>,
    in_flight: Arc<InFlight>,
    conns: Option<Arc<Semaphore>>,
}

impl AgentHandle {
    fn off() -> Self {
        Self {
            task: None,
            in_flight: Arc::new(InFlight::default()),
            conns: None,
        }
    }

    /// Wait, at most `grace`, for the agent to stop: the accept loop first, then
    /// the tool calls still running (an approval may still be handing emails to
    /// `spawn_emails`, so this runs before `drain_emails`), then the sessions.
    /// Call it after the shutdown flag has been set.
    pub(crate) async fn finish(self, grace: std::time::Duration) {
        let Some(task) = self.task else {
            return;
        };
        let deadline = tokio::time::Instant::now() + grace;
        let _ = tokio::time::timeout_at(deadline, task).await;
        let left = self
            .in_flight
            .drain(deadline.saturating_duration_since(tokio::time::Instant::now()))
            .await;
        if left > 0 {
            tracing::warn!(
                target: "calrs::agent",
                calls = left,
                "MCP tool calls still running at shutdown"
            );
        }
        if let Some(conns) = self.conns {
            let all = u32::try_from(MAX_CONNECTIONS).unwrap_or(u32::MAX);
            if tokio::time::timeout_at(deadline, conns.acquire_many(all))
                .await
                .is_err()
            {
                tracing::warn!(
                    target: "calrs::agent",
                    sessions = MAX_CONNECTIONS - conns.available_permits(),
                    "MCP sessions still closing at shutdown"
                );
            }
        }
    }
}
