//! The TON/USD rate an invoice is priced at. Two public sources (CoinGecko,
//! Binance `GRAMUSDT`), each with its own timestamp; a quote older than the
//! freshness limit counts as missing. Both fresh and within the spread → the
//! mean. Only one fresh → accepted only near the last two-source rate. The
//! result must sit inside the owner's corridor. Anything else refuses the
//! booking: a missing or suspicious rate never prices a meeting.
//!
//! Decimal strings become micro-USD without floating point.

use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceKind {
    CoinGecko,
    Binance,
}

impl SourceKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::CoinGecko => "coingecko",
            Self::Binance => "binance",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "coingecko" => Some(Self::CoinGecko),
            "binance" => Some(Self::Binance),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RateQuote {
    pub source: SourceKind,
    pub usd_micro: u64,
    pub at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RateError {
    Http(String),
    Parse(&'static str),
}

impl std::fmt::Display for RateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(class) => write!(f, "request failed ({class})"),
            Self::Parse(what) => write!(f, "unexpected answer: {what}"),
        }
    }
}

/// The owner's rules (from the environment).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RatePolicy {
    pub min_usd_micro: u64,
    pub max_usd_micro: u64,
    /// Largest allowed spread between two sources, in basis points.
    pub max_spread_bp: u64,
    pub max_age: Duration,
    /// Single-source rule: largest move from the last two-source rate.
    pub max_jump_bp: u64,
}

/// The rate written into an invoice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedRate {
    pub usd_micro: u64,
    /// `coingecko=1.038000;binance=1.038000`
    pub sources: String,
    /// The oldest quote used.
    pub at: DateTime<Utc>,
    /// Both sources agreed (such a rate may anchor the single-source rule).
    pub confirmed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RateRefusal {
    NoQuote,
    Divergent { a: u64, b: u64 },
    SingleUnconfirmed,
    OutOfCorridor(u64),
}

impl std::fmt::Display for RateRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoQuote => f.write_str("no fresh rate from any source"),
            Self::Divergent { a, b } => write!(
                f,
                "sources disagree ({} vs {})",
                micro_to_string(*a),
                micro_to_string(*b)
            ),
            Self::SingleUnconfirmed => f.write_str(
                "only one source answered and there is no recent two-source rate near it",
            ),
            Self::OutOfCorridor(r) => {
                write!(
                    f,
                    "rate {} is outside the owner's corridor",
                    micro_to_string(*r)
                )
            }
        }
    }
}

/// Micro-USD as a decimal string with six places.
pub fn micro_to_string(micro: u64) -> String {
    format!("{}.{:06}", micro / 1_000_000, micro % 1_000_000)
}

/// Parse a non-negative decimal ("1.03800000", "1.038", "2") into millionths,
/// truncating past six places. No sign, no exponent. `None` for zero.
pub fn parse_decimal_micro(s: &str) -> Option<u64> {
    let s = s.trim();
    let (whole, frac) = s.split_once('.').unwrap_or((s, ""));
    if whole.is_empty()
        || whole.len() > 9
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let mut frac6: String = frac.chars().take(6).collect();
    while frac6.len() < 6 {
        frac6.push('0');
    }
    let micro = whole
        .parse::<u64>()
        .ok()?
        .checked_mul(1_000_000)?
        .checked_add(frac6.parse::<u64>().ok()?)?;
    (micro > 0).then_some(micro)
}

/// `{"the-open-network":{"usd":1.038,"last_updated_at":1794244583}}`
pub fn parse_coingecko(body: &[u8], id: &str) -> Result<RateQuote, RateError> {
    let v: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| RateError::Parse("not JSON"))?;
    let entry = v.get(id).ok_or(RateError::Parse("coin id missing"))?;
    let usd = match entry.get("usd") {
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return Err(RateError::Parse("usd missing")),
    };
    let usd_micro = parse_decimal_micro(&usd).ok_or(RateError::Parse("usd not a decimal"))?;
    let at = entry
        .get("last_updated_at")
        .and_then(serde_json::Value::as_i64)
        .and_then(|s| Utc.timestamp_opt(s, 0).single())
        .ok_or(RateError::Parse("last_updated_at missing"))?;
    Ok(RateQuote {
        source: SourceKind::CoinGecko,
        usd_micro,
        at,
    })
}

/// Binance `GET /api/v3/ticker/24hr?symbol=…`: `lastPrice` (string) and
/// `closeTime` (milliseconds). A frozen pair keeps an old `closeTime`.
pub fn parse_binance(body: &[u8], symbol: &str) -> Result<RateQuote, RateError> {
    let v: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| RateError::Parse("not JSON"))?;
    if v.get("symbol").and_then(serde_json::Value::as_str) != Some(symbol) {
        return Err(RateError::Parse("symbol mismatch"));
    }
    let price = v
        .get("lastPrice")
        .and_then(serde_json::Value::as_str)
        .ok_or(RateError::Parse("lastPrice missing"))?;
    let usd_micro =
        parse_decimal_micro(price).ok_or(RateError::Parse("lastPrice not a decimal"))?;
    let at = v
        .get("closeTime")
        .and_then(serde_json::Value::as_i64)
        .and_then(|ms| Utc.timestamp_millis_opt(ms).single())
        .ok_or(RateError::Parse("closeTime missing"))?;
    Ok(RateQuote {
        source: SourceKind::Binance,
        usd_micro,
        at,
    })
}

/// Allowed clock skew for a quote stamped in the future.
const FUTURE_SKEW_SECS: i64 = 120;
/// How long a two-source rate anchors the single-source rule.
const ANCHOR_MAX_AGE_HOURS: i64 = 24;

/// Decide the rate from the quotes at hand. `anchor` is the last rate both
/// sources agreed on, with when it was taken.
pub fn decide(
    quotes: &[RateQuote],
    policy: &RatePolicy,
    anchor: Option<(u64, DateTime<Utc>)>,
    now: DateTime<Utc>,
) -> Result<AcceptedRate, RateRefusal> {
    let mut fresh: Vec<&RateQuote> = Vec::new();
    for q in quotes {
        let age = now - q.at;
        if age > policy.max_age || age < -Duration::seconds(FUTURE_SKEW_SECS) || q.usd_micro == 0 {
            continue;
        }
        // One quote per source: the newest.
        if let Some(pos) = fresh.iter().position(|f| f.source == q.source) {
            if fresh[pos].at < q.at {
                fresh[pos] = q;
            }
            continue;
        }
        fresh.push(q);
    }
    let accepted = match fresh.as_slice() {
        [] => return Err(RateRefusal::NoQuote),
        [one] => {
            let Some((anchor_rate, anchor_at)) = anchor else {
                return Err(RateRefusal::SingleUnconfirmed);
            };
            if now - anchor_at > Duration::hours(ANCHOR_MAX_AGE_HOURS) || anchor_rate == 0 {
                return Err(RateRefusal::SingleUnconfirmed);
            }
            let diff = u128::from(one.usd_micro.abs_diff(anchor_rate));
            if diff * 10_000 > u128::from(policy.max_jump_bp) * u128::from(anchor_rate) {
                return Err(RateRefusal::SingleUnconfirmed);
            }
            AcceptedRate {
                usd_micro: one.usd_micro,
                sources: format!("{}={}", one.source.name(), micro_to_string(one.usd_micro)),
                at: one.at,
                confirmed: false,
            }
        }
        [a, b, ..] => {
            let (x, y) = (a.usd_micro, b.usd_micro);
            let diff = u128::from(x.abs_diff(y));
            if diff * 10_000 > u128::from(policy.max_spread_bp) * u128::from(x.min(y)) {
                return Err(RateRefusal::Divergent { a: x, b: y });
            }
            let mean = ((u128::from(x) + u128::from(y)) / 2) as u64;
            let (first, second) = if a.source == SourceKind::CoinGecko {
                (a, b)
            } else {
                (b, a)
            };
            AcceptedRate {
                usd_micro: mean,
                sources: format!(
                    "{}={};{}={}",
                    first.source.name(),
                    micro_to_string(first.usd_micro),
                    second.source.name(),
                    micro_to_string(second.usd_micro)
                ),
                at: a.at.min(b.at),
                confirmed: true,
            }
        }
    };
    if accepted.usd_micro < policy.min_usd_micro || accepted.usd_micro > policy.max_usd_micro {
        return Err(RateRefusal::OutOfCorridor(accepted.usd_micro));
    }
    Ok(accepted)
}

/// Where quotes come from: HTTP in production, a fake in tests.
#[async_trait]
pub trait RateSource: Send + Sync {
    async fn quotes(&self) -> Vec<Result<RateQuote, RateError>>;
}

/// The two public sources over HTTP.
pub struct HttpRates {
    client: reqwest::Client,
    sources: Vec<SourceKind>,
    coingecko_url: String,
    coingecko_id: String,
    coingecko_key: Option<String>,
    binance_url: String,
    binance_symbol: String,
}

/// Per-request timeout of a rate source.
const RATE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(4);

impl HttpRates {
    pub fn new(cfg: &super::config::TonConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(RATE_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("calrs/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default();
        Self {
            client,
            sources: cfg.rate_sources.clone(),
            coingecko_url: cfg.coingecko_url.clone(),
            coingecko_id: cfg.coingecko_id.clone(),
            coingecko_key: cfg.coingecko_key.clone(),
            binance_url: cfg.binance_url.clone(),
            binance_symbol: cfg.binance_symbol.clone(),
        }
    }

    async fn fetch(&self, req: reqwest::RequestBuilder) -> Result<Vec<u8>, RateError> {
        let resp = req
            .send()
            .await
            .map_err(|e| RateError::Http(super::toncenter::reqwest_class(&e)))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(RateError::Http(format!("http_{}", status.as_u16())));
        }
        let body = resp
            .bytes()
            .await
            .map_err(|e| RateError::Http(super::toncenter::reqwest_class(&e)))?;
        if body.len() > 64 * 1024 {
            return Err(RateError::Parse("answer too large"));
        }
        Ok(body.to_vec())
    }

    async fn coingecko(&self) -> Result<RateQuote, RateError> {
        let mut req = self
            .client
            .get(format!("{}/simple/price", self.coingecko_url))
            .query(&[
                ("ids", self.coingecko_id.as_str()),
                ("vs_currencies", "usd"),
                ("include_last_updated_at", "true"),
            ]);
        if let Some(key) = &self.coingecko_key {
            req = req.header("x-cg-demo-api-key", key);
        }
        let body = self.fetch(req).await?;
        parse_coingecko(&body, &self.coingecko_id)
    }

    async fn binance(&self) -> Result<RateQuote, RateError> {
        let req = self
            .client
            .get(format!("{}/ticker/24hr", self.binance_url))
            .query(&[("symbol", self.binance_symbol.as_str())]);
        let body = self.fetch(req).await?;
        parse_binance(&body, &self.binance_symbol)
    }
}

#[async_trait]
impl RateSource for HttpRates {
    async fn quotes(&self) -> Vec<Result<RateQuote, RateError>> {
        let want_cg = self.sources.contains(&SourceKind::CoinGecko);
        let want_bn = self.sources.contains(&SourceKind::Binance);
        let (cg, bn) = tokio::join!(
            async {
                if want_cg {
                    Some(self.coingecko().await)
                } else {
                    None
                }
            },
            async {
                if want_bn {
                    Some(self.binance().await)
                } else {
                    None
                }
            }
        );
        [cg, bn].into_iter().flatten().collect()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/rates")
            .join(name);
        std::fs::read(path).unwrap()
    }

    pub(crate) fn policy() -> RatePolicy {
        RatePolicy {
            min_usd_micro: 500_000,
            max_usd_micro: 5_000_000,
            max_spread_bp: 300,
            max_age: Duration::minutes(10),
            max_jump_bp: 1000,
        }
    }

    fn q(source: SourceKind, usd_micro: u64, at: DateTime<Utc>) -> RateQuote {
        RateQuote {
            source,
            usd_micro,
            at,
        }
    }

    #[test]
    fn decimals_without_floats() {
        assert_eq!(parse_decimal_micro("1.03800000"), Some(1_038_000));
        assert_eq!(parse_decimal_micro("1.038"), Some(1_038_000));
        assert_eq!(parse_decimal_micro("2"), Some(2_000_000));
        assert_eq!(parse_decimal_micro("3.2412999"), Some(3_241_299));
        for bad in ["", "0", "0.0000001", "-1", "1e-3", "1.0.0", "abc", ".5"] {
            assert_eq!(parse_decimal_micro(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn recorded_answers_parse() {
        let cg = parse_coingecko(&fixture("coingecko.json"), "the-open-network").unwrap();
        assert_eq!(cg.source, SourceKind::CoinGecko);
        assert!(cg.usd_micro > 500_000 && cg.usd_micro < 5_000_000, "{cg:?}");
        assert!(cg.at.timestamp() > 1_794_000_000);
        let bn = parse_binance(&fixture("binance-GRAMUSDT.json"), "GRAMUSDT").unwrap();
        assert!(bn.usd_micro > 500_000 && bn.usd_micro < 5_000_000, "{bn:?}");
        assert_eq!(
            parse_binance(&fixture("binance-GRAMUSDT.json"), "TONUSDT"),
            Err(RateError::Parse("symbol mismatch"))
        );
        assert!(parse_coingecko(&fixture("coingecko.json"), "toncoin").is_err());
        assert!(
            parse_coingecko(b"{\"the-open-network\":{\"usd\":1.0", "the-open-network").is_err()
        );
    }

    #[test]
    fn recorded_quotes_taken_together_give_the_mean() {
        let cg = parse_coingecko(&fixture("coingecko.json"), "the-open-network").unwrap();
        let bn = parse_binance(&fixture("binance-GRAMUSDT.json"), "GRAMUSDT").unwrap();
        let now = cg.at.max(bn.at) + Duration::seconds(30);
        let r = decide(&[cg.clone(), bn.clone()], &policy(), None, now).unwrap();
        assert_eq!(r.usd_micro, (cg.usd_micro + bn.usd_micro) / 2);
        assert!(r.confirmed);
        assert!(r.sources.starts_with("coingecko="), "{}", r.sources);
        assert!(r.sources.contains(";binance="), "{}", r.sources);
    }

    #[test]
    fn the_frozen_tonusdt_pair_is_refused_as_stale() {
        let frozen = parse_binance(&fixture("binance-TONUSDT-break.json"), "TONUSDT").unwrap();
        let cg = parse_coingecko(&fixture("coingecko.json"), "the-open-network").unwrap();
        let now = cg.at + Duration::seconds(30);
        // Frozen since 2026-10-06: only CoinGecko is fresh, with no anchor.
        assert_eq!(
            decide(&[cg.clone(), frozen.clone()], &policy(), None, now),
            Err(RateRefusal::SingleUnconfirmed)
        );
        // Alone it is no quote at all.
        assert_eq!(
            decide(&[frozen], &policy(), None, now),
            Err(RateRefusal::NoQuote)
        );
    }

    #[test]
    fn spread_rule() {
        let now = Utc::now();
        let p = policy();
        // 3.00 % apart: accepted.
        let ok = decide(
            &[
                q(SourceKind::CoinGecko, 1_000_000, now),
                q(SourceKind::Binance, 1_030_000, now),
            ],
            &p,
            None,
            now,
        )
        .unwrap();
        assert_eq!(ok.usd_micro, 1_015_000);
        // 3.01 %: refused.
        assert_eq!(
            decide(
                &[
                    q(SourceKind::CoinGecko, 1_000_000, now),
                    q(SourceKind::Binance, 1_030_100, now)
                ],
                &p,
                None,
                now
            ),
            Err(RateRefusal::Divergent {
                a: 1_000_000,
                b: 1_030_100
            })
        );
    }

    #[test]
    fn single_source_rule() {
        let now = Utc::now();
        let p = policy();
        let stale = now - Duration::minutes(11);
        let quotes = [
            q(SourceKind::CoinGecko, 1_050_000, now),
            q(SourceKind::Binance, 1_000_000, stale),
        ];
        // No anchor: refused.
        assert_eq!(
            decide(&quotes, &p, None, now),
            Err(RateRefusal::SingleUnconfirmed)
        );
        // Anchor within 10 % and younger than 24 h: accepted, unconfirmed.
        let r = decide(
            &quotes,
            &p,
            Some((1_000_000, now - Duration::hours(2))),
            now,
        )
        .unwrap();
        assert_eq!(r.usd_micro, 1_050_000);
        assert!(!r.confirmed);
        assert_eq!(r.sources, "coingecko=1.050000");
        // Anchor too old.
        assert_eq!(
            decide(
                &quotes,
                &p,
                Some((1_000_000, now - Duration::hours(25))),
                now
            ),
            Err(RateRefusal::SingleUnconfirmed)
        );
        // Jump over 10 %.
        assert_eq!(
            decide(&quotes, &p, Some((900_000, now - Duration::hours(1))), now),
            Err(RateRefusal::SingleUnconfirmed)
        );
    }

    #[test]
    fn no_quote_future_quote_and_corridor() {
        let now = Utc::now();
        let p = policy();
        assert_eq!(decide(&[], &p, None, now), Err(RateRefusal::NoQuote));
        let future = now + Duration::minutes(10);
        assert_eq!(
            decide(
                &[q(SourceKind::CoinGecko, 1_000_000, future)],
                &p,
                None,
                now
            ),
            Err(RateRefusal::NoQuote)
        );
        assert_eq!(
            decide(
                &[
                    q(SourceKind::CoinGecko, 6_000_000, now),
                    q(SourceKind::Binance, 6_000_000, now)
                ],
                &p,
                None,
                now
            ),
            Err(RateRefusal::OutOfCorridor(6_000_000))
        );
        assert_eq!(
            decide(
                &[
                    q(SourceKind::CoinGecko, 400_000, now),
                    q(SourceKind::Binance, 400_000, now)
                ],
                &p,
                None,
                now
            ),
            Err(RateRefusal::OutOfCorridor(400_000))
        );
    }
}
