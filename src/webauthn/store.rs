//! ilia fork: passkey credentials and WebAuthn ceremonies in SQLite
//! (`migrations/ilia_010_webauthn.sql`). Every function takes the caller's
//! connection, so it runs inside the caller's transaction, after the caller
//! took the write lock.

use anyhow::Result;
use chrono::Utc;
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::SqliteConnection;

use super::verify::{self, Registration};
use super::{Rp, MAX_KEYS};

/// What a ceremony is for. `Login` and `Enroll` belong to an MFA challenge
/// (no session yet), `Confirm` and `Register` to a session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Login,
    Enroll,
    Confirm,
    Register,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Login => "login",
            Kind::Enroll => "enroll",
            Kind::Confirm => "confirm",
            Kind::Register => "register",
        }
    }
}

/// What a ceremony is bound to: the hash of the MFA challenge token, or the
/// session id (the session cookie's value, as `sessions.id` stores it).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Binding<'a> {
    Challenge(&'a str),
    Session(&'a str),
}

fn hash_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Has TOTP or at least one passkey. `?1` is the user id. Every place that
/// asks whether an account has MFA uses this.
pub(crate) const HAS_FACTOR_SQL: &str = "(EXISTS (SELECT 1 FROM user_mfa WHERE user_id = ?1)
     OR EXISTS (SELECT 1 FROM ilia_webauthn_credentials WHERE user_id = ?1))";

pub(crate) async fn has_factor(conn: &mut SqliteConnection, user_id: &str) -> Result<bool> {
    Ok(sqlx::query_scalar(&format!("SELECT {HAS_FACTOR_SQL}"))
        .bind(user_id)
        .fetch_one(conn)
        .await?)
}

pub(crate) async fn has_totp(conn: &mut SqliteConnection, user_id: &str) -> Result<bool> {
    Ok(
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_mfa WHERE user_id = ?)")
            .bind(user_id)
            .fetch_one(conn)
            .await?,
    )
}

pub(crate) async fn key_count(conn: &mut SqliteConnection, user_id: &str) -> Result<i64> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM ilia_webauthn_credentials WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(conn)
            .await?,
    )
}

pub(crate) async fn recovery_codes_left(conn: &mut SqliteConnection, user_id: &str) -> Result<i64> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM mfa_recovery_codes WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(conn)
            .await?,
    )
}

/// Starts a ceremony: 32 random bytes, of which only the SHA-256 is
/// stored. Earlier ceremonies of the same kind on the same binding, and
/// every expired one, are deleted first, so only the latest survives. A
/// challenge-bound ceremony never outlives its MFA challenge. Returns the
/// raw challenge for the options JSON, or `None` when the binding is gone.
pub(crate) async fn begin(
    conn: &mut SqliteConnection,
    user_id: &str,
    kind: Kind,
    binding: Binding<'_>,
) -> Result<Option<[u8; 32]>> {
    let now = Utc::now().timestamp();
    sqlx::query("DELETE FROM ilia_webauthn_ceremonies WHERE expires_at <= ?")
        .bind(now)
        .execute(&mut *conn)
        .await?;
    let mut challenge = [0u8; 32];
    OsRng.fill_bytes(&mut challenge);
    let hash = hash_hex(&challenge);
    let inserted = match binding {
        Binding::Challenge(token_hash) => {
            sqlx::query(
                "DELETE FROM ilia_webauthn_ceremonies WHERE kind = ? AND mfa_token_hash = ?",
            )
            .bind(kind.as_str())
            .bind(token_hash)
            .execute(&mut *conn)
            .await?;
            sqlx::query(
                "INSERT INTO ilia_webauthn_ceremonies
                     (challenge_hash, user_id, kind, mfa_token_hash, password_hash, expires_at)
                 SELECT ?, c.user_id, ?, c.token_hash, c.password_hash, MIN(?, c.expires_at)
                 FROM mfa_challenges c WHERE c.token_hash = ? AND c.user_id = ?",
            )
            .bind(&hash)
            .bind(kind.as_str())
            .bind(now + super::CEREMONY_SECONDS)
            .bind(token_hash)
            .bind(user_id)
            .execute(&mut *conn)
            .await?
        }
        Binding::Session(session_id) => {
            sqlx::query("DELETE FROM ilia_webauthn_ceremonies WHERE kind = ? AND session_id = ?")
                .bind(kind.as_str())
                .bind(session_id)
                .execute(&mut *conn)
                .await?;
            sqlx::query(
                "INSERT INTO ilia_webauthn_ceremonies
                     (challenge_hash, user_id, kind, session_id, password_hash, expires_at)
                 SELECT ?, u.id, ?, s.id, u.password_hash, ?
                 FROM sessions s JOIN users u ON u.id = s.user_id
                 WHERE s.id = ? AND u.id = ? AND u.password_hash IS NOT NULL",
            )
            .bind(&hash)
            .bind(kind.as_str())
            .bind(now + super::CEREMONY_SECONDS)
            .bind(session_id)
            .bind(user_id)
            .execute(&mut *conn)
            .await?
        }
    };
    Ok((inserted.rows_affected() == 1).then_some(challenge))
}

/// Uses up the ceremony `challenge` answers, whatever happens next: the
/// caller commits this delete even when verification then fails, so a
/// challenge is never tried twice. True when a live ceremony of `kind`,
/// bound to `binding`, for `user_id`, under the user's current password
/// hash, was there.
pub(crate) async fn burn(
    conn: &mut SqliteConnection,
    challenge: &[u8],
    kind: Kind,
    binding: Binding<'_>,
    user_id: &str,
) -> Result<bool> {
    let (column, value) = match binding {
        Binding::Challenge(token_hash) => ("mfa_token_hash", token_hash),
        Binding::Session(session_id) => ("session_id", session_id),
    };
    let deleted = sqlx::query(&format!(
        "DELETE FROM ilia_webauthn_ceremonies
         WHERE challenge_hash = ? AND kind = ? AND {column} = ? AND user_id = ? AND expires_at > ?
           AND password_hash = (SELECT password_hash FROM users WHERE id = ?)"
    ))
    .bind(hash_hex(challenge))
    .bind(kind.as_str())
    .bind(value)
    .bind(user_id)
    .bind(Utc::now().timestamp())
    .bind(user_id)
    .execute(conn)
    .await?;
    Ok(deleted.rows_affected() == 1)
}

/// A registered credential.
#[derive(sqlx::FromRow, Clone, Debug)]
pub(crate) struct Credential {
    pub id: String,
    pub rp_id: String,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub algorithm: i64,
    pub sign_count: i64,
    pub backup_eligible: bool,
    pub backup_state: bool,
    pub transports: String,
    pub name: String,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}

const COLUMNS: &str = "id, rp_id, credential_id, public_key, algorithm, sign_count,
    backup_eligible, backup_state, transports, name, created_at, last_used_at";

pub(crate) async fn list(conn: &mut SqliteConnection, user_id: &str) -> Result<Vec<Credential>> {
    Ok(sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM ilia_webauthn_credentials WHERE user_id = ?
         ORDER BY created_at, id"
    ))
    .bind(user_id)
    .fetch_all(conn)
    .await?)
}

/// The user's keys registered for this RP ID: the ones a ceremony here can
/// use.
pub(crate) async fn usable(
    conn: &mut SqliteConnection,
    user_id: &str,
    rp: &Rp,
) -> Result<Vec<Credential>> {
    Ok(list(conn, user_id)
        .await?
        .into_iter()
        .filter(|key| key.rp_id == rp.id)
        .collect())
}

pub(crate) async fn get(
    conn: &mut SqliteConnection,
    user_id: &str,
    id: &str,
) -> Result<Option<Credential>> {
    Ok(sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM ilia_webauthn_credentials WHERE id = ? AND user_id = ?"
    ))
    .bind(id)
    .bind(user_id)
    .fetch_optional(conn)
    .await?)
}

async fn by_credential_id(
    conn: &mut SqliteConnection,
    user_id: &str,
    rp_id: &str,
    credential_id: &[u8],
) -> Result<Option<Credential>> {
    Ok(sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM ilia_webauthn_credentials
         WHERE credential_id = ? AND user_id = ? AND rp_id = ?"
    ))
    .bind(credential_id)
    .bind(user_id)
    .bind(rp_id)
    .fetch_optional(conn)
    .await?)
}

/// Why a new credential was not stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotAdded {
    /// This credential ID is registered already, to this user or another.
    Exists,
    Limit,
}

pub(crate) async fn add(
    conn: &mut SqliteConnection,
    user_id: &str,
    rp: &Rp,
    registration: &Registration,
    transports: &str,
    name: &str,
) -> Result<std::result::Result<String, NotAdded>> {
    if key_count(&mut *conn, user_id).await? >= MAX_KEYS {
        return Ok(Err(NotAdded::Limit));
    }
    let transports = serde_json::to_string(&verify::sanitize_transports(transports))?;
    let id = uuid::Uuid::new_v4().to_string();
    let inserted = sqlx::query(
        "INSERT INTO ilia_webauthn_credentials
             (id, user_id, rp_id, credential_id, public_key, algorithm, sign_count,
              uv_initialized, backup_eligible, backup_state, transports, aaguid, name, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(credential_id) DO NOTHING",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&rp.id)
    .bind(&registration.credential_id)
    .bind(&registration.public_key)
    .bind(registration.algorithm)
    .bind(i64::from(registration.sign_count))
    .bind(registration.user_verified)
    .bind(registration.backup_eligible)
    .bind(registration.backup_state)
    .bind(transports)
    .bind(registration.aaguid.map(|a| a.to_vec()))
    .bind(name)
    .bind(Utc::now().timestamp())
    .execute(conn)
    .await?;
    Ok(if inserted.rows_affected() == 1 {
        Ok(id)
    } else {
        Err(NotAdded::Exists)
    })
}

pub(crate) async fn rename(
    conn: &mut SqliteConnection,
    user_id: &str,
    id: &str,
    name: &str,
) -> Result<bool> {
    Ok(
        sqlx::query("UPDATE ilia_webauthn_credentials SET name = ? WHERE id = ? AND user_id = ?")
            .bind(name)
            .bind(id)
            .bind(user_id)
            .execute(conn)
            .await?
            .rows_affected()
            == 1,
    )
}

pub(crate) async fn delete(conn: &mut SqliteConnection, user_id: &str, id: &str) -> Result<bool> {
    Ok(
        sqlx::query("DELETE FROM ilia_webauthn_credentials WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(conn)
            .await?
            .rows_affected()
            == 1,
    )
}

/// The default name of a new key: "Security key N" for one bound to a
/// device, "Passkey N" for a synced one, N one past the user's count.
pub(crate) async fn default_name(
    conn: &mut SqliteConnection,
    user_id: &str,
    lang: &str,
    synced: bool,
) -> Result<String> {
    let n = key_count(conn, user_id).await? + 1;
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("n", n);
    let key = if synced {
        "ilia-passkey-default-device"
    } else {
        "ilia-passkey-default-security-key"
    };
    Ok(crate::i18n::translate(lang, key, Some(&args)))
}

/// The decoded fields of an assertion, sizes capped before decoding.
struct Decoded {
    credential_id: Vec<u8>,
    client_data: Vec<u8>,
    auth_data: Vec<u8>,
    signature: Vec<u8>,
    user_handle: Vec<u8>,
}

fn decode(fields: &super::AssertionFields) -> std::result::Result<Decoded, verify::Error> {
    Ok(Decoded {
        credential_id: verify::decode_field(&fields.credential_id, verify::MAX_CREDENTIAL_ID)?,
        client_data: verify::decode_field(&fields.client_data, verify::MAX_CLIENT_DATA)?,
        auth_data: verify::decode_field(&fields.authenticator_data, verify::MAX_AUTH_DATA)?,
        signature: verify::decode_field(&fields.signature, verify::MAX_SIGNATURE)?,
        user_handle: verify::decode_field(&fields.user_handle, verify::MAX_USER_HANDLE)?,
    })
}

/// Why an assertion was refused, for the log line. `Verify` names the
/// verifier's step; the others are about the stored state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    Verify(verify::Error),
    /// No live ceremony of this kind on this binding answers the challenge
    /// (replayed, expired, another login's, or never issued).
    NoCeremony,
    /// Not one of this user's keys for this RP ID.
    UnknownCredential,
    /// Another assertion of the same key landed first (the counter moved).
    Raced,
}

impl Refusal {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Refusal::Verify(e) => e.code(),
            Refusal::NoCeremony => "no_ceremony",
            Refusal::UnknownCredential => "unknown_credential",
            Refusal::Raced => "raced",
        }
    }
}

/// Checks a passkey assertion for `user_id` and records it: burns the
/// ceremony, verifies against the stored key, checks the user handle, and
/// moves the counter with a compare-and-swap. Returns the credential's row
/// id. The caller commits whatever the outcome (the burn must stick) and
/// creates nothing on `Err`.
pub(crate) async fn check_assertion(
    conn: &mut SqliteConnection,
    rp: &Rp,
    user_id: &str,
    fields: &super::AssertionFields,
    kind: Kind,
    binding: Binding<'_>,
) -> Result<std::result::Result<String, Refusal>> {
    let decoded = match decode(fields) {
        Ok(d) => d,
        Err(e) => return Ok(Err(Refusal::Verify(e))),
    };
    let challenge = match verify::client_challenge(&decoded.client_data) {
        Ok(c) => c,
        Err(e) => return Ok(Err(Refusal::Verify(e))),
    };
    if !burn(&mut *conn, &challenge, kind, binding, user_id).await? {
        return Ok(Err(Refusal::NoCeremony));
    }
    let Some(key) = by_credential_id(&mut *conn, user_id, &rp.id, &decoded.credential_id).await?
    else {
        return Ok(Err(Refusal::UnknownCredential));
    };
    let stored_count = u32::try_from(key.sign_count).unwrap_or(u32::MAX);
    let expected = verify::Expected {
        rp_id: &rp.id,
        origin: &rp.origin,
        challenge: &challenge,
    };
    let stored = verify::StoredKey {
        public_key: &key.public_key,
        algorithm: key.algorithm,
        sign_count: stored_count,
        backup_eligible: key.backup_eligible,
    };
    let assertion = match verify::verify_assertion(
        &expected,
        &stored,
        &decoded.client_data,
        &decoded.auth_data,
        &decoded.signature,
    )
    .and_then(|a| {
        verify::user_handle_matches(&decoded.user_handle, &super::user_handle(user_id)).map(|_| a)
    }) {
        Ok(a) => a,
        Err(e) => {
            if e == verify::Error::CounterRegression {
                tracing::warn!(
                    user_id = %user_id, credential = %key.id,
                    stored = stored_count,
                    "passkey refused: possible cloned authenticator (signature counter did not grow)"
                );
            }
            return Ok(Err(Refusal::Verify(e)));
        }
    };
    let updated = sqlx::query(
        "UPDATE ilia_webauthn_credentials
         SET sign_count = ?, backup_state = ?, uv_initialized = MAX(uv_initialized, ?),
             last_used_at = ?
         WHERE id = ? AND user_id = ? AND sign_count = ?",
    )
    .bind(i64::from(assertion.sign_count))
    .bind(assertion.backup_state)
    .bind(assertion.user_verified)
    .bind(Utc::now().timestamp())
    .bind(&key.id)
    .bind(user_id)
    .bind(key.sign_count)
    .execute(conn)
    .await?;
    if updated.rows_affected() != 1 {
        return Ok(Err(Refusal::Raced));
    }
    Ok(Ok(key.id))
}

/// A registration as posted, decoded and checked against the ceremony it
/// answers (which this burns). The caller commits the burn whatever the
/// outcome.
pub(crate) async fn check_registration(
    conn: &mut SqliteConnection,
    rp: &Rp,
    user_id: &str,
    fields: &super::RegistrationFields,
    kind: Kind,
    binding: Binding<'_>,
) -> Result<std::result::Result<Registration, Refusal>> {
    let decoded = (|| {
        let raw_id = verify::decode_field(&fields.credential_id, verify::MAX_CREDENTIAL_ID)?;
        let client_data = verify::decode_field(&fields.client_data, verify::MAX_CLIENT_DATA)?;
        let auth_data = verify::decode_field(&fields.authenticator_data, verify::MAX_AUTH_DATA)?;
        let public_key = verify::decode_field(&fields.public_key, verify::MAX_PUBLIC_KEY)?;
        let algorithm: i64 = fields
            .algorithm
            .trim()
            .parse()
            .map_err(|_| verify::Error::UnsupportedAlgorithm)?;
        Ok::<_, verify::Error>((raw_id, client_data, auth_data, public_key, algorithm))
    })();
    let (raw_id, client_data, auth_data, public_key, algorithm) = match decoded {
        Ok(d) => d,
        Err(e) => return Ok(Err(Refusal::Verify(e))),
    };
    let challenge = match verify::client_challenge(&client_data) {
        Ok(c) => c,
        Err(e) => return Ok(Err(Refusal::Verify(e))),
    };
    if !burn(&mut *conn, &challenge, kind, binding, user_id).await? {
        return Ok(Err(Refusal::NoCeremony));
    }
    let expected = verify::Expected {
        rp_id: &rp.id,
        origin: &rp.origin,
        challenge: &challenge,
    };
    Ok(verify::verify_registration(
        &expected,
        &client_data,
        &auth_data,
        &public_key,
        algorithm,
        &raw_id,
    )
    .map_err(Refusal::Verify))
}
