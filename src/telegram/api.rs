//! The Telegram Bot API (core.telegram.org/bots/api), the parts calrs uses:
//! `getMe`, `getUpdates` (long polling), `sendMessage`, `setMyCommands`,
//! `setMyDescription` and `setMyShortDescription`.
//!
//! Every request goes through [`HttpApi::call`], the one place that sees a URL:
//! the token is part of it (`https://api.telegram.org/bot<token>/METHOD`), so
//! reqwest errors are stripped of their URL and the remaining text is
//! redacted before it becomes a [`TgError`]. Response parsing is a pure
//! function ([`parse_response`]) tested against the documented shapes.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::config::{redact, BotConfig};

/// The Bot API endpoint.
const API_BASE: &str = "https://api.telegram.org";

/// `getUpdates` long-poll timeout, in seconds. The HTTP timeout is longer.
pub(crate) const POLL_TIMEOUT_SECS: u32 = 50;

/// A Bot API response envelope.
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    pub(crate) ok: bool,
    pub(crate) result: Option<T>,
    pub(crate) description: Option<String>,
    pub(crate) error_code: Option<i64>,
    pub(crate) parameters: Option<ResponseParameters>,
}

/// `ResponseParameters`: why a request failed and when it may be repeated.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ResponseParameters {
    pub(crate) migrate_to_chat_id: Option<i64>,
    pub(crate) retry_after: Option<u64>,
}

/// One incoming update. Only `message` is read; every other kind
/// (`edited_message`, `channel_post`, `my_chat_member`, ...) is ignored.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Update {
    pub(crate) update_id: i64,
    #[serde(default)]
    pub(crate) message: Option<Message>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Message {
    pub(crate) message_id: i64,
    #[serde(default)]
    pub(crate) from: Option<User>,
    pub(crate) chat: Chat,
    #[serde(default)]
    pub(crate) date: i64,
    #[serde(default)]
    pub(crate) text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct User {
    pub(crate) id: i64,
    pub(crate) is_bot: bool,
    #[serde(default)]
    pub(crate) first_name: String,
    #[serde(default)]
    pub(crate) username: Option<String>,
    #[serde(default)]
    pub(crate) language_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Chat {
    pub(crate) id: i64,
    /// private, group, supergroup or channel.
    #[serde(rename = "type")]
    pub(crate) kind: String,
}

/// A message calrs sends: plain text (no `parse_mode`, so nothing in it is
/// markup) and URL buttons.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Outgoing {
    pub(crate) text: String,
    pub(crate) buttons: Vec<Button>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Button {
    pub(crate) label: String,
    pub(crate) url: String,
}

/// The `sendMessage` request body.
#[derive(Debug, Serialize)]
pub(crate) struct SendMessage<'a> {
    pub(crate) chat_id: i64,
    pub(crate) text: &'a str,
    pub(crate) link_preview_options: LinkPreviewOptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reply_markup: Option<InlineKeyboardMarkup>,
}

#[derive(Debug, Serialize)]
pub(crate) struct LinkPreviewOptions {
    pub(crate) is_disabled: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct InlineKeyboardMarkup {
    pub(crate) inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct InlineKeyboardButton {
    pub(crate) text: String,
    pub(crate) url: String,
}

impl<'a> SendMessage<'a> {
    /// The request for `msg`. Buttons are sent only with `https://` links
    /// (Telegram refuses others in a keyboard); message composition already
    /// moved any other link into the text.
    pub(crate) fn new(chat_id: i64, msg: &'a Outgoing) -> Self {
        let rows: Vec<Vec<InlineKeyboardButton>> = msg
            .buttons
            .iter()
            .filter(|b| b.url.starts_with("https://"))
            .map(|b| {
                vec![InlineKeyboardButton {
                    text: b.label.clone(),
                    url: b.url.clone(),
                }]
            })
            .collect();
        SendMessage {
            chat_id,
            text: &msg.text,
            link_preview_options: LinkPreviewOptions { is_disabled: true },
            reply_markup: (!rows.is_empty()).then_some(InlineKeyboardMarkup {
                inline_keyboard: rows,
            }),
        }
    }
}

/// Why a Bot API call failed. No variant ever carries the token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TgError {
    /// 401, or 404 (Telegram's answer to a token it does not know).
    Unauthorized,
    /// 409: a webhook is set, or another process polls with the same token.
    Conflict(String),
    /// 429, with the seconds to wait.
    RateLimited { retry_after: u64 },
    /// 400: the request itself is wrong (chat not found, text too long).
    BadRequest(String),
    /// 403: the user blocked the bot, or the bot cannot write to the chat.
    Forbidden(String),
    /// 5xx.
    Server(u16),
    /// No answer: DNS, TLS, connection, timeout. Already redacted.
    Transport(String),
    /// An answer calrs could not read.
    Other(String),
}

impl TgError {
    /// Short label for structured logs.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            TgError::Unauthorized => "unauthorized",
            TgError::Conflict(_) => "conflict",
            TgError::RateLimited { .. } => "rate_limited",
            TgError::BadRequest(_) => "bad_request",
            TgError::Forbidden(_) => "forbidden",
            TgError::Server(_) => "server",
            TgError::Transport(_) => "transport",
            TgError::Other(_) => "other",
        }
    }

    /// The same error with every string passed through [`redact`].
    pub(crate) fn redacted(self, token: &str) -> Self {
        match self {
            TgError::Conflict(s) => TgError::Conflict(redact(&s, token)),
            TgError::BadRequest(s) => TgError::BadRequest(redact(&s, token)),
            TgError::Forbidden(s) => TgError::Forbidden(redact(&s, token)),
            TgError::Transport(s) => TgError::Transport(redact(&s, token)),
            TgError::Other(s) => TgError::Other(redact(&s, token)),
            other => other,
        }
    }
}

impl std::fmt::Display for TgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TgError::Unauthorized => write!(f, "the bot token was refused (401/404)"),
            TgError::Conflict(d) => write!(
                f,
                "conflict: a webhook is set or another process polls this bot ({d})"
            ),
            TgError::RateLimited { retry_after } => {
                write!(f, "rate limited by Telegram, retry after {retry_after} s")
            }
            TgError::BadRequest(d) => write!(f, "request refused: {d}"),
            TgError::Forbidden(d) => write!(f, "forbidden: {d}"),
            TgError::Server(code) => write!(f, "Telegram server error (HTTP {code})"),
            TgError::Transport(d) => write!(f, "could not reach Telegram: {d}"),
            TgError::Other(d) => write!(f, "unexpected Telegram response: {d}"),
        }
    }
}

impl std::error::Error for TgError {}

/// A Bot API answer as a value of `T`, or the error it describes. Pure: the
/// HTTP status and the body as received.
pub(crate) fn parse_response<T: serde::de::DeserializeOwned>(
    status: u16,
    body: &str,
) -> Result<T, TgError> {
    let envelope: ApiResponse<serde_json::Value> = match serde_json::from_str(body) {
        Ok(envelope) => envelope,
        Err(_) if status >= 500 => return Err(TgError::Server(status)),
        Err(_) => return Err(TgError::Other(format!("HTTP {status}, not JSON"))),
    };
    if envelope.ok {
        let result = envelope
            .result
            .ok_or_else(|| TgError::Other("ok without result".to_string()))?;
        return serde_json::from_value(result)
            .map_err(|e| TgError::Other(format!("unexpected result shape: {e}")));
    }
    let code = envelope.error_code.unwrap_or(i64::from(status));
    let description = envelope
        .description
        .unwrap_or_else(|| format!("HTTP {status}"));
    Err(match code {
        401 | 404 => TgError::Unauthorized,
        409 => TgError::Conflict(description),
        429 => TgError::RateLimited {
            retry_after: envelope.parameters.and_then(|p| p.retry_after).unwrap_or(5),
        },
        400 => TgError::BadRequest(description),
        403 => TgError::Forbidden(description),
        500..=599 => TgError::Server(code as u16),
        _ => TgError::Other(description),
    })
}

/// The Bot API methods calrs calls. A trait so tests drive everything with a
/// fake and no request ever leaves the machine.
#[async_trait]
pub(crate) trait BotApi: Send + Sync {
    async fn get_me(&self) -> Result<User, TgError>;
    /// Raw updates: each is parsed on its own, so one odd update cannot stop
    /// the offset from advancing past the batch.
    async fn get_updates(&self, offset: i64) -> Result<Vec<serde_json::Value>, TgError>;
    async fn send_message(&self, chat_id: i64, msg: &Outgoing) -> Result<(), TgError>;
    /// `setMyCommands` for one language (`None`: the default list).
    async fn set_my_commands(
        &self,
        commands: &[(String, String)],
        language_code: Option<&str>,
    ) -> Result<(), TgError>;
    /// `setMyDescription` (the text above the Start button of an empty
    /// chat) and `setMyShortDescription` (the bot's profile) for one
    /// language (`None`: the default).
    async fn set_my_description(
        &self,
        description: &str,
        short_description: &str,
        language_code: Option<&str>,
    ) -> Result<(), TgError>;
}

/// The real Bot API over HTTPS.
pub(crate) struct HttpApi {
    config: BotConfig,
    client: reqwest::Client,
}

impl std::fmt::Debug for HttpApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpApi")
            .field("config", &self.config)
            .finish()
    }
}

impl HttpApi {
    pub(crate) fn new(config: BotConfig) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .https_only(true)
            .user_agent("calrs-telegram/1")
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    /// POST `body` as JSON to `method`, with `timeout` for the whole request.
    async fn call<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        body: &serde_json::Value,
        timeout: Duration,
    ) -> Result<T, TgError> {
        let token = self.config.token();
        let url = format!("{API_BASE}/bot{token}/{method}");
        let transport = |e: reqwest::Error| {
            let kind = if e.is_timeout() {
                "timeout"
            } else if e.is_connect() {
                "connect"
            } else {
                "request"
            };
            TgError::Transport(format!("{kind}: {}", e.without_url())).redacted(token)
        };
        let response = self
            .client
            .post(url)
            .timeout(timeout)
            .json(body)
            .send()
            .await
            .map_err(transport)?;
        let status = response.status().as_u16();
        let text = response.text().await.map_err(transport)?;
        parse_response(status, &text).map_err(|e| e.redacted(token))
    }
}

#[async_trait]
impl BotApi for HttpApi {
    async fn get_me(&self) -> Result<User, TgError> {
        self.call("getMe", &serde_json::json!({}), Duration::from_secs(15))
            .await
    }

    async fn get_updates(&self, offset: i64) -> Result<Vec<serde_json::Value>, TgError> {
        let body = serde_json::json!({
            "offset": offset,
            "limit": 50,
            "timeout": POLL_TIMEOUT_SECS,
            "allowed_updates": ["message"],
        });
        self.call(
            "getUpdates",
            &body,
            Duration::from_secs(u64::from(POLL_TIMEOUT_SECS) + 20),
        )
        .await
    }

    async fn send_message(&self, chat_id: i64, msg: &Outgoing) -> Result<(), TgError> {
        let body = serde_json::to_value(SendMessage::new(chat_id, msg))
            .map_err(|e| TgError::Other(e.to_string()))?;
        let _: serde_json::Value = self
            .call("sendMessage", &body, Duration::from_secs(15))
            .await?;
        Ok(())
    }

    async fn set_my_commands(
        &self,
        commands: &[(String, String)],
        language_code: Option<&str>,
    ) -> Result<(), TgError> {
        let list: Vec<serde_json::Value> = commands
            .iter()
            .map(|(command, description)| {
                serde_json::json!({"command": command, "description": description})
            })
            .collect();
        let mut body = serde_json::json!({ "commands": list });
        if let Some(code) = language_code {
            body["language_code"] = serde_json::json!(code);
        }
        let _: serde_json::Value = self
            .call("setMyCommands", &body, Duration::from_secs(15))
            .await?;
        Ok(())
    }

    async fn set_my_description(
        &self,
        description: &str,
        short_description: &str,
        language_code: Option<&str>,
    ) -> Result<(), TgError> {
        for (method, field, text) in [
            ("setMyDescription", "description", description),
            (
                "setMyShortDescription",
                "short_description",
                short_description,
            ),
        ] {
            let mut body = serde_json::json!({ field: text });
            if let Some(code) = language_code {
                body["language_code"] = serde_json::json!(code);
            }
            let _: serde_json::Value = self.call(method, &body, Duration::from_secs(15)).await?;
        }
        Ok(())
    }
}
