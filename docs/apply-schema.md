# `calrs config apply`: schema v1 (frozen)

ilia fork. This file freezes the document, the report and the module API that the
section steps implement against. Code: `src/web/apply/`. Tests: `src/web/tests/apply_*.rs`.
A complete example document, checked by a test: [`apply-example.json`](apply-example.json).

## Command

```
calrs --data-dir D config apply --stdin [--dry-run]   # document on stdin
calrs --data-dir D config apply --export [--user E]    # stored state, same schema
```

- **stdout** gets exactly one JSON line: the report for `--stdin`, the document for `--export`.
- **On error** nothing goes to stdout, except after a commit (below). One line goes to stderr (`calrs config apply: invalid <path>: …`, `refused <path>: …; nothing was written`, `internal error: …; nothing was written`, or `internal error after the commit: …; the settings were written, run again to put the file in place`).
- **Exit codes:**
  - `0` means applied, or nothing to change.
  - `2` means the document is invalid or a handler refused it. Nothing was written.
  - `1` means an internal error. Everything was rolled back, except in one case: the transaction committed and then a staged file (the avatar) could not be moved into place (`ApplyError::AfterCommit`). The settings are then written and the message says so. The report line is still printed on stdout, as on success, so the caller sees what was written and whether `restart` is needed: the next run finds those values stored and could not ask for the restart any more. That run finds the file missing (`"(file missing)"`) and places it.
  - There is no exit code 3: the run is one transaction (see "Transaction" below).
- **`--dry-run`** runs every check and every handler on a writable connection, then always rolls back. Its report is the one a real run would print, with `"dry_run": true`.
- **`--export`** opens the database `mode=ro` and writes nothing.
  - Without `--user` it exports the only enabled admin. If there are several admins, it exits 2.
  - Caveat: `main` runs `db::migrate`, the key creation and `migrate_passwords` on its own read-write pool before any command runs. A binary newer than the database therefore still migrates it, even for `--export`.
- **Input:** at most 1 MiB of UTF-8 (`DOCUMENT_MAX_BYTES`).
- **Templates:** a refused form renders its page from the relative `templates/` directory. Run the command with the release directory as cwd.
- **Logging (open item for the lead, `src/main.rs`):** `tracing` writes to **stdout**, and handlers log at `info` (for example `admin: theme updated`).
  - Until `main` sends logs to stderr, run with `RUST_LOG=off` so that stdout holds only the JSON line.
  - The fix is one line, `.with_writer(std::io::stderr)` on the subscriber in `src/main.rs`. That file is not in A1's set.

## Document

- `format` must be `"calrs-apply-v1"`. It is checked before anything else, so a newer document is named as one. `user` is required.
- **Every object** is `deny_unknown_fields`, at every level. An unknown key exits 2 before anything is read or written.
- **How keys are read:**
  - A key that is left out means leave as is.
  - `null` is accepted only where marked *nullable*, and means clear or inherit. Anywhere else, `null` exits 2.
  - `""` is never a value. Where text may be emptied, use `null` (`check_opt_text`).
- **Keywords** are closed sets, spelled exactly as listed below: lowercase, as in the database and the forms.

```jsonc
{
  "format": "calrs-apply-v1",
  "user": "mail@ilia.ae",                 // email of an enabled admin, canonical (no whitespace)

  "profile": {                            // Settings → Profile (settings_save); A2
    "name": "Ilia Arestov",               // 1..255 bytes
    "username": "ilia",                   // canonical: [a-z0-9-], 2..100, unique
    "title": "Fractional CTO",            // nullable
    "bio": "Markdown…",                   // nullable; links http(s), mailto or relative only
    "booking_email": null,                // nullable; ≤255, contains @, domain has a dot
    "timezone": "Asia/Almaty",            // IANA, as chrono_tz spells it
    "language": "ru",                     // nullable (null = browser); a code i18n::is_supported accepts
    "public_page_language": "visitor",    // visitor | host
    "week_start": null,                   // nullable (null = inherit the site's); mon..sun
    "allow_dynamic_group": false,
    "lend_resource_write": false,
    "weekly_hours": [                     // default availability; the whole week
      {"day": "mon", "windows": [{"start": "10:00", "end": "19:00"}]}
    ],
    "avatar": {"file": "/var/lib/calrs/managed/avatar", "sha256": "<64 lowercase hex>"}  // nullable: null removes
  },

  "site": {                               // Admin (auth_config); A3
    "theme": "custom",                    // default|nord|dracula|gruvbox|solarized|tokyo-night|vates|custom
    "custom": {"accent": "#313131", "accent_hover": "#111111",
               "bg": "#f6f6f6", "surface": "#ffffff", "text": "#313131"},  // all five, "#rrggbb"
    "week_start": "mon",                  // mon..sun
    "german_formal": true,                // true = "Sie"
    "registration": false,
    "allowed_email_domains": "ilia.ae",   // nullable; canonical comma list, lowercase
    "mfa_policy": "optional",             // required | optional
    "company_link": "https://ilia.ae",    // nullable; is_safe_company_link
    "jitsi": {"base_url": "https://meet.jit.si", "pattern": null, "display_name": "Jitsi Meet"},
                                          // or "off"; when an object, all three keys are required
                                          // (pattern and display_name may be null)
    "oidc": "off",                        // v1: only "off"
    "google_oauth2": "off",
    "captcha": "off",
    "meeting_webhook": "off",
    "sms_allow_all_users": false,
    "ton": {                              // ilia fork; left out = no change (see "TON settings" below)
      "window_min": 30, "grace_min": 15,  // whole numbers, or null (the default, still owned)
      "underpay_tolerance_ton": "0",      // canonical strings: TON, USD, %, the ladder, the email
      "rate_min_usd": "0.3", "rate_max_usd": "5",
      "alert_email": "mail@ilia.ae",
      "ladder": "free,20,40,60,80,100,200,300,400,500",
      "free_per_guest_days": 30
    }
  },

  "sources": [                            // A4; each must already exist, matched on url + username
    {"url": "https://…", "username": "aa@10112.in",
     "write_calendar": "Calendar",        // nullable (null = no write-back); a calendar display name
     "sync_calendars": ["Calendar"]}      // display names synced, every other one not; NOT nullable
  ],

  "event_types": {                        // the admin's personal event types; A4
    "prune": "none",                      // none | disable | delete; left out = none
    "items": [
      {
        "slug": "15min",                  // required; the key
        "title": "…",                     // 1..255
        "description": "…",               // nullable; Markdown, MCP's check_description
        "duration_min": 15,               // 1..1440
        "slot_interval_min": null,        // nullable
        "buffer_before": 0, "buffer_after": 15, "min_notice_min": 240,
        "requires_confirmation": false,
        "pending_blocks_slot": true,
        "pending_expiry_hours": 24,       // nullable (null = never expire)
        "visibility": "public",           // public | internal | private
        "location_type": "jitsi_auto",    // link|phone|in_person|custom|jitsi_auto|webhook_auto|google_meet
        "location_value": null,           // nullable
        "meeting_pattern_override": null, // nullable
        "timezone": null,                 // nullable (null = follow the profile)
        "weekly_hours": null,             // nullable (null = the profile's default hours); as profile.weekly_hours
        "booking_horizon_days": 14,       // nullable
        "bookable_from": null, "bookable_until": null,   // nullable, YYYY-MM-DD, from ≤ until
        "reminder_minutes": 60,           // nullable
        "max_additional_guests": 1,
        "first_slot_only": false,
        "default_calendar_view": "month", // month | week | column
        "week_start": null,               // nullable; mon..sun
        "cancel_notice_min": 120, "reschedule_notice_min": 120,   // nullable
        "sms_phone_mode": "off",          // off | optional | required
        "frequency_limits": [{"count": 4, "period": "day"}],      // whole list; [] = none; day|week|month|year
        "ton_price": null,                // nullable (null = free); or the whole price, see "TON price" below:
                                          // {"usd": "500.00", "window_min": null, "enabled": true}
        "telegram_guest_mode": "optional",// off | optional | required
        "enabled": true
      }
    ]
  }
}
```

- **Event type items:**
  - Field names, values and ranges are the MCP `EventTypePatch` ones (`src/web/agent/dto.rs`).
  - Three things differ: `timezone` is nullable here; `weekly_hours`, `frequency_limits`, `ton_price`, `telegram_guest_mode` and `enabled` are extra keys; and there is no `version`.
  - A key that is left out keeps the stored value. On creation, a missing key takes the new event type form's default.
  - These are not in v1 and are left as they are: busy calendars, resources, watchers, team fields and date overrides.
- **TON price (`ton_price`, added after v1 froze; same format string):** the price section of the event type form (`ton_prices`).
  - `null` is free: no price, a stored one is removed (the section unchecked and blank).
  - Otherwise an object, always the whole price: exactly one of `ton` (the invoice amount itself, no exchange rate), `usd` (converted at the rate of the moment the guest books, inside the server's corridor `CALRS_TON_RATE_MIN_USD`..`MAX_USD`) and `"ladder": true` (ilia fork: the site's demand ladder, `CALRS_TON_LADDER`, in TON; only `true` is accepted, `null` and `false` are refused); `window_min`, the minutes the guest has to pay (10..240, `null` = the server's `CALRS_TON_WINDOW_MIN`), left out = `null`; `enabled`, left out = `true`, `false` keeps the price switched off (the type books free, the section's unchecked box with an amount).
  - The amount is a string spelled as the form shows a stored price: `"500.00"`, `"100.00"`, `"12.35"`. `"500"` is refused with `spell it "500.00"`, so a document and an export compare as text.
  - Checks: the form's own (`ton_pay::validate_price_form`, its English message): USD 1..10000 with at most two decimals, TON 1..100000 in steps of 0.01, the window 10..240. What is stored is what the form stores for the same fields (`ton_pay::price_write`, `ton_pay::write_price`, `store::save_price`), written after the form, as `timezone` is: the form's own save opens a transaction, which SQLite refuses inside the run's.
  - Like the form, a price switched on makes a type with no Telegram mode chosen ask for the guest's Telegram (`required`). A `telegram_guest_mode` in the item is written after the price and wins. Without one, the mode the price set is reported as `event_types[<slug>].telegram_guest_mode`.
  - Nothing is refused because payments are off on the server: the form does not refuse either (it notes that a paid type cannot be booked until they are), and `config apply` does not run with `calrs serve`'s environment. Open invoices keep their terms; the new price applies to bookings made from then on.
  - An older calrs refuses the key (`deny_unknown_fields`, exit 2, nothing written). The marker of a calrs that knows it is the literal `TON_PRICE_SHAPE` in `src/web/apply/event_types.rs`: `ton_price: null or {ton|usd, window_min, enabled}`, in the source and in the binary.
  - `"ladder": true` came later still: an older calrs refuses it as an unknown key (exit 2, nothing written). Its marker is the literal `TON_PRICE_LADDER_SHAPE` in the same file, `ton_price: null or {ton|usd, window_min, enabled} or {ladder: true, window_min, enabled}`; `TON_PRICE_SHAPE` is kept as it was. An export prints a ladder price as `{"ladder": true, "window_min": …, "enabled": …}`, which applies back with no change.
- **TON settings (`site.ton`, ilia fork, added after v1 froze; same format string):** the tuning keys of the admin's TON page (`/dashboard/admin/ton`), stored in `ton_settings` (`src/web/apply/ton.rs`; the keys are `crate::ton::settings::SPECS`).
  - Keys: `window_min`, `grace_min`, `fuse_hours`, `renew_min_lead_hours`, `renew_window_min`, `max_renewals`, `rate_max_age_min`, `free_per_guest_days` (JSON integers); `underpay_tolerance_ton` (TON), `rate_min_usd`, `rate_max_usd` (USD per TON), `rate_max_spread_pct`, `rate_max_jump_pct` (%), `alert_email`, `ladder` (`"free,20,40"`) (strings). Any key may be `null`: the default, still owned by apply. Any other key (`network`, `address`, the indexer, the API keys, the limits) is refused as unknown: they belong to the server's environment.
  - Values go through the parser the environment and the page use (`settings::to_env`), with the environment's ranges and messages, and must be canonical: `"0.01"`, not `"0.010"`; `"0.3"`, not `"0.30"`; `"free,20"`, not `"free, 20"`. Then the whole stored result is checked (`settings::validate`, strict): half a corridor is refused under `site.ton.rate_min_usd`.
  - **A missing `ton` changes nothing.** Given, each of its keys is written with the owner `apply` (read-only on the page). The one exception to "a key left out means leave as is": a key apply owned before and the present section leaves out goes back to the page, keeping its value (a `null` row is removed), reported as `site.ton.<key>.owner` (`"apply"` to `"panel"` or `null`).
  - **`config apply` cannot see `calrs serve`'s environment** (it is its own process). It validates as if no `CALRS_TON_*` were set, and at run time a variable set in the server's environment still wins over what apply stored (env > DB > default). Keep a key either in the environment or in the document, not both.
  - Report: `site.ton.<key>` (before and after in the document's shape; `null` for the default or no row) and `site.ton.<key>.owner` (`"apply"`, `"panel"` or `null`). A value change sets `restart` (`calrs serve` resolves the settings at startup); an owner change alone does not. Each change gets a row in `ton_settings_audit` (source `apply`, the admin of `user`): the durable trail. A `calrs::audit` log event follows each change only once the run has committed (never on `--dry-run` or a rollback); the ilia-booking role runs `config apply` with `RUST_LOG=off`, so its runs log none.
  - An older calrs refuses `ton` as an unknown key of `site` (exit 2, nothing written). The marker of a calrs that knows it is the literal `TON_SETTINGS_SHAPE` in `src/web/apply/ton.rs`, `site.ton: {window_min .. free_per_guest_days}, each a value or null (apply-owned)`, in the source and in the binary.
  - Export: the keys apply owns, `null` for a default it owns; no `ton` key when it owns none, so applying an export is no change. A stored value the document cannot state (a hand edit) is left out.
- **Not in the document:** `base_url` and `allow_private_hosts` belong to the environment, so they are unknown keys. There are no secrets in v1.

### Canonical values

Where a handler quietly normalises or drops input, the section's `check` refuses any input that isn't already in final form. It exits 2 with the document path, so desired equals stored after a write and a second run reports nothing.

The rule covers:
- the username (lowercase, filtered);
- a timezone that doesn't parse (the handler keeps the old one silently);
- an unsupported language (the handler stores NULL);
- a colour that isn't `#` plus 6 hex digits;
- whitespace that `trim` would drop;
- weekly windows that are unsorted, overlapping or more than 10 a day;
- a slug the handler would rewrite;
- `custom` given with a theme other than `custom`;
- a TON price amount not spelled as the form shows it (`"500"` for `"500.00"`).

The checks reuse the existing validators: `check_text`/`check_opt_text`/`check_timezone` (below), `i18n::is_supported`, `admin::is_safe_company_link`, `validate_event_type_form`, `parse_bookable_range`, and from `ops_write` `check_title`, `check_description`, `weekly_rules` and `schedule_string`.

## Report

```json
{"format":"calrs-apply-v1","dry_run":false,
 "changed":[{"path":"site.theme","before":"default","after":"custom"}],
 "restart":true,"unmanaged":{"event_types":[]}}
```

**Paths:**
- `profile.<key>` and `site.<key>`;
- `site.custom.<colour>` and `site.jitsi` (one path for the whole value: `"off"` or the object);
- a feature switched `off`: one path per column, `site.<feature>.<column>`, for example `site.oidc.enabled`, `site.oidc.client_secret`, `site.captcha.instance_url`, `site.meeting_webhook.secret`;
- `sources[<index in the document>].write_calendar` (the calendar's name or null) and `sources[<index>].sync_calendars` (the sorted list of names synced, before and after). A calendar with an empty display name is named by its `href`;
- `event_types[<slug>].<key>`;
- `event_types[<slug>]` with `before: null, after: "created"` for a created event type, or `before: "present", after: null` for a deleted one. Disabling by prune is `event_types[<slug>].enabled`, true to false.

**Values:**
- `before` and `after` are JSON in the document's own shape. `weekly_hours` and `frequency_limits` appear as whole lists, `ton_price` as the whole object with every key (or `null`).
- A cleared secret column shows `"(secret set)"` changing to `null` (`SECRET_SET`).
- `profile.avatar` shows `"sha256:<hex>"`, or `null`.

**`restart`** is true when a value `calrs serve` caches at startup changed: the theme and `custom.*`, `company_link`, `german_formal`, `captcha`, `jitsi`, `meeting_webhook` and (ilia fork) a `site.ton` value. Record these with `Ctx::record_restart`.

**`unmanaged.event_types`** lists the slugs of the admin's personal event types that the document leaves out, as they were before pruning, sorted by slug.

**Order of `changed`:** profile, site, sources, event types, as the sections run; within a section, the order of the keys in the schema above; event types item by item in the document's order (a created type is one entry), then the pruned ones by slug.

## Transaction: one transaction (decided; the §3.3 fallback is not needed)

**Mechanism:**
- `open_pool` opens a pool of exactly one connection, with `idle_timeout` and `max_lifetime` set to `None` so the connection is never reaped mid-run, and a 10 s acquire timeout.
- `Txn::begin` issues `BEGIN IMMEDIATE`, then `CREATE TEMP TABLE calrs_apply_txn` inside the transaction.
- `AppState` is built on that pool (`build_state`), so every statement of every handler runs on that connection, inside the transaction.
- `Txn::check` selects from the temp table. It fails if the transaction was rolled back, including SQLite's own rollback after a disk-full or I/O error, and if the connection was replaced. It runs after every section and before the commit.

**Evidence**, from `src/web/tests/apply_txn.rs` (all pass):
- **sqlx-core 0.7.4 code:** `PoolConnection::return_to_pool` only checks lifetime, runs `after_release` (none is set) and pings. It never rolls back or resets a connection. sqlx-sqlite tracks only transactions it began itself (`transaction_depth`), so a raw `BEGIN` stays open between statements.
- **`a_refusal_in_the_last_section_leaves_the_database_unchanged`:**
  - Inside one `Txn` it posts real forms: `settings_save` (profile and default hours), `admin_update_accent` (custom theme), `create_event_type` and a refused `update_event_type` (duration 0, refused with the form's own message).
  - While the transaction is open, the run's connection sees every write, and a second connection (as `calrs serve` would be) sees none.
  - After `ROLLBACK`, a snapshot of every row of every table is identical to the one taken before.
- **`a_commit_keeps_every_section`:** the same posts, committed, are all visible to the other connection.
- **`a_nested_begin_fails_fast_and_keeps_the_run_transaction`:** `pool.begin()` inside the run fails at once ("cannot start a transaction within a transaction"). It doesn't deadlock, and the run's transaction stays usable.
- **`a_lost_transaction_is_detected`:** after a stray `ROLLBACK`, `Txn::check` and `commit` fail.
- **`another_writer_waits_for_the_run`:** a write from another pool waits on `BEGIN IMMEDIATE` and lands after the commit. No update is lost.
- **`a_read_only_pool_refuses_writes`:** this is why `--dry-run` uses a writable connection plus rollback.
- **`a_pool_of_several_connections_is_refused`:** `Txn::begin` refuses any pool that is not single-connection.

**Consequences for the sections:**
- **Never hold a `PoolConnection` or a `Transaction` of `ctx.pool()` across a handler call.** The handler would wait 10 s for the connection and fail.
- **Never call `pool.begin()`.** SQLite refuses the nested `BEGIN`. This is why A3 extracts the MFA policy UPDATE and session DELETE from `mfa.rs` as plain statements on `&SqlitePool`, without `begin()`.
- **Never call a handler that begins a transaction.** That includes `duplicate_event_type` (`duplicate_event_type_once`) and every handler in `mfa.rs`.
- **No network I/O.** The write lock is held for the whole run.
- **Dry-run is exact.** It equals the real run up to the commit, and rolling back undoes every handler write.

**What runs where:**

| | Section writes | Read-back verify | Unchanged sections |
|---|---|---|---|
| Real run | inside the transaction | inside the transaction | skip their handler: no post when nothing differs |
| Dry run | the same, rolled back | the same | the same |

- **Skip unchanged sections.** A section posts its form only when the stored value differs from the desired one. This avoids `updated_at` churn and side effects; for example, `mfa_policy: required` deletes sessions.
- **Files** are staged in `Ctx::staged` (`files::Staged`) and moved into place by `files::finish` only after the commit. They are removed on rollback or dry run.

## Pipeline (`apply::run`)

1. `parse`: the format first, then strict serde.
2. `build_state` on the one-connection pool, then `Txn::begin`, then `admin(pool, Some(&doc.user))` (an enabled admin).
3. For each section present, call its `check`, in order: profile, site, sources, event_types. Checks read only.
4. For each section present, call its `apply` in the same order, then `Txn::check`. After profile, also call `ctx.reload_me()`. Event types go last because their form reads the profile (timezone, default hours) and the site (Jitsi for `jitsi_auto`).
5. Call each section's `verify`. A difference is `ApplyError::stored_differs`.
6. `COMMIT`, or `ROLLBACK` when `dry_run` or on any error. Then `files::finish(staged, committed)`: after a commit it moves each staged file into place (a failure is `AfterCommit`, exit 1, and `run` puts the run's report into it for `cli` to print); otherwise it only removes the temporary files.

## Module API (frozen; section steps implement these, mod.rs is not edited by them)

Every section module (`profile.rs` A2, `site.rs` A3, `sources.rs` and `event_types.rs` A4) has exactly these four functions. `T` is `Profile`, `Site`, `[Source]` or `EventTypes`, and `E` is `Profile`, `Site`, `Vec<Source>` or `EventTypes`:

```rust
pub(super) async fn check(ctx: &Ctx, section: &T) -> Result<(), ApplyError>;      // reads only
pub(super) async fn apply(ctx: &mut Ctx, section: &T) -> Result<(), ApplyError>;  // posts, ctx.record*
pub(super) async fn verify(ctx: &Ctx, section: &T) -> Result<(), ApplyError>;     // read back == desired
pub(super) async fn export(ctx: &Ctx) -> Result<Option<E>, ApplyError>;          // stored state, reads only
```

All four sections are implemented. The other files:
- `files.rs` (A2) owns `pub(crate) struct Staged` and `pub(super) fn finish(staged: Vec<Staged>, committed: bool) -> Result<(), ApplyError>`.
- `export.rs` (A5) assembles the sections' `export`s.

Tests:
- `apply_txn.rs`: the transaction (above).
- `apply_profile.rs`, `apply_site.rs`, `apply_event_types.rs` (event types and sources): each section's writes, second runs, dry runs, refusals with the database snapshot unchanged, and exports.
- `apply_roundtrip.rs`: book.ilia.ae's whole document (dry run equal to the run, a second run with no change, the export equal to the document and applying as no change); the export of a fresh and of a hand-edited database applying as no change; the example document; the parity table; a file failing after the commit; the one-line report; the strict parser.

**Parity table** (`apply_roundtrip.rs`, `every_value_apply_refuses_…`). Each value apply refuses is also posted to the web form itself, and the form's outcome is asserted:
- **refuses** it (for example `javascript:` as company link, a duration of 0, a location that needs a value and has none);
- **rewrites** it, so a run that posted it would store another value and report it again on every run (an uppercase username, `Asia/Almty`, language `xx`, theme `Nord`, a padded title, `""` for a nullable text, a slug with a space or a capital, a reminder of 0, empty weekly hours);
- or **keeps** it, where apply is stricter on purpose, with the reason next to the case: a colour that is not `#rrggbb` (the theme CSS is then not written at all), a bio link the public page drops, overlapping windows and the MCP ranges (so the two writers agree), a company link with no host, a non-canonical domain list, a Jitsi base URL that is not http(s).

Profile cases beyond the table are in `apply_profile.rs`.

Helpers in `src/web/apply/mod.rs`, all `pub(crate)`:

```rust
// Context
impl Ctx {
    fn new(state: Arc<AppState>, me: User, dry_run: bool) -> Ctx;
    fn pool(&self) -> &SqlitePool;                // the one connection, inside the transaction
    fn auth_user(&self) -> AuthUser;              // ctx.me, lang "en", no impersonation
    fn admin_user(&self) -> AdminUser;            // ctx.me, lang "en"
    async fn reload_me(&mut self) -> Result<(), ApplyError>;
    fn record(&mut self, path: impl Into<String>, before: impl Serialize, after: impl Serialize) -> bool;
    fn record_restart(&mut self, path: impl Into<String>, before: impl Serialize, after: impl Serialize) -> bool;
    fn changes(&self) -> &[Change];
}
// fields: ctx.state, ctx.me, ctx.dry_run, ctx.unmanaged_event_types: Vec<String>, ctx.staged: Vec<files::Staged>

// Posting to a handler
async fn form<T: DeserializeOwned>(fields: &[(&str, String)]) -> Result<(HeaderMap, Form<T>), ApplyError>;
async fn answered(path: &str, response: Response) -> Result<(), ApplyError>;
//   Ok for a redirect, or a 2xx page without class="error" (settings_save answers 200 with its page);
//   otherwise Refused with the page's error text. Always read back afterwards anyway.
// Example:
//   let (headers, form) = apply::form(&fields).await?;
//   let response = crate::web::dashboard::settings_save(State(ctx.state.clone()), ctx.auth_user(), headers, form)
//       .await.into_response();
//   apply::answered("profile", response).await?;

// Canonical input
fn check_text(path: &str, value: &str, max_bytes: Option<usize>) -> Result<(), ApplyError>;
fn check_opt_text(path: &str, value: Option<&str>, max_bytes: Option<usize>) -> Result<(), ApplyError>;
fn check_timezone(path: &str, value: &str) -> Result<chrono_tz::Tz, ApplyError>;

// Errors (exit 2, 2, 1)
ApplyError::invalid(path, message) / ApplyError::refused(path, message) / ApplyError::internal(context, error)
ApplyError::after_commit(context, error)                 // files::finish after a commit; exit 1; run() adds the report
ApplyError::report(&self) -> Option<&Report>            // that report (AfterCommit only); cli prints it on stdout
ApplyError::from_tool(path, &ToolError)                 // an ops_write check's error
ApplyError::stored_differs(path, &wanted: &Value, &stored: &Value)
impl From<sqlx::Error> for ApplyError                    // `?` on queries → Internal

// Types
Day::{key, from_key, to_mcp, day_of_week}; DayHours::to_mcp(&[DayHours]) -> Vec<dto::DayHoursIn>
// every keyword enum (Theme, MfaPolicy, Off, Prune, GuestMode, Visibility, LocationType,
// CalendarView, PhoneMode, Period, PublicPageLanguage, Day) has key() -> &'static str and from_key(&str) -> Option<Self>
const FORMAT, SECRET_SET, DOCUMENT_MAX_BYTES
```

The pipeline and the CLI are `parse`, `admin`, `open_pool`, `Txn::{begin, check, commit, rollback}`, `run`, `export_document`, `read_document` and `cli`.

Shared test fixtures in `src/web/tests/apply_support.rs`:
- `ApplyEnv::new()` sets up a temp data dir with a migrated `calrs.db`, the admin `ADMIN_EMAIL` and an account.
- Fields: `env.pool`, a 5-connection pool outside the run; `env.admin_id`; `env.account_id`.
- Methods: `env.apply(json, dry_run)`, `env.export()`, `env.snapshot()`, `env.single(read_only)`.
- Free functions: `db_snapshot(pool)` (every row of every table) and `empty_doc()`.

Each section step adds its own tests in its own file. If one needs a change in `mod.rs`, `apply_support.rs` or `tests/mod.rs`, it asks the lead and does not edit them itself.
