//! Outgoing: where a guest's reply goes, and how it finds its booking again.
//!
//! - Reply-To: `local+<token>@domain` in plus mode, else the bare system
//!   mailbox; no Reply-To at all when no mailbox is read (the reply then goes
//!   to the system From, never to the owner).
//! - Message-ID: `<calrs.<token>.<16 hex>@domain>` for a booking's mail, so
//!   In-Reply-To and References carry the token whatever the client does
//!   with Reply-To.
//! - `X-Calrs-Origin: 1` on every message, so our own mail coming back
//!   (a forward rule, a misdirected copy) is recognised as a loop.
//! - The subject tag ` [#<token>]` on host messages (`email::messages`).
//! - A guest `.ics`'s ORGANIZER and the guest pages' contact address.

use hmac::{Hmac, Mac};
use lettre::message::header::{Header, HeaderName, HeaderValue};
use lettre::message::{Mailbox, MessageBuilder};
use sha2::Sha256;

/// Characters of a token (80 bits of base32).
pub(crate) const TOKEN_LEN: usize = 16;
/// The prefix of our Message-IDs.
pub(crate) const MESSAGE_ID_PREFIX: &str = "calrs.";
/// Our loop marker.
pub(crate) const ORIGIN_HEADER: &str = "X-Calrs-Origin";

const BASE32: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";

/// RFC 4648 base32, lowercase, unpadded, cut to `len` characters.
fn base32(bytes: &[u8], len: usize) -> String {
    let mut out = String::with_capacity(len);
    let (mut acc, mut bits) = (0u32, 0u32);
    for &b in bytes {
        acc = (acc << 8) | u32::from(b);
        bits += 8;
        while bits >= 5 && out.len() < len {
            bits -= 5;
            out.push(BASE32[((acc >> bits) & 31) as usize] as char);
        }
        acc &= (1 << bits) - 1;
        if out.len() >= len {
            break;
        }
    }
    if bits > 0 && out.len() < len {
        out.push(BASE32[((acc << (5 - bits)) & 31) as usize] as char);
    }
    out
}

/// The token of a booking (its iCalendar `uid`) under `key`.
pub(crate) fn token_with(key: &[u8; 32], uid: &str) -> String {
    let mut mac =
        <Hmac<Sha256> as Mac>::new_from_slice(key).expect("HMAC accepts a key of any length");
    mac.update(b"calrs-reply-v1\0");
    mac.update(uid.as_bytes());
    base32(&mac.finalize().into_bytes(), TOKEN_LEN)
}

/// The token of a booking, when the key is known.
pub(crate) fn token(uid: &str) -> Option<String> {
    super::key().map(|key| token_with(&key, uid))
}

/// Whether `value` has a token's shape.
pub(crate) fn token_shaped(value: &str) -> bool {
    value.len() == TOKEN_LEN && value.bytes().all(|b| BASE32.contains(&b))
}

/// The address a guest's reply about `uid` goes to, when a mailbox is read.
pub(crate) fn reply_address(uid: Option<&str>) -> Option<String> {
    if !super::relay() {
        return None;
    }
    let config = super::config()?;
    if config.plus {
        if let Some(token) = uid.and_then(token) {
            let (local, domain) = config.parts();
            return Some(format!("{local}+{token}@{domain}"));
        }
    }
    Some(config.address.clone())
}

/// ` [#<token>]` for a host message's subject, when a mailbox is read.
pub(crate) fn subject_tag(uid: &str) -> Option<String> {
    if !super::enabled() {
        return None;
    }
    token(uid).map(|t| format!(" [#{t}]"))
}

#[cfg(not(test))]
static SYSTEM_FROM: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);
#[cfg(test)]
thread_local! {
    static SYSTEM_FROM: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Remember the SMTP From address (`email::load_smtp_config` calls this),
/// for the ORGANIZER of guest invites when no mailbox is configured.
pub(crate) fn note_system_from(address: &str) {
    let address = address.trim().to_string();
    #[cfg(not(test))]
    {
        let mut slot = SYSTEM_FROM.write().unwrap_or_else(|e| e.into_inner());
        if slot.as_deref() != Some(address.as_str()) {
            *slot = Some(address);
        }
    }
    #[cfg(test)]
    SYSTEM_FROM.with(|s| *s.borrow_mut() = Some(address));
}

/// The system's own address: the mailbox, else the SMTP From.
pub(crate) fn system_address() -> Option<String> {
    if let Some(config) = super::config() {
        return Some(config.address.clone());
    }
    #[cfg(not(test))]
    {
        if let Some(env) = std::env::var("CALRS_SMTP_FROM_EMAIL")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
        {
            return Some(env);
        }
        SYSTEM_FROM
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    #[cfg(test)]
    SYSTEM_FROM.with(|s| s.borrow().clone())
}

/// The ORGANIZER address of a guest's `.ics` in relay mode: where a reply
/// about `uid` goes, else the system address. `None`: nothing is known
/// (no SMTP at all), and the line is left out.
pub(crate) fn guest_organizer(uid: &str) -> Option<String> {
    reply_address(Some(uid)).or_else(system_address)
}

/// The contact address a guest page shows: in relay mode the booking's
/// reply address (none when no mailbox is read), else upstream's host
/// address.
pub(crate) fn guest_contact(uid: Option<&str>, host_email: String) -> String {
    if super::relay() {
        reply_address(uid).unwrap_or_default()
    } else {
        host_email
    }
}

/// A `mailto:` link to the booking's reply address, with the subject tag
/// when the address alone does not name the booking (bare mode).
pub(crate) fn guest_mailto(uid: &str) -> Option<(String, String)> {
    let address = reply_address(Some(uid))?;
    let tagged = super::config().is_some_and(|c| !c.plus);
    let href = match token(uid).filter(|_| tagged) {
        Some(t) => format!(
            "mailto:{address}?subject={}",
            urlencoding::encode(&format!("[#{t}]"))
        ),
        None => format!("mailto:{address}"),
    };
    Some((href, address))
}

/// The domain of an address.
fn domain_of(address: &str) -> &str {
    address
        .rsplit_once('@')
        .map(|(_, d)| d.trim())
        .filter(|d| !d.is_empty())
        .unwrap_or("calrs.invalid")
}

/// A Message-ID for a booking's mail (`uid`), or a plain one.
pub(crate) fn message_id(uid: Option<&str>, from_email: &str) -> String {
    let domain = super::config()
        .map(|c| c.parts().1.to_string())
        .unwrap_or_else(|| domain_of(from_email).to_string());
    let random: [u8; 8] = rand::random();
    let random = hex::encode(random);
    match uid.and_then(token) {
        Some(t) => format!("<{MESSAGE_ID_PREFIX}{t}.{random}@{domain}>"),
        None => format!("<{MESSAGE_ID_PREFIX}{random}@{domain}>"),
    }
}

/// The tokens in Message-IDs (In-Reply-To, References): `calrs.<token>.`.
pub(crate) fn tokens_in_ids(ids: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (at, _) in ids.match_indices(MESSAGE_ID_PREFIX) {
        let rest = &ids[at + MESSAGE_ID_PREFIX.len()..];
        if rest.len() > TOKEN_LEN && rest.as_bytes()[TOKEN_LEN] == b'.' {
            let candidate = rest[..TOKEN_LEN].to_ascii_lowercase();
            if token_shaped(&candidate) && !found.contains(&candidate) {
                found.push(candidate);
            }
        }
    }
    found
}

/// `X-Calrs-Origin`, the loop marker.
#[derive(Clone)]
struct Origin;

impl Header for Origin {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str(ORIGIN_HEADER)
    }
    fn parse(_: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Origin)
    }
    fn display(&self) -> HeaderValue {
        HeaderValue::new(Self::name(), "1".to_string())
    }
}

/// Our loop marker on any message, in relay mode.
pub(crate) fn mark(builder: MessageBuilder) -> MessageBuilder {
    if super::relay() {
        builder.header(Origin)
    } else {
        builder
    }
}

/// A guest-facing message in relay mode: the loop marker, the Message-ID
/// with the booking's token, and Reply-To the system mailbox (none when no
/// mailbox is read).
pub(crate) fn stamp(
    builder: MessageBuilder,
    booking: Option<&str>,
    from_email: &str,
) -> MessageBuilder {
    let mut builder = mark(builder).message_id(Some(message_id(booking, from_email)));
    if let Some(address) = reply_address(booking).and_then(|a| a.parse().ok()) {
        builder = builder.reply_to(Mailbox::new(None, address));
    }
    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbox::{test_mode, Config};

    #[test]
    fn base32_matches_rfc_4648() {
        assert_eq!(base32(b"foobar", 10), "mzxw6ytboi");
        assert_eq!(base32(b"f", 2), "my");
    }

    #[test]
    fn tokens_are_stable_per_booking_and_differ_between_them() {
        let key = [3u8; 32];
        let a = token_with(&key, "uid-a");
        assert_eq!(a.len(), TOKEN_LEN);
        assert!(token_shaped(&a));
        assert_eq!(a, token_with(&key, "uid-a"));
        assert_ne!(a, token_with(&key, "uid-b"));
        assert_ne!(a, token_with(&[4u8; 32], "uid-a"));
    }

    #[test]
    fn reply_to_by_mode() {
        let _off = test_mode(false, None);
        assert_eq!(reply_address(Some("u1")), None);
        drop(_off);
        let _no_inbox = test_mode(true, None);
        assert_eq!(reply_address(Some("u1")), None);
        drop(_no_inbox);
        let _bare = test_mode(true, Some(Config::for_tests("booking@ilia.ae", false)));
        assert_eq!(
            reply_address(Some("u1")).as_deref(),
            Some("booking@ilia.ae")
        );
        let (href, shown) = guest_mailto("u1").unwrap();
        assert_eq!(shown, "booking@ilia.ae");
        assert!(
            href.starts_with("mailto:booking@ilia.ae?subject=%5B%23"),
            "{href}"
        );
        drop(_bare);
        let _plus = test_mode(true, Some(Config::for_tests("booking@ilia.ae", true)));
        let t = token("u1").unwrap();
        assert_eq!(
            reply_address(Some("u1")),
            Some(format!("booking+{t}@ilia.ae"))
        );
        assert_eq!(reply_address(None).as_deref(), Some("booking@ilia.ae"));
        assert_eq!(
            guest_mailto("u1").unwrap().0,
            format!("mailto:booking+{t}@ilia.ae")
        );
    }

    #[test]
    fn message_ids_carry_the_token_and_are_found_again() {
        let _bare = test_mode(true, Some(Config::for_tests("booking@ilia.ae", false)));
        let t = token("u1").unwrap();
        let id = message_id(Some("u1"), "noreply@other.example");
        assert!(id.starts_with(&format!("<calrs.{t}.")), "{id}");
        assert!(id.ends_with("@ilia.ae>"), "{id}");
        let refs = format!("<x@gmail.com> {id} <CALRS.{}.ab@ilia.ae>", t.to_uppercase());
        assert_eq!(tokens_in_ids(&refs), vec![t.clone()]);
        assert!(tokens_in_ids("<calrs.0123456789abcdef@ilia.ae>").is_empty());
        let plain = message_id(None, "noreply@other.example");
        assert!(plain.starts_with("<calrs.") && plain.ends_with("@ilia.ae>"));
        assert!(tokens_in_ids(&plain).is_empty());
    }

    #[test]
    fn the_organizer_is_the_system_never_unknown_when_smtp_is_known() {
        let _relay = test_mode(true, None);
        note_system_from("noreply@test.com");
        assert_eq!(guest_organizer("u1").as_deref(), Some("noreply@test.com"));
        assert_eq!(guest_contact(Some("u1"), "host@x.example".into()), "");
        drop(_relay);
        assert_eq!(
            guest_contact(Some("u1"), "host@x.example".into()),
            "host@x.example"
        );
    }
}
