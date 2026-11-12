# Passkeys and security keys (WebAuthn): implementation plan

Status (2026-11-12): **stages 1–3 implemented**, uncommitted, full fork check green; see §10
for what was built differently from this plan and what is left. This plan builds on the research note `webauthn-design.md`, which
covers the library choice, the verifier sketch and the spec references. Where the two differ,
the **owner decision** below wins. It replaces the note's "Phase 1: keys on top of TOTP".

## 0. Owner decision (binding)

- Login is **password + one second factor of the user's choice**. The second factor is either
  a **passkey** (WebAuthn: YubiKey or another security key, Touch ID / Face ID / Windows Hello,
  or a synced passkey) or **TOTP**.
- Passkey is the default. The big primary button is **"Sign in with passkey"**. The secondary
  buttons are **"Use authenticator code"** and **"Use recovery code"**.
- A user may register several keys (at most 10).
- Keys work **without TOTP**:
  - credentials reference `users`, not `user_mfa`;
  - the session gate and the admin MFA policy count "has TOTP **or** has at least 1 passkey"
    as having MFA;
  - recovery codes exist for key-only users too. They are generated when the **first** factor
    is enrolled and shown once. Regenerating them requires a factor.
- Every management action needs **password + a current factor** (a key assertion, a TOTP code
  or a recovery code) and revokes the user's other sessions. The actions are: add, rename or
  delete a key; enable or disable TOTP; regenerate recovery codes. The user's last factor cannot
  be removed while MFA is required.
- `calrs user reset-mfa` also removes passkeys.
- The RP ID and origin come **only** from the `CALRS_BASE_URL` environment variable and are
  compared as exact strings. The feature is off (UI hidden, ceremony routes return 404) when
  that variable is missing, not `https://`, or its host is an IP address. The one exception is
  `http://localhost[:port]`, allowed for development.
- `userVerification: "preferred"`, `attestation: "none"`, algorithms -8, -7 and -257 (RSA keys
  of at least 2048 bits).
- Counter and clone rules apply. BE/BS are stored, and the UI shows "Hardware key" or "Synced
  passkey".
- Ceremonies are single-use and tied to the MFA challenge (at login) or to the session (in
  settings).
- The JS is a same-origin static file with no inline handlers. The browser sends plain form
  POSTs (no `fetch`).
- **No new crates.** `Cargo.lock` changes only in the `calrs` package's own `dependencies`
  list (see §1).

The staged `config apply` work in this worktree stays as it is. All passkey work goes on top as
unstaged changes:
- `src/mfa.rs`, `src/web/mod.rs`, `src/web/tests/mod.rs`, `CHANGELOG.md` and
  `docs/src/cli-reference.md` are already staged. Passkey edits to them stay unstaged, layered
  over the staged version. Do not `git add` them.
- `mfa::store_policy` (staged) is kept unchanged.

---

## 1. Dependencies (stage 0, proven before any code)

`cargo tree --locked -e features -i <crate> --target x86_64-unknown-linux-musl` was run in
this worktree on 2026-11-12. It shows these crates and features already compiled in, all pulled
in by `openidconnect 4.0.1`:

| crate | version | features already enabled |
|---|---|---|
| p256 | 0.13.2 | alloc arithmetic default digest ecdsa ecdsa-core pem pkcs8 sha2 sha256 std |
| ecdsa | 0.16.9 | alloc arithmetic der digest hazmat pem pkcs8 rfc6979 signing spki std verifying |
| ed25519-dalek | 2.2.0 | alloc default fast pem pkcs8 std zeroize |
| rsa | 0.9.10 | default pem std u64_digit |
| sha2 | 0.10.9 | default oid std (already a direct dependency) |
| signature | 2.2.0 | alloc digest rand_core std |
| spki / pkcs8 | 0.7.3 / 0.10.2 | alloc default pem std |

**`Cargo.toml` additions.** Each uses default features, which are already on, so it enables
nothing new:

```toml
# ilia fork: passkeys (src/webauthn). Already in the build through openidconnect,
# same versions and features; Cargo.lock gains only the root's dependency names.
p256 = "0.13"
ed25519-dalek = { version = "2", features = ["pkcs8"] }
rsa = "0.9"
```

URL parsing uses `reqwest::Url`, which reqwest re-exports, so we don't add `url`.
Base64url decoding uses `base64` 0.22 (already direct) with `URL_SAFE_NO_PAD`.

**Proof, recorded in the commit message:**
1. `git diff Cargo.lock` shows only three added names in `[[package]] name = "calrs"`
   `dependencies`.
2. Running `cargo tree -e features --target x86_64-unknown-linux-musl` before and after gives
   identical output, apart from the root's new edges.
3. `cargo tree -d` is unchanged.

Tests use the same crates:
- `p256::ecdsa::SigningKey` and `ed25519_dalek::SigningKey::from_bytes`;
- `rsa::pkcs1v15::SigningKey<Sha256>` over an RSA-2048 PKCS#8 DER fixture stored in hex under
  `src/webauthn/testdata/`. Generating RSA in tests would be slow, and the fixture is public
  test material.

`signature::{Signer, Verifier}` are reached through `p256::ecdsa::signature`, so we don't
declare `signature` directly.

---

## 2. Schema: `migrations/ilia_010_webauthn.sql`

Register it in `src/db.rs` after `ilia_009_inbox`. Update the migration-count comment and test
(`src/db.rs` around line 1034). If another fork branch claims `ilia_010` first, renumber this
one.

```sql
-- ilia fork: passkeys and security keys as a second factor (docs/passkeys-plan.md).

-- Recovery codes belong to the user, not to the TOTP row: a passkey-only
-- account has them too. Upstream's table referenced user_mfa(user_id) ON DELETE
-- CASCADE; nothing references this table, so it is rebuilt in place.
CREATE TABLE ilia_mfa_recovery_codes_new (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash TEXT NOT NULL,
    PRIMARY KEY (user_id, code_hash)
);
INSERT INTO ilia_mfa_recovery_codes_new (user_id, code_hash)
    SELECT user_id, code_hash FROM mfa_recovery_codes;
DROP TABLE mfa_recovery_codes;
ALTER TABLE ilia_mfa_recovery_codes_new RENAME TO mfa_recovery_codes;

CREATE TABLE ilia_webauthn_credentials (
    id TEXT PRIMARY KEY,                                   -- uuid v4, used in URLs/forms
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rp_id TEXT NOT NULL,                                   -- RP ID at registration
    credential_id BLOB NOT NULL UNIQUE
        CHECK (length(credential_id) BETWEEN 1 AND 1023),
    public_key BLOB NOT NULL,                              -- SPKI DER (public)
    algorithm INTEGER NOT NULL CHECK (algorithm IN (-8, -7, -257)),
    sign_count INTEGER NOT NULL CHECK (sign_count BETWEEN 0 AND 4294967295),
    uv_initialized INTEGER NOT NULL CHECK (uv_initialized IN (0, 1)),
    backup_eligible INTEGER NOT NULL CHECK (backup_eligible IN (0, 1)),
    backup_state INTEGER NOT NULL CHECK (backup_state IN (0, 1)),
    transports TEXT NOT NULL DEFAULT '[]',                 -- JSON array, known values only
    aaguid BLOB CHECK (aaguid IS NULL OR length(aaguid) = 16),
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
    created_at INTEGER NOT NULL,
    last_used_at INTEGER
);
CREATE INDEX ilia_webauthn_credentials_user ON ilia_webauthn_credentials(user_id);

-- One WebAuthn challenge. The row is found by the SHA-256 of the challenge the
-- browser echoes in clientDataJSON, so no extra token travels in the form.
CREATE TABLE ilia_webauthn_ceremonies (
    challenge_hash TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('login', 'enroll', 'confirm', 'register')),
    mfa_token_hash TEXT REFERENCES mfa_challenges(token_hash) ON DELETE CASCADE,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL,                           -- users.password_hash at creation
    expires_at INTEGER NOT NULL,
    CHECK ((kind IN ('login', 'enroll')) = (mfa_token_hash IS NOT NULL AND session_id IS NULL)),
    CHECK ((kind IN ('confirm', 'register')) = (session_id IS NOT NULL AND mfa_token_hash IS NULL))
);
CREATE INDEX ilia_webauthn_ceremonies_user ON ilia_webauthn_ceremonies(user_id);
```

### Why this shape

- **The rebuild runs safely inside the migration transaction with `foreign_keys = ON`.**
  `mfa_recovery_codes` is only a child table, so `DROP TABLE` has nothing to cascade. The data
  is copied 1:1. This is a fork divergence in an upstream table, so add it to `docs/fork.md`.
  Upstream's own `ALTER TABLE … ADD COLUMN` would still apply to the rebuilt table.
- **Recovery codes no longer cascade.** Disabling TOTP no longer deletes the codes, so the
  code must delete them explicitly whenever a change leaves the user with **zero factors**
  (§4.6), and in `reset`.
- **Cascades cover the rest.** Deleting a user removes keys, codes and ceremonies.
  `revoke_access`, which deletes sessions and MFA challenges, removes every ceremony bound to
  them. Suspension, password reset, OIDC linking and every MFA change therefore kill pending
  ceremonies with no new code.
- **Only public data is stored.** The public key, the counter and the flags are public. A
  stolen database cannot be used to forge an assertion.
- **`session_id` duplicates `sessions.id`**, which is the plaintext session token. The FK
  needs it, and it adds no exposure beyond `sessions`. It is added to the agent's sentinel
  list (§8).

### Upstream tables
`user_mfa`, `mfa_challenges` and `mfa_attempts` are unchanged. An enroll challenge still
carries a pre-generated TOTP secret. The passkey path ignores it and deletes the challenge on
completion.

---

## 3. Module layout and the gate

```
src/webauthn/mod.rs      Rp (RP ID + origin from CALRS_BASE_URL), limits, user handle,
                         options JSON builders, Proof form types
src/webauthn/verify.rs   pure verifier: no I/O, no panics (stage 1)
src/webauthn/store.rs    credential and ceremony SQL on a &mut SqliteConnection (stage 2)
src/webauthn/flows.rs    axum handlers for the passkey-only routes, router() (stage 2)
src/webauthn/softkey.rs  #[cfg(test)] software authenticator (ES256, Ed25519, RS256)
src/webauthn/tests.rs    verifier unit tests and spec vectors
src/webauthn/testdata/   rsa2048.pkcs8.hex, webauthn-3 §16 vectors
assets/webauthn.js       browser side (stage 3)
```

`src/main.rs` gets `mod webauthn;`, with a fork comment.

### 3.1 `Rp`

```rust
pub(crate) struct Rp { pub id: String, pub origin: String, pub name: String }
impl Rp {
    /// From CALRS_BASE_URL only (never the DB fallback, never Host/X-Forwarded-*).
    pub(crate) fn from_env() -> Option<Rp>;
    fn from_base_url(raw: &str) -> Option<Rp>;   // unit-tested
}
```

**Rules for `from_base_url`:**
- Parse with `reqwest::Url`.
- Reject a username or password in the URL.
- The host must be `Host::Domain`, so an IP address (IPv4 or IPv6) is refused.
- Accept scheme `https`, or `http` only when the host is `localhost`.
- `id` = the host, ASCII and lowercase (punycode for IDN, which is what browsers use).
- `origin` = `url.origin().ascii_serialization()`, for example `https://book.ilia.ae`. A
  default port is dropped; any other port is kept.
- `name` = `id`, which is what authenticators show.
- A path or a trailing `/` in the base URL is ignored.

**Where it is read.** The value is read once, in `build_state`, into a new
`AppState::passkeys: Option<webauthn::Rp>` field (a single line with a fork comment, next to
`ton: TonSlot::from_env()`).
- Tests build the state with `web::build_state`, set the field through `Arc::get_mut` and
  call `web::router(state)`. No test touches the process environment, which other tests set in
  parallel.
- `settings::base_url_env()` becomes `pub(crate)`. It is the only change to `settings.rs`.

**Why the environment variable and not the DB value.** The DB base URL can be edited from the
admin page. The RP ID binds every key, so it must not move with a web form. This is documented
as a fork decision.

### 3.2 "Has MFA" becomes "has a factor"

A single SQL fragment, `mfa::HAS_FACTOR`, takes a user-id parameter that is bound twice:

```sql
(EXISTS (SELECT 1 FROM user_mfa WHERE user_id = ?1)
 OR EXISTS (SELECT 1 FROM ilia_webauthn_credentials WHERE user_id = ?1))
```

Every place that meant "has a TOTP row" switches to it:

| Place | Change |
|---|---|
| `auth::validate_session` (`src/auth.rs:94`) | `OR (s.mfa_verified = 1 AND (EXISTS (SELECT 1 FROM user_mfa WHERE user_id = u.id) OR EXISTS (SELECT 1 FROM ilia_webauthn_credentials WHERE user_id = u.id)))` |
| OIDC auto-link refusal (`src/auth.rs:1632`) | `local_mfa` = has a factor, so a key-only account is not silently handed to the IdP |
| `mfa::begin_login` `active` | has a factor (a login challenge, not enroll, for key-only users) |
| `mfa::enrolled` | renamed in meaning: has a factor. New `mfa::has_totp` for the TOTP-only checks |
| `mfa::policy_save` | the local admin proves a factor through `consume_proof` (§4.5) |
| `mfa::consume_code` recovery branch | no longer requires a `user_mfa` row (codes exist only while the user has a factor) |

Credentials with a different `rp_id` still count as a factor. That fails closed: the user then
needs TOTP, a recovery code or `reset-mfa`. They are listed in the UI as "registered for X, not
usable here".

**Feature off with keys enrolled.** If `passkeys` is `None` (the base URL was removed) and a
user has only keys, login falls back to recovery codes. The settings page still lets the user
delete keys with a recovery code. `docs/operations.md` explains this.

---

## 4. Flows and state machines

### 4.0 Ceremony rules (all four kinds)

- **Challenge.** 32 bytes from `OsRng`. Only `challenge_hash = hex(sha256(challenge))` is
  stored. The raw challenge goes only into the options JSON.
- **Expiry.** `expires_at = min(now + 600, parent expiry)`, where the parent is the MFA
  challenge for `login` and `enroll` (600 s too, so the ceremony lasts as long as the
  page). The page `timeout` is 300000 ms, counted from the button press.
- **One live ceremony per parent and kind.** Creating one first deletes any earlier ceremony
  of the same kind for the same `mfa_token_hash` or `session_id`. Expired rows are purged in
  that same statement batch, the way `begin_login` purges old challenges.
- **Single use.** The POST handler takes the write lock first, then runs
  `DELETE … WHERE challenge_hash = ? AND kind = ? AND <binding> AND user_id = ? AND expires_at > ? RETURNING password_hash`.
  The row is deleted **even when verification fails**: the transaction commits the burn and
  creates no session. A failed or replayed assertion can therefore never be retried against the
  same challenge.
- **Binding.**
  - `login` and `enroll`: `mfa_token_hash = hash(cookie __Host-calrs_mfa)`, and
    `mfa::challenge()` must still be valid (user enabled, local account, same password hash).
  - `confirm` and `register`: `session_id` = the current session cookie, and
    `password_hash = users.password_hash`. A ceremony therefore dies with a password change,
    even when the session survives it.
- **Attempt budget.** A passkey assertion at `/auth/mfa/passkey` does **not** call
  `reserve_attempt`, because a signature cannot be guessed. Otherwise someone holding the
  password could lock the owner out by failing codes on purpose. Failures are logged as
  `tracing::warn!(user_id, ip, reason)`. Every management or policy POST still calls
  `reserve_attempt`, because it checks the password, exactly as today.
- A successful login, enrolment or management change calls `clear_attempts`.

### 4.1 WebAuthn parameters (built in `webauthn::mod.rs`, embedded as JSON)

**`create()`:**
- `rp {id, name}`;
- `user {id: b64url(handle), name: email, displayName: users.name}`, where
  `handle = sha256("calrs-webauthn-user\0" || users.id)[..16]`: deterministic, no personal
  data, no storage;
- `challenge`;
- `pubKeyCredParams` in the order `[-7, -8, -257]`: an authenticator takes the first one it
  supports, and Safari's `getPublicKey()` returns only ES256 keys, so ES256 leads;
- `timeout: 300000`;
- `attestation: "none"`;
- `authenticatorSelection {residentKey: "discouraged", userVerification: "preferred"}`.
  `discouraged` saves YubiKey discoverable slots. Apple and Google make synced discoverable
  passkeys anyway. Passwordless login would need re-registration later, and that is
  acceptable;
- `excludeCredentials`: the user's credentials with the same `rp_id` and their transports;
- `hints` are set by the button in JS: `["security-key"]` or `["client-device", "hybrid"]`.

**`get()`:**
- `challenge`, `rpId`, `timeout: 300000`, `userVerification: "preferred"`;
- `allowCredentials`: the user's credentials with the current `rp_id`, with transports. If
  there are none, no passkey method is offered.

The options are passed to the template as a `serde_json::Value` and rendered with
`{{ options|tojson }}` inside `<script type="application/json" id="webauthn-options">`.
minijinja's `tojson` escapes `<`, `>`, `&` and `'`, so the JSON cannot close the tag.

### 4.2 Verifier (`src/webauthn/verify.rs`, stage 1)

```rust
pub(crate) struct Expected<'a> { rp_id: &'a str, origin: &'a str, challenge: &'a [u8] }
pub(crate) struct Registration { credential_id, public_key, algorithm, sign_count,
                                 uv, backup_eligible, backup_state, aaguid }
pub(crate) struct StoredKey<'a> { public_key, algorithm, sign_count, backup_eligible }
pub(crate) struct Assertion { sign_count: u32, uv: bool, backup_state: bool }
pub(crate) fn client_challenge(client_data: &[u8]) -> Result<Vec<u8>>;  // to find the row
pub(crate) fn verify_registration(e, client_data, auth_data, spki, alg, raw_id) -> Result<Registration>;
pub(crate) fn verify_assertion(e, key: &StoredKey, client_data, auth_data, sig) -> Result<Assertion>;
pub(crate) fn counter_ok(stored: u32, received: u32) -> bool;
```

**Steps, each with its own error variant and its own negative test:**
1. **Size caps**, checked before decoding:
   - clientDataJSON ≤ 4096;
   - authenticatorData 37..=4096;
   - signature ≤ 1024;
   - SPKI ≤ 1024;
   - credential ID 1..=1023;
   - userHandle ≤ 64;
   - transports: at most 8 values from the set `usb nfc ble smart-card hybrid internal`;
     unknown values are dropped.
2. **clientDataJSON:**
   - UTF-8 with no BOM, parsed with serde_json;
   - `type` is `webauthn.create` or `webauthn.get`, matching the ceremony kind;
   - `challenge` decodes from base64url with no padding and equals the stored challenge
     (constant-time comparison);
   - `origin == rp.origin` as an exact string;
   - `crossOrigin` is absent or `false`;
   - `topOrigin` is absent.
3. **authenticatorData:**
   - `rpIdHash == sha256(rp_id)`;
   - UP = 1;
   - UV is recorded and not required;
   - BS = 1 with BE = 0 is rejected;
   - the counter is the 4-byte big-endian value.
4. **Registration only:**
   - AT = 1;
   - parse `aaguid(16) | credIdLen(2) | credId` and require `credId == rawId`. The COSE key and
     the extensions after it are not parsed: this is the §5.2.1.1 simplification from the
     design note;
   - `alg ∈ {-8, -7, -257}`;
   - the SPKI parses as the type that `alg` implies:
     - `ed25519_dalek::VerifyingKey::from_public_key_der`;
     - `p256::ecdsa::VerifyingKey::from_public_key_der`;
     - `rsa::RsaPublicKey::from_public_key_der` with a modulus of 2048..=4096 bits.
5. **Assertion only:**
   - the signature is over `authData || sha256(clientDataJSON)`;
   - Ed25519 uses `verify_strict`;
   - ES256 uses a DER signature. High-S is accepted, and a test checks this;
   - RS256 uses PKCS#1 v1.5 with SHA-256;
   - BE must equal the stored `backup_eligible`.
6. **Counter (`counter_ok`):**
   - `stored == 0 && received == 0` is OK;
   - `received > stored` is OK;
   - anything else is refused and logged as `warn!("possible cloned authenticator")`.

   The key is not disabled automatically. The user can remove it.

### 4.3 Login (`/auth/mfa`, no session yet)

```
POST /auth/login (password OK) → mfa::begin_login
   ├─ no factor, policy optional, not setup  → plain session (unchanged)
   ├─ has a factor (TOTP or ≥1 key)          → challenge purpose=login → 302 /auth/mfa
   └─ no factor, (required or setup)         → challenge purpose=enroll (+TOTP secret) → 302 /auth/mfa

GET /auth/mfa[?method=passkey|totp|recovery]          (purpose=login)
   default method: passkey if Rp is set and the user has ≥1 key for this rp_id,
                   else totp if has_totp, else recovery
   method=passkey → create ceremony(kind=login, mfa_token_hash) → page with
                    [Sign in with passkey] (primary) + links "Use authenticator code"
                    (only if has_totp) and "Use recovery code"
   method=totp / recovery → the existing code form (labels differ) + link back to
                    "Use passkey" (if usable) and the other code method

POST /auth/mfa/passkey   (new; CSRF; fields: credential_id, client_data,
                          authenticator_data, signature, user_handle)
   tx: write lock on mfa_challenges row → challenge() valid, purpose=login
       → burn ceremony (kind=login, this token) → credential by (credential_id,
       user_id, rp_id) → verify_assertion → user_handle (if present) == handle
       → CAS: UPDATE … SET sign_count=?, backup_state=?, uv_initialized=max(…),
         last_used_at=? WHERE id=? AND sign_count=<old>  (1 row or fail)
       → DELETE mfa_challenges row → clear_attempts → verified_session → commit
   success → 302 /dashboard with session cookie, challenge cookie cleared
   failure → commit the burn only; warn log; re-render GET view with
             invalid=true and a fresh ceremony (same challenge cookie)

POST /auth/mfa   (unchanged route and form: TOTP or recovery code)
```

The `?method=` value only chooses labels and which block is shown. `POST /auth/mfa` accepts a
TOTP code or a recovery code whichever method is shown, as it does today.

### 4.4 First-factor enrolment (`purpose=enroll`, no factor yet)

Reached when MFA is required at login, at registration, or through Settings → "Set up
two-factor authentication" (`action=enable`, password only, unchanged).

```
GET /auth/mfa[?method=passkey|totp]            (purpose=enroll)
   default: passkey if Rp is set, else totp
   passkey → ceremony(kind=enroll, mfa_token_hash) → name field + two buttons
             [Add a security key] (hints security-key) [Add a passkey on this device or phone]
             + link "Use an authenticator app instead" (?method=totp)
   totp    → existing QR + code form (upstream) + link "Use a passkey instead"

POST /auth/mfa/passkey/enroll   (new; CSRF; credential_id, client_data,
                                 authenticator_data, public_key, algorithm,
                                 transports, name)
   tx: write lock → challenge() valid, purpose=enroll → burn ceremony(kind=enroll)
       → verify_registration → user has no factor yet (else fail: someone finished
         the other path first) → key count < 10 → INSERT credential (UNIQUE
         credential_id; conflict → "already registered")
       → new_recovery_codes → revoke_access → DELETE challenge → clear_attempts
       → verified_session → commit
   success → recovery-codes page (mode=recovery, new session cookie), as TOTP enrolment
```

TOTP enrolment through `/auth/mfa` (`verify_challenge`, purpose `enroll`) changes in one way.
It records `had_factor = HAS_FACTOR` **before** inserting `user_mfa`, and calls
`new_recovery_codes` only when `!had_factor`. The other path (§4.6, `totp_add`) relies on this.
The rest is unchanged: insert, `revoke_access`, verified session.

`challenge_view` for `purpose=enroll` shows the passkey or TOTP choice only when the user has
no factor. A key-only user who came through `totp_add` sees only the TOTP QR.

### 4.5 Step-up: "Confirm it's you" (management and policy)

Every management action is a two-step flow: an **intent**, then a **confirmation**. The
confirmation is one POST that carries the password and the proof.

```
GET /dashboard/settings/mfa/confirm?action=<a>[&credential=<uuid>][&policy=required|optional]
   AuthUser, local, not impersonating (else 403; action=policy uses the real session user, as
   /dashboard/admin and policy_save do); unknown action / foreign credential → 404
   → if Rp is set and the user has usable keys: ceremony(kind=confirm, session_id)
   → page: what is about to happen (localized, with the key name),
           password field,
           [Confirm with passkey] (primary; JS fills hidden assertion fields and submits),
           "Use authenticator code" / "Use recovery code" (?method=…) → code field +
           [Confirm with code]
           (for rename: the new-name field; for policy: hidden policy value)
   the form posts to /dashboard/settings/mfa (or /dashboard/admin/mfa for policy)
```

`mfa::Proof` is parsed from the same form:
- `Proof::Passkey(AssertionFields)` when `credential_id` is non-empty;
- otherwise `Proof::Code(code)`.

`mfa::consume_proof(&mut tx, state, user, proof, Binding::Session(session_id))`:
- `Code`: the existing `consume_code`, covering TOTP with the `last_step` compare-and-swap and
  recovery codes.
- `Passkey`: burn the `confirm` ceremony for this session, then verify the assertion as in
  §4.3, including the counter compare-and-swap.

Both run inside the action's own transaction, after `same_local_user`. The password is checked
before the transaction, after `reserve_attempt`, as today.

### 4.6 Management actions (`POST /dashboard/settings/mfa`, extended `ManagementForm`)

The existing fields stay: `_csrf`, `password`, `code`, `action`. New fields, all
`#[serde(default)]`:
- `credential` (a key uuid);
- `name`;
- the assertion fields `credential_id`, `client_data`, `authenticator_data`, `signature`,
  `user_handle`.

| `action` | Needs a factor | Effect, in one transaction | Session afterwards |
|---|---|---|---|
| `enable` | no, password only (user has no factor) | `begin_login(setup = true)` → enroll challenge → `/auth/mfa` (§4.4) | unchanged until enrolment completes |
| `regenerate` | yes | `new_recovery_codes` | verified; codes page |
| `disable` (= remove the authenticator app) | yes | delete `user_mfa`; **last-factor rule** | verified if keys remain, else plain |
| `totp_add` | yes (user has keys, no TOTP) | new `mfa::begin_totp_enroll`: an enroll challenge with a secret for a user who already has a factor → `/auth/mfa` QR; completion keeps the existing codes | at completion: revoke + verified |
| `passkey_add` | yes | create a `register` ceremony bound to the **new** session → render the register page (name field, two buttons) | revoke + verified (new cookie set on the register page) |
| `passkey_rename` | yes | `UPDATE … SET name = ? WHERE id = ? AND user_id = ?` (1 row) | revoke + verified |
| `passkey_delete` | yes | `DELETE … WHERE id = ? AND user_id = ?` (1 row); **last-factor rule** | verified if a factor remains, else plain |

**Last-factor rule.** It runs after the delete, inside the transaction, while holding the write
lock:
- if `mfa_required` and the user now has no factor, roll back and show "cannot remove your last
  method";
- if the policy is optional and the user now has no factor, also
  `DELETE FROM mfa_recovery_codes WHERE user_id = ?`.

**The rest of every action** follows the existing `manage()`:
- `revoke_access` (other sessions and challenges, and with them the ceremonies);
- `clear_attempts`;
- a new session, verified while the user still has a factor;
- `tracing::info!(user_id, action, credential = <uuid>)`.

**`POST /dashboard/settings/mfa/passkeys`** is new. It finishes `passkey_add`.
- **Fields:** CSRF, the registration fields and `name`. No password: the `register` ceremony
  is the proof. It was issued after a successful step-up and is bound to this session and to
  this password hash.
- **In the transaction:**
  1. take the write lock;
  2. `same_local_user`;
  3. burn the `register` ceremony for this session;
  4. `verify_registration`;
  5. check the user has fewer than 10 keys;
  6. insert the key, with the UNIQUE credential ID;
  7. `revoke_access`, which ends every *other* session. The current session was already
     re-issued at step-up, so it is kept here (the same-session exclusion is the only
     difference from the other actions);
  8. log.
- **Result:** redirect to `/dashboard/settings/mfa`.
- **Failure:** the burn is committed, and the page shows an error with a link back.

**Names:**
- trimmed;
- 1..=64 characters;
- no control characters;
- escaped by the template;
- prefilled with "Security key N" or "Passkey N" from i18n.

**Refusals:** OIDC users and impersonation get 403 on every route here, as today.

### 4.7 Admin MFA policy (`POST /dashboard/admin/mfa`)

- **OIDC admin:** unchanged (direct POST, no local proof).
- **Local admin:**
  - `templates/admin.html` replaces the password and code fields with a GET form to
    `/dashboard/settings/mfa/confirm?action=policy&policy=…` labelled "Continue";
  - the confirm page posts `policy`, `password` and the proof to `/dashboard/admin/mfa`;
  - `policy_save` swaps `consume_code(…)` for `consume_proof(…, Binding::Session)`, and the rest
    is unchanged (`store_policy`, staged).
- **Fail closed:** without a factor, a local admin still cannot change the policy.
- **Rename:** the admin page's "MFA" label becomes "two-factor authentication (passkey or
  authenticator app)", using the existing `mfa-policy-help` with a fork override.

### 4.8 Operator reset and other paths

`mfa::reset` (the `calrs user reset-mfa` CLI) adds two statements:
- `DELETE FROM ilia_webauthn_credentials WHERE user_id = ?`;
- `DELETE FROM mfa_recovery_codes WHERE user_id = ?`.

Codes no longer cascade from `user_mfa`, so the second delete is required. The CLI help in
`src/commands/user.rs` becomes "Reset local MFA (authenticator app, passkeys, recovery codes)
and revoke sessions". Update `docs/src/cli-reference.md` to match.

Unchanged and verified by tests:
- suspension (`set_user_enabled`);
- user deletion (cascade);
- OIDC linking (refused while the user has a factor);
- password reset by email or by the admin;
- `calrs config apply mfa_policy` (`store_policy`).

### 4.9 Static JS: `GET /static/webauthn.js`

- Served from `include_str!("../../assets/webauthn.js")` in `webauthn::flows`.
- Headers: `Content-Type: application/javascript; charset=utf-8` and
  `Cache-Control: no-cache`. `nosniff` comes from the global layer.
- 404 when the feature is off.
- No CORS header.
- About 150 lines, wrapped in an IIFE with `'use strict'`, using no globals. On
  `DOMContentLoaded` it reads `#webauthn-options` and binds `click` on
  `button[data-webauthn]`, which is `"get"` or `"create"`. Each button has `data-hints` and
  `data-form`.
- **Feature detection:**
  - `window.PublicKeyCredential` must exist;
  - for create, `getPublicKey` must exist on `AuthenticatorAttestationResponse.prototype`;
  - otherwise hide the buttons and un-hide `[data-webauthn-unsupported]`.
- **Options:** `PublicKeyCredential.parseCreationOptionsFromJSON` /
  `parseRequestOptionsFromJSON` when present. Otherwise decode base64url by hand: `challenge`,
  `user.id`, and `allow/excludeCredentials[].id`.
- **get:**
  - fill `credential_id` (`rawId`), `client_data`, `authenticator_data`, `signature` and
    `user_handle` as base64url, then call `form.requestSubmit()`, falling back to
    `form.submit()`;
  - the button is disabled while waiting.
- **create:**
  - require the name field to be valid (`reportValidity()`);
  - fill `credential_id`, `client_data`, `authenticator_data` (`getAuthenticatorData()`),
    `public_key` (`getPublicKey()`), `algorithm` (`getPublicKeyAlgorithm()`) and `transports`
    (a JSON array from `getTransports()`), then submit.
- **Errors:** `NotAllowedError` and `AbortError` un-hide `[data-webauthn-cancelled]`;
  `InvalidStateError` on create un-hides `[data-webauthn-exists]`. Every user-visible string
  lives in the template, so all i18n stays in Fluent.
- **Page setup:** the template loads `<script src="/static/webauthn.js" defer>`. There are no
  inline handlers and no `fetch`, so `connect-src` is untouched. The CSP is unchanged.
  `Permissions-Policy` is unchanged, because `publickey-credentials-*` defaults to `self`; a
  test checks that the header does not list them.
- **Without JS:** a `<noscript>` notice is shown, and the secondary code methods work as plain
  links and forms.

---

## 5. Every route and form

| Method | Path | Auth | Fields | Owner |
|---|---|---|---|---|
| GET | `/auth/mfa?method=` | MFA cookie | none | upstream handler + `Query<MethodQuery>` |
| POST | `/auth/mfa` | MFA cookie | `_csrf`, `code` | upstream (unchanged) |
| POST | `/auth/mfa/passkey` | MFA cookie (login) | `_csrf`, `credential_id`, `client_data`, `authenticator_data`, `signature`, `user_handle` | `webauthn::flows`, 404 if off |
| POST | `/auth/mfa/passkey/enroll` | MFA cookie (enroll) | `_csrf`, `credential_id`, `client_data`, `authenticator_data`, `public_key`, `algorithm`, `transports`, `name` | `webauthn::flows`, 404 if off |
| GET | `/dashboard/settings/mfa` | session, local, not impersonating | none | upstream handler, new view (§6) |
| GET | `/dashboard/settings/mfa/confirm` | same | query `action`, `credential`, `policy`, `method` | `webauthn::flows` (works when off: code methods only) |
| POST | `/dashboard/settings/mfa` | same | `_csrf`, `password`, `code`, `action`, `credential`, `name`, assertion fields | upstream handler, extended |
| POST | `/dashboard/settings/mfa/passkeys` | same + `register` ceremony | `_csrf`, registration fields, `name` | `webauthn::flows`, 404 if off |
| POST | `/dashboard/admin/mfa` | admin | `_csrf`, `policy`, `password`, `code`, assertion fields | upstream handler, `consume_proof` |
| GET | `/static/webauthn.js` | none | none | `webauthn::flows`, 404 if off |

`webauthn::flows::router()` is merged in `web::router` next to `crate::mfa::router()`, with a
single fork line. `"static"` is not a user slug, so check the `RESERVED` list in
`src/web/mod.rs` (`/static/intl-tel-input` already relies on that) and add `"static"` only if
it is missing.

Every response that renders MFA pages keeps `Cache-Control: no-store`,
`Referrer-Policy: no-referrer` and a fresh CSRF cookie (the existing `mfa::render`).

---

## 6. UI

- **`templates/auth/mfa.html` (upstream).** Add small hooks only:
  - `{% include "auth/ilia_mfa_factor.html" %}` in the `login` and `enroll` branches, shown
    when `passkey_view` is true;
  - the `settings` branch becomes `{% include "auth/ilia_mfa_settings.html" %}`;
  - new modes `confirm` and `register` include `auth/ilia_mfa_confirm.html` and
    `auth/ilia_passkey_register.html`;
  - the `recovery` mode uses `ilia-mfa-recovery-help`, which mentions passkeys.
- **`templates/auth/ilia_mfa_factor.html` (new).** The login chooser:
  - help text, then the big `.btn` "Sign in with passkey";
  - below it, `.btn btn-secondary` links "Use authenticator code" (only when TOTP is set up)
    and "Use recovery code";
  - the hidden assertion form, the `noscript`, unsupported, cancelled and invalid notices,
    the options JSON, and the script tag;
  - in the enroll variant: the name field, the two create buttons, and "Use an authenticator
    app instead".
- **`templates/auth/ilia_mfa_settings.html` (new).** The overview:
  - **Status line.** "Two-factor authentication is on / off", plus "required by your
    administrator" when it applies.
  - **Passkeys and security keys.** A list. Each row shows:
    - the name;
    - a badge, "Hardware key" (BE = 0) or "Synced passkey" (BE = 1);
    - the added date and the last-used date (or "not used yet");
    - when the `rp_id` differs, a warning "registered for X";
    - "Rename" and "Remove" links to the confirm page.

    Below the list: "Add a security key" and "Add a passkey on this device or phone" (both via
    confirm, or via `enable` when the user has no factor yet). The limit is noted at 10 keys,
    and a "keep a second key" hint is shown when the user has only one key.
  - **Authenticator app.** On or off, with "Set up authenticator app" (`totp_add`, or `enable`
    when there is no factor) or "Remove authenticator app".
  - **Recovery codes.** "N recovery codes left" (a plural) and "Generate new recovery codes".
  - When the feature is off: the notice `ilia-passkey-unavailable`. Existing keys are listed
    and removable.
  - Footer: `ilia-mfa-changes-help` and "Back to settings".
- **`templates/auth/ilia_mfa_confirm.html` (new).**
  - The action sentence, then the password field.
  - Passkey primary, or the code field with a submit button.
  - For rename, the new name. For policy, a hidden `policy`.
  - Links to the other methods.
- **`templates/auth/ilia_passkey_register.html` (new).** The name field, the two create
  buttons, the help text and the notices.
- **`templates/admin.html` (upstream).** The local-admin policy form becomes a GET form to the
  confirm page. The OIDC branch is unchanged.
- **`templates/settings.html`.** The link stays as it is.

Dates use the existing `format_date`-style helpers in the user's timezone, whichever the
settings pages already use. No new CSS beyond a small `.passkey-list` block in the fork partial,
which uses the existing variables.

### 6.1 i18n

The new keys go in a section `## ilia fork: passkeys and security keys (docs/passkeys-plan.md)`
in **all 10 full locales**: en, de, es, et, fr, it, nl, pl, pt, ru.
- **`de-formal`** (an overlay) gets a `Sie` override for every key whose `de` text addresses
  the reader.
- **Plural.** `ilia-mfa-recovery-left` is a plural. `ru` and `pl` need one, few and many
  (`plural_messages_carry_the_locale_categories`).
- **Quality.** EN and RU are written with care, as below. The other locales get good
  translations that follow each locale's existing `mfa-*` wording.

| key | EN | RU |
|---|---|---|
| ilia-passkey-sign-in | Sign in with passkey | Войти с ключом доступа |
| ilia-mfa-login-help-passkey | Confirm this sign-in with your passkey or security key. The sign-in expires after 10 minutes. | Подтвердите вход ключом доступа или аппаратным ключом. Вход нужно завершить за 10 минут. |
| ilia-mfa-use-totp | Use authenticator code | Ввести код из приложения |
| ilia-mfa-use-recovery | Use recovery code | Ввести резервный код |
| ilia-mfa-use-passkey | Use passkey | Использовать ключ доступа |
| ilia-mfa-totp-login-help | Enter the six-digit code from your authenticator app. | Введите шестизначный код из приложения-аутентификатора. |
| ilia-mfa-recovery-code | Recovery code | Резервный код |
| ilia-mfa-recovery-login-help | Enter one of the recovery codes you saved. Each code works only once. | Введите один из сохранённых резервных кодов. Каждый код действует только один раз. |
| ilia-passkey-unsupported | This browser cannot use passkeys or security keys. Choose another method below, or open this page in an up-to-date browser. | Этот браузер не поддерживает ключи доступа и аппаратные ключи. Выберите другой способ ниже или откройте страницу в свежей версии браузера. |
| ilia-passkey-noscript | Passkeys need JavaScript. Turn it on for this site, or choose another method below. | Для ключей доступа нужен JavaScript. Включите его для этого сайта или выберите другой способ ниже. |
| ilia-passkey-cancelled | The request was cancelled or timed out. Try again, or choose another method. | Запрос отменён или время ожидания истекло. Попробуйте ещё раз или выберите другой способ. |
| ilia-passkey-invalid | This key could not be verified. Try again, or choose another method. | Не удалось проверить ключ. Попробуйте ещё раз или выберите другой способ. |
| ilia-mfa-enroll-help | Choose how you will confirm each sign-in. A passkey or security key is the safest choice: it works only on this site, so a fake copy of the page cannot use it. | Выберите, чем вы будете подтверждать каждый вход. Надёжнее всего ключ доступа или аппаратный ключ: он работает только на этом сайте, и поддельная копия страницы не сможет им воспользоваться. |
| ilia-passkey-add-security-key | Add a security key | Добавить аппаратный ключ |
| ilia-passkey-add-device | Add a passkey on this device or phone | Добавить ключ доступа на этом устройстве или телефоне |
| ilia-mfa-use-app-instead | Use an authenticator app instead | Использовать приложение-аутентификатор |
| ilia-passkey-name | Key name | Название ключа |
| ilia-passkey-name-help | For example “YubiKey on keyring” or “MacBook”. Up to 64 characters. | Например, «YubiKey на связке» или «MacBook». До 64 символов. |
| ilia-passkey-default-security-key | Security key { $n } | Аппаратный ключ { $n } |
| ilia-passkey-default-device | Passkey { $n } | Ключ доступа { $n } |
| ilia-passkey-register-help | Press the button and follow your browser: insert and touch the security key, or confirm with Touch ID, Face ID, Windows Hello or your phone. | Нажмите кнопку и следуйте подсказкам браузера: вставьте аппаратный ключ и коснитесь его или подтвердите вход через Touch ID, Face ID, Windows Hello или телефон. |
| ilia-passkey-exists | This key is already registered. | Этот ключ уже зарегистрирован. |
| ilia-mfa-recovery-help | Save these recovery codes somewhere safe; they are shown only once. Each code replaces a passkey or an authenticator code for one sign-in. New codes replace all previous ones. | Сохраните эти резервные коды в надёжном месте: они показываются только один раз. Каждый код заменяет ключ или код из приложения при одном входе. Новые коды заменяют все прежние. |
| ilia-mfa-status-on | Two-factor authentication is on. | Двухфакторная аутентификация включена. |
| ilia-mfa-status-off | Two-factor authentication is off. | Двухфакторная аутентификация выключена. |
| ilia-passkey-list | Passkeys and security keys | Ключи доступа и аппаратные ключи |
| ilia-passkey-none | No passkeys or security keys yet. | Ключей пока нет. |
| ilia-passkey-kind-hardware | Hardware key | Аппаратный ключ |
| ilia-passkey-kind-synced | Synced passkey | Синхронизируемый ключ доступа |
| ilia-passkey-added | Added { $date } | Добавлен { $date } |
| ilia-passkey-last-used | Last used { $date } | Последний вход: { $date } |
| ilia-passkey-never-used | Not used yet | Ещё не использовался |
| ilia-passkey-other-site | Registered for { $rp_id }; it does not work at this site address. | Зарегистрирован для { $rp_id } и не работает по текущему адресу сайта. |
| ilia-passkey-rename | Rename | Переименовать |
| ilia-passkey-delete | Remove | Удалить |
| ilia-passkey-new-name | New name | Новое название |
| ilia-passkey-limit | You have reached the maximum number of keys ({ $max }). Remove one to add another. | Достигнуто максимальное число ключей ({ $max }). Удалите один, чтобы добавить новый. |
| ilia-passkey-backup-advice | Add a second key and keep it somewhere safe, so that losing one key does not lock you out. | Добавьте второй ключ и храните его в надёжном месте, чтобы потеря одного ключа не лишила вас доступа. |
| ilia-passkey-unavailable | Passkeys are off: the server’s public address (CALRS_BASE_URL) is not an https:// address. Ask the administrator. | Ключи доступа отключены: публичный адрес сервера (CALRS_BASE_URL) не начинается с https://. Обратитесь к администратору. |
| ilia-mfa-totp-title | Authenticator app | Приложение-аутентификатор |
| ilia-mfa-totp-on | An authenticator app is set up. | Приложение-аутентификатор подключено. |
| ilia-mfa-totp-off | No authenticator app is set up. | Приложение-аутентификатор не подключено. |
| ilia-mfa-totp-add | Set up authenticator app | Подключить приложение-аутентификатор |
| ilia-mfa-totp-remove | Remove authenticator app | Отключить приложение-аутентификатор |
| ilia-mfa-recovery-title | Recovery codes | Резервные коды |
| ilia-mfa-recovery-left | `{ $count -> [one] { $count } recovery code left *[other] { $count } recovery codes left }` | `{ $count -> [one] Остался { $count } резервный код [few] Осталось { $count } резервных кода [many] Осталось { $count } резервных кодов *[other] Осталось { $count } резервного кода }` |
| ilia-mfa-last-factor | Two-factor authentication is required, so your last sign-in method cannot be removed. Add another one first. | Двухфакторная аутентификация обязательна, поэтому последний способ подтверждения удалить нельзя. Сначала добавьте другой. |
| ilia-mfa-last-factor-off | Removing your last method turns two-factor authentication off and deletes your recovery codes. | Удаление последнего способа отключит двухфакторную аутентификацию и удалит резервные коды. |
| ilia-mfa-changes-help | Every change here needs your password and a passkey or code, and signs out your other sessions. | Любое изменение здесь требует пароля и ключа или кода и завершает остальные ваши сеансы. |
| ilia-mfa-confirm-title | Confirm it’s you | Подтвердите, что это вы |
| ilia-mfa-confirm-help | Enter your password, then confirm with a passkey or a code. | Введите пароль, затем подтвердите ключом или кодом. |
| ilia-passkey-confirm | Confirm with passkey | Подтвердить ключом |
| ilia-mfa-confirm-code | Confirm with code | Подтвердить кодом |
| ilia-mfa-action-passkey-add | You are adding a passkey or security key. | Вы добавляете ключ доступа или аппаратный ключ. |
| ilia-mfa-action-passkey-rename | You are renaming the key “{ $name }”. | Вы переименовываете ключ «{ $name }». |
| ilia-mfa-action-passkey-delete | You are removing the key “{ $name }”. | Вы удаляете ключ «{ $name }». |
| ilia-mfa-action-totp-add | You are setting up an authenticator app. | Вы подключаете приложение-аутентификатор. |
| ilia-mfa-action-totp-remove | You are removing the authenticator app. | Вы отключаете приложение-аутентификатор. |
| ilia-mfa-action-regenerate | You are replacing all recovery codes. | Вы заменяете все резервные коды. |
| ilia-mfa-action-policy | You are changing the two-factor policy for local accounts. | Вы меняете правило двухфакторной аутентификации для локальных учётных записей. |
| ilia-mfa-policy-continue | Continue | Продолжить |

Fork overrides of upstream keys (in the fork section, same key IDs in all locales) are avoided.
Templates use the `ilia-*` keys instead, so upstream text updates still merge cleanly.

---

## 7. Stages and files

### Stage 1: Verifier (no behaviour change, about 1 day)
- `Cargo.toml`: three direct dependencies (§1).
- `Cargo.lock`: root dependency names only.
- `src/main.rs`: `mod webauthn;`.
- `src/webauthn/mod.rs`: `Rp`, `from_base_url`, user handle, limits, form types, options
  builders.
- `src/webauthn/verify.rs`.
- `src/webauthn/softkey.rs`, `#[cfg(test)]`.
- `src/webauthn/tests.rs` and `src/webauthn/testdata/*`.
- Gate: the full check passes, and the lock and feature proofs from §1 are recorded.

### Stage 2: Flows (about 1.5 days)
- `migrations/ilia_010_webauthn.sql`.
- `src/db.rs`: registration and the count test.
- `src/webauthn/store.rs`: credential CRUD, ceremony create and burn, counter compare-and-swap,
  key count, `has_factor`.
- `src/webauthn/flows.rs`:
  - `POST /auth/mfa/passkey`;
  - `POST /auth/mfa/passkey/enroll`;
  - `GET /dashboard/settings/mfa/confirm`;
  - `POST /dashboard/settings/mfa/passkeys`;
  - `GET /static/webauthn.js`;
  - `router()`.
- `src/mfa.rs`:
  - `HAS_FACTOR`, `has_totp`, and `enrolled` meaning "has a factor";
  - `begin_login` "active" now means "has a factor";
  - `begin_totp_enroll`;
  - the recovery branch of `consume_code`;
  - `had_factor` in `verify_challenge`;
  - `Proof` / `consume_proof`;
  - `manage` gains the new actions and the last-factor rule;
  - `policy_save` uses `consume_proof`;
  - `reset` gains the two deletes;
  - `challenge_page` gains `Query`;
  - `challenge_view` and `settings_page` pass the passkey context;
  - `pub(crate)` on `challenge`, `verified_session`, `clear_attempts`, `same_local_user`,
    `new_recovery_codes`, `render`, `completion`, `CHALLENGE_COOKIE` and `SESSION_COOKIE`.
- `src/auth.rs`: the gate (line 94) and the OIDC link check (line 1632).
- `src/settings.rs`: `base_url_env` becomes `pub(crate)`.
- `src/web/mod.rs`: `AppState::passkeys`, one line in `build_state`, a router merge, and
  `RESERVED` if needed.
- `src/commands/user.rs`: the `ResetMfa` help text.
- Tests:
  - `src/mfa/tests.rs`: adjust and extend;
  - `src/web/tests/ilia_passkeys.rs`, new;
  - `src/web/tests/mod.rs`: `mod ilia_passkeys;`;
  - `src/web/tests/agent.rs`: seed and sentinels.

### Stage 3: UI, i18n, docs (about 1 day)
- `assets/webauthn.js`.
- `templates/auth/mfa.html` (hooks).
- New templates:
  - `templates/auth/ilia_mfa_factor.html`;
  - `templates/auth/ilia_mfa_settings.html`;
  - `templates/auth/ilia_mfa_confirm.html`;
  - `templates/auth/ilia_passkey_register.html`.
- `templates/admin.html` (the policy form).
- `i18n/{en,de,es,et,fr,it,nl,pl,pt,ru}/main.ftl` and `i18n/de-formal/main.ftl`.
- `docs/src/authentication.md`: passkeys, choosing a factor, recovery codes for key-only
  users, the base URL requirement.
- `docs/src/security.md`: the "phishing-resistant when a passkey is used" wording, the
  verifier's scope and simplifications.
- `docs/src/cli-reference.md`: `reset-mfa`.
- `CHANGELOG.md` `[Unreleased]`: a fork entry.
- `CLAUDE.md`:
  - a "Passkeys (ilia fork)" section with its "Upstream files touched" list, in the style of
    the other fork sections;
  - the "Local MFA" paragraph says a factor is TOTP or a passkey.
- `ARCHITECTURE.md`: `src/webauthn/` and the new templates and routes.
- Passkey tests never set `CALRS_BASE_URL`: `Rp` is injected through `AppState` (§3.1). Any
  test of `Rp::from_env` itself takes `test_support::ENV_LOCK` and an `EnvGuard`.
- `src/web/apply/site.rs`: the staged module comment changes from "a TOTP code" to "a second
  factor". Optional, and kept unstaged.
- ilia-booking repository, for the lead, in Russian:
  - `docs/decisions.md`: our own verifier, `CALRS_BASE_URL` only, the recovery-code table
    rebuild;
  - `docs/operations.md`: two YubiKeys, the RP ID binding, `reset-mfa`, base URL changes;
  - `docs/fork.md`: the touched upstream files and the table divergence.
- Molecule: no change, because the role already sets `CALRS_BASE_URL`. Optionally,
  `verify.yml` can check that `/static/webauthn.js` returns 200.

---

## 8. Tests

### 8.1 Verifier unit tests (`src/webauthn/tests.rs`)

**Spec vector:**
- `spec_vector_es256_none_attestation_registers_and_asserts`, using WebAuthn L3 §16.2.

**Software key round trips:**
- `es256_roundtrip`;
- `ed25519_roundtrip`;
- `rs256_2048_roundtrip`.

**clientDataJSON:**
- `es256_high_s_is_accepted`;
- `wrong_type_rejected` (`create` vs `get`);
- `challenge_mismatch_rejected`;
- `challenge_with_padding_or_std_alphabet_rejected`;
- `origin_variants_rejected`, covering:
  - `http://book.example.test`;
  - `https://book.example.test:8443`;
  - `https://book.example.test/`;
  - `https://evil.book.example.test`;
  - `https://book.example.test.evil`;
  - uppercase host;
  - `null`;
- `cross_origin_true_rejected` and `top_origin_present_rejected` (the §16.4 and §16.5 vectors);
- `bom_and_invalid_utf8_rejected`.

**authenticatorData:**
- `rp_id_hash_mismatch_rejected`;
- `user_present_required`;
- `uv_recorded_not_required`;
- `bs_without_be_rejected`;
- `be_change_rejected_on_assertion`.

**Signatures and keys:**
- `signature_over_other_data_rejected`;
- `signature_by_other_key_rejected`;
- `alg_spki_mismatch_rejected`, covering every pair of the three algorithms;
- `unsupported_alg_rejected` (-35, -36, -257 with RSA-1024, 0);
- `rsa_1024_rejected` and `rsa_4096_accepted`.

**Registration data:**
- `registration_requires_at_flag`;
- `registration_cred_id_must_equal_raw_id`;
- `credential_id_over_1023_rejected`.

**Robustness:**
- `truncated_auth_data_never_panics`: every prefix length from 0 to the full length, for
  registration and assertion;
- `junk_input_never_panics`: a seeded loop over random bytes in every field.

**Counter:**
- `counter_rules`: 0/0 OK, 5→6 OK, 5→5 refused, 5→4 refused, 0→7 OK, 7→0 refused.

**RP from the base URL:**
- `rp_from_base_url`, which:
  - accepts `https://book.ilia.ae`, `https://book.ilia.ae/` and `https://h:8443/x`;
  - accepts `http://localhost:3000`;
  - rejects `http://book.ilia.ae`, `https://1.2.3.4`, `https://[::1]` and
    `https://u:p@host`;
  - returns None for the empty string and for garbage.

**Other:**
- `user_handle_is_stable_and_has_no_email`;
- `options_json_shape`:
  - the algorithm order;
  - `attestation` is `none`;
  - UV is `preferred`;
  - `excludeCredentials` only for the same `rp_id`;
  - `tojson` escapes `</script>`.

### 8.2 MFA unit tests (`src/mfa/tests.rs`, existing tests must stay green)

**Recovery codes:**
- `recovery_codes_survive_migration_rebuild`: seed through the pre-ilia_010 schema, run the
  migration, check the codes still work;
- `key_only_user_has_recovery_codes_and_they_log_in`.

**Session gate:**
- `session_gate_counts_a_passkey_as_mfa`: `mfa_required = 1`, a verified session, keys only →
  valid;
- `session_gate_refuses_after_last_key_removed_by_reset`.

**Login and enrolment:**
- `begin_login_gives_login_challenge_for_key_only_user`;
- `totp_add_for_key_user_keeps_existing_codes`;
- `first_factor_passkey_generates_codes_once`.

**Last-factor rule and `reset`:**
- `disable_totp_keeps_codes_while_keys_remain`;
- `removing_last_factor_when_optional_deletes_codes_and_unverifies`;
- `last_factor_cannot_be_removed_when_required`, for both `disable` and `passkey_delete`;
- `reset_mfa_removes_totp_keys_codes_attempts_sessions`.

**OIDC and policy:**
- `oidc_link_refused_for_key_only_user`;
- `policy_save_accepts_passkey_proof`.

### 8.3 HTTP flow tests (`src/web/tests/ilia_passkeys.rs`, tower `oneshot`, `Rp` injected through `AppState`)

**Full login flows:**
- `login_password_then_passkey_issues_verified_session`, also with `mfa_required = 1`;
- `login_page_defaults_to_passkey_and_offers_secondary_methods`:
  - the totp link only when TOTP is set up;
  - the recovery link always;
- `login_with_recovery_code_for_key_only_user`.

**Replay, expiry and binding:**
- `assertion_replay_is_refused` (the same POST twice);
- `failed_assertion_burns_the_ceremony`;
- `expired_ceremony_refused`;
- `ceremony_from_another_mfa_token_refused`;
- `only_latest_ceremony_survives`;
- `other_users_credential_refused`;
- `user_handle_mismatch_refused`.

**Parallelism, counter, budget:**
- `parallel_assertions_only_one_session` (two tasks, one ceremony, one row in `sessions`);
- `counter_regression_refused_and_logged`;
- `synced_zero_counter_accepted`;
- `passkey_login_does_not_consume_attempt_budget` (the budget is exhausted with wrong codes,
  and the key still works).

**Enrolment:**
- `enroll_first_factor_with_passkey_shows_codes_and_revokes_sessions`;
- `enroll_passkey_after_totp_completed_in_other_tab_refused`;
- `duplicate_credential_id_refused_even_for_other_user`;
- `eleventh_key_refused`.

**Management:**
- `management_requires_password_and_factor`: each action without a password, with a wrong
  password, without a proof, with a wrong code, and with a passkey proof bound to another
  session;
- `passkey_add_requires_step_up_and_register_ceremony_is_single_use`;
- `register_ceremony_dies_with_password_change`;
- `rename_and_delete_revoke_other_sessions`;
- `delete_foreign_key_is_404_and_changes_nothing`.

**Access and the feature switch:**
- `impersonation_and_oidc_get_403` on every new route;
- `feature_off_routes_404_and_no_button` with `Rp` = None;
- `feature_off_existing_keys_listed_and_removable_with_recovery_code`;
- `key_for_other_rp_id_is_not_offered_but_counts_as_factor`.

**Admin policy:**
- `policy_change_via_confirm_page_with_passkey`;
- `oidc_admin_policy_unchanged`.

**Static file and headers:**
- `webauthn_js_served_with_js_content_type_nosniff_no_cors`;
- `csp_and_permissions_policy_unchanged_on_mfa_pages`;
- `mfa_pages_no_store_and_no_referrer`;
- `options_json_cannot_break_out_of_script_tag` (a user name containing `</script>`).

### 8.4 Existing suites touched
- **`src/web/tests/agent.rs`, `seed_everything`.** Add one credential and one ceremony row,
  and add `("ilia_webauthn_credentials", "credential_id" | "public_key" | "name" |
  "transports")` and `("ilia_webauthn_ceremonies", "session_id")` to `SENTINEL_COLUMNS`.
  `challenge_hash` and `password_hash` are already covered by `secret_named`.
- **`src/db.rs`.** The migration count and the new names.
- **`src/i18n.rs`.** The existing tests run unchanged:
  - `every_locale_covers_every_english_key`;
  - `plural_messages_carry_the_locale_categories`;
  - the `de-formal` overlay checks.

### 8.5 Optional E2E
`functional_test.py` with a WebDriver virtual authenticator. It needs geckodriver 0.37 or newer
and Firefox 152 or newer. It runs one ES256 register-and-login flow, and it is not part of the
required check.

### 8.6 Check
```
export DOCKER_HOST=unix://$HOME/.docker/run/docker.sock
bw.sh <worktree> rw 'cargo fmt'
bw.sh <worktree> ro 'cargo fmt --check && cargo clippy --locked --profile=test && cargo test --locked --bin calrs'
```
On "No space left on device", stop and report.

---

## 9. Security notes (reviewer checklist)

- **RP ID and origin.** They come only from `CALRS_BASE_URL` and are compared as exact
  strings. `Host` and `X-Forwarded-*` are never read. `crossOrigin` and `topOrigin` are
  refused, which matches `frame-ancestors 'self'`.
- **Ceremonies.**
  - single use, burned on failure;
  - expire within 10 minutes (the MFA challenge's lifetime);
  - bound to the MFA token (login) or to the session and password hash (settings);
  - cascade-deleted by `revoke_access`, suspension, user deletion and password changes.
- **Counter and backup flags.** The counter uses a compare-and-swap inside the write-locked
  transaction. A clone is refused and logged. BE is fixed at registration. BS without BE is
  refused.
- **No guessable secret is untrottled.** The passkey and recovery paths skip the attempt
  budget at login only. Every management or policy POST still takes an attempt, because it
  checks the password.
- **Removing factors.** The user's last factor cannot be removed while MFA is required. When
  it is removed under an optional policy, the recovery codes go with it, so no orphaned codes
  remain.
- **Fail closed.** Unknown algorithms, short RSA keys and malformed input are refused, and no
  path panics (covered by the fuzz loops). With the feature off, the ceremony routes return
  404, and keys still count as a factor.
- **Browser side.** There are no inline handlers. The JS file is same-origin. The CSP and
  `Permissions-Policy` are unchanged. The options JSON is HTML-safe through `tojson`.
- **Dependencies and stored data.** No new crates. Only public keys and counters are stored.

---

## 10. Status and deviations

**Done.**
- Stage 1 (verifier): `Cargo.toml` gains `p256`, `ed25519-dalek`, `rsa`; `Cargo.lock` changes
  only in the `calrs` package's dependency names. `src/webauthn/{mod,verify,softkey,tests}.rs`
  and `testdata/`.
- Stage 2 (flows): migration, store, flows, the gate, `mfa.rs` actions, `reset-mfa`, the agent
  sentinels.
- Stage 3 (UI, i18n, docs): the templates, `assets/webauthn.js`, about 64 `ilia-*` keys in all
  11 locales, `CHANGELOG.md`, `README.md`, `CLAUDE.md`, `ARCHITECTURE.md`,
  `docs/src/authentication.md` and `docs/src/security.md`.

**Built differently from the plan.**
- The key name is optional; when empty it is "Security key N" or "Passkey N", from the key's
  backup flag.
- The settings page has two add buttons, **Add a security key** and **Add a passkey on this
  device or phone**. Each opens the confirm page with `kind=security-key|device`, which a hidden
  field carries to `POST /dashboard/settings/mfa` (`ManagementForm::kind`) and on to the
  register page, where the chosen button leads. Both buttons are always on the register page.
- A failed passkey shows the generic `mfa-invalid` banner, so `ilia-passkey-invalid` is not used.
- One key was added in stage 3: `ilia-passkey-kinds-help`, the sentence under the two add
  buttons. It does not address the reader in German, so `de-formal` inherits it.
- With passkeys off and no key, the settings page has no passkey section at all, and the
  first-factor page is upstream's QR page; the "passkeys are off" notice is shown only to a user
  who has keys.
- The flow tests live in `src/webauthn/flow_tests.rs`, and the page tests in
  `src/webauthn/flow_tests/ui.rs`, not in `src/web/tests/ilia_passkeys.rs`, so the staged
  `src/web/tests/mod.rs` is not touched.
- `Rp::from_env()` returns `None` under `cfg(test)`, so no test reads the environment.
- `Permissions-Policy` is left as it is: `publickey-credentials-create` and `-get` default to
  `self`, so listing them would change nothing. A test pins that the header does not mention
  them.
- `webauthn.js` also catches an Enter-key submit of a passkey form whose key fields are empty
  (the password on the confirm page, the key name) and runs the key button instead, so the
  server never counts that as a wrong attempt. It disables every key button while a ceremony
  runs.

**Left.**
- ilia-booking docs in Russian (`docs/decisions.md`, `docs/operations.md`, `docs/fork.md`): our
  own verifier, `CALRS_BASE_URL` only, the recovery-code table rebuild, two YubiKeys, the RP ID
  binding when the address changes, `reset-mfa`.
- The optional E2E test with a WebDriver virtual authenticator (§8.5), and the optional Molecule
  check that `/static/webauthn.js` answers 200.
- A real-browser check on the owner's devices (YubiKey over USB and NFC, Touch ID in Safari,
  an Android phone over hybrid) before release.

