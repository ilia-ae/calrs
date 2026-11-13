//! ilia fork: `config apply` across the sections (docs/apply-schema.md):
//! book.ilia.ae's whole document (design §5, MFA kept optional) applied,
//! re-applied and exported; `--export` applying as no change on a fresh and
//! on a hand-edited database; the parity table (every value apply refuses,
//! posted to the web form itself); a file failing after the commit; the
//! one-line output. Shared fixtures: `apply_support`. Each section's own
//! cases are in `apply_profile.rs`, `apply_site.rs` and
//! `apply_event_types.rs`.

use super::apply_support::*;
use super::*;
use crate::web::apply::{self, ApplyError, Ctx, Document, Report};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};

const JPEG: &[u8] = b"\xFF\xD8\xFF\xE0\x00\x10JFIF\x00 the owner's photo";

/// A document of `sections` (an object of section keys) for the admin.
fn doc(sections: Value) -> String {
    let mut doc = json!({"format": apply::FORMAT, "user": ADMIN_EMAIL});
    for (key, value) in sections.as_object().unwrap() {
        doc[key] = value.clone();
    }
    doc.to_string()
}

fn to_json(document: &Document) -> String {
    serde_json::to_string(document).unwrap()
}

/// `bytes` in the env's `managed/` directory, as the role puts the avatar
/// there: its path and SHA-256.
fn image(env: &ApplyEnv, bytes: &[u8]) -> (String, String) {
    let dir = env.dir.join("managed");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("avatar");
    std::fs::write(&path, bytes).unwrap();
    let sha = hex::encode(<sha2::Sha256 as sha2::Digest>::digest(bytes));
    (path.to_str().unwrap().to_string(), sha)
}

/// The Kerio source of the admin, as `source add` and a first sync leave
/// it: the main calendar, a synced "Holidays", an unsynced old one.
async fn seed_kerio(env: &ApplyEnv) {
    sqlx::query(
        "INSERT INTO caldav_sources (id, account_id, name, url, username)
         VALUES ('kerio', ?, 'Kerio', 'https://mail.10112.in/caldav/', 'aa@10112.in')",
    )
    .bind(&env.account_id)
    .execute(&env.pool)
    .await
    .unwrap();
    for (id, name, href, synced) in [
        (
            "cal-main",
            "Calendar",
            "/caldav/users/10112.in/aa/Calendar/",
            1,
        ),
        (
            "cal-hol",
            "Holidays",
            "/caldav/users/10112.in/aa/Holidays/",
            1,
        ),
        (
            "cal-old",
            "Archive",
            "/caldav/users/10112.in/aa/Archive/",
            0,
        ),
    ] {
        sqlx::query(
            "INSERT INTO calendars (id, source_id, href, display_name, sync_enabled)
             VALUES (?, 'kerio', ?, ?, ?)",
        )
        .bind(id)
        .bind(href)
        .bind(name)
        .bind(synced)
        .execute(&env.pool)
        .await
        .unwrap();
    }
}

/// book.ilia.ae's document (settings design §5, with the lead's choices:
/// MFA `optional` for now), every key of every section given.
fn production(avatar_file: &str, avatar_sha: &str) -> Value {
    let mut doc = production_v1(avatar_file, avatar_sha);
    // ilia fork keys added after v1 (outside the literal: `json!` would hit
    // the recursion limit).
    doc["sources"][0]["name"] = json!("Kerio");
    for item in doc["event_types"]["items"].as_array_mut().unwrap() {
        item["busy_calendars"] = Value::Null;
        item["overrides"] = json!([]);
    }
    doc
}

fn production_v1(avatar_file: &str, avatar_sha: &str) -> Value {
    let weekday = |day: &str| json!({"day": day, "windows": [{"start": "10:00", "end": "19:00"}]});
    json!({
        "profile": {
            "name": "Ilia Arestov",
            "username": "ilia",
            "title": "Fractional CTO",
            "bio": "Fractional CTO based in Dubai. 15 years in IT, 1000+ projects in 38 countries, \
                    6 active patents in information security and big data. IT consulting, \
                    information security, business automation, data analytics and risk \
                    management for companies in MENA and the CIS.",
            "booking_email": null,
            "timezone": "Asia/Almaty",
            "language": "ru",
            "public_page_language": "visitor",
            "week_start": null,
            "allow_dynamic_group": false,
            "lend_resource_write": false,
            "weekly_hours": (["mon", "tue", "wed", "thu", "fri"].map(weekday)),
            "avatar": {"file": avatar_file, "sha256": avatar_sha}
        },
        "site": {
            "theme": "custom",
            "custom": {"accent": "#313131", "accent_hover": "#111111", "bg": "#f6f6f6",
                       "surface": "#ffffff", "text": "#313131"},
            "week_start": "mon",
            "german_formal": true,
            "registration": false,
            "allowed_email_domains": "ilia.ae",
            "mfa_policy": "optional",
            "company_link": "https://ilia.ae",
            "jitsi": {"base_url": "https://meet.jit.si", "pattern": null, "display_name": "Jitsi Meet"},
            "oidc": "off",
            "google_oauth2": "off",
            "captcha": "off",
            "meeting_webhook": "off",
            "sms": "off",
            "sms_allow_all_users": false
        },
        "sources": [{
            "url": "https://mail.10112.in/caldav/",
            "username": "aa@10112.in",
            "write_calendar": "Calendar",
            "sync_calendars": ["Calendar", "Holidays"]
        }],
        "event_types": {"items": [{
            "slug": "15min",
            "title": "Free 15-minute CTO consultation",
            "description": "A free 15-minute call to discuss your IT project, information \
                            security or automation question. [About me](https://ilia.ae/en/about/)",
            "duration_min": 15,
            "slot_interval_min": null,
            "buffer_before": 0,
            "buffer_after": 15,
            "min_notice_min": 240,
            "requires_confirmation": false,
            "pending_blocks_slot": true,
            "pending_expiry_hours": 24,
            "visibility": "public",
            "location_type": "jitsi_auto",
            "location_value": null,
            "meeting_pattern_override": null,
            "timezone": null,
            "weekly_hours": null,
            "booking_horizon_days": 14,
            "bookable_from": null,
            "bookable_until": null,
            "reminder_minutes": 60,
            "max_additional_guests": 1,
            "first_slot_only": false,
            "default_calendar_view": "month",
            "week_start": null,
            "cancel_notice_min": 120,
            "reschedule_notice_min": 120,
            "sms_phone_mode": "off",
            "frequency_limits": [{"count": 4, "period": "day"}],
            "ton_price": null,
            "telegram_guest_mode": "optional",
            "enabled": true
        }, {
            "slug": "cto-60",
            "title": "CTO Consultation",
            "description": "A one-hour **CTO consultation**. Rate: $500/hour, paid in TON when you \
                            book. [About me](https://ilia.ae/en/about/)",
            "duration_min": 60,
            "slot_interval_min": null,
            "buffer_before": 0,
            "buffer_after": 15,
            "min_notice_min": 1440,
            "requires_confirmation": false,
            "pending_blocks_slot": true,
            "pending_expiry_hours": 24,
            "visibility": "public",
            "location_type": "jitsi_auto",
            "location_value": null,
            "meeting_pattern_override": null,
            "timezone": null,
            "weekly_hours": null,
            "booking_horizon_days": 14,
            "bookable_from": null,
            "bookable_until": null,
            "reminder_minutes": 60,
            "max_additional_guests": 1,
            "first_slot_only": false,
            "default_calendar_view": "month",
            "week_start": null,
            "cancel_notice_min": 120,
            "reschedule_notice_min": 120,
            "sms_phone_mode": "off",
            "frequency_limits": [],
            "ton_price": {"usd": "500.00", "window_min": null, "enabled": true},
            "telegram_guest_mode": "optional",
            "enabled": true
        }]}
    })
}

fn paths(report: &Report) -> Vec<&str> {
    report.changed.iter().map(|c| c.path.as_str()).collect()
}

/// Files the run left in the data directory under a temporary name.
fn leftovers(env: &ApplyEnv) -> Vec<String> {
    std::fs::read_dir(&env.dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".calrs-apply-"))
        .collect()
}

/// The admin's avatar files.
fn avatar_files(env: &ApplyEnv) -> Vec<String> {
    let Ok(dir) = std::fs::read_dir(env.dir.join("avatars")) else {
        return Vec::new();
    };
    let mut names: Vec<String> = dir
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

// --- The whole document ---

#[tokio::test]
async fn the_production_document_applies_once_then_changes_nothing_and_exports_itself() {
    let env = ApplyEnv::new().await;
    seed_kerio(&env).await;
    let (file, sha) = image(&env, JPEG);
    let json = doc(production(&file, &sha));

    // The dry run reports what the run will do and writes nothing.
    let before = env.snapshot().await;
    let dry = env.apply(&json, true).await.unwrap();
    assert!(dry.dry_run);
    assert_eq!(env.snapshot().await, before, "the dry run wrote");
    assert!(avatar_files(&env).is_empty());
    assert!(leftovers(&env).is_empty());

    let report = env.apply(&json, false).await.unwrap();
    assert!(!report.dry_run);
    assert_eq!(report.changed, dry.changed);
    assert_eq!(report.restart, dry.restart);
    assert_eq!(report.unmanaged, dry.unmanaged);
    assert!(report.restart, "the theme and the company link changed");
    assert_eq!(report.unmanaged.event_types, Vec::<String>::new());
    let changed = paths(&report);
    for path in [
        "profile.name",
        "profile.username",
        "profile.title",
        "profile.bio",
        "profile.timezone",
        "profile.language",
        "profile.weekly_hours",
        "profile.avatar",
        "site.theme",
        "site.custom.accent",
        "site.german_formal",
        "site.registration",
        "site.allowed_email_domains",
        "site.company_link",
        "site.jitsi",
        "sources[0].write_calendar",
        "event_types[15min]",
        "event_types[cto-60]",
    ] {
        assert!(changed.contains(&path), "{path} not in {changed:?}");
    }
    // Already so on a fresh database: not reported.
    for path in ["profile.booking_email", "site.mfa_policy", "site.oidc"] {
        assert!(!changed.iter().any(|p| p.starts_with(path)), "{path}");
    }
    let avatar = report
        .changed
        .iter()
        .find(|c| c.path == "profile.avatar")
        .unwrap();
    assert_eq!(avatar.before, Value::Null);
    assert_eq!(avatar.after, json!(format!("sha256:{sha}")));
    assert_eq!(avatar_files(&env), vec![format!("{}.jpg", env.admin_id)]);
    assert!(leftovers(&env).is_empty());

    // What the public pages read.
    let (username, timezone): (String, String) =
        sqlx::query_as("SELECT username, timezone FROM users WHERE id = ?")
            .bind(&env.admin_id)
            .fetch_one(&env.pool)
            .await
            .unwrap();
    assert_eq!(
        (username.as_str(), timezone.as_str()),
        ("ilia", "Asia/Almaty")
    );
    let write_href: Option<String> =
        sqlx::query_scalar("SELECT write_calendar_href FROM caldav_sources WHERE id = 'kerio'")
            .fetch_one(&env.pool)
            .await
            .unwrap();
    assert_eq!(
        write_href.as_deref(),
        Some("/caldav/users/10112.in/aa/Calendar/")
    );
    let rules: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT r.day_of_week, r.start_time, r.end_time FROM availability_rules r
         JOIN event_types et ON et.id = r.event_type_id
         WHERE et.slug = '15min' ORDER BY r.day_of_week",
    )
    .fetch_all(&env.pool)
    .await
    .unwrap();
    assert_eq!(
        rules,
        (1..=5)
            .map(|d| (d, "10:00".to_string(), "19:00".to_string()))
            .collect::<Vec<_>>(),
        "weekly_hours null: the profile's hours of the same run"
    );

    let price: (String, Option<i64>, Option<i64>, i64) = sqlx::query_as(
        "SELECT p.currency, p.price_usd_cents, p.window_min, p.enabled FROM ton_prices p
         JOIN event_types et ON et.id = p.event_type_id WHERE et.slug = 'cto-60'",
    )
    .fetch_one(&env.pool)
    .await
    .unwrap();
    assert_eq!(price, ("USD".to_string(), Some(50_000), None, 1));
    let priced: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ton_prices")
        .fetch_one(&env.pool)
        .await
        .unwrap();
    assert_eq!(priced, 1, "the free consultation stays free");

    // The second run: nothing to change, not even a timestamp.
    let applied = env.snapshot().await;
    let again = env.apply(&json, false).await.unwrap();
    assert_eq!(again.changed, []);
    assert!(!again.restart);
    assert_eq!(env.snapshot().await, applied);

    // The export is the document (the avatar named where it now lives),
    // reads back under the strict parser, and applies as no change.
    let exported = env.export().await.unwrap();
    let line = to_json(&exported);
    assert!(!line.contains('\n'), "one line");
    assert_eq!(apply::parse(&line).unwrap(), exported);
    let mut wanted = apply::parse(&json).unwrap();
    let stored_avatar = exported
        .profile
        .as_ref()
        .and_then(|p| p.avatar.clone())
        .flatten()
        .unwrap();
    assert_eq!(stored_avatar.sha256, sha);
    assert!(stored_avatar
        .file
        .ends_with(&format!("avatars/{}.jpg", env.admin_id)));
    wanted.profile.as_mut().unwrap().avatar = Some(Some(stored_avatar));
    assert_eq!(exported, wanted);

    let report = env.apply(&line, false).await.unwrap();
    assert_eq!(report.changed, []);
    assert!(!report.restart);
    assert_eq!(env.snapshot().await, applied);
}

#[tokio::test]
async fn the_export_of_a_fresh_database_applies_as_no_change() {
    let env = ApplyEnv::new().await;
    seed_kerio(&env).await;
    let exported = env.export().await.unwrap();
    assert_eq!(exported.format, apply::FORMAT);
    assert_eq!(exported.user, ADMIN_EMAIL);
    assert!(exported.profile.is_some());
    assert!(exported.site.is_some());
    assert_eq!(exported.sources.as_ref().map(Vec::len), Some(1));
    assert_eq!(
        exported.event_types.as_ref().map(|e| e.items.len()),
        Some(0)
    );

    let before = env.snapshot().await;
    for dry_run in [true, false] {
        let report = env.apply(&to_json(&exported), dry_run).await.unwrap();
        assert_eq!(report.changed, [], "dry run {dry_run}");
        assert!(!report.restart);
        assert_eq!(env.snapshot().await, before, "dry run {dry_run}");
    }
}

/// Values written outside the forms (by hand, by an older calrs, by a
/// migration) that no document can state: the export leaves each such key
/// out, so applying it still changes nothing.
#[tokio::test]
async fn the_export_of_a_hand_edited_database_applies_as_no_change() {
    let env = ApplyEnv::new().await;
    seed_kerio(&env).await;
    let exec = |sql: &'static str| {
        let pool = env.pool.clone();
        async move {
            sqlx::query(sql).execute(&pool).await.unwrap();
        }
    };
    exec(
        "UPDATE users SET username = 'Admin', title = ' x ', timezone = 'Mars/Olympus', \
          language = 'xx' WHERE email = 'admin@example.com'",
    )
    .await;
    exec(
        "UPDATE auth_config SET theme = 'custom', allowed_email_domains = 'Example.com, x.org', \
          jitsi_pattern = '{event}', oidc_client_id = 'calrs', company_link = 'javascript:x' \
          WHERE id = 'singleton'",
    )
    .await;
    // Two calendars of one name: neither can be named by the document.
    exec("UPDATE calendars SET display_name = 'Calendar' WHERE id = 'cal-hol'").await;
    // An event type the forms of today would refuse: a padded title, no
    // weekly hours, a horizon past the MCP range, a link location with no
    // value, a reminder of 0.
    sqlx::query(
        "INSERT INTO event_types (id, account_id, slug, title, duration_min, location_type, \
         booking_horizon_days, reminder_minutes)
         VALUES ('et-old', ?, 'old-call', ' Old call ', 30, 'link', 400, 0)",
    )
    .bind(&env.account_id)
    .execute(&env.pool)
    .await
    .unwrap();
    // ilia fork: a padded source name, a busy calendar of a shared name and
    // a date override the overrides page let through ("9:00").
    exec("UPDATE caldav_sources SET name = ' Kerio ' WHERE id = 'kerio'").await;
    exec("INSERT INTO event_type_calendars (event_type_id, calendar_id) VALUES ('et-old', 'cal-hol')")
        .await;
    sqlx::query(
        "INSERT INTO availability_overrides (id, event_type_id, date, start_time, end_time, \
         is_blocked) VALUES ('ov-old', 'et-old', ?, '9:00', '10:00', 0)",
    )
    .bind(
        (chrono::Utc::now().date_naive() + chrono::Duration::days(3))
            .format("%Y-%m-%d")
            .to_string(),
    )
    .execute(&env.pool)
    .await
    .unwrap();

    let exported = env.export().await.unwrap();
    let profile = exported.profile.clone().unwrap();
    assert_eq!(profile.username, None);
    assert_eq!(profile.title, None);
    assert_eq!(profile.timezone, None);
    let site = exported.site.clone().unwrap();
    assert_eq!(site.theme, None);
    assert_eq!(site.allowed_email_domains, None);
    assert_eq!(site.company_link, None);
    assert_eq!(site.jitsi, None);
    assert_eq!(site.oidc, None);
    let source = &exported.sources.as_ref().unwrap()[0];
    assert_eq!(source.write_calendar, Some(None));
    assert_eq!(
        source.sync_calendars, None,
        "two synced calendars are named Calendar"
    );
    assert_eq!(source.name, None, "a padded name");
    let items = &exported.event_types.as_ref().unwrap().items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].slug, "old-call");
    assert_eq!(items[0].busy_calendars, None, "a calendar of a shared name");
    assert_eq!(items[0].overrides, None, "an override spelled \"9:00\"");

    let before = env.snapshot().await;
    for dry_run in [true, false] {
        let report = env.apply(&to_json(&exported), dry_run).await.unwrap();
        assert_eq!(report.changed, [], "dry run {dry_run}");
        assert_eq!(env.snapshot().await, before, "dry run {dry_run}");
    }
}

/// The document names the only enabled admin; with two, `--export` needs
/// `--user`, and a document for someone else is refused.
#[tokio::test]
async fn the_admin_is_named_or_the_only_one() {
    let env = ApplyEnv::new().await;
    let second = insert_role_user(&env.pool, "second@example.com", "admin").await;
    let error = env.export().await.unwrap_err();
    assert_eq!(error.exit_code(), 2);
    assert!(error.to_string().contains("--user"), "{error}");

    let pool = env.single(true).await;
    let exported =
        apply::export_document(pool.clone(), env.dir.clone(), [0u8; 32], Some(ADMIN_EMAIL))
            .await
            .unwrap();
    pool.close().await;
    assert_eq!(exported.user, ADMIN_EMAIL);

    sqlx::query("UPDATE users SET enabled = 0 WHERE id = ?")
        .bind(&second)
        .execute(&env.pool)
        .await
        .unwrap();
    let json = doc(json!({})).replace(ADMIN_EMAIL, "second@example.com");
    let before = env.snapshot().await;
    let error = env.apply(&json, false).await.unwrap_err();
    assert!(
        matches!(&error, ApplyError::Invalid { path, .. } if path == "user"),
        "{error}"
    );
    assert_eq!(env.snapshot().await, before);
}

// --- The example document (docs/apply-example.json) ---

const EXAMPLE: &str = include_str!("../../../docs/apply-example.json");

/// The keys of the object at `pointer`, sorted.
fn keys(value: &Value, pointer: &str) -> Vec<String> {
    let mut keys: Vec<String> = value
        .pointer(pointer)
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("no object at {pointer}"))
        .keys()
        .cloned()
        .collect();
    keys.sort();
    keys
}

/// The fork's example applies (on a server with its source), a second run
/// changes nothing, it names every key the export knows, and the parser
/// keeps every key it gives.
#[tokio::test]
async fn the_example_document_is_valid_complete_and_applies() {
    let example: Value = serde_json::from_str(EXAMPLE).unwrap();
    let parsed = apply::parse(EXAMPLE).unwrap();
    assert_eq!(
        serde_json::to_value(&parsed).unwrap(),
        example,
        "the parser drops or rewrites a key of the example"
    );

    let env = ApplyEnv::new().await;
    sqlx::query(
        "INSERT INTO caldav_sources (id, account_id, name, url, username)
         VALUES ('src', ?, 'Mail', 'https://mail.example.com/caldav/', 'me@example.com')",
    )
    .bind(&env.account_id)
    .execute(&env.pool)
    .await
    .unwrap();
    for (id, name, synced) in [
        ("c1", "Calendar", 0),
        ("c2", "Holidays", 1),
        ("c3", "Birthdays", 1),
    ] {
        sqlx::query(
            "INSERT INTO calendars (id, source_id, href, display_name, sync_enabled)
             VALUES (?, 'src', ?, ?, ?)",
        )
        .bind(id)
        .bind(format!("/caldav/me/{name}/"))
        .bind(name)
        .bind(synced)
        .execute(&env.pool)
        .await
        .unwrap();
    }
    let (file, sha) = image(&env, JPEG);
    let mut json = example.clone();
    json["user"] = json!(ADMIN_EMAIL);
    json["profile"]["avatar"] = json!({"file": file, "sha256": sha});
    let json = json.to_string();

    let report = env.apply(&json, false).await.unwrap();
    let changed = paths(&report);
    for path in [
        "sources[0].write_calendar",
        "sources[0].sync_calendars",
        "event_types[15min]",
        "event_types[intro]",
    ] {
        assert!(changed.contains(&path), "{path} not in {changed:?}");
    }
    let applied = env.snapshot().await;
    let again = env.apply(&json, false).await.unwrap();
    assert_eq!(again.changed, []);
    assert_eq!(env.snapshot().await, applied);

    let exported = serde_json::to_value(env.export().await.unwrap()).unwrap();
    for pointer in [
        "",
        "/profile",
        "/profile/avatar",
        "/site",
        "/site/custom",
        "/site/jitsi",
        "/sources/0",
    ] {
        assert_eq!(
            keys(&example, pointer),
            keys(&exported, pointer),
            "{pointer}"
        );
    }
    assert_eq!(
        keys(&example, "/event_types/items/0"),
        keys(&exported, "/event_types/items/0")
    );
    let report = env.apply(&exported.to_string(), false).await.unwrap();
    assert_eq!(report.changed, []);
    assert_eq!(env.snapshot().await, applied);
}

// --- A file after the commit ---

/// The database commits, then the avatar cannot be moved into place: the
/// error says the settings were written (exit 1) and still carries the
/// report, `restart` included (the next run finds the theme stored and could
/// not ask for it any more); the next run, which sees the file missing, puts
/// the file there.
#[tokio::test]
async fn an_avatar_that_cannot_be_placed_after_the_commit_is_placed_by_the_next_run() {
    let env = ApplyEnv::new().await;
    let (file, sha) = image(&env, JPEG);
    let json = doc(json!({
        "profile": {"avatar": {"file": file, "sha256": sha}},
        "site": {"theme": "nord"},
    }));
    // A file where the avatars directory goes.
    std::fs::write(env.dir.join("avatars"), b"in the way").unwrap();

    let error = env.apply(&json, false).await.unwrap_err();
    assert!(matches!(error, ApplyError::AfterCommit { .. }), "{error}");
    assert_eq!(error.exit_code(), 1);
    let message = error.to_string();
    assert!(message.contains("the settings were written"), "{message}");
    assert!(!message.contains("nothing was written"), "{message}");
    let report = error.report().expect("the report of the committed run");
    assert_eq!(paths(report), ["profile.avatar", "site.theme"]);
    assert!(report.restart, "the theme is cached at startup");
    assert!(!report.dry_run);
    let line = serde_json::to_string(report).unwrap();
    assert!(!line.contains('\n'), "{line}");
    assert_eq!(serde_json::from_str::<Report>(&line).unwrap(), *report);
    assert!(leftovers(&env).is_empty());
    let stored: Option<String> = sqlx::query_scalar("SELECT avatar_path FROM users WHERE id = ?")
        .bind(&env.admin_id)
        .fetch_one(&env.pool)
        .await
        .unwrap();
    assert!(stored.is_some(), "the commit stands");

    std::fs::remove_file(env.dir.join("avatars")).unwrap();
    let report = env.apply(&json, false).await.unwrap();
    assert_eq!(paths(&report), ["profile.avatar"]);
    assert_eq!(report.changed[0].before, json!("(file missing)"));
    assert_eq!(
        std::fs::read(
            env.dir
                .join("avatars")
                .join(format!("{}.jpg", env.admin_id))
        )
        .unwrap(),
        JPEG
    );
    let again = env.apply(&json, false).await.unwrap();
    assert_eq!(again.changed, []);
}

// --- The output ---

/// What `config apply` prints: one line, in a shape the role parses
/// (`changed`, `restart`, `unmanaged.event_types`), and that a reader with
/// the same strict schema reads back.
#[tokio::test]
async fn the_report_is_one_json_line() {
    let env = ApplyEnv::new().await;
    let report = env
        .apply(
            &doc(json!({"profile": {"bio": "line one\n\nline two"}})),
            false,
        )
        .await
        .unwrap();
    let line = serde_json::to_string(&report).unwrap();
    assert!(!line.contains('\n'), "{line}");
    let value: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(value["format"], json!(apply::FORMAT));
    assert_eq!(value["dry_run"], json!(false));
    assert_eq!(value["restart"], json!(false));
    assert_eq!(value["unmanaged"]["event_types"], json!([]));
    assert_eq!(
        value["changed"],
        json!([{"path": "profile.bio", "before": null, "after": "line one\n\nline two"}])
    );
    assert_eq!(serde_json::from_str::<Report>(&line).unwrap(), report);
}

#[tokio::test]
async fn a_document_is_read_strictly_before_anything_else() {
    let env = ApplyEnv::new().await;
    let before = env.snapshot().await;
    let cases: &[(&str, &str)] = &[
        ("", "(document)"),
        ("[]", "format"),
        (r#"{"user":"admin@example.com"}"#, "format"),
        (
            r#"{"format":"calrs-apply-v2","user":"admin@example.com","future":1}"#,
            "format",
        ),
        (r#"{"format":"calrs-apply-v1"}"#, "(document)"),
        (
            r#"{"format":"calrs-apply-v1","user":"admin@example.com","extra":1}"#,
            "(document)",
        ),
        (
            r#"{"format":"calrs-apply-v1","user":"admin@example.com","profile":null}"#,
            "(document)",
        ),
        (
            r#"{"format":"calrs-apply-v1","user":"admin@example.com","sources":[{"url":"u","username":"n","sync_calendars":null}]}"#,
            "(document)",
        ),
        (
            r#"{"format":"calrs-apply-v1","user":" admin@example.com"}"#,
            "user",
        ),
        (
            r#"{"format":"calrs-apply-v1","user":"nobody@example.com"}"#,
            "user",
        ),
    ];
    for (json, path) in cases {
        let error = env.apply(json, false).await.unwrap_err();
        assert!(
            matches!(&error, ApplyError::Invalid { path: p, .. } if p == path),
            "{json}: {error}"
        );
        assert_eq!(error.exit_code(), 2);
        assert_eq!(env.snapshot().await, before, "{json}");
    }
    let too_long = " ".repeat(apply::DOCUMENT_MAX_BYTES as usize + 1);
    assert!(apply::read_document(too_long.as_bytes()).is_err());
    let report = env.apply(&empty_doc(), false).await.unwrap();
    assert_eq!(report.changed, []);
    assert_eq!(env.snapshot().await, before);
}

// --- Parity with the web forms ---

/// The web form a parity case posts to.
#[derive(Clone, Copy, Debug)]
enum Card {
    /// Settings → Profile (`settings_save`).
    Profile,
    /// Admin → Theme (`admin_update_accent`).
    Theme,
    /// Admin → Company link.
    CompanyLink,
    /// Admin → Authentication (registration and domains).
    Auth,
    /// Admin → Jitsi.
    Jitsi,
    /// Event types → New (`create_event_type`).
    NewEventType,
    /// Event types → New, its price section on, in US dollars.
    NewUsdPriced,
    /// Event types → New, its price section on, in TON.
    NewTonPriced,
}

/// What `Card::NewEventType` posts.
const NEW_EVENT_TYPE: [(&str, &str); 15] = [
    ("title", "Call"),
    ("slug", "call"),
    ("description", ""),
    ("duration_min", "30"),
    ("location_type", "custom"),
    ("location_value", "By phone"),
    ("visibility", "public"),
    ("timezone", "UTC"),
    ("avail_schedule", "1:09:00-17:00"),
    ("min_notice_min", "60"),
    ("buffer_before", "0"),
    ("buffer_after", "0"),
    ("reminder_minutes", "1440"),
    ("max_additional_guests", "0"),
    ("default_calendar_view", "month"),
];

impl Card {
    /// The fields the page posts when nothing is changed.
    fn base(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Card::Profile => &[
                ("name", "Admin"),
                ("username", "admin"),
                ("title", ""),
                ("bio", ""),
                ("booking_email", ""),
                ("timezone", "UTC"),
                ("language", ""),
                ("public_page_language", "visitor"),
                ("week_start", ""),
                ("avail_schedule", "1:09:00-17:00"),
            ],
            Card::Theme => &[
                ("theme", "custom"),
                ("custom_accent", "#313131"),
                ("custom_accent_hover", "#111111"),
                ("custom_bg", "#f6f6f6"),
                ("custom_surface", "#ffffff"),
                ("custom_text", "#313131"),
            ],
            Card::CompanyLink => &[("company_link", "")],
            Card::Auth => &[
                ("registration_enabled", "on"),
                ("allowed_email_domains", ""),
            ],
            Card::Jitsi => &[
                ("jitsi_base_url", "https://meet.jit.si"),
                ("jitsi_pattern", ""),
                ("jitsi_display_name", ""),
            ],
            Card::NewEventType => &NEW_EVENT_TYPE,
            Card::NewUsdPriced => &[
                ("ton_price_enabled", "on"),
                ("ton_price_currency", "USD"),
                ("ton_price_amount", "40.00"),
                ("ton_window_min", ""),
            ],
            Card::NewTonPriced => &[
                ("ton_price_enabled", "on"),
                ("ton_price_currency", "TON"),
                ("ton_price_amount", "100.00"),
                ("ton_window_min", ""),
            ],
        }
    }

    /// The fields posted under `base`: a priced card is the new event type
    /// form with its price section.
    fn under(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Card::NewUsdPriced | Card::NewTonPriced => &NEW_EVENT_TYPE,
            _ => &[],
        }
    }
}

/// Post `card` with `overrides` on top of its base fields, as the admin.
async fn post(env: &ApplyEnv, card: Card, overrides: &[(&'static str, &str)]) -> Response {
    let mut fields: Vec<(&'static str, String)> = card
        .under()
        .iter()
        .chain(card.base())
        .map(|(k, v)| (*k, v.to_string()))
        .collect();
    for (key, value) in overrides {
        match fields.iter_mut().find(|(k, _)| k == key) {
            Some(field) => field.1 = value.to_string(),
            None => fields.push((key, value.to_string())),
        }
    }
    let state = crate::web::build_state(env.pool.clone(), env.dir.clone(), [0u8; 32]).await;
    let me = apply::admin(&env.pool, Some(ADMIN_EMAIL)).await.unwrap();
    let ctx = Ctx::new(state, me, false);
    let state = || State(ctx.state.clone());
    match card {
        Card::Profile => {
            let (headers, form) = apply::form(&fields).await.unwrap();
            crate::web::dashboard::settings_save(state(), ctx.auth_user(), headers, form)
                .await
                .into_response()
        }
        Card::Theme => {
            let (headers, form) = apply::form(&fields).await.unwrap();
            crate::web::admin::admin_update_accent(state(), ctx.admin_user(), headers, form)
                .await
                .into_response()
        }
        Card::CompanyLink => {
            let (headers, form) = apply::form(&fields).await.unwrap();
            crate::web::admin::admin_update_company_link(state(), ctx.admin_user(), headers, form)
                .await
                .into_response()
        }
        Card::Auth => {
            let (headers, form) = apply::form(&fields).await.unwrap();
            crate::web::admin::admin_update_auth(state(), ctx.admin_user(), headers, form)
                .await
                .into_response()
        }
        Card::Jitsi => {
            let (headers, form) = apply::form(&fields).await.unwrap();
            crate::web::admin::admin_update_jitsi(state(), ctx.admin_user(), headers, form)
                .await
                .into_response()
        }
        Card::NewEventType | Card::NewUsdPriced | Card::NewTonPriced => {
            let (headers, form) = apply::form(&fields).await.unwrap();
            crate::web::event_types::create_event_type(state(), ctx.auth_user(), headers, form)
                .await
                .into_response()
        }
    }
}

/// Whether the page refused the form: an error on the page it answers with,
/// or a redirect to `?error=` (the admin cards).
async fn web_refused(response: Response) -> bool {
    let to_error = response
        .headers()
        .get(axum::http::header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|l| l.contains("error="));
    to_error || apply::answered("web", response).await.is_err()
}

/// What the web form does with a value apply refuses.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Web {
    /// Refuses it and stores nothing.
    Refuses,
    /// Stores something else: a run that posted it would store another
    /// value than the document's, and report it again on every run.
    Rewrites,
    /// Stores it as given. apply is stricter on purpose; the reason is
    /// next to the case.
    Keeps,
}

struct Parity {
    card: Card,
    field: &'static str,
    value: &'static str,
    /// The stored value after the post, as text (one row, or NULL).
    read: &'static str,
    /// What `read` gives when the value is stored as given.
    given: Option<&'static str>,
    web: Web,
    /// The document apply refuses, and the path it names.
    doc: Value,
    path: &'static str,
}

/// Every value of the canonical-value table (design §6) and the cases the
/// sections refuse beyond it, posted once to the web form and once in a
/// document. apply never accepts what the form would rewrite or refuse;
/// where it refuses what the form keeps, the case says why. Profile cases
/// beyond these are in `apply_profile.rs` (`the_web_form_rewrites_what_
/// apply_refuses`, `what_the_web_form_refuses_is_refused`).
#[tokio::test]
async fn every_value_apply_refuses_the_web_form_refuses_rewrites_or_keeps_on_purpose() {
    const USER: &str = "SELECT username FROM users WHERE email = 'admin@example.com'";
    const PROFILE_HOURS: &str = "SELECT group_concat(r, ',') FROM (SELECT day_of_week || ':' \
        || start_time || '-' || end_time AS r FROM user_availability_rules \
        ORDER BY day_of_week, start_time)";
    const ET_HOURS: &str = "SELECT group_concat(r, ',') FROM (SELECT day_of_week || ':' \
        || start_time || '-' || end_time AS r FROM availability_rules \
        ORDER BY day_of_week, start_time)";
    let item = |extra: Value| {
        let mut item = json!({"slug": "call", "title": "Call", "location_type": "custom",
                              "location_value": "By phone"});
        for (k, v) in extra.as_object().unwrap() {
            item[k] = v.clone();
        }
        json!({"event_types": {"items": [item]}})
    };
    const PRICES: &str = "SELECT group_concat(currency || ':' \
        || coalesce(price_nano, price_usd_cents) || ':' || coalesce(window_min, '')) \
        FROM ton_prices";
    let window = |start: &str, end: &str| json!({"start": start, "end": end});
    let overlapping =
        json!([{"day": "mon", "windows": [window("10:00", "12:00"), window("11:00", "13:00")]}]);
    let cases = vec![
        // --- Profile ---
        Parity {
            card: Card::Profile,
            field: "username",
            value: "Ilia",
            read: USER,
            given: Some("Ilia"),
            web: Web::Rewrites,
            doc: json!({"profile": {"username": "Ilia"}}),
            path: "profile.username",
        },
        Parity {
            card: Card::Profile,
            field: "timezone",
            value: "Asia/Almty",
            read: "SELECT timezone FROM users WHERE email = 'admin@example.com'",
            given: Some("Asia/Almty"),
            web: Web::Rewrites,
            doc: json!({"profile": {"timezone": "Asia/Almty"}}),
            path: "profile.timezone",
        },
        Parity {
            card: Card::Profile,
            field: "language",
            value: "xx",
            read: "SELECT language FROM users WHERE email = 'admin@example.com'",
            given: Some("xx"),
            web: Web::Rewrites,
            doc: json!({"profile": {"language": "xx"}}),
            path: "profile.language",
        },
        // Kept: overlapping windows are the MCP tools' refusal
        // (`weekly_rules`), kept so the two writers agree.
        Parity {
            card: Card::Profile,
            field: "avail_schedule",
            value: "1:10:00-12:00,11:00-13:00",
            read: PROFILE_HOURS,
            given: Some("1:10:00-12:00,1:11:00-13:00"),
            web: Web::Keeps,
            doc: json!({"profile": {"weekly_hours": overlapping}}),
            path: "profile.weekly_hours",
        },
        // Kept, but the public page drops the link: a run would store a
        // bio whose link never shows.
        Parity {
            card: Card::Profile,
            field: "bio",
            value: "[x](javascript:alert(1))",
            read: "SELECT bio FROM users WHERE email = 'admin@example.com'",
            given: Some("[x](javascript:alert(1))"),
            web: Web::Keeps,
            doc: json!({"profile": {"bio": "[x](javascript:alert(1))"}}),
            path: "profile.bio",
        },
        // --- Site ---
        // Kept, but `custom_theme_css` then writes no theme at all.
        Parity {
            card: Card::Theme,
            field: "custom_accent",
            value: "#31313",
            read: "SELECT custom_accent FROM auth_config",
            given: Some("#31313"),
            web: Web::Keeps,
            doc: json!({"site": {"theme": "custom", "custom": {"accent": "#31313",
                "accent_hover": "#111111", "bg": "#f6f6f6", "surface": "#ffffff", "text": "#313131"}}}),
            path: "site.custom.accent",
        },
        Parity {
            card: Card::Theme,
            field: "theme",
            value: "Nord",
            read: "SELECT theme FROM auth_config",
            given: Some("Nord"),
            web: Web::Rewrites,
            doc: json!({"site": {"theme": "Nord"}}),
            path: "(document)",
        },
        Parity {
            card: Card::CompanyLink,
            field: "company_link",
            value: "javascript:alert(1)",
            read: "SELECT company_link FROM auth_config",
            given: Some("javascript:alert(1)"),
            web: Web::Refuses,
            doc: json!({"site": {"company_link": "javascript:alert(1)"}}),
            path: "site.company_link",
        },
        Parity {
            card: Card::CompanyLink,
            field: "company_link",
            value: " https://ilia.ae",
            read: "SELECT company_link FROM auth_config",
            given: Some(" https://ilia.ae"),
            web: Web::Rewrites,
            doc: json!({"site": {"company_link": " https://ilia.ae"}}),
            path: "site.company_link",
        },
        // Kept: a link with no host goes nowhere.
        Parity {
            card: Card::CompanyLink,
            field: "company_link",
            value: "https://",
            read: "SELECT company_link FROM auth_config",
            given: Some("https://"),
            web: Web::Keeps,
            doc: json!({"site": {"company_link": "https://"}}),
            path: "site.company_link",
        },
        Parity {
            card: Card::Auth,
            field: "allowed_email_domains",
            value: "",
            read: "SELECT allowed_email_domains FROM auth_config",
            given: Some(""),
            web: Web::Rewrites,
            doc: json!({"site": {"allowed_email_domains": ""}}),
            path: "site.allowed_email_domains",
        },
        // Kept: one spelling per list, so a document and the database
        // compare as text.
        Parity {
            card: Card::Auth,
            field: "allowed_email_domains",
            value: "Ilia.ae, example.org",
            read: "SELECT allowed_email_domains FROM auth_config",
            given: Some("Ilia.ae, example.org"),
            web: Web::Keeps,
            doc: json!({"site": {"allowed_email_domains": "Ilia.ae, example.org"}}),
            path: "site.allowed_email_domains",
        },
        Parity {
            card: Card::Jitsi,
            field: "jitsi_base_url",
            value: "",
            read: "SELECT jitsi_base_url FROM auth_config",
            given: Some(""),
            web: Web::Rewrites,
            doc: json!({"site": {"jitsi": {"base_url": "", "pattern": null, "display_name": null}}}),
            path: "site.jitsi.base_url",
        },
        // Kept: meeting links are built from it on every booking.
        Parity {
            card: Card::Jitsi,
            field: "jitsi_base_url",
            value: "ftp://meet.example",
            read: "SELECT jitsi_base_url FROM auth_config",
            given: Some("ftp://meet.example"),
            web: Web::Keeps,
            doc: json!({"site": {"jitsi": {"base_url": "ftp://meet.example", "pattern": null,
                "display_name": null}}}),
            path: "site.jitsi.base_url",
        },
        // --- Event types ---
        Parity {
            card: Card::NewEventType,
            field: "slug",
            value: "My Call",
            read: "SELECT group_concat(slug) FROM event_types",
            given: Some("My Call"),
            web: Web::Rewrites,
            doc: item(json!({"slug": "My Call"})),
            path: "event_types[My Call]",
        },
        Parity {
            card: Card::NewEventType,
            field: "slug",
            value: "15Min",
            read: "SELECT group_concat(slug) FROM event_types",
            given: Some("15Min"),
            web: Web::Rewrites,
            doc: item(json!({"slug": "15Min"})),
            path: "event_types[15Min]",
        },
        Parity {
            card: Card::NewEventType,
            field: "title",
            value: " Padded",
            read: "SELECT group_concat(title) FROM event_types",
            given: Some(" Padded"),
            web: Web::Rewrites,
            doc: item(json!({"title": " Padded"})),
            path: "event_types[call].title",
        },
        Parity {
            card: Card::NewEventType,
            field: "description",
            value: "",
            read: "SELECT group_concat(description) FROM event_types",
            given: Some(""),
            web: Web::Rewrites,
            doc: item(json!({"description": ""})),
            path: "event_types[call].description",
        },
        Parity {
            card: Card::NewEventType,
            field: "duration_min",
            value: "0",
            read: "SELECT group_concat(duration_min) FROM event_types",
            given: Some("0"),
            web: Web::Refuses,
            doc: item(json!({"duration_min": 0})),
            path: "event_types[call].duration_min",
        },
        Parity {
            card: Card::NewEventType,
            field: "buffer_after",
            value: "-1",
            read: "SELECT group_concat(buffer_after) FROM event_types",
            given: Some("-1"),
            web: Web::Refuses,
            doc: item(json!({"buffer_after": -1})),
            path: "event_types[call].buffer_after",
        },
        Parity {
            card: Card::NewEventType,
            field: "timezone",
            value: "Asia/Almty",
            read: "SELECT group_concat(timezone) FROM event_types",
            given: Some("Asia/Almty"),
            web: Web::Rewrites,
            doc: item(json!({"timezone": "Asia/Almty"})),
            path: "event_types[call].timezone",
        },
        Parity {
            card: Card::NewEventType,
            field: "location_value",
            value: "",
            read: "SELECT group_concat(slug) FROM event_types",
            given: Some("call"),
            web: Web::Refuses,
            doc: json!({"event_types": {"items": [{"slug": "call", "title": "Call"}]}}),
            path: "event_types[call].location_value",
        },
        Parity {
            card: Card::NewEventType,
            field: "avail_schedule",
            value: "",
            read: ET_HOURS,
            given: None,
            web: Web::Rewrites,
            doc: item(json!({"weekly_hours": []})),
            path: "event_types[call].weekly_hours",
        },
        // Kept: the MCP tools' refusal (`weekly_rules`), as for the profile.
        Parity {
            card: Card::NewEventType,
            field: "avail_schedule",
            value: "1:10:00-12:00,11:00-13:00",
            read: ET_HOURS,
            given: Some("1:10:00-12:00,1:11:00-13:00"),
            web: Web::Keeps,
            doc: item(json!({"weekly_hours": overlapping})),
            path: "event_types[call].weekly_hours[0].windows",
        },
        // The form stores 0 as no reminder (null in a document).
        Parity {
            card: Card::NewEventType,
            field: "reminder_minutes",
            value: "0",
            read: "SELECT group_concat(reminder_minutes) FROM event_types",
            given: Some("0"),
            web: Web::Rewrites,
            doc: item(json!({"reminder_minutes": 0})),
            path: "event_types[call].reminder_minutes",
        },
        // Kept: the MCP range; the server's horizon caps it anyway.
        Parity {
            card: Card::NewEventType,
            field: "booking_horizon_days",
            value: "400",
            read: "SELECT group_concat(booking_horizon_days) FROM event_types",
            given: Some("400"),
            web: Web::Keeps,
            doc: item(json!({"booking_horizon_days": 400})),
            path: "event_types[call].booking_horizon_days",
        },
        // --- The TON price section ---
        Parity {
            card: Card::NewUsdPriced,
            field: "ton_price_amount",
            value: "0.50",
            read: PRICES,
            given: Some("USD:50:"),
            web: Web::Refuses,
            doc: item(json!({"ton_price": {"usd": "0.50"}})),
            path: "event_types[call].ton_price",
        },
        Parity {
            card: Card::NewUsdPriced,
            field: "ton_price_amount",
            value: "40.005",
            read: PRICES,
            given: None,
            web: Web::Refuses,
            doc: item(json!({"ton_price": {"usd": "40.005"}})),
            path: "event_types[call].ton_price",
        },
        Parity {
            card: Card::NewUsdPriced,
            field: "ton_window_min",
            value: "5",
            read: PRICES,
            given: Some("USD:4000:5"),
            web: Web::Refuses,
            doc: item(json!({"ton_price": {"usd": "40.00", "window_min": 5}})),
            path: "event_types[call].ton_price",
        },
        Parity {
            card: Card::NewTonPriced,
            field: "ton_price_amount",
            value: "0.005",
            read: PRICES,
            given: None,
            web: Web::Refuses,
            doc: item(json!({"ton_price": {"ton": "0.005"}})),
            path: "event_types[call].ton_price",
        },
        // Kept: one spelling per price, the one the form shows a stored
        // price in, so a document and an export compare as text.
        Parity {
            card: Card::NewUsdPriced,
            field: "ton_price_amount",
            value: "500",
            read: PRICES,
            given: Some("USD:50000:"),
            web: Web::Keeps,
            doc: item(json!({"ton_price": {"usd": "500"}})),
            path: "event_types[call].ton_price.usd",
        },
        Parity {
            card: Card::NewTonPriced,
            field: "ton_price_amount",
            value: "100",
            read: PRICES,
            given: Some("TON:100000000000:"),
            web: Web::Keeps,
            doc: item(json!({"ton_price": {"ton": "100"}})),
            path: "event_types[call].ton_price.ton",
        },
        // Rewritten: the form trims the amount.
        Parity {
            card: Card::NewUsdPriced,
            field: "ton_price_amount",
            value: " 40.00",
            read: PRICES,
            given: Some(" 40.00"),
            web: Web::Rewrites,
            doc: item(json!({"ton_price": {"usd": " 40.00"}})),
            path: "event_types[call].ton_price.usd",
        },
    ];
    let mut wrong = Vec::new();
    for case in &cases {
        // The web form, on a database of its own.
        let env = ApplyEnv::new().await;
        let read = || {
            let pool = env.pool.clone();
            async move {
                sqlx::query_scalar::<_, Option<String>>(case.read)
                    .fetch_one(&pool)
                    .await
                    .unwrap()
            }
        };
        let stored_before = read().await;
        let refused = web_refused(post(&env, case.card, &[(case.field, case.value)]).await).await;
        let stored = read().await;
        let web = if refused {
            assert_eq!(
                stored, stored_before,
                "{:?} {} refused but stored",
                case.card, case.value
            );
            Web::Refuses
        } else if stored.as_deref() != case.given {
            Web::Rewrites
        } else {
            Web::Keeps
        };
        if web != case.web {
            wrong.push(format!(
                "{:?} {}={:?}: the web form {web:?} (stored {stored:?}), the table says {:?}",
                case.card, case.field, case.value, case.web
            ));
        }

        // The document, on a fresh database.
        let env = ApplyEnv::new().await;
        let before = env.snapshot().await;
        let json = doc(case.doc.clone());
        match env.apply(&json, false).await {
            Err(ApplyError::Invalid { path, message }) if path.starts_with(case.path) => {
                assert!(!message.is_empty());
            }
            other => wrong.push(format!(
                "{json}: expected invalid {}, got {other:?}",
                case.path
            )),
        }
        assert_eq!(env.snapshot().await, before, "{json} wrote");
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));

    // The accent the theme form keeps: no theme is written.
    let env = ApplyEnv::new().await;
    post(&env, Card::Theme, &[("custom_accent", "#31313")]).await;
    assert_eq!(crate::web::branding::build_theme_css(&env.pool).await, "");
    post(&env, Card::Theme, &[]).await;
    assert!(crate::web::branding::build_theme_css(&env.pool)
        .await
        .contains("#313131"));
}
