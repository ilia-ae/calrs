# Architecture

A map of calrs for new contributors and coding agents: what the main pieces
are, which file owns what, and where a change of a given kind goes. It is a
guide to the code, not a spec; when it disagrees with the code, the code wins
and this file needs a fix.

## Overview

calrs is a single Rust binary (`src/main.rs`). The same executable is a CLI
(`calrs source add`, `calrs sync`, `calrs user create`, ...) and, with
`calrs serve`, an axum web server that renders minijinja templates. All state
lives in one SQLite database (WAL mode) under the data directory. Calendars are
read and written through pluggable providers (CalDAV, Exchange EWS). Emails go
out over SMTP with `.ics` attachments, optional SMS through a pluggable gateway.
Every user-facing string goes through Fluent (`i18n/`).

```
 guest browser
      |
      v
 public pages (web/public_pages.rs)  --->  availability (web/availability.rs)
      |                                     rules + overrides + busy times
      | POST booking form                   (synced events, bookings, resources)
      v
 booking handlers (web/booking.rs)
      |  validate, re-check slot, insert row (pending or confirmed)
      +--> calendar write-back (web/calendar_push.rs, web/writeback.rs, providers/)
      +--> emails, spawned in the background (web/notify.rs -> email/)
      +--> SMS (sms/)
      v
 token pages from those emails (web/booking_actions.rs)
   approve, decline, cancel, reschedule, claim, ICS download

 background loop, every 60 s (run_reminder_loop in web/notify.rs)
   session cleanup, pending sweep (web/pending.rs), reminders,
   write-back retries (web/writeback.rs), sync of the stalest source
   (commands/sync.rs)
```

## Where things live

### Top level of `src/`

| Path | Owns |
|---|---|
| `src/main.rs` | CLI definition (clap), `serve` startup, spawning the background loop |
| `src/db.rs` | SQLite pool setup and the migration list (`migrate()`) |
| `src/models.rs` | Domain structs (mostly documentation; queries use tuples) |
| `src/auth.rs` | Passwords, sessions, OIDC, extractors (`AuthUser`, `AdminUser`, `OptionalAuthUser`), login/register handlers |
| `src/mfa.rs` | Local TOTP two-factor authentication: enrollment, login challenges, recovery codes, the admin policy, `/auth/mfa` and `/dashboard/settings/mfa` routes (tests in `src/mfa/tests.rs`); in the ilia fork also the passkey proof (`Proof`, `consume_proof`), the management actions and the last-factor rule |
| `src/webauthn/` | ilia fork: passkeys and security keys as a second factor. `mod.rs` (`Rp` from `CALRS_BASE_URL`, user handle, options JSON, form types, key names), `verify.rs` (the pure verifier: clientDataJSON, authenticator data, Ed25519/ES256/RS256, counter rule), `store.rs` (credential and ceremony SQL, `has_factor`), `flows.rs` (`/auth/mfa/passkey`, `/auth/mfa/passkey/enroll`, `/dashboard/settings/mfa/confirm`, `/dashboard/settings/mfa/passkeys`, `/static/webauthn.js`, the passkey half of the MFA pages); tests in `tests.rs` (verifier, spec vectors), `flow_tests.rs` (HTTP flows) and `flow_tests/ui.rs` (pages, script, headers), with the software authenticator `softkey.rs` |
| `src/crypto.rs` | AES-256-GCM for stored credentials, secret key management |
| `src/brand.rs` | ilia fork: the instance name `CALRS_BRAND_NAME` in place of "calrs" in tab titles, the sign-in and registration pages, the email signature and footer and the TOTP issuer: `configured()`, `register()` (template global `brand_name`), the email helpers |
| `src/credentials.rs` | ilia fork: secrets from systemd credentials (`LoadCredentialEncrypted=`): `load_into_env(NAMES)` at the start of `main`, before the runtime, sets each variable from `$CREDENTIALS_DIRECTORY/<NAME>` unless already set; names only in the log, see CLAUDE.md "Systemd credentials" |
| `src/settings.rs` | Runtime settings (base URL, private-host policy): env overrides DB |
| `src/net_guard.rs` | Outbound HTTP to configured URLs: guarded resolver that checks and pins addresses (#363), literal and redirect checks, proxy handling (tests in `src/net_guard/tests.rs`) |
| `src/trusted_proxies.rs` | Which TCP peers may set `X-Forwarded-For` (`CALRS_TRUSTED_PROXIES`), the client address and rate-limit key |
| `src/i18n.rs` | Fluent loader, language detection, the `t()` template global, i18n guard tests |
| `src/booking_time.rs` | How booking times are stored (legacy wall clock vs UTC instant) |
| `src/rrule.rs` | RRULE expansion for recurring events |
| `src/busy_only.rs` | ilia fork: busy-only calendar storage. `CALRS_BUSY_ONLY` (on by default), `keep()` for the content fields, `reduce()`/`StoredIcal` (the `raw_ical` of a recurring row keeps only its EXDATEs and RECURRENCE-IDs), `scrub_stored()` at startup (from `db::migrate`), see CLAUDE.md "Busy-only calendar storage" |
| `src/horizon.rs` | ilia fork: the fork-wide maximum horizon (`CALRS_MAX_HORIZON_DAYS`, 14, 1–31): `limit()`, `max_days_ahead()`, the sync window (`sync_window`, `SyncWindow`, `covers_horizon`, `vevent_in_window`); `None` in tests unless `test_limit`, see CLAUDE.md "Horizon and boundaries" |
| `src/utils.rs` | iCal helpers (`split_vevents`, `parse_content_line`, `extract_vevent_field`, `extract_vevent_tzid`, datetime and timezone parsing) |
| `src/tzid.rs` | Resolving an event's TZID to an IANA zone: by name (IANA, Windows, vendor prefix, display name), else by matching the resource's `VTIMEZONE`, else a display name's fixed offset (`event_timezone` at sync, `resolve_or_offset` when reading stored events) |
| `src/windows_zones.rs` | Windows timezone names (`Eastern Standard Time`) mapped to IANA ids, from CLDR's `windowsZones.xml` |
| `src/resources.rs` | Shared bookable resources: feed sync, busy intervals, pick at booking time |
| `src/booking_window.rs` | `BookingWindow`: booking horizon ∩ bookable date range, shared by web and CLI |
| `src/google_meet.rs` | Google Meet links through the Google Calendar API |
| `src/oauth2_caldav.rs` | Google OAuth2 tokens for CalDAV access |
| `src/caldav/mod.rs` | CalDAV client: discovery, fetch, sync-collection, PUT/DELETE |
| `src/caldav/partial.rs` | ilia fork: boundaries-only requests: partial retrieval bodies, etag listings, `calendar-multiget`, etag-only `sync-collection`, the "partial refused/ignored" memory, the Kerio fixture; `mock_dav.rs` (tests only) is a Kerio-style mock server that honours them and counts bytes |
| `src/ews/` | Exchange Web Services client (SOAP, autodiscover, parsing) |
| `src/providers/` | `CalendarProvider` trait (`mod.rs`), CalDAV adapter, dispatch in `src/providers/factory.rs` |
| `src/sms/` | `SmsProvider` trait, gateway adapters, phone normalisation, message bodies |
| `src/inbox/` | ilia fork: guest mail through the system. `reply.rs` (outgoing: reply tokens, Reply-To, Message-ID, the subject tag, the guest `.ics` ORGANIZER and page contact, `X-Calrs-Origin`), `config.rs` (`CALRS_INBOX_*`), `imap.rs` (our minimal IMAP client with size caps), `tls.rs` (IMAPS, webpki roots plus the CA file, or a certificate pin), `mime.rs` (mail-parser), `filter.rs` (autoreplies, bounces, lists, iTIP, loops), `matcher.rs` (token → booking, and who sent it), `clean.rs` (quotes, signatures, cap), `store.rs` (folder state, dedupe), `poller.rs` (one session, bounded searches, back-off), `organizer.rs` (bookings whose guest invites keep the host as ORGANIZER), `mod.rs` (`init_key`, `relay()`, `spawn`); `fake.rs` (tests) is an in-process IMAP server with a test CA; see CLAUDE.md "Inbox (ilia fork)" |
| `src/telegram/` | ilia fork: Telegram notifications. Bot API client and parsing (`api.rs`), token redaction (`config.rs`), long polling (`poller.rs`), commands (`commands.rs`), guest texts and host replies (`relay.rs`), the outbox (`worker.rs`), message bodies (`message.rs`), SQL (`store.rs`), deep-link tokens, usernames, `TelegramEvent` and the `notify` seam (`mod.rs`) |
| `src/email/` | Every email calrs sends (see below) |
| `src/web/` | The web server (see below) |
| `src/commands/` | One file per CLI subcommand; `src/commands/sync.rs` is also used by the web server; `sync_window.rs` (ilia fork) is its windowed full fetch (etags first), etag delta, prune and `synced_until` |
| `src/test_support.rs` | `ENV_LOCK` / `EnvGuard` for tests touching `CALRS_*` variables |

### `src/web/`

| File | Owns |
|---|---|
| `mod.rs` | `AppState`, `create_router` (every route), middleware (CSRF, CSP, embed), rate limiter, client IP, error helpers, sidebar context, booking error page helpers |
| `public_pages.rs` | Public GET pages: user and team profiles, slot pickers, booking forms, dynamic group links, legacy redirects |
| `booking.rs` | Booking form POST handlers for every URL shape, input validation, location resolution, SMS phone mode |
| `booking_actions.rs` | Token pages reached from emails: approve, decline, guest cancel, reschedule (guest and host), claim, ICS |
| `availability.rs` | Slot computation, busy times, frequency limits, user working hours, timezone helpers, horizon, troubleshoot view |
| `hosts.rs` | Who hosts a booking: host identity, collective members, write targets, reschedule and approval hosts |
| `calendar_push.rs` | Pushing and deleting booking events on host calendars and resource collections |
| `writeback.rs` | Write-back failure recording and retry (#162) |
| `pending.rs` | Pending requests: expiry deadline, sweep (expire and nudge), pending settings on event types |
| `notify.rs` | `spawn_emails` / `drain_emails`, new-booking notifications, reminders, watchers, SMS on new booking, the background loop |
| `meeting.rs` | Auto-generated meeting links (Jitsi, webhook, Google Meet) |
| `captcha.rs` | Cap captcha config and server-side verification |
| `dashboard.rs` | Dashboard overview, event types and bookings lists, host cancel/confirm, settings, avatar |
| `directory.rs` | Searchable booking directory, visibility checks, personal meeting favorites and dashboard shortcuts |
| `event_types.rs` | Event type forms (personal and team), availability overrides, form parsing helpers |
| `invites.rs` | Invite Links page, invite management, bulk invites, quick links |
| `teams.rs` | Teams listing, creation, settings, avatars |
| `sources.rs` | Calendar sources: add, edit, test, sync, remove, write-back setup, calendars to sync, Google connect |
| `admin.rs` | Admin panel: users, auth and OIDC, theme, captcha, SMTP, SMS, meeting links, resources, impersonation |
| `branding.rs` | Theme CSS, embed pages, static assets baked into the binary |
| `format.rs` | Date and time labels for pages, emails and the dashboard |
| `telegram.rs` | ilia fork: Profile & Settings → Telegram (host linking, guest field per event type), the booking form's Telegram helpers, dashboard labels |
| `debug_booking.rs` | ilia fork: the admin's debug booking page `/dashboard/admin/debug-booking` and its switch (`crate::ton::settings::DEBUG_BOOKING`: env `CALRS_DEBUG_BOOKING` > `ton_settings` row > off, resolved per `/dashboard` request by `middleware`, which answers the page 404 while off; the toggle `POST /dashboard/admin/ton/debug-booking` at the top of the TON page; the sidebar link through the `debug_booking_on()` template function; the TEST badge marks): the prefilled guest form, booking through `handle_booking_for_user`, the own-chat Telegram routing, `debug_bookings` and "cancel all my test bookings" through `cancel_booking`; template `templates/ilia/debug_booking.html`, tests in `tests/debug_booking.rs` (a child of `tests/ton.rs`) |
| `lang_switch.rs` | ilia fork: the language menu of public pages: `?lang=` sets the `calrs_lang` cookie and redirects to the same page, and the `lang_switcher()` template function (`templates/partials/lang_switcher.html`) |
| `embed_origins.rs` | ilia fork: which sites may frame an embedded booking page (`CALRS_EMBED_ORIGINS`, `AppState::embed_origins`): the `frame-ancestors` that `csp_middleware` puts in place of `'self'` for `?embed=1`, see CLAUDE.md "Embed origins (ilia fork)" |
| `messages.rs` | ilia fork: host <-> guest messages, one thread per booking: `send_host_message` (the one path for the dashboard, the agent and Telegram replies, with the limits), guest texts, the thread page `/dashboard/bookings/{id}/messages`, see CLAUDE.md "Messages with guests" |
| `error_page.rs` | ilia fork: the layer right above `lang_switch` (the innermost) giving upstream's bare error answers a real status (404, 410, 403, 400) and a styled page (`templates/ilia/error_page.html`): a bare message of `BARE`, the booking error page's title (`booking_error`, a task-local), an empty 404 to a browser; tests in `tests/error_page.rs` |
| `week_start.rs` | Calendar display preference: event override → account owner's default → site default (Monday); named weekday keys, independent of availability's `0=Sunday` storage |
| `tests/` | Web tests, one file per area (below) |

`src/web/tests/mod.rs` holds the shared fixtures (`setup_test_app`, `get_authed`,
`post_form`, `body_string`, `spawn_capturing_mta`, ...). Each area file starts
with `use super::*` and sees those fixtures and every item of `web`:
`security`, `auth_pages`, `branding`, `format`, `availability`, `resources`,
`hosts`, `event_types`, `invites`, `booking`, `public_pages`,
`booking_actions`, `dashboard`, `sources`, `admin`, `emails`,
`page_language`, `guards`, plus `booking_time`, `pending_hold`,
`writeback_failures`, `organizer` and `duplicate_event_type`.

### `src/email/`

| File | Owns |
|---|---|
| `mod.rs` | Shared data (`BookingDetails`, `CancellationDetails`, `RescheduleDetails`), language helpers, `pub use` of the files below |
| `transport.rs` | `SmtpConfig` (env and DB), `Sender` (who an email comes from), `send_email` with its timeouts |
| `render.rs` | HTML and plain-text layout, ICS generation, time conversion, `EmailParts` / `send_composed` |
| `guest.rs` | Guest emails: confirmation, pending, decline, cancellation, reminder, reschedule, pick a new time, invite |
| `host.rs` | Host, member, watcher and claimant emails |
| `ton.rs` | ilia fork: TON payment emails |
| `messages.rs` | ilia fork: a host's message to the guest (`send_guest_host_message`, Reply-To the host) |
| `tests/` | `transport.rs`, `render.rs`, `messages.rs`, shared fixtures in `tests/mod.rs` |

Call sites use `crate::email::name` whatever file `name` lives in.

### Outside `src/`

| Path | Owns |
|---|---|
| `migrations/` | SQL migrations, numbered |
| `templates/` | minijinja templates (`base.html`, `dashboard_base.html` and the pages extending them) |
| `i18n/{lang}/main.ftl` | Translations, English is the source of truth |
| `assets/` | Files embedded in the binary (logo, fonts, `embed.js`, vendored intl-tel-input, the ilia fork's `webauthn.js`) |
| `docs/src/` | User documentation (mdBook) |

## Key concepts and who owns them

- **AppState and request lifecycle** (`src/web/mod.rs`). `create_router` builds
  every route over `Arc<AppState>` (pool, templates, secret key, rate limiters,
  captcha config, CSP string, theme CSS). Middleware: `csrf_cookie_middleware`
  (double-submit cookie, checked by `verify_csrf_token` in each POST handler),
  `csp_middleware` (CSP rebuilt from captcha settings, relaxed for embeds and
  booking forms). Booking POSTs are rate limited per client IP
  (`client_ip_for_rate_limit`: the TCP peer, or, when the peer is a trusted
  proxy, `X-Forwarded-For` read right to left past trusted hops; see
  `src/trusted_proxies.rs`).
- **Auth extractors** (`src/auth.rs`). `AuthUser` redirects to login,
  `AdminUser` returns 403, `OptionalAuthUser` for public pages. Each resolves
  the viewer's language once.
- **i18n** (`src/i18n.rs`, `i18n/`). Host pages use the user's saved language;
  public pages follow the visitor's browser, `/u/` pages per the host's
  "public page language" setting (`user_page_lang` in
  `src/web/public_pages.rs`); guest emails use the language stored on the
  booking. A language picked with the switcher (ilia fork, `calrs_lang`
  cookie, `i18n::chosen_from_headers`, `src/web/lang_switch.rs`) comes
  before all of those on public pages, and before the browser only on the
  dashboard. Guard tests: `src/i18n.rs` (key coverage, plurals, keys used from
  Rust), `src/web/tests/guards.rs` (lang passed to host templates, no English
  on the booking error page, escaping).
- **Email sending** (`src/web/notify.rs`, `src/email/`). Handlers never wait on
  SMTP: they hand futures to `spawn_emails`, and `drain_emails` waits for them
  at shutdown. `Sender` in `src/email/transport.rs` decides the From and
  Reply-To (guest emails go out under the host's name, #221). Each email is
  written in its reader's language.
- **Calendar providers and write-back** (`src/providers/`, `src/caldav/`,
  `src/ews/`, `src/web/calendar_push.rs`, `src/web/writeback.rs`).
  `CalendarProvider` is the trait, `src/providers/factory.rs` builds one from a
  `caldav_sources` row. Confirmed bookings are pushed to the host calendar,
  verified, and failures are recorded and retried by the background loop.
  Sync lives in `src/commands/sync.rs` (`sync_if_stale` on page loads) and
  `src/commands/sync_health.rs` (failure tracking).
- **Availability** (`src/web/availability.rs`). `compute_slots` combines
  weekly rules, date overrides, minimum notice, horizon, frequency limits and
  busy intervals (synced events, bookings including pending holds, resources).
  Team event types pick a member (round robin) or require all (collective).
  The troubleshoot view explains a day's result.
- **Booking lifecycle**. Statuses: `pending`, `confirmed`, `cancelled`,
  `declined`. Creation: `src/web/booking.rs`. Pending hold and expiry:
  `src/web/pending.rs`. Approve/decline/cancel/reschedule/claim by token:
  `src/web/booking_actions.rs`; the same from the dashboard:
  `src/web/dashboard.rs`. Storage of times: `src/booking_time.rs`. Paths
  that rebuild `BookingDetails` from the row read what the guest typed
  through `booking_guest_input`, `booking_additional_attendees` and
  `booking_ics_sequence` (`src/web/calendar_push.rs`), not their own
  queries. A trigger stamps `bookings.first_confirmed_at` (migration 071),
  so an approval can tell a rescheduled booking from a new request: it texts
  and emails the guest "moved" and sends the attendees the moved invite
  instead of "added". Every path that ends a booking reads who holds its
  invite through `with_invite_holders`, so its attendees, and the guest of a
  declined or expired reschedule, get a CANCEL only when an invite went out. A
  reschedule bumps `bookings.ics_sequence` (migration 072), the SEQUENCE
  every invite, CANCEL and push carries; so does ending a confirmed booking,
  so its CANCEL outranks the copy it ends. ICS text is folded once per
  generator by `fold_ics` (`src/email/render.rs`).
- **Organizer snapshot** (`src/web/hosts.rs`, migration
  `070_booking_organizer.sql`). Multi-host bookings store the organizer they
  were created with, so a later push never changes the ICS organizer (Google
  refuses that).
- **Resources** (`src/resources.rs`, wiring in `src/web/calendar_push.rs` and
  `src/web/admin.rs`).
- **SMS** (`src/sms/`, sends from `src/web/notify.rs` and the booking and
  action handlers, admin card in `src/web/admin.rs`).
- **Telegram** (ilia fork, `src/telegram/`, see CLAUDE.md "Telegram
  notifications"): `crate::telegram::notify` from the same handlers, the
  poller and outbox started by `serve`, pages in `src/web/telegram.rs`.
  TON payment events reach it through `src/web/ton_pay/telegram.rs`
  (the payment page's deep link, the alert sink, the expiry event, the
  renewal's "pay again").
- **Messages with guests** (ilia fork, `src/web/messages.rs`): the
  dashboard page, the agent's `send_guest_message` (`src/web/agent/
  {tools,ops}_messages.rs`) and `src/telegram/relay.rs` all go through
  `send_host_message`; email through `src/email/messages.rs`, Telegram
  through `TelegramEvent::{HostMessage, GuestMessage}`. Guest email
  replies come in through `src/inbox/` (`poller::handle` stores them with
  `store_guest_message(.., "email", ..)` and relays them with
  `telegram::relay::relay_guest_email`).
- **Inbox** (ilia fork, `src/inbox/`, see CLAUDE.md "Inbox (ilia fork)"):
  `Sender::Host { booking, .. }` in `src/email/transport.rs` calls
  `inbox::reply::stamp`; `src/email/render.rs` asks `inbox::reply` for a
  guest `.ics` ORGANIZER; `serve` starts `inbox::spawn`; tests in
  `src/inbox/tests.rs` (protocol, TLS, the bytes measurement, `#[ignore]`)
  and `src/web/tests/inbox.rs` (the poller against a database).
- **Busy-only storage** (ilia fork, `src/busy_only.rs`): the sync upserts
  in `src/commands/sync.rs` and the feed upsert in `src/resources.rs` bind
  through it; the troubleshoot view and the CLI label an untitled event
  "Busy".
- **Horizon and boundaries** (ilia fork, `src/horizon.rs`,
  `src/caldav/partial.rs`, `src/commands/sync_window.rs`, see CLAUDE.md
  "Horizon and boundaries"): `booking_window::effective_date_bounds` caps
  every window, `sync.rs` calls into `sync_window` when there is a
  window; tests in `src/web/tests/boundaries.rs` (the bytes measurement in
  `boundaries_bytes.rs`, `#[ignore]`).
- **Optimize** (ilia fork, see CLAUDE.md "Optimize (ilia fork)"): the
  release profile in `.cargo/config.toml`, the watcher's on-demand
  heartbeat (`TonRuntime::{wake_if_idle, step_now, ready_on_demand}`,
  `watcher::IDLE_PRICED`), the Telegram menu hash (`poller::menu_hash`),
  hot-path indexes in `migrations/ilia_007_optimize.sql`, and the
  measurements in `src/web/tests/ton_perf.rs` (all `#[ignore]`).
- **Passkeys** (ilia fork, `src/webauthn/`, see CLAUDE.md "Passkeys and
  security keys (ilia fork)"): `webauthn::flows::router()` is merged next to
  `mfa::router()`; `AppState::passkeys` holds the RP read once from
  `CALRS_BASE_URL`; the MFA pages include the `templates/auth/ilia_*.html`
  partials; "has a factor" (`webauthn::store::has_factor`) is TOTP or a key
  in `auth::validate_session`, the OIDC link check, `mfa::begin_login` and the
  policy. Migration `ilia_010_webauthn.sql`.
- **Embed origins** (ilia fork, `src/web/embed_origins.rs`): `?embed=1` on a
  booking surface (`is_embedded_booking_request`) gets `frame-ancestors` from
  `AppState::embed_origins` (`CALRS_EMBED_ORIGINS`, read once; unset `*` as
  upstream, `none`, or a list of origins), applied in `csp_middleware`, which
  also turns `html_response`'s `X-Frame-Options: ALLOWALL` back into
  `SAMEORIGIN` when no other site may frame the page. `assets/embed.js` fills
  `[data-calrs-inline]` elements on load. Tests in
  `src/web/tests/embed_origins.rs`.
- **TON price by demand and TON settings** (ilia fork, docs/src/ton.md):
  the pure rules in `src/ton/ladder.rs` (the ladder, `price(n)`,
  `accepts`), `src/ton/config.rs` (`Tuning`, `TonConfig::resolve`) and
  `src/ton/settings.rs` (the keys the page and apply may store, env > DB >
  default, the one parser `to_env`); the demand count and the stored rows
  in `src/web/ton_pay/store.rs` (`demand_n`, `load_settings`); the quote
  and the recount inside the booking transaction in
  `src/web/ton_pay/invoice.rs` (`prepare`, `open`) and `flow.rs`; the
  public price line in `pricing.rs`; writes and the audit trail in
  `src/web/ton_pay/settings.rs`; the admin page `/dashboard/admin/ton` in
  `src/web/ton_pay/settings_page.rs` (`templates/ton_settings.html`); the
  hot apply in `TonSlot::apply_settings` (a retuned `TonRuntime`, the
  watcher's supervisor restarts on `TonSlot::generation`); the alert email
  through `alerts::alert_email`; `config apply`'s `site.ton` in
  `src/web/apply/ton.rs`. Migrations `ilia_011_ton_ladder.sql` and
  `ilia_012_ton_settings_audit.sql`; tests in
  `src/web/tests/{ton_ladder,ton_settings,apply_ton}.rs`.
- **Captcha** (`src/web/captcha.rs`, CSP in `src/web/mod.rs`, widget in
  `templates/book.html`).
- **Migrations** (`migrations/`, `src/db.rs`). A new file must also be added
  to the list in `migrate()` in `src/db.rs`, or it never runs on existing
  installs.
- **Background loop** (`run_reminder_loop` in `src/web/notify.rs`, spawned by
  `src/main.rs`): session cleanup, pending sweep, reminders, write-back
  retries, sync of the stalest source.

## Where to add things

- **A route or page.** Add the handler to the file that owns the area (table
  above), register it in `create_router` in `src/web/mod.rs`, add the template
  under `templates/`. Host pages pass `lang => auth_user.lang` and
  `sidebar => sidebar_context(...)`. Every POST handler checks CSRF. Tests go
  in the matching `src/web/tests/<area>.rs`.
- **An email.** Compose and send it in `src/email/guest.rs` or
  `src/email/host.rs` (guest-facing senders use `Sender::Host`; a guard test
  in `src/email/tests/transport.rs` enforces it). Call it from a handler
  through `spawn_emails`. Tests in `src/email/tests/messages.rs`.
- **A translatable string.** Add the key to `i18n/en/main.ftl` and every
  other locale, on the `i18n` branch. Use `{{ t("key") }}` in templates or
  `crate::i18n::translate` in Rust. Run `cargo test i18n::`.
- **A migration.** `migrations/NNN_description.sql` plus its entry in
  `migrate()` in `src/db.rs`.
- **A calendar provider.** Implement `CalendarProvider` (`src/providers/mod.rs`)
  and add it to `build_provider` in `src/providers/factory.rs`.
- **An SMS gateway.** One adapter file in `src/sms/` implementing
  `SmsProvider`, and one `ProviderSpec` entry in `src/sms/factory.rs`.
- **A CLI subcommand.** `src/commands/<name>.rs`, `pub mod` in
  `src/commands/mod.rs`, a variant in the `Commands` enum and a match arm in
  `src/main.rs`.

Tests sit next to their area: `#[cfg(test)] mod tests` at the bottom of most
files, `src/web/tests/` and `src/email/tests/` for the two large modules.
Tests that set `CALRS_*` environment variables take
`crate::test_support::ENV_LOCK`.
