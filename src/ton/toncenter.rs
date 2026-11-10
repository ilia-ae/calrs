//! toncenter API v3 (TON Index): the three read-only GETs the watcher makes,
//! our own lenient parsers for their answers, and the [`TonIndexer`] trait the
//! watcher talks to (HTTP in production, an in-memory fake in tests).
//!
//! Numbers arrive both as strings and as integers (live answers have `lt`,
//! `value`, `gen_utime` as strings and `now` as an integer), so every numeric
//! field goes through [`value_u64`]/[`value_u128`]. Unknown fields are
//! ignored; a missing required field is an error for that item only.
//!
//! The base URL may carry a provider key in its path (Chainstack), so it is
//! treated as a secret: errors carry a class, never a URL, and the key of
//! toncenter itself goes only in the `X-Api-Key` header.

use std::collections::HashMap;

use async_trait::async_trait;
use base64::Engine;

use super::Network;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MasterchainInfo {
    pub seqno: u64,
    /// How far the indexer has got (Unix seconds).
    pub gen_utime: u64,
    pub global_id: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BookEntry {
    pub user_friendly: Option<String>,
    pub interfaces: Vec<String>,
}

impl BookEntry {
    /// toncenter recognised the account as a wallet contract.
    pub fn is_wallet(&self) -> bool {
        self.interfaces.iter().any(|i| i.starts_with("wallet"))
    }
}

/// One `GET /transactions` answer: the raw transactions (parsed one by one
/// by `decide::parse_tx`, so one odd item does not sink the page) and the
/// address book keyed by upper-case raw address.
#[derive(Clone, Debug, Default)]
pub struct TxPage {
    pub transactions: Vec<serde_json::Value>,
    pub address_book: HashMap<String, BookEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexerError {
    /// HTTP 429.
    RateLimited,
    /// HTTP 401 (no key) or 403 (a key of the other network).
    Auth(u16),
    /// HTTP 5xx.
    Server(u16),
    /// Any other non-2xx status.
    Http(u16),
    /// Connection, timeout, body: a class name, never a URL.
    Network(String),
    Parse(&'static str),
}

impl IndexerError {
    /// A short class stored in `ton_watch.last_error` and logged.
    pub fn class(&self) -> String {
        match self {
            Self::RateLimited => "rate_limited".into(),
            Self::Auth(code) | Self::Server(code) | Self::Http(code) => format!("http_{code}"),
            Self::Network(class) => format!("network_{class}"),
            Self::Parse(what) => format!("parse_{what}"),
        }
    }
}

impl std::fmt::Display for IndexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.class())
    }
}

/// The `global_id` of each network's masterchain (checked live 2026-11-09).
pub fn expected_global_id(network: Network) -> i64 {
    match network {
        Network::Mainnet => -239,
        Network::Testnet => -3,
    }
}

/// An unsigned integer given as a JSON number or a decimal string.
pub fn value_u64(v: &serde_json::Value) -> Option<u64> {
    match v {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) => {
            s.parse().ok()
        }
        _ => None,
    }
}

/// [`value_u64`] for amounts, which may exceed `u64` in principle.
pub fn value_u128(v: &serde_json::Value) -> Option<u128> {
    match v {
        serde_json::Value::Number(n) => n.as_u64().map(u128::from),
        serde_json::Value::String(s)
            if !s.is_empty() && s.len() <= 38 && s.bytes().all(|b| b.is_ascii_digit()) =>
        {
            s.parse().ok()
        }
        _ => None,
    }
}

/// A signed integer given as a JSON number or a decimal string.
pub fn value_i64(v: &serde_json::Value) -> Option<i64> {
    match v {
        serde_json::Value::Number(n) => n.as_i64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// A 32-byte hash in base64 (either alphabet, padded or not) or hex, as
/// lower-case hex.
pub fn hash_to_hex(s: &str) -> Option<String> {
    let s = s.trim();
    if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Some(s.to_ascii_lowercase());
    }
    let unpadded = s.trim_end_matches('=');
    let engine = if unpadded.contains('-') || unpadded.contains('_') {
        &base64::engine::general_purpose::URL_SAFE_NO_PAD
    } else {
        &base64::engine::general_purpose::STANDARD_NO_PAD
    };
    let bytes = engine.decode(unpadded).ok()?;
    (bytes.len() == 32).then(|| hex::encode(bytes))
}

fn parse_json(body: &[u8]) -> Result<serde_json::Value, IndexerError> {
    serde_json::from_slice(body).map_err(|_| IndexerError::Parse("json"))
}

/// `GET /masterchainInfo` → `last.{seqno, gen_utime, global_id}`.
pub fn parse_masterchain_info(body: &[u8]) -> Result<MasterchainInfo, IndexerError> {
    let v = parse_json(body)?;
    let last = v.get("last").ok_or(IndexerError::Parse("last"))?;
    Ok(MasterchainInfo {
        seqno: last
            .get("seqno")
            .and_then(value_u64)
            .ok_or(IndexerError::Parse("seqno"))?,
        gen_utime: last
            .get("gen_utime")
            .and_then(value_u64)
            .ok_or(IndexerError::Parse("gen_utime"))?,
        global_id: last
            .get("global_id")
            .and_then(value_i64)
            .ok_or(IndexerError::Parse("global_id"))?,
    })
}

/// `GET /transactions` → the page. `transactions` must be a list.
pub fn parse_tx_page(body: &[u8]) -> Result<TxPage, IndexerError> {
    let v = parse_json(body)?;
    let transactions = v
        .get("transactions")
        .and_then(serde_json::Value::as_array)
        .ok_or(IndexerError::Parse("transactions"))?
        .clone();
    let mut address_book = HashMap::new();
    if let Some(book) = v.get("address_book").and_then(serde_json::Value::as_object) {
        for (raw, entry) in book {
            let Some(key) = super::address::normalize_raw(raw) else {
                continue;
            };
            let user_friendly = entry
                .get("user_friendly")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
            let interfaces = entry
                .get("interfaces")
                .and_then(serde_json::Value::as_array)
                .map(|list| {
                    list.iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            address_book.insert(
                key,
                BookEntry {
                    user_friendly,
                    interfaces,
                },
            );
        }
    }
    Ok(TxPage {
        transactions,
        address_book,
    })
}

/// `GET /accountStates` → the status of `raw` (`active`, `uninit`,
/// `frozen`, …), or `None` when the indexer knows no such account.
pub fn parse_account_status(body: &[u8], raw: &str) -> Result<Option<String>, IndexerError> {
    let v = parse_json(body)?;
    let accounts = v
        .get("accounts")
        .and_then(serde_json::Value::as_array)
        .ok_or(IndexerError::Parse("accounts"))?;
    for account in accounts {
        let addr = account
            .get("address")
            .and_then(serde_json::Value::as_str)
            .and_then(super::address::normalize_raw);
        if addr.as_deref() == Some(raw) {
            return Ok(account
                .get("status")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string));
        }
    }
    Ok(None)
}

/// The newest transaction of an account as (lt, hash hex).
pub fn latest_of(page: &TxPage) -> Option<(u64, String)> {
    page.transactions
        .iter()
        .filter_map(|tx| {
            let lt = tx.get("lt").and_then(value_u64)?;
            let hash = tx
                .get("hash")
                .and_then(serde_json::Value::as_str)
                .and_then(hash_to_hex)?;
            Some((lt, hash))
        })
        .max_by_key(|(lt, _)| *lt)
}

/// What the watcher needs from an indexer.
#[async_trait]
pub trait TonIndexer: Send + Sync {
    async fn masterchain_info(&self) -> Result<MasterchainInfo, IndexerError>;
    /// Transactions of `account_raw` with `lt >= start_lt`, ascending.
    async fn transactions_from(
        &self,
        account_raw: &str,
        start_lt: u64,
        limit: u32,
    ) -> Result<TxPage, IndexerError>;
    /// The newest transaction (for the first cursor of a new address).
    async fn latest_tx(&self, account_raw: &str) -> Result<Option<(u64, String)>, IndexerError>;
    async fn account_status(&self, account_raw: &str) -> Result<Option<String>, IndexerError>;
}

/// The error class of a reqwest error, without its URL.
pub fn reqwest_class(e: &reqwest::Error) -> String {
    if e.is_timeout() {
        "timeout".into()
    } else if e.is_connect() {
        "connect".into()
    } else if e.is_body() || e.is_decode() {
        "body".into()
    } else if e.is_redirect() {
        "redirect".into()
    } else {
        "request".into()
    }
}

/// The largest answer read (a page of 100 transactions is about 300 KB).
const MAX_BODY: usize = 8 * 1024 * 1024;
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
/// The longest `Retry-After` honoured inside one call; beyond it the call
/// fails as rate limited and the watcher backs off.
const MAX_RETRY_AFTER: std::time::Duration = std::time::Duration::from_secs(5);

/// toncenter (or a provider serving the same paths) over HTTPS.
pub struct HttpIndexer {
    client: reqwest::Client,
    base: String,
    key: Option<String>,
    /// Every request waits for its turn: at most one per `interval` overall.
    next_slot: tokio::sync::Mutex<tokio::time::Instant>,
    interval: std::time::Duration,
}

impl HttpIndexer {
    /// `base` without a trailing slash, e.g. `https://toncenter.com/api/v3`.
    pub fn new(base: &str, key: Option<String>, interval: std::time::Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("calrs/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default();
        Self {
            client,
            base: base.trim_end_matches('/').to_string(),
            key,
            next_slot: tokio::sync::Mutex::new(tokio::time::Instant::now()),
            interval,
        }
    }

    async fn wait_turn(&self) {
        let mut next = self.next_slot.lock().await;
        let now = tokio::time::Instant::now();
        if *next > now {
            tokio::time::sleep_until(*next).await;
        }
        *next = tokio::time::Instant::now().max(*next) + self.interval;
    }

    async fn get(&self, path: &str, query: &[(&str, String)]) -> Result<Vec<u8>, IndexerError> {
        let mut retried = false;
        loop {
            self.wait_turn().await;
            let mut req = self.client.get(format!("{}{path}", self.base)).query(query);
            if let Some(key) = &self.key {
                req = req.header("X-Api-Key", key);
            }
            let resp = req
                .send()
                .await
                .map_err(|e| IndexerError::Network(reqwest_class(&e)))?;
            let status = resp.status().as_u16();
            if status == 429 {
                let wait = resp
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.trim().parse::<u64>().ok())
                    .map(std::time::Duration::from_secs)
                    .unwrap_or(std::time::Duration::from_secs(2));
                if retried || wait > MAX_RETRY_AFTER {
                    return Err(IndexerError::RateLimited);
                }
                retried = true;
                tokio::time::sleep(wait).await;
                continue;
            }
            match status {
                200..=299 => {}
                401 | 403 => return Err(IndexerError::Auth(status)),
                500..=599 => return Err(IndexerError::Server(status)),
                _ => return Err(IndexerError::Http(status)),
            }
            if resp.content_length().is_some_and(|n| n > MAX_BODY as u64) {
                return Err(IndexerError::Parse("too_large"));
            }
            let body = resp
                .bytes()
                .await
                .map_err(|e| IndexerError::Network(reqwest_class(&e)))?;
            if body.len() > MAX_BODY {
                return Err(IndexerError::Parse("too_large"));
            }
            return Ok(body.to_vec());
        }
    }
}

#[async_trait]
impl TonIndexer for HttpIndexer {
    async fn masterchain_info(&self) -> Result<MasterchainInfo, IndexerError> {
        parse_masterchain_info(&self.get("/masterchainInfo", &[]).await?)
    }

    async fn transactions_from(
        &self,
        account_raw: &str,
        start_lt: u64,
        limit: u32,
    ) -> Result<TxPage, IndexerError> {
        let body = self
            .get(
                "/transactions",
                &[
                    ("account", account_raw.to_string()),
                    ("start_lt", start_lt.to_string()),
                    ("sort", "asc".to_string()),
                    ("limit", limit.to_string()),
                ],
            )
            .await?;
        parse_tx_page(&body)
    }

    async fn latest_tx(&self, account_raw: &str) -> Result<Option<(u64, String)>, IndexerError> {
        let body = self
            .get(
                "/transactions",
                &[
                    ("account", account_raw.to_string()),
                    ("sort", "desc".to_string()),
                    ("limit", "1".to_string()),
                ],
            )
            .await?;
        Ok(latest_of(&parse_tx_page(&body)?))
    }

    async fn account_status(&self, account_raw: &str) -> Result<Option<String>, IndexerError> {
        let body = self
            .get(
                "/accountStates",
                &[
                    ("address", account_raw.to_string()),
                    ("include_boc", "false".to_string()),
                ],
            )
            .await?;
        parse_account_status(&body, account_raw)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn fixture(name: &str) -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/toncenter")
            .join(name);
        std::fs::read(path).unwrap()
    }

    const TF: &str = "0:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8";
    const TN_BUSY: &str = "0:CCA3DC4922DEAA32160983996E8482B5F179112552943DE3A0AC0D0E449A1381";

    #[test]
    fn masterchain_info_of_both_networks() {
        let m = parse_masterchain_info(&fixture("mainnet-masterchainInfo.json")).unwrap();
        assert_eq!(m.global_id, expected_global_id(Network::Mainnet));
        assert!(m.gen_utime > 1_794_000_000 && m.seqno > 0);
        let t = parse_masterchain_info(&fixture("testnet-masterchainInfo.json")).unwrap();
        assert_eq!(t.global_id, expected_global_id(Network::Testnet));
        assert_ne!(m.global_id, t.global_id);
        // gen_utime as a number works too; a foreign global_id parses (the
        // watcher refuses it).
        let other = parse_masterchain_info(
            br#"{"last":{"seqno":5,"gen_utime":1794244665,"global_id":"7"}}"#,
        )
        .unwrap();
        assert_eq!(other.global_id, 7);
        assert_eq!(other.gen_utime, 1_794_244_665);
        for bad in [
            &b"{}"[..],
            br#"{"last":{"seqno":5,"global_id":-3}}"#,
            br#"{"last":{"seqno":5,"gen_utime":"x","global_id":-3}}"#,
            br#"{"last":"#,
            b"",
        ] {
            assert!(parse_masterchain_info(bad).is_err());
        }
    }

    #[test]
    fn account_states_active_and_unknown() {
        assert_eq!(
            parse_account_status(&fixture("mainnet-accountStates-active.json"), TF).unwrap(),
            Some("active".to_string())
        );
        // A never-used testnet account: the indexer answers an empty list.
        let fresh = "0:5C1A3E0F9B7D24C68E01F3A5B79D2C4E6F8A0B1C3D5E7F9A2B4C6D8E0F1A3B5C";
        assert_eq!(
            parse_account_status(&fixture("testnet-accountStates-uninit.json"), fresh).unwrap(),
            None
        );
        assert_eq!(
            parse_account_status(
                br#"{"accounts":[{"address":"0:5c1a3e0f9b7d24c68e01f3a5b79d2c4e6f8a0b1c3d5e7f9a2b4c6d8e0f1a3b5c","status":"uninit"}]}"#,
                fresh
            )
            .unwrap(),
            Some("uninit".to_string())
        );
        assert!(parse_account_status(b"{\"accounts\":5}", fresh).is_err());
    }

    #[test]
    fn transaction_pages_and_the_address_book() {
        let page = parse_tx_page(&fixture("mainnet-transactions-tf.json")).unwrap();
        assert_eq!(page.transactions.len(), 3);
        let sender = "0:077F69B02858C4A6BC33E5F052E64E5BAD13AB95DB61442CFE0958B97959BDE4";
        assert!(
            page.address_book.contains_key(sender),
            "{:?}",
            page.address_book.keys()
        );
        let p1 = parse_tx_page(&fixture("testnet-transactions-page1.json")).unwrap();
        let p2 = parse_tx_page(&fixture("testnet-transactions-page2.json")).unwrap();
        assert_eq!(p1.transactions.len(), 100);
        assert_eq!(p2.transactions.len(), 100);
        let lts = |p: &TxPage| -> Vec<u64> {
            p.transactions
                .iter()
                .map(|t| value_u64(&t["lt"]).unwrap())
                .collect()
        };
        let (l1, l2) = (lts(&p1), lts(&p2));
        assert!(l1.windows(2).all(|w| w[0] < w[1]));
        assert!(l2.windows(2).all(|w| w[0] < w[1]));
        // The boundary: page 2 starts after page 1 ends.
        assert!(l1.last().unwrap() < l2.first().unwrap());
        for t in &p1.transactions {
            assert_eq!(t["account"].as_str(), Some(TN_BUSY));
        }
        assert!(parse_tx_page(b"{\"transactions\":{}}").is_err());
        assert!(parse_tx_page(b"{\"address_book\":{}}").is_err());
        let latest =
            latest_of(&parse_tx_page(&fixture("mainnet-latest-tf.json")).unwrap()).unwrap();
        assert!(latest.0 >= 115_518_197_000_030);
        assert_eq!(latest.1.len(), 64);
    }

    #[test]
    fn hashes_and_numbers() {
        assert_eq!(
            hash_to_hex("MZ6ED1IMolkR1b7Bc/YCDeYVSKV+OzrZcv3ck2S9D7g=").as_deref(),
            hash_to_hex("MZ6ED1IMolkR1b7Bc_YCDeYVSKV-OzrZcv3ck2S9D7g").as_deref()
        );
        let h = hash_to_hex("MZ6ED1IMolkR1b7Bc/YCDeYVSKV+OzrZcv3ck2S9D7g=").unwrap();
        assert_eq!(h.len(), 64);
        assert_eq!(hash_to_hex(&h.to_uppercase()).as_deref(), Some(h.as_str()));
        assert_eq!(hash_to_hex("AAAA"), None);
        assert_eq!(
            value_u64(&serde_json::json!("115518197000030")),
            Some(115_518_197_000_030)
        );
        assert_eq!(
            value_u64(&serde_json::json!(1794222938)),
            Some(1_794_222_938)
        );
        assert_eq!(value_u64(&serde_json::json!("-1")), None);
        assert_eq!(value_u64(&serde_json::json!(1.5)), None);
        assert_eq!(value_u64(&serde_json::json!(null)), None);
        assert_eq!(value_u128(&serde_json::json!("2433333")), Some(2_433_333));
    }

    /// A one-shot HTTP server answering each connection with the next canned
    /// response; records the request heads it saw.
    async fn canned_server(
        answers: Vec<(u16, &'static str, Vec<(&'static str, &'static str)>)>,
    ) -> (String, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = seen.clone();
        tokio::spawn(async move {
            for (status, body, headers) in answers {
                let Ok((mut sock, _)) = listener.accept().await else {
                    return;
                };
                let mut buf = vec![0u8; 8192];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                log.lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&buf[..n]).to_string());
                let mut head = format!(
                    "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n",
                    body.len()
                );
                for (k, v) in headers {
                    head.push_str(&format!("{k}: {v}\r\n"));
                }
                head.push_str("\r\n");
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(body.as_bytes()).await;
                let _ = sock.shutdown().await;
            }
        });
        (
            format!("http://127.0.0.1:{port}/secret-path-key/api/v3"),
            seen,
        )
    }

    #[tokio::test]
    async fn http_status_classes_and_no_url_in_errors() {
        let body_429 = std::str::from_utf8(&fixture("error-429.json"))
            .unwrap()
            .to_string()
            .leak();
        let (base, _) = canned_server(vec![
            (401, "{}", vec![]),
            (403, "{}", vec![]),
            (500, "{}", vec![]),
            (418, "{}", vec![]),
            (429, body_429, vec![("retry-after", "60")]),
        ])
        .await;
        let idx = HttpIndexer::new(&base, None, std::time::Duration::from_millis(1));
        let expect = [
            IndexerError::Auth(401),
            IndexerError::Auth(403),
            IndexerError::Server(500),
            IndexerError::Http(418),
            IndexerError::RateLimited,
        ];
        for want in expect {
            let got = idx.masterchain_info().await.unwrap_err();
            assert_eq!(got, want);
            let text = format!("{got} {got:?}");
            assert!(!text.contains("secret-path-key"), "{text}");
            assert!(!text.contains("127.0.0.1"), "{text}");
        }
        // Nobody listening: a network class, still without the URL.
        let dead = HttpIndexer::new(
            "http://127.0.0.1:9/secret-path-key",
            Some("k".into()),
            std::time::Duration::from_millis(1),
        );
        let err = dead.masterchain_info().await.unwrap_err();
        assert!(matches!(err, IndexerError::Network(_)), "{err:?}");
        assert!(!err.class().contains("secret"), "{}", err.class());
    }

    #[tokio::test]
    async fn a_short_429_is_retried_once_and_the_key_goes_in_a_header() {
        let info = std::str::from_utf8(&fixture("testnet-masterchainInfo.json"))
            .unwrap()
            .to_string()
            .leak();
        let (base, seen) = canned_server(vec![
            (429, "{\"ok\":false}", vec![("retry-after", "0")]),
            (200, info, vec![]),
        ])
        .await;
        let idx = HttpIndexer::new(
            &base,
            Some("the-api-key".into()),
            std::time::Duration::from_millis(1),
        );
        let m = idx.masterchain_info().await.unwrap();
        assert_eq!(m.global_id, -3);
        let heads = seen.lock().unwrap().clone();
        assert_eq!(heads.len(), 2);
        for h in heads {
            assert!(
                h.starts_with("GET /secret-path-key/api/v3/masterchainInfo"),
                "{h}"
            );
            assert!(
                h.to_ascii_lowercase().contains("x-api-key: the-api-key"),
                "{h}"
            );
            assert!(!h.contains("api_key="), "{h}");
        }
    }

    #[tokio::test]
    async fn queries_use_the_raw_account_and_ascending_order() {
        let page = std::str::from_utf8(&fixture("mainnet-transactions-tf.json"))
            .unwrap()
            .to_string()
            .leak();
        let (base, seen) = canned_server(vec![(200, page, vec![])]).await;
        let idx = HttpIndexer::new(&base, None, std::time::Duration::from_millis(1));
        let p = idx.transactions_from(TF, 42, 100).await.unwrap();
        assert_eq!(p.transactions.len(), 3);
        let head = seen.lock().unwrap()[0].clone();
        let line = head.lines().next().unwrap().to_string();
        assert!(line.contains("account=0%3A83DFD552"), "{line}");
        assert!(line.contains("start_lt=42"), "{line}");
        assert!(line.contains("sort=asc"), "{line}");
        assert!(line.contains("limit=100"), "{line}");
        assert!(!head.to_ascii_lowercase().contains("x-api-key"), "{head}");
    }

    #[tokio::test]
    async fn requests_are_spaced_by_the_bucket() {
        let info = std::str::from_utf8(&fixture("mainnet-masterchainInfo.json"))
            .unwrap()
            .to_string()
            .leak();
        let (base, _) = canned_server(vec![(200, info, vec![]), (200, info, vec![])]).await;
        let idx = HttpIndexer::new(&base, None, std::time::Duration::from_millis(300));
        let t = std::time::Instant::now();
        idx.masterchain_info().await.unwrap();
        idx.masterchain_info().await.unwrap();
        assert!(t.elapsed() >= std::time::Duration::from_millis(300));
    }
}
