//! Agent configuration from the environment of `calrs serve`: which system
//! users may connect and with which role, the admin account the agent acts as,
//! and the session timeouts. Everything is checked at startup and the agent
//! fails closed: an invalid value means no connection is served.

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

/// What a connection may do, decided by serve from the peer's uid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    /// Read tools only; write tools are neither listed nor callable.
    ReadOnly,
    /// Every tool.
    Write,
}

impl Role {
    pub(crate) fn can_write(self) -> bool {
        self == Role::Write
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Role::ReadOnly => "read_only",
            Role::Write => "write",
        })
    }
}

/// Peer uid to (system user name, role).
#[derive(Clone, Debug, Default)]
pub(crate) struct RoleMap(BTreeMap<u32, (String, Role)>);

impl RoleMap {
    pub(crate) fn insert(&mut self, uid: u32, user: &str, role: Role) {
        self.0.insert(uid, (user.to_string(), role));
    }

    /// The user name and role for `uid`, if it may connect at all.
    pub(crate) fn get(&self, uid: u32) -> Option<(&str, Role)> {
        self.0.get(&uid).map(|(user, role)| (user.as_str(), *role))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// `calrs-mcp(1001)=write, calrs-mcp-ro(1002)=read_only`, for the startup log.
impl fmt::Display for RoleMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (uid, (user, role))) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{user}({uid})={role}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AgentConfig {
    pub(crate) roles: RoleMap,
    /// Email of the calrs admin account every tool acts as.
    pub(crate) admin_email: String,
    /// A session with no incoming message and no running call for this long
    /// is closed.
    pub(crate) idle_timeout: Duration,
    /// A session is closed after this long whatever it is doing.
    pub(crate) max_session: Duration,
    pub(crate) handshake_timeout: Duration,
    /// A session whose peer has not read a waiting answer for this long is
    /// closed.
    pub(crate) write_stall_timeout: Duration,
}

/// Why the configuration was refused. Names the variable and the problem;
/// carries system user names at most, never other values.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ConfigError {
    pub(crate) var: &'static str,
    pub(crate) reason: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.var, self.reason)
    }
}

impl std::error::Error for ConfigError {}

pub(crate) const USERS_VAR: &str = "CALRS_AGENT_USERS";
pub(crate) const ADMIN_EMAIL_VAR: &str = "CALRS_AGENT_ADMIN_EMAIL";
pub(crate) const IDLE_VAR: &str = "CALRS_AGENT_IDLE_TIMEOUT_SECS";
pub(crate) const MAX_SESSION_VAR: &str = "CALRS_AGENT_MAX_SESSION_SECS";

const IDLE_DEFAULT: u64 = 1800;
const IDLE_RANGE: (u64, u64) = (60, 86_400);
const MAX_SESSION_DEFAULT: u64 = 28_800;
const MAX_SESSION_RANGE: (u64, u64) = (300, 604_800);

impl AgentConfig {
    /// Read the configuration through `env` (production passes
    /// `std::env::var`), resolving user names against `passwd`, the text of
    /// `/etc/passwd`.
    pub(crate) fn from_lookup(
        env: impl Fn(&str) -> Option<String>,
        passwd: &str,
    ) -> Result<Self, ConfigError> {
        let roles = parse_roles(env(USERS_VAR).as_deref().unwrap_or(""), passwd)?;
        let admin_email = env(ADMIN_EMAIL_VAR)
            .map(|v| v.trim().to_string())
            .unwrap_or_default();
        if admin_email.is_empty() {
            return Err(err(ADMIN_EMAIL_VAR, "must be set"));
        }
        if !admin_email.contains('@') || admin_email.chars().any(char::is_whitespace) {
            return Err(err(ADMIN_EMAIL_VAR, "is not an email address"));
        }
        Ok(Self {
            roles,
            admin_email,
            idle_timeout: seconds(&env, IDLE_VAR, IDLE_DEFAULT, IDLE_RANGE)?,
            max_session: seconds(
                &env,
                MAX_SESSION_VAR,
                MAX_SESSION_DEFAULT,
                MAX_SESSION_RANGE,
            )?,
            handshake_timeout: super::HANDSHAKE_TIMEOUT,
            write_stall_timeout: super::WRITE_STALL_TIMEOUT,
        })
    }
}

fn err(var: &'static str, reason: impl Into<String>) -> ConfigError {
    ConfigError {
        var,
        reason: reason.into(),
    }
}

/// Parse `user=role,user=role`. Fails closed on anything unexpected: an
/// empty list, a malformed entry, an unknown role, a name listed twice, a user
/// missing from `passwd`, uid 0, or two names sharing a uid.
fn parse_roles(raw: &str, passwd: &str) -> Result<RoleMap, ConfigError> {
    let mut map = RoleMap::default();
    let mut seen = BTreeMap::<String, u32>::new();
    for (i, entry) in raw.split(',').map(str::trim).enumerate() {
        if entry.is_empty() {
            continue;
        }
        let n = i + 1;
        let (user, role) = entry
            .split_once('=')
            .ok_or_else(|| err(USERS_VAR, format!("entry {n} is not user=role")))?;
        let (user, role) = (user.trim(), role.trim());
        if !valid_user_name(user) {
            return Err(err(
                USERS_VAR,
                format!("entry {n}: not a valid system user name"),
            ));
        }
        let role = match role {
            "read" => Role::ReadOnly,
            "write" => Role::Write,
            _ => {
                return Err(err(
                    USERS_VAR,
                    format!("entry {n}: role must be read or write"),
                ))
            }
        };
        if seen.contains_key(user) {
            return Err(err(USERS_VAR, format!("user {user} is listed twice")));
        }
        let uid = uid_of(passwd, user)
            .ok_or_else(|| err(USERS_VAR, format!("user {user} not found in /etc/passwd")))?;
        if uid == 0 {
            return Err(err(USERS_VAR, format!("user {user} has uid 0")));
        }
        if let Some((other, _)) = map.get(uid) {
            return Err(err(
                USERS_VAR,
                format!("users {other} and {user} share uid {uid}"),
            ));
        }
        seen.insert(user.to_string(), uid);
        map.insert(uid, user, role);
    }
    if map.is_empty() {
        return Err(err(USERS_VAR, "must list at least one user=role"));
    }
    Ok(map)
}

/// `^[a-z_][a-z0-9_-]{0,31}$`, the portable shape of a system user name.
fn valid_user_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    name.len() <= 32
        && (first.is_ascii_lowercase() || first == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// The uid of `name` in `passwd` (the `/etc/passwd` format). Local users only:
/// NSS sources such as LDAP are not consulted, so a user that exists only
/// there is "not found" and the configuration fails closed.
pub(crate) fn uid_of(passwd: &str, name: &str) -> Option<u32> {
    passwd
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .find_map(|line| {
            let mut fields = line.split(':');
            let user = fields.next()?;
            let _password = fields.next()?;
            let uid = fields.next()?;
            (user == name).then(|| uid.parse().ok()).flatten()
        })
}

fn seconds(
    env: &impl Fn(&str) -> Option<String>,
    var: &'static str,
    default: u64,
    (min, max): (u64, u64),
) -> Result<Duration, ConfigError> {
    let Some(raw) = env(var)
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
    else {
        return Ok(Duration::from_secs(default));
    };
    match raw.parse::<u64>() {
        Ok(secs) if (min..=max).contains(&secs) => Ok(Duration::from_secs(secs)),
        _ => Err(err(
            var,
            format!("must be a whole number of seconds from {min} to {max}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const PASSWD: &str = "\
root:x:0:0:root:/root:/bin/bash
# comment line
daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin
calrs:x:998:998::/var/lib/calrs:/usr/sbin/nologin
calrs-mcp:x:1001:1001::/nonexistent:/bin/sh
calrs-mcp-ro:x:1002:1002::/nonexistent:/bin/sh
twin:x:1001:1001::/nonexistent:/bin/sh
broken-line
";

    fn config(vars: &[(&str, &str)]) -> Result<AgentConfig, ConfigError> {
        let vars: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        AgentConfig::from_lookup(|k| vars.get(k).cloned(), PASSWD)
    }

    fn base(users: &str) -> Vec<(&'static str, &str)> {
        vec![(USERS_VAR, users), (ADMIN_EMAIL_VAR, "owner@example.test")]
    }

    #[test]
    fn uid_of_reads_passwd_lines() {
        assert_eq!(uid_of(PASSWD, "calrs-mcp"), Some(1001));
        assert_eq!(uid_of(PASSWD, "calrs-mcp-ro"), Some(1002));
        assert_eq!(uid_of(PASSWD, "root"), Some(0));
        assert_eq!(uid_of(PASSWD, "nobody"), None);
        assert_eq!(uid_of(PASSWD, "broken-line"), None);
        assert_eq!(uid_of(PASSWD, "calrs-mc"), None);
        assert_eq!(uid_of(PASSWD, "# comment line"), None);
    }

    #[test]
    fn role_map_parses_users_and_roles() {
        let c = config(&base("calrs-mcp=write, calrs-mcp-ro=read")).unwrap();
        assert_eq!(c.roles.get(1001), Some(("calrs-mcp", Role::Write)));
        assert_eq!(c.roles.get(1002), Some(("calrs-mcp-ro", Role::ReadOnly)));
        assert_eq!(c.roles.get(998), None);
        assert_eq!(c.roles.get(0), None);
        assert_eq!(
            c.roles.to_string(),
            "calrs-mcp(1001)=write, calrs-mcp-ro(1002)=read_only"
        );
        assert_eq!(c.admin_email, "owner@example.test");
        assert_eq!(c.idle_timeout, Duration::from_secs(1800));
        assert_eq!(c.max_session, Duration::from_secs(28_800));
        assert_eq!(c.handshake_timeout, super::super::HANDSHAKE_TIMEOUT);
        assert_eq!(c.write_stall_timeout, super::super::WRITE_STALL_TIMEOUT);
    }

    #[test]
    fn role_map_fails_closed() {
        for (users, needle) in [
            ("", "at least one"),
            (" , ", "at least one"),
            ("calrs-mcp", "not user=role"),
            ("calrs-mcp=admin", "read or write"),
            ("calrs-mcp=", "read or write"),
            ("Calrs=read", "valid system user name"),
            ("=read", "valid system user name"),
            ("calrs mcp=read", "valid system user name"),
            ("nobody=read", "not found"),
            ("root=write", "uid 0"),
            ("calrs-mcp=read,calrs-mcp=write", "listed twice"),
            ("calrs-mcp=read,twin=read", "share uid 1001"),
        ] {
            let error = config(&base(users)).unwrap_err();
            assert_eq!(error.var, USERS_VAR, "{users}");
            assert!(error.reason.contains(needle), "{users}: {error}");
        }
        let missing = AgentConfig::from_lookup(|_| None, PASSWD).unwrap_err();
        assert_eq!(missing.var, USERS_VAR);
    }

    #[test]
    fn admin_email_is_required() {
        let error = config(&[(USERS_VAR, "calrs-mcp=write")]).unwrap_err();
        assert_eq!(error.var, ADMIN_EMAIL_VAR);
        for bad in ["", "   ", "owner", "own er@example.test"] {
            let error =
                config(&[(USERS_VAR, "calrs-mcp=write"), (ADMIN_EMAIL_VAR, bad)]).unwrap_err();
            assert_eq!(error.var, ADMIN_EMAIL_VAR, "{bad:?}");
        }
    }

    #[test]
    fn timeouts_have_defaults_and_ranges() {
        let mut vars = base("calrs-mcp=write");
        vars.push((IDLE_VAR, "60"));
        vars.push((MAX_SESSION_VAR, "604800"));
        let c = config(&vars).unwrap();
        assert_eq!(c.idle_timeout, Duration::from_secs(60));
        assert_eq!(c.max_session, Duration::from_secs(604_800));

        for (var, value) in [
            (IDLE_VAR, "59"),
            (IDLE_VAR, "86401"),
            (IDLE_VAR, "ten"),
            (IDLE_VAR, "-5"),
            (MAX_SESSION_VAR, "299"),
            (MAX_SESSION_VAR, "604801"),
            (MAX_SESSION_VAR, "1.5"),
        ] {
            let mut vars = base("calrs-mcp=write");
            vars.push((var, value));
            let error = config(&vars).unwrap_err();
            assert_eq!(error.var, var, "{var}={value}");
        }
    }

    #[test]
    fn errors_name_the_variable_only() {
        let error = config(&base("calrs-mcp=root-secret")).unwrap_err();
        assert!(!error.to_string().contains("root-secret"), "{error}");
    }
}
