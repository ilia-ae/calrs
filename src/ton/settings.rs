//! ilia fork: the TON settings the admin page (`/dashboard/admin/ton`) and
//! `config apply` (the `ton` section) may store, and who wins.
//!
//! **One value per key, three places, one parser.** A key is read from the
//! environment (`CALRS_TON_*`), else from its `ton_settings` row, else it
//! takes its default: env > DB > default. A row has one value and one owner
//! tag, `panel` or `apply`; the tag decides who may change it (a row owned
//! by `apply` is read-only on the page), never which value wins. Values are
//! stored in the environment's own text format, and every source goes
//! through [`super::config::Tuning::parse`], so the page, the environment
//! and `config apply` accept exactly the same values.
//!
//! **Only tuning keys** ([`KEYS`]). The network, the address, the indexer
//! URL and keys, the rate sources and their URLs, the limits and
//! `CALRS_TON_ALLOW_LOCAL_ENDPOINTS` stay with the environment (and the
//! systemd credentials): a database row for any other name is ignored.
//!
//! **One key that is not TON's:** [`DEBUG_BOOKING`] (`CALRS_DEBUG_BOOKING`,
//! field `debug_booking`, a [`Kind::Bool`]), the switch of the admin's test
//! booking page (`crate::web::debug_booking`). It is toggled at the top of
//! the TON page and stored, owned and audited exactly like the TON keys
//! (one mechanism), but no TON parser reads it ([`debug_booking`] does),
//! and unlike them it is read on every request: a change is live at once.
//! Its companion [`DEBUG_BOOKING_TELEGRAM`] (`CALRS_DEBUG_BOOKING_TELEGRAM`,
//! field `debug_booking_telegram`, a [`Kind::Telegram`]) is the guest
//! Telegram username that page prefills when the admin's linked chat has
//! none (the Debug block's own form, or `config apply`), resolved the same
//! way ([`debug_booking_telegram`]). [`is_debug`] names both.
//!
//! A row whose value is empty means "the default, owned": `config apply`
//! writes one for a key it declares as `null`, so the page keeps it locked.
//! Pure: no database, no web (the SQL is `crate::web::ton_pay::settings`).

use std::collections::BTreeMap;

use super::amount::parse_ton;
use super::config::{self as cfg, ConfigError, IntSpec, PctSpec, Tuning, Vars};
use super::ladder::{ton_text, Ladder};

/// Who stored a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Owner {
    /// The admin page.
    Panel,
    /// `calrs config apply` (the `ton` section).
    Apply,
}

impl Owner {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Panel => "panel",
            Self::Apply => "apply",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "panel" => Some(Self::Panel),
            "apply" => Some(Self::Apply),
            _ => None,
        }
    }
}

/// The stored rows (`ton_settings`), by variable name: the value in the
/// environment's text format (empty: the default) and its owner.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DbValues(pub BTreeMap<String, (String, Owner)>);

impl DbValues {
    pub fn value(&self, var: &str) -> Option<&str> {
        self.0.get(var).map(|(v, _)| v.as_str())
    }

    pub fn owner(&self, var: &str) -> Option<Owner> {
        self.0.get(var).map(|(_, o)| *o)
    }

    pub fn set(&mut self, var: &str, value: String, owner: Owner) {
        self.0.insert(var.to_string(), (value, owner));
    }

    pub fn remove(&mut self, var: &str) {
        self.0.remove(var);
    }
}

/// The longest per-guest limit of free ladder steps, in days.
pub const FREE_PER_GUEST_DAYS_MAX: i64 = cfg::FREE_PER_GUEST_DAYS.max;

/// Where a key's effective value comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Env,
    Apply,
    Panel,
    Default,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Apply => "apply",
            Self::Panel => "panel",
            Self::Default => "default",
        }
    }
}

/// How a key's value is written by a person (the page, `config apply`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A whole number in its own unit (minutes, hours, days, a count).
    Int(IntSpec),
    /// Stored in nanotons, written in TON.
    Nano(IntSpec),
    /// A positive USD decimal (the corridor).
    Usd,
    /// A percentage, at most two decimals.
    Pct(PctSpec),
    Email,
    Ladder,
    /// On or off, stored as `1` or `0` ([`parse_bool`]).
    Bool,
    /// A Telegram username as the booking form's field takes it (`@name`,
    /// `t.me/name`), stored bare (`name`).
    Telegram,
}

/// A key the page and `config apply` may store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    /// The environment variable, also the `ton_settings.key`.
    pub var: &'static str,
    /// The form field and the `config apply` key (`ton.<field>`).
    pub field: &'static str,
    pub kind: Kind,
}

impl Key {
    /// The default as a person writes it ("" when it is "not set").
    pub fn default_text(&self) -> String {
        match self.kind {
            Kind::Int(s) => s.default.to_string(),
            Kind::Nano(s) => ton_text(s.default as u128),
            Kind::Pct(s) => bp_text(s.default_bp),
            Kind::Ladder => super::ladder::DEFAULT.to_string(),
            Kind::Bool => "off".to_string(),
            Kind::Usd | Kind::Email | Kind::Telegram => String::new(),
        }
    }
}

/// The test booking page's switch (`crate::web::debug_booking`): off by
/// default; the environment, if set, wins and locks the toggle.
pub const DEBUG_BOOKING: &str = "CALRS_DEBUG_BOOKING";
/// The guest Telegram username the test booking page prefills when the
/// admin's linked chat has no username: unset by default.
pub const DEBUG_BOOKING_TELEGRAM: &str = "CALRS_DEBUG_BOOKING_TELEGRAM";

/// The keys of the test booking page, not TON's: stored, owned and audited
/// like the others, never part of the TON form and read per request (a
/// change needs no restart).
pub fn is_debug(var: &str) -> bool {
    var == DEBUG_BOOKING || var == DEBUG_BOOKING_TELEGRAM
}

const fn key(var: &'static str, field: &'static str, kind: Kind) -> Key {
    Key { var, field, kind }
}

/// Every stored key, in the order of the page and of `config apply`.
pub const SPECS: &[Key] = &[
    key(
        cfg::WINDOW_MIN.var,
        "window_min",
        Kind::Int(cfg::WINDOW_MIN),
    ),
    key(cfg::GRACE_MIN.var, "grace_min", Kind::Int(cfg::GRACE_MIN)),
    key(
        cfg::FUSE_HOURS.var,
        "fuse_hours",
        Kind::Int(cfg::FUSE_HOURS),
    ),
    key(
        cfg::RENEW_MIN_LEAD_HOURS.var,
        "renew_min_lead_hours",
        Kind::Int(cfg::RENEW_MIN_LEAD_HOURS),
    ),
    key(
        cfg::RENEW_WINDOW_MIN.var,
        "renew_window_min",
        Kind::Int(cfg::RENEW_WINDOW_MIN),
    ),
    key(
        cfg::MAX_RENEWALS.var,
        "max_renewals",
        Kind::Int(cfg::MAX_RENEWALS),
    ),
    key(
        cfg::UNDERPAY_TOLERANCE_NANO.var,
        "underpay_tolerance_ton",
        Kind::Nano(cfg::UNDERPAY_TOLERANCE_NANO),
    ),
    key(cfg::RATE_MIN_USD, "rate_min_usd", Kind::Usd),
    key(cfg::RATE_MAX_USD, "rate_max_usd", Kind::Usd),
    key(
        cfg::RATE_MAX_SPREAD_PCT.var,
        "rate_max_spread_pct",
        Kind::Pct(cfg::RATE_MAX_SPREAD_PCT),
    ),
    key(
        cfg::RATE_MAX_AGE_MIN.var,
        "rate_max_age_min",
        Kind::Int(cfg::RATE_MAX_AGE_MIN),
    ),
    key(
        cfg::RATE_MAX_JUMP_PCT.var,
        "rate_max_jump_pct",
        Kind::Pct(cfg::RATE_MAX_JUMP_PCT),
    ),
    key(cfg::ALERT_EMAIL, "alert_email", Kind::Email),
    key(cfg::LADDER, "ladder", Kind::Ladder),
    key(
        cfg::FREE_PER_GUEST_DAYS.var,
        "free_per_guest_days",
        Kind::Int(cfg::FREE_PER_GUEST_DAYS),
    ),
    key(DEBUG_BOOKING, "debug_booking", Kind::Bool),
    key(
        DEBUG_BOOKING_TELEGRAM,
        "debug_booking_telegram",
        Kind::Telegram,
    ),
];

/// The variable names of [`SPECS`]: the only `ton_settings` keys read.
pub const KEYS: &[&str] = &[
    cfg::WINDOW_MIN.var,
    cfg::GRACE_MIN.var,
    cfg::FUSE_HOURS.var,
    cfg::RENEW_MIN_LEAD_HOURS.var,
    cfg::RENEW_WINDOW_MIN.var,
    cfg::MAX_RENEWALS.var,
    cfg::UNDERPAY_TOLERANCE_NANO.var,
    cfg::RATE_MIN_USD,
    cfg::RATE_MAX_USD,
    cfg::RATE_MAX_SPREAD_PCT.var,
    cfg::RATE_MAX_AGE_MIN.var,
    cfg::RATE_MAX_JUMP_PCT.var,
    cfg::ALERT_EMAIL,
    cfg::LADDER,
    cfg::FREE_PER_GUEST_DAYS.var,
    DEBUG_BOOKING,
    DEBUG_BOOKING_TELEGRAM,
];

pub fn by_var(var: &str) -> Option<&'static Key> {
    SPECS.iter().find(|k| k.var == var)
}

pub fn by_field(field: &str) -> Option<&'static Key> {
    SPECS.iter().find(|k| k.field == field)
}

/// A set, non-blank value, trimmed (as `config::Vars` reads one).
fn non_blank(v: Option<String>) -> Option<String> {
    v.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// The value `calrs serve` uses for `var`: the environment, else (tuning
/// keys only) the stored row, else `None` (the default).
pub fn lookup(env: &dyn Fn(&str) -> Option<String>, db: &DbValues, var: &str) -> Option<String> {
    if let Some(v) = non_blank(env(var)) {
        return Some(v);
    }
    by_var(var)?;
    non_blank(db.value(var).map(str::to_string))
}

/// Where `var`'s effective value comes from. A row with an empty value is
/// still its owner's (the default, kept locked).
pub fn source(env: &dyn Fn(&str) -> Option<String>, db: &DbValues, var: &str) -> Source {
    if non_blank(env(var)).is_some() {
        return Source::Env;
    }
    match db.owner(var) {
        Some(Owner::Apply) => Source::Apply,
        Some(Owner::Panel) => Source::Panel,
        None => Source::Default,
    }
}

/// The merged tuning, strictly checked: what the page and `config apply`
/// validate before they store anything (the whole result, so cross-field
/// rules such as the corridor run on what would be in effect).
pub fn validate(
    env: &dyn Fn(&str) -> Option<String>,
    db: &DbValues,
) -> Result<Tuning, ConfigError> {
    Tuning::parse(&Vars(|k: &str| lookup(env, db, k)), true)
}

/// A value and where it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved<T> {
    pub value: T,
    pub source: Source,
}

/// The process environment as a lookup (`calrs serve` and the CLI).
pub fn process_env(var: &str) -> Option<String> {
    std::env::var(var).ok()
}

/// The demand ladder in effect (env > DB > default), by the one parser.
pub fn ladder(
    env: &dyn Fn(&str) -> Option<String>,
    db: &DbValues,
) -> Result<Resolved<Ladder>, ConfigError> {
    Ok(Resolved {
        value: Vars(|k: &str| lookup(env, db, k)).ladder(cfg::LADDER)?,
        source: source(env, db, cfg::LADDER),
    })
}

/// The per-guest limit of free steps in effect, in days (0: none).
pub fn free_per_guest_days(
    env: &dyn Fn(&str) -> Option<String>,
    db: &DbValues,
) -> Result<Resolved<i64>, ConfigError> {
    Ok(Resolved {
        value: Vars(|k: &str| lookup(env, db, k)).int_spec(cfg::FREE_PER_GUEST_DAYS)?,
        source: source(env, db, cfg::FREE_PER_GUEST_DAYS.var),
    })
}

/// An on/off value as the environment, the page and the database write it:
/// `1`, `true`, `yes`, `on` or `0`, `false`, `no`, `off` (any case, trimmed).
/// Anything else (blank included) is `None`.
pub fn parse_bool(text: &str) -> Option<bool> {
    match text.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// The test booking switch in effect: the environment (set and not blank)
/// wins, else the stored row, else off. A value that is not on/off is off
/// (fail closed), still from where it was set, so an environment typo
/// keeps the toggle locked rather than handing it to the page.
pub fn debug_booking(env: &dyn Fn(&str) -> Option<String>, db: &DbValues) -> Resolved<bool> {
    Resolved {
        value: lookup(env, db, DEBUG_BOOKING)
            .and_then(|v| parse_bool(&v))
            .unwrap_or(false),
        source: source(env, db, DEBUG_BOOKING),
    }
}

/// The guest Telegram username the test booking page prefills (bare, no
/// `@`): the environment, else the stored row, else none. A value that is
/// not a username is none, still from where it was set (locked there).
pub fn debug_booking_telegram(
    env: &dyn Fn(&str) -> Option<String>,
    db: &DbValues,
) -> Resolved<Option<String>> {
    Resolved {
        value: lookup(env, db, DEBUG_BOOKING_TELEGRAM)
            .and_then(|v| crate::telegram::username::normalize(&v)),
        source: source(env, db, DEBUG_BOOKING_TELEGRAM),
    }
}

/// The key a [`ConfigError`] is about (its message names the variable).
pub fn key_of_error(e: &ConfigError) -> Option<&'static Key> {
    SPECS.iter().find(|k| e.0.starts_with(k.var))
}

/// A value as a person wrote it (the page's units: TON for the tolerance)
/// to the stored text. `Ok(None)`: blank, the default. The message names
/// the variable; the range and format are the environment's own.
pub fn to_env(key: &Key, human: &str) -> Result<Option<String>, ConfigError> {
    let human = human.trim();
    if human.is_empty() {
        return Ok(None);
    }
    let one = |text: String| move |k: &str| (k == key.var).then(|| text.clone());
    let text = match key.kind {
        Kind::Int(spec) => Vars(one(human.to_string())).int_spec(spec)?.to_string(),
        Kind::Nano(spec) => {
            let nano = parse_ton(human)
                .and_then(|n| i64::try_from(n).ok())
                .ok_or_else(|| {
                    ConfigError(format!("{}: an amount in TON, such as 0.01", key.var))
                })?;
            Vars(one(nano.to_string())).int_spec(spec)?.to_string()
        }
        Kind::Usd => {
            let micro = Vars(one(human.to_string())).usd(key.var)?.unwrap_or(0);
            micro_text(micro)
        }
        Kind::Pct(spec) => bp_text(Vars(one(human.to_string())).pct_spec(spec)?),
        Kind::Email => Vars(one(human.to_string()))
            .email(key.var)?
            .unwrap_or_default(),
        Kind::Ladder => Vars(one(human.to_string())).ladder(key.var)?.text(),
        Kind::Bool => match parse_bool(human) {
            Some(true) => "1".to_string(),
            Some(false) => "0".to_string(),
            None => return Err(ConfigError(format!("{}: on or off (1 or 0)", key.var))),
        },
        Kind::Telegram => crate::telegram::username::normalize(human).ok_or_else(|| {
            ConfigError(format!(
                "{}: a Telegram username: a Latin letter, then letters, digits and _, 4 to 32 characters (@name or t.me/name also work)",
                key.var
            ))
        })?,
    };
    Ok(Some(text))
}

/// A stored (or environment) value in the page's units.
pub fn to_human(key: &Key, env_text: &str) -> String {
    let text = env_text.trim();
    match key.kind {
        Kind::Nano(_) => text
            .parse::<u128>()
            .map(ton_text)
            .unwrap_or_else(|_| text.to_string()),
        Kind::Bool => match parse_bool(text) {
            Some(true) => "on".to_string(),
            Some(false) => "off".to_string(),
            None => text.to_string(),
        },
        _ => text.to_string(),
    }
}

/// The canonical spelling of a value written by a person: what
/// [`to_env`] stores, read back in the page's units. `config apply`
/// refuses anything else, so a document and an export compare as text.
pub fn canonical(key: &Key, human: &str) -> Result<Option<String>, ConfigError> {
    Ok(to_env(key, human)?.map(|t| to_human(key, &t)))
}

/// Micro-units as a decimal without trailing zeros ("0.3", "5").
fn micro_text(micro: u64) -> String {
    let whole = micro / 1_000_000;
    let frac = format!("{:06}", micro % 1_000_000);
    let frac = frac.trim_end_matches('0');
    if frac.is_empty() {
        whole.to_string()
    } else {
        format!("{whole}.{frac}")
    }
}

/// Basis points as a percentage ("3", "0.5").
fn bp_text(bp: u64) -> String {
    micro_text(bp * 10_000)
}

/// The price table of the page's preview: `(n, step)` for `n` from 0 to
/// one past the ceiling's step.
pub fn ladder_preview(ladder: &Ladder) -> Vec<(usize, String)> {
    (0..=ladder.steps().len())
        .map(|n| (n, ladder.price(n as i64).text()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn no_env(_: &str) -> Option<String> {
        None
    }

    fn row(value: &str, owner: Owner) -> (String, Owner) {
        (value.to_string(), owner)
    }

    #[test]
    fn env_wins_then_db_then_default() {
        let env: HashMap<&str, &str> = HashMap::from([("CALRS_TON_GRACE_MIN", "20")]);
        let env = |k: &str| env.get(k).map(|v| v.to_string());
        let mut db = DbValues::default();
        db.0.insert("CALRS_TON_GRACE_MIN".into(), row("30", Owner::Panel));
        db.0.insert("CALRS_TON_WINDOW_MIN".into(), row("45", Owner::Apply));
        db.0.insert("CALRS_TON_FUSE_HOURS".into(), row("", Owner::Apply));
        assert_eq!(
            lookup(&env, &db, "CALRS_TON_GRACE_MIN").as_deref(),
            Some("20")
        );
        assert_eq!(source(&env, &db, "CALRS_TON_GRACE_MIN"), Source::Env);
        assert_eq!(
            lookup(&env, &db, "CALRS_TON_WINDOW_MIN").as_deref(),
            Some("45")
        );
        assert_eq!(source(&env, &db, "CALRS_TON_WINDOW_MIN"), Source::Apply);
        // An empty row: the default, still owned.
        assert_eq!(lookup(&env, &db, "CALRS_TON_FUSE_HOURS"), None);
        assert_eq!(source(&env, &db, "CALRS_TON_FUSE_HOURS"), Source::Apply);
        assert_eq!(source(&env, &db, "CALRS_TON_MAX_RENEWALS"), Source::Default);
        let t = validate(&env, &db).unwrap();
        assert_eq!((t.grace_min, t.window_min, t.fuse_hours), (20, 45, 6));
    }

    /// The address, the network and every key outside `KEYS` never come
    /// from the database.
    #[test]
    fn identity_keys_are_env_only() {
        let mut db = DbValues::default();
        for var in [
            "CALRS_TON_NETWORK",
            "CALRS_TON_ADDRESS",
            "CALRS_TON_API_URL",
            "CALRS_TON_API_KEY",
            "CALRS_TON_ALLOW_LOCAL_ENDPOINTS",
            "CALRS_TON_POLL_SECS",
        ] {
            db.0.insert(var.into(), row("x", Owner::Panel));
            assert_eq!(lookup(&no_env, &db, var), None, "{var}");
        }
        let resolved = cfg::TonConfig::resolve(&no_env, &db).unwrap();
        assert!(resolved.is_none(), "payments stay off without the env");
        assert_eq!(KEYS.len(), SPECS.len());
        for (k, spec) in KEYS.iter().zip(SPECS) {
            assert_eq!(*k, spec.var);
            // The test booking switch is the one key no TON parser reads.
            assert!(cfg::VARS.contains(k) || is_debug(k), "{k}");
        }
    }

    #[test]
    fn values_are_converted_and_checked_as_the_env() {
        let k = |f: &str| by_field(f).unwrap();
        let ok = |f: &str, v: &str| to_env(k(f), v).unwrap();
        assert_eq!(ok("window_min", " 45 "), Some("45".into()));
        assert_eq!(ok("window_min", ""), None);
        assert_eq!(
            ok("underpay_tolerance_ton", "0.01"),
            Some("10000000".into())
        );
        assert_eq!(ok("underpay_tolerance_ton", "0"), Some("0".into()));
        assert_eq!(ok("rate_min_usd", "0.30"), Some("0.3".into()));
        assert_eq!(ok("rate_max_spread_pct", "3.0"), Some("3".into()));
        assert_eq!(ok("rate_max_jump_pct", "12.5"), Some("12.5".into()));
        assert_eq!(ok("ladder", "FREE, 20 ,40.50"), Some("free,20,40.5".into()));
        assert_eq!(ok("alert_email", "a@b.c"), Some("a@b.c".into()));
        assert_eq!(to_human(k("underpay_tolerance_ton"), "10000000"), "0.01");
        assert_eq!(
            canonical(k("underpay_tolerance_ton"), "0.010").unwrap(),
            Some("0.01".into())
        );
        for (f, v) in [
            ("window_min", "9"),
            ("window_min", "241"),
            ("window_min", "ten"),
            ("grace_min", "4"),
            ("fuse_hours", "49"),
            ("max_renewals", "6"),
            ("renew_window_min", "6h"),
            ("underpay_tolerance_ton", "0.2"),
            ("underpay_tolerance_ton", "-1"),
            ("rate_min_usd", "-1"),
            ("rate_min_usd", "abc"),
            ("rate_max_spread_pct", "0.05"),
            ("rate_max_spread_pct", "21"),
            ("rate_max_age_min", "0"),
            ("rate_max_jump_pct", "0.5"),
            ("alert_email", "not-an-address"),
            ("ladder", "20,free"),
            ("free_per_guest_days", "91"),
        ] {
            let e = to_env(k(f), v).unwrap_err();
            assert!(e.0.starts_with(k(f).var), "{f}={v}: {e}");
            assert_eq!(key_of_error(&e), Some(k(f)));
        }
    }

    /// Cross-field rules run on the merged result: half a corridor from the
    /// database with nothing from the environment is refused, as it would
    /// be in the environment; strict checks catch a bad rate rule even
    /// without a corridor.
    #[test]
    fn the_whole_result_is_validated() {
        let mut db = DbValues::default();
        db.0.insert("CALRS_TON_RATE_MIN_USD".into(), row("0.3", Owner::Panel));
        let e = validate(&no_env, &db).unwrap_err();
        assert!(e.0.contains("go together"), "{e}");
        db.0.insert("CALRS_TON_RATE_MAX_USD".into(), row("0.2", Owner::Panel));
        assert!(validate(&no_env, &db).is_err());
        db.0.insert("CALRS_TON_RATE_MAX_USD".into(), row("5", Owner::Panel));
        let t = validate(&no_env, &db).unwrap();
        assert_eq!(t.rate_policy.unwrap().min_usd_micro, 300_000);
        let mut db = DbValues::default();
        db.0.insert(
            "CALRS_TON_RATE_MAX_SPREAD_PCT".into(),
            row("50", Owner::Panel),
        );
        assert!(validate(&no_env, &db).is_err());
        // The environment's own reading ignores it without a corridor.
        assert!(Tuning::parse(&Vars(|k: &str| lookup(&no_env, &db, k)), false).is_ok());
    }

    /// The test booking switch: env > DB > off, an unknown value off but
    /// still locked where it was set, and the one spelling stored.
    #[test]
    fn the_debug_switch_resolves_like_the_others() {
        let k = by_field("debug_booking").unwrap();
        assert_eq!(k.var, DEBUG_BOOKING);
        assert_eq!(k.default_text(), "off");
        let off = debug_booking(&no_env, &DbValues::default());
        assert_eq!((off.value, off.source), (false, Source::Default));
        let mut db = DbValues::default();
        db.set(DEBUG_BOOKING, "1".into(), Owner::Panel);
        let r = debug_booking(&no_env, &db);
        assert_eq!((r.value, r.source), (true, Source::Panel));
        db.set(DEBUG_BOOKING, "".into(), Owner::Apply);
        let r = debug_booking(&no_env, &db);
        assert_eq!((r.value, r.source), (false, Source::Apply));
        let env_off = |v: &str| (v == DEBUG_BOOKING).then(|| "0".to_string());
        db.set(DEBUG_BOOKING, "1".into(), Owner::Panel);
        let r = debug_booking(&env_off, &db);
        assert_eq!((r.value, r.source), (false, Source::Env), "env wins");
        let env_typo = |v: &str| (v == DEBUG_BOOKING).then(|| "enabled".to_string());
        let r = debug_booking(&env_typo, &db);
        assert_eq!((r.value, r.source), (false, Source::Env), "fails closed");
        let env_blank = |v: &str| (v == DEBUG_BOOKING).then(|| " ".to_string());
        assert_eq!(debug_booking(&env_blank, &db).source, Source::Panel);
        for (human, stored) in [
            ("on", "1"),
            (" TRUE ", "1"),
            ("1", "1"),
            ("off", "0"),
            ("No", "0"),
        ] {
            assert_eq!(
                to_env(k, human).unwrap().as_deref(),
                Some(stored),
                "{human}"
            );
        }
        assert_eq!(to_env(k, "").unwrap(), None);
        assert!(to_env(k, "2").unwrap_err().0.starts_with(DEBUG_BOOKING));
        assert_eq!(to_human(k, "1"), "on");
        assert_eq!(canonical(k, "yes").unwrap().as_deref(), Some("on"));
        // No TON parser reads it: the tuning is valid either way.
        assert!(validate(&no_env, &db).is_ok());
    }

    /// The prefilled guest Telegram username: env > DB > none, stored
    /// bare, checked like the booking form's field.
    #[test]
    fn the_debug_telegram_username_resolves_like_the_others() {
        let k = by_field("debug_booking_telegram").unwrap();
        assert_eq!(k.var, DEBUG_BOOKING_TELEGRAM);
        assert!(is_debug(k.var) && is_debug(DEBUG_BOOKING));
        assert!(!is_debug(cfg::WINDOW_MIN.var));
        assert_eq!(k.default_text(), "");
        let none = debug_booking_telegram(&no_env, &DbValues::default());
        assert_eq!((none.value, none.source), (None, Source::Default));
        for (human, stored) in [
            ("getmonolith", "getmonolith"),
            ("@getmonolith", "getmonolith"),
            (" https://t.me/getmonolith ", "getmonolith"),
        ] {
            assert_eq!(
                to_env(k, human).unwrap().as_deref(),
                Some(stored),
                "{human}"
            );
        }
        assert_eq!(to_env(k, " ").unwrap(), None);
        for bad in ["abc", "1abc", "get monolith", "getmonolith_", "@"] {
            let e = to_env(k, bad).unwrap_err();
            assert!(e.0.starts_with(DEBUG_BOOKING_TELEGRAM), "{bad}: {e}");
        }
        assert_eq!(
            canonical(k, "@getmonolith").unwrap().as_deref(),
            Some("getmonolith")
        );
        let mut db = DbValues::default();
        db.set(DEBUG_BOOKING_TELEGRAM, "getmonolith".into(), Owner::Apply);
        let r = debug_booking_telegram(&no_env, &db);
        assert_eq!(
            (r.value.as_deref(), r.source),
            (Some("getmonolith"), Source::Apply)
        );
        let env = |v: &str| (v == DEBUG_BOOKING_TELEGRAM).then(|| "@other_user".to_string());
        let r = debug_booking_telegram(&env, &db);
        assert_eq!(
            (r.value.as_deref(), r.source),
            (Some("other_user"), Source::Env)
        );
        let typo = |v: &str| (v == DEBUG_BOOKING_TELEGRAM).then(|| "no".to_string());
        let r = debug_booking_telegram(&typo, &db);
        assert_eq!((r.value, r.source), (None, Source::Env), "not a username");
        // No TON parser reads it.
        assert!(validate(&no_env, &db).is_ok());
    }

    #[test]
    fn defaults_and_preview() {
        let t = validate(&no_env, &DbValues::default()).unwrap();
        assert_eq!(t.ladder, Ladder::default());
        assert_eq!(t.free_per_guest_days, 30);
        assert_eq!(
            by_field("ladder").unwrap().default_text(),
            super::super::ladder::DEFAULT
        );
        assert_eq!(by_field("rate_max_spread_pct").unwrap().default_text(), "3");
        assert_eq!(
            by_field("underpay_tolerance_ton").unwrap().default_text(),
            "0"
        );
        let p = ladder_preview(&Ladder::parse("free,20,40").unwrap());
        assert_eq!(
            p,
            vec![
                (0, "free".to_string()),
                (1, "20".to_string()),
                (2, "40".to_string()),
                (3, "40".to_string())
            ]
        );
    }
}
