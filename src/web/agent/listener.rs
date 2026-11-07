//! The admin socket: where it comes from (systemd or a dev path), the accept
//! loop, and the life of one connection, from the uid check before the first
//! byte to the audit line when it closes.

use super::audit;
use super::server::{AgentServer, Session};
use super::transport::{GuardedReader, GuardedWriter, SessionTransport};
use super::{AgentCtx, MAX_FRAME_BYTES, MAX_UNANSWERED};
use rmcp::transport::async_rw::AsyncRwTransport;
use rmcp::ServiceExt;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{watch, Notify};

/// Where the admin socket comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ListenerSource {
    /// fd 3, passed by systemd socket activation (`calrs-admin.socket`).
    Systemd,
    /// A path serve binds itself (`CALRS_AGENT_SOCKET`, development and tests).
    Path(PathBuf),
}

impl fmt::Display for ListenerSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ListenerSource::Systemd => f.write_str("systemd (LISTEN_FDS)"),
            ListenerSource::Path(path) => write!(f, "{}", path.display()),
        }
    }
}

/// Decide where the socket comes from. systemd's fd counts only when
/// `LISTEN_PID` names this process (`pid`); then `LISTEN_FDS` must be exactly
/// 1, since calrs takes no other socket. Otherwise `CALRS_AGENT_SOCKET`, else
/// no socket at all. When both are present, systemd wins.
pub(crate) fn listener_source(
    env: impl Fn(&str) -> Option<String>,
    pid: u32,
) -> Result<Option<ListenerSource>, String> {
    let path = env("CALRS_AGENT_SOCKET")
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty());
    let for_us = env("LISTEN_PID").and_then(|p| p.trim().parse::<u32>().ok()) == Some(pid);
    if for_us {
        let fds = env("LISTEN_FDS").unwrap_or_default();
        if fds.trim() != "1" {
            return Err(format!(
                "LISTEN_FDS is {:?}, but calrs takes exactly one socket (the admin socket)",
                fds.trim()
            ));
        }
        if path.is_some() {
            tracing::warn!(
                target: "calrs::agent",
                "both LISTEN_FDS and CALRS_AGENT_SOCKET are set; using the systemd socket"
            );
        }
        return Ok(Some(ListenerSource::Systemd));
    }
    Ok(path.map(|p| ListenerSource::Path(PathBuf::from(p))))
}

/// Open the listener for `source`.
pub(crate) fn open(source: &ListenerSource) -> io::Result<UnixListener> {
    match source {
        ListenerSource::Systemd => take_systemd_listener(),
        ListenerSource::Path(path) => bind_path(path),
    }
}

/// Set once fd 3 has been taken, so it can never get two owners.
static SYSTEMD_FD_TAKEN: AtomicBool = AtomicBool::new(false);

/// First fd systemd passes (`SD_LISTEN_FDS_START`).
const SD_LISTEN_FDS_START: std::os::unix::io::RawFd = 3;

/// Take the socket systemd passed as fd 3. The one `unsafe` in calrs.
#[allow(unsafe_code)]
fn take_systemd_listener() -> io::Result<UnixListener> {
    use std::os::unix::io::FromRawFd;
    if SYSTEMD_FD_TAKEN.swap(true, Ordering::SeqCst) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "the systemd socket was already taken",
        ));
    }
    // SAFETY: `listener_source` returned `Systemd` only because LISTEN_PID names
    // this process and LISTEN_FDS is 1, so systemd opened fd 3 for us before exec
    // and nothing else in calrs uses it (no other code takes inherited fds); the
    // take-once flag above makes this the only owner. If fd 3 is not an AF_UNIX
    // socket, `local_addr` fails and the descriptor is closed with the error.
    let inherited = unsafe { std::os::unix::net::UnixListener::from_raw_fd(SD_LISTEN_FDS_START) };
    inherited.local_addr()?;
    // A duplicate made with F_DUPFD_CLOEXEC, so no child process inherits the
    // socket; the original fd 3, which systemd passed without CLOEXEC, is closed.
    let listener = inherited.try_clone()?;
    drop(inherited);
    listener.set_nonblocking(true)?;
    UnixListener::from_std(listener)
}

/// Bind `path` (development and tests). A stale socket left by an earlier run
/// is replaced; any other file there is an error. The socket ends up 0600:
/// in production systemd owns the socket and its permissions instead.
fn bind_path(path: &Path) -> io::Result<UnixListener> {
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_socket() => std::fs::remove_file(path)?,
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "the path exists and is not a socket",
            ))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    let listener = UnixListener::bind(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

/// Whether a shutdown receiver says "stop": the flag is set or the sender is
/// gone.
pub(crate) fn is_shutdown(shutdown: &watch::Receiver<bool>) -> bool {
    *shutdown.borrow() || shutdown.has_changed().is_err()
}

/// Resolve once the shutdown receiver says "stop".
async fn shutdown_signalled(shutdown: &mut watch::Receiver<bool>) {
    while !is_shutdown(shutdown) {
        if shutdown.changed().await.is_err() {
            return;
        }
    }
}

const ACCEPT_BACKOFF_MIN: Duration = Duration::from_millis(100);
const ACCEPT_BACKOFF_MAX: Duration = Duration::from_secs(5);

/// Once a session is told to stop, rmcp gives waiting answers 2 seconds (5
/// after the peer closed) and then closes its transport, which waits for the
/// write in progress. A write still waiting this long after the stop is cut,
/// so the close can finish and the slot is freed.
const CLOSE_GRACE: Duration = Duration::from_secs(5);

/// Accept connections until shutdown, one task each.
pub(crate) async fn accept_loop(listener: UnixListener, ctx: Arc<AgentCtx>) {
    check_acting_admin(&ctx).await;
    let mut shutdown = ctx.shutdown.clone();
    let mut backoff = ACCEPT_BACKOFF_MIN;
    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    backoff = ACCEPT_BACKOFF_MIN;
                    let ctx = ctx.clone();
                    tokio::spawn(async move {
                        handle_connection(stream, ctx).await;
                    });
                }
                Err(error) => {
                    tracing::warn!(
                        target: "calrs::agent",
                        %error,
                        "accept on the admin socket failed"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(ACCEPT_BACKOFF_MAX);
                }
            },
            () = shutdown_signalled(&mut shutdown) => break,
        }
    }
    tracing::info!(target: "calrs::agent", "MCP agent stopped accepting connections");
}

/// With an invalid configuration: accept and close every connection, so the
/// pump on the other side fails at once instead of waiting in the backlog.
pub(crate) async fn refuse_loop(listener: UnixListener, mut shutdown: watch::Receiver<bool>) {
    let mut backoff = ACCEPT_BACKOFF_MIN;
    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    backoff = ACCEPT_BACKOFF_MIN;
                    let cred = stream.peer_cred().ok();
                    let (uid, pid) = (cred.map(|c| c.uid()), cred.and_then(|c| c.pid()));
                    audit::refused(uid, pid, "config_error");
                }
                Err(_) => {
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(ACCEPT_BACKOFF_MAX);
                }
            },
            () = shutdown_signalled(&mut shutdown) => break,
        }
    }
}

/// Warn at startup when the acting admin account cannot be used; calls fail
/// until it can, and each call checks again.
async fn check_acting_admin(ctx: &AgentCtx) {
    let usable: Result<Option<i64>, sqlx::Error> = sqlx::query_scalar(
        "SELECT 1 FROM users WHERE email = ? AND enabled = 1 AND role = 'admin'",
    )
    .bind(&ctx.config.admin_email)
    .fetch_optional(&ctx.state.pool)
    .await;
    match usable {
        Ok(Some(_)) => {}
        Ok(None) => tracing::warn!(
            target: "calrs::agent",
            "CALRS_AGENT_ADMIN_EMAIL names no enabled admin account; tool calls fail until it does"
        ),
        Err(error) => tracing::warn!(
            target: "calrs::agent",
            %error,
            "could not check the acting admin account"
        ),
    }
}

/// How a connection ended, for the caller and for tests.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConnOutcome {
    /// Closed before reading a byte; the reason is also in the audit log.
    Refused(&'static str),
    /// `MAX_CONNECTIONS` sessions were already open.
    Busy,
    /// A session ran and closed for `reason`.
    Served { reason: &'static str },
}

/// Serve one connection. Nothing is read from the socket until the peer's uid
/// has been mapped to a role; an unknown uid is closed unread.
pub(crate) async fn handle_connection(stream: UnixStream, ctx: Arc<AgentCtx>) -> ConnOutcome {
    // Who is it? This asks the kernel (SO_PEERCRED) and reads nothing; done
    // first so every refusal below can name the peer.
    let cred = match stream.peer_cred() {
        Ok(cred) => cred,
        Err(_) => {
            audit::refused(None, None, "no_peer_credentials");
            return ConnOutcome::Refused("no_peer_credentials");
        }
    };
    let (uid, pid) = (cred.uid(), cred.pid());
    // Not during shutdown.
    if ctx.shutting_down() {
        audit::refused(Some(uid), pid, "shutting_down");
        return ConnOutcome::Refused("shutting_down");
    }
    // The role comes from the uid alone: never root, never a group. Checked
    // before the slot, so an unknown uid is always audited as such and never
    // holds a slot.
    let Some((user, role)) = ctx.config.roles.get(uid) else {
        audit::refused(Some(uid), pid, "unknown_uid");
        return ConnOutcome::Refused("unknown_uid");
    };
    // A free slot, or refuse; no queueing.
    let Ok(_permit) = ctx.conns.clone().try_acquire_owned() else {
        audit::refused(Some(uid), pid, "busy");
        return ConnOutcome::Busy;
    };
    let session = Arc::new(Session::new(uid, pid, user, role));
    audit::accepted(&session);

    // Only now does anything read from the socket.
    let (read_half, write_half) = stream.into_split();
    let reader = GuardedReader::new(read_half, MAX_FRAME_BYTES, session.clone());
    let writer = GuardedWriter::new(write_half, session.clone());
    // The guarded reader cuts the session first; the transport's own cap (it
    // counts the newline) is a backstop.
    let transport =
        AsyncRwTransport::new_server(reader, writer).with_max_line_length(MAX_FRAME_BYTES + 1);
    let transport = SessionTransport::new(transport, session.clone(), MAX_UNANSWERED);
    let server = AgentServer::new(ctx.clone(), session.clone());

    // The handshake, bounded in time and by shutdown.
    let mut shutdown = ctx.shutdown.clone();
    let handshake = server.serve(transport);
    let handshake = tokio::select! {
        result = tokio::time::timeout(ctx.config.handshake_timeout, handshake) => result,
        () = shutdown_signalled(&mut shutdown) => return closed(&session, "shutdown"),
    };
    let running = match handshake {
        Ok(Ok(running)) => running,
        Ok(Err(_)) if session.frame_too_large() => return closed(&session, "frame_too_large"),
        Ok(Err(_)) => return closed(&session, "handshake_failed"),
        Err(_) => return closed(&session, "handshake_timeout"),
    };

    // Supervise: client gone, shutdown, idle, too old, or not reading.
    let mut cancel = Some(running.cancellation_token());
    let mut reason: Option<&'static str> = None;
    let mut stopped_at: Option<tokio::time::Instant> = None;
    let waiting = running.waiting();
    tokio::pin!(waiting);
    let idle_timeout = ctx.config.idle_timeout;
    let idle = tokio::time::sleep(idle_timeout);
    tokio::pin!(idle);
    let max_session = tokio::time::sleep(ctx.config.max_session);
    tokio::pin!(max_session);
    let stall_timeout = ctx.config.write_stall_timeout;
    let mut stall_check = tokio::time::interval(
        (stall_timeout / 4).clamp(Duration::from_millis(10), Duration::from_secs(1)),
    );
    stall_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut stop = |why: &'static str,
                    reason: &mut Option<&'static str>,
                    stopped_at: &mut Option<tokio::time::Instant>| {
        if let Some(token) = cancel.take() {
            token.cancel();
            *stopped_at = Some(tokio::time::Instant::now());
        }
        reason.get_or_insert(why);
    };
    loop {
        tokio::select! {
            _ = &mut waiting => break,
            () = shutdown_signalled(&mut shutdown), if reason.is_none() => {
                stop("shutdown", &mut reason, &mut stopped_at);
            }
            () = &mut idle, if reason.is_none() => {
                let idle_for = session.idle_for();
                if idle_for >= idle_timeout && session.in_flight.count() == 0 {
                    stop("idle", &mut reason, &mut stopped_at);
                } else {
                    // Busy or recently active: look again when the timeout
                    // could next be reached, at most once a second.
                    let wait = idle_timeout.saturating_sub(idle_for);
                    let wait = wait.max(Duration::from_secs(1));
                    idle.as_mut().reset(tokio::time::Instant::now() + wait);
                }
            }
            () = &mut max_session, if reason.is_none() => {
                stop("max_session", &mut reason, &mut stopped_at);
            }
            _ = stall_check.tick() => {
                // A peer that leaves an answer unread holds a slot and, with
                // MAX_UNANSWERED reached, gets nothing more read: close it.
                // Also after any stop, a write still waiting is cut so rmcp
                // can close its transport.
                if session.writes.stalled_for() >= stall_timeout {
                    stop("write_stalled", &mut reason, &mut stopped_at);
                    session.writes.cut();
                } else if stopped_at.is_some_and(|at| at.elapsed() >= CLOSE_GRACE) {
                    session.writes.cut();
                }
            }
        }
    }
    let reason = reason.unwrap_or(if session.frame_too_large() {
        "frame_too_large"
    } else {
        "client_closed"
    });
    // The permit is released when `_permit` drops after this.
    closed(&session, reason)
}

fn closed(session: &Session, reason: &'static str) -> ConnOutcome {
    audit::closed(session, reason);
    ConnOutcome::Served { reason }
}

/// Counts running tool calls and wakes waiters when none are left. rmcp runs
/// each call in a detached task that cancelling a session does not stop, so
/// this is how shutdown and the idle timeout know a call is still running.
#[derive(Default)]
pub(crate) struct InFlight {
    count: AtomicUsize,
    drained: Notify,
}

impl InFlight {
    pub(crate) fn count(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }

    /// Count one call until the guard drops (also on panic).
    pub(crate) fn enter(self: &Arc<Self>) -> InFlightGuard {
        self.count.fetch_add(1, Ordering::AcqRel);
        InFlightGuard(self.clone())
    }

    /// Wait until no call is running, or `timeout` passes. Returns how many
    /// were still running (the algorithm of `TaskTracker::drain` in notify.rs).
    pub(crate) async fn drain(&self, timeout: Duration) -> usize {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let notified = self.drained.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.count() == 0 {
                return 0;
            }
            if tokio::time::timeout_at(deadline, notified).await.is_err() {
                return self.count();
            }
        }
    }
}

pub(crate) struct InFlightGuard(Arc<InFlight>);

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        if self.0.count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.0.drained.notify_waiters();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn source(vars: &[(&str, &str)], pid: u32) -> Result<Option<ListenerSource>, String> {
        let vars: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        listener_source(|k| vars.get(k).cloned(), pid)
    }

    #[test]
    fn no_socket_configured_means_off() {
        assert_eq!(source(&[], 42), Ok(None));
        assert_eq!(source(&[("CALRS_AGENT_SOCKET", "  ")], 42), Ok(None));
    }

    #[test]
    fn systemd_counts_only_for_this_process() {
        let fds = [("LISTEN_PID", "41"), ("LISTEN_FDS", "1")];
        assert_eq!(source(&fds, 42), Ok(None));
        assert_eq!(source(&fds, 41), Ok(Some(ListenerSource::Systemd)));
        assert_eq!(
            source(&[("LISTEN_FDS", "1")], 41),
            Ok(None),
            "no LISTEN_PID"
        );
    }

    #[test]
    fn systemd_must_pass_exactly_one_socket() {
        for fds in ["2", "0", "", "x"] {
            let result = source(&[("LISTEN_PID", "7"), ("LISTEN_FDS", fds)], 7);
            assert!(result.is_err(), "LISTEN_FDS={fds:?}");
        }
        assert!(source(&[("LISTEN_PID", "7")], 7).is_err());
    }

    #[test]
    fn path_is_the_fallback_and_systemd_wins() {
        let path = ("CALRS_AGENT_SOCKET", "/tmp/calrs-agent.sock");
        assert_eq!(
            source(&[path], 7),
            Ok(Some(ListenerSource::Path(PathBuf::from(
                "/tmp/calrs-agent.sock"
            ))))
        );
        assert_eq!(
            source(&[path, ("LISTEN_PID", "8"), ("LISTEN_FDS", "1")], 7),
            Ok(Some(ListenerSource::Path(PathBuf::from(
                "/tmp/calrs-agent.sock"
            )))),
            "systemd fds for another process are ignored"
        );
        assert_eq!(
            source(&[path, ("LISTEN_PID", "7"), ("LISTEN_FDS", "1")], 7),
            Ok(Some(ListenerSource::Systemd))
        );
    }

    #[tokio::test]
    async fn bind_path_replaces_a_stale_socket_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("calrs_agent_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("s");
        let first = bind_path(&path).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        drop(first);
        // The socket file stays behind; a second bind replaces it.
        let _second = bind_path(&path).unwrap();

        let file = dir.join("f");
        std::fs::write(&file, b"not a socket").unwrap();
        assert!(bind_path(&file).is_err());
        assert_eq!(std::fs::read(&file).unwrap(), b"not a socket");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn in_flight_drains_or_times_out() {
        let tracker = Arc::new(InFlight::default());
        assert_eq!(tracker.drain(Duration::from_millis(10)).await, 0);

        let guard = tracker.enter();
        let second = tracker.enter();
        assert_eq!(tracker.count(), 2);
        assert_eq!(tracker.drain(Duration::from_millis(20)).await, 2);

        drop(second);
        let waiter = {
            let tracker = tracker.clone();
            tokio::spawn(async move { tracker.drain(Duration::from_secs(5)).await })
        };
        tokio::time::sleep(Duration::from_millis(20)).await;
        drop(guard);
        assert_eq!(waiter.await.unwrap(), 0);
        assert_eq!(tracker.count(), 0);
    }
}
