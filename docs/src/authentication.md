# Authentication

calrs supports two authentication methods: local accounts and OIDC (OpenID Connect) SSO.

![Login page](images/login.png)

## Local accounts

### Registration

- The first user to register becomes **admin**
- Registration can be enabled/disabled from the admin dashboard or CLI
- Registration can be restricted to specific email domains

```bash
# Disable open registration
calrs config auth --registration false

# Restrict to a domain
calrs config auth --allowed-domains company.com

# Allow any domain
calrs config auth --allowed-domains any
```

### Password hashing

Passwords are hashed with **Argon2** (via the `argon2` crate with `password-hash`). Plain-text passwords are never stored.

### Sessions

- Server-side sessions stored in SQLite
- 30-day TTL
- Session ID in an **HttpOnly** cookie (not accessible to JavaScript)
- Sessions are invalidated on logout

### Two-factor authentication (TOTP)

Local accounts can enable two-factor authentication in **Profile & Settings >
Manage two-factor authentication**. In this fork the second step can also be a
passkey or a security key instead of an authenticator app; see
[Passkeys and security keys](#passkeys-and-security-keys-ilia-fork). For the
authenticator app, confirm your password, scan the QR code with
an authenticator app (or enter the displayed key manually), then enter its
six-digit code. Setup expires after 10 minutes. The authenticator uses SHA-1,
six digits, and a 30-second interval; keep the server and phone clocks in sync.

Save the ten recovery codes displayed after setup. Each code can replace an
authenticator code once, and the codes are not displayed again. You can generate
a replacement set in the same settings page using your password and a valid
authenticator or recovery code. This invalidates all previous recovery codes.

Once enabled, local login requires your password followed by an authenticator
or recovery code. A used authenticator code cannot be used again, including for
a settings change; wait for the app's next code.

After ten wrong codes in a row, the account has to wait 15 minutes before each
further try, and the wait doubles after every further ten wrong codes, up to a
day. A correct code starts the count over. The limit is per account, so starting
a new login or changing networks does not reset it. Recovery codes are never
held back by it, so if someone who knows your password keeps entering wrong
codes, you can still sign in with a recovery code; then change your password.
Every wrong code is logged with the account and the client address, so an
administrator can see a guessing attempt in the server log.

Enabling MFA, disabling it, or replacing recovery codes signs out your other
sessions. Ordinary profile, authentication, SMTP, and password saves preserve
MFA configuration. Administrators cannot change another user's MFA while
impersonating them.

#### Requiring MFA

In **Admin > Two-factor authentication**, set the local account policy to
**Required**. Local administrators must enroll first and confirm policy changes
with their password and a code (in this fork: or a passkey, on the "Confirm it's
you" page that **Continue** opens). OIDC administrators authenticate through their
identity provider instead, and change the policy without signing in again, so
their account's protection at the identity provider is what guards it.

Requiring MFA invalidates existing local sessions that have not passed MFA.
Local users without MFA must enroll at their next login, including newly
registered users, before they can access the dashboard. Users cannot disable
MFA while it is required. Switching the policy back to **Optional** preserves
users' existing MFA settings. OIDC logins are unaffected: enforce MFA at the
identity provider for SSO accounts.

Required enrollment happens at login, so the first person to log in with an
account's password sets up its authenticator. Require MFA before handing out
passwords, or have each user enroll while the policy is still optional.

An account that has set up local MFA is not linked to single sign-on
automatically when an identity provider asserts the same email address, since
that would replace its local MFA with the provider's. Run `calrs user reset-mfa`
first if the account should move to SSO.

#### Lost authenticator and recovery codes

A server operator can reset a local user's MFA:

```bash
calrs user reset-mfa alice@example.com
```

Use the same data directory as the server (`--data-dir` or `CALRS_DATA_DIR`).
This removes the user's authenticator, passkeys and security keys (ilia fork)
and recovery codes, and revokes their sessions and pending login challenges. It does not change their password or the
instance policy. If MFA is required, the user must enroll again at next login.
Back up the instance's `secret.key` together with the database: TOTP secrets use
the same encryption key as other stored credentials.

### Passkeys and security keys (ilia fork)

A local account's second step can be a **passkey** instead of, or next to, an
authenticator app:

- a **security key** such as a YubiKey (USB or NFC, FIDO2 or U2F);
- a **passkey on this device or phone**: Touch ID, Face ID, Windows Hello, an
  Android phone, or a passkey synced by iCloud Keychain, Google Password
  Manager or a password manager.

A passkey is the safest choice: the browser only uses it on the site it was
registered for, so a look-alike page that relays your password cannot get the
second step from it, while it can ask for a six-digit code. Your fingerprint or
face never leaves your device; calrs stores only the key's public half.

#### Requirements

Passkeys are on only when `CALRS_BASE_URL` is set to an `https://` address with
a host name, for example `https://cal.example.com` (`http://localhost:<port>` is
allowed for development). The host name, here `cal.example.com`, is the
"relying party ID" every key is bound to, and the browser's origin must match
`CALRS_BASE_URL` exactly. The base URL saved in the admin panel is not used for
this, so a web form can never move the binding. Without a usable
`CALRS_BASE_URL` the passkey buttons are hidden and the passkey routes answer
404; keys already registered are still listed and can be removed with a code.

**Changing the public address** (another host name, not only another path)
makes every registered key unusable at the new address: the settings page
marks it "Registered for" the old host name. Users then sign in with the authenticator app or a
recovery code, remove the old key and add it again. Keep an authenticator app
or the recovery codes when you plan such a move.

The browser needs WebAuthn with `getPublicKey()` and JavaScript for this site.
Without them the page says so and offers the code methods. calrs asks every
new key for an ES256 (P-256) key pair first, which every current browser can
read, so a YubiKey or another security key also registers in Safari. A key
that offers only Ed25519 or RSA cannot be added in Safari, whose
`getPublicKey()` returns nothing for those types: the page says the browser
cannot read this type of key, and another browser (Firefox, Chrome) works.

Open the site at the address in `CALRS_BASE_URL` exactly: on any other host
name for the same server (a `www.` alias, say) the browser refuses the key, and
the page says where passkeys work. A sign-in started with the password stays
valid for 10 minutes, and its passkey request with it.

#### Signing in

After the password, the page leads with **Sign in with passkey**. Press it and
follow the browser: touch the security key (and enter its PIN if it has one),
or confirm with Touch ID, Face ID, Windows Hello or your phone. Below it,
**Use authenticator code** (when an authenticator app is set up) and **Use
recovery code** switch to a code field; both work without JavaScript. A failed
or cancelled passkey does not count toward the wrong-code limit, so someone who
knows your password cannot lock you out of your key by entering wrong codes.

#### Setting it up

With no second step yet, **Set up two-factor authentication** asks for your
password, then offers **Add a security key** and **Add a passkey on this device
or phone**, or **Use an authenticator app instead**. The key's name is
optional ("Security key 1" or "Passkey 1" is used when it is empty, up to 64
characters). The ten recovery codes are shown once, after this first step; a
recovery code replaces a passkey or an authenticator code for one sign-in. When
MFA is required, the same choice appears at the next login.

#### Managing keys

**Manage two-factor authentication** lists every key with its name, its kind
(**Hardware key** for a key bound to one device, **Synced passkey** for one your
devices share), when it was added and last used. From there you can add a key
(**Add a security key** or **Add a passkey on this device or phone**), rename or
remove a key, set up or remove the authenticator app, and generate new recovery
codes. Every change first opens **Confirm it's you**: your password, then a
passkey (or an authenticator or recovery code). Each change signs out your
other sessions. Up to 10 keys can be registered; register a second key and keep
it somewhere safe, so that losing one does not lock you out.

While MFA is required, your last method (the last key, or the authenticator
app when no key is left) cannot be removed; add another one first. Nor can the
last method that works at this address: a key registered for another host
name, or any key while passkeys are off, does not count for that, so the
authenticator app stays while it is the only working method (such a stale key
can still be removed). Under an
optional policy, removing the last method turns two-factor authentication off
and deletes the recovery codes.

A security key whose signature counter goes backwards (a sign of a cloned key)
is refused and logged as "possible cloned authenticator"; remove it and add a
new one. Synced passkeys report no counter and are not affected.

The administrator's MFA policy counts a passkey like an authenticator app:
**Required** is met by either. OIDC accounts are unaffected.

### User management (CLI)

```bash
calrs user create --email alice@example.com --name "Alice" --admin
calrs user list
calrs user set-password alice@example.com
calrs user promote alice@example.com    # → admin
calrs user demote alice@example.com     # → user
calrs user disable alice@example.com
calrs user enable alice@example.com
```

## OIDC / SSO

calrs supports OpenID Connect for single sign-on, tested with Keycloak and compatible with any OIDC provider (Authentik, Auth0, etc.).

> Using **Authentik**? See the dedicated [Authentik (OIDC SSO)](./authentik.md) page — it covers the full setup plus the `email_verified` claim gotcha introduced in Authentik 2025.10 that otherwise blocks every login.

### Features

- **Authorization code flow with PKCE** — no client secret stored in the browser
- **Auto-discovery** — reads `.well-known/openid-configuration` from the issuer URL
- **User linking by email** — if a local user exists with the same email, the OIDC identity is linked
- **Auto-registration** — new users are created on first OIDC login (if enabled)
- **Group sync** — groups from the `groups` JWT claim are synced on each login and can be linked to teams

### Configuration

```bash
calrs config oidc \
  --issuer-url https://keycloak.example.com/realms/your-realm \
  --client-id calrs \
  --client-secret YOUR_CLIENT_SECRET \
  --enabled true \
  --auto-register true
```

Or from the **Admin dashboard > OIDC** section.

### Keycloak setup

1. Create a new **OpenID Connect** client:
   - **Client ID:** `calrs`
   - **Client authentication:** ON (confidential)
   - **Valid redirect URIs:** `https://your-calrs-host/auth/oidc/callback`
   - **Web origins:** `https://your-calrs-host`
2. Copy the **Client secret** from the Credentials tab
3. Set `CALRS_BASE_URL` to your public URL before starting the server

The login page will show a **"Sign in with SSO"** button when OIDC is enabled.

## User roles

| Role | Capabilities |
|---|---|
| `user` | Manage own event types, calendar sources, bookings |
| `team admin` | Everything above + manage team event types and team members |
| `admin` | Everything above + user management, auth settings, OIDC config, SMTP config |

The first registered user is automatically promoted to admin.

## Email notifications (SMTP)

SMTP configuration is required for booking confirmation emails. Without it, bookings still work but no emails are sent.

```bash
calrs config smtp \
  --host smtp.example.com \
  --port 587 \
  --username calrs@example.com \
  --from-email calrs@example.com \
  --from-name "calrs"

# Test the configuration
calrs config smtp-test you@example.com

# View current config
calrs config show
```

Or configure from the **Admin dashboard > SMTP** section.
