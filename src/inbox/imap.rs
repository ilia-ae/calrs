//! A minimal IMAP4rev1 client (RFC 3501, plus MOVE RFC 6851 and UIDPLUS
//! RFC 4315), generic over any async stream (TLS in production, a plain
//! socket or duplex in tests).
//!
//! Why our own: the protocol subset is ten commands, a crate would bring
//! 8 to 18 more crates (docs/decisions.md, the comparison table), and only a
//! reader of our own refuses an oversized literal before buffering it.
//!
//! Hard caps: a line 64 KiB, a literal 512 KiB, a command's whole response
//! 1 MiB; past any of them the session is dropped. Every command has a
//! timeout. Nothing here logs: errors carry no server text (it can echo
//! what we sent), and the caller logs counts and outcomes.

use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub(crate) const LINE_MAX: usize = 64 * 1024;
pub(crate) const LITERAL_MAX: usize = 512 * 1024;
pub(crate) const RESPONSE_MAX: usize = 1024 * 1024;
pub(crate) const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub(crate) enum Error {
    /// The connection failed or closed.
    Io(std::io::ErrorKind),
    Timeout,
    /// The server said something we cannot read.
    Protocol(&'static str),
    /// Over a size cap.
    TooLarge,
    /// The server refused the command (`NO`).
    No,
    /// The server rejected the command (`BAD`).
    Bad,
    /// The server closed the session (`BYE`).
    Bye,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(kind) => write!(f, "connection error ({kind:?})"),
            Error::Timeout => f.write_str("timed out"),
            Error::Protocol(what) => write!(f, "protocol error ({what})"),
            Error::TooLarge => f.write_str("response over the size limit"),
            Error::No => f.write_str("refused (NO)"),
            Error::Bad => f.write_str("rejected (BAD)"),
            Error::Bye => f.write_str("closed by the server (BYE)"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.kind())
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

/// A piece of a response line: text, or the bytes of a literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Seg {
    Text(Vec<u8>),
    Lit(Vec<u8>),
}

/// One logical response line, literals included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Line(pub(crate) Vec<Seg>);

impl Line {
    /// The text before the first literal.
    fn head(&self) -> &[u8] {
        match self.0.first() {
            Some(Seg::Text(t)) => t,
            _ => b"",
        }
    }

    /// Parsed into values.
    pub(crate) fn values(&self) -> Vec<Value> {
        Parser::new(&self.0).all()
    }
}

/// A parsed IMAP value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    Atom(String),
    Str(Vec<u8>),
    List(Vec<Value>),
    Nil,
}

impl Value {
    fn atom(&self) -> Option<&str> {
        match self {
            Value::Atom(a) => Some(a),
            _ => None,
        }
    }
    fn number(&self) -> Option<u32> {
        self.atom().and_then(|a| a.parse().ok())
    }
}

/// Tokenises the segments of a line.
struct Parser<'a> {
    segs: &'a [Seg],
    seg: usize,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(segs: &'a [Seg]) -> Self {
        Self {
            segs,
            seg: 0,
            pos: 0,
        }
    }

    fn text(&self) -> &'a [u8] {
        match self.segs.get(self.seg) {
            Some(Seg::Text(t)) => t,
            _ => b"",
        }
    }

    fn peek(&self) -> Option<u8> {
        self.text().get(self.pos).copied()
    }

    fn all(&mut self) -> Vec<Value> {
        let mut out = Vec::new();
        while let Some(v) = self.value(0) {
            out.push(v);
        }
        out
    }

    fn skip_spaces(&mut self) {
        while self.peek() == Some(b' ') {
            self.pos += 1;
        }
    }

    fn value(&mut self, depth: usize) -> Option<Value> {
        if depth > 16 {
            return None;
        }
        self.skip_spaces();
        let c = self.peek()?;
        match c {
            b'(' => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_spaces();
                    match self.peek() {
                        None => return Some(Value::List(items)),
                        Some(b')') => {
                            self.pos += 1;
                            return Some(Value::List(items));
                        }
                        _ => match self.value(depth + 1) {
                            Some(v) => items.push(v),
                            None => return Some(Value::List(items)),
                        },
                    }
                }
            }
            b')' => {
                // Stray close: skip it.
                self.pos += 1;
                self.value(depth)
            }
            b'"' => {
                self.pos += 1;
                let mut s = Vec::new();
                while let Some(c) = self.peek() {
                    self.pos += 1;
                    match c {
                        b'\\' => {
                            if let Some(n) = self.peek() {
                                s.push(n);
                                self.pos += 1;
                            }
                        }
                        b'"' => break,
                        c => s.push(c),
                    }
                }
                Some(Value::Str(s))
            }
            b'{' => {
                // `{n}` or `{n+}` closes this text segment; the literal
                // is the next segment.
                let text = self.text();
                let end = text[self.pos..]
                    .iter()
                    .position(|&b| b == b'}')
                    .map_or(text.len(), |e| self.pos + e + 1);
                if end == text.len() && matches!(self.segs.get(self.seg + 1), Some(Seg::Lit(_))) {
                    let Some(Seg::Lit(lit)) = self.segs.get(self.seg + 1) else {
                        unreachable!()
                    };
                    self.seg += 2;
                    self.pos = 0;
                    return Some(Value::Str(lit.clone()));
                }
                let start = self.pos;
                self.pos = end;
                Some(Value::Atom(
                    String::from_utf8_lossy(&text[start..end]).into_owned(),
                ))
            }
            _ => {
                let text = self.text();
                let start = self.pos;
                let mut bracket = 0usize;
                while let Some(c) = self.peek() {
                    match c {
                        b'[' => bracket += 1,
                        b']' => bracket = bracket.saturating_sub(1),
                        b' ' | b'(' | b')' if bracket == 0 => break,
                        _ => {}
                    }
                    self.pos += 1;
                }
                if self.pos == start {
                    self.pos += 1;
                }
                let atom = String::from_utf8_lossy(&text[start..self.pos]).into_owned();
                if atom.eq_ignore_ascii_case("NIL") {
                    Some(Value::Nil)
                } else {
                    Some(Value::Atom(atom))
                }
            }
        }
    }
}

/// The literal length announced at the end of a raw line: `{n}` or `{n+}`.
fn literal_len(raw: &[u8]) -> Option<usize> {
    let body = raw.strip_suffix(b"}")?;
    let open = body.iter().rposition(|&b| b == b'{')?;
    let digits = &body[open + 1..];
    let digits = digits.strip_suffix(b"+").unwrap_or(digits);
    if digits.is_empty() || digits.len() > 9 || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(digits).ok()?.parse().ok()
}

/// What a SELECT reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Selected {
    pub(crate) uidvalidity: u32,
    pub(crate) uidnext: Option<u32>,
    pub(crate) exists: u32,
}

/// One message of a FETCH response.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Fetched {
    pub(crate) uid: u32,
    pub(crate) size: Option<u32>,
    pub(crate) flags: Vec<String>,
    pub(crate) body: Option<Vec<u8>>,
}

/// An IMAP session over `S`.
pub(crate) struct Client<S> {
    stream: S,
    buf: Vec<u8>,
    start: usize,
    tag: u32,
    pub(crate) caps: Vec<String>,
    timeout: Duration,
    /// Bytes read for the current command.
    budget: usize,
    /// Messages in the selected folder: the last `* n EXISTS`, less the
    /// `* n EXPUNGE`s since. `None` before a SELECT.
    exists: Option<u32>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S> {
    pub(crate) fn new(stream: S) -> Self {
        Self {
            stream,
            buf: Vec::with_capacity(8192),
            start: 0,
            tag: 0,
            caps: Vec::new(),
            timeout: COMMAND_TIMEOUT,
            budget: 0,
            exists: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Whether the server advertised `cap`.
    pub(crate) fn has(&self, cap: &str) -> bool {
        self.caps.iter().any(|c| c.eq_ignore_ascii_case(cap))
    }

    async fn fill(&mut self) -> Result<()> {
        if self.start > 0 && self.start == self.buf.len() {
            self.buf.clear();
            self.start = 0;
        } else if self.start > 64 * 1024 {
            self.buf.drain(..self.start);
            self.start = 0;
        }
        let mut chunk = [0u8; 8192];
        let n = self.stream.read(&mut chunk).await?;
        if n == 0 {
            return Err(Error::Io(std::io::ErrorKind::UnexpectedEof));
        }
        self.budget += n;
        if self.budget > RESPONSE_MAX + LINE_MAX {
            return Err(Error::TooLarge);
        }
        self.buf.extend_from_slice(&chunk[..n]);
        Ok(())
    }

    /// One raw line without its CRLF.
    async fn raw_line(&mut self) -> Result<Vec<u8>> {
        let mut scanned = self.start;
        loop {
            if let Some(i) = self.buf[scanned..].iter().position(|&b| b == b'\n') {
                let end = scanned + i;
                let mut line = self.buf[self.start..end].to_vec();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                self.start = end + 1;
                if line.len() > LINE_MAX {
                    return Err(Error::TooLarge);
                }
                return Ok(line);
            }
            scanned = self.buf.len();
            if scanned - self.start > LINE_MAX {
                return Err(Error::TooLarge);
            }
            let consumed = self.start;
            self.fill().await?;
            // `fill` may have moved the buffer.
            scanned = scanned - consumed + self.start;
        }
    }

    async fn exact(&mut self, n: usize) -> Result<Vec<u8>> {
        if n > LITERAL_MAX {
            return Err(Error::TooLarge);
        }
        while self.buf.len() - self.start < n {
            self.fill().await?;
        }
        let out = self.buf[self.start..self.start + n].to_vec();
        self.start += n;
        Ok(out)
    }

    /// One logical response line, its literals read in.
    async fn line(&mut self) -> Result<Line> {
        let mut segs = Vec::new();
        loop {
            let raw = self.raw_line().await?;
            let lit = literal_len(&raw);
            segs.push(Seg::Text(raw));
            match lit {
                Some(n) => segs.push(Seg::Lit(self.exact(n).await?)),
                None => return Ok(Line(segs)),
            }
        }
    }

    /// Read the greeting. `true`: already authenticated (PREAUTH).
    pub(crate) async fn greeting(&mut self) -> Result<bool> {
        self.budget = 0;
        let timeout = self.timeout;
        let line = match tokio::time::timeout(timeout, self.line()).await {
            Ok(r) => r?,
            Err(_) => return Err(Error::Timeout),
        };
        let head = line.head();
        if head.starts_with(b"* OK") {
            Ok(false)
        } else if head.starts_with(b"* PREAUTH") {
            Ok(true)
        } else if head.starts_with(b"* BYE") {
            Err(Error::Bye)
        } else {
            Err(Error::Protocol("greeting"))
        }
    }

    fn next_tag(&mut self) -> String {
        self.tag = self.tag.wrapping_add(1);
        format!("a{:04}", self.tag)
    }

    /// Send a command made of `parts`; literals wait for the server's `+`.
    /// Returns the untagged lines.
    async fn run(&mut self, parts: &[Arg]) -> Result<Vec<Line>> {
        self.budget = 0;
        let tag = self.next_tag();
        let timeout = self.timeout;
        let work = async {
            let mut out = tag.clone().into_bytes();
            for part in parts {
                match part {
                    Arg::Raw(bytes) => {
                        out.push(b' ');
                        out.extend_from_slice(bytes);
                    }
                    Arg::Literal(bytes) => {
                        out.extend_from_slice(format!(" {{{}}}\r\n", bytes.len()).as_bytes());
                        self.stream.write_all(&out).await?;
                        self.stream.flush().await?;
                        out.clear();
                        let reply = self.line().await?;
                        if !reply.head().starts_with(b"+") {
                            return Err(status_error(reply.head(), &tag));
                        }
                        out.extend_from_slice(bytes);
                    }
                }
            }
            out.extend_from_slice(b"\r\n");
            self.stream.write_all(&out).await?;
            self.stream.flush().await?;
            let mut untagged = Vec::new();
            loop {
                let line = self.line().await?;
                let head = line.head();
                if head.starts_with(b"* ") {
                    if head.starts_with(b"* BYE") && !parts_are_logout(parts) {
                        return Err(Error::Bye);
                    }
                    untagged.push(line);
                    if untagged.len() > 10_000 {
                        return Err(Error::TooLarge);
                    }
                } else if head.starts_with(b"+") {
                    return Err(Error::Protocol("unexpected continuation"));
                } else if head.starts_with(tag.as_bytes()) && head.get(tag.len()) == Some(&b' ') {
                    let status = &head[tag.len() + 1..];
                    if starts_ci(status, b"OK") {
                        return Ok(untagged);
                    }
                    return Err(status_error(head, &tag));
                } else {
                    return Err(Error::Protocol("unknown response"));
                }
            }
        };
        let lines = match tokio::time::timeout(timeout, work).await {
            Ok(r) => r?,
            Err(_) => return Err(Error::Timeout),
        };
        self.track_exists(&lines);
        Ok(lines)
    }

    /// Keep [`Client::exists`] current from a command's untagged lines.
    fn track_exists(&mut self, lines: &[Line]) {
        for line in lines {
            let values = line.values();
            if let (Some(Value::Atom(n)), Some(Value::Atom(what))) = (values.get(1), values.get(2))
            {
                if what.eq_ignore_ascii_case("EXISTS") {
                    if let Ok(n) = n.parse() {
                        self.exists = Some(n);
                    }
                } else if what.eq_ignore_ascii_case("EXPUNGE") {
                    if let Some(e) = self.exists.as_mut() {
                        *e = e.saturating_sub(1);
                    }
                }
            }
        }
    }

    /// `CAPABILITY`, stored in `caps`.
    pub(crate) async fn capability(&mut self) -> Result<()> {
        let lines = self.run(&[raw(b"CAPABILITY")]).await?;
        let mut caps = Vec::new();
        for line in &lines {
            let values = line.values();
            if values
                .get(1)
                .and_then(Value::atom)
                .is_some_and(|a| a.eq_ignore_ascii_case("CAPABILITY"))
            {
                caps.extend(
                    values[2..]
                        .iter()
                        .filter_map(|v| v.atom().map(str::to_string)),
                );
            }
        }
        self.caps = caps;
        Ok(())
    }

    /// Log in: `LOGIN`, or `AUTHENTICATE PLAIN` when the server disables
    /// LOGIN. A refusal is [`Error::No`]; it is never retried here.
    pub(crate) async fn login(&mut self, user: &str, password: &str) -> Result<()> {
        if self.has("LOGINDISABLED") {
            if !self.has("AUTH=PLAIN") {
                return Err(Error::Protocol("no usable login mechanism"));
            }
            return self.authenticate_plain(user, password).await;
        }
        let user = astring(user.as_bytes());
        let pass = astring(password.as_bytes());
        self.run(&[raw(b"LOGIN"), user, pass]).await.map(|_| ())
    }

    async fn authenticate_plain(&mut self, user: &str, password: &str) -> Result<()> {
        use base64::Engine;
        let mut raw = Vec::with_capacity(user.len() + password.len() + 2);
        raw.push(0);
        raw.extend_from_slice(user.as_bytes());
        raw.push(0);
        raw.extend_from_slice(password.as_bytes());
        let encoded = base64::engine::general_purpose::STANDARD.encode(&raw);
        self.budget = 0;
        let tag = self.next_tag();
        let timeout = self.timeout;
        let work = async {
            self.stream
                .write_all(format!("{tag} AUTHENTICATE PLAIN\r\n").as_bytes())
                .await?;
            self.stream.flush().await?;
            let reply = self.line().await?;
            if !reply.head().starts_with(b"+") {
                return Err(status_error(reply.head(), &tag));
            }
            self.stream
                .write_all(format!("{encoded}\r\n").as_bytes())
                .await?;
            self.stream.flush().await?;
            loop {
                let line = self.line().await?;
                let head = line.head();
                if head.starts_with(b"* ") {
                    continue;
                }
                if head.starts_with(tag.as_bytes()) && head.get(tag.len()) == Some(&b' ') {
                    if starts_ci(&head[tag.len() + 1..], b"OK") {
                        return Ok(());
                    }
                    return Err(status_error(head, &tag));
                }
                return Err(Error::Protocol("unknown response"));
            }
        };
        match tokio::time::timeout(timeout, work).await {
            Ok(r) => r,
            Err(_) => Err(Error::Timeout),
        }
    }

    /// `SELECT folder`.
    pub(crate) async fn select(&mut self, folder: &str) -> Result<Selected> {
        let lines = self
            .run(&[raw(b"SELECT"), astring(folder.as_bytes())])
            .await?;
        let mut selected = Selected::default();
        let mut validity = false;
        for line in &lines {
            let values = line.values();
            match (values.get(1), values.get(2)) {
                (Some(Value::Atom(n)), Some(Value::Atom(what)))
                    if what.eq_ignore_ascii_case("EXISTS") =>
                {
                    selected.exists = n.parse().unwrap_or(0);
                }
                (Some(Value::Atom(ok)), Some(Value::Atom(code)))
                    if ok.eq_ignore_ascii_case("OK") =>
                {
                    let code = code.trim_start_matches('[').trim_end_matches(']');
                    let mut words = code.split_whitespace();
                    match (words.next(), words.next().and_then(|n| n.parse().ok())) {
                        (Some(w), Some(n)) if w.eq_ignore_ascii_case("UIDVALIDITY") => {
                            selected.uidvalidity = n;
                            validity = true;
                        }
                        (Some(w), Some(n)) if w.eq_ignore_ascii_case("UIDNEXT") => {
                            selected.uidnext = Some(n);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        if !validity {
            return Err(Error::Protocol("no UIDVALIDITY"));
        }
        Ok(selected)
    }

    pub(crate) async fn noop(&mut self) -> Result<()> {
        self.run(&[raw(b"NOOP")]).await.map(|_| ())
    }

    /// `UID SEARCH <criteria>`: the UIDs, ascending.
    pub(crate) async fn uid_search(&mut self, criteria: &str) -> Result<Vec<u32>> {
        let lines = self
            .run(&[raw(b"UID SEARCH"), raw(criteria.as_bytes())])
            .await?;
        let mut uids = Vec::new();
        for line in &lines {
            let values = line.values();
            if values
                .get(1)
                .and_then(Value::atom)
                .is_some_and(|a| a.eq_ignore_ascii_case("SEARCH"))
            {
                uids.extend(values[2..].iter().filter_map(Value::number));
            }
        }
        uids.sort_unstable();
        uids.dedup();
        Ok(uids)
    }

    /// The UID of the last message: `FETCH <n> (UID)`, `n` the folder's
    /// message count from EXISTS. iCloud answers `FETCH * …` (and any bare
    /// `*`) with `BAD Parse Error`, so the number is spelled out; `*` only
    /// before any EXISTS was seen. An empty folder has none.
    pub(crate) async fn last_uid(&mut self) -> Result<Option<u32>> {
        let n = match self.exists {
            Some(0) => return Ok(None),
            Some(n) => n.to_string(),
            None => "*".to_string(),
        };
        let lines = self
            .run(&[raw(b"FETCH"), raw(n.as_bytes()), raw(b"(UID)")])
            .await?;
        Ok(fetched(&lines).into_iter().map(|f| f.uid).max())
    }

    /// `UID FETCH <set> <items>`.
    pub(crate) async fn uid_fetch(&mut self, set: &str, items: &str) -> Result<Vec<Fetched>> {
        let lines = self
            .run(&[
                raw(b"UID FETCH"),
                raw(set.as_bytes()),
                raw(items.as_bytes()),
            ])
            .await?;
        Ok(fetched(&lines))
    }

    /// Add `flag` (`\Seen`, `\Deleted`) without asking for the result.
    pub(crate) async fn uid_add_flag(&mut self, uid: u32, flag: &str) -> Result<()> {
        let cmd = format!("UID STORE {uid} +FLAGS.SILENT ({flag})");
        self.run(&[raw(cmd.as_bytes())]).await.map(|_| ())
    }

    pub(crate) async fn uid_move(&mut self, uid: u32, folder: &str) -> Result<()> {
        let cmd = format!("UID MOVE {uid}");
        self.run(&[raw(cmd.as_bytes()), astring(folder.as_bytes())])
            .await
            .map(|_| ())
    }

    pub(crate) async fn uid_copy(&mut self, uid: u32, folder: &str) -> Result<()> {
        let cmd = format!("UID COPY {uid}");
        self.run(&[raw(cmd.as_bytes()), astring(folder.as_bytes())])
            .await
            .map(|_| ())
    }

    pub(crate) async fn uid_expunge(&mut self, uid: u32) -> Result<()> {
        let cmd = format!("UID EXPUNGE {uid}");
        self.run(&[raw(cmd.as_bytes())]).await.map(|_| ())
    }

    pub(crate) async fn logout(&mut self) -> Result<()> {
        self.run(&[raw(b"LOGOUT")]).await.map(|_| ())
    }
}

fn parts_are_logout(parts: &[Arg]) -> bool {
    matches!(parts.first(), Some(Arg::Raw(r)) if r == b"LOGOUT")
}

fn starts_ci(s: &[u8], prefix: &[u8]) -> bool {
    s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

fn status_error(head: &[u8], tag: &str) -> Error {
    let status = head
        .strip_prefix(tag.as_bytes())
        .map(|s| s.trim_ascii_start())
        .unwrap_or(head);
    if starts_ci(status, b"NO") {
        Error::No
    } else if starts_ci(status, b"BAD") {
        Error::Bad
    } else if starts_ci(status, b"* BYE") {
        Error::Bye
    } else {
        Error::Protocol("unexpected status")
    }
}

/// A command argument.
pub(crate) enum Arg {
    Raw(Vec<u8>),
    /// Sent as a synchronizing literal.
    Literal(Vec<u8>),
}

fn raw(bytes: &[u8]) -> Arg {
    Arg::Raw(bytes.to_vec())
}

/// An `astring` argument: a quoted string when it is printable ASCII, else a
/// literal (8-bit passwords).
fn astring(value: &[u8]) -> Arg {
    if value.iter().all(|b| (0x20..0x7f).contains(b)) {
        let mut q = Vec::with_capacity(value.len() + 2);
        q.push(b'"');
        for &b in value {
            if b == b'"' || b == b'\\' {
                q.push(b'\\');
            }
            q.push(b);
        }
        q.push(b'"');
        Arg::Raw(q)
    } else {
        Arg::Literal(value.to_vec())
    }
}

/// The FETCH responses among untagged lines.
fn fetched(lines: &[Line]) -> Vec<Fetched> {
    let mut out = Vec::new();
    for line in lines {
        let values = line.values();
        let is_fetch = values
            .get(2)
            .and_then(Value::atom)
            .is_some_and(|a| a.eq_ignore_ascii_case("FETCH"));
        let Some(Value::List(items)) = values.get(3).filter(|_| is_fetch) else {
            continue;
        };
        let mut f = Fetched::default();
        let mut uid = None;
        let mut i = 0;
        while i + 1 < items.len() {
            let key = items[i].atom().unwrap_or("").to_ascii_uppercase();
            let value = &items[i + 1];
            if key == "UID" {
                uid = value.number();
            } else if key == "RFC822.SIZE" {
                f.size = value.number();
            } else if key == "FLAGS" {
                if let Value::List(flags) = value {
                    f.flags = flags
                        .iter()
                        .filter_map(|v| v.atom().map(str::to_string))
                        .collect();
                }
            } else if key.starts_with("BODY[") {
                if let Value::Str(bytes) = value {
                    f.body = Some(bytes.clone());
                }
            }
            i += 2;
        }
        if let Some(uid) = uid {
            f.uid = uid;
            out.push(f);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str, lits: &[&[u8]]) -> Line {
        let mut segs = Vec::new();
        let parts: Vec<&str> = text.split("\u{0}").collect();
        for (i, p) in parts.iter().enumerate() {
            segs.push(Seg::Text(p.as_bytes().to_vec()));
            if let Some(l) = lits.get(i) {
                segs.push(Seg::Lit(l.to_vec()));
            }
        }
        Line(segs)
    }

    #[test]
    fn literal_lengths() {
        assert_eq!(literal_len(b"* 1 FETCH (BODY[] {12}"), Some(12));
        assert_eq!(literal_len(b"a LOGIN {5+}"), Some(5));
        assert_eq!(literal_len(b"* OK done"), None);
        assert_eq!(literal_len(b"* OK {abc}"), None);
        assert_eq!(literal_len(b"* OK {9999999999}"), None);
    }

    #[test]
    fn fetch_responses_parse_with_literals_quoted_strings_and_sections() {
        let l = line(
            "* 3 FETCH (UID 41 RFC822.SIZE 1200 FLAGS (\\Seen $Junk) BODY[]<0> {5}\u{0})",
            &[b"hi\r\nx"],
        );
        let f = fetched(&[l]);
        assert_eq!(
            f,
            vec![Fetched {
                uid: 41,
                size: Some(1200),
                flags: vec!["\\Seen".into(), "$Junk".into()],
                body: Some(b"hi\r\nx".to_vec()),
            }]
        );
        let l = line(
            "* 4 FETCH (BODY[HEADER.FIELDS (FROM TO)] \"a\\\"b\" UID 7)",
            &[],
        );
        let f = fetched(&[l]);
        assert_eq!(f[0].uid, 7);
        assert_eq!(f[0].body.as_deref(), Some(&b"a\"b"[..]));
        let l = line("* 5 FETCH (UID 8 BODY[] NIL)", &[]);
        assert_eq!(fetched(&[l])[0].body, None);
    }

    #[test]
    fn quoting_escapes_and_falls_back_to_a_literal() {
        match astring(b"p\"a\\ss") {
            Arg::Raw(r) => assert_eq!(r, b"\"p\\\"a\\\\ss\""),
            Arg::Literal(_) => panic!("quoted expected"),
        }
        assert!(matches!(astring("пароль".as_bytes()), Arg::Literal(_)));
    }
}
