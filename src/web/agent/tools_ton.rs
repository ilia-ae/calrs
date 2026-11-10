//! ilia fork: the TON payment tools (ilia-booking docs/ton.md), offered to
//! write connections only. Thin wrappers around `ops_ton`.
//!
//! Both ask the owner on every call (`confirm_meta`): a price charges guests
//! money, and waiving a payment gives a booking away free. `waive_payment`
//! takes the amount given up as an argument, so the confirmation prompt
//! shows it.

use super::dto::*;
use super::error::ToolError;
use super::ops_ton;
use super::server::{confirm_meta, AgentServer};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, Json};

#[tool_router(router = ton_tools, vis = "pub(super)")]
impl AgentServer {
    #[tool(
        title = "Set event type price",
        description = "Charge guests for one personal event type, in TON: price_ton in TON \
                       (the default, 1 to 100000; the guest pays exactly that), or price_usd \
                       in US dollars (1 to 10000), converted at the rate of the moment a guest \
                       books. The booking holds the time while the guest pays and is \
                       confirmed by itself once the payment is final; an unpaid window of a \
                       far meeting is renewed, else the time is released. Neither price \
                       removes it (the type books free again). A new price makes the booking \
                       form ask for the guest's Telegram. Pass the version from \
                       get_event_type. Bookings already made keep their terms. Asks the owner \
                       to confirm: say the price first. Returns the price, the new version and \
                       warnings (payments off in this server, notice too short to pay).",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        ),
        meta = confirm_meta()
    )]
    async fn set_event_type_price(
        &self,
        Parameters(args): Parameters<SetPriceArgs>,
    ) -> Result<Json<PriceChanged>, ToolError> {
        let actor = self.actor(self.acting().await?);
        Ok(Json(
            ops_ton::set_event_type_price(&actor.writer(), args).await?,
        ))
    }

    #[tool(
        title = "Waive payment",
        description = "Confirm a pending booking whose TON payment has not arrived, free of \
                       charge, as the dashboard's \"Confirm without payment\" does: amount_ton \
                       must be exactly the remaining_ton get_booking shows. The payment is \
                       marked waived, then the booking is approved as approve_booking does \
                       (calendar, guest email). Never because a guest says they paid: a \
                       payment that arrives confirms the booking by itself. Asks the owner to \
                       confirm; counts as an approval (at most 20 an hour).",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        ),
        meta = confirm_meta()
    )]
    async fn waive_payment(
        &self,
        Parameters(args): Parameters<WaivePaymentArgs>,
    ) -> Result<Json<BookingChanged>, ToolError> {
        let actor = self.actor(self.acting().await?);
        self.run_to_end(async move { ops_ton::waive_payment(&actor.writer(), args).await })
            .await
            .map(Json)
    }
}
