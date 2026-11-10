//! TON payment settings, from the environment of `calrs serve` only: the
//! address and network can never be changed from the database, the web, MCP
//! or the CLI. `CALRS_TON_NETWORK` unset means payments are off. Any invalid
//! value is an error, which also leaves payments off (fail closed): priced
//! event types are then not bookable, never free.
//!
//! `CALRS_TON_API_URL` may embed a provider key in its path (Chainstack), so
//! neither it nor the API keys ever appear in an error, the log or `Debug`.

use chrono::Duration;

use super::address::TonAddress;
use super::rate::{parse_decimal_micro, RatePolicy, SourceKind};
use super::Network;

#[derive(Clone)]
pub struct TonConfig {
    pub network: Network,
    pub address: TonAddress,
    /// `0:HEX`.
    pub address_raw: String,
    /// What guests are shown: `UQ…` (mainnet) / `0Q…` (testnet).
    pub address_friendly: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub poll_secs: u64,
    pub window_min: i64,
    pub grace_min: i64,
    pub expiry_margin_min: i64,
    pub fuse_hours: i64,
    pub underpay_tolerance_nano: u128,
    pub record_min_nano: u128,
    pub max_open_per_email: i64,
    pub max_open_per_ip: i64,
    pub max_open_total: i64,
    pub max_per_day: i64,
    /// Invoices per client network (IPv4 /24, IPv6 /64) in 24 hours,
    /// cancelled and expired ones included.
    pub max_per_ip_day: i64,
    pub rate_sources: Vec<SourceKind>,
    pub coingecko_url: String,
    pub coingecko_id: String,
    pub coingecko_key: Option<String>,
    pub binance_url: String,
    pub binance_symbol: String,
    pub rate_policy: RatePolicy,
    pub alert_email: Option<String>,
}

impl std::fmt::Debug for TonConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TonConfig")
            .field("network", &self.network)
            .field("address", &self.address_friendly)
            .field("indexer_host", &self.indexer_host())
            .field("api_key", &self.api_key.as_ref().map(|_| "<set>"))
            .field(
                "coingecko_key",
                &self.coingecko_key.as_ref().map(|_| "<set>"),
            )
            .finish_non_exhaustive()
    }
}

/// Why the settings were refused: names the variable, never its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn err(msg: impl Into<String>) -> ConfigError {
    ConfigError(msg.into())
}

/// Every variable read (for documentation and the redaction test).
pub const VARS: &[&str] = &[
    "CALRS_TON_NETWORK",
    "CALRS_TON_ADDRESS",
    "CALRS_TON_API_URL",
    "CALRS_TON_API_KEY",
    "CALRS_TON_ALLOW_LOCAL_ENDPOINTS",
    "CALRS_TON_POLL_SECS",
    "CALRS_TON_WINDOW_MIN",
    "CALRS_TON_GRACE_MIN",
    "CALRS_TON_EXPIRY_MARGIN_MIN",
    "CALRS_TON_FUSE_HOURS",
    "CALRS_TON_UNDERPAY_TOLERANCE_NANO",
    "CALRS_TON_RECORD_MIN_NANO",
    "CALRS_TON_MAX_OPEN_PER_EMAIL",
    "CALRS_TON_MAX_OPEN_PER_IP",
    "CALRS_TON_MAX_OPEN_TOTAL",
    "CALRS_TON_MAX_PER_DAY",
    "CALRS_TON_MAX_PER_IP_DAY",
    "CALRS_TON_RATE_SOURCES",
    "CALRS_TON_COINGECKO_KEY",
    "CALRS_TON_COINGECKO_ID",
    "CALRS_TON_COINGECKO_URL",
    "CALRS_TON_BINANCE_SYMBOL",
    "CALRS_TON_BINANCE_URL",
    "CALRS_TON_RATE_MIN_USD",
    "CALRS_TON_RATE_MAX_USD",
    "CALRS_TON_RATE_MAX_SPREAD_PCT",
    "CALRS_TON_RATE_MAX_AGE_MIN",
    "CALRS_TON_RATE_MAX_JUMP_PCT",
    "CALRS_TON_ALERT_EMAIL",
];

struct Vars<F: Fn(&str) -> Option<String>>(F);

impl<F: Fn(&str) -> Option<String>> Vars<F> {
    /// A set, non-blank value, trimmed.
    fn get(&self, name: &str) -> Option<String> {
        (self.0)(name)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    fn int(&self, name: &str, default: i64, min: i64, max: i64) -> Result<i64, ConfigError> {
        let Some(raw) = self.get(name) else {
            return Ok(default);
        };
        let v: i64 = raw
            .parse()
            .map_err(|_| err(format!("{name} must be a whole number")))?;
        if !(min..=max).contains(&v) {
            return Err(err(format!("{name} must be between {min} and {max}")));
        }
        Ok(v)
    }

    fn flag(&self, name: &str) -> Result<bool, ConfigError> {
        match self.get(name).as_deref() {
            None | Some("0") | Some("false") | Some("no") => Ok(false),
            Some("1") | Some("true") | Some("yes") => Ok(true),
            Some(_) => Err(err(format!("{name} must be 0 or 1"))),
        }
    }

    /// A percentage ("3", "0.5") in basis points, within `[min_bp, max_bp]`.
    fn pct_bp(
        &self,
        name: &str,
        default_bp: u64,
        min_bp: u64,
        max_bp: u64,
    ) -> Result<u64, ConfigError> {
        let Some(raw) = self.get(name) else {
            return Ok(default_bp);
        };
        let bp = parse_decimal_micro(&raw)
            .map(|micro| micro / 10_000)
            .ok_or_else(|| err(format!("{name} must be a positive decimal")))?;
        if !(min_bp..=max_bp).contains(&bp) {
            return Err(err(format!(
                "{name} must be between {} and {}",
                min_bp as f64 / 100.0,
                max_bp as f64 / 100.0
            )));
        }
        Ok(bp)
    }

    /// An endpoint base URL: `https://`, or loopback `http://` with the
    /// local-endpoints flag. Returned without a trailing slash.
    fn endpoint(
        &self,
        name: &str,
        default: &str,
        allow_local: bool,
    ) -> Result<String, ConfigError> {
        let raw = self.get(name).unwrap_or_else(|| default.to_string());
        let url = reqwest::Url::parse(&raw).map_err(|_| err(format!("{name} is not a URL")))?;
        if url.fragment().is_some() || url.query().is_some() {
            return Err(err(format!("{name} must not carry a query or fragment")));
        }
        let host = url.host_str().unwrap_or("");
        let local = matches!(host, "127.0.0.1" | "localhost" | "[::1]" | "::1");
        match url.scheme() {
            "https" if !host.is_empty() => {}
            "http" if local && allow_local => {}
            "http" if local => {
                return Err(err(format!(
                    "{name}: a loopback http:// endpoint needs CALRS_TON_ALLOW_LOCAL_ENDPOINTS=1 (dev and tests only)"
                )))
            }
            _ => return Err(err(format!("{name} must be an https:// URL"))),
        }
        Ok(raw.trim_end_matches('/').to_string())
    }
}

/// Host of a URL for the log (`toncenter.com`), never its path or query.
pub fn host_of(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .unwrap_or_else(|| "?".into())
}

impl TonConfig {
    pub fn from_env() -> Result<Option<TonConfig>, ConfigError> {
        Self::from_vars(|k| std::env::var(k).ok())
    }

    pub fn from_vars(
        get: impl Fn(&str) -> Option<String>,
    ) -> Result<Option<TonConfig>, ConfigError> {
        let v = Vars(get);
        let network = match v.get("CALRS_TON_NETWORK").as_deref() {
            None => return Ok(None),
            Some("mainnet") => Network::Mainnet,
            Some("testnet") => Network::Testnet,
            Some(_) => return Err(err("CALRS_TON_NETWORK must be mainnet or testnet")),
        };
        let addr_raw = v
            .get("CALRS_TON_ADDRESS")
            .ok_or_else(|| err("CALRS_TON_ADDRESS is required when CALRS_TON_NETWORK is set"))?;
        let address = TonAddress::for_network(&addr_raw, network)
            .map_err(|e| err(format!("CALRS_TON_ADDRESS: {e}")))?;
        let allow_local = v.flag("CALRS_TON_ALLOW_LOCAL_ENDPOINTS")?;
        let default_api = match network {
            Network::Mainnet => "https://toncenter.com/api/v3",
            Network::Testnet => "https://testnet.toncenter.com/api/v3",
        };
        let api_url = v.endpoint("CALRS_TON_API_URL", default_api, allow_local)?;
        let api_key = v.get("CALRS_TON_API_KEY");
        if api_key
            .as_deref()
            .is_some_and(|k| k.chars().any(|c| c.is_control()))
        {
            return Err(err("CALRS_TON_API_KEY contains control characters"));
        }

        let mut rate_sources = Vec::new();
        let sources = v
            .get("CALRS_TON_RATE_SOURCES")
            .unwrap_or_else(|| "coingecko,binance".into());
        for name in sources.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let kind = SourceKind::parse(name)
                .ok_or_else(|| err("CALRS_TON_RATE_SOURCES is a list of coingecko and binance"))?;
            if !rate_sources.contains(&kind) {
                rate_sources.push(kind);
            }
        }
        if rate_sources.is_empty() {
            return Err(err("CALRS_TON_RATE_SOURCES names no source"));
        }
        let coingecko_id = v
            .get("CALRS_TON_COINGECKO_ID")
            .unwrap_or_else(|| "the-open-network".into());
        if coingecko_id.is_empty()
            || coingecko_id.len() > 64
            || !coingecko_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(err("CALRS_TON_COINGECKO_ID must match [a-z0-9-]{1,64}"));
        }
        let binance_symbol = v
            .get("CALRS_TON_BINANCE_SYMBOL")
            .unwrap_or_else(|| "GRAMUSDT".into());
        if !(3..=20).contains(&binance_symbol.len())
            || !binance_symbol
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        {
            return Err(err("CALRS_TON_BINANCE_SYMBOL must match [A-Z0-9]{3,20}"));
        }
        let coingecko_url = v.endpoint(
            "CALRS_TON_COINGECKO_URL",
            "https://api.coingecko.com/api/v3",
            allow_local,
        )?;
        let binance_url = v.endpoint(
            "CALRS_TON_BINANCE_URL",
            "https://api.binance.com/api/v3",
            allow_local,
        )?;

        let corridor = |name: &str| -> Result<u64, ConfigError> {
            let raw = v
                .get(name)
                .ok_or_else(|| err(format!("{name} is required: the owner's TON/USD corridor")))?;
            parse_decimal_micro(&raw)
                .ok_or_else(|| err(format!("{name} must be a positive decimal")))
        };
        let min_usd_micro = corridor("CALRS_TON_RATE_MIN_USD")?;
        let max_usd_micro = corridor("CALRS_TON_RATE_MAX_USD")?;
        if min_usd_micro >= max_usd_micro {
            return Err(err(
                "CALRS_TON_RATE_MIN_USD must be below CALRS_TON_RATE_MAX_USD",
            ));
        }
        let rate_policy = RatePolicy {
            min_usd_micro,
            max_usd_micro,
            max_spread_bp: v.pct_bp("CALRS_TON_RATE_MAX_SPREAD_PCT", 300, 10, 2000)?,
            max_age: Duration::minutes(v.int("CALRS_TON_RATE_MAX_AGE_MIN", 10, 1, 60)?),
            max_jump_bp: v.pct_bp("CALRS_TON_RATE_MAX_JUMP_PCT", 1000, 100, 5000)?,
        };

        let alert_email = v.get("CALRS_TON_ALERT_EMAIL");
        if alert_email
            .as_deref()
            .is_some_and(|e| !e.contains('@') || e.contains(char::is_whitespace) || e.len() > 254)
        {
            return Err(err("CALRS_TON_ALERT_EMAIL is not an email address"));
        }

        Ok(Some(TonConfig {
            network,
            address,
            address_raw: address.raw(),
            address_friendly: address.for_guest(network),
            api_url,
            api_key,
            poll_secs: v.int("CALRS_TON_POLL_SECS", 10, 5, 60)? as u64,
            window_min: v.int("CALRS_TON_WINDOW_MIN", 30, 10, 240)?,
            grace_min: v.int("CALRS_TON_GRACE_MIN", 15, 5, 60)?,
            expiry_margin_min: v.int("CALRS_TON_EXPIRY_MARGIN_MIN", 3, 1, 30)?,
            fuse_hours: v.int("CALRS_TON_FUSE_HOURS", 6, 1, 48)?,
            underpay_tolerance_nano: v.int(
                "CALRS_TON_UNDERPAY_TOLERANCE_NANO",
                0,
                0,
                100_000_000,
            )? as u128,
            record_min_nano: v.int(
                "CALRS_TON_RECORD_MIN_NANO",
                100_000_000,
                0,
                1_000_000_000_000,
            )? as u128,
            max_open_per_email: v.int("CALRS_TON_MAX_OPEN_PER_EMAIL", 1, 1, 100)?,
            max_open_per_ip: v.int("CALRS_TON_MAX_OPEN_PER_IP", 2, 1, 100)?,
            max_open_total: v.int("CALRS_TON_MAX_OPEN_TOTAL", 5, 1, 100)?,
            max_per_day: v.int("CALRS_TON_MAX_PER_DAY", 50, 1, 1000)?,
            max_per_ip_day: v.int("CALRS_TON_MAX_PER_IP_DAY", 10, 1, 1000)?,
            rate_sources,
            coingecko_url,
            coingecko_id,
            coingecko_key: v.get("CALRS_TON_COINGECKO_KEY"),
            binance_url,
            binance_symbol,
            rate_policy,
            alert_email,
        }))
    }

    /// The indexer's host, for the log.
    pub fn indexer_host(&self) -> String {
        host_of(&self.api_url)
    }

    /// Invoice-time rules for an event type (`window_min` overrides).
    pub fn window_rules(&self, window_min: Option<i64>) -> super::deadlines::WindowRules {
        super::deadlines::WindowRules {
            window_min: window_min.unwrap_or(self.window_min),
            grace_min: self.grace_min,
            fuse_hours: self.fuse_hours,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::HashMap;

    /// The TON Foundation address in its testnet-only form (`0Q…`).
    pub(crate) fn testnet_addr() -> String {
        TonAddress::parse("UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI")
            .unwrap()
            .0
            .friendly(false, true)
    }

    pub(crate) fn base() -> HashMap<&'static str, String> {
        HashMap::from([
            ("CALRS_TON_NETWORK", "mainnet".to_string()),
            (
                "CALRS_TON_ADDRESS",
                "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI".to_string(),
            ),
            ("CALRS_TON_RATE_MIN_USD", "0.5".to_string()),
            ("CALRS_TON_RATE_MAX_USD", "5".to_string()),
        ])
    }

    fn load(vars: &HashMap<&'static str, String>) -> Result<Option<TonConfig>, ConfigError> {
        TonConfig::from_vars(|k| vars.get(k).cloned())
    }

    fn with(k: &'static str, val: &str) -> Result<Option<TonConfig>, ConfigError> {
        let mut vars = base();
        vars.insert(k, val.to_string());
        load(&vars)
    }

    #[test]
    fn unset_network_means_off() {
        assert!(TonConfig::from_vars(|_| None).unwrap().is_none());
        assert!(
            TonConfig::from_vars(|k| (k == "CALRS_TON_NETWORK").then(|| "  ".into()))
                .unwrap()
                .is_none()
        );
        // Other variables alone do not switch it on.
        let mut vars = base();
        vars.remove("CALRS_TON_NETWORK");
        assert!(load(&vars).unwrap().is_none());
    }

    #[test]
    fn defaults() {
        let c = load(&base()).unwrap().unwrap();
        assert_eq!(c.network, Network::Mainnet);
        assert_eq!(
            c.address_raw,
            "0:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8"
        );
        assert_eq!(
            c.address_friendly,
            "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI"
        );
        assert_eq!(c.api_url, "https://toncenter.com/api/v3");
        assert_eq!(c.api_key, None);
        assert_eq!(
            (
                c.poll_secs,
                c.window_min,
                c.grace_min,
                c.expiry_margin_min,
                c.fuse_hours
            ),
            (10, 30, 15, 3, 6)
        );
        assert_eq!(c.underpay_tolerance_nano, 0);
        assert_eq!(c.record_min_nano, 100_000_000);
        assert_eq!(
            (
                c.max_open_per_email,
                c.max_open_per_ip,
                c.max_open_total,
                c.max_per_day,
                c.max_per_ip_day
            ),
            (1, 2, 5, 50, 10)
        );
        assert_eq!(
            c.rate_sources,
            vec![SourceKind::CoinGecko, SourceKind::Binance]
        );
        assert_eq!(c.binance_symbol, "GRAMUSDT");
        assert_eq!(c.coingecko_id, "the-open-network");
        assert_eq!(c.rate_policy.max_spread_bp, 300);
        assert_eq!(c.rate_policy.max_jump_bp, 1000);
        assert_eq!(c.rate_policy.max_age, Duration::minutes(10));
        assert_eq!(c.rate_policy.min_usd_micro, 500_000);
        assert_eq!(c.rate_policy.max_usd_micro, 5_000_000);
        assert_eq!(c.indexer_host(), "toncenter.com");
    }

    #[test]
    fn testnet_defaults_and_addresses() {
        let mut vars = base();
        vars.insert("CALRS_TON_NETWORK", "testnet".into());
        let c = load(&vars).unwrap().unwrap();
        assert_eq!(c.api_url, "https://testnet.toncenter.com/api/v3");
        assert!(
            c.address_friendly.starts_with("0Q"),
            "{}",
            c.address_friendly
        );
        vars.insert("CALRS_TON_ADDRESS", testnet_addr());
        assert!(load(&vars).unwrap().is_some());
        // The testnet-only form on mainnet is refused.
        let e = with("CALRS_TON_ADDRESS", &testnet_addr()).unwrap_err();
        assert!(e.0.contains("testnet-only"), "{e}");
    }

    #[test]
    fn invalid_values_are_errors() {
        for (k, val) in [
            ("CALRS_TON_NETWORK", "main"),
            (
                "CALRS_TON_ADDRESS",
                "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBX",
            ),
            (
                "CALRS_TON_ADDRESS",
                "-1:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8",
            ),
            ("CALRS_TON_POLL_SECS", "4"),
            ("CALRS_TON_POLL_SECS", "61"),
            ("CALRS_TON_POLL_SECS", "ten"),
            ("CALRS_TON_WINDOW_MIN", "9"),
            ("CALRS_TON_WINDOW_MIN", "241"),
            ("CALRS_TON_GRACE_MIN", "4"),
            ("CALRS_TON_EXPIRY_MARGIN_MIN", "0"),
            ("CALRS_TON_FUSE_HOURS", "49"),
            ("CALRS_TON_UNDERPAY_TOLERANCE_NANO", "-1"),
            ("CALRS_TON_UNDERPAY_TOLERANCE_NANO", "100000001"),
            ("CALRS_TON_RECORD_MIN_NANO", "1000000000001"),
            ("CALRS_TON_MAX_OPEN_PER_EMAIL", "0"),
            ("CALRS_TON_MAX_PER_DAY", "1001"),
            ("CALRS_TON_MAX_PER_IP_DAY", "0"),
            ("CALRS_TON_RATE_SOURCES", "kraken"),
            ("CALRS_TON_RATE_SOURCES", ","),
            ("CALRS_TON_COINGECKO_ID", "The Open Network"),
            ("CALRS_TON_BINANCE_SYMBOL", "gramusdt"),
            ("CALRS_TON_BINANCE_SYMBOL", "AB"),
            ("CALRS_TON_RATE_MIN_USD", "5"),
            ("CALRS_TON_RATE_MIN_USD", "-1"),
            ("CALRS_TON_RATE_MAX_USD", "abc"),
            ("CALRS_TON_RATE_MAX_SPREAD_PCT", "0.05"),
            ("CALRS_TON_RATE_MAX_SPREAD_PCT", "21"),
            ("CALRS_TON_RATE_MAX_AGE_MIN", "0"),
            ("CALRS_TON_RATE_MAX_JUMP_PCT", "0.5"),
            ("CALRS_TON_ALLOW_LOCAL_ENDPOINTS", "maybe"),
            ("CALRS_TON_ALERT_EMAIL", "not-an-address"),
            ("CALRS_TON_API_URL", "http://toncenter.com/api/v3"),
            ("CALRS_TON_API_URL", "ftp://toncenter.com"),
            ("CALRS_TON_API_URL", "toncenter.com"),
            (
                "CALRS_TON_API_URL",
                "https://toncenter.com/api/v3?api_key=x",
            ),
            ("CALRS_TON_API_URL", "http://127.0.0.1:18080/api/v3"),
            ("CALRS_TON_COINGECKO_URL", "http://localhost:1/cg"),
            ("CALRS_TON_BINANCE_URL", "http://example.com/bn"),
        ] {
            assert!(with(k, val).is_err(), "{k}={val}");
        }
        // A missing address or corridor: error, not "off".
        for k in [
            "CALRS_TON_ADDRESS",
            "CALRS_TON_RATE_MIN_USD",
            "CALRS_TON_RATE_MAX_USD",
        ] {
            let mut vars = base();
            vars.remove(k);
            let e = load(&vars).unwrap_err();
            assert!(e.0.contains(k), "{e}");
        }
    }

    #[test]
    fn valid_overrides() {
        let c = with("CALRS_TON_RATE_SOURCES", " binance ")
            .unwrap()
            .unwrap();
        assert_eq!(c.rate_sources, vec![SourceKind::Binance]);
        let c = with("CALRS_TON_RATE_MAX_SPREAD_PCT", "0.5")
            .unwrap()
            .unwrap();
        assert_eq!(c.rate_policy.max_spread_bp, 50);
        let c = with("CALRS_TON_UNDERPAY_TOLERANCE_NANO", "10000000")
            .unwrap()
            .unwrap();
        assert_eq!(c.underpay_tolerance_nano, 10_000_000);
        let c = with(
            "CALRS_TON_API_URL",
            "https://ton-mainnet.core.chainstack.com/KEY123/api/v3/",
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            c.api_url,
            "https://ton-mainnet.core.chainstack.com/KEY123/api/v3"
        );
        assert_eq!(c.indexer_host(), "ton-mainnet.core.chainstack.com");
        let mut vars = base();
        vars.insert("CALRS_TON_ALLOW_LOCAL_ENDPOINTS", "1".into());
        vars.insert("CALRS_TON_API_URL", "http://127.0.0.1:18080/api/v3".into());
        vars.insert(
            "CALRS_TON_COINGECKO_URL",
            "http://localhost:18080/cg".into(),
        );
        let c = load(&vars).unwrap().unwrap();
        assert_eq!(c.api_url, "http://127.0.0.1:18080/api/v3");
        // The flag does not open other plain-http hosts.
        vars.insert("CALRS_TON_BINANCE_URL", "http://example.com/bn".into());
        assert!(load(&vars).is_err());
    }

    #[test]
    fn secrets_never_show_in_errors_or_debug() {
        let mut vars = base();
        vars.insert(
            "CALRS_TON_API_URL",
            "https://h.example/SECRETPATH/api/v3".into(),
        );
        vars.insert("CALRS_TON_API_KEY", "SECRETKEY".into());
        vars.insert("CALRS_TON_COINGECKO_KEY", "SECRETCG".into());
        let c = load(&vars).unwrap().unwrap();
        let dbg = format!("{c:?}");
        for s in ["SECRETPATH", "SECRETKEY", "SECRETCG"] {
            assert!(!dbg.contains(s), "{dbg}");
        }
        vars.insert(
            "CALRS_TON_API_URL",
            "http://h.example/SECRETPATH/api/v3".into(),
        );
        let e = load(&vars).unwrap_err();
        assert!(!e.0.contains("SECRETPATH"), "{e}");
        assert!(VARS.iter().all(|v| v.starts_with("CALRS_TON_")));
    }
}
