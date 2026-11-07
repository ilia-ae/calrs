//! The read tools, offered to both roles. Thin wrappers: each checks the
//! acting admin and calls `ops_read`, where the queries live.
//!
//! All of them are read-only and idempotent. Only `get_slots` reaches outside
//! calrs (it may sync a stale calendar source first, as the booking page
//! does), so it alone is marked open-world.

use super::dto::*;
use super::error::ToolError;
use super::ops_read::{self, PageReq};
use super::server::AgentServer;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, Json};

#[tool_router(router = read_tools, vis = "pub(super)")]
impl AgentServer {
    #[tool(
        title = "Who am I",
        description = "The calrs account these tools act as, this connection's role \
                       (read_only or write) and the server's limits.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn whoami(&self, _args: Parameters<NoArgs>) -> Result<Json<WhoAmI>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(ops_read::whoami(&self.ctx, &self.session, &me)))
    }

    #[tool(
        title = "Overview",
        description = "Counts at a glance: event types, pending and upcoming bookings with \
                       the next start, calendar sources and failing syncs, failed calendar \
                       writes. No guest data.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_overview(&self, _args: Parameters<NoArgs>) -> Result<Json<Overview>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(ops_read::overview(&self.ctx.state.pool, &me).await?))
    }

    #[tool(
        title = "Server settings",
        description = "Server-wide settings: version, public base URL, whether registration, \
                       MFA, OIDC and captcha are on, and whether SMTP and SMS are configured \
                       and switched on. Only flags, no secrets.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_settings(
        &self,
        _args: Parameters<NoArgs>,
    ) -> Result<Json<SettingsOut>, ToolError> {
        self.acting().await?;
        Ok(Json(ops_read::settings(&self.ctx.state).await?))
    }

    #[tool(
        title = "List event types",
        description = "The acting account's personal event types (bookable meeting kinds), \
                       oldest first, with slug, title, duration, visibility, timezone and \
                       active bookings. Paginated.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn list_event_types(
        &self,
        Parameters(args): Parameters<PageArgs>,
    ) -> Result<Json<Page<EventTypeSummary>>, ToolError> {
        let me = self.acting().await?;
        let page = PageReq::parse(args.cursor.as_deref(), args.limit)?;
        Ok(Json(
            ops_read::list_event_types(&self.ctx.state.pool, &me, page).await?,
        ))
    }

    #[tool(
        title = "Get event type",
        description = "Full configuration of one event type by slug: durations, buffers, \
                       notice, confirmation, location, timezone, booking window, weekly \
                       hours, date overrides, the calendars checked for conflicts, frequency \
                       limits, and a version token for later changes.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_event_type(
        &self,
        Parameters(args): Parameters<SlugArgs>,
    ) -> Result<Json<EventTypeDetail>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(
            ops_read::get_event_type(&self.ctx.state.pool, &me, &args.slug).await?,
        ))
    }

    #[tool(
        title = "Free slots",
        description = "Start times a guest could book for one event type, per day, as the \
                       booking page computes them (may refresh a stale calendar first). \
                       Disabled event types are computed too, as a preview.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_slots(
        &self,
        Parameters(args): Parameters<SlotsArgs>,
    ) -> Result<Json<SlotsOut>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(ops_read::get_slots(&self.ctx.state, &me, args).await?))
    }

    #[tool(
        title = "List calendar sources",
        description = "Connected calendar sources with their sync state, which calendar \
                       receives bookings, and each calendar's id and whether it is synced. \
                       Calendar names are calendar_supplied data. Paginated.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn list_sources(
        &self,
        Parameters(args): Parameters<PageArgs>,
    ) -> Result<Json<Page<SourceOut>>, ToolError> {
        let me = self.acting().await?;
        let page = PageReq::parse(args.cursor.as_deref(), args.limit)?;
        Ok(Json(
            ops_read::list_sources(&self.ctx.state.pool, &me, page).await?,
        ))
    }

    #[tool(
        title = "List bookings",
        description = "Bookings of the acting account's event types: pending requests, \
                       upcoming confirmed ones (default), or the last 90 days. Times are \
                       RFC 3339; guest name, masked email and timezone are guest_supplied \
                       data. Paginated.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn list_bookings(
        &self,
        Parameters(args): Parameters<BookingsArgs>,
    ) -> Result<Json<Page<BookingSummary>>, ToolError> {
        let me = self.acting().await?;
        let page = PageReq::parse(args.cursor.as_deref(), args.limit)?;
        let filter = args.filter.unwrap_or_default();
        Ok(Json(
            ops_read::list_bookings(&self.ctx.state.pool, &me, filter, page).await?,
        ))
    }
}
