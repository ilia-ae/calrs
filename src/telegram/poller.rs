//! Long polling: `getMe` once (the bot's username for deep links), then
//! `getUpdates` in a loop until the shutdown flag is set. No webhook and no
//! listening port: the only traffic is outgoing HTTPS to api.telegram.org.

use std::sync::Arc;
use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::watch;

use super::api::TgError;
use super::{commands, message, store, Runtime};

/// `^[A-Za-z][A-Za-z0-9_]{2,29}[Bb][Oo][Tt]$`: what @BotFather allows.
pub(crate) fn valid_bot_username(name: &str) -> bool {
    let b = name.as_bytes();
    (6..=33).contains(&b.len())
        && b[0].is_ascii_alphabetic()
        && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'_')
        && name[name.len() - 3..].eq_ignore_ascii_case("bot")
}

/// Backoff of the poller.
#[derive(Debug, Clone)]
pub(crate) struct PollPolicy {
    pub(crate) get_me_first: Duration,
    pub(crate) get_me_max: Duration,
    pub(crate) unauthorized: Duration,
    pub(crate) conflict: Duration,
    pub(crate) transport_first: Duration,
    pub(crate) transport_max: Duration,
    pub(crate) retry_after_unit: Duration,
}

impl Default for PollPolicy {
    fn default() -> Self {
        Self {
            get_me_first: Duration::from_secs(5),
            get_me_max: Duration::from_secs(300),
            unauthorized: Duration::from_secs(600),
            conflict: Duration::from_secs(60),
            transport_first: Duration::from_secs(1),
            transport_max: Duration::from_secs(60),
            retry_after_unit: Duration::from_secs(1),
        }
    }
}

/// Sleep, or return `false` at once when the shutdown flag is set.
async fn pause(stop: &mut watch::Receiver<bool>, wait: Duration) -> bool {
    if *stop.borrow() {
        return false;
    }
    tokio::select! {
        _ = tokio::time::sleep(wait) => !*stop.borrow(),
        _ = stop.changed() => false,
    }
}

/// A little jitter (0-25 %) so restarts do not poll in lockstep.
fn jitter(d: Duration) -> Duration {
    use rand::Rng;
    let extra = rand::thread_rng().gen_range(0..=d.as_millis() as u64 / 4);
    d + Duration::from_millis(extra)
}

/// Run until the shutdown flag is set.
pub(crate) async fn run(
    rt: Arc<Runtime>,
    pool: SqlitePool,
    mut stop: watch::Receiver<bool>,
    policy: PollPolicy,
) {
    // getMe: who the bot is. Until it answers, the cached name serves links.
    let mut backoff = policy.get_me_first;
    loop {
        if *stop.borrow() {
            return;
        }
        let result = tokio::select! {
            r = rt.api.get_me() => r,
            _ = stop.changed() => return,
        };
        match result {
            Ok(me) => match me.username.filter(|u| valid_bot_username(u)) {
                Some(name) => {
                    tracing::info!(target: "calrs::telegram", username = %format!("@{name}"), "telegram bot ready");
                    store::save_bot_username(&pool, &name).await;
                    rt.set_bot_username(Some(name));
                    break;
                }
                None => {
                    tracing::error!(target: "calrs::telegram", "getMe returned no usable bot username; no deep links");
                    break;
                }
            },
            Err(TgError::Unauthorized) => {
                tracing::error!(
                    target: "calrs::telegram",
                    "the bot token was refused (revoked?); retrying in 10 minutes"
                );
                if !pause(&mut stop, policy.unauthorized).await {
                    return;
                }
            }
            Err(e) => {
                tracing::warn!(target: "calrs::telegram", kind = e.kind(), error = %e, "getMe failed");
                if !pause(&mut stop, jitter(backoff)).await {
                    return;
                }
                backoff = (backoff * 2).min(policy.get_me_max);
            }
        }
    }

    // The command menu and the bot's description (the text above the Start
    // button of an empty chat), best-effort: the default in English, then
    // one per shipped language (Telegram takes two-letter codes, so
    // de-formal is served by de).
    let site = commands::site_name();
    let mut langs: Vec<Option<&str>> = vec![None];
    langs.extend(
        crate::i18n::supported_with_labels()
            .map(|(code, _)| code)
            .filter(|code| code.len() == 2 && *code != "en")
            .map(Some),
    );
    for lang in langs {
        let text_lang = lang.unwrap_or("en");
        let list = message::commands(text_lang);
        if let Err(e) = rt.api.set_my_commands(&list, lang).await {
            tracing::warn!(target: "calrs::telegram", kind = e.kind(), error = %e, "setMyCommands failed");
        }
        let (description, short) = message::description(text_lang, &site);
        if let Err(e) = rt.api.set_my_description(&description, &short, lang).await {
            tracing::warn!(target: "calrs::telegram", kind = e.kind(), error = %e, "setMyDescription failed");
        }
    }

    let mut offset = store::load_offset(&pool).await;
    let mut backoff = policy.transport_first;
    let mut unauthorized_logged = false;
    loop {
        if *stop.borrow() {
            return;
        }
        let result = tokio::select! {
            r = rt.api.get_updates(offset) => r,
            _ = stop.changed() => return,
        };
        let wait = match result {
            Ok(batch) => {
                backoff = policy.transport_first;
                unauthorized_logged = false;
                for raw in &batch {
                    if let Some(id) = raw.get("update_id").and_then(|v| v.as_i64()) {
                        offset = offset.max(id.saturating_add(1));
                    }
                    commands::handle_update(&rt, &pool, raw).await;
                }
                if !batch.is_empty() {
                    store::save_offset(&pool, offset).await;
                }
                continue;
            }
            Err(TgError::Unauthorized) => {
                if !unauthorized_logged {
                    tracing::error!(target: "calrs::telegram", "getUpdates: the bot token was refused; retrying in 10 minutes");
                    unauthorized_logged = true;
                }
                policy.unauthorized
            }
            Err(TgError::Conflict(_)) => {
                tracing::error!(
                    target: "calrs::telegram",
                    "getUpdates 409 Conflict: a webhook is set or another process polls this bot"
                );
                policy.conflict
            }
            Err(TgError::RateLimited { retry_after }) => policy
                .retry_after_unit
                .saturating_mul(u32::try_from(retry_after).unwrap_or(u32::MAX)),
            Err(e) => {
                tracing::warn!(target: "calrs::telegram", kind = e.kind(), error = %e, "getUpdates failed");
                let wait = jitter(backoff);
                backoff = (backoff * 2).min(policy.transport_max);
                wait
            }
        };
        if !pause(&mut stop, wait).await {
            return;
        }
    }
}
