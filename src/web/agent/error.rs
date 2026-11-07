//! Tool errors: what the agent tells the model when a call fails. They come
//! back as `isError` results, so the model reads the reason and can fix its
//! arguments; protocol errors are kept for what rmcp itself rejects.
//!
//! A message names the problem and, at most, echoes input that was already
//! validated. It never carries database error text or row values: internal
//! failures are logged in full and the model gets a generic line.

use rmcp::model::{CallToolResult, ContentBlock, IntoContents};
use std::fmt;

#[derive(Debug)]
pub(crate) struct ToolError(String);

impl ToolError {
    /// An argument was rejected; `field` is its name in the input schema.
    pub(crate) fn invalid(field: &str, why: impl fmt::Display) -> Self {
        Self(format!("invalid {field}: {why}"))
    }

    pub(crate) fn not_found(what: &str) -> Self {
        Self(format!("{what} not found"))
    }

    pub(crate) fn unknown_tool() -> Self {
        Self("unknown tool".to_string())
    }

    pub(crate) fn denied(why: &str) -> Self {
        Self(why.to_string())
    }

    pub(crate) fn unavailable(why: &str) -> Self {
        Self(why.to_string())
    }

    pub(crate) fn limited() -> Self {
        Self("rate limit reached for this hour; try again later".to_string())
    }

    pub(crate) fn writes_limited() -> Self {
        Self(format!(
            "limit of {} changes an hour reached; try again later",
            super::WRITES_PER_HOUR
        ))
    }

    /// A cap of its own reached: `what` names the calls it counts.
    pub(crate) fn capped(what: &str, per_hour: u32) -> Self {
        Self(format!(
            "limit of {per_hour} {what} an hour reached; try again later"
        ))
    }

    /// Log `error` (target `calrs::agent`) with what was being done, and
    /// return a message that reveals neither.
    pub(crate) fn internal(context: &str, error: &dyn fmt::Display) -> Self {
        tracing::error!(target: "calrs::agent", context, %error, "agent tool failed");
        Self("internal error (details are in the server log)".to_string())
    }

    pub(crate) fn message(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_result(self) -> CallToolResult {
        CallToolResult::error(vec![ContentBlock::text(self.0)])
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Lets a tool return `Result<Json<T>, ToolError>`: rmcp turns the error into
/// a result with `isError: true` and this text.
impl IntoContents for ToolError {
    fn into_contents(self) -> Vec<ContentBlock> {
        vec![ContentBlock::text(self.0)]
    }
}

/// A database error inside a tool.
pub(crate) fn db(context: &'static str) -> impl FnOnce(sqlx::Error) -> ToolError {
    move |error| ToolError::internal(context, &error)
}
