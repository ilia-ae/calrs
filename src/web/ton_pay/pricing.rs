//! ilia fork: what the pages show of the demand ladder (ilia-booking
//! pricing design §1.5, with the critique's changes). The booking side
//! (quote before the transaction, recount inside it, free step, per-guest
//! limit) is in `invoice::prepare` and `invoice::open`; the rules of the
//! ladder itself in `crate::ton::ladder`; where its setting comes from in
//! `crate::ton::settings`.

use chrono::Utc;
use minijinja::context;

use super::{store, TARGET};
use crate::ton::ladder::{ton_text, Ladder};
use crate::web::AppState;

/// The ladder in effect for a page that may show while payments are off
/// (the event type form): the running runtime's when payments are on, else
/// the one resolved now (env > DB > default). `None` when it cannot be
/// read (the error is logged; nothing about it is shown).
pub(crate) async fn current_ladder(state: &AppState) -> Option<Ladder> {
    if let Some(rt) = state.ton.get() {
        return Some(rt.cfg.ladder.clone());
    }
    let db = match state.pool.acquire().await {
        Ok(mut conn) => store::load_settings(&mut conn).await,
        Err(e) => Err(e),
    };
    let db = db
        .map_err(|e| tracing::error!(target: TARGET, error = %e, "cannot read the TON settings"))
        .ok()?;
    let env = state.ton.env();
    crate::ton::settings::ladder(&|k| env.get(k), &db)
        .map_err(|e| tracing::error!(target: TARGET, error = %e, "the demand ladder is invalid"))
        .ok()
        .map(|r| r.value)
}

/// The ladder for the price section of the event type form (shown
/// read-only next to the "by demand" mode): its text and its steps.
pub(crate) async fn form_ctx(state: &AppState) -> Option<minijinja::Value> {
    let ladder = current_ladder(state).await?;
    let steps: Vec<String> = ladder.steps().iter().map(|s| s.text()).collect();
    Some(context! {
        text => ladder.text(),
        steps => steps,
    })
}

/// The price line of a public page for a ladder type: the price now (one
/// COUNT of the owner's demand), the ladder in short, and the value the
/// booking form carries as `quoted_nano`. `None` while payments are off
/// (the type is then not bookable) or when the count fails.
pub(crate) async fn badge(state: &AppState, owner_user_id: &str) -> Option<minijinja::Value> {
    let rt = state.ton.get()?;
    let ladder = &rt.cfg.ladder;
    let n = match state.pool.acquire().await {
        Ok(mut conn) => store::demand_n(&mut conn, owner_user_id, Utc::now(), None).await,
        Err(e) => Err(e),
    }
    .map_err(|e| tracing::error!(target: TARGET, error = %e, "cannot count the demand"))
    .ok()?;
    let step = ladder.price(n);
    Some(context! {
        ladder => true,
        now_free => step.is_free(),
        now_ton => step.nano().map(ton_text),
        quote => step.quote_value(),
        starts_free => ladder.starts_free(),
        from_ton => ladder.lowest_price().map(ton_text),
        to_ton => ladder.ceiling().nano().map(ton_text),
    })
}
