//! ilia fork: secrets from systemd credentials instead of the environment.
//!
//! The ilia-booking role keeps every secret out of `calrs.env`: each one is
//! encrypted on the server (`systemd-creds encrypt`, the host key and the TPM2
//! when there is one) and handed to `calrs serve` with
//! `LoadCredentialEncrypted=<NAME>:/etc/calrs/creds/<NAME>.cred`. systemd
//! decrypts it into `$CREDENTIALS_DIRECTORY/<NAME>`, a per-service file only
//! this service's user can read, and never puts the value in the unit's
//! environment (`systemctl show`) or in the process's initial environment
//! (`/proc/<pid>/environ`).
//!
//! The rest of calrs reads its configuration from environment variables, so
//! [`load_into_env`], called first thing in `main` while the process still has
//! one thread, sets the variable each credential replaces, in this process
//! only. A variable already set wins (an operator's explicit override, and the
//! tests). The credential name is the variable's name; only the names in
//! [`NAMES`] are read. Values are never logged: the report names variables.

use std::path::Path;
use zeroize::Zeroizing;

/// The variables that may come from a credential of the same name: every
/// secret calrs reads from the environment.
pub(crate) const NAMES: &[&str] = &[
    "CALRS_SMTP_PASSWORD",
    "CALRS_SMS_API_SECRET",
    "CALRS_TELEGRAM_BOT_TOKEN",
    "CALRS_INBOX_IMAP_PASSWORD",
    "CALRS_TON_API_KEY",
    "CALRS_TON_API_URL",
    "CALRS_TON_COINGECKO_KEY",
    "CALRS_THREECX_API_KEY",
    "CALRS_SECRET_KEY",
];

/// Credentials larger than this are refused (systemd's own cap is 1 MiB; no
/// secret of ours is anywhere near this).
const MAX_LEN: u64 = 64 * 1024;

/// What happened to one name. Holds no value, so it can be logged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// The credential was read and the variable set.
    Loaded,
    /// The variable was already set; the credential was left unread.
    AlreadySet,
    /// The credential exists but could not be used (the reason holds no value).
    Unusable(&'static str),
}

/// The result of one [`load_into_env`] call, for logging once tracing runs.
#[derive(Debug, Default)]
pub(crate) struct Report {
    /// `CREDENTIALS_DIRECTORY` was set but is not a readable directory.
    pub(crate) missing_dir: bool,
    pub(crate) names: Vec<(&'static str, Outcome)>,
}

/// ilia fork: the names [`load_into_env`] actually set from a credential
/// (`Outcome::Loaded`), for the TON settings page's "where the key comes
/// from". A variable already set (`AlreadySet`) is not here: its value is
/// the environment's, whatever credential file exists.
static LOADED: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();

/// The names the process's [`load_into_env`] loaded from credentials.
pub(crate) fn loaded_names() -> &'static [&'static str] {
    LOADED.get().map_or(&[], Vec::as_slice)
}

impl Report {
    /// The names whose value came from a credential.
    pub(crate) fn loaded(&self) -> Vec<&'static str> {
        self.names
            .iter()
            .filter(|(_, o)| *o == Outcome::Loaded)
            .map(|(n, _)| *n)
            .collect()
    }

    /// Logs what was loaded (names only). Silent outside systemd.
    pub(crate) fn log(&self) {
        if self.missing_dir {
            tracing::warn!(
                "CREDENTIALS_DIRECTORY is set but not readable: no systemd credentials loaded"
            );
        }
        let loaded = self.loaded();
        if !loaded.is_empty() {
            tracing::info!(names = %loaded.join(","), "secrets loaded from systemd credentials");
        }
        for (name, outcome) in &self.names {
            match outcome {
                Outcome::Loaded => {}
                Outcome::AlreadySet => tracing::warn!(
                    var = name,
                    "variable already set in the environment; its systemd credential is ignored"
                ),
                Outcome::Unusable(why) => {
                    tracing::warn!(var = name, reason = why, "systemd credential not used")
                }
            }
        }
    }
}

/// A credential's value, with one trailing newline removed (`echo value |
/// systemd-creds encrypt …` adds one). Not `Debug`, so it cannot be logged by
/// accident, and wiped on drop.
pub(crate) struct Value(Zeroizing<String>);

impl Value {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// Reads one credential file: `Ok(None)` when there is none.
fn read_one(dir: &Path, name: &str) -> Result<Option<Value>, &'static str> {
    let path = dir.join(name);
    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("cannot read the credential file"),
    };
    if !meta.is_file() {
        return Err("not a regular file");
    }
    if meta.len() > MAX_LEN {
        return Err("larger than 64 KiB");
    }
    let bytes =
        Zeroizing::new(std::fs::read(&path).map_err(|_| "cannot read the credential file")?);
    let text = std::str::from_utf8(&bytes).map_err(|_| "not valid UTF-8")?;
    if text.contains('\0') {
        return Err("contains a NUL byte");
    }
    let text = text.strip_suffix('\n').unwrap_or(text);
    Ok(Some(Value(Zeroizing::new(text.to_owned()))))
}

/// Reads the credentials in `dir` for `names` and decides each one, without
/// touching the environment: `is_set` says whether a variable is already set.
/// Pure apart from reading files, so the tests need no process-wide state.
pub(crate) fn collect(
    dir: &Path,
    names: &[&'static str],
    is_set: impl Fn(&str) -> bool,
) -> Vec<(&'static str, Result<Value, Outcome>)> {
    let mut out = Vec::new();
    for &name in names {
        match read_one(dir, name) {
            Ok(None) => {}
            Ok(Some(_)) if is_set(name) => out.push((name, Err(Outcome::AlreadySet))),
            Ok(Some(value)) => out.push((name, Ok(value))),
            Err(why) => out.push((name, Err(Outcome::Unusable(why)))),
        }
    }
    out
}

/// Sets the variables of `names` from `$CREDENTIALS_DIRECTORY`, when systemd
/// gave the process one.
///
/// Must run before any other thread exists: `std::env::set_var` is not safe
/// while another thread may read the environment, which is why `main` builds
/// the tokio runtime only after this.
pub(crate) fn load_into_env(names: &[&'static str]) -> Report {
    let mut report = Report::default();
    let Some(dir) = std::env::var_os("CREDENTIALS_DIRECTORY") else {
        return report;
    };
    let dir = Path::new(&dir);
    if !dir.is_dir() {
        report.missing_dir = true;
        return report;
    }
    for (name, decided) in collect(dir, names, |n| std::env::var_os(n).is_some()) {
        match decided {
            Ok(value) => {
                std::env::set_var(name, value.as_str());
                report.names.push((name, Outcome::Loaded));
            }
            Err(outcome) => report.names.push((name, outcome)),
        }
    }
    let _ = LOADED.set(report.loaded());
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dir(std::path::PathBuf);

    impl Dir {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!("calrs-creds-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&p).unwrap();
            Dir(p)
        }
        fn put(&self, name: &str, bytes: &[u8]) {
            std::fs::write(self.0.join(name), bytes).unwrap();
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn values(got: &[(&'static str, Result<Value, Outcome>)]) -> Vec<(&'static str, String)> {
        got.iter()
            .map(|(n, r)| {
                (
                    *n,
                    match r {
                        Ok(v) => format!("={}", v.as_str()),
                        Err(o) => format!("{o:?}"),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn reads_known_names_and_trims_one_newline() {
        let d = Dir::new();
        d.put("CALRS_SMTP_PASSWORD", b"p@ss $word\n\n");
        d.put("CALRS_TON_API_KEY", b"key-without-newline");
        d.put("CALRS_TELEGRAM_BOT_TOKEN", b"123:abc\r\n");
        d.put("NOT_A_KNOWN_NAME", b"ignored");
        let got = collect(&d.0, NAMES, |_| false);
        assert_eq!(
            values(&got),
            vec![
                ("CALRS_SMTP_PASSWORD", "=p@ss $word\n".to_string()),
                ("CALRS_TELEGRAM_BOT_TOKEN", "=123:abc\r".to_string()),
                ("CALRS_TON_API_KEY", "=key-without-newline".to_string()),
            ]
        );
    }

    #[test]
    fn a_variable_already_set_wins() {
        let d = Dir::new();
        d.put("CALRS_SMTP_PASSWORD", b"from-credential");
        let got = collect(&d.0, NAMES, |n| n == "CALRS_SMTP_PASSWORD");
        assert_eq!(
            values(&got),
            vec![("CALRS_SMTP_PASSWORD", "AlreadySet".to_string())]
        );
    }

    #[test]
    fn unusable_credentials_are_reported_without_their_value() {
        let d = Dir::new();
        d.put("CALRS_SMTP_PASSWORD", b"\xff\xfe-secret-bytes");
        d.put("CALRS_TON_API_KEY", b"abc\0def-secret");
        std::fs::create_dir(d.0.join("CALRS_TON_API_URL")).unwrap();
        d.put(
            "CALRS_TON_COINGECKO_KEY",
            "x".repeat(MAX_LEN as usize + 1).as_bytes(),
        );
        let got = collect(&d.0, NAMES, |_| false);
        let shown = format!("{:?}", values(&got));
        assert_eq!(
            values(&got),
            vec![
                (
                    "CALRS_SMTP_PASSWORD",
                    "Unusable(\"not valid UTF-8\")".to_string()
                ),
                (
                    "CALRS_TON_API_KEY",
                    "Unusable(\"contains a NUL byte\")".to_string()
                ),
                (
                    "CALRS_TON_API_URL",
                    "Unusable(\"not a regular file\")".to_string()
                ),
                (
                    "CALRS_TON_COINGECKO_KEY",
                    "Unusable(\"larger than 64 KiB\")".to_string()
                ),
            ]
        );
        assert!(!shown.contains("secret"));
    }

    #[test]
    fn the_report_holds_names_only() {
        let report = Report {
            missing_dir: false,
            names: vec![
                ("CALRS_SMTP_PASSWORD", Outcome::Loaded),
                ("CALRS_TON_API_KEY", Outcome::AlreadySet),
            ],
        };
        let shown = format!("{report:?}");
        assert!(shown.contains("CALRS_SMTP_PASSWORD"));
        assert!(!shown.contains('='));
    }

    #[test]
    fn every_name_is_a_secret_calrs_reads() {
        // A name here that calrs no longer reads would be a credential the role
        // encrypts for nothing; the role's list (vars/main.yml) mirrors this one.
        let sources = [
            include_str!("email/transport.rs"),
            include_str!("sms/mod.rs"),
            include_str!("telegram/config.rs"),
            include_str!("inbox/config.rs"),
            include_str!("ton/config.rs"),
            include_str!("web/threecx.rs"),
            include_str!("crypto.rs"),
        ]
        .concat();
        for name in NAMES {
            assert!(
                sources.contains(&format!("\"{name}\"")),
                "{name} is not read"
            );
        }
    }

    /// The one test that sets real variables: test-only names, under the
    /// shared lock, removed afterwards.
    #[tokio::test]
    async fn load_into_env_sets_missing_variables_only() {
        let _lock = crate::test_support::ENV_LOCK.lock().await;
        let d = Dir::new();
        d.put("CALRS_TEST_CRED_A", b"value-a\n");
        d.put("CALRS_TEST_CRED_B", b"from-credential");
        std::env::set_var("CALRS_TEST_CRED_B", "from-environment");
        let old = std::env::var_os("CREDENTIALS_DIRECTORY");
        std::env::set_var("CREDENTIALS_DIRECTORY", &d.0);
        let report = load_into_env(&[
            "CALRS_TEST_CRED_A",
            "CALRS_TEST_CRED_B",
            "CALRS_TEST_CRED_C",
        ]);
        let a = std::env::var("CALRS_TEST_CRED_A");
        let b = std::env::var("CALRS_TEST_CRED_B");
        let c = std::env::var_os("CALRS_TEST_CRED_C");
        std::env::set_var("CREDENTIALS_DIRECTORY", d.0.join("missing"));
        let missing = load_into_env(&["CALRS_TEST_CRED_A"]);
        match old {
            Some(v) => std::env::set_var("CREDENTIALS_DIRECTORY", v),
            None => std::env::remove_var("CREDENTIALS_DIRECTORY"),
        }
        for n in ["CALRS_TEST_CRED_A", "CALRS_TEST_CRED_B"] {
            std::env::remove_var(n);
        }
        assert_eq!(a.as_deref(), Ok("value-a"));
        assert_eq!(b.as_deref(), Ok("from-environment"));
        assert!(c.is_none());
        assert_eq!(
            report.names,
            vec![
                ("CALRS_TEST_CRED_A", Outcome::Loaded),
                ("CALRS_TEST_CRED_B", Outcome::AlreadySet),
            ]
        );
        assert!(!report.missing_dir);
        assert!(missing.missing_dir && missing.names.is_empty());
    }
}
