# Security

This page documents calrs's security measures and known limitations.

## Authentication

- **Password hashing** — Argon2 with random salt (via the `argon2` + `password-hash` crates). Passwords are never stored in plaintext.
- **Sessions** — 32-byte random tokens (cryptographically secure via `OsRng`), stored server-side in SQLite with 30-day TTL.
- **Cookie flags** — All session cookies use `HttpOnly; Secure; SameSite=Lax`. The `Secure` flag ensures cookies are only sent over HTTPS.
- **OIDC** — Authorization code flow with PKCE, state validation, and nonce verification. Tested with Keycloak.
- **Local MFA** — Optional TOTP, enforceable by the administrator for local accounts. A ten-minute challenge grants no session access until verification or required enrollment succeeds. Secrets are encrypted with AES-256-GCM; recovery codes contain 128 random bits and are stored as SHA-256 hashes. Code consumption and session creation are transactional to prevent replay under concurrent requests. Setup and recovery pages use `Cache-Control: no-store`.

MFA code verification and management also have a persistent account-wide limit
of ten attempts per 15 minutes. TOTP accepts the current 30-second interval and
one neighboring interval in either direction. Previously accepted intervals
cannot be reused. TOTP protects against a stolen password alone; it is not
phishing-resistant. See [MFA setup and recovery](authentication.md#two-factor-authentication-totp).

### Passkeys and security keys (ilia fork)

A passkey or security key (WebAuthn) can be the second factor instead of, or
next to, TOTP. Used at sign-in it is phishing-resistant: the browser signs the
origin it is on, and the server accepts only its own. See
[Passkeys and security keys](authentication.md#passkeys-and-security-keys-ilia-fork).

- **Verifier.** A minimal relying-party verifier of the fork's own
  (`src/webauthn/verify.rs`, no I/O, no panics on any input, checked step by
  step against WebAuthn Level 3 §7.1 and §7.2 and its §16 test vectors). It
  adds no crate: `p256`, `ed25519-dalek` and `rsa` were already in the build
  through `openidconnect`, with the same versions and features.
- **Relying party.** The RP ID and the origin come only from the
  `CALRS_BASE_URL` environment variable (an `https://` URL with a host name, or
  `http://localhost` for development), never from `Host` or `X-Forwarded-*`
  and never from the admin-editable base URL. The origin in `clientDataJSON`
  must equal it exactly; `crossOrigin: true` and any `topOrigin` are refused,
  as is an `rpIdHash` for another RP ID. Without a usable value the feature is
  off and its routes answer 404.
- **Parameters.** Attestation `none`; algorithms Ed25519 (-8), ES256 (-7) and
  RS256 (-257, RSA keys of 2048 to 4096 bits), and the declared algorithm must
  match the key; user presence required; user verification `preferred`
  (recorded, not required, since the password is the first factor). Size caps
  on every field are checked before decoding.
- **Simplification.** The server does not parse the CBOR `attestationObject`:
  with attestation `none` nothing signs the registration, so the key comes
  from the browser's `getPublicKey()` (SPKI) and the authenticator data from
  `getAuthenticatorData()`, as §5.2.1.1 provides. The credential ID in the
  authenticator data must equal the one the browser reports.
- **Ceremonies.** 32 random bytes from `OsRng` per challenge, stored only as a
  SHA-256 hash, valid at most 10 minutes (as long as the sign-in it belongs
  to; the browser waits 5 minutes for the key after a press), single use (deleted when the answer
  arrives, even when it fails), and bound to the pending login challenge or,
  in the settings, to the session and the current password hash. Revoking a
  user's access (suspension, password change, any MFA change) ends them.
- **Counter and backup flags.** The signature counter is updated with a
  compare-and-swap in the same transaction that issues the session; a counter
  that does not grow (unless both are 0, as synced passkeys report) is refused
  and logged as a possible clone. Backup eligibility is fixed at registration,
  and backup state without eligibility is refused. Both are shown to the user
  as "Hardware key" or "Synced passkey".
- **What is stored.** The public key, the counter, the flags, the transports
  and the name. A copy of the database cannot be used to sign in with a key.
- **Attempts.** A passkey at sign-in takes nothing from the wrong-code limit,
  since a signature cannot be guessed; every refusal is logged with the account,
  the client address and a reason code. Every settings or policy form that
  checks the password still counts an attempt.
- **Changes.** Adding, renaming or removing a key, adding or removing TOTP and
  replacing the recovery codes all need the password and a current factor, and
  sign out the user's other sessions. The last factor cannot be removed while
  MFA is required. `calrs user reset-mfa` removes keys too.
- **Browser side.** `/static/webauthn.js` is a same-origin file with no inline
  handlers; the options travel in a `<script type="application/json">` block
  and the answer goes back in an ordinary form POST, so the CSP and
  `Permissions-Policy` are unchanged (the `publickey-credentials-*` features
  default to `self`).

## Rate limiting

Login attempts are rate-limited per IP address:

- **10 attempts** per **15-minute window**
- After the limit, further attempts return an error without checking credentials
- The client address is the TCP peer, or, when the peer is a trusted proxy, the address that proxy reports in `X-Forwarded-For` (see below)

The same rule identifies the client for every per-address limit (sign-in, two-factor codes, bookings). An IPv6 client counts by its /64, so rotating addresses inside it does not buy a new budget.

### Trusted proxies

calrs believes `X-Forwarded-For` only from a **trusted proxy**. It joins every `X-Forwarded-For` line in the order they arrived (HAProxy adds a line of its own rather than extending the client's) and reads the list right to left, skipping the hops that are themselves trusted proxies; the first address that is not one is the client. A port on a hop (`198.51.100.7:5678`, `[2001:db8::1]:443`) is dropped. A hop that is not an address at all stops the walk, and every such request shares one bucket of its own, never the proxy's. A peer that is not trusted is the client itself, and whatever `X-Forwarded-For` it sends is ignored, so a client cannot claim a fresh address on every attempt. `X-Real-IP` and `Forwarded` (RFC 7239) are ignored: configure your proxy to set `X-Forwarded-For`.

The trusted list is `CALRS_TRUSTED_PROXIES` when set: a comma or space separated list of addresses and CIDR ranges (`127.0.0.1, 10.0.0.0/8, fd00::/8`), or `none` to trust no peer. A `/0` range (`0.0.0.0/0`, `::/0`) is refused, since it would trust every peer. An unusable or refused value trusts no peer and logs an error. When it is unset, the default is loopback (`127.0.0.0/8`, `::1`), private (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), unique local (`fc00::/7`) and link-local (`169.254.0.0/16`, `fe80::/10`). `calrs serve` logs the list in force when it starts.

What this means for common setups:

- **Caddy or Nginx on the same host, or on the same Docker network**: works out of the box, the proxy's address is in the default list. Make sure it sets `X-Forwarded-For` (Caddy does; Nginx needs the line below).
- **Docker's userland proxy** (ports published over IPv6, or hairpin connections from the host): every client arrives from the bridge gateway, a trusted address that sends no header, so all clients share one rate-limit bucket and one person mistyping a password can lock everyone out for 15 minutes. Put the reverse proxy in front, or publish the port with the userland proxy disabled so calrs sees real client addresses.
- **calrs exposed directly on a LAN, no proxy**: by default LAN peers are trusted, so a LAN client could forge `X-Forwarded-For`. Set `CALRS_TRUSTED_PROXIES=none`.
- **Behind a proxy, with LAN users**: the default trusts every private address, so a LAN user whose request goes through the proxy can still add a forged private hop that calrs skips. List only the proxy's own address (`CALRS_TRUSTED_PROXIES=127.0.0.1`, or its Docker address) rather than relying on the default ranges.
- **`none` behind a proxy** turns every client into the proxy's address: one bucket for everyone. Use `none` only when nothing sits in front of calrs.
- **calrs exposed directly on the internet**: public peers are never trusted by default, so each client is counted by its own address.
- **A proxy on a public address** (a load balancer, a CDN): list its addresses or ranges, otherwise every client is counted as the proxy.

### Nginx

```nginx
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
```

### Caddy

Caddy sets `X-Forwarded-For` automatically.

### Booking endpoints

Booking submissions are rate-limited per IP address:

- **10 attempts** per **5-minute window**
- Applies to all booking handlers (user, team, and legacy)

## Embedding

Pages are sent with `frame-ancestors 'self'` and `X-Frame-Options: SAMEORIGIN`, so other sites cannot frame them. The exception is a public booking page (`/u/…`, `/team/…`, `/g/…`, a legacy `/{slug}` or `/{slug}/book`) asked for with `?embed=1`, which is what `embed.js` loads in its iframe: sign-in, the dashboard, token pages and the TON payment page stay unframeable whatever the query says.

Which sites may frame an embedded booking page is `CALRS_EMBED_ORIGINS` (ilia fork), read once at startup and logged by `calrs serve`:

- unset: any site (`frame-ancestors *`), as upstream;
- `*`: the same, explicitly;
- `none`: no other site; the page keeps `frame-ancestors 'self'` and `X-Frame-Options: SAMEORIGIN`;
- a comma or space separated list of origins, such as `https://example.org https://www.example.org`: `frame-ancestors 'self'` plus exactly those (`'self'` keeps the embed code generator's preview working). An origin is `https://host[:port]` with no path, query, user info or wildcard; plain `http://` only for `localhost`, `127.0.0.1` and `.onion` hosts. At most 16.

A value that does not parse, an empty one included, turns embedding off and logs `CALRS_EMBED_ORIGINS refused` with the reason: failing open would let every site frame the booking form. With a list, legacy browsers that read only `X-Frame-Options` still get `ALLOWALL`, since that header cannot name origins; every current browser follows `frame-ancestors` and ignores it.

`embed.js` fills every element with `data-calrs-inline="<booking link>"` (optionally `data-calrs-config='{"theme":"auto"}'`) when it loads, so a host page can embed with one `<script src=".../embed.js" defer></script>` and no inline script, and its own CSP only needs this origin in `script-src` and `frame-src`. It loads only `http(s)` links.

The widget speaks the host page's language and takes its theme (ilia fork). `embed.js` sends the primary subtag of the page's `<html lang>` as `lang` (`config.lang` overrides it, `""` leaves the visitor's browser language); for an embedded booking request a `lang` naming a shipped language is that request's language pick, carried through the flow's links and the form's POST, so the booking stores it and the guest's emails follow it. Nothing is stored in a cookie, and any other value is ignored, never echoed. Without `config.theme`, the background behind the widget decides between `light` and `dark`, and a later change (a site's own dark-mode switch) reaches the widget as a `calrs:theme` message, which the embedded page accepts only from its parent and only as `light` or `dark`. The iframe gets the same `color-scheme`, so it stays transparent.

## CSRF protection

All POST forms are protected against cross-site request forgery using the **double-submit cookie** pattern:

- A `calrs_csrf` cookie is set automatically on every response (via middleware)
- Client-side JavaScript reads the cookie and injects a hidden `_csrf` field into all POST forms
- On submission, the server verifies that the cookie value matches the form field
- Mismatches return a `403 Forbidden` response

This protects all 31 POST endpoints including booking submissions, settings changes, admin actions, and authentication forms. Multipart forms (avatar/logo/favicon upload) pass the token via query parameter.

The cookie uses `SameSite=Lax` and is intentionally NOT `HttpOnly` so the client-side script can read it.

## Input validation

All user-submitted data is validated server-side:

- **Booking forms** — name (1–255 chars), email (format + length), notes (max 5,000 chars), date (max 365 days in the future)
- **Registration** — name (1–255 chars), email format and length validation
- **Settings** — name length, booking email format validation
- **Avatar upload** — strict content-type whitelist (JPEG, PNG, GIF, WebP only)
- **HTML templates** — `maxlength` attributes on form inputs (defense in depth)

## ICS injection protection

User-supplied values (guest name, email, event title, location, notes) are sanitized before being inserted into `.ics` calendar invites:

- Carriage returns (`\r`) and newlines (`\n`) are stripped to prevent ICS field injection
- Semicolons and commas are escaped per RFC 5545

This prevents attackers from injecting arbitrary iCalendar properties (e.g., extra attendees, recurrence rules) through booking form fields.

## SQL injection

All database queries use parameterized bindings via `sqlx`. No SQL is constructed through string concatenation.

## XSS (cross-site scripting)

All HTML output is rendered through Minijinja, which **auto-escapes** all template variables by default. No `|safe` or `|raw` filters are used.

## Double-booking prevention

A SQLite partial unique index prevents two bookings for the same event type and time slot:

```sql
CREATE UNIQUE INDEX idx_bookings_no_overlap
ON bookings(event_type_id, start_at)
WHERE status IN ('confirmed', 'pending');
```

Additionally, all booking handlers wrap the availability check and INSERT in a database transaction (`BEGIN IMMEDIATE`), preventing race conditions between concurrent requests.

## Error handling

Web handlers use explicit error handling instead of panics. Template rendering failures, date parsing errors, and database errors return user-friendly HTTP error responses rather than crashing the server process.

## Token-based actions

Certain actions can be performed without authentication, using single-use-like tokens:

- **Cancel token** — allows guests to cancel their booking via a link in the confirmation email
- **Confirm token** — allows hosts to approve or decline pending bookings via links in the approval request email

Tokens are UUID v4 (128-bit random), stored with unique indexes in the database. They are not invalidated after use (the booking status check prevents replay — a token for an already-confirmed booking shows "already approved"). These links should be treated as sensitive — anyone with the link can perform the action.

## Known limitations

### CalDAV credential storage

CalDAV and SMTP passwords are encrypted at rest using **AES-256-GCM**. The encryption key is auto-generated at `$DATA_DIR/secret.key` on first run, or can be provided via the `CALRS_SECRET_KEY` environment variable. Legacy hex-encoded passwords (from pre-v0.10.0) are auto-migrated to encrypted format on startup. Protect your `secret.key` file with filesystem permissions.

### No brute-force account lockout

Rate limiting is per-IP, not per-account. A distributed attack from many IPs would not be rate-limited. Consider using fail2ban or your reverse proxy's rate limiting for additional protection.

### SSRF (server-side request forgery)

CalDAV and EWS source URLs are user-supplied, and resource feeds are supplied by admins. When one is saved, `validate_caldav_url()` refuses a URL whose hostname resolves to a private or reserved IP range (loopback, RFC1918, link-local, CGNAT, ULA, IPv6 forms that carry such an IPv4 address, etc.).

The requests themselves do not rely on that first lookup. Every client that fetches one of these URLs (CalDAV, EWS and Autodiscover, resource feeds and their CalDAV collections, the `calrs resource probe` command) resolves the host itself at each connection, refuses the connection when any address is private or reserved, and connects only to the addresses it checked, so a DNS answer that changes between the check and the request cannot send it to an internal host. This also covers hosts a server names on its own (CalDAV hrefs on another host, redirects): a host name goes through the same check. An IP literal never needs a lookup, so it is checked against the allowlist at every request instead, the saved URL's own literal included: a source on a private IP literal stops working when its entry is removed from the allowlist. A refused request records a generic error on the sources page and in the owner's alert email; the address itself is only logged, at warn level, for the administrator. List an IP literal in the allowlist the way it appears in the URL, without brackets for IPv6 (`10.0.0.5`, `fd00::1`); any spelling of the same address matches.

#### Behind an outbound proxy

Calendar requests honour `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` and `NO_PROXY` (lowercase forms too), since many deployments, on-prem Exchange ones especially, reach the outside only through one. A request that goes through the proxy cannot be pinned by calrs: the proxy resolves the host name, not calrs. For those requests the URL is still checked when it is saved, server-supplied IP literals and redirects are still checked, and calrs only resolves the proxy's own host (exempt from the check, since you configured it); **filtering private addresses is then the proxy's job**, so configure it to refuse internal destinations. calrs logs this when it starts, for every command. Hosts that `NO_PROXY` sends direct get the full check and pinning. A URL, server-supplied href or redirect that names a proxy's own host (on any port) is refused unless that host is on the private-host allowlist, since calendar URLs never need to target the proxy itself. This build has no SOCKS support: a `socks5://` proxy setting is ignored, logged at startup, and the requests it would have carried go direct, checked and pinned.

Left out on purpose: the fixed Google endpoints (constants, not configuration), and the admin-configured captcha server, meeting webhook, SMS gateway and OIDC issuer, which were never under the private-host policy because pointing them at a service on the local network is their usual setup. The captcha verification, the meeting webhook and the SMS gateways follow no redirects. An egress firewall that keeps calrs away from internal ranges remains good practice.

#### Allowing private CalDAV hosts (self-hosting)

Self-hosted deployments often run calrs and the CalDAV server on the same private network (e.g. a docker-compose stack where `http://radicale:5232` resolves to an RFC1918 address). To permit specific hostnames to resolve to private/reserved IPs, configure a comma-separated allowlist of hostnames (or literal IPs). Two ways, in order of precedence:

1. **Environment variable** (takes precedence — ops override):
   ```
   CALRS_ALLOW_PRIVATE_HOSTS=radicale,nextcloud.local,127.0.0.1
   ```
2. **Admin UI / CLI** (persisted in the DB, used when the env var is unset):
   - Admin panel → **System settings** → *Private-host allowlist*
   - `calrs config general --allow-private-hosts radicale,nextcloud.local,127.0.0.1`

When the environment variable is set it overrides the stored value (the admin panel shows a "set by environment" badge), and neither the admin panel nor `calrs config general` will write that field to the database — they refuse and say so. That is deliberate: a value stored while the env var was set would sit dormant and then silently become the effective allowlist the day the variable is removed. Change it where it is set, or unset the variable first. Matching is case-insensitive and exact (no wildcards or subdomain matching). Only the listed hosts bypass the private-IP check; every other host is still validated. Keep this list as small as possible, scheme validation (http/https only) still applies.

In a trusted multi-user deployment (e.g., behind OIDC) this is low risk. For public-registration instances, configure egress filtering at the network level.

## Recommendations for production

1. **Always use HTTPS** — the `Secure` cookie flag requires it
2. **Set `CALRS_BASE_URL`** to your public HTTPS URL
3. **Configure your reverse proxy** to set `X-Forwarded-For` correctly, and list it in `CALRS_TRUSTED_PROXIES` if it is not on a private address (or set `none` when nothing sits in front of calrs)
4. **Restrict filesystem access** to the data directory (contains the SQLite database with credentials)
5. **Disable registration** if using OIDC (`calrs config auth --registration false`)
6. **Keep calrs updated** for security patches
