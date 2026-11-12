//! ilia fork: passkeys and security keys (WebAuthn) as a second factor
//! (docs/passkeys-plan.md).
//!
//! A local account's second factor is TOTP or a passkey (a YubiKey or
//! another security key, Touch ID / Face ID / Windows Hello, a synced
//! passkey), or both. This module holds:
//!
//! - [`verify`]: the relying-party verifier, pure functions with no I/O that
//!   check a registration or an assertion against W3C Web Authentication
//!   Level 3, §7.1 and §7.2, with `attestation: "none"`. The browser hands
//!   over the credential public key as SPKI DER (`getPublicKey()`) and the
//!   authenticator data as bytes (`getAuthenticatorData()`), so no CBOR is
//!   decoded (§5.2.1.1);
//! - [`Rp`]: the RP ID and origin, from `CALRS_BASE_URL` only;
//! - [`store`]: credentials and ceremonies in SQLite;
//! - [`flows`]: the routes that only passkeys have (sign in, first-factor
//!   enrolment, "confirm it's you", registration from the settings, the
//!   static script). `src/mfa.rs` keeps the rest and calls in here.
//!
//! The crypto is the RustCrypto stack already in the build through
//! `openidconnect` (same versions and features): `p256` for ES256,
//! `ed25519-dalek` for EdDSA, `rsa` for RS256, `sha2` for the hashes.

pub(crate) mod flows;
pub(crate) mod store;
pub(crate) mod verify;

use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

/// Keys a user may register.
pub(crate) const MAX_KEYS: i64 = 10;
/// A key name: trimmed, 1 to 64 characters, no control characters.
pub(crate) const MAX_NAME_CHARS: usize = 64;
/// How long a ceremony lives: as long as the MFA challenge it belongs to
/// (`mfa::CHALLENGE_SECONDS`, the "10 minutes" the sign-in page states), so
/// a key pressed on a page that is still valid is not refused as expired.
/// A ceremony bound to a challenge never outlives it either way.
pub(crate) const CEREMONY_SECONDS: i64 = crate::mfa::CHALLENGE_SECONDS;
/// The browser's `timeout`: how long one press of a passkey button waits
/// for the key, counted from the press.
pub(crate) const BROWSER_TIMEOUT_SECONDS: i64 = 300;

/// The relying party: the RP ID (a host name) and the origin every
/// clientDataJSON must carry, as exact strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Rp {
    pub id: String,
    pub origin: String,
}

impl Rp {
    /// From `CALRS_BASE_URL` only: never the base URL stored in the
    /// database (an admin edits it from a web form, and the RP ID binds
    /// every key), never `Host` or `X-Forwarded-*`. `None` turns passkeys
    /// off. Tests never read the environment (other tests set this variable
    /// in parallel): they put an `Rp` into `AppState::passkeys` themselves.
    pub(crate) fn from_env() -> Option<Rp> {
        if cfg!(test) {
            return None;
        }
        let raw = std::env::var("CALRS_BASE_URL").ok()?;
        let rp = Rp::from_base_url(&raw);
        if rp.is_none() && !raw.trim().is_empty() {
            tracing::warn!(
                "passkeys off: CALRS_BASE_URL is not an https:// address with a host name"
            );
        }
        rp
    }

    /// `https://<domain>[:port][/path]`, or `http://localhost[:port]` for
    /// development. No user name or password, no IP address. The path is
    /// ignored; a default port is dropped, another one kept in the origin.
    pub(crate) fn from_base_url(raw: &str) -> Option<Rp> {
        let url = reqwest::Url::parse(raw.trim()).ok()?;
        if !url.username().is_empty() || url.password().is_some() {
            return None;
        }
        // `domain()` is `None` for an IP address (v4 or v6).
        let host = url.domain()?.to_ascii_lowercase();
        if host.is_empty() || host.ends_with('.') {
            return None;
        }
        match url.scheme() {
            "https" => {}
            "http" if host == "localhost" => {}
            _ => return None,
        }
        let origin = url.origin().ascii_serialization();
        Some(Rp { id: host, origin })
    }
}

/// The WebAuthn user handle: 16 bytes derived from the user id, so it
/// carries no personal data and needs no storage.
pub(crate) fn user_handle(user_id: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"calrs-webauthn-user\0");
    hasher.update(user_id.as_bytes());
    hasher.finalize()[..16].to_vec()
}

pub(crate) fn b64url(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// A key name as typed: trimmed; `None` when empty, longer than
/// [`MAX_NAME_CHARS`] or holding a control character. Escaped where shown.
pub(crate) fn clean_name(raw: &str) -> Option<String> {
    let name = raw.trim();
    if name.is_empty()
        || name.chars().count() > MAX_NAME_CHARS
        || name.chars().any(|c| c.is_control())
    {
        return None;
    }
    Some(name.to_string())
}

/// One credential as `allowCredentials` / `excludeCredentials` lists it.
pub(crate) struct Descriptor<'a> {
    pub credential_id: &'a [u8],
    pub transports: &'a str,
}

fn descriptors(keys: &[Descriptor<'_>]) -> serde_json::Value {
    keys.iter()
        .map(|key| {
            json!({
                "type": "public-key",
                "id": b64url(key.credential_id),
                "transports": verify::sanitize_transports(key.transports),
            })
        })
        .collect()
}

/// `PublicKeyCredentialCreationOptionsJSON` for `navigator.credentials.create()`.
/// `residentKey: "discouraged"` keeps YubiKey slots free; platforms that
/// always make discoverable passkeys still do.
pub(crate) fn creation_options(
    rp: &Rp,
    user_id: &str,
    email: &str,
    display_name: &str,
    challenge: &[u8],
    exclude: &[Descriptor<'_>],
) -> serde_json::Value {
    let params: Vec<serde_json::Value> = verify::ALGORITHMS
        .iter()
        .map(|alg| json!({"type": "public-key", "alg": alg}))
        .collect();
    json!({
        "rp": {"id": rp.id, "name": rp.id},
        "user": {
            "id": b64url(&user_handle(user_id)),
            "name": email,
            "displayName": display_name,
        },
        "challenge": b64url(challenge),
        "pubKeyCredParams": params,
        "timeout": BROWSER_TIMEOUT_SECONDS * 1000,
        "attestation": "none",
        "authenticatorSelection": {
            "residentKey": "discouraged",
            "requireResidentKey": false,
            "userVerification": "preferred",
        },
        "excludeCredentials": descriptors(exclude),
    })
}

/// `PublicKeyCredentialRequestOptionsJSON` for `navigator.credentials.get()`.
pub(crate) fn request_options(
    rp: &Rp,
    challenge: &[u8],
    allow: &[Descriptor<'_>],
) -> serde_json::Value {
    json!({
        "challenge": b64url(challenge),
        "rpId": rp.id,
        "timeout": BROWSER_TIMEOUT_SECONDS * 1000,
        "userVerification": "preferred",
        "allowCredentials": descriptors(allow),
    })
}

/// The fields `webauthn.js` fills from `navigator.credentials.get()`, all
/// base64url without padding. Absent fields are empty.
#[derive(Deserialize, Default, Clone)]
pub(crate) struct AssertionFields {
    #[serde(default)]
    pub _csrf: Option<String>,
    #[serde(default)]
    pub credential_id: String,
    #[serde(default)]
    pub client_data: String,
    #[serde(default)]
    pub authenticator_data: String,
    #[serde(default)]
    pub signature: String,
    #[serde(default)]
    pub user_handle: String,
}

impl AssertionFields {
    /// A passkey proof was posted (rather than a code).
    pub(crate) fn present(&self) -> bool {
        !self.credential_id.is_empty()
    }
}

/// The fields `webauthn.js` fills from `navigator.credentials.create()`.
#[derive(Deserialize, Default)]
pub(crate) struct RegistrationFields {
    #[serde(default)]
    pub _csrf: Option<String>,
    #[serde(default)]
    pub credential_id: String,
    #[serde(default)]
    pub client_data: String,
    #[serde(default)]
    pub authenticator_data: String,
    #[serde(default)]
    pub public_key: String,
    #[serde(default)]
    pub algorithm: String,
    #[serde(default)]
    pub transports: String,
    #[serde(default)]
    pub name: String,
}

#[cfg(test)]
mod flow_tests;
#[cfg(test)]
pub(crate) mod softkey;
#[cfg(test)]
mod tests;
