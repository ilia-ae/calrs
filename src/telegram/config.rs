//! `CALRS_TELEGRAM_BOT_TOKEN`: the only switch of the feature, and the only
//! secret. The token is part of every Bot API URL, so it never reaches a log,
//! an error or a page: `BotConfig` prints `<redacted>`, and every error string
//! that could carry a URL passes [`redact`].

/// The environment variable holding the bot token from @BotFather.
pub(crate) const TOKEN_ENV: &str = "CALRS_TELEGRAM_BOT_TOKEN";

/// What replaces a token in any text that leaves this module.
pub(crate) const REDACTED: &str = "<redacted>";

/// The bot's credentials. Built only from a token that has the shape
/// @BotFather hands out.
#[derive(Clone)]
pub(crate) struct BotConfig {
    token: String,
}

impl std::fmt::Debug for BotConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotConfig")
            .field("token", &REDACTED)
            .finish()
    }
}

impl BotConfig {
    /// The token, for building request URLs only.
    pub(crate) fn token(&self) -> &str {
        &self.token
    }

    /// The bot's numeric id: the part of the token before `:`. Public (getMe
    /// returns it), so it may be stored; it tells whether the token now
    /// belongs to another bot than the stored update offset.
    pub(crate) fn bot_id(&self) -> Option<i64> {
        self.token
            .split_once(':')
            .and_then(|(id, _)| id.parse().ok())
    }

    #[cfg(test)]
    pub(crate) fn for_tests(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }
}

/// `^[0-9]{5,16}:[A-Za-z0-9_-]{30,50}$`, the shape of a Bot API token.
pub(crate) fn valid_token(token: &str) -> bool {
    let Some((id, secret)) = token.split_once(':') else {
        return false;
    };
    (5..=16).contains(&id.len())
        && id.bytes().all(|b| b.is_ascii_digit())
        && (30..=50).contains(&secret.len())
        && secret
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// What the environment says, without logging: `Ok(None)` when unset or
/// empty (Telegram off), `Err(())` when set to something that is not a token.
pub(crate) fn parse(raw: Option<&str>) -> Result<Option<BotConfig>, ()> {
    match raw.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(token) if valid_token(token) => Ok(Some(BotConfig {
            token: token.to_string(),
        })),
        Some(_) => Err(()),
    }
}

/// The bot configuration from `CALRS_TELEGRAM_BOT_TOKEN`, or `None` (off).
/// A malformed value is logged without the value and leaves Telegram off.
pub(crate) fn from_env() -> Option<BotConfig> {
    match parse(std::env::var(TOKEN_ENV).ok().as_deref()) {
        Ok(Some(config)) => Some(config),
        Ok(None) => {
            tracing::info!(
                target: "calrs::telegram",
                "CALRS_TELEGRAM_BOT_TOKEN is not set; Telegram off"
            );
            None
        }
        Err(()) => {
            tracing::error!(
                target: "calrs::telegram",
                "CALRS_TELEGRAM_BOT_TOKEN is malformed; Telegram off"
            );
            None
        }
    }
}

/// `text` with the token, its secret half, and anything else shaped like a
/// Bot API path segment (`bot<digits>:<secret>`) replaced by `<redacted>`.
pub(crate) fn redact(text: &str, token: &str) -> String {
    let mut out = text.to_string();
    if !token.is_empty() {
        out = out.replace(token, REDACTED);
        if let Some((_, secret)) = token.split_once(':') {
            if secret.len() >= 8 {
                out = out.replace(secret, REDACTED);
            }
        }
    }
    redact_token_shapes(&out)
}

/// Any `<5+ digits>:<20+ token characters>` run, whoever's token it is.
fn redact_token_shapes(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut copied = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() && (i == 0 || !bytes[i - 1].is_ascii_digit()) {
            let digits_end = (i..bytes.len())
                .find(|&j| !bytes[j].is_ascii_digit())
                .unwrap_or(bytes.len());
            if digits_end - i >= 5 && bytes.get(digits_end) == Some(&b':') {
                let secret_start = digits_end + 1;
                let secret_end = (secret_start..bytes.len())
                    .find(|&j| {
                        !(bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'-')
                    })
                    .unwrap_or(bytes.len());
                if secret_end - secret_start >= 20 {
                    out.push_str(&text[copied..i]);
                    out.push_str(REDACTED);
                    copied = secret_end;
                    i = secret_end;
                    continue;
                }
            }
            i = digits_end;
            continue;
        }
        i += 1;
    }
    out.push_str(&text[copied..]);
    out
}
