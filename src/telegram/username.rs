//! The Telegram username a guest types on the booking form. It is only
//! information for the host: a bot cannot write to a username, so what links
//! a chat is the deep-link token, never this field.

/// Per-event-type guest Telegram field, `telegram_event_types.guest_mode`.
pub(crate) mod mode {
    /// No field and no link. The default.
    pub(crate) const OFF: &str = "off";
    /// Field shown, may be left empty; the link is offered either way.
    pub(crate) const OPTIONAL: &str = "optional";
    /// Field shown and required.
    pub(crate) const REQUIRED: &str = "required";

    pub(crate) const ALL: &[&str] = &[OFF, OPTIONAL, REQUIRED];

    /// Anything stored or posted as a known mode, defaulting to off.
    pub(crate) fn parse(value: &str) -> &'static str {
        match value.trim() {
            OPTIONAL => OPTIONAL,
            REQUIRED => REQUIRED,
            _ => OFF,
        }
    }
}

/// A username as typed (`@name`, `name`, `t.me/name`, `https://t.me/name`),
/// normalised to `name`: a Latin letter, then letters, digits and `_`, 4 to 32
/// characters, not ending in `_`. `None` when it is not one.
pub(crate) fn normalize(raw: &str) -> Option<String> {
    let mut value = raw.trim();
    for prefix in [
        "https://t.me/",
        "http://t.me/",
        "https://telegram.me/",
        "http://telegram.me/",
        "t.me/",
        "telegram.me/",
    ] {
        if let Some(rest) = value.strip_prefix(prefix) {
            value = rest;
            break;
        }
    }
    let value = value.strip_prefix('@').unwrap_or(value);
    valid(value).then(|| value.to_string())
}

/// The username rule, on an already stripped value.
pub(crate) fn valid(value: &str) -> bool {
    let bytes = value.as_bytes();
    (4..=32).contains(&bytes.len())
        && bytes[0].is_ascii_alphabetic()
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'_')
        && !value.ends_with('_')
}

/// The username to store for a booking, honouring the event type's mode.
/// `Err` carries the localised (title, message) pair for the booking error
/// page, the shape `resolve_guest_phone` returns.
pub(crate) fn resolve(
    mode: &str,
    raw: Option<&str>,
    lang: &str,
) -> Result<Option<String>, (String, String)> {
    let mode = mode::parse(mode);
    if mode == mode::OFF {
        return Ok(None);
    }
    let error = |message: String| {
        (
            crate::i18n::translate(lang, "tg-book-invalid-title", None),
            message,
        )
    };
    match raw.map(str::trim).filter(|v| !v.is_empty()) {
        None if mode == mode::REQUIRED => Err(error(crate::i18n::translate(
            lang,
            "tg-book-required",
            None,
        ))),
        None => Ok(None),
        Some(value) => match normalize(value) {
            Some(name) => Ok(Some(name)),
            None => Err(error(crate::i18n::translate(lang, "tg-book-invalid", None))),
        },
    }
}
