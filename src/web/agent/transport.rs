//! What sits between the admin socket and rmcp, and keeps one session's
//! memory bounded whatever the peer does:
//!
//! - `GuardedReader` caps the length of a line and stamps the session's last
//!   activity for the idle timeout;
//! - `GuardedWriter` notes when a write to the peer has been waiting, so the
//!   listener can close a session whose peer stopped reading, and fails every
//!   write once the session is cut, so rmcp can finish closing;
//! - `SessionTransport` stops reading while `MAX_UNANSWERED` requests are
//!   unanswered and paces incoming messages. rmcp handles each request in a
//!   task of its own and queues every answer for writing, so without this a
//!   peer that writes requests and never reads its answers would grow serve's
//!   memory without end.

use super::server::Session;
use rmcp::model::{ClientNotification, JsonRpcMessage, RequestId};
use rmcp::service::{RxJsonRpcMessage, TxJsonRpcMessage};
use rmcp::transport::Transport;
use rmcp::RoleServer;
use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{ready, Context, Poll, Waker};
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::Notify;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The socket's read half, seen through two guards: every read stamps the
/// session's last activity (for the idle timeout), and more than `max_line`
/// bytes without a newline is an `InvalidData` error that ends the session.
pub(crate) struct GuardedReader<R> {
    inner: R,
    max_line: usize,
    since_newline: usize,
    session: Arc<Session>,
}

impl<R> GuardedReader<R> {
    pub(crate) fn new(inner: R, max_line: usize, session: Arc<Session>) -> Self {
        Self {
            inner,
            max_line,
            since_newline: 0,
            session,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for GuardedReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let before = buf.filled().len();
        ready!(Pin::new(&mut this.inner).poll_read(cx, buf))?;
        let fresh = &buf.filled()[before..];
        if fresh.is_empty() {
            return Poll::Ready(Ok(()));
        }
        this.session.touch();
        for &byte in fresh {
            if byte == b'\n' {
                this.since_newline = 0;
            } else {
                this.since_newline += 1;
                if this.since_newline > this.max_line {
                    this.session.mark_frame_too_large();
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "message exceeds the frame limit",
                    )));
                }
            }
        }
        Poll::Ready(Ok(()))
    }
}

/// The state of the writes to one peer, shared by the session's writer and
/// the listener that supervises it.
#[derive(Default)]
pub(crate) struct WriteWatch {
    state: Mutex<WriteWait>,
    cut: AtomicBool,
}

#[derive(Default)]
struct WriteWait {
    /// When the current write started waiting for the peer to read.
    since: Option<Instant>,
    /// The task of that write, woken when writes are cut.
    waker: Option<Waker>,
}

impl WriteWatch {
    /// How long the current write has been waiting; zero when none is.
    pub(crate) fn stalled_for(&self) -> Duration {
        lock(&self.state)
            .since
            .map(|since| since.elapsed())
            .unwrap_or_default()
    }

    /// Fail every write from now on, the one waiting included, so rmcp's
    /// tasks stop waiting for a peer that does not read and the session can
    /// close.
    pub(crate) fn cut(&self) {
        self.cut.store(true, Ordering::Release);
        let waker = lock(&self.state).waker.take();
        if let Some(waker) = waker {
            waker.wake();
        }
    }

    pub(crate) fn is_cut(&self) -> bool {
        self.cut.load(Ordering::Acquire)
    }

    /// Run one poll of the socket's write half and record its outcome.
    fn poll<T>(
        &self,
        cx: &mut Context<'_>,
        poll: impl FnOnce(&mut Context<'_>) -> Poll<io::Result<T>>,
    ) -> Poll<io::Result<T>> {
        if self.is_cut() {
            return Poll::Ready(Err(cut_error()));
        }
        let result = poll(cx);
        let mut state = lock(&self.state);
        if result.is_pending() {
            state.since.get_or_insert_with(Instant::now);
            if !state
                .waker
                .as_ref()
                .is_some_and(|waker| waker.will_wake(cx.waker()))
            {
                state.waker = Some(cx.waker().clone());
            }
        } else {
            state.since = None;
            state.waker = None;
        }
        drop(state);
        // Checked after the waker is stored: `cut` sets the flag before it
        // takes the waker, so either it wakes this task or this sees the flag.
        if result.is_pending() && self.is_cut() {
            return Poll::Ready(Err(cut_error()));
        }
        result
    }
}

fn cut_error() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "the session was closed")
}

/// The socket's write half, reporting to the session's `WriteWatch`.
pub(crate) struct GuardedWriter<W> {
    inner: W,
    session: Arc<Session>,
}

impl<W> GuardedWriter<W> {
    pub(crate) fn new(inner: W, session: Arc<Session>) -> Self {
        Self { inner, session }
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for GuardedWriter<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let inner = &mut this.inner;
        this.session
            .writes
            .poll(cx, |cx| Pin::new(inner).poll_write(cx, buf))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let inner = &mut this.inner;
        this.session
            .writes
            .poll(cx, |cx| Pin::new(inner).poll_flush(cx))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}

/// Requests read but not answered yet, by id: being handled, or answered
/// and waiting to be written.
#[derive(Default)]
struct Unanswered {
    ledger: Mutex<Ledger>,
    answered: Notify,
}

#[derive(Default)]
struct Ledger {
    ids: HashMap<RequestId, usize>,
    total: usize,
}

impl Unanswered {
    fn count(&self) -> usize {
        lock(&self.ledger).total
    }

    fn add(&self, id: RequestId) {
        let mut ledger = lock(&self.ledger);
        *ledger.ids.entry(id).or_insert(0) += 1;
        ledger.total += 1;
    }

    /// One request with `id` needs no more answer. An id not waiting for one
    /// is ignored, so a late cancellation or a stray answer cannot count twice.
    fn settle(&self, id: &RequestId) {
        let mut ledger = lock(&self.ledger);
        let Some(waiting) = ledger.ids.get_mut(id) else {
            return;
        };
        *waiting -= 1;
        if *waiting == 0 {
            ledger.ids.remove(id);
        }
        ledger.total -= 1;
        drop(ledger);
        self.answered.notify_one();
    }
}

/// Settles a request when the write of its answer ends, written, failed or
/// dropped.
struct Answer {
    unanswered: Arc<Unanswered>,
    id: RequestId,
}

impl Drop for Answer {
    fn drop(&mut self) {
        self.unanswered.settle(&self.id);
    }
}

/// A token bucket: `burst` messages at once, then `per_sec` a second.
#[derive(Debug)]
struct Pace {
    per_sec: f64,
    burst: f64,
    tokens: f64,
    last: Instant,
}

impl Pace {
    fn new(per_sec: u32, burst: u32, now: Instant) -> Self {
        Self {
            per_sec: f64::from(per_sec),
            burst: f64::from(burst),
            tokens: f64::from(burst),
            last: now,
        }
    }

    /// How long to wait at `now` before the next message may be read.
    fn wait(&mut self, now: Instant) -> Duration {
        let earned = now.saturating_duration_since(self.last).as_secs_f64() * self.per_sec;
        self.tokens = (self.tokens + earned).min(self.burst);
        self.last = now;
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64((1.0 - self.tokens) / self.per_sec)
        }
    }

    /// A message was read.
    fn take(&mut self) {
        self.tokens -= 1.0;
    }
}

/// rmcp's transport for one session, with two limits on what it reads.
/// While `limit` requests are unanswered, nothing more is read, so the
/// socket's buffers push back on the peer; and messages are read at most
/// `MESSAGES_PER_SECOND` a second after a burst of `MESSAGE_BURST`. Both
/// delay the peer rather than close the session.
pub(crate) struct SessionTransport<T> {
    inner: T,
    session: Arc<Session>,
    unanswered: Arc<Unanswered>,
    limit: usize,
    pace: Pace,
}

impl<T> SessionTransport<T> {
    pub(crate) fn new(inner: T, session: Arc<Session>, limit: usize) -> Self {
        Self {
            inner,
            session,
            unanswered: Arc::new(Unanswered::default()),
            limit,
            pace: Pace::new(
                super::MESSAGES_PER_SECOND,
                super::MESSAGE_BURST,
                Instant::now(),
            ),
        }
    }
}

impl<T: Transport<RoleServer>> Transport<RoleServer> for SessionTransport<T> {
    type Error = T::Error;

    fn send(
        &mut self,
        item: TxJsonRpcMessage<RoleServer>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
        let answers = match &item {
            JsonRpcMessage::Response(response) => Some(response.id.clone()),
            JsonRpcMessage::Error(error) => error.id.clone(),
            JsonRpcMessage::Request(_) | JsonRpcMessage::Notification(_) => None,
        };
        let answer = answers.map(|id| Answer {
            unanswered: self.unanswered.clone(),
            id,
        });
        let send = self.inner.send(item);
        async move {
            let result = send.await;
            drop(answer);
            result
        }
    }

    /// Cancel-safe like the inner `receive`: the waits come first, and the
    /// books are kept only once a message has been read.
    async fn receive(&mut self) -> Option<RxJsonRpcMessage<RoleServer>> {
        loop {
            let wait = self.pace.wait(Instant::now());
            if wait.is_zero() {
                break;
            }
            tokio::time::sleep(wait).await;
        }
        while self.unanswered.count() >= self.limit {
            // `notify_one` keeps a permit when nobody waits yet, so an answer
            // between the check and this wait is not missed.
            self.unanswered.answered.notified().await;
        }
        let message = self.inner.receive().await?;
        self.pace.take();
        self.session.count_message();
        match &message {
            JsonRpcMessage::Request(request) => self.unanswered.add(request.id.clone()),
            // rmcp drops the answer of a request the peer cancelled.
            JsonRpcMessage::Notification(notification) => {
                if let ClientNotification::CancelledNotification(cancelled) =
                    &notification.notification
                {
                    if let Some(id) = &cancelled.params.request_id {
                        self.unanswered.settle(id);
                    }
                }
            }
            JsonRpcMessage::Response(_) | JsonRpcMessage::Error(_) => {}
        }
        Some(message)
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        self.inner.close().await
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::Role;
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn session() -> Arc<Session> {
        Arc::new(Session::new(1, None, "u", Role::ReadOnly))
    }

    #[tokio::test]
    async fn guarded_reader_stamps_activity_and_caps_lines() {
        let session = session();
        let (mut client, server) = tokio::io::duplex(64 * 1024);
        let mut reader = GuardedReader::new(server, 8, session.clone());

        client.write_all(b"12345678\nabc").await.unwrap();
        let mut buf = vec![0u8; 64];
        let n = reader.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"12345678\nabc");
        assert!(!session.frame_too_large());

        client.write_all(b"defghi").await.unwrap();
        let error = reader.read(&mut buf).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(session.frame_too_large());
    }

    #[tokio::test]
    async fn guarded_writer_notes_a_stall_and_cut_fails_the_waiting_write() {
        let session = session();
        // Room for 16 bytes; the peer never reads.
        let (mut peer, server) = tokio::io::duplex(16);
        let mut writer = GuardedWriter::new(server, session.clone());

        writer.write_all(b"0123456789").await.unwrap();
        assert_eq!(session.writes.stalled_for(), Duration::ZERO);

        let blocked = tokio::spawn(async move {
            let result = writer.write_all(&[b'x'; 64]).await;
            (writer, result)
        });
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!blocked.is_finished(), "the write waits for the peer");
        assert!(session.writes.stalled_for() >= Duration::from_millis(40));

        session.writes.cut();
        let (mut writer, result) = blocked.await.unwrap();
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::BrokenPipe);
        let again = writer.write_all(b"y").await.unwrap_err();
        assert_eq!(again.kind(), io::ErrorKind::BrokenPipe);

        // What was written before the cut is still there.
        let mut buf = vec![0u8; 10];
        peer.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"0123456789");
    }

    #[tokio::test]
    async fn guarded_writer_clears_the_stall_once_the_peer_reads() {
        let session = session();
        let (mut peer, server) = tokio::io::duplex(16);
        let mut writer = GuardedWriter::new(server, session.clone());
        let blocked = tokio::spawn(async move {
            writer.write_all(&[b'x'; 64]).await.unwrap();
            writer
        });
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(session.writes.stalled_for() > Duration::ZERO);

        let mut buf = vec![0u8; 64];
        peer.read_exact(&mut buf).await.unwrap();
        blocked.await.unwrap();
        assert_eq!(session.writes.stalled_for(), Duration::ZERO);
        assert!(!session.writes.is_cut());
    }

    #[test]
    fn unanswered_requests_settle_once() {
        let unanswered = Unanswered::default();
        let (one, two) = (RequestId::Number(1), RequestId::String("two".into()));
        unanswered.add(one.clone());
        unanswered.add(two.clone());
        unanswered.add(two.clone());
        assert_eq!(unanswered.count(), 3);
        unanswered.settle(&two);
        assert_eq!(unanswered.count(), 2);
        unanswered.settle(&one);
        unanswered.settle(&one);
        unanswered.settle(&RequestId::Number(9));
        assert_eq!(unanswered.count(), 1);

        // An answer settles its request when its write ends, however it ends.
        let unanswered = Arc::new(unanswered);
        drop(Answer {
            unanswered: unanswered.clone(),
            id: two,
        });
        assert_eq!(unanswered.count(), 0);
    }

    #[tokio::test]
    async fn session_transport_reads_no_further_while_requests_are_unanswered() {
        use rmcp::transport::async_rw::AsyncRwTransport;
        let session = session();
        // The peer writes requests and never reads an answer.
        let (mut peer, server) = tokio::io::duplex(64 * 1024);
        let (read, write) = tokio::io::split(server);
        let inner = AsyncRwTransport::new_server(read, write);
        let mut transport = SessionTransport::new(inner, session.clone(), 2);
        let ping = |id: u32| format!("{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"ping\"}}\n");
        let cancel = "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/cancelled\",\
                      \"params\":{\"requestId\":1}}\n";
        for line in [ping(1), ping(2), cancel.to_string(), ping(3)] {
            peer.write_all(line.as_bytes()).await.unwrap();
        }
        let blocked = Duration::from_millis(50);
        let answer = |id: i64| {
            TxJsonRpcMessage::<RoleServer>::error(
                rmcp::ErrorData::internal_error("x", None),
                Some(RequestId::Number(id)),
            )
        };

        for id in [1, 2] {
            match transport.receive().await {
                Some(JsonRpcMessage::Request(request)) => {
                    assert_eq!(request.id, RequestId::Number(id))
                }
                other => panic!("{other:?}"),
            }
        }
        // Two unanswered: the cancellation waits in the socket.
        assert!(tokio::time::timeout(blocked, transport.receive())
            .await
            .is_err());
        // An answer written lets the next message in, which cancels the
        // other request; that frees room for the third.
        transport.send(answer(2)).await.unwrap();
        assert!(matches!(
            transport.receive().await,
            Some(JsonRpcMessage::Notification(_))
        ));
        assert_eq!(transport.unanswered.count(), 0);
        assert!(matches!(
            transport.receive().await,
            Some(JsonRpcMessage::Request(_))
        ));
        assert_eq!(transport.unanswered.count(), 1);
        // An answer whose write is dropped, never done, settles too.
        drop(transport.send(answer(3)));
        assert_eq!(transport.unanswered.count(), 0);
        assert_eq!(session.messages(), 4);
    }

    #[test]
    fn pace_allows_a_burst_then_a_steady_rate() {
        let start = Instant::now();
        let mut pace = Pace::new(20, 3, start);
        for _ in 0..3 {
            assert_eq!(pace.wait(start), Duration::ZERO);
            pace.take();
        }
        let wait = pace.wait(start);
        assert!(
            wait > Duration::from_millis(45) && wait <= Duration::from_millis(50),
            "{wait:?}"
        );
        let later = start + Duration::from_millis(60);
        assert_eq!(pace.wait(later), Duration::ZERO);
        pace.take();
        // A long pause earns no more than the burst.
        let much_later = later + Duration::from_secs(60);
        for _ in 0..3 {
            assert_eq!(pace.wait(much_later), Duration::ZERO);
            pace.take();
        }
        assert!(pace.wait(much_later) > Duration::ZERO);
    }
}
