//! Helpers for tests that touch process-global state.
//!
//! Environment variables belong to the process, and Rust runs tests in
//! parallel threads inside one process. So a test that sets `CALRS_*` races
//! every test that reads it, including tests in other modules. A lock private
//! to one module is not enough: `commands::config` had exactly that, and
//! `config_general_set_and_clear` still failed intermittently because
//! `caldav`'s allowlist test was setting the same variable from another
//! module. Everything that touches this environment now shares one lock.

/// Serialises tests that read or write the runtime-settings environment.
/// Async-aware, so a guard can be held across an `.await`.
pub(crate) static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// The runtime-settings variables. Anything added here is also cleared and
/// restored by [`EnvGuard`].
const VARS: [&str; 2] = ["CALRS_BASE_URL", "CALRS_ALLOW_PRIVATE_HOSTS"];

/// Clears the runtime-settings env vars for the duration of a test and puts
/// them back afterwards, so one test cannot leak an override into the next.
/// Hold [`ENV_LOCK`] for as long as you hold this.
pub(crate) struct EnvGuard {
    old: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    pub(crate) fn new() -> Self {
        let old = VARS
            .iter()
            .map(|n| (*n, std::env::var(n).ok()))
            .collect::<Vec<_>>();
        for n in VARS {
            std::env::remove_var(n);
        }
        Self { old }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (n, v) in &self.old {
            match v {
                Some(v) => std::env::set_var(n, v),
                None => std::env::remove_var(n),
            }
        }
    }
}

/// Installs `subscriber` as this thread's default for a test that reads the
/// log, until the guard drops.
///
/// tracing-core caches one `Interest` per callsite for the whole process.
/// While only one dispatcher is alive, it takes that interest from the
/// default subscriber of whichever thread hits the callsite first. A
/// parallel test with no subscriber can then cache `never`, and the capture
/// misses its own events (this made `posted_fields_cannot_forge_log_lines`
/// and two agent audit tests fail now and then). [`pin_dispatch`] keeps a
/// second dispatcher alive, so tracing-core combines the interests of all
/// live dispatchers and each thread's own subscriber decides per event. The
/// rebuild re-checks callsites already cached before this capture began.
pub(crate) fn capture_default<S>(subscriber: S) -> tracing::subscriber::DefaultGuard
where
    S: tracing::Subscriber + Send + Sync + 'static,
{
    pin_dispatch();
    let guard = tracing::subscriber::set_default(subscriber);
    tracing::callsite::rebuild_interest_cache();
    guard
}

/// Keeps a second dispatcher alive for the whole test run (see
/// [`capture_default`]). It is nobody's default and answers `never`, so it
/// receives no events and, combined with a capture's `always`, makes the
/// callsite `sometimes`: checked per event against the thread's subscriber.
/// Call it before `with_default` in a capture that does not use
/// [`capture_default`].
pub(crate) fn pin_dispatch() {
    static PIN: std::sync::OnceLock<tracing::Dispatch> = std::sync::OnceLock::new();
    PIN.get_or_init(|| tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default()));
}

/// A server that answers every request with a `307` to a second server on
/// the same loopback address, and that second server's hit counter. A client
/// that follows redirects re-sends its request (and body) there.
pub(crate) async fn redirect_trap() -> (String, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let hits = std::sync::Arc::new(AtomicUsize::new(0));
    let target = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_port = target.local_addr().unwrap().port();
    let seen = hits.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = target.accept().await {
            seen.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                let _ = sock.read(&mut buf).await;
                let _ = sock
                    .write_all(b"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: 46\r\nconnection: close\r\n\r\n{\"success\":true,\"url\":\"https://meet.test/x\"}  ")
                    .await;
            });
        }
    });
    let front = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let front_port = front.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = front.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                let _ = sock.read(&mut buf).await;
                let reply = format!(
                    "HTTP/1.1 307 Temporary Redirect\r\nlocation: http://127.0.0.1:{target_port}/elsewhere\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                );
                let _ = sock.write_all(reply.as_bytes()).await;
            });
        }
    });
    (format!("http://127.0.0.1:{front_port}/"), hits)
}
