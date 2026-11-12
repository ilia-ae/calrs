//! ilia fork: `config apply --export`, the stored state as a document.
//!
//! Each section module reads its own part (`<section>::export`), the same
//! readers its `verify` compares with, so applying an export changes
//! nothing. A stored value no document can state (written outside the
//! forms: an uppercase username, a custom theme without its colours, two
//! calendars of one name) leaves its key out, so the export still applies
//! as no change but does not show that value.
//! Tests: `apply_roundtrip.rs` (the whole document, a fresh and a
//! hand-edited database) and each section's own file.

use super::{event_types, profile, site, sources, ApplyError, Ctx, Document, FORMAT};

/// The admin's (`ctx.me`) stored state, every section present.
pub(super) async fn export(ctx: &Ctx) -> Result<Document, ApplyError> {
    Ok(Document {
        format: FORMAT.to_string(),
        user: ctx.me.email.clone(),
        profile: profile::export(ctx).await?,
        site: site::export(ctx).await?,
        sources: sources::export(ctx).await?,
        event_types: event_types::export(ctx).await?,
    })
}
