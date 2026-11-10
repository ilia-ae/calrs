//! The inbox's environment (docs/operations.md, «Почтовый ящик системы»).
//!
//! Off unless `CALRS_INBOX_IMAP_HOST`, `_USER` and `_PASSWORD` are all set.
//! A partial or invalid block logs one warning naming the variable (never a
//! value) and leaves the inbox off. Implicit TLS only: there is no plaintext
//! and no STARTTLS mode.

use std::path::PathBuf;
use std::time::Duration;

/// Everything the inbox reads from the environment.
pub(crate) const ENV_VARS: &[&str] = &[
    "CALRS_INBOX_IMAP_HOST",
    "CALRS_INBOX_IMAP_PORT",
    "CALRS_INBOX_IMAP_USER",
    "CALRS_INBOX_IMAP_PASSWORD",
    "CALRS_INBOX_IMAP_CA_FILE",
    "CALRS_INBOX_IMAP_CERT_SHA256",
    "CALRS_INBOX_ADDRESS",
    "CALRS_INBOX_PLUS",
    "CALRS_INBOX_POLL_MIN",
    "CALRS_INBOX_FOLDER",
    "CALRS_INBOX_DONE_FOLDER",
];

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Config {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) user: String,
    pub(crate) password: String,
    /// An extra PEM trust anchor: the CA that issued the server's
    /// certificate (`tls`).
    pub(crate) ca_file: Option<PathBuf>,
    /// Trust exactly the server certificate with this SHA-256 instead
    /// (Kerio's self-signed one, `tls`).
    pub(crate) cert_sha256: Option<[u8; 32]>,
    /// The system mailbox's address, lowercased (`booking@ilia.ae`).
    pub(crate) address: String,
    /// Reply-To `local+<token>@domain` (needs a `local+*` alias on Kerio).
    pub(crate) plus: bool,
    pub(crate) poll: Duration,
    pub(crate) folder: String,
    /// Where processed mail is moved; unset: it is only marked `\Seen`.
    pub(crate) done_folder: Option<String>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("user", &self.user)
            .field("password", &"<redacted>")
            .field("ca_file", &self.ca_file)
            .field("cert_sha256", &self.cert_sha256.map(hex::encode))
            .field("address", &self.address)
            .field("plus", &self.plus)
            .field("poll", &self.poll)
            .field("folder", &self.folder)
            .field("done_folder", &self.done_folder)
            .finish()
    }
}

impl Config {
    /// The local part and the domain of the address.
    pub(crate) fn parts(&self) -> (&str, &str) {
        self.address
            .split_once('@')
            .unwrap_or((&self.address, "invalid"))
    }

    #[cfg(test)]
    pub(crate) fn for_tests(address: &str, plus: bool) -> Self {
        Self {
            host: "imap.test".into(),
            port: 993,
            user: address.into(),
            password: "secret-pw".into(),
            ca_file: None,
            cert_sha256: None,
            address: address.to_ascii_lowercase(),
            plus,
            poll: Duration::from_secs(120),
            folder: "INBOX".into(),
            done_folder: None,
        }
    }
}

/// A plain `local@domain.tld`: ASCII letters, digits and `._-` in the local
/// part (no `+`: plus mode appends one), a dotted domain.
pub(crate) fn plain_address(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && local.len() <= 64
        && local
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        && domain.contains('.')
        && !domain.starts_with(['.', '-'])
        && !domain.ends_with(['.', '-'])
        && !domain.contains("..")
        && domain
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-'))
}

/// A SHA-256 fingerprint: 64 hex digits, with or without `:` or spaces
/// between them (as `openssl x509 -fingerprint -sha256` prints it).
pub(crate) fn parse_sha256(value: &str) -> Option<[u8; 32]> {
    let digits: String = value
        .chars()
        .filter(|c| !matches!(c, ':' | ' '))
        .collect::<String>()
        .to_ascii_lowercase();
    let digits = digits.strip_prefix("sha256fingerprint=").unwrap_or(&digits);
    let bytes = hex::decode(digits).ok()?;
    bytes.try_into().ok()
}

/// A folder name sent as an IMAP quoted string: printable ASCII without
/// `"` and `\` (a non-ASCII name would need modified UTF-7).
fn valid_folder(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .bytes()
            .all(|b| (0x20..0x7f).contains(&b) && b != b'"' && b != b'\\')
}

/// A host name or IP literal: no spaces, slashes or scheme.
fn valid_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b':' | b'[' | b']'))
}

/// Parse the environment through `env` (a lookup that returns non-empty
/// values only). `Ok(None)`: off and nothing set; `Err(var)`: set but
/// unusable, named by the variable that is wrong.
pub(crate) fn parse(env: impl Fn(&str) -> Option<String>) -> Result<Option<Config>, &'static str> {
    let get = |name: &str| {
        env(name)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    let host = get("CALRS_INBOX_IMAP_HOST");
    let user = get("CALRS_INBOX_IMAP_USER");
    // The password is taken as is: spaces may belong to it.
    let password = env("CALRS_INBOX_IMAP_PASSWORD").filter(|v| !v.is_empty());
    let (host, user, password) = match (host, user, password) {
        (None, None, None) => {
            if ENV_VARS.iter().any(|v| get(v).is_some()) {
                return Err("CALRS_INBOX_IMAP_HOST");
            }
            return Ok(None);
        }
        (Some(h), Some(u), Some(p)) => (h, u, p),
        (None, _, _) => return Err("CALRS_INBOX_IMAP_HOST"),
        (_, None, _) => return Err("CALRS_INBOX_IMAP_USER"),
        (_, _, None) => return Err("CALRS_INBOX_IMAP_PASSWORD"),
    };
    if !valid_host(&host) {
        return Err("CALRS_INBOX_IMAP_HOST");
    }
    if user.contains(['\r', '\n', '\0']) {
        return Err("CALRS_INBOX_IMAP_USER");
    }
    if password.contains(['\r', '\n', '\0']) {
        return Err("CALRS_INBOX_IMAP_PASSWORD");
    }
    let port = match get("CALRS_INBOX_IMAP_PORT") {
        None => 993,
        Some(v) => match v.parse::<u16>() {
            Ok(p) if p > 0 => p,
            _ => return Err("CALRS_INBOX_IMAP_PORT"),
        },
    };
    let address = get("CALRS_INBOX_ADDRESS")
        .or_else(|| user.contains('@').then(|| user.clone()))
        .or_else(|| get("CALRS_SMTP_FROM_EMAIL"))
        .map(|a| a.to_ascii_lowercase());
    let Some(address) = address.filter(|a| plain_address(a)) else {
        return Err("CALRS_INBOX_ADDRESS");
    };
    let plus = match get("CALRS_INBOX_PLUS").as_deref() {
        None | Some("0") | Some("false") | Some("no") | Some("off") => false,
        Some("1") | Some("true") | Some("yes") | Some("on") => true,
        Some(_) => return Err("CALRS_INBOX_PLUS"),
    };
    let poll = match get("CALRS_INBOX_POLL_MIN") {
        None => 2,
        Some(v) => match v.parse::<u64>() {
            Ok(m) if (1..=60).contains(&m) => m,
            _ => return Err("CALRS_INBOX_POLL_MIN"),
        },
    };
    let folder = get("CALRS_INBOX_FOLDER").unwrap_or_else(|| "INBOX".to_string());
    if !valid_folder(&folder) {
        return Err("CALRS_INBOX_FOLDER");
    }
    let done_folder = get("CALRS_INBOX_DONE_FOLDER");
    if done_folder
        .as_deref()
        .is_some_and(|d| !valid_folder(d) || d.eq_ignore_ascii_case(&folder))
    {
        return Err("CALRS_INBOX_DONE_FOLDER");
    }
    let ca_file = get("CALRS_INBOX_IMAP_CA_FILE").map(PathBuf::from);
    let cert_sha256 = match get("CALRS_INBOX_IMAP_CERT_SHA256") {
        None => None,
        // One way of trusting the server, not two.
        Some(_) if ca_file.is_some() => return Err("CALRS_INBOX_IMAP_CERT_SHA256"),
        Some(v) => match parse_sha256(&v) {
            Some(pin) => Some(pin),
            None => return Err("CALRS_INBOX_IMAP_CERT_SHA256"),
        },
    };
    Ok(Some(Config {
        host,
        port,
        user,
        password,
        ca_file,
        cert_sha256,
        address,
        plus,
        poll: Duration::from_secs(poll * 60),
        folder,
        done_folder,
    }))
}

/// The configuration from the environment, or `None` (off). An unusable
/// block warns once, naming the variable and no value.
#[cfg_attr(test, allow(dead_code))]
pub(crate) fn from_env() -> Option<Config> {
    match parse(|name| std::env::var(name).ok()) {
        Ok(config) => config,
        Err(var) => {
            tracing::warn!(target: "calrs::inbox", variable = var, "inbox configuration is incomplete or invalid; inbox off");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |name| map.get(name).cloned()
    }

    const BASE: &[(&str, &str)] = &[
        ("CALRS_INBOX_IMAP_HOST", "mail.ilia.ae"),
        ("CALRS_INBOX_IMAP_USER", "booking@ilia.ae"),
        ("CALRS_INBOX_IMAP_PASSWORD", " p w "),
    ];

    fn with(extra: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
        let mut all: Vec<_> = BASE.to_vec();
        all.extend_from_slice(extra);
        all
    }

    #[test]
    fn nothing_set_is_off_and_a_partial_block_names_what_is_missing() {
        assert_eq!(parse(env(&[])), Ok(None));
        assert_eq!(
            parse(env(&[("CALRS_INBOX_IMAP_HOST", "mail.ilia.ae")])),
            Err("CALRS_INBOX_IMAP_USER")
        );
        assert_eq!(
            parse(env(&[("CALRS_INBOX_PLUS", "1")])),
            Err("CALRS_INBOX_IMAP_HOST")
        );
        assert_eq!(parse(env(&BASE[..2])), Err("CALRS_INBOX_IMAP_PASSWORD"));
    }

    #[test]
    fn the_kerio_example_takes_its_defaults() {
        let c = parse(env(BASE)).unwrap().unwrap();
        assert_eq!(c.port, 993);
        assert_eq!(c.address, "booking@ilia.ae");
        assert_eq!(c.password, " p w ");
        assert!(!c.plus);
        assert_eq!(c.poll, Duration::from_secs(120));
        assert_eq!(c.folder, "INBOX");
        assert_eq!(c.done_folder, None);
        assert_eq!(c.parts(), ("booking", "ilia.ae"));
        assert!(!format!("{c:?}").contains("p w"));
    }

    #[test]
    fn each_value_is_checked() {
        for (var, bad) in [
            ("CALRS_INBOX_IMAP_PORT", "0"),
            ("CALRS_INBOX_IMAP_PORT", "70000"),
            ("CALRS_INBOX_POLL_MIN", "0"),
            ("CALRS_INBOX_POLL_MIN", "61"),
            ("CALRS_INBOX_PLUS", "maybe"),
            ("CALRS_INBOX_ADDRESS", "booking+x@ilia.ae"),
            ("CALRS_INBOX_ADDRESS", "not an address"),
            ("CALRS_INBOX_FOLDER", "Входящие"),
            ("CALRS_INBOX_DONE_FOLDER", "inbox"),
            ("CALRS_INBOX_IMAP_HOST", "https://mail.ilia.ae"),
        ] {
            assert_eq!(parse(env(&with(&[(var, bad)]))), Err(var), "{var}={bad}");
        }
        let c = parse(env(&with(&[
            ("CALRS_INBOX_IMAP_PORT", "19993"),
            ("CALRS_INBOX_PLUS", "1"),
            ("CALRS_INBOX_POLL_MIN", "60"),
            ("CALRS_INBOX_ADDRESS", "Booking@Ilia.ae"),
            ("CALRS_INBOX_DONE_FOLDER", "calrs/done"),
            ("CALRS_INBOX_IMAP_CA_FILE", "/etc/calrs/inbox-ca.pem"),
        ])))
        .unwrap()
        .unwrap();
        assert_eq!(c.port, 19993);
        assert!(c.plus);
        assert_eq!(c.address, "booking@ilia.ae");
        assert_eq!(c.done_folder.as_deref(), Some("calrs/done"));
        assert!(c.ca_file.is_some());
    }

    #[test]
    fn a_certificate_pin_is_64_hex_digits_and_excludes_a_ca_file() {
        let fingerprint = "AB:".repeat(31) + "CD";
        let c = parse(env(&with(&[(
            "CALRS_INBOX_IMAP_CERT_SHA256",
            Box::leak(fingerprint.clone().into_boxed_str()),
        )])))
        .unwrap()
        .unwrap();
        let mut want = [0xABu8; 32];
        want[31] = 0xCD;
        assert_eq!(c.cert_sha256, Some(want));
        assert_eq!(
            parse_sha256(&format!("sha256 Fingerprint={fingerprint}")),
            Some(want)
        );
        assert_eq!(parse_sha256(&"ab".repeat(32)), Some([0xAB; 32]));
        for bad in ["ab", "zz", &"ab".repeat(33)] {
            assert_eq!(parse_sha256(bad), None, "{bad}");
        }
        assert_eq!(
            parse(env(&with(&[("CALRS_INBOX_IMAP_CERT_SHA256", "not hex")]))),
            Err("CALRS_INBOX_IMAP_CERT_SHA256")
        );
        assert_eq!(
            parse(env(&with(&[
                ("CALRS_INBOX_IMAP_CA_FILE", "/etc/calrs/inbox-ca.pem"),
                (
                    "CALRS_INBOX_IMAP_CERT_SHA256",
                    Box::leak(fingerprint.into_boxed_str())
                ),
            ]))),
            Err("CALRS_INBOX_IMAP_CERT_SHA256")
        );
    }

    #[test]
    fn a_login_name_without_a_domain_uses_the_smtp_address() {
        let c = parse(env(&[
            ("CALRS_INBOX_IMAP_HOST", "mail.ilia.ae"),
            ("CALRS_INBOX_IMAP_USER", "booking"),
            ("CALRS_INBOX_IMAP_PASSWORD", "x"),
            ("CALRS_SMTP_FROM_EMAIL", "booking@ilia.ae"),
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(c.address, "booking@ilia.ae");
        assert_eq!(
            parse(env(&[
                ("CALRS_INBOX_IMAP_HOST", "mail.ilia.ae"),
                ("CALRS_INBOX_IMAP_USER", "booking"),
                ("CALRS_INBOX_IMAP_PASSWORD", "x"),
            ])),
            Err("CALRS_INBOX_ADDRESS")
        );
    }
}
