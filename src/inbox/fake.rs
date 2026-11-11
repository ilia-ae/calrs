//! Tests only: an in-process IMAP server. Plain TCP on 127.0.0.1 for the
//! protocol and poller tests, and TLS with a static test certificate (a test
//! CA, valid for a century, and a `localhost`/`127.0.0.1` leaf) for the
//! `CALRS_INBOX_IMAP_CA_FILE` path. It records every command (LOGIN without
//! its arguments) and counts connections and logins.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

use super::tls::{ConnectError, Connected, Connector};

pub(crate) const USER: &str = "booking@ilia.ae";
pub(crate) const PASSWORD: &str = "secret-pw";

/// The test CA (also what `CALRS_INBOX_IMAP_CA_FILE` points at).
pub(crate) const CA_PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIBgTCCASegAwIBAgITJ9AiDK+ESM20loPPoPLmUWIehTAKBggqhkjOPQQDAjAe\nMRwwGgYDVQQDDBNjYWxycyB0ZXN0IGluYm94IENBMCAXDTI2MTExMDExMzAwOFoY\nDzIxMjYxMDE3MTEzMDA4WjAeMRwwGgYDVQQDDBNjYWxycyB0ZXN0IGluYm94IENB\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEX7lny0AqbAbrXumO3BumMVU8AMWk\nEjg2KFsNj2NjUBSNwzDN1VKSfSoV9YSn2f32TMDHQcVBDubioLiJYZCPNKNCMEAw\nHQYDVR0OBBYEFLz0uXZZ2w242oVgPGIvVGsm1ai7MA8GA1UdEwEB/wQFMAMBAf8w\nDgYDVR0PAQH/BAQDAgEGMAoGCCqGSM49BAMCA0gAMEUCIQDmvgnoUtC+SDM92BYx\nn9w2sWbl6fmFCTmqbzNJhoQNiQIgKi5KF67ND6ikB5vyW6yGTrcFMQdcKSp/ic99\nuFqAFi8=\n-----END CERTIFICATE-----\n";
const LEAF_PEM: &str = "-----BEGIN CERTIFICATE-----
MIIByjCCAW+gAwIBAgIUaEhAHEgnXvIyEqUFLcmxaaUacmIwCgYIKoZIzj0EAwIw
HjEcMBoGA1UEAwwTY2FscnMgdGVzdCBpbmJveCBDQTAgFw0yNjExMTAxMTUxNDNa
GA8yMTI2MTAxNzExNTE0M1owFDESMBAGA1UEAwwJbG9jYWxob3N0MFkwEwYHKoZI
zj0CAQYIKoZIzj0DAQcDQgAE3u8Z6A9DE9ozM/eMOZHhanE5xXiHUoGVqZjElLh1
rtSYFZ7V7dEXPgts0j79iCAFqV4Ig2aVf3uWxSvvz2Q5F6OBkjCBjzAMBgNVHRMB
Af8EAjAAMA4GA1UdDwEB/wQEAwIHgDATBgNVHSUEDDAKBggrBgEFBQcDATAaBgNV
HREEEzARgglsb2NhbGhvc3SHBH8AAAEwHQYDVR0OBBYEFAf5XDMUnPYx1QSD7G/n
Z6ffviuHMB8GA1UdIwQYMBaAFLz0uXZZ2w242oVgPGIvVGsm1ai7MAoGCCqGSM49
BAMCA0kAMEYCIQDn6p3ftGb6lT2hFum9wajm6zBxKAjWgey4oSo7GPLUbAIhAKFh
hgvDWnw0UDXHwtq4dqbwtoYwnNBKKznABpNGt/O+
-----END CERTIFICATE-----
";
const LEAF_KEY: &str = "-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgGoJMpYF0efJemwSA\nhxoUk9ht5F4wPKrIX9A6oHyd1z6hRANCAATe7xnoD0MT2jMz94w5keFqcTnFeIdS\ngZWpmMSUuHWu1JgVntXt0Rc+C2zSPv2IIAWpXgiDZpV/e5bFK+/PZDkX\n-----END PRIVATE KEY-----\n";

#[derive(Debug, Clone)]
pub(crate) struct FakeMsg {
    pub(crate) uid: u32,
    pub(crate) raw: Vec<u8>,
    pub(crate) flags: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct FakeState {
    pub(crate) uidvalidity: u32,
    pub(crate) uidnext: u32,
    pub(crate) msgs: Vec<FakeMsg>,
    /// Extra capabilities (`MOVE`, `UIDPLUS`).
    pub(crate) caps: Vec<&'static str>,
    pub(crate) accept_login: bool,
    /// Answer SELECT with NO (a folder that does not exist).
    pub(crate) refuse_select: bool,
    /// Close the connection, unanswered, at the next command with this
    /// name (once).
    pub(crate) drop_at: Option<&'static str>,
    pub(crate) connections: u32,
    pub(crate) logins: u32,
    pub(crate) commands: Vec<String>,
    /// (uid, folder) moved or copied away.
    pub(crate) moved: Vec<(u32, String)>,
    pub(crate) copied: Vec<(u32, String)>,
    pub(crate) expunged: Vec<u32>,
    /// Answer like iCloud (imap.mail.me.com, 2026-11-11): a bare `*` as a
    /// message set (`FETCH * (UID)`, `UID FETCH * …`, `UID SEARCH UID *`)
    /// is `BAD Parse Error`, and `UID <n>:*` above the last UID matches
    /// nothing (RFC 3501: the last message).
    pub(crate) icloud: bool,
}

impl Default for FakeState {
    fn default() -> Self {
        Self {
            uidvalidity: 1000,
            uidnext: 1,
            msgs: Vec::new(),
            caps: Vec::new(),
            accept_login: true,
            refuse_select: false,
            drop_at: None,
            connections: 0,
            logins: 0,
            commands: Vec::new(),
            moved: Vec::new(),
            copied: Vec::new(),
            expunged: Vec::new(),
            icloud: false,
        }
    }
}

impl FakeState {
    /// Deliver a message; returns its UID.
    pub(crate) fn deliver(&mut self, raw: &str, seen: bool) -> u32 {
        let uid = self.uidnext;
        self.uidnext += 1;
        self.msgs.push(FakeMsg {
            uid,
            raw: raw.as_bytes().to_vec(),
            flags: if seen {
                vec!["\\Seen".into()]
            } else {
                Vec::new()
            },
        });
        uid
    }

    pub(crate) fn flags_of(&self, uid: u32) -> Option<Vec<String>> {
        self.msgs
            .iter()
            .find(|m| m.uid == uid)
            .map(|m| m.flags.clone())
    }

    /// Commands seen, by name (`UID FETCH`, `LOGIN`, …).
    pub(crate) fn count(&self, name: &str) -> usize {
        self.commands.iter().filter(|c| c.starts_with(name)).count()
    }
}

pub(crate) type Shared = Arc<Mutex<FakeState>>;

pub(crate) fn shared() -> Shared {
    Arc::new(Mutex::new(FakeState::default()))
}

/// Serve `state` on a plain TCP port of 127.0.0.1.
pub(crate) async fn serve(state: Shared) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(session(stream, state.clone()));
        }
    });
    port
}

/// A certificate the way Kerio's built-in one is made (`openssl req
/// -x509`): self-signed, CN=localhost, CA:TRUE, no subjectAltName.
pub(crate) const KERIO_SELF_SIGNED_PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIBXjCCAQSgAwIBAgIUHPk4z6EjvWG2OYwA1jPK6+MCj5AwCgYIKoZIzj0EAwIw\nFDESMBAGA1UEAwwJbG9jYWxob3N0MCAXDTI2MTExMDEyNDIzMloYDzIxMjYxMDE3\nMTI0MjMyWjAUMRIwEAYDVQQDDAlsb2NhbGhvc3QwWTATBgcqhkjOPQIBBggqhkjO\nPQMBBwNCAATChqQSEH/E3iuQcBzRmCk3o9bv5DHF9zO07HUigkrJhowPzMPTLP8r\n8unyIGkd8v+VSjlPkcMxujfk+dnUNji3ozIwMDAPBgNVHRMBAf8EBTADAQH/MB0G\nA1UdDgQWBBSK9yQozuvvoANPCk+jryT8PfJugTAKBggqhkjOPQQDAgNIADBFAiAe\n0Y2Ph3fI3qjf+W2kVqAF7plwNcXAngt539rIeVtooAIhAK4JwpZGUk01QTMw+Vr+\nC493gW74PpehubbQxYkowxzO\n-----END CERTIFICATE-----\n";
const KERIO_SELF_SIGNED_KEY: &str = "-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgohvAijI3RgOHOCIq\nSn4JNDsJmrXYkMbfxaMoZAlUNImhRANCAATChqQSEH/E3iuQcBzRmCk3o9bv5DHF\n9zO07HUigkrJhowPzMPTLP8r8unyIGkd8v+VSjlPkcMxujfk+dnUNji3\n-----END PRIVATE KEY-----\n";
/// `openssl x509 -noout -fingerprint -sha256` of it.
pub(crate) const KERIO_SELF_SIGNED_SHA256: &str =
    "D0:D0:F8:3C:E7:0B:5F:A8:F8:CB:BC:CE:7B:94:B4:35:17:C8:BA:C9:84:0B:A5:51:B8:79:E0:29:E5:D1:BD:28";

/// Serve `state` over TLS with the test certificate.
pub(crate) async fn serve_tls(state: Shared) -> u16 {
    serve_tls_with(state, LEAF_PEM, LEAF_KEY).await
}

/// Serve `state` over TLS with the Kerio-style self-signed certificate.
pub(crate) async fn serve_tls_self_signed(state: Shared) -> u16 {
    serve_tls_with(state, KERIO_SELF_SIGNED_PEM, KERIO_SELF_SIGNED_KEY).await
}

async fn serve_tls_with(state: Shared, cert_pem: &str, key_pem: &str) -> u16 {
    use tokio_rustls::rustls::{self, pki_types::pem::PemObject};
    let chain =
        vec![rustls::pki_types::CertificateDer::from_pem_slice(cert_pem.as_bytes()).unwrap()];
    let key = rustls::pki_types::PrivateKeyDer::from_pem_slice(key_pem.as_bytes()).unwrap();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(chain, key)
        .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let (acceptor, state) = (acceptor.clone(), state.clone());
            tokio::spawn(async move {
                if let Ok(tls) = acceptor.accept(stream).await {
                    session(tls, state).await;
                }
            });
        }
    });
    port
}

/// A connector to the plain fake (no TLS).
pub(crate) fn plain_connector(port: u16) -> Connector {
    Arc::new(move || {
        Box::pin(async move {
            let s = tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .map_err(|_| ConnectError::Unreachable)?;
            Ok(Box::new(s) as Connected)
        })
    })
}

/// A connector that never reaches anything.
pub(crate) fn dead_connector() -> Connector {
    Arc::new(|| Box::pin(async { Err(ConnectError::Unreachable) }))
}

/// The UIDs of an IMAP sequence set (`5`, `3,4`, `2:*`).
fn uid_set(set: &str, max: u32, icloud: bool) -> Vec<u32> {
    let mut out = Vec::new();
    for part in set.split(',') {
        match part.split_once(':') {
            Some((a, b)) => {
                let a: u32 = a.parse().unwrap_or(1);
                if icloud && b == "*" && a > max {
                    continue;
                }
                let b: u32 = if b == "*" {
                    max
                } else {
                    b.parse().unwrap_or(max)
                };
                let (lo, hi) = (a.min(b), a.max(b));
                out.extend(lo..=hi);
            }
            None => out.extend(part.parse::<u32>().ok()),
        }
    }
    out
}

/// The field names of a `HEADER.FIELDS (…)` fetch item, uppercased.
fn header_fields(items: &str) -> Option<Vec<String>> {
    let at = items.find("HEADER.FIELDS (")? + "HEADER.FIELDS (".len();
    let end = items[at..].find(')')? + at;
    Some(
        items[at..end]
            .split_whitespace()
            .map(str::to_ascii_uppercase)
            .collect(),
    )
}

/// The header lines of `raw` named in `names` (with their continuation
/// lines), then a blank line, as a server answers `HEADER.FIELDS`.
fn pick_headers(raw: &[u8], names: &[String]) -> Vec<u8> {
    let text = String::from_utf8_lossy(raw);
    let header = text
        .split_once("\r\n\r\n")
        .map_or(text.as_ref(), |(h, _)| h);
    let mut out = String::new();
    let mut keep = false;
    for line in header.split("\r\n") {
        if !line.starts_with([' ', '\t']) {
            let name = line.split(':').next().unwrap_or("").to_ascii_uppercase();
            keep = names.contains(&name);
        }
        if keep {
            out.push_str(line);
            out.push_str("\r\n");
        }
    }
    out.push_str("\r\n");
    out.into_bytes()
}

fn unquote(s: &str) -> String {
    s.trim()
        .trim_matches('"')
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

async fn session<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    stream: S,
    state: Shared,
) {
    let (read, mut write) = tokio::io::split(stream);
    let mut reader = BufReader::new(read);
    state.lock().unwrap().connections += 1;
    let _ = write
        .write_all(b"* OK [CAPABILITY IMAP4rev1] fake IMAP ready\r\n")
        .await;
    loop {
        let mut raw = Vec::new();
        match reader.read_until(b'\n', &mut raw).await {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        let mut line = String::from_utf8_lossy(&raw).trim_end().to_string();
        // A synchronizing literal: ask for it, read it, then the rest.
        let mut literals = Vec::new();
        while let Some(open) = line.rfind('{').filter(|_| line.ends_with('}')) {
            let n: usize = line[open + 1..line.len() - 1]
                .trim_end_matches('+')
                .parse()
                .unwrap_or(0);
            let _ = write.write_all(b"+ go ahead\r\n").await;
            let mut lit = vec![0u8; n];
            if reader.read_exact(&mut lit).await.is_err() {
                return;
            }
            literals.push(String::from_utf8_lossy(&lit).into_owned());
            line.truncate(open);
            let mut more = Vec::new();
            if reader.read_until(b'\n', &mut more).await.is_err() {
                return;
            }
            line.push('\u{0}');
            line.push_str(String::from_utf8_lossy(&more).trim_end());
        }
        let (tag, rest) = line.split_once(' ').unwrap_or((&line, ""));
        let tag = tag.to_string();
        let upper = rest.to_ascii_uppercase();
        let name = if upper.starts_with("UID ") {
            upper.splitn(3, ' ').take(2).collect::<Vec<_>>().join(" ")
        } else {
            upper.split(' ').next().unwrap_or("").to_string()
        };
        let reply = {
            let mut st = state.lock().unwrap();
            if st.drop_at == Some(leak(&name)) {
                st.drop_at = None;
                st.commands.push(name.clone());
                None
            } else {
                st.commands.push(if name == "LOGIN" {
                    "LOGIN".into()
                } else {
                    rest.to_string()
                });
                Some(answer(&mut st, &tag, &name, rest, &literals))
            }
        };
        let Some(reply) = reply else {
            return;
        };
        for chunk in reply {
            if write.write_all(&chunk).await.is_err() {
                return;
            }
            let _ = write.flush().await;
        }
        if name == "LOGOUT" {
            return;
        }
    }
}

/// Command names compared against `drop_at`.
fn leak(name: &str) -> &'static str {
    match name {
        "NOOP" => "NOOP",
        "SELECT" => "SELECT",
        "LOGIN" => "LOGIN",
        "UID SEARCH" => "UID SEARCH",
        "UID FETCH" => "UID FETCH",
        "UID STORE" => "UID STORE",
        _ => "",
    }
}

/// The response chunks to one command. A body literal is split across two
/// writes, so the client has to put it back together.
fn answer(
    st: &mut FakeState,
    tag: &str,
    name: &str,
    rest: &str,
    literals: &[String],
) -> Vec<Vec<u8>> {
    let ok = |text: &str| format!("{tag} OK {text}\r\n").into_bytes();
    let args = rest.split_once(' ').map_or("", |x| x.1).to_string();
    match name {
        "CAPABILITY" => {
            let mut caps = vec!["IMAP4rev1"];
            caps.extend(st.caps.iter().copied());
            vec![
                format!("* CAPABILITY {}\r\n", caps.join(" ")).into_bytes(),
                ok("done"),
            ]
        }
        "LOGIN" => {
            st.logins += 1;
            let mut parts = args.splitn(2, ' ');
            let user = parts.next().map(unquote).unwrap_or_default();
            let mut pass = parts.next().map(unquote).unwrap_or_default();
            if let Some(lit) = literals.first() {
                pass = lit.clone();
            }
            if st.accept_login && user == USER && pass == PASSWORD {
                vec![ok("LOGIN completed")]
            } else {
                vec![
                    format!("{tag} NO [AUTHENTICATIONFAILED] Invalid credentials\r\n").into_bytes(),
                ]
            }
        }
        "SELECT" if st.refuse_select => {
            vec![format!("{tag} NO [NONEXISTENT] Unknown folder\r\n").into_bytes()]
        }
        "SELECT" => vec![
            format!(
                "* {} EXISTS\r\n* 0 RECENT\r\n* FLAGS (\\Seen \\Deleted)\r\n",
                st.msgs.len()
            )
            .into_bytes(),
            format!(
                "* OK [UIDVALIDITY {}] UIDs valid\r\n* OK [UIDNEXT {}] next\r\n",
                st.uidvalidity, st.uidnext
            )
            .into_bytes(),
            format!("{tag} OK [READ-WRITE] SELECT completed\r\n").into_bytes(),
        ],
        "NOOP" => vec![
            format!("* {} EXISTS\r\n", st.msgs.len()).into_bytes(),
            ok("NOOP"),
        ],
        "UID SEARCH" => {
            let crit = rest
                .splitn(3, ' ')
                .nth(2)
                .unwrap_or("")
                .to_ascii_uppercase();
            let max = st.msgs.iter().map(|m| m.uid).max().unwrap_or(0);
            // Keys are ANDed: `UNSEEN`, `UID <set>`, `ALL`.
            let mut unseen = false;
            let mut want: Option<std::collections::HashSet<u32>> = None;
            let mut words = crit.split_whitespace();
            while let Some(word) = words.next() {
                match word {
                    "UNSEEN" => unseen = true,
                    "UID" => {
                        let set = words.next().unwrap_or("");
                        if st.icloud && set == "*" {
                            return vec![format!("{tag} BAD Parse Error\r\n").into_bytes()];
                        }
                        want = Some(uid_set(set, max, st.icloud).into_iter().collect());
                    }
                    _ => {}
                }
            }
            let uids: Vec<u32> = st
                .msgs
                .iter()
                .filter(|m| !unseen || !m.flags.iter().any(|f| f == "\\Seen"))
                .filter(|m| want.as_ref().is_none_or(|w| w.contains(&m.uid)))
                .map(|m| m.uid)
                .collect();
            let list: Vec<String> = uids.iter().map(u32::to_string).collect();
            vec![
                format!("* SEARCH {}\r\n", list.join(" "))
                    .trim_end()
                    .as_bytes()
                    .iter()
                    .copied()
                    .chain(b"\r\n".iter().copied())
                    .collect(),
                ok("SEARCH"),
            ]
        }
        "FETCH" => {
            // `FETCH <n> (UID)` or `FETCH * (UID)`: that message's UID.
            let set = rest.split(' ').nth(1).unwrap_or("");
            if st.icloud && set == "*" {
                return vec![format!("{tag} BAD Parse Error\r\n").into_bytes()];
            }
            let at = if set == "*" {
                st.msgs.len()
            } else {
                set.parse().unwrap_or(0)
            };
            let mut out = Vec::new();
            if let Some(m) = at.checked_sub(1).and_then(|i| st.msgs.get(i)) {
                out.push(format!("* {at} FETCH (UID {})\r\n", m.uid).into_bytes());
            }
            out.push(ok("FETCH"));
            out
        }
        "UID FETCH" => {
            let mut parts = rest.splitn(4, ' ').skip(2);
            let set = parts.next().unwrap_or("");
            let items = parts.next().unwrap_or("").to_ascii_uppercase();
            let max = st.msgs.iter().map(|m| m.uid).max().unwrap_or(0);
            if st.icloud && set == "*" {
                return vec![format!("{tag} BAD Parse Error\r\n").into_bytes()];
            }
            let want = uid_set(set, max, st.icloud);
            let mut out = Vec::new();
            for (i, m) in st.msgs.iter().enumerate() {
                if !want.contains(&m.uid) {
                    continue;
                }
                if let Some(names) = header_fields(&items) {
                    // `BODY.PEEK[HEADER.FIELDS (…)]`: those header lines
                    // (folded ones whole) and the blank line after them.
                    let picked = pick_headers(&m.raw, &names);
                    let mut head = format!(
                        "* {} FETCH (UID {} RFC822.SIZE {} BODY[HEADER.FIELDS ({})] {{{}}}\r\n",
                        i + 1,
                        m.uid,
                        m.raw.len(),
                        names.join(" "),
                        picked.len()
                    )
                    .into_bytes();
                    head.extend_from_slice(&picked);
                    head.extend_from_slice(b")\r\n");
                    out.push(head);
                } else if items.contains("BODY.PEEK[]") {
                    let (a, b) = m.raw.split_at(m.raw.len() / 2);
                    let mut head = format!(
                        "* {} FETCH (UID {} BODY[]<0> {{{}}}\r\n",
                        i + 1,
                        m.uid,
                        m.raw.len()
                    )
                    .into_bytes();
                    head.extend_from_slice(a);
                    out.push(head);
                    let mut tail = b.to_vec();
                    tail.extend_from_slice(b")\r\n");
                    out.push(tail);
                } else {
                    out.push(
                        format!(
                            "* {} FETCH (UID {} RFC822.SIZE {} FLAGS ({}))\r\n",
                            i + 1,
                            m.uid,
                            m.raw.len(),
                            m.flags.join(" ")
                        )
                        .into_bytes(),
                    );
                }
            }
            out.push(ok("FETCH"));
            out
        }
        "UID STORE" => {
            let mut parts = rest.splitn(5, ' ').skip(2);
            let uid: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
            let flag = rest
                .split('(')
                .nth(1)
                .unwrap_or("")
                .trim_end_matches(')')
                .to_string();
            if let Some(m) = st.msgs.iter_mut().find(|m| m.uid == uid) {
                if !m.flags.contains(&flag) {
                    m.flags.push(flag);
                }
            }
            vec![ok("STORE")]
        }
        "UID MOVE" | "UID COPY" => {
            if name == "UID MOVE" && !st.caps.contains(&"MOVE") {
                return vec![format!("{tag} BAD unknown command\r\n").into_bytes()];
            }
            let mut parts = rest.splitn(4, ' ').skip(2);
            let uid: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
            let folder = unquote(parts.next().unwrap_or(""));
            let mut out = Vec::new();
            if name == "UID MOVE" {
                // RFC 6851: the moved message is expunged here, by number.
                if let Some(i) = st.msgs.iter().position(|m| m.uid == uid) {
                    st.msgs.remove(i);
                    out.push(format!("* {} EXPUNGE\r\n", i + 1).into_bytes());
                }
                st.moved.push((uid, folder));
            } else {
                st.copied.push((uid, folder));
            }
            out.push(ok("done"));
            out
        }
        "UID EXPUNGE" => {
            let uid: u32 = rest
                .splitn(3, ' ')
                .nth(2)
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            let deleted = st
                .msgs
                .iter()
                .position(|m| m.uid == uid && m.flags.iter().any(|f| f == "\\Deleted"));
            let mut out = Vec::new();
            if let Some(i) = deleted {
                st.msgs.remove(i);
                st.expunged.push(uid);
                out.push(format!("* {} EXPUNGE\r\n", i + 1).into_bytes());
            }
            out.push(ok("EXPUNGE"));
            out
        }
        "LOGOUT" => vec![b"* BYE logging out\r\n".to_vec(), ok("LOGOUT")],
        _ => vec![format!("{tag} BAD unknown command\r\n").into_bytes()],
    }
}
