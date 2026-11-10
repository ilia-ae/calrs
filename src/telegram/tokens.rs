//! Deep-link tokens: `t.me/<bot>?start=<token>`. 32 random bytes, base64url
//! without padding (43 characters, inside Telegram's 64-character
//! `[A-Za-z0-9_-]` alphabet), behind a one-letter kind. Only the SHA-256 of a
//! token is stored, so the database alone links nobody.

use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// What a token links.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// One booking, for its guest (`b_`).
    Booking,
    /// A user's own chat, for host notifications (`u_`).
    User,
}

impl Kind {
    fn prefix(self) -> &'static str {
        match self {
            Kind::Booking => "b_",
            Kind::User => "u_",
        }
    }
}

/// Characters after the prefix.
const BODY_LEN: usize = 43;

/// A fresh token and its stored hash.
pub(crate) fn new_token(kind: Kind) -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let plain = format!(
        "{}{}",
        kind.prefix(),
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    );
    let hash = hash(&plain);
    (plain, hash)
}

/// A booking token (`b_…`) over 32 bytes the caller derived (TON: from the
/// payment link, `web::ton_pay::telegram::link_token`), with the same shape
/// as a fresh one.
pub(crate) fn booking_token_from(bytes: &[u8; 32]) -> String {
    format!(
        "{}{}",
        Kind::Booking.prefix(),
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    )
}

/// Hex SHA-256 of a token, as stored.
pub(crate) fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// A `/start` payload that has the shape of one of our tokens, as its kind
/// and the hash to look up. Anything else is `None`.
pub(crate) fn parse_payload(payload: &str) -> Option<(Kind, String)> {
    let kind = if payload.starts_with("b_") {
        Kind::Booking
    } else if payload.starts_with("u_") {
        Kind::User
    } else {
        return None;
    };
    let body = &payload[2..];
    let well_formed = body.len() == BODY_LEN
        && body
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    well_formed.then(|| (kind, hash(payload)))
}
