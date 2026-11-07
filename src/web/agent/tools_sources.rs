//! The calendar tools, offered to write connections only: which calendars of
//! a source are synced, a sync on demand, and the calendar new bookings are
//! written to. Thin wrappers around `ops_sources`; the changes run to their
//! end in a task of their own (`AgentServer::run_to_end`).
//!
//! Choosing the write calendar decides where every new booking goes, so
//! `set_write_calendar` asks the owner first (`confirm_meta`). Sources
//! themselves (address, account, password) cannot be changed here.

use super::dto::*;
use super::error::ToolError;
use super::ops_sources;
use super::server::{confirm_meta, AgentServer};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, Json};

#[tool_router(router = source_tools, vis = "pub(super)")]
impl AgentServer {
    #[tool(
        title = "Set source calendars",
        description = "Choose which calendars of one calendar source are synced (ids from \
                       list_sources), at least one. Every other calendar of the source stops \
                       being synced: its events are dropped and no longer block times. \
                       Returns the source as list_sources shows it.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn set_source_calendars(
        &self,
        Parameters(args): Parameters<SourceCalendarsArgs>,
    ) -> Result<Json<SourceChanged>, ToolError> {
        let actor = self.actor(self.acting().await?);
        self.run_to_end(
            async move { ops_sources::set_source_calendars(&actor.writer(), args).await },
        )
        .await
        .map(Json)
    }

    #[tool(
        title = "Sync source",
        description = "Fetch one calendar source's events from its server now; at most once \
                       in 5 minutes per source. Returns whether the sync succeeded and the \
                       source as list_sources shows it; the server's error text is not shown.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn sync_source(
        &self,
        Parameters(args): Parameters<SourceArgs>,
    ) -> Result<Json<SourceSynced>, ToolError> {
        let actor = self.actor(self.acting().await?);
        self.run_to_end(
            async move { ops_sources::sync_source(&actor.writer(), &args.source_id).await },
        )
        .await
        .map(Json)
    }

    #[tool(
        title = "Set write calendar",
        description = "Choose the calendar of one source that new bookings are written to, by \
                       its id from list_sources, or null to write none to this source. Asks \
                       the owner to confirm. Returns the source as list_sources shows it.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        ),
        meta = confirm_meta()
    )]
    async fn set_write_calendar(
        &self,
        Parameters(args): Parameters<WriteCalendarArgs>,
    ) -> Result<Json<SourceChanged>, ToolError> {
        let actor = self.actor(self.acting().await?);
        self.run_to_end(async move { ops_sources::set_write_calendar(&actor.writer(), args).await })
            .await
            .map(Json)
    }
}
