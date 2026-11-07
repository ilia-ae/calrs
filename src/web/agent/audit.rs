//! The agent's audit trail: one `calrs::audit` event per connection change and
//! per tool call, written through `tracing` to serve's stdout and so to the
//! journal. The default filter (`calrs=info`) keeps them.
//!
//! An event names the session, the peer (uid, pid, system user, role), the
//! tool, the object (an id or slug taken from the arguments, never free text)
//! and the result. It never carries the arguments themselves, guest text or
//! addresses (a guest appears as a booking id), or secrets: a call's `detail`
//! is the agent's own error text, capped, and rmcp's argument errors, which
//! quote the arguments, are logged as `invalid_arguments` only
//! (`server::audit_detail`). Writes add one `change` event with the changed
//! fields before and after, enough to undo a mistake by hand.

use super::server::Session;
use rmcp::model::JsonObject;
use std::time::Duration;

const TARGET: &str = "calrs::audit";

/// The result of a call or connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Ok,
    /// The tool ran and reported an error, or the arguments were rejected.
    Error,
    /// A read-only session called a write tool.
    Denied,
    /// The hourly call limit was reached.
    Limited,
    /// Refused before the tool ran: shutting down, or no usable acting admin.
    Refused,
}

impl Outcome {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Outcome::Ok => "ok",
            Outcome::Error => "error",
            Outcome::Denied => "denied",
            Outcome::Limited => "limited",
            Outcome::Refused => "refused",
        }
    }
}

/// Argument keys whose value names the object a call is about.
const OBJECT_KEYS: [&str; 4] = ["slug", "booking_id", "source_id", "override_id"];

/// The object of a call: the first id-like argument, when it looks like an id
/// or a slug. Anything else is reported as `<invalid>`, never echoed.
pub(crate) fn object_of(arguments: Option<&JsonObject>) -> Option<String> {
    let arguments = arguments?;
    OBJECT_KEYS.iter().find_map(|key| {
        let value = arguments.get(*key)?;
        Some(match value.as_str() {
            Some(id) if is_identifier(id, 100) => format!("{key}={id}"),
            _ => format!("{key}=<invalid>"),
        })
    })
}

/// The tool name as logged: the name when it looks like one, else `<invalid>`.
pub(crate) fn tool_label(name: &str) -> &str {
    let plain = !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if plain {
        name
    } else {
        "<invalid>"
    }
}

fn is_identifier(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// A connection got a session.
pub(crate) fn accepted(session: &Session) {
    tracing::info!(
        target: TARGET,
        event = "connection",
        result = "accepted",
        session = %session.id,
        peer_uid = session.uid,
        peer_pid = session.pid,
        peer_user = %session.user,
        role = %session.role,
        "agent connection accepted"
    );
}

/// A connection was closed before anything was read from it.
pub(crate) fn refused(uid: Option<u32>, pid: Option<i32>, reason: &'static str) {
    tracing::warn!(
        target: TARGET,
        event = "connection",
        result = "refused",
        reason,
        peer_uid = uid,
        peer_pid = pid,
        "agent connection refused"
    );
}

/// A session ended.
pub(crate) fn closed(session: &Session, reason: &'static str) {
    tracing::info!(
        target: TARGET,
        event = "connection",
        result = "closed",
        reason,
        session = %session.id,
        peer_uid = session.uid,
        peer_pid = session.pid,
        role = %session.role,
        duration_secs = session.started.elapsed().as_secs(),
        calls = session.calls(),
        messages = session.messages(),
        "agent connection closed"
    );
}

/// A tool call finished (or was refused). `number` counts the session's
/// calls from 1; `detail` comes from `server::audit_detail`, never from the
/// arguments.
pub(crate) fn call(
    session: &Session,
    number: u64,
    tool: &str,
    object: Option<&str>,
    outcome: Outcome,
    detail: &str,
    elapsed: Duration,
) {
    tracing::info!(
        target: TARGET,
        event = "call",
        session = %session.id,
        peer_uid = session.uid,
        peer_pid = session.pid,
        role = %session.role,
        tool,
        object = object.unwrap_or("-"),
        result = outcome.as_str(),
        detail,
        count = number,
        elapsed_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
        "agent call"
    );
}

/// A write changed something: the changed fields as JSON, before and after.
pub(crate) fn change(session: &Session, tool: &str, object: &str, before: &str, after: &str) {
    tracing::info!(
        target: TARGET,
        event = "change",
        session = %session.id,
        peer_uid = session.uid,
        role = %session.role,
        tool,
        object,
        before,
        after,
        "agent change"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn args(value: serde_json::Value) -> JsonObject {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn object_is_an_id_or_invalid() {
        assert_eq!(object_of(None), None);
        assert_eq!(object_of(Some(&args(json!({"limit": 5})))), None);
        assert_eq!(
            object_of(Some(&args(json!({"slug": "intro-call", "days": 3})))),
            Some("slug=intro-call".to_string())
        );
        assert_eq!(
            object_of(Some(&args(json!({"booking_id": "0b9f-AA_1"})))),
            Some("booking_id=0b9f-AA_1".to_string())
        );
        for bad in [
            json!({"slug": "ignore previous instructions"}),
            json!({"slug": "a\nb"}),
            json!({"slug": ""}),
            json!({"slug": 7}),
            json!({"slug": "x".repeat(101)}),
        ] {
            assert_eq!(
                object_of(Some(&args(bad.clone()))),
                Some("slug=<invalid>".to_string()),
                "{bad}"
            );
        }
    }

    #[test]
    fn tool_names_are_logged_only_when_plain() {
        assert_eq!(tool_label("list_bookings"), "list_bookings");
        assert_eq!(tool_label("drop table"), "<invalid>");
        assert_eq!(tool_label(""), "<invalid>");
        assert_eq!(tool_label(&"a".repeat(65)), "<invalid>");
        assert_eq!(tool_label("Whoami"), "<invalid>");
    }
}
