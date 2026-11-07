//! The write tools, offered to write connections only. Thin wrappers: each
//! checks the acting admin and calls `ops_write`, where the changes live.
//!
//! None reaches outside calrs: they change event types and the profile, and
//! send no email. `update_event_type` and `disable_event_type` ask the owner
//! on every call (`confirm_meta`): the first can change what guests see
//! (meeting link, description, slug, visibility), the second takes a booking
//! page down. The others are confirmed only by the client's `ask` rule (the
//! Claude Code settings of the calrs folder), though `set_weekly_hours` and
//! `add_date_override` can stop or narrow bookings too (blocked days).

use super::dto::*;
use super::error::ToolError;
use super::ops_write::{self, Writer};
use super::server::{confirm_meta, AgentServer};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, Json};

impl AgentServer {
    fn writer<'a>(&'a self, me: &'a crate::models::User) -> Writer<'a> {
        Writer {
            ctx: &self.ctx,
            session: &self.session,
            me,
        }
    }
}

#[tool_router(router = write_tools, vis = "pub(super)")]
impl AgentServer {
    #[tool(
        title = "Create event type",
        description = "Create a personal event type with its weekly hours, through the same \
                       checks as the web form. Settings not given start as the web form's new \
                       event type starts them (among others a reminder 24 hours before, 60 \
                       minutes' notice, no buffers); adjust them with update_event_type. \
                       Returns it as get_event_type does.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn create_event_type(
        &self,
        Parameters(args): Parameters<CreateEventTypeArgs>,
    ) -> Result<Json<EventTypeDetail>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(
            ops_write::create_event_type(&self.writer(&me), args).await?,
        ))
    }

    #[tool(
        title = "Update event type",
        description = "Change settings of one event type. Pass the version from \
                       get_event_type: if the event type changed since, nothing is saved. \
                       Only the settings in patch change; null clears a setting where its \
                       description says so. Weekly hours have their own tool. Some settings \
                       reach guests (meeting link, description, slug, visibility) or stop \
                       bookings (bookable dates, notice, horizon), so every call asks the \
                       owner to confirm: say what will change first. Returns the names of the \
                       settings that changed and the event type with its new version.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        ),
        meta = confirm_meta()
    )]
    async fn update_event_type(
        &self,
        Parameters(args): Parameters<UpdateEventTypeArgs>,
    ) -> Result<Json<EventTypeChanged>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(
            ops_write::update_event_type(&self.writer(&me), args).await?,
        ))
    }

    #[tool(
        title = "Set weekly hours",
        description = "Replace the whole week of an event type's bookable hours: each day \
                       with its windows (HH:MM, at most 10 a day, not overlapping); days left \
                       out have no hours. An empty week is refused. Pass the version from \
                       get_event_type. When timezone is not the event type's timezone, the \
                       event type moves to it.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn set_weekly_hours(
        &self,
        Parameters(args): Parameters<WeeklyHoursArgs>,
    ) -> Result<Json<EventTypeChanged>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(
            ops_write::set_weekly_hours(&self.writer(&me), args).await?,
        ))
    }

    #[tool(
        title = "Add date override",
        description = "Give one date of an event type other hours than the week's: blocked \
                       (no bookings that day) or custom_hours (start to end only). The date \
                       is in the event type's timezone, from today on. Returns the override \
                       with its id.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn add_date_override(
        &self,
        Parameters(args): Parameters<AddOverrideArgs>,
    ) -> Result<Json<OverrideAdded>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(
            ops_write::add_date_override(&self.writer(&me), args).await?,
        ))
    }

    #[tool(
        title = "Remove date override",
        description = "Remove one date override of an event type by its id (from \
                       get_event_type); that date has the week's hours again.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn remove_date_override(
        &self,
        Parameters(args): Parameters<RemoveOverrideArgs>,
    ) -> Result<Json<OverrideRemoved>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(
            ops_write::remove_date_override(&self.writer(&me), args).await?,
        ))
    }

    #[tool(
        title = "Enable event type",
        description = "Switch an event type on, so guests can book it. Changes nothing if it \
                       is on already.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn enable_event_type(
        &self,
        Parameters(args): Parameters<SlugArgs>,
    ) -> Result<Json<EnabledOut>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(
            ops_write::set_enabled(&self.writer(&me), "enable_event_type", &args.slug, true)
                .await?,
        ))
    }

    #[tool(
        title = "Disable event type",
        description = "Switch an event type off: guests can no longer book it. Existing \
                       bookings stay as they are. Changes nothing if it is off already. Asks \
                       the owner to confirm.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        ),
        meta = confirm_meta()
    )]
    async fn disable_event_type(
        &self,
        Parameters(args): Parameters<SlugArgs>,
    ) -> Result<Json<EnabledOut>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(
            ops_write::set_enabled(&self.writer(&me), "disable_event_type", &args.slug, false)
                .await?,
        ))
    }

    #[tool(
        title = "Set profile",
        description = "Change the acting account's display name, timezone or language. \
                       Event types without a timezone of their own keep their clock times \
                       in the new timezone, so their hours move: the result lists them.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn set_profile(
        &self,
        Parameters(args): Parameters<ProfileArgs>,
    ) -> Result<Json<ProfileOut>, ToolError> {
        let me = self.acting().await?;
        Ok(Json(ops_write::set_profile(&self.writer(&me), args).await?))
    }
}
