//! ilia fork: the WebAuthn relying-party verifier (docs/passkeys-plan.md §4.2).
//!
//! Pure functions: no I/O, no clock, no panics. Every length is checked
//! before it is used and every slice is taken with `get`, so any input, of
//! any length, gives `Ok` or one [`Error`] (the tests feed it every prefix
//! and random junk).
//!
//! What is checked follows W3C Web Authentication Level 3, §7.1 (registering
//! a new credential) and §7.2 (verifying an authentication assertion), with
//! `attestation: "none"`:
//!
//! - **clientDataJSON** (§5.8.1): UTF-8 with no BOM, one JSON object with
//!   `type`, `challenge` and `origin` as strings, no field twice; `type` is
//!   `webauthn.create` or `webauthn.get` as the ceremony requires; the
//!   challenge decodes from base64url without padding and equals ours
//!   (constant time); `origin` equals ours as an exact string (§13.4.9);
//!   `crossOrigin` is absent or `false`; `topOrigin` is absent. Unknown
//!   fields are allowed, as §5.8.1.2 requires.
//! - **authenticatorData** (§6.1): `rpIdHash` is SHA-256 of our RP ID; UP is
//!   set; UV is reported, not required (the password is the first factor);
//!   BS is not set without BE; the counter is read big-endian.
//! - **Registration**: AT is set; the attested credential data gives the
//!   AAGUID and a credential ID of 1 to 1023 bytes equal to the `rawId`;
//!   the credential public key after it is one well-formed CBOR map,
//!   followed by nothing, or by one extensions map when ED is set. The key
//!   itself is taken from `getPublicKey()` (SPKI DER), whose type must be
//!   what the declared algorithm implies: -8 Ed25519, -7 ECDSA P-256, -257
//!   RSA of 2048 to 4096 bits. The COSE key is only skipped, not decoded
//!   (§5.2.1.1 exists so that a relying party need not parse CBOR); under
//!   `none` attestation nothing signs the registration, so the client is
//!   the only source of the key either way.
//! - **Assertion**: AT is not set; without ED the data is exactly 37 bytes,
//!   with ED one well-formed CBOR map follows; the signature over `authenticatorData || SHA-256(clientDataJSON)`
//!   verifies with the stored key (Ed25519 `verify_strict`, ES256 DER with
//!   high S accepted, RS256 PKCS#1 v1.5); BE equals the stored value; the
//!   counter follows [`counter_ok`] (§6.1.1).

use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// COSE algorithm identifiers (IANA "COSE Algorithms").
pub(crate) const ALG_EDDSA: i64 = -8;
pub(crate) const ALG_ES256: i64 = -7;
pub(crate) const ALG_RS256: i64 = -257;
/// The order offered in `pubKeyCredParams`, which an authenticator reads
/// as a preference: it creates its key with the first one it supports.
/// ES256 leads because every browser's `getPublicKey()` returns it, while
/// Safari's returns nothing for Ed25519 or RSA (a YubiKey 5 that was offered
/// Ed25519 first could not be registered there). Ed25519 and RS256 stay
/// accepted for authenticators without ES256 (Windows Hello may use RS256).
pub(crate) const ALGORITHMS: [i64; 3] = [ALG_ES256, ALG_EDDSA, ALG_RS256];

/// Size caps, checked before anything is decoded.
pub(crate) const MAX_CLIENT_DATA: usize = 4096;
pub(crate) const MIN_AUTH_DATA: usize = 37;
pub(crate) const MAX_AUTH_DATA: usize = 4096;
pub(crate) const MAX_SIGNATURE: usize = 1024;
pub(crate) const MAX_PUBLIC_KEY: usize = 1024;
/// §5.1: a credential ID is at most 1023 bytes.
pub(crate) const MAX_CREDENTIAL_ID: usize = 1023;
/// §5.4.3: a user handle is at most 64 bytes.
pub(crate) const MAX_USER_HANDLE: usize = 64;
/// Challenges we issue are 32 bytes; §13.4.3 asks for at least 16.
pub(crate) const MIN_CHALLENGE: usize = 16;
pub(crate) const MAX_CHALLENGE: usize = 64;
pub(crate) const RSA_MIN_BITS: usize = 2048;
pub(crate) const RSA_MAX_BITS: usize = 4096;
pub(crate) const MAX_TRANSPORTS: usize = 8;
/// `AuthenticatorTransport` values (§5.8.4) we keep; others are dropped.
pub(crate) const TRANSPORTS: [&str; 6] = ["usb", "nfc", "ble", "smart-card", "hybrid", "internal"];

/// authenticatorData flag bits (§6.1).
const FLAG_UP: u8 = 0x01;
const FLAG_UV: u8 = 0x04;
const FLAG_BE: u8 = 0x08;
const FLAG_BS: u8 = 0x10;
const FLAG_AT: u8 = 0x40;
const FLAG_ED: u8 = 0x80;

const TYPE_CREATE: &str = "webauthn.create";
const TYPE_GET: &str = "webauthn.get";

/// What the relying party expects: its RP ID and origin (from
/// `CALRS_BASE_URL`) and the challenge of this ceremony.
#[derive(Clone, Copy)]
pub(crate) struct Expected<'a> {
    pub rp_id: &'a str,
    pub origin: &'a str,
    pub challenge: &'a [u8],
}

/// A verified new credential, ready to store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Registration {
    pub credential_id: Vec<u8>,
    /// SPKI DER as re-encoded from the parsed key.
    pub public_key: Vec<u8>,
    pub algorithm: i64,
    pub sign_count: u32,
    pub user_verified: bool,
    pub backup_eligible: bool,
    pub backup_state: bool,
    /// `None` when the authenticator (or the client, under `none`
    /// attestation) gave all zeros.
    pub aaguid: Option<[u8; 16]>,
}

/// A stored credential as an assertion is checked against it.
#[derive(Clone, Copy)]
pub(crate) struct StoredKey<'a> {
    pub public_key: &'a [u8],
    pub algorithm: i64,
    pub sign_count: u32,
    pub backup_eligible: bool,
}

/// A verified assertion: what the stored credential is updated with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Assertion {
    pub sign_count: u32,
    pub user_verified: bool,
    pub backup_state: bool,
}

/// Why a ceremony was refused: one variant per check, so a log line or a
/// test names the step. None of them carries the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Error {
    /// A form field is not base64url without padding, or is too long.
    FieldEncoding,
    ClientDataSize,
    AuthDataSize,
    SignatureSize,
    PublicKeySize,
    CredentialIdSize,
    UserHandleSize,
    /// clientDataJSON is not UTF-8, or starts with a BOM.
    ClientDataEncoding,
    /// clientDataJSON is not one JSON object with the required string fields.
    ClientDataJson,
    ClientDataType,
    Challenge,
    Origin,
    CrossOrigin,
    TopOrigin,
    RpIdHash,
    UserNotPresent,
    BackupStateWithoutEligibility,
    BackupEligibilityChanged,
    AttestedDataMissing,
    AttestedDataUnexpected,
    /// authenticatorData does not have the shape its flags announce.
    AuthDataMalformed,
    CredentialIdMismatch,
    UnsupportedAlgorithm,
    /// The SPKI does not parse as the key type the algorithm implies.
    PublicKey,
    RsaKeySize,
    WeakKey,
    /// The signature is not well formed for the algorithm.
    SignatureEncoding,
    BadSignature,
    CounterRegression,
    UserHandleMismatch,
}

impl Error {
    /// A stable short code for logs (`reason = …`).
    pub(crate) fn code(self) -> &'static str {
        match self {
            Error::FieldEncoding => "field_encoding",
            Error::ClientDataSize => "client_data_size",
            Error::AuthDataSize => "auth_data_size",
            Error::SignatureSize => "signature_size",
            Error::PublicKeySize => "public_key_size",
            Error::CredentialIdSize => "credential_id_size",
            Error::UserHandleSize => "user_handle_size",
            Error::ClientDataEncoding => "client_data_encoding",
            Error::ClientDataJson => "client_data_json",
            Error::ClientDataType => "client_data_type",
            Error::Challenge => "challenge",
            Error::Origin => "origin",
            Error::CrossOrigin => "cross_origin",
            Error::TopOrigin => "top_origin",
            Error::RpIdHash => "rp_id_hash",
            Error::UserNotPresent => "user_not_present",
            Error::BackupStateWithoutEligibility => "backup_state_without_eligibility",
            Error::BackupEligibilityChanged => "backup_eligibility_changed",
            Error::AttestedDataMissing => "attested_data_missing",
            Error::AttestedDataUnexpected => "attested_data_unexpected",
            Error::AuthDataMalformed => "auth_data_malformed",
            Error::CredentialIdMismatch => "credential_id_mismatch",
            Error::UnsupportedAlgorithm => "unsupported_algorithm",
            Error::PublicKey => "public_key",
            Error::RsaKeySize => "rsa_key_size",
            Error::WeakKey => "weak_key",
            Error::SignatureEncoding => "signature_encoding",
            Error::BadSignature => "bad_signature",
            Error::CounterRegression => "counter_regression",
            Error::UserHandleMismatch => "user_handle_mismatch",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "webauthn: {}", self.code())
    }
}

impl std::error::Error for Error {}

/// Decodes one base64url form field (no padding), refusing more than `max`
/// decoded bytes before decoding anything.
pub(crate) fn decode_field(value: &str, max: usize) -> Result<Vec<u8>, Error> {
    // 4 characters carry 3 bytes; anything longer cannot fit in `max`.
    if value.len() > max.div_ceil(3).saturating_mul(4) {
        return Err(Error::FieldEncoding);
    }
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| Error::FieldEncoding)?;
    if bytes.len() > max {
        return Err(Error::FieldEncoding);
    }
    Ok(bytes)
}

/// The challenge a clientDataJSON carries, to find the ceremony it answers.
/// Only the shape is checked here; [`verify_registration`] and
/// [`verify_assertion`] check everything again against that ceremony.
pub(crate) fn client_challenge(client_data: &[u8]) -> Result<Vec<u8>, Error> {
    let parsed = parse_client_data(client_data)?;
    decode_challenge(&parsed.challenge)
}

/// §6.1.1: a counter that is not zero on both sides must grow. Anything
/// else may be a cloned authenticator.
pub(crate) fn counter_ok(stored: u32, received: u32) -> bool {
    (stored == 0 && received == 0) || received > stored
}

/// §7.2 step 6: when the authenticator returns a user handle it must be
/// this user's. An empty field means none was returned.
pub(crate) fn user_handle_matches(received: &[u8], expected: &[u8]) -> Result<(), Error> {
    if received.len() > MAX_USER_HANDLE {
        return Err(Error::UserHandleSize);
    }
    if received.is_empty() || bool::from(received.ct_eq(expected)) {
        Ok(())
    } else {
        Err(Error::UserHandleMismatch)
    }
}

/// The transports `getTransports()` reported, as a JSON array of strings:
/// known values only, each once, at most [`MAX_TRANSPORTS`]. They are hints
/// for `allowCredentials`, so anything malformed gives none.
pub(crate) fn sanitize_transports(raw: &str) -> Vec<&'static str> {
    if raw.len() > 512 {
        return Vec::new();
    }
    let Ok(values) = serde_json::from_str::<Vec<String>>(raw) else {
        return Vec::new();
    };
    let mut out: Vec<&'static str> = Vec::new();
    for value in &values {
        if let Some(known) = TRANSPORTS.iter().find(|t| **t == value.as_str()) {
            if !out.contains(known) && out.len() < MAX_TRANSPORTS {
                out.push(known);
            }
        }
    }
    out
}

/// §7.1: verifies a registration (`navigator.credentials.create()`).
pub(crate) fn verify_registration(
    expected: &Expected<'_>,
    client_data: &[u8],
    auth_data: &[u8],
    public_key: &[u8],
    algorithm: i64,
    raw_id: &[u8],
) -> Result<Registration, Error> {
    check_sizes(client_data, auth_data)?;
    if public_key.is_empty() || public_key.len() > MAX_PUBLIC_KEY {
        return Err(Error::PublicKeySize);
    }
    if raw_id.is_empty() || raw_id.len() > MAX_CREDENTIAL_ID {
        return Err(Error::CredentialIdSize);
    }

    check_client_data(expected, client_data, TYPE_CREATE)?;
    let data = parse_auth_data(auth_data)?;
    check_common_flags(expected, &data)?;

    if data.flags & FLAG_AT == 0 {
        return Err(Error::AttestedDataMissing);
    }
    let attested = parse_attested(data.rest)?;
    check_extensions(data.flags, attested.extensions)?;
    if attested.credential_id != raw_id {
        return Err(Error::CredentialIdMismatch);
    }

    let key = PublicKey::parse(algorithm, public_key)?;
    let aaguid = if attested.aaguid == [0u8; 16] {
        None
    } else {
        Some(attested.aaguid)
    };
    Ok(Registration {
        credential_id: raw_id.to_vec(),
        public_key: key.to_der()?,
        algorithm,
        sign_count: data.sign_count,
        user_verified: data.flags & FLAG_UV != 0,
        backup_eligible: data.flags & FLAG_BE != 0,
        backup_state: data.flags & FLAG_BS != 0,
        aaguid,
    })
}

/// §7.2: verifies an assertion (`navigator.credentials.get()`) made with a
/// stored credential.
pub(crate) fn verify_assertion(
    expected: &Expected<'_>,
    key: &StoredKey<'_>,
    client_data: &[u8],
    auth_data: &[u8],
    signature: &[u8],
) -> Result<Assertion, Error> {
    check_sizes(client_data, auth_data)?;
    if signature.is_empty() || signature.len() > MAX_SIGNATURE {
        return Err(Error::SignatureSize);
    }
    if key.public_key.is_empty() || key.public_key.len() > MAX_PUBLIC_KEY {
        return Err(Error::PublicKeySize);
    }

    check_client_data(expected, client_data, TYPE_GET)?;
    let data = parse_auth_data(auth_data)?;
    check_common_flags(expected, &data)?;

    if data.flags & FLAG_AT != 0 {
        return Err(Error::AttestedDataUnexpected);
    }
    check_extensions(data.flags, data.rest)?;
    let backup_eligible = data.flags & FLAG_BE != 0;
    if backup_eligible != key.backup_eligible {
        return Err(Error::BackupEligibilityChanged);
    }

    let public = PublicKey::parse(key.algorithm, key.public_key)?;
    let mut message = Vec::with_capacity(auth_data.len() + 32);
    message.extend_from_slice(auth_data);
    message.extend_from_slice(&Sha256::digest(client_data));
    public.verify(&message, signature)?;

    if !counter_ok(key.sign_count, data.sign_count) {
        return Err(Error::CounterRegression);
    }
    Ok(Assertion {
        sign_count: data.sign_count,
        user_verified: data.flags & FLAG_UV != 0,
        backup_state: data.flags & FLAG_BS != 0,
    })
}

fn check_sizes(client_data: &[u8], auth_data: &[u8]) -> Result<(), Error> {
    if client_data.is_empty() || client_data.len() > MAX_CLIENT_DATA {
        return Err(Error::ClientDataSize);
    }
    if auth_data.len() < MIN_AUTH_DATA || auth_data.len() > MAX_AUTH_DATA {
        return Err(Error::AuthDataSize);
    }
    Ok(())
}

/// A JSON member that only has to be noticed: any value, `null` included,
/// counts as present.
#[derive(Default)]
struct Present(Option<serde_json::Value>);

impl<'de> Deserialize<'de> for Present {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        serde_json::Value::deserialize(d).map(|v| Present(Some(v)))
    }
}

/// §5.8.1 `CollectedClientData`. serde refuses a known field given twice,
/// so a second `origin` cannot shadow the first.
#[derive(Deserialize)]
struct ClientData {
    #[serde(rename = "type")]
    kind: String,
    challenge: String,
    origin: String,
    #[serde(default, rename = "crossOrigin")]
    cross_origin: Present,
    #[serde(default, rename = "topOrigin")]
    top_origin: Present,
}

fn parse_client_data(client_data: &[u8]) -> Result<ClientData, Error> {
    if client_data.is_empty() || client_data.len() > MAX_CLIENT_DATA {
        return Err(Error::ClientDataSize);
    }
    let text = std::str::from_utf8(client_data).map_err(|_| Error::ClientDataEncoding)?;
    if text.starts_with('\u{feff}') {
        return Err(Error::ClientDataEncoding);
    }
    // An object, not an array: serde would map an array onto the struct
    // by position.
    if !text.starts_with('{') {
        return Err(Error::ClientDataJson);
    }
    serde_json::from_str(text).map_err(|_| Error::ClientDataJson)
}

fn decode_challenge(value: &str) -> Result<Vec<u8>, Error> {
    let bytes = decode_field(value, MAX_CHALLENGE).map_err(|_| Error::Challenge)?;
    if bytes.len() < MIN_CHALLENGE {
        return Err(Error::Challenge);
    }
    Ok(bytes)
}

fn check_client_data(expected: &Expected<'_>, client_data: &[u8], kind: &str) -> Result<(), Error> {
    let parsed = parse_client_data(client_data)?;
    if parsed.kind != kind {
        return Err(Error::ClientDataType);
    }
    let challenge = decode_challenge(&parsed.challenge)?;
    // Fail closed on a caller that lost its challenge or origin.
    if expected.challenge.len() < MIN_CHALLENGE
        || !bool::from(challenge.as_slice().ct_eq(expected.challenge))
    {
        return Err(Error::Challenge);
    }
    if expected.origin.is_empty() || parsed.origin != expected.origin {
        return Err(Error::Origin);
    }
    match parsed.cross_origin.0 {
        None | Some(serde_json::Value::Bool(false)) => {}
        Some(_) => return Err(Error::CrossOrigin),
    }
    if parsed.top_origin.0.is_some() {
        return Err(Error::TopOrigin);
    }
    Ok(())
}

struct AuthData<'a> {
    rp_id_hash: &'a [u8],
    flags: u8,
    sign_count: u32,
    /// Attested credential data and extensions, as the flags announce.
    rest: &'a [u8],
}

fn parse_auth_data(auth_data: &[u8]) -> Result<AuthData<'_>, Error> {
    let rp_id_hash = auth_data.get(..32).ok_or(Error::AuthDataSize)?;
    let flags = *auth_data.get(32).ok_or(Error::AuthDataSize)?;
    let counter: [u8; 4] = auth_data
        .get(33..37)
        .and_then(|c| c.try_into().ok())
        .ok_or(Error::AuthDataSize)?;
    let rest = auth_data.get(37..).ok_or(Error::AuthDataSize)?;
    Ok(AuthData {
        rp_id_hash,
        flags,
        sign_count: u32::from_be_bytes(counter),
        rest,
    })
}

fn check_common_flags(expected: &Expected<'_>, data: &AuthData<'_>) -> Result<(), Error> {
    let want = Sha256::digest(expected.rp_id.as_bytes());
    if expected.rp_id.is_empty() || !bool::from(data.rp_id_hash.ct_eq(want.as_slice())) {
        return Err(Error::RpIdHash);
    }
    if data.flags & FLAG_UP == 0 {
        return Err(Error::UserNotPresent);
    }
    if data.flags & FLAG_BS != 0 && data.flags & FLAG_BE == 0 {
        return Err(Error::BackupStateWithoutEligibility);
    }
    Ok(())
}

struct Attested<'a> {
    aaguid: [u8; 16],
    credential_id: &'a [u8],
    /// What follows the credential public key.
    extensions: &'a [u8],
}

/// §6.5.1: `aaguid(16) | credentialIdLength(2, big-endian) | credentialId |
/// credentialPublicKey (COSE, CBOR) | extensions`.
fn parse_attested(rest: &[u8]) -> Result<Attested<'_>, Error> {
    let aaguid: [u8; 16] = rest
        .get(..16)
        .and_then(|a| a.try_into().ok())
        .ok_or(Error::AuthDataMalformed)?;
    let length: [u8; 2] = rest
        .get(16..18)
        .and_then(|l| l.try_into().ok())
        .ok_or(Error::AuthDataMalformed)?;
    let length = usize::from(u16::from_be_bytes(length));
    if length == 0 || length > MAX_CREDENTIAL_ID {
        return Err(Error::CredentialIdSize);
    }
    let end = 18usize
        .checked_add(length)
        .ok_or(Error::AuthDataMalformed)?;
    let credential_id = rest.get(18..end).ok_or(Error::AuthDataMalformed)?;
    let key = rest.get(end..).ok_or(Error::AuthDataMalformed)?;
    let after_key = skip_cbor_map(key).ok_or(Error::AuthDataMalformed)?;
    Ok(Attested {
        aaguid,
        credential_id,
        extensions: after_key,
    })
}

/// Nested CBOR deeper than this is refused (a COSE key is flat; extension
/// outputs nest a level or two).
const MAX_CBOR_DEPTH: u8 = 8;

/// Skips one non-empty CBOR map and returns what follows it.
fn skip_cbor_map(bytes: &[u8]) -> Option<&[u8]> {
    let first = *bytes.first()?;
    // Major type 5 (map), and not the empty map 0xa0.
    if first & 0xe0 != 0xa0 || first == 0xa0 {
        return None;
    }
    skip_cbor(bytes, 0)
}

/// Skips one well-formed CBOR data item (RFC 8949 §3) and returns what
/// follows it. Nothing is decoded: this only proves the authenticator data
/// has the shape its flags announce. Definite lengths only (CTAP2 canonical
/// CBOR has no others), text must be UTF-8, nesting is capped. Every item
/// consumes at least one byte, so a huge announced count ends as soon as
/// the input does.
pub(super) fn skip_cbor(bytes: &[u8], depth: u8) -> Option<&[u8]> {
    if depth > MAX_CBOR_DEPTH {
        return None;
    }
    let (&initial, rest) = bytes.split_first()?;
    let major = initial >> 5;
    let info = initial & 0x1f;
    let (argument, rest) = match info {
        0..=23 => (u64::from(info), rest),
        24 => {
            let (&byte, rest) = rest.split_first()?;
            (u64::from(byte), rest)
        }
        25 => (
            u64::from(u16::from_be_bytes(rest.get(..2)?.try_into().ok()?)),
            rest.get(2..)?,
        ),
        26 => (
            u64::from(u32::from_be_bytes(rest.get(..4)?.try_into().ok()?)),
            rest.get(4..)?,
        ),
        27 => (
            u64::from_be_bytes(rest.get(..8)?.try_into().ok()?),
            rest.get(8..)?,
        ),
        // 28-30 are reserved, 31 is an indefinite length.
        _ => return None,
    };
    match major {
        0 | 1 => Some(rest),
        2 | 3 => {
            let length = usize::try_from(argument).ok()?;
            let content = rest.get(..length)?;
            if major == 3 {
                std::str::from_utf8(content).ok()?;
            }
            rest.get(length..)
        }
        4 | 5 => {
            let items = if major == 5 {
                argument.checked_mul(2)?
            } else {
                argument
            };
            let mut rest = rest;
            for _ in 0..items {
                rest = skip_cbor(rest, depth + 1)?;
            }
            Some(rest)
        }
        6 => skip_cbor(rest, depth + 1),
        // Simple values and floats; a one-byte simple value below 32 is
        // not well formed (RFC 8949 §3.3).
        _ => (info != 24 || argument >= 32).then_some(rest),
    }
}

/// What follows the attested credential data or the counter: nothing
/// without ED, exactly one extensions map with it.
fn check_extensions(flags: u8, rest: &[u8]) -> Result<(), Error> {
    let trailing = if flags & FLAG_ED == 0 {
        rest
    } else {
        skip_cbor_map(rest).ok_or(Error::AuthDataMalformed)?
    };
    if trailing.is_empty() {
        Ok(())
    } else {
        Err(Error::AuthDataMalformed)
    }
}

enum PublicKey {
    Ed25519(ed25519_dalek::VerifyingKey),
    P256(p256::ecdsa::VerifyingKey),
    Rsa(rsa::RsaPublicKey),
}

impl PublicKey {
    fn parse(algorithm: i64, spki: &[u8]) -> Result<PublicKey, Error> {
        use p256::pkcs8::DecodePublicKey;
        use rsa::traits::PublicKeyParts;
        match algorithm {
            ALG_EDDSA => {
                let key = ed25519_dalek::VerifyingKey::from_public_key_der(spki)
                    .map_err(|_| Error::PublicKey)?;
                if key.is_weak() {
                    return Err(Error::WeakKey);
                }
                Ok(PublicKey::Ed25519(key))
            }
            ALG_ES256 => p256::ecdsa::VerifyingKey::from_public_key_der(spki)
                .map(PublicKey::P256)
                .map_err(|_| Error::PublicKey),
            ALG_RS256 => {
                let key =
                    rsa::RsaPublicKey::from_public_key_der(spki).map_err(|_| Error::PublicKey)?;
                let bits = key.n().bits();
                if !(RSA_MIN_BITS..=RSA_MAX_BITS).contains(&bits) {
                    return Err(Error::RsaKeySize);
                }
                Ok(PublicKey::Rsa(key))
            }
            _ => Err(Error::UnsupportedAlgorithm),
        }
    }

    fn to_der(&self) -> Result<Vec<u8>, Error> {
        use p256::pkcs8::EncodePublicKey;
        let document = match self {
            PublicKey::Ed25519(key) => key.to_public_key_der(),
            PublicKey::P256(key) => key.to_public_key_der(),
            PublicKey::Rsa(key) => key.to_public_key_der(),
        };
        document
            .map(|d| d.as_bytes().to_vec())
            .map_err(|_| Error::PublicKey)
    }

    fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), Error> {
        use p256::ecdsa::signature::Verifier;
        match self {
            PublicKey::Ed25519(key) => {
                let signature = ed25519_dalek::Signature::from_slice(signature)
                    .map_err(|_| Error::SignatureEncoding)?;
                key.verify_strict(message, &signature)
                    .map_err(|_| Error::BadSignature)
            }
            PublicKey::P256(key) => {
                // DER as §6.5.5 requires; a high S is valid ECDSA and is
                // accepted (the §16.2 vector has one).
                let signature = p256::ecdsa::Signature::from_der(signature)
                    .map_err(|_| Error::SignatureEncoding)?;
                key.verify(message, &signature)
                    .map_err(|_| Error::BadSignature)
            }
            PublicKey::Rsa(key) => {
                let signature = rsa::pkcs1v15::Signature::try_from(signature)
                    .map_err(|_| Error::SignatureEncoding)?;
                rsa::pkcs1v15::VerifyingKey::<Sha256>::new(key.clone())
                    .verify(message, &signature)
                    .map_err(|_| Error::BadSignature)
            }
        }
    }
}
