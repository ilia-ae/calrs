//! Inbox tests over the in-process fake server (`fake.rs`): the protocol
//! client, its caps, TLS with the CA file. The poller against a database
//! (matching, storing, relaying, back-off) is in `web/tests/inbox.rs`.

use super::fake::{self, CA_PEM, PASSWORD, USER};
use super::imap::{Client, Error};
use super::tls;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn plain_client(port: u16) -> Client<tokio::net::TcpStream> {
    let s = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    Client::new(s).with_timeout(std::time::Duration::from_secs(5))
}

#[tokio::test]
async fn a_session_selects_searches_and_fetches_split_literals() {
    let state = fake::shared();
    let (a, b) = {
        let mut st = state.lock().unwrap();
        st.caps = vec!["MOVE", "UIDPLUS"];
        let a = st.deliver("Subject: one\r\n\r\nbody one\r\n", true);
        let b = st.deliver("Subject: two\r\n\r\nbody two, a bit longer\r\n", false);
        (a, b)
    };
    let port = fake::serve(state.clone()).await;
    let mut c = plain_client(port).await;
    assert!(!c.greeting().await.unwrap());
    c.capability().await.unwrap();
    assert!(c.has("MOVE") && c.has("uidplus"));
    c.login(USER, PASSWORD).await.unwrap();
    let sel = c.select("INBOX").await.unwrap();
    assert_eq!(
        (sel.uidvalidity, sel.uidnext, sel.exists),
        (1000, Some(3), 2)
    );
    assert_eq!(c.uid_search("UNSEEN").await.unwrap(), vec![b]);
    // `n:*` past the last UID still names the last message.
    assert_eq!(c.uid_search("UID 9:*").await.unwrap(), vec![b]);
    let meta = c
        .uid_fetch(&format!("{a},{b}"), "(UID RFC822.SIZE FLAGS)")
        .await
        .unwrap();
    assert_eq!(meta.len(), 2);
    assert_eq!(meta[0].flags, vec!["\\Seen".to_string()]);
    assert_eq!(
        meta[1].size,
        Some("Subject: two\r\n\r\nbody two, a bit longer\r\n".len() as u32)
    );
    let body = c
        .uid_fetch(&b.to_string(), "(UID BODY.PEEK[]<0.262144>)")
        .await
        .unwrap();
    assert_eq!(
        body[0].body.as_deref(),
        Some(&b"Subject: two\r\n\r\nbody two, a bit longer\r\n"[..])
    );
    c.uid_add_flag(b, "\\Seen").await.unwrap();
    c.uid_move(b, "done").await.unwrap();
    assert_eq!(c.last_uid().await.unwrap(), Some(a));
    c.logout().await.unwrap();
    let st = state.lock().unwrap();
    assert_eq!(st.moved, vec![(b, "done".to_string())]);
    assert_eq!(st.logins, 1);
    // The password is never in what the server recorded.
    assert!(!st.commands.iter().any(|c| c.contains(PASSWORD)));
}

#[tokio::test]
async fn a_refused_login_is_no_and_an_8_bit_password_goes_as_a_literal() {
    let state = fake::shared();
    state.lock().unwrap().accept_login = false;
    let port = fake::serve(state.clone()).await;
    let mut c = plain_client(port).await;
    c.greeting().await.unwrap();
    c.capability().await.unwrap();
    assert!(matches!(c.login(USER, PASSWORD).await, Err(Error::No)));
    // A non-ASCII password: the fake asks for the literal with `+`.
    assert!(matches!(c.login(USER, "пароль").await, Err(Error::No)));
    assert_eq!(state.lock().unwrap().logins, 2);
}

/// A scripted server on a duplex: whatever `script` says, then silence.
async fn scripted(script: Vec<u8>) -> Client<tokio::io::DuplexStream> {
    let (client, mut server) = tokio::io::duplex(1 << 20);
    tokio::spawn(async move {
        let mut sink = [0u8; 4096];
        let _ = server.write_all(&script).await;
        while let Ok(n) = server.read(&mut sink).await {
            if n == 0 {
                break;
            }
        }
    });
    Client::new(client).with_timeout(std::time::Duration::from_millis(500))
}

#[tokio::test]
async fn caps_drop_the_session_before_buffering_too_much() {
    // A line over 64 KiB.
    let mut script = b"* OK hi\r\n* ".to_vec();
    script.extend(std::iter::repeat_n(b'x', 70 * 1024));
    let mut c = scripted(script).await;
    c.greeting().await.unwrap();
    assert!(matches!(c.noop().await, Err(Error::TooLarge)));
    // A literal announced over 512 KiB is refused before it is read.
    let mut c = scripted(b"* OK hi\r\n* 1 FETCH (UID 1 BODY[] {600000}\r\nxx".to_vec()).await;
    c.greeting().await.unwrap();
    assert!(matches!(
        c.uid_fetch("1", "(BODY.PEEK[])").await,
        Err(Error::TooLarge)
    ));
    // Silence is a timeout, and a BYE ends the session.
    let mut c = scripted(b"* OK hi\r\n".to_vec()).await;
    c.greeting().await.unwrap();
    assert!(matches!(c.noop().await, Err(Error::Timeout)));
    let mut c = scripted(b"* BYE go away\r\n".to_vec()).await;
    assert!(matches!(c.greeting().await, Err(Error::Bye)));
}

#[tokio::test]
async fn untagged_lines_interleave_with_the_answer() {
    let mut c = scripted(
        b"* OK hi\r\n* 3 EXISTS\r\n* 1 RECENT\r\n* SEARCH 4 2\r\n* 5 EXPUNGE\r\na0001 OK done\r\n"
            .to_vec(),
    )
    .await;
    c.greeting().await.unwrap();
    assert_eq!(c.uid_search("ALL").await.unwrap(), vec![2, 4]);
}

#[tokio::test]
async fn tls_trusts_the_ca_file_and_refuses_without_it() {
    let state = fake::shared();
    state
        .lock()
        .unwrap()
        .deliver("Subject: x\r\n\r\nx\r\n", false);
    let port = fake::serve_tls(state.clone()).await;
    // Without the CA the certificate is untrusted: a transport failure.
    let refused = tls::connect_tls("localhost", port, tls::client_config(None).unwrap()).await;
    assert!(matches!(refused, Err(tls::ConnectError::Certificate(_))));
    let tls_config = tls::client_config(Some(CA_PEM.as_bytes())).unwrap();
    for host in ["localhost", "127.0.0.1"] {
        let stream = tls::connect_tls(host, port, tls_config.clone())
            .await
            .unwrap();
        let mut c = Client::new(stream);
        c.greeting().await.unwrap();
        c.capability().await.unwrap();
        c.login(USER, PASSWORD).await.unwrap();
        assert_eq!(c.select("INBOX").await.unwrap().exists, 1);
        c.logout().await.unwrap();
    }
    // A CA file without a certificate is refused as such.
    assert_eq!(
        tls::client_config(Some(b"not a pem")).err(),
        Some(tls::ConnectError::CaFile)
    );
    // The production connector reads the file.
    let dir = std::env::temp_dir().join(format!("calrs-inbox-ca-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let ca = dir.join("ca.pem");
    std::fs::write(&ca, CA_PEM).unwrap();
    let mut config = super::Config::for_tests(USER, false);
    config.host = "localhost".into();
    config.port = port;
    config.ca_file = Some(ca);
    let connect = tls::connector(std::sync::Arc::new(config));
    assert!(connect().await.is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}

/// Kerio's built-in certificate (self-signed, CA:TRUE, a CN and no SAN) is
/// refused by webpki even as the CA file, with the reason in the error; the
/// pin of its SHA-256 is what trusts it.
#[tokio::test]
async fn a_kerio_self_signed_certificate_needs_the_pin_not_the_ca_file() {
    let state = fake::shared();
    let port = fake::serve_tls_self_signed(state.clone()).await;
    let as_ca = tls::client_config(Some(fake::KERIO_SELF_SIGNED_PEM.as_bytes())).unwrap();
    match tls::connect_tls("localhost", port, as_ca).await {
        Err(tls::ConnectError::Certificate(why)) => {
            assert_eq!(why, "CaUsedAsEndEntity");
            // The reason, never the certificate.
            assert!(!why.contains("MII"), "{why}");
            let shown = tls::ConnectError::Certificate(why.clone()).to_string();
            assert!(
                shown.starts_with("the server certificate was refused ("),
                "{shown}"
            );
        }
        other => panic!("expected a refused certificate, got {:?}", other.err()),
    }
    // webpki refuses a CA:TRUE certificate as the server's own before it
    // looks for an issuer, with the CA file or without.
    let none = tls::client_config(None).unwrap();
    assert_eq!(
        tls::connect_tls("localhost", port, none).await.err(),
        Some(tls::ConnectError::Certificate("CaUsedAsEndEntity".into()))
    );
    // The pin trusts it, under any name.
    let pin = super::config::parse_sha256(fake::KERIO_SELF_SIGNED_SHA256).unwrap();
    for host in ["localhost", "127.0.0.1"] {
        let stream = tls::connect_tls(host, port, tls::pinned_config(pin).unwrap())
            .await
            .unwrap();
        let mut c = Client::new(stream);
        c.greeting().await.unwrap();
        c.logout().await.unwrap();
    }
    // A wrong pin is refused, and so is the right pin for another server.
    let mut wrong = pin;
    wrong[0] ^= 1;
    assert_eq!(
        tls::connect_tls("localhost", port, tls::pinned_config(wrong).unwrap())
            .await
            .err(),
        Some(tls::ConnectError::Certificate(
            "does not match the pinned SHA-256".into()
        ))
    );
    let other = fake::serve_tls(fake::shared()).await;
    assert!(
        tls::connect_tls("localhost", other, tls::pinned_config(pin).unwrap())
            .await
            .is_err()
    );
    // The production connector takes the pin from the configuration.
    let mut config = super::Config::for_tests(USER, false);
    config.host = "localhost".into();
    config.port = port;
    config.cert_sha256 = Some(pin);
    let connect = tls::connector(std::sync::Arc::new(config));
    assert!(connect().await.is_ok());
}

/// Counts the bytes a stream reads and writes.
struct Counting<S> {
    inner: S,
    read: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    written: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl<S: tokio::io::AsyncRead + Unpin> tokio::io::AsyncRead for Counting<S> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let r = std::pin::Pin::new(&mut self.inner).poll_read(cx, buf);
        let n = buf.filled().len() - before;
        self.read.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
        r
    }
}

impl<S: tokio::io::AsyncWrite + Unpin> tokio::io::AsyncWrite for Counting<S> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let r = std::pin::Pin::new(&mut self.inner).poll_write(cx, buf);
        if let std::task::Poll::Ready(Ok(n)) = &r {
            self.written
                .fetch_add(*n, std::sync::atomic::Ordering::Relaxed);
        }
        r
    }
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// Measurement (docs/operations.md, «Производительность»): IMAP bytes of
/// a session, an idle poll and a poll that reads one 3 KB reply, without
/// TLS overhead. `cargo test --locked perf_inbox_bytes -- --ignored --nocapture`.
#[tokio::test]
#[ignore]
async fn perf_inbox_bytes() {
    use super::filter::Ours;
    use super::poller::{poll_once, Ctx, Policy};
    use std::sync::atomic::Ordering::Relaxed;
    let pool = crate::telegram::fake::memory_pool().await;
    let state = fake::shared();
    let port = fake::serve(state.clone()).await;
    let (read, written) = (
        std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
    );
    let stream = Counting {
        inner: tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .unwrap(),
        read: read.clone(),
        written: written.clone(),
    };
    let ctx = Ctx::new(
        pool,
        std::sync::Arc::new(super::Config::for_tests(USER, true)),
        super::key().unwrap(),
        Ours::new(USER, None),
    );
    let total = || read.load(Relaxed) + written.load(Relaxed);
    let mut c = Client::new(stream);
    c.greeting().await.unwrap();
    c.capability().await.unwrap();
    c.login(USER, PASSWORD).await.unwrap();
    c.capability().await.unwrap();
    let sel = c.select("INBOX").await.unwrap();
    let setup = total();
    poll_once(&mut c, &ctx, &Policy::default(), sel)
        .await
        .unwrap();
    let first = total() - setup;
    let before = total();
    poll_once(&mut c, &ctx, &Policy::default(), sel)
        .await
        .unwrap();
    let idle = total() - before;
    let body = format!(
        "From: g@x.example\r\nTo: {USER}\r\nSubject: Re: hi\r\nMessage-ID: <m@x.example>\r\n\r\n{}\r\n",
        "Thank you, see you then. ".repeat(120)
    );
    state.lock().unwrap().deliver(&body, false);
    let before = total();
    poll_once(&mut c, &ctx, &Policy::default(), sel)
        .await
        .unwrap();
    let one = total() - before;
    println!(
        "inbox bytes: session setup {setup} B, first poll {first} B, idle poll {idle} B, \
         poll with one {} B reply {one} B; a day of idle polls every 2 min: {} KB",
        body.len(),
        idle * 720 / 1024
    );
}
