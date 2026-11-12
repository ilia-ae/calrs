//! ilia fork: `config apply`, the `event_types` and `sources` sections
//! (docs/apply-schema.md). Shared fixtures: `apply_support`.

use super::apply_support::*;
use super::*;
use crate::web::apply::{self, ApplyError, Change, Report};
use serde_json::{json, Value};

/// A document with `sections` (an object of section keys) for the admin.
fn doc(sections: Value) -> String {
    let mut doc = json!({"format": apply::FORMAT, "user": ADMIN_EMAIL});
    for (key, value) in sections.as_object().unwrap() {
        doc[key] = value.clone();
    }
    doc.to_string()
}

/// An `event_types` document with `items` and `prune`.
fn et_doc(items: Value, prune: Option<&str>) -> String {
    let mut section = json!({ "items": items });
    if let Some(prune) = prune {
        section["prune"] = json!(prune);
    }
    doc(json!({ "event_types": section }))
}

/// The event type of book.ilia.ae (design §5), every key given.
fn consultation() -> Value {
    json!({
        "slug": "15min",
        "title": "Free 15-minute CTO consultation",
        "description": "A short call about your **IT** question. [About me](https://ilia.ae)",
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
        "telegram_guest_mode": "optional",
        "enabled": true
    })
}

fn paths(report: &Report) -> Vec<&str> {
    report.changed.iter().map(|c| c.path.as_str()).collect()
}

fn change<'a>(report: &'a Report, path: &str) -> &'a Change {
    report
        .changed
        .iter()
        .find(|c| c.path == path)
        .unwrap_or_else(|| panic!("no change at {path}: {:?}", report.changed))
}

async fn et_id(env: &ApplyEnv, slug: &str) -> Option<String> {
    sqlx::query_scalar("SELECT id FROM event_types WHERE slug = ? AND team_id IS NULL")
        .bind(slug)
        .fetch_optional(&env.pool)
        .await
        .unwrap()
}

async fn column<T>(env: &ApplyEnv, slug: &str, column: &str) -> T
where
    T: for<'r> sqlx::Decode<'r, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Send + Unpin,
{
    sqlx::query_scalar(&format!(
        "SELECT {column} FROM event_types WHERE slug = ? AND team_id IS NULL"
    ))
    .bind(slug)
    .fetch_one(&env.pool)
    .await
    .unwrap()
}

async fn rules(env: &ApplyEnv, slug: &str) -> Vec<(i64, String, String)> {
    sqlx::query_as(
        "SELECT r.day_of_week, r.start_time, r.end_time FROM availability_rules r
         JOIN event_types et ON et.id = r.event_type_id
         WHERE et.slug = ? ORDER BY 1, 2, 3",
    )
    .bind(slug)
    .fetch_all(&env.pool)
    .await
    .unwrap()
}

async fn limits(env: &ApplyEnv, slug: &str) -> Vec<(i64, String)> {
    sqlx::query_as(
        "SELECT l.max_bookings, l.period FROM booking_frequency_limits l
         JOIN event_types et ON et.id = l.event_type_id
         WHERE et.slug = ? ORDER BY 2, 1",
    )
    .bind(slug)
    .fetch_all(&env.pool)
    .await
    .unwrap()
}

async fn telegram_mode(env: &ApplyEnv, slug: &str) -> Option<String> {
    sqlx::query_scalar(
        "SELECT t.guest_mode FROM telegram_event_types t
         JOIN event_types et ON et.id = t.event_type_id WHERE et.slug = ?",
    )
    .bind(slug)
    .fetch_optional(&env.pool)
    .await
    .unwrap()
}

/// The profile's default hours, replaced.
async fn set_profile_hours(env: &ApplyEnv, hours: &[(i64, &str, &str)]) {
    sqlx::query("DELETE FROM user_availability_rules WHERE user_id = ?")
        .bind(&env.admin_id)
        .execute(&env.pool)
        .await
        .unwrap();
    for (day, start, end) in hours {
        sqlx::query(
            "INSERT INTO user_availability_rules (id, user_id, day_of_week, start_time, end_time)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&env.admin_id)
        .bind(day)
        .bind(start)
        .bind(end)
        .execute(&env.pool)
        .await
        .unwrap();
    }
}

/// `json` is refused with exit 2 under `path`, and nothing is written.
async fn refused(env: &ApplyEnv, json: &str, path: &str) -> ApplyError {
    let before = env.snapshot().await;
    let error = env.apply(json, false).await.unwrap_err();
    assert_eq!(error.exit_code(), 2, "{error}");
    assert!(
        error.to_string().contains(path),
        "expected {path} in: {error}"
    );
    assert_eq!(env.snapshot().await, before, "a refused run wrote: {error}");
    error
}

// --- Event types ---

#[tokio::test]
async fn the_first_run_creates_by_slug_and_the_second_changes_nothing() {
    let env = ApplyEnv::new().await;
    let json = et_doc(json!([consultation()]), None);

    let report = env.apply(&json, false).await.unwrap();
    assert_eq!(
        report.changed,
        vec![Change {
            path: "event_types[15min]".into(),
            before: Value::Null,
            after: json!("created"),
        }]
    );
    assert!(!report.restart);
    assert!(report.unmanaged.event_types.is_empty());

    assert_eq!(
        column::<String>(&env, "15min", "title").await,
        "Free 15-minute CTO consultation"
    );
    assert_eq!(column::<i64>(&env, "15min", "duration_min").await, 15);
    assert_eq!(column::<i64>(&env, "15min", "buffer_after").await, 15);
    assert_eq!(column::<i64>(&env, "15min", "min_notice_min").await, 240);
    assert_eq!(
        column::<Option<i64>>(&env, "15min", "reminder_minutes").await,
        Some(60)
    );
    assert_eq!(
        column::<Option<i64>>(&env, "15min", "booking_horizon_days").await,
        Some(14)
    );
    assert_eq!(
        column::<Option<i64>>(&env, "15min", "cancel_notice_min").await,
        Some(120)
    );
    assert_eq!(
        column::<String>(&env, "15min", "location_type").await,
        "jitsi_auto"
    );
    assert_eq!(
        column::<Option<String>>(&env, "15min", "timezone").await,
        None,
        "timezone null follows the profile"
    );
    assert_eq!(column::<i64>(&env, "15min", "enabled").await, 1);
    // weekly_hours null: the profile's hours, the web's seed here.
    let seed: Vec<(i64, String, String)> = (1..=5)
        .map(|d| (d, "09:00".to_string(), "17:00".to_string()))
        .collect();
    assert_eq!(rules(&env, "15min").await, seed);
    assert_eq!(limits(&env, "15min").await, vec![(4, "day".to_string())]);
    assert_eq!(
        telegram_mode(&env, "15min").await.as_deref(),
        Some("optional")
    );

    let before = env.snapshot().await;
    let again = env.apply(&json, false).await.unwrap();
    assert!(again.changed.is_empty(), "{:?}", again.changed);
    assert_eq!(
        env.snapshot().await,
        before,
        "a run with nothing to do wrote"
    );
}

#[tokio::test]
async fn a_changed_field_is_updated_and_the_others_stay() {
    let env = ApplyEnv::new().await;
    env.apply(&et_doc(json!([consultation()]), None), false)
        .await
        .unwrap();

    let report = env
        .apply(
            &et_doc(
                json!([{
                    "slug": "15min",
                    "title": "Free CTO call",
                    "duration_min": 30,
                    "weekly_hours": [
                        {"day": "mon", "windows": [{"start": "10:00", "end": "13:00"},
                                                   {"start": "14:00", "end": "19:00"}]},
                        {"day": "sat", "windows": [{"start": "11:00", "end": "12:00"}]}
                    ]
                }]),
                None,
            ),
            false,
        )
        .await
        .unwrap();
    assert_eq!(
        paths(&report),
        [
            "event_types[15min].title",
            "event_types[15min].duration_min",
            "event_types[15min].weekly_hours"
        ]
    );
    let duration = change(&report, "event_types[15min].duration_min");
    assert_eq!(
        (&duration.before, &duration.after),
        (&json!(15), &json!(30))
    );
    let hours = change(&report, "event_types[15min].weekly_hours");
    assert_eq!(hours.before.as_array().unwrap().len(), 5);
    assert_eq!(
        hours.after,
        json!([
            {"day": "mon", "windows": [{"start": "10:00", "end": "13:00"},
                                       {"start": "14:00", "end": "19:00"}]},
            {"day": "sat", "windows": [{"start": "11:00", "end": "12:00"}]}
        ])
    );

    assert_eq!(column::<i64>(&env, "15min", "duration_min").await, 30);
    assert_eq!(
        rules(&env, "15min").await,
        vec![
            (1, "10:00".into(), "13:00".into()),
            (1, "14:00".into(), "19:00".into()),
            (6, "11:00".into(), "12:00".into()),
        ]
    );
    // Keys left out keep their stored values.
    assert_eq!(column::<i64>(&env, "15min", "buffer_after").await, 15);
    assert_eq!(column::<i64>(&env, "15min", "min_notice_min").await, 240);
    assert_eq!(limits(&env, "15min").await, vec![(4, "day".to_string())]);
    assert_eq!(
        telegram_mode(&env, "15min").await.as_deref(),
        Some("optional")
    );
    assert_eq!(
        column::<Option<String>>(&env, "15min", "timezone").await,
        None
    );
}

#[tokio::test]
async fn limits_telegram_and_enabled_are_written() {
    let env = ApplyEnv::new().await;
    env.apply(&et_doc(json!([consultation()]), None), false)
        .await
        .unwrap();

    let item = json!([{
        "slug": "15min",
        "frequency_limits": [{"count": 10, "period": "month"}, {"count": 2, "period": "day"}],
        "telegram_guest_mode": "required",
        "enabled": false,
        "requires_confirmation": true,
        "pending_blocks_slot": false,
        "pending_expiry_hours": null,
        "week_start": "sun"
    }]);
    let report = env.apply(&et_doc(item.clone(), None), false).await.unwrap();
    assert_eq!(
        paths(&report),
        [
            "event_types[15min].requires_confirmation",
            "event_types[15min].pending_blocks_slot",
            "event_types[15min].pending_expiry_hours",
            "event_types[15min].week_start",
            "event_types[15min].frequency_limits",
            "event_types[15min].telegram_guest_mode",
            "event_types[15min].enabled"
        ]
    );
    assert_eq!(
        change(&report, "event_types[15min].frequency_limits").after,
        json!([{"count": 2, "period": "day"}, {"count": 10, "period": "month"}])
    );
    assert_eq!(
        limits(&env, "15min").await,
        vec![(2, "day".to_string()), (10, "month".to_string())]
    );
    assert_eq!(
        telegram_mode(&env, "15min").await.as_deref(),
        Some("required")
    );
    assert_eq!(column::<i64>(&env, "15min", "enabled").await, 0);
    assert_eq!(
        column::<i64>(&env, "15min", "requires_confirmation").await,
        1
    );
    assert_eq!(column::<i64>(&env, "15min", "pending_blocks_slot").await, 0);
    assert_eq!(
        column::<Option<i64>>(&env, "15min", "pending_expiry_hours").await,
        None
    );
    assert_eq!(
        column::<Option<String>>(&env, "15min", "week_start")
            .await
            .as_deref(),
        Some("sun")
    );
    assert!(env
        .apply(&et_doc(item, None), false)
        .await
        .unwrap()
        .changed
        .is_empty());

    // [] removes every limit; telegram off and enabled again.
    let report = env
        .apply(
            &et_doc(
                json!([{"slug": "15min", "frequency_limits": [],
                        "telegram_guest_mode": "off", "enabled": true}]),
                None,
            ),
            false,
        )
        .await
        .unwrap();
    assert_eq!(report.changed.len(), 3, "{:?}", report.changed);
    assert!(limits(&env, "15min").await.is_empty());
    assert_eq!(telegram_mode(&env, "15min").await.as_deref(), Some("off"));
    assert_eq!(column::<i64>(&env, "15min", "enabled").await, 1);
}

#[tokio::test]
async fn a_timezone_is_pinned_cleared_or_pinned_to_the_one_it_follows() {
    let env = ApplyEnv::new().await;
    let with_tz = |tz: Value| {
        let mut item = consultation();
        item["timezone"] = tz;
        et_doc(json!([item]), None)
    };
    env.apply(&with_tz(json!("Europe/Berlin")), false)
        .await
        .unwrap();
    assert_eq!(
        column::<Option<String>>(&env, "15min", "timezone")
            .await
            .as_deref(),
        Some("Europe/Berlin")
    );

    // The web form cannot clear a timezone; apply can.
    let report = env.apply(&with_tz(Value::Null), false).await.unwrap();
    assert_eq!(paths(&report), ["event_types[15min].timezone"]);
    assert_eq!(
        column::<Option<String>>(&env, "15min", "timezone").await,
        None
    );

    // Nor pin the timezone it follows (the admin's, UTC).
    let profile_tz: String = sqlx::query_scalar("SELECT timezone FROM users WHERE id = ?")
        .bind(&env.admin_id)
        .fetch_one(&env.pool)
        .await
        .unwrap();
    let report = env.apply(&with_tz(json!(profile_tz)), false).await.unwrap();
    assert_eq!(paths(&report), ["event_types[15min].timezone"]);
    assert_eq!(
        column::<Option<String>>(&env, "15min", "timezone").await,
        Some(profile_tz.clone())
    );
    assert!(env
        .apply(&with_tz(json!(profile_tz)), false)
        .await
        .unwrap()
        .changed
        .is_empty());
}

#[tokio::test]
async fn weekly_hours_null_follows_the_profile_hours() {
    let env = ApplyEnv::new().await;
    set_profile_hours(&env, &[(1, "10:00", "19:00"), (2, "10:00", "19:00")]).await;
    let json = et_doc(json!([consultation()]), None);
    env.apply(&json, false).await.unwrap();
    assert_eq!(
        rules(&env, "15min").await,
        vec![
            (1, "10:00".into(), "19:00".into()),
            (2, "10:00".into(), "19:00".into())
        ]
    );

    set_profile_hours(&env, &[(3, "11:00", "12:00")]).await;
    let report = env.apply(&json, false).await.unwrap();
    assert_eq!(paths(&report), ["event_types[15min].weekly_hours"]);
    let hours = change(&report, "event_types[15min].weekly_hours");
    assert_eq!(
        hours.after,
        json!([{"day": "wed", "windows": [{"start": "11:00", "end": "12:00"}]}])
    );
    assert_eq!(
        rules(&env, "15min").await,
        vec![(3, "11:00".into(), "12:00".into())]
    );
    assert!(env.apply(&json, false).await.unwrap().changed.is_empty());
}

#[tokio::test]
async fn prune_lists_disables_or_deletes_the_types_left_out() {
    let env = ApplyEnv::new().await;
    insert_personal_et(&env.pool, &env.admin_id, "old-a").await;
    insert_personal_et(&env.pool, &env.admin_id, "old-b").await;
    let items = json!([consultation()]);

    let report = env
        .apply(&et_doc(items.clone(), None), false)
        .await
        .unwrap();
    assert_eq!(report.unmanaged.event_types, ["old-a", "old-b"]);
    assert_eq!(paths(&report), ["event_types[15min]"]);
    assert_eq!(column::<i64>(&env, "old-a", "enabled").await, 1);

    let report = env
        .apply(&et_doc(items.clone(), Some("disable")), false)
        .await
        .unwrap();
    assert_eq!(
        paths(&report),
        ["event_types[old-a].enabled", "event_types[old-b].enabled"]
    );
    let disabled = change(&report, "event_types[old-a].enabled");
    assert_eq!(
        (&disabled.before, &disabled.after),
        (&json!(true), &json!(false))
    );
    assert_eq!(column::<i64>(&env, "old-a", "enabled").await, 0);
    assert_eq!(column::<i64>(&env, "old-b", "enabled").await, 0);
    let again = env
        .apply(&et_doc(items.clone(), Some("disable")), false)
        .await
        .unwrap();
    assert!(again.changed.is_empty());
    assert_eq!(again.unmanaged.event_types, ["old-a", "old-b"]);

    // A pending or confirmed booking stops a delete, before anything is written.
    let old_a = et_id(&env, "old-a").await.unwrap();
    sqlx::query(
        "INSERT INTO bookings (id, event_type_id, uid, guest_name, guest_email, guest_timezone,
                               start_at, end_at, status, cancel_token, reschedule_token)
         VALUES ('b1', ?, 'uid-1', 'G', 'g@example.com', 'UTC',
                 '2099-01-01T10:00:00', '2099-01-01T10:30:00', 'confirmed', 'c1', 'r1')",
    )
    .bind(&old_a)
    .execute(&env.pool)
    .await
    .unwrap();
    refused(
        &env,
        &et_doc(items.clone(), Some("delete")),
        "event_types[old-a]",
    )
    .await;

    sqlx::query("UPDATE bookings SET status = 'cancelled' WHERE id = 'b1'")
        .execute(&env.pool)
        .await
        .unwrap();
    let report = env
        .apply(&et_doc(items.clone(), Some("delete")), false)
        .await
        .unwrap();
    assert_eq!(paths(&report), ["event_types[old-a]", "event_types[old-b]"]);
    let deleted = change(&report, "event_types[old-a]");
    assert_eq!(
        (&deleted.before, &deleted.after),
        (&json!("present"), &Value::Null)
    );
    assert_eq!(report.unmanaged.event_types, ["old-a", "old-b"]);
    assert!(et_id(&env, "old-a").await.is_none());
    assert!(et_id(&env, "old-b").await.is_none());
    assert!(et_id(&env, "15min").await.is_some());
    let again = env
        .apply(&et_doc(items, Some("delete")), false)
        .await
        .unwrap();
    assert!(again.changed.is_empty() && again.unmanaged.event_types.is_empty());
}

#[tokio::test]
async fn a_slug_taken_by_a_team_event_type_is_reported() {
    let env = ApplyEnv::new().await;
    sqlx::query("INSERT INTO teams (id, name, slug) VALUES ('t1', 'Team', 'team')")
        .execute(&env.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO event_types (id, account_id, slug, title, duration_min, team_id)
         VALUES ('team-et', ?, '15min', 'Team call', 30, 't1')",
    )
    .bind(&env.account_id)
    .execute(&env.pool)
    .await
    .unwrap();
    let error = refused(
        &env,
        &et_doc(json!([consultation()]), None),
        "event_types[15min]",
    )
    .await;
    assert!(error.to_string().contains("team event type"), "{error}");
}

#[tokio::test]
async fn non_canonical_or_out_of_range_items_are_refused_and_nothing_is_written() {
    let env = ApplyEnv::new().await;
    env.apply(&et_doc(json!([consultation()]), None), false)
        .await
        .unwrap();
    let window = |start: &str, end: &str| json!({"start": start, "end": end});
    let cases: Vec<(Value, &str)> = vec![
        (
            json!({"slug": "My Call", "title": "T"}),
            "event_types[My Call]",
        ),
        (json!({"slug": "15Min", "title": "T"}), "event_types[15Min]"),
        (
            json!({"slug": "15min", "title": " Padded"}),
            "event_types[15min].title",
        ),
        (
            json!({"slug": "15min", "title": ""}),
            "event_types[15min].title",
        ),
        (
            json!({"slug": "15min", "description": ""}),
            "event_types[15min].description",
        ),
        (
            json!({"slug": "15min", "timezone": "Asia/Almty"}),
            "event_types[15min].timezone",
        ),
        (
            json!({"slug": "15min", "duration_min": 0}),
            "event_types[15min].duration_min",
        ),
        (
            json!({"slug": "15min", "duration_min": 1441}),
            "event_types[15min].duration_min",
        ),
        (
            json!({"slug": "15min", "reminder_minutes": 0}),
            "event_types[15min].reminder_minutes",
        ),
        (
            json!({"slug": "15min", "buffer_after": -1}),
            "event_types[15min].buffer_after",
        ),
        (
            json!({"slug": "15min", "booking_horizon_days": 400}),
            "event_types[15min].booking_horizon_days",
        ),
        (
            json!({"slug": "15min", "bookable_from": "2026-12-01", "bookable_until": "2026-11-01"}),
            "event_types[15min].bookable_until",
        ),
        (
            json!({"slug": "15min", "bookable_from": "2026-1-5"}),
            "event_types[15min].bookable_from",
        ),
        (
            json!({"slug": "15min", "weekly_hours": [{"day": "mon", "windows": [
                window("10:00", "12:00"), window("11:00", "13:00")]}]}),
            "event_types[15min].weekly_hours[0].windows",
        ),
        (
            json!({"slug": "15min", "weekly_hours": [{"day": "mon", "windows": [
                window("14:00", "15:00"), window("10:00", "12:00")]}]}),
            "event_types[15min].weekly_hours[0].windows",
        ),
        (
            json!({"slug": "15min", "weekly_hours": [{"day": "mon", "windows": [
                window("9:00", "12:00")]}]}),
            "event_types[15min].weekly_hours[0].windows[0].start",
        ),
        (
            json!({"slug": "15min", "weekly_hours": [
                {"day": "mon", "windows": [window("09:00", "12:00")]},
                {"day": "mon", "windows": [window("13:00", "14:00")]}]}),
            "event_types[15min].weekly_hours[1].day",
        ),
        (
            json!({"slug": "15min", "weekly_hours": []}),
            "event_types[15min].weekly_hours",
        ),
        (
            json!({"slug": "15min", "frequency_limits": [{"count": 0, "period": "day"}]}),
            "event_types[15min].frequency_limits[0].count",
        ),
        (
            json!({"slug": "15min", "frequency_limits": [
                {"count": 1, "period": "day"}, {"count": 2, "period": "day"}]}),
            "event_types[15min].frequency_limits[1].period",
        ),
        (
            json!({"slug": "15min", "location_type": "link"}),
            "event_types[15min].location_value",
        ),
        (
            json!({"slug": "15min", "location_type": "link", "location_value": "javascript:alert(1)"}),
            "event_types[15min].location_value",
        ),
        (
            json!({"slug": "15min", "description": "[x](javascript:alert(1))"}),
            "event_types[15min].description",
        ),
        (json!({"slug": "new-one"}), "event_types[new-one].title"),
        // The new form's location is a link, which needs a value.
        (
            json!({"slug": "new-two", "title": "T"}),
            "event_types[new-two].location_value",
        ),
        (json!({"slug": "15min", "colour": "red"}), "(document)"),
        (
            json!({"slug": "15min", "visibility": "secret"}),
            "(document)",
        ),
        (json!({"slug": "15min", "duration_min": null}), "(document)"),
    ];
    for (item, path) in cases {
        refused(&env, &et_doc(json!([item]), None), path).await;
    }
    refused(
        &env,
        &et_doc(json!([{"slug": "15min"}, {"slug": "15min"}]), None),
        "event_types[15min]",
    )
    .await;
    refused(&env, &et_doc(json!([]), Some("purge")), "(document)").await;
}

#[tokio::test]
async fn a_dry_run_reports_what_a_real_run_does_and_writes_nothing() {
    let env = ApplyEnv::new().await;
    insert_personal_et(&env.pool, &env.admin_id, "old").await;
    let json = et_doc(json!([consultation()]), Some("disable"));

    let before = env.snapshot().await;
    let dry = env.apply(&json, true).await.unwrap();
    assert!(dry.dry_run);
    assert_eq!(env.snapshot().await, before, "the dry run wrote");

    let real = env.apply(&json, false).await.unwrap();
    assert!(!real.dry_run);
    assert_eq!(dry.changed, real.changed);
    assert_eq!(dry.unmanaged, real.unmanaged);
    assert_eq!(
        paths(&real),
        ["event_types[15min]", "event_types[old].enabled"]
    );
}

#[tokio::test]
async fn an_exported_section_applies_with_no_change() {
    let env = ApplyEnv::new().await;
    insert_personal_et(&env.pool, &env.admin_id, "hand-made").await;
    let mut item = consultation();
    item["timezone"] = json!("Europe/Berlin");
    item["weekly_hours"] = json!([{"day": "tue", "windows": [{"start": "10:00", "end": "11:00"}]}]);
    env.apply(&et_doc(json!([item]), None), false)
        .await
        .unwrap();

    let exported = env.export().await.unwrap();
    let event_types = exported.event_types.unwrap();
    assert_eq!(event_types.items.len(), 2);
    let json = doc(json!({ "event_types": serde_json::to_value(&event_types).unwrap() }));
    let before = env.snapshot().await;
    let report = env.apply(&json, false).await.unwrap();
    assert!(report.changed.is_empty(), "{:?}", report.changed);
    assert_eq!(env.snapshot().await, before);

    // The consultation follows the profile's hours: exported as null.
    let mut follows = consultation();
    follows["slug"] = json!("follows");
    env.apply(&et_doc(json!([follows]), None), false)
        .await
        .unwrap();
    let exported = env.export().await.unwrap().event_types.unwrap();
    let item = exported.items.iter().find(|i| i.slug == "follows").unwrap();
    assert_eq!(item.weekly_hours, Some(None));
    assert_eq!(item.timezone, Some(None));
}

// --- TON price ---

/// The paid consultation of book.ilia.ae: the free one at $500 an hour.
fn paid() -> Value {
    let mut item = consultation();
    item["slug"] = json!("cto-60");
    item["title"] = json!("CTO Consultation");
    item["duration_min"] = json!(60);
    item["min_notice_min"] = json!(1440);
    item["frequency_limits"] = json!([]);
    item["ton_price"] = json!({"usd": "500.00", "window_min": null, "enabled": true});
    item
}

/// The `ton_prices` row of `slug`: currency, nanotons, cents, window,
/// enabled.
type PriceCols = (String, Option<i64>, Option<i64>, Option<i64>, i64);

async fn price(env: &ApplyEnv, slug: &str) -> Option<PriceCols> {
    sqlx::query_as(
        "SELECT p.currency, p.price_nano, p.price_usd_cents, p.window_min, p.enabled
         FROM ton_prices p JOIN event_types et ON et.id = p.event_type_id
         WHERE et.slug = ? AND et.team_id IS NULL",
    )
    .bind(slug)
    .fetch_optional(&env.pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn a_ton_price_is_created_changed_switched_off_and_removed() {
    let env = ApplyEnv::new().await;
    let json = et_doc(json!([consultation(), paid()]), None);
    let report = env.apply(&json, false).await.unwrap();
    assert_eq!(
        paths(&report),
        ["event_types[15min]", "event_types[cto-60]"]
    );
    assert_eq!(price(&env, "15min").await, None);
    assert_eq!(
        price(&env, "cto-60").await,
        Some(("USD".into(), None, Some(50_000), None, 1))
    );
    // The item's Telegram mode wins over the one a new price sets.
    assert_eq!(
        telegram_mode(&env, "cto-60").await.as_deref(),
        Some("optional")
    );
    let applied = env.snapshot().await;
    let again = env.apply(&json, false).await.unwrap();
    assert!(again.changed.is_empty(), "{:?}", again.changed);
    assert_eq!(
        env.snapshot().await,
        applied,
        "a run with nothing to do wrote"
    );

    // In TON, with a window of its own; window_min and enabled left out
    // are the defaults, so the short form states the same price.
    let with = |price: Value| et_doc(json!([{"slug": "cto-60", "ton_price": price}]), None);
    let report = env
        .apply(&with(json!({"ton": "420.50", "window_min": 20})), false)
        .await
        .unwrap();
    assert_eq!(
        report.changed,
        vec![Change {
            path: "event_types[cto-60].ton_price".into(),
            before: json!({"usd": "500.00", "window_min": null, "enabled": true}),
            after: json!({"ton": "420.50", "window_min": 20, "enabled": true}),
        }]
    );
    assert_eq!(
        price(&env, "cto-60").await,
        Some(("TON".into(), Some(420_500_000_000), None, Some(20), 1))
    );
    assert!(env
        .apply(
            &with(json!({"ton": "420.50", "window_min": 20, "enabled": true})),
            false
        )
        .await
        .unwrap()
        .changed
        .is_empty());

    // Switched off: kept, the type books free (the form's unchecked box).
    let report = env
        .apply(
            &with(json!({"ton": "420.50", "window_min": 20, "enabled": false})),
            false,
        )
        .await
        .unwrap();
    assert_eq!(paths(&report), ["event_types[cto-60].ton_price"]);
    assert_eq!(
        price(&env, "cto-60").await,
        Some(("TON".into(), Some(420_500_000_000), None, Some(20), 0))
    );

    // null: free again, the row is gone; the Telegram mode stays.
    let report = env.apply(&with(Value::Null), false).await.unwrap();
    let change = change(&report, "event_types[cto-60].ton_price");
    assert_eq!(change.after, Value::Null);
    assert_eq!(price(&env, "cto-60").await, None);
    assert_eq!(
        telegram_mode(&env, "cto-60").await.as_deref(),
        Some("optional")
    );
    assert!(env
        .apply(&with(Value::Null), false)
        .await
        .unwrap()
        .changed
        .is_empty());
}

/// A price switched on asks for the guest's Telegram on a type that has no
/// mode chosen, as on the form: reported when the item gives no mode,
/// overridden when it gives one.
#[tokio::test]
async fn a_new_price_sets_the_telegram_mode_the_form_would() {
    let env = ApplyEnv::new().await;
    insert_personal_et(&env.pool, &env.admin_id, "plain").await;
    insert_personal_et(&env.pool, &env.admin_id, "quiet").await;
    let usd = json!({"usd": "40.00"});
    let report = env
        .apply(
            &et_doc(
                json!([{"slug": "plain", "ton_price": usd},
                       {"slug": "quiet", "ton_price": usd, "telegram_guest_mode": "off"}]),
                None,
            ),
            false,
        )
        .await
        .unwrap();
    assert_eq!(
        paths(&report),
        [
            "event_types[plain].ton_price",
            "event_types[plain].telegram_guest_mode",
            "event_types[quiet].ton_price"
        ]
    );
    let mode = change(&report, "event_types[plain].telegram_guest_mode");
    assert_eq!(
        (&mode.before, &mode.after),
        (&json!("off"), &json!("required"))
    );
    assert_eq!(
        telegram_mode(&env, "plain").await.as_deref(),
        Some("required")
    );
    assert_eq!(telegram_mode(&env, "quiet").await.as_deref(), Some("off"));
    let again = env
        .apply(
            &et_doc(
                json!([{"slug": "plain", "ton_price": usd},
                       {"slug": "quiet", "ton_price": usd, "telegram_guest_mode": "off"}]),
                None,
            ),
            false,
        )
        .await
        .unwrap();
    assert!(again.changed.is_empty(), "{:?}", again.changed);
}

/// A price created switched off sets no Telegram mode, and an item's `off`
/// that is already in force writes none either: the type has no
/// `telegram_event_types` row, which reads as `off` (the form's default).
/// A check of the stored mode must read a missing row as `off` (the
/// Molecule scenario `calrs_ton` of ilia-booking does).
#[tokio::test]
async fn a_switched_off_price_with_telegram_off_leaves_no_mode_row() {
    let env = ApplyEnv::new().await;
    let mut item = paid();
    item["slug"] = json!("paused");
    item["ton_price"] = json!({"usd": "500.00", "window_min": null, "enabled": false});
    item["telegram_guest_mode"] = json!("off");
    let json = et_doc(json!([item]), None);
    let report = env.apply(&json, false).await.unwrap();
    assert_eq!(paths(&report), ["event_types[paused]"]);
    assert_eq!(
        price(&env, "paused").await,
        Some(("USD".into(), None, Some(50_000), None, 0))
    );
    assert_eq!(telegram_mode(&env, "paused").await, None);
    let exported = env.export().await.unwrap().event_types.unwrap();
    let paused = exported.items.iter().find(|i| i.slug == "paused").unwrap();
    assert_eq!(
        serde_json::to_value(paused.telegram_guest_mode).unwrap(),
        json!("off")
    );
    let again = env.apply(&json, false).await.unwrap();
    assert!(again.changed.is_empty(), "{:?}", again.changed);
    assert_eq!(telegram_mode(&env, "paused").await, None);
}

/// The price section's own refusals (`validate_price_form`), the spelling
/// of the stored price, and the object's shape.
#[tokio::test]
async fn a_ton_price_the_form_refuses_or_would_respell_is_refused() {
    let env = ApplyEnv::new().await;
    env.apply(&et_doc(json!([paid()]), None), false)
        .await
        .unwrap();
    let cases: Vec<(Value, &str)> = vec![
        (json!({"usd": "500"}), "event_types[cto-60].ton_price.usd"),
        (json!({"usd": "500.5"}), "event_types[cto-60].ton_price.usd"),
        (json!({"ton": "100"}), "event_types[cto-60].ton_price.ton"),
        (
            json!({"usd": " 500.00"}),
            "event_types[cto-60].ton_price.usd",
        ),
        (json!({"usd": ""}), "event_types[cto-60].ton_price.usd"),
        (json!({"usd": "0.50"}), "event_types[cto-60].ton_price"),
        (json!({"usd": "10000.01"}), "event_types[cto-60].ton_price"),
        (json!({"usd": "5e2"}), "event_types[cto-60].ton_price"),
        (json!({"ton": "0.005"}), "event_types[cto-60].ton_price"),
        (json!({"ton": "100000.01"}), "event_types[cto-60].ton_price"),
        (
            json!({"usd": "500.00", "window_min": 5}),
            "event_types[cto-60].ton_price",
        ),
        (
            json!({"usd": "500.00", "window_min": 241}),
            "event_types[cto-60].ton_price",
        ),
        (
            json!({"usd": "500.00", "ton": "100.00"}),
            "event_types[cto-60].ton_price",
        ),
        (json!({}), "event_types[cto-60].ton_price"),
        (json!({"window_min": 30}), "event_types[cto-60].ton_price"),
        (json!({"usd": 500}), "(document)"),
        (json!({"usd": null}), "(document)"),
        (json!({"usd": "500.00", "currency": "USD"}), "(document)"),
        (json!("500.00"), "(document)"),
    ];
    for (value, path) in cases {
        let error = refused(
            &env,
            &et_doc(
                json!([{"slug": "cto-60", "ton_price": value.clone()}]),
                None,
            ),
            path,
        )
        .await;
        if path.ends_with(".usd") && value["usd"] == json!("500") {
            assert!(
                error.to_string().contains(r#"spell it "500.00""#),
                "{error}"
            );
        }
    }
    // The form's own message, in English.
    let error = refused(
        &env,
        &et_doc(
            json!([{"slug": "cto-60", "ton_price": {"usd": "0.50"}}]),
            None,
        ),
        "event_types[cto-60].ton_price",
    )
    .await;
    assert!(
        error.to_string().contains("from 1 to 10000"),
        "not the form's message: {error}"
    );
    // A new type is checked the same way, before anything is written.
    let mut new = paid();
    new["slug"] = json!("cto-90");
    new["ton_price"] = json!({"ton": "12.345"});
    refused(
        &env,
        &et_doc(json!([new]), None),
        "event_types[cto-90].ton_price",
    )
    .await;
}

#[tokio::test]
async fn a_priced_dry_run_reports_what_a_real_run_does_and_writes_nothing() {
    let env = ApplyEnv::new().await;
    insert_personal_et(&env.pool, &env.admin_id, "plain").await;
    let json = et_doc(
        json!([paid(), {"slug": "plain", "ton_price": {"ton": "25.00", "window_min": 60}}]),
        None,
    );
    let before = env.snapshot().await;
    let dry = env.apply(&json, true).await.unwrap();
    assert_eq!(env.snapshot().await, before, "the dry run wrote");
    let real = env.apply(&json, false).await.unwrap();
    assert_eq!(dry.changed, real.changed);
    assert_eq!(
        paths(&real),
        [
            "event_types[cto-60]",
            "event_types[plain].ton_price",
            "event_types[plain].telegram_guest_mode"
        ]
    );
}

#[tokio::test]
async fn exported_prices_apply_with_no_change() {
    let env = ApplyEnv::new().await;
    let mut off = paid();
    off["slug"] = json!("paused");
    off["ton_price"] = json!({"ton": "100.00", "window_min": 45, "enabled": false});
    env.apply(&et_doc(json!([consultation(), paid(), off]), None), false)
        .await
        .unwrap();
    // A price set through the web form, not by apply.
    let hand = insert_personal_et(&env.pool, &env.admin_id, "hand-made").await;
    sqlx::query(
        "INSERT INTO ton_prices (event_type_id, currency, price_nano, window_min, enabled, updated_at)
         VALUES (?, 'TON', 12340000000, NULL, 1, '2026-11-12T00:00:00Z')",
    )
    .bind(&hand)
    .execute(&env.pool)
    .await
    .unwrap();

    let exported = env.export().await.unwrap().event_types.unwrap();
    let by_slug = |slug: &str| {
        exported
            .items
            .iter()
            .find(|i| i.slug == slug)
            .map(|i| serde_json::to_value(&i.ton_price).unwrap())
            .unwrap()
    };
    assert_eq!(by_slug("15min"), Value::Null);
    assert_eq!(
        by_slug("cto-60"),
        json!({"usd": "500.00", "window_min": null, "enabled": true})
    );
    assert_eq!(
        by_slug("paused"),
        json!({"ton": "100.00", "window_min": 45, "enabled": false})
    );
    assert_eq!(
        by_slug("hand-made"),
        json!({"ton": "12.34", "window_min": null, "enabled": true})
    );
    let json = doc(json!({ "event_types": serde_json::to_value(&exported).unwrap() }));
    let before = env.snapshot().await;
    let report = env.apply(&json, false).await.unwrap();
    assert!(report.changed.is_empty(), "{:?}", report.changed);
    assert_eq!(env.snapshot().await, before);
}

/// ilia fork: `ton_price: {ladder: true, window_min, enabled}` prices by
/// the site's demand ladder; it round-trips through the export, a second
/// run changes nothing, and malformed forms are refused before anything is
/// written.
#[tokio::test]
async fn a_ladder_price_applies_exports_and_round_trips() {
    let env = ApplyEnv::new().await;
    let mut item = paid();
    item["ton_price"] = json!({"ladder": true, "window_min": 20, "enabled": true});
    let json = et_doc(json!([item]), None);
    env.apply(&json, false).await.unwrap();
    let mode: String = sqlx::query_scalar(
        "SELECT p.mode FROM ton_prices p JOIN event_types et ON et.id = p.event_type_id
         WHERE et.slug = 'cto-60'",
    )
    .fetch_one(&env.pool)
    .await
    .unwrap();
    assert_eq!(mode, "ladder");
    assert_eq!(
        price(&env, "cto-60").await,
        Some(("TON".into(), None, None, Some(20), 1))
    );
    assert!(env.apply(&json, false).await.unwrap().changed.is_empty());

    // From a fixed price to the ladder and back is one change each.
    let with = |price: Value| et_doc(json!([{"slug": "cto-60", "ton_price": price}]), None);
    let report = env
        .apply(&with(json!({"ton": "100.00"})), false)
        .await
        .unwrap();
    assert_eq!(
        report.changed,
        vec![Change {
            path: "event_types[cto-60].ton_price".into(),
            before: json!({"ladder": true, "window_min": 20, "enabled": true}),
            after: json!({"ton": "100.00", "window_min": null, "enabled": true}),
        }]
    );
    env.apply(&with(json!({"ladder": true, "enabled": false})), false)
        .await
        .unwrap();
    assert_eq!(
        price(&env, "cto-60").await,
        Some(("TON".into(), None, None, None, 0))
    );

    let exported = env.export().await.unwrap().event_types.unwrap();
    let item = exported.items.iter().find(|i| i.slug == "cto-60").unwrap();
    assert_eq!(
        serde_json::to_value(&item.ton_price).unwrap(),
        json!({"ladder": true, "window_min": null, "enabled": false})
    );
    let json = doc(json!({ "event_types": serde_json::to_value(&exported).unwrap() }));
    let before = env.snapshot().await;
    assert!(env.apply(&json, false).await.unwrap().changed.is_empty());
    assert_eq!(env.snapshot().await, before);

    for (value, path) in [
        (
            json!({"ladder": false}),
            "event_types[cto-60].ton_price.ladder",
        ),
        (
            json!({"ladder": true, "ton": "100.00"}),
            "event_types[cto-60].ton_price",
        ),
        (
            json!({"ladder": true, "usd": "500.00"}),
            "event_types[cto-60].ton_price",
        ),
        (
            json!({"ladder": true, "window_min": 5}),
            "event_types[cto-60].ton_price",
        ),
        (json!({"ladder": null}), "(document)"),
        (json!({"ladder": "yes"}), "(document)"),
    ] {
        refused(
            &env,
            &et_doc(json!([{"slug": "cto-60", "ton_price": value}]), None),
            path,
        )
        .await;
    }
}

// --- Sources ---

/// A Kerio-like source of the admin with three calendars, one of them
/// synced off, and a cached event in the synced "Holidays".
async fn seed_source(env: &ApplyEnv) {
    sqlx::query(
        "INSERT INTO caldav_sources (id, account_id, name, url, username)
         VALUES ('src', ?, 'Kerio', 'https://mail.example.test/caldav/', 'aa@example.test')",
    )
    .bind(&env.account_id)
    .execute(&env.pool)
    .await
    .unwrap();
    for (id, name, href, synced) in [
        ("cal-work", Some("Work"), "/caldav/users/aa/Calendar/", 1),
        ("cal-hol", Some("Holidays"), "/caldav/users/aa/Holidays/", 1),
        ("cal-old", None, "/caldav/users/aa/Old/", 0),
        (
            "cal-evil",
            Some("Elsewhere"),
            "https://evil.example/cal/",
            0,
        ),
    ] {
        sqlx::query(
            "INSERT INTO calendars (id, source_id, href, display_name, sync_enabled)
             VALUES (?, 'src', ?, ?, ?)",
        )
        .bind(id)
        .bind(href)
        .bind(name)
        .bind(synced)
        .execute(&env.pool)
        .await
        .unwrap();
    }
    sqlx::query(
        "INSERT INTO events (id, calendar_id, uid, start_at, end_at)
         VALUES ('ev1', 'cal-hol', 'ev-uid', '20991201T100000', '20991201T110000')",
    )
    .execute(&env.pool)
    .await
    .unwrap();
}

fn source(extra: Value) -> String {
    let mut source =
        json!({"url": "https://mail.example.test/caldav/", "username": "aa@example.test"});
    for (key, value) in extra.as_object().unwrap() {
        source[key] = value.clone();
    }
    doc(json!({ "sources": [source] }))
}

async fn synced(env: &ApplyEnv) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT id FROM calendars WHERE source_id = 'src' AND sync_enabled = 1 ORDER BY id",
    )
    .fetch_all(&env.pool)
    .await
    .unwrap()
}

async fn write_href(env: &ApplyEnv) -> Option<String> {
    sqlx::query_scalar("SELECT write_calendar_href FROM caldav_sources WHERE id = 'src'")
        .fetch_one(&env.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn sources_set_the_write_calendar_and_the_synced_calendars() {
    let env = ApplyEnv::new().await;
    seed_source(&env).await;
    let json = source(json!({
        "write_calendar": "Work",
        "sync_calendars": ["Work", "/caldav/users/aa/Old/"]
    }));

    let report = env.apply(&json, false).await.unwrap();
    assert_eq!(
        report.changed,
        vec![
            Change {
                path: "sources[0].write_calendar".into(),
                before: Value::Null,
                after: json!("Work"),
            },
            Change {
                path: "sources[0].sync_calendars".into(),
                before: json!(["Holidays", "Work"]),
                after: json!(["/caldav/users/aa/Old/", "Work"]),
            },
        ]
    );
    assert!(!report.restart);
    assert_eq!(
        write_href(&env).await.as_deref(),
        Some("/caldav/users/aa/Calendar/")
    );
    assert_eq!(synced(&env).await, ["cal-old", "cal-work"]);
    let cached: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE calendar_id = 'cal-hol'")
            .fetch_one(&env.pool)
            .await
            .unwrap();
    assert_eq!(cached, 0, "a calendar turned off keeps no events");

    let before = env.snapshot().await;
    assert!(env.apply(&json, false).await.unwrap().changed.is_empty());
    assert_eq!(env.snapshot().await, before);

    // null: no write-back.
    let report = env
        .apply(&source(json!({"write_calendar": null})), false)
        .await
        .unwrap();
    assert_eq!(paths(&report), ["sources[0].write_calendar"]);
    assert_eq!(write_href(&env).await, None);

    // The export applies with no change.
    let exported = env.export().await.unwrap().sources.unwrap();
    assert_eq!(exported.len(), 1);
    assert_eq!(exported[0].write_calendar, Some(None));
    let json = doc(json!({ "sources": serde_json::to_value(&exported).unwrap() }));
    assert!(env.apply(&json, false).await.unwrap().changed.is_empty());
}

#[tokio::test]
async fn sources_refuse_what_the_local_rows_cannot_back() {
    let env = ApplyEnv::new().await;
    seed_source(&env).await;

    let error = refused(
        &env,
        &source(json!({"write_calendar": "Kalender"})),
        "sources[0].write_calendar",
    )
    .await;
    // The names the source has, so the role can be fixed in one go.
    for name in ["\"Work\"", "\"Holidays\"", "\"Elsewhere\""] {
        assert!(error.to_string().contains(name), "{error}");
    }
    refused(
        &env,
        &source(json!({"write_calendar": "Elsewhere"})),
        "sources[0].write_calendar",
    )
    .await;
    refused(
        &env,
        &source(json!({"sync_calendars": ["Work", "Nope"]})),
        "sources[0].sync_calendars[1]",
    )
    .await;
    refused(
        &env,
        &source(json!({"sync_calendars": []})),
        "sources[0].sync_calendars",
    )
    .await;
    refused(
        &env,
        &source(json!({"sync_calendars": ["Work", "Work"]})),
        "sources[0].sync_calendars[1]",
    )
    .await;
    refused(
        &env,
        &source(json!({"write_calendar": " Work"})),
        "sources[0].write_calendar",
    )
    .await;
    refused(
        &env,
        &doc(
            json!({"sources": [{"url": "https://other.example/", "username": "aa@example.test"}]}),
        ),
        "sources[0]",
    )
    .await;
    refused(&env, &source(json!({"sync_calendars": null})), "(document)").await;
    refused(&env, &source(json!({"password": "x"})), "(document)").await;

    // Two calendars of one name cannot be told apart.
    sqlx::query(
        "INSERT INTO calendars (id, source_id, href, display_name) VALUES ('cal-work2', 'src', '/w2/', 'Work')",
    )
    .execute(&env.pool)
    .await
    .unwrap();
    let error = refused(
        &env,
        &source(json!({"write_calendar": "Work"})),
        "sources[0].write_calendar",
    )
    .await;
    assert!(error.to_string().contains("2 calendars"), "{error}");
}

#[tokio::test]
async fn event_types_follow_the_profile_set_in_the_same_run() {
    let env = ApplyEnv::new().await;
    let json = doc(json!({
        "profile": {
            "timezone": "Asia/Almaty",
            "weekly_hours": [
                {"day": "mon", "windows": [{"start": "10:00", "end": "19:00"}]},
                {"day": "fri", "windows": [{"start": "10:00", "end": "19:00"}]}
            ]
        },
        "event_types": {"items": [consultation()]}
    }));
    let report = env.apply(&json, false).await.unwrap();
    assert!(
        paths(&report).contains(&"event_types[15min]"),
        "{:?}",
        report.changed
    );
    assert_eq!(
        rules(&env, "15min").await,
        vec![
            (1, "10:00".into(), "19:00".into()),
            (5, "10:00".into(), "19:00".into())
        ]
    );
    assert_eq!(
        column::<Option<String>>(&env, "15min", "timezone").await,
        None
    );
    let before = env.snapshot().await;
    let again = env.apply(&json, false).await.unwrap();
    assert!(again.changed.is_empty(), "{:?}", again.changed);
    assert_eq!(env.snapshot().await, before);
}
