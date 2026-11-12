# CLI Reference

## Global options

```
--data-dir <PATH>    Custom data directory (env: CALRS_DATA_DIR)
```

## Commands

### `calrs source`

Manage CalDAV calendar sources.

```
calrs source add [OPTIONS]
    --url <URL>           CalDAV server URL
    --username <USERNAME> CalDAV username
    --name <NAME>         Display name for this source
    --no-test             Skip the connection test
    --password-stdin      Read the password from stdin (needs --url, --username, --name)

calrs source update <ID> [--name <NAME>] [--url <URL>] [--username <USERNAME>]
                         [--password | --password-stdin]

calrs source list

calrs source test <ID>    Test a connection (ID prefix match)

calrs source remove <ID>  Remove a source and all its data (ID prefix match)
```

Passwords are read from the terminal with echo off. Scripts (Ansible, CI) pipe them on stdin with `--password-stdin` instead, the same flag `docker login` uses; one trailing newline is dropped. Without a terminal and without the flag the command fails rather than store an empty password.

```
printf '%s' "$CALDAV_PASSWORD" | calrs source add --url https://cal.example.com/dav/ \
  --username alice --name Work --password-stdin
```

### `calrs sync`

Pull latest events from all CalDAV sources.

```
calrs sync [OPTIONS]
    --full    Full re-sync (ignore sync tokens)
```

Exits with status 1 when any source (or any calendar of a source) failed to sync, after listing the failures, so it can be used from cron or a monitoring check.

### `calrs calendar`

View synced calendar events.

```
calrs calendar show [OPTIONS]
    --from <DATE>    Start date (YYYY-MM-DD)
    --to <DATE>      End date (YYYY-MM-DD)
```

### `calrs event-type`

Manage bookable event types.

```
calrs event-type create [OPTIONS]
    --title <TITLE>              Event type title (required)
    --slug <SLUG>                URL slug (required)
    --duration <MINUTES>         Duration in minutes (required)
    --description <DESC>         Description
    --buffer-before <MINUTES>    Buffer before (default: 0)
    --buffer-after <MINUTES>     Buffer after (default: 0)
    --bookable-from <DATE>       First bookable day, YYYY-MM-DD (inclusive)
    --bookable-until <DATE>      Last bookable day, YYYY-MM-DD (inclusive)

calrs event-type list

calrs event-type slots <SLUG> [OPTIONS]
    --days <DAYS>    Number of days to show (default: 7, from the first bookable day,
                     clamped to the booking horizon and the bookable date range)
```

`list` shows the [bookable date range](./event-types.md#bookable-date-range) of each event type, marked *upcoming* or *expired* when it is not running.

`slots` consults any [shared resources](./resources.md) attached to the event type: slots where a required resource is busy are not printed, matching the web booking page.

If the event type sets a [booking horizon](./event-types.md#booking-horizon), `--days` is clamped to it, so the CLI shows the same window the booking page offers. Asking for `--days 30` on an event type with a 5-day horizon prints 6 days (today plus five).

With a bookable date range, the window starts at the range's first day instead of today and stops at its last day, so `slots` on a fair three weeks out prints the fair, not an empty week.

### `calrs booking`

Manage bookings.

```
calrs booking create <SLUG> [OPTIONS]
    --date <DATE>          Booking date (YYYY-MM-DD)
    --time <TIME>          Start time (HH:MM)
    --name <NAME>          Guest name
    --email <EMAIL>        Guest email
    --timezone <TZ>        Guest timezone (default: UTC)
    --notes <NOTES>        Optional notes

calrs booking list [OPTIONS]
    --upcoming    Show only upcoming bookings

calrs booking cancel <ID>    Cancel a booking (ID prefix match)
```

`booking create` checks the event type's minimum notice, weekly rules, booking horizon and [bookable date range](./event-types.md#bookable-date-range), like the booking page. It does not apply the page's 366-day limit, since it books on the host's behalf.

`cancel` marks the booking cancelled, sends the cancellation emails, deletes the event from the host's CalDAV write-back calendar, and releases any [shared resource](./resources.md) reservation from the resource's CalDAV calendar(s).

### `calrs config`

Configure SMTP, authentication, and OIDC.

```
calrs config smtp [OPTIONS]
    --host <HOST>           SMTP server hostname
    --port <PORT>           SMTP port (default: 587)
    --username <USERNAME>   SMTP username (omit for an unauthenticated relay)
    --from-email <EMAIL>    Sender email address
    --from-name <NAME>      Sender display name
    --tls-mode <MODE>       starttls (default), tls, or none

calrs config show           Display current configuration

calrs config smtp-test <EMAIL>   Send a test email

calrs config auth [OPTIONS]
    --registration <BOOL>        Enable/disable registration
    --allowed-domains <DOMAINS>  Comma-separated domains or "any"

calrs config general [OPTIONS]      # Without options: show the current values
    --base-url <URL>                Public base URL (empty string clears it)
    --allow-private-hosts <HOSTS>   Hostnames allowed to resolve to private IPs (empty clears)
    --company-link <URL>            Link behind the logo on public pages (empty clears)

calrs config apply --stdin [--dry-run]     # Apply a settings document read from stdin
calrs config apply --export [--user <EMAIL>]   # Print the stored settings as a document

calrs config oidc [OPTIONS]
    --issuer-url <URL>        OIDC issuer URL
    --client-id <ID>          Client ID
    --client-secret <SECRET>  Client secret
    --enabled <BOOL>          Enable/disable OIDC
    --auto-register <BOOL>    Auto-create users on first login
```

`--company-link` is the setting of Admin → Company link and is checked the same way: only `http://` and `https://` links are accepted, anything else is an error and nothing is written. `calrs config dump` shows it as `auth.company_link`. A running `calrs serve` reads it at startup, so restart it to show a new link.

#### Settings as code: `config apply` (ilia fork)

`calrs config apply` reads one JSON document that declares the settings a deployment manages, applies it and prints one JSON line. The format is `calrs-apply-v1`; the full schema is in `docs/apply-schema.md` in the repository and a complete example is `docs/apply-example.json`.

| Section | What it sets | Through |
|---|---|---|
| `profile` | Settings → Profile of the admin named by `user`: name, username, title, bio, booking email, timezone, language, public page language, week start, dynamic groups, resource lending, default weekly hours, and the avatar (a file already on the server and its SHA-256) | the profile form (`settings_save`) |
| `site` | The admin panel: theme and custom colours, week start, German form of address, registration and allowed email domains, MFA policy, company link, Jitsi; OIDC, Google, captcha and meeting webhook can only be switched `"off"` (every field and secret cleared); the SMS policy; `ton` (ilia fork), the TON tuning keys of Admin → TON payments | each card's own form; `ton` writes the TON settings the page writes |
| `sources` | For an existing source (matched on `url` and `username`): the write calendar and the synced calendars, by display name | the local calendar list only; nothing connects to the server |
| `event_types` | The admin's personal event types by slug, every field of the event type form, weekly hours (`null` follows the profile), frequency limits, the TON price (`ton_price`), Telegram guest mode, enabled; `prune` is `none` (default), `disable` or `delete` for the personal types the document leaves out | the event type form (`create_event_type`, `update_event_type`); the price through the form's price section check and save |

A key left out keeps its setting. `null` clears or inherits, where a setting may be empty; `""` is never accepted. Every object refuses unknown keys, so an older calrs given a newer document stops before writing anything. `base_url` and `allow_private_hosts` are not keys: they belong to the environment. No secret is accepted.

`ton_price` is `null` (free) or the whole price: `{"usd": "500.00", "window_min": null, "enabled": true}`, or `"ton"` in place of `"usd"` for a price in TON. `window_min` (`null`: the server's default) and `enabled` (`false`: the price is kept, the type books free) may be left out. The checks and the limits are the event type form's. A price switched on makes a type with no Telegram mode chosen ask guests for their Telegram, as on the form; `telegram_guest_mode` in the same item wins. Nothing is refused because TON payments are not configured: the form does not refuse it either, and `config apply` does not read `calrs serve`'s environment. A priced type can be booked only once payments are on. `{"ladder": true, "window_min": null, "enabled": true}` prices the type by the site's demand ladder instead (TON; see [TON payments](ton.md)).

`site.ton` (ilia fork) sets the TON tuning keys of **Admin → TON payments**: `window_min`, `grace_min`, `fuse_hours`, `renew_min_lead_hours`, `renew_window_min`, `max_renewals` and `free_per_guest_days` as whole numbers; `underpay_tolerance_ton`, `rate_min_usd`, `rate_max_usd`, `rate_max_spread_pct`, `rate_max_jump_pct`, `alert_email` and `ladder` (`"free,20,40,…"`) as canonical strings; `rate_max_age_min` as a whole number; `debug_booking` (the Debug switch at the top of the page, for the admin's debug booking page) as `true` or `false`, live without a restart; `debug_booking_telegram` (the guest Telegram username that page prefills when the admin's linked chat has none) as a username without `@`, live too; `null` for the default. Left out, nothing changes. Given, its keys belong to `config apply` (read-only on the page), and a key it owned before and now leaves out goes back to the page with its value (reported as `site.ton.<key>.owner`). `config apply` cannot see `calrs serve`'s environment: it validates the stored values as if no `CALRS_TON_*` were set, and a variable set in the server's environment still wins there. The network, the wallet address, the indexer and the API keys are not keys.

Each value must already be in the form the web form stores it in: a value the form would rewrite or drop (an uppercase username, an unknown timezone, a colour that is not `#rrggbb`, a padded title, a slug with a space, a price of `"500"` rather than `"500.00"`) is refused with its path. That is what makes a second run report no change.

The whole run is one SQLite transaction on a single connection. `--dry-run` runs every check and every form, then rolls back, so its report is the one a run would print. `--export` opens the database read-only and prints the stored state in the same format; applying it changes nothing. A stored value no document can state (written by hand, say) leaves its key out of the export.

Output, on stdout, one line:

```json
{"format":"calrs-apply-v1","dry_run":false,
 "changed":[{"path":"site.theme","before":"default","after":"custom"},
            {"path":"event_types[15min]","before":null,"after":"created"}],
 "restart":true,"unmanaged":{"event_types":["old-call"]}}
```

- `changed` lists each setting that changed. A cleared secret shows as `"(secret set)"` changing to `null`, the avatar as `"sha256:<hex>"`.
- `restart` is true when a value `calrs serve` reads only at startup changed (the theme and colours, the company link, the German form of address, Jitsi, captcha, meeting webhook): restart it.
- `unmanaged.event_types` lists the admin's personal event types the document does not name, before any prune.

Exit codes: `0` applied or nothing to change; `2` the document is invalid or a form refused it, nothing written; `1` an internal error, rolled back. The one exception to "rolled back": when the avatar cannot be moved into place after the commit, the message says the settings were written, the report line is still printed on stdout (so a script still sees `restart`), and the next run places the file. Otherwise errors go to stderr only.

Run it with the release directory as the working directory (a refused form renders its page from `templates/`), and with `RUST_LOG=off`, since log lines also go to stdout:

```
cd /opt/calrs/current
RUST_LOG=off calrs --data-dir /var/lib/calrs config apply --stdin --dry-run < settings.json
```

#### Relaying through a local MTA

Leaving the username empty configures an unauthenticated relay, which is how you
send through a local MTA (Postfix, OpenSMTPD, Stalwart, Mailpit). calrs then
attaches no credentials at all, rather than authenticating with an empty
username, which such a relay rejects because it advertises no AUTH mechanism.

The TLS mode prompt accepts three values:

| Mode | Transport |
|---|---|
| `starttls` (default) | Plain connection upgraded with STARTTLS, typically port 587 |
| `tls` | Implicit TLS from the first byte, typically port 465 |
| `none` | No encryption, typically port 25 |

Use `none` only for a relay on the same machine. calrs validates certificates
against a compiled-in Mozilla root bundle rather than the system trust store, so
a relay presenting a self-signed or private-CA certificate (Debian's Postfix
default, for one) cannot be reached with `starttls` or `tls` either, and `none`
over the loopback is the way through. Mail sent this way, and any credentials
sent with it, cross the connection in the clear; calrs logs a warning if you
combine `none` with a username, or point it at a host that is not loopback.

The same applies to the `CALRS_SMTP_*` environment block, where only
`CALRS_SMTP_HOST` and `CALRS_SMTP_FROM_EMAIL` are required. A full local-relay
configuration is:

```
CALRS_SMTP_HOST=localhost
CALRS_SMTP_PORT=25
CALRS_SMTP_TLS_MODE=none
CALRS_SMTP_FROM_EMAIL=noreply@example.com
```

If the environment block sets no `CALRS_SMTP_USERNAME` while the database holds
SMTP credentials, the environment still wins and calrs relays unauthenticated.
It logs a warning once at startup when that happens, because the admin form
locks itself whenever the environment governs.

The two credential variables are not interchangeable when only one is present.
`CALRS_SMTP_PASSWORD` without `CALRS_SMTP_USERNAME` is an error: the password
can never be sent, so the configuration contradicts itself. A username without a
password is accepted and only warns, since a permissive relay may still
authenticate on the username alone, though in practice it usually means the
password never reached the process.

The equivalent from the CLI, with no prompts:

```
calrs config smtp --host localhost --port 25 --tls-mode none \
  --username '' --from-email noreply@example.com --from-name calrs
```

### `calrs resource`

Probe resource calendar URLs before adding them as [shared resources](./resources.md).

```
calrs resource probe [OPTIONS]
    --url <URL>           Resource calendar URL (ICS publish feed or CalDAV collection)
    --username <USERNAME> Username for authenticated CalDAV access (password prompted)
    --write-test          Write test: PUT a temporary event, verify it exists, then delete it
```

For CalDAV URLs the probe runs the full RFC 4791 discovery fallback. The write test confirms that reservation write-back will work with the given credentials.

### `calrs user`

Manage users (admin operations).

```
calrs user create [OPTIONS]
    --email <EMAIL>    User email
    --name <NAME>      User display name
    --admin            Grant admin role
    --password-stdin   Read the password from stdin (needs --email and --name)
    --timezone <TZ>    Profile timezone, an IANA name such as Europe/Paris (default: UTC)

calrs user list

calrs user set-timezone <EMAIL> <TZ>   # Profile timezone, as in Settings → Profile

calrs user set-password <EMAIL> [--password-stdin]   # Also signs out all of its sessions
calrs user reset-mfa <EMAIL>           # Reset local MFA (authenticator app, passkeys, recovery codes) and revoke all sessions

calrs user promote <EMAIL>     Promote to admin

calrs user demote <EMAIL>      Demote to regular user

calrs user disable <EMAIL>     Disable user account

calrs user enable <EMAIL>      Enable user account
```

`set-timezone` accepts the IANA names the profile form offers (`Asia/Almaty`, `Europe/Paris`, `UTC`) and writes the same setting. It exits 0 without writing when the timezone is already set, and fails for an unknown name or user, so a script can run it on every deploy; `calrs config dump` shows each user's `timezone` to compare with first.

### `calrs serve`

Start the web server.

```
calrs serve [OPTIONS]
    --port <PORT>    Port to listen on (default: 3000)
```
