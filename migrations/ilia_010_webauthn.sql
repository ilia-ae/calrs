-- ilia fork: passkeys and security keys (WebAuthn) as a second factor
-- (docs/passkeys-plan.md). A local account's second factor is TOTP or a
-- passkey, or both.

-- Recovery codes belong to the user, not to the TOTP row: an account with
-- passkeys only has them too. Upstream's table referenced user_mfa(user_id)
-- ON DELETE CASCADE; nothing references this table, so it is rebuilt in
-- place, rows copied as they are. The code deletes the codes when an
-- account's last factor goes (src/mfa.rs).
CREATE TABLE ilia_mfa_recovery_codes_new (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash TEXT NOT NULL,
    PRIMARY KEY (user_id, code_hash)
);
INSERT INTO ilia_mfa_recovery_codes_new (user_id, code_hash)
    SELECT user_id, code_hash FROM mfa_recovery_codes;
DROP TABLE mfa_recovery_codes;
ALTER TABLE ilia_mfa_recovery_codes_new RENAME TO mfa_recovery_codes;

-- One registered credential. Everything here is public: the key, the
-- counter and the flags cannot be used to make an assertion.
CREATE TABLE ilia_webauthn_credentials (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- The RP ID it was registered for (the host of CALRS_BASE_URL then).
    rp_id TEXT NOT NULL,
    credential_id BLOB NOT NULL UNIQUE
        CHECK (length(credential_id) BETWEEN 1 AND 1023),
    -- SubjectPublicKeyInfo, DER.
    public_key BLOB NOT NULL,
    algorithm INTEGER NOT NULL CHECK (algorithm IN (-8, -7, -257)),
    sign_count INTEGER NOT NULL CHECK (sign_count BETWEEN 0 AND 4294967295),
    uv_initialized INTEGER NOT NULL CHECK (uv_initialized IN (0, 1)),
    -- BE: 1 = a synced passkey, 0 = a key bound to one device.
    backup_eligible INTEGER NOT NULL CHECK (backup_eligible IN (0, 1)),
    backup_state INTEGER NOT NULL CHECK (backup_state IN (0, 1)),
    -- JSON array of known AuthenticatorTransport values.
    transports TEXT NOT NULL DEFAULT '[]',
    aaguid BLOB CHECK (aaguid IS NULL OR length(aaguid) = 16),
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
    created_at INTEGER NOT NULL,
    last_used_at INTEGER
);
CREATE INDEX ilia_webauthn_credentials_user ON ilia_webauthn_credentials(user_id);

-- One WebAuthn challenge, single use. Found by the SHA-256 of the challenge
-- the browser echoes in clientDataJSON, so no other token travels in the
-- form. A login or first-factor ceremony belongs to the MFA challenge, a
-- settings one to the session and the password hash it was issued under;
-- revoking either deletes it.
CREATE TABLE ilia_webauthn_ceremonies (
    challenge_hash TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('login', 'enroll', 'confirm', 'register')),
    mfa_token_hash TEXT REFERENCES mfa_challenges(token_hash) ON DELETE CASCADE,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    CHECK ((kind IN ('login', 'enroll')) = (mfa_token_hash IS NOT NULL AND session_id IS NULL)),
    CHECK ((kind IN ('confirm', 'register')) = (session_id IS NOT NULL AND mfa_token_hash IS NULL))
);
CREATE INDEX ilia_webauthn_ceremonies_user ON ilia_webauthn_ceremonies(user_id);
CREATE INDEX ilia_webauthn_ceremonies_token ON ilia_webauthn_ceremonies(mfa_token_hash);
CREATE INDEX ilia_webauthn_ceremonies_session ON ilia_webauthn_ceremonies(session_id);
