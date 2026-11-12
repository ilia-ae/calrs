//! ilia fork: `config apply`, the `profile` section (Settings → Profile and
//! the avatar; docs/apply-schema.md). Shared fixtures: `apply_support`.

use super::apply_support::*;
use super::*;
use crate::web::apply::{self, ApplyError, Ctx, Document, Report};
use axum::extract::State;
use axum::response::IntoResponse;

const JPEG: &[u8] = b"\xFF\xD8\xFF\xE0\x00\x10JFIF\x00 the first image";
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR the second image";

/// A document with `profile` (a JSON object) as its profile.
fn doc(profile: &str) -> String {
    format!(
        r#"{{"format":"{}","user":"{ADMIN_EMAIL}","profile":{profile}}}"#,
        apply::FORMAT
    )
}

/// `bytes` written to `name` in the env's `managed/` directory: its path
/// and SHA-256.
fn image(env: &ApplyEnv, name: &str, bytes: &[u8]) -> (String, String) {
    let dir = env.dir.join("managed");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    let sha = hex::encode(<sha2::Sha256 as sha2::Digest>::digest(bytes));
    (path.to_str().unwrap().to_string(), sha)
}

fn avatar(file: &str, sha: &str) -> String {
    format!(r#"{{"file":"{file}","sha256":"{sha}"}}"#)
}

/// Every key of the profile, with `avatar` (a JSON value).
fn full_profile(avatar: &str) -> String {
    format!(
        r#"{{"name":"Ilia Arestov","username":"ilia","title":"Fractional CTO",
        "bio":"Fractional CTO in [Dubai](https://ilia.ae).\n\nWrite to <mail@ilia.ae>.",
        "booking_email":"book@ilia.ae","timezone":"Asia/Almaty","language":"ru",
        "public_page_language":"host","week_start":"sun","allow_dynamic_group":false,
        "lend_resource_write":true,
        "weekly_hours":[{{"day":"mon","windows":[{{"start":"10:00","end":"13:00"}},{{"start":"14:00","end":"19:00"}}]}},
                        {{"day":"fri","windows":[{{"start":"10:00","end":"16:00"}}]}}],
        "avatar":{avatar}}}"#
    )
}

fn paths(report: &Report) -> Vec<&str> {
    report.changed.iter().map(|c| c.path.as_str()).collect()
}

fn invalid_path(result: Result<Report, ApplyError>) -> String {
    match result {
        Err(ApplyError::Invalid { path, .. }) => path,
        Err(other) => panic!("expected an invalid document, got {other}"),
        Ok(report) => panic!("expected an invalid document, got {report:?}"),
    }
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

type ProfileRow = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    String,
    Option<String>,
    bool,
    bool,
    Option<String>,
);

async fn profile_row(env: &ApplyEnv) -> ProfileRow {
    sqlx::query_as(
        "SELECT name, username, title, bio, booking_email, timezone, language, \
         public_page_language, week_start, allow_dynamic_group, lend_resource_write, avatar_path \
         FROM users WHERE id = ?",
    )
    .bind(&env.admin_id)
    .fetch_one(&env.pool)
    .await
    .unwrap()
}

async fn rules(env: &ApplyEnv) -> Vec<(i32, String, String)> {
    sqlx::query_as(
        "SELECT day_of_week, start_time, end_time FROM user_availability_rules \
         WHERE user_id = ? ORDER BY day_of_week, start_time",
    )
    .bind(&env.admin_id)
    .fetch_all(&env.pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn every_field_is_written_and_a_second_run_changes_nothing() {
    let env = ApplyEnv::new().await;
    let (file, sha) = image(&env, "avatar", JPEG);
    let json = doc(&full_profile(&avatar(&file, &sha)));

    let report = env.apply(&json, false).await.unwrap();
    assert_eq!(
        paths(&report),
        [
            "profile.name",
            "profile.username",
            "profile.title",
            "profile.bio",
            "profile.booking_email",
            "profile.timezone",
            "profile.language",
            "profile.public_page_language",
            "profile.week_start",
            "profile.allow_dynamic_group",
            "profile.lend_resource_write",
            "profile.weekly_hours",
            "profile.avatar",
        ]
    );
    assert!(!report.dry_run);
    assert!(
        !report.restart,
        "nothing in the profile is cached at startup"
    );
    let avatar_change = report.changed.last().unwrap();
    assert_eq!(avatar_change.before, serde_json::Value::Null);
    assert_eq!(
        avatar_change.after,
        serde_json::json!(format!("sha256:{sha}"))
    );

    let jpg = format!("{}.jpg", env.admin_id);
    assert_eq!(
        profile_row(&env).await,
        (
            "Ilia Arestov".to_string(),
            Some("ilia".to_string()),
            Some("Fractional CTO".to_string()),
            Some(
                "Fractional CTO in [Dubai](https://ilia.ae).\n\nWrite to <mail@ilia.ae>."
                    .to_string()
            ),
            Some("book@ilia.ae".to_string()),
            "Asia/Almaty".to_string(),
            Some("ru".to_string()),
            "host".to_string(),
            Some("sun".to_string()),
            false,
            true,
            Some(jpg.clone()),
        )
    );
    assert_eq!(
        rules(&env).await,
        [
            (1, "10:00".to_string(), "13:00".to_string()),
            (1, "14:00".to_string(), "19:00".to_string()),
            (5, "10:00".to_string(), "16:00".to_string()),
        ]
    );
    let account_name: String = sqlx::query_scalar("SELECT name FROM accounts WHERE id = ?")
        .bind(&env.account_id)
        .fetch_one(&env.pool)
        .await
        .unwrap();
    assert_eq!(
        account_name, "Ilia Arestov",
        "the form also renames the account"
    );
    assert_eq!(avatar_files(&env), [jpg.as_str()]);
    assert_eq!(
        std::fs::read(env.dir.join("avatars").join(&jpg)).unwrap(),
        JPEG
    );
    assert!(leftovers(&env).is_empty());

    // Nothing differs now: no change, no post (not even `updated_at`).
    let before = env.snapshot().await;
    let again = env.apply(&json, false).await.unwrap();
    assert!(again.changed.is_empty(), "{:?}", again.changed);
    assert!(!again.restart);
    assert_eq!(env.snapshot().await, before);
    assert_eq!(avatar_files(&env), [jpg]);
    assert!(leftovers(&env).is_empty());
}

#[tokio::test]
async fn a_dry_run_reports_the_real_run_and_writes_nothing() {
    let env = ApplyEnv::new().await;
    let (file, sha) = image(&env, "avatar", JPEG);
    let json = doc(&full_profile(&avatar(&file, &sha)));
    let before = env.snapshot().await;

    let dry = env.apply(&json, true).await.unwrap();
    assert!(dry.dry_run);
    assert_eq!(env.snapshot().await, before);
    assert!(avatar_files(&env).is_empty());
    assert!(leftovers(&env).is_empty());

    let real = env.apply(&json, false).await.unwrap();
    assert_eq!(dry.changed, real.changed);
    assert_eq!(dry.restart, real.restart);
}

#[tokio::test]
async fn a_partial_document_leaves_the_other_fields_alone() {
    let env = ApplyEnv::new().await;
    let (file, sha) = image(&env, "avatar", JPEG);
    env.apply(&doc(&full_profile(&avatar(&file, &sha))), false)
        .await
        .unwrap();
    let row = profile_row(&env).await;
    let hours = rules(&env).await;

    let report = env.apply(&doc(r#"{"title":"CTO"}"#), false).await.unwrap();
    assert_eq!(paths(&report), ["profile.title"]);
    assert_eq!(
        report.changed[0].before,
        serde_json::json!("Fractional CTO")
    );
    assert_eq!(report.changed[0].after, serde_json::json!("CTO"));
    let mut expected = row;
    expected.2 = Some("CTO".to_string());
    assert_eq!(profile_row(&env).await, expected);
    assert_eq!(rules(&env).await, hours);
    assert_eq!(avatar_files(&env), [format!("{}.jpg", env.admin_id)]);

    // An empty profile changes nothing at all.
    let before = env.snapshot().await;
    assert!(env
        .apply(&doc("{}"), false)
        .await
        .unwrap()
        .changed
        .is_empty());
    assert_eq!(env.snapshot().await, before);
}

#[tokio::test]
async fn null_clears_and_inherits() {
    let env = ApplyEnv::new().await;
    env.apply(&doc(&full_profile("null")), false).await.unwrap();

    let report = env
        .apply(
            &doc(r#"{"title":null,"bio":null,"booking_email":null,"language":null,"week_start":null}"#),
            false,
        )
        .await
        .unwrap();
    assert_eq!(
        paths(&report),
        [
            "profile.title",
            "profile.bio",
            "profile.booking_email",
            "profile.language",
            "profile.week_start",
        ]
    );
    assert!(report.changed.iter().all(|c| c.after.is_null()));
    let row = profile_row(&env).await;
    assert_eq!(
        (row.2, row.3, row.4, row.6, row.8),
        (None, None, None, None, None)
    );
    assert_eq!(row.0, "Ilia Arestov", "the rest stays");
}

#[tokio::test]
async fn non_canonical_input_is_refused_and_nothing_is_written() {
    let env = ApplyEnv::new().await;
    let (svg, svg_sha) = image(
        &env,
        "avatar.svg",
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    );
    let mut big_bytes = PNG.to_vec();
    big_bytes.resize(2 * 1024 * 1024 + 1, 0);
    let (big, big_sha) = image(&env, "big.png", &big_bytes);
    let (jpeg, jpeg_sha) = image(&env, "avatar.jpg", JPEG);
    let long_name = "n".repeat(256);
    let hours =
        |windows: &str| format!(r#"{{"weekly_hours":[{{"day":"mon","windows":[{windows}]}}]}}"#);

    let cases: Vec<(String, &str)> = vec![
        (r#"{"username":"Ilia"}"#.into(), "profile.username"),
        (r#"{"username":"ilia.a"}"#.into(), "profile.username"),
        (r#"{"username":"i"}"#.into(), "profile.username"),
        (r#"{"username":" ilia"}"#.into(), "profile.username"),
        (r#"{"name":" Ilia"}"#.into(), "profile.name"),
        (r#"{"name":""}"#.into(), "profile.name"),
        (format!(r#"{{"name":"{long_name}"}}"#), "profile.name"),
        (r#"{"title":""}"#.into(), "profile.title"),
        (r#"{"title":"CTO "}"#.into(), "profile.title"),
        (r#"{"bio":""}"#.into(), "profile.bio"),
        (
            r#"{"bio":"See [me](javascript:alert(1))"}"#.into(),
            "profile.bio",
        ),
        (
            r#"{"bio":"See [me](data:text/html,x)"}"#.into(),
            "profile.bio",
        ),
        (r#"{"booking_email":""}"#.into(), "profile.booking_email"),
        (r#"{"timezone":"Asia/Almty"}"#.into(), "profile.timezone"),
        (r#"{"timezone":" Asia/Almaty"}"#.into(), "profile.timezone"),
        (r#"{"language":"xx"}"#.into(), "profile.language"),
        (r#"{"language":"RU"}"#.into(), "profile.language"),
        (r#"{"language":"auto"}"#.into(), "profile.language"),
        (r#"{"language":""}"#.into(), "profile.language"),
        (
            hours(r#"{"start":"09:00","end":"12:00"},{"start":"11:00","end":"13:00"}"#),
            "profile.weekly_hours[0].windows",
        ),
        (
            hours(r#"{"start":"13:00","end":"14:00"},{"start":"09:00","end":"10:00"}"#),
            "profile.weekly_hours[0].windows",
        ),
        (
            hours(r#"{"start":"9:00","end":"12:00"}"#),
            "profile.weekly_hours[0].windows[0].start",
        ),
        (
            hours(r#"{"start":"09:00 ","end":"12:00"}"#),
            "profile.weekly_hours[0].windows[0].start",
        ),
        (
            hours(r#"{"start":"12:00","end":"09:00"}"#),
            "profile.weekly_hours[0].windows[0]",
        ),
        (r#"{"weekly_hours":[]}"#.into(), "profile.weekly_hours"),
        (
            r#"{"weekly_hours":[{"day":"mon","windows":[]}]}"#.into(),
            "profile.weekly_hours",
        ),
        (
            r#"{"weekly_hours":[{"day":"mon","windows":[{"start":"09:00","end":"10:00"}]},
                                {"day":"mon","windows":[{"start":"11:00","end":"12:00"}]}]}"#
                .into(),
            "profile.weekly_hours[1].day",
        ),
        (
            format!(r#"{{"avatar":{}}}"#, avatar(&svg, &svg_sha)),
            "profile.avatar.file",
        ),
        (
            format!(r#"{{"avatar":{}}}"#, avatar(&big, &big_sha)),
            "profile.avatar.file",
        ),
        (
            format!(r#"{{"avatar":{}}}"#, avatar(&jpeg, &"0".repeat(64))),
            "profile.avatar.sha256",
        ),
        (
            format!(
                r#"{{"avatar":{}}}"#,
                avatar(&jpeg, &jpeg_sha.to_uppercase())
            ),
            "profile.avatar.sha256",
        ),
        (
            format!(
                r#"{{"avatar":{}}}"#,
                avatar("managed/avatar.jpg", &jpeg_sha)
            ),
            "profile.avatar.file",
        ),
        (
            format!(
                r#"{{"avatar":{}}}"#,
                avatar(env.dir.join("nothing-here").to_str().unwrap(), &jpeg_sha)
            ),
            "profile.avatar.file",
        ),
        // Refused by the strict parser: null where not nullable, an unknown key.
        (r#"{"name":null}"#.into(), "(document)"),
        (r#"{"username":null}"#.into(), "(document)"),
        (r#"{"nickname":"ilia"}"#.into(), "(document)"),
        (r#"{"week_start":"monday"}"#.into(), "(document)"),
        (r#"{"public_page_language":"browser"}"#.into(), "(document)"),
        (r#"{"avatar":{"file":"/x"}}"#.into(), "(document)"),
    ];

    let before = env.snapshot().await;
    for (profile, path) in cases {
        // Each with a valid change next to it: the whole run is refused.
        let profile = profile.replacen('{', r#"{"lend_resource_write":true,"#, 1);
        let result = env.apply(&doc(&profile), false).await;
        assert_eq!(invalid_path(result), path, "{profile}");
        assert_eq!(env.snapshot().await, before, "{profile}");
        assert!(avatar_files(&env).is_empty(), "{profile}");
        assert!(leftovers(&env).is_empty(), "{profile}");
    }
}

/// The web form, posted as the settings page posts it with `overrides`.
async fn post_settings(
    env: &ApplyEnv,
    overrides: &[(&'static str, &str)],
) -> axum::response::Response {
    let mut fields: Vec<(&'static str, String)> = [
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
    ]
    .iter()
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
    let (headers, form) = apply::form(&fields).await.unwrap();
    crate::web::dashboard::settings_save(State(ctx.state.clone()), ctx.auth_user(), headers, form)
        .await
        .into_response()
}

async fn column(env: &ApplyEnv, column: &str) -> Option<String> {
    sqlx::query_scalar(&format!("SELECT {column} FROM users WHERE id = ?"))
        .bind(&env.admin_id)
        .fetch_one(&env.pool)
        .await
        .unwrap()
}

/// Parity: what apply refuses as not canonical, the web form quietly
/// rewrites, so a run that posted it would store something else and every
/// later run would report it again.
#[tokio::test]
async fn the_web_form_rewrites_what_apply_refuses() {
    let cases: &[(&str, &str, &str, &str)] = &[
        // (form field, document key, value, what the form stores)
        ("username", "username", "Ilia", "ilia"),
        ("username", "username", " ilia", "ilia"),
        ("username", "username", "ilia.a", "iliaa"),
        ("name", "name", " Ilia", "Ilia"),
        ("title", "title", "CTO ", "CTO"),
        ("timezone", "timezone", "Asia/Almty", "UTC"),
        ("timezone", "timezone", " Asia/Almaty", "Asia/Almaty"),
        ("language", "language", "xx", "(null)"),
        ("language", "language", "RU", "(null)"),
    ];
    for (field, key, value, stored) in cases {
        let env = ApplyEnv::new().await;
        let response = post_settings(&env, &[(*field, *value)]).await;
        apply::answered("profile", response).await.unwrap();
        let got = column(&env, field).await.unwrap_or_else(|| "(null)".into());
        assert_eq!(
            got.as_str(),
            *stored,
            "the form stores {value:?} as {got:?}"
        );
        assert_ne!(got.as_str(), *value);

        let json = doc(&format!(r#"{{"{key}":{}}}"#, serde_json::json!(value)));
        assert_eq!(
            invalid_path(env.apply(&json, false).await),
            format!("profile.{key}")
        );
    }
}

/// Parity: what the web form refuses, apply refuses too, through the form
/// itself, and nothing is written.
#[tokio::test]
async fn what_the_web_form_refuses_is_refused() {
    let cases: &[(&str, &str)] = &[
        (r#"{"booking_email":"nobody"}"#, "booking_email"),
        (r#"{"booking_email":"me@localhost"}"#, "booking_email"),
        (r#"{"username":"taken"}"#, "username"),
    ];
    for (profile, field) in cases {
        let env = ApplyEnv::new().await;
        insert_role_user(&env.pool, "taken@example.com", "user").await;
        let value: serde_json::Value =
            serde_json::from_str::<serde_json::Value>(profile).unwrap()[field].clone();
        let response = post_settings(&env, &[(*field, value.as_str().unwrap())]).await;
        assert!(
            matches!(
                apply::answered("profile", response).await,
                Err(ApplyError::Refused { .. })
            ),
            "the web form takes {profile}"
        );

        // With a new avatar next to it: the staged file goes too.
        let (file, sha) = image(&env, "avatar", JPEG);
        let profile = profile.replacen('{', &format!(r#"{{"avatar":{},"#, avatar(&file, &sha)), 1);
        let before = env.snapshot().await;
        match env.apply(&doc(&profile), false).await {
            Err(ApplyError::Refused { path, message }) => {
                assert_eq!(path, "profile");
                assert!(message.contains("the web form refused it"), "{message}");
            }
            other => panic!("{profile}: expected a refusal, got {other:?}"),
        }
        assert_eq!(env.snapshot().await, before, "{profile}");
        assert!(avatar_files(&env).is_empty(), "{profile}");
        assert!(leftovers(&env).is_empty(), "{profile}");
    }
}

#[tokio::test]
async fn the_same_image_elsewhere_changes_nothing() {
    let env = ApplyEnv::new().await;
    let (file, sha) = image(&env, "avatar", JPEG);
    env.apply(
        &doc(&format!(r#"{{"avatar":{}}}"#, avatar(&file, &sha))),
        false,
    )
    .await
    .unwrap();
    let (copy, copy_sha) = image(&env, "copy-of-avatar", JPEG);
    assert_eq!(copy_sha, sha);
    let before = env.snapshot().await;
    let report = env
        .apply(
            &doc(&format!(r#"{{"avatar":{}}}"#, avatar(&copy, &sha))),
            false,
        )
        .await
        .unwrap();
    assert!(report.changed.is_empty(), "{:?}", report.changed);
    assert_eq!(env.snapshot().await, before);
}

#[tokio::test]
async fn a_new_image_replaces_the_file_and_the_old_extension_goes() {
    let env = ApplyEnv::new().await;
    let (jpeg, jpeg_sha) = image(&env, "avatar.jpg", JPEG);
    env.apply(
        &doc(&format!(r#"{{"avatar":{}}}"#, avatar(&jpeg, &jpeg_sha))),
        false,
    )
    .await
    .unwrap();
    let (png, png_sha) = image(&env, "avatar.png", PNG);

    let report = env
        .apply(
            &doc(&format!(r#"{{"avatar":{}}}"#, avatar(&png, &png_sha))),
            false,
        )
        .await
        .unwrap();
    assert_eq!(paths(&report), ["profile.avatar"]);
    assert_eq!(
        report.changed[0].before,
        serde_json::json!(format!("sha256:{jpeg_sha}"))
    );
    assert_eq!(
        report.changed[0].after,
        serde_json::json!(format!("sha256:{png_sha}"))
    );
    let name = format!("{}.png", env.admin_id);
    assert_eq!(profile_row(&env).await.11, Some(name.clone()));
    assert_eq!(avatar_files(&env), [name.as_str()]);
    assert_eq!(
        std::fs::read(env.dir.join("avatars").join(&name)).unwrap(),
        PNG
    );

    // Another image of the same type overwrites the same file.
    let other = [PNG, b" again".as_slice()].concat();
    let (png2, png2_sha) = image(&env, "avatar2.png", &other);
    env.apply(
        &doc(&format!(r#"{{"avatar":{}}}"#, avatar(&png2, &png2_sha))),
        false,
    )
    .await
    .unwrap();
    assert_eq!(avatar_files(&env), [name.as_str()]);
    assert_eq!(
        std::fs::read(env.dir.join("avatars").join(&name)).unwrap(),
        other
    );
    assert!(leftovers(&env).is_empty());
}

#[tokio::test]
async fn null_removes_the_avatar() {
    let env = ApplyEnv::new().await;
    let (file, sha) = image(&env, "avatar", JPEG);
    env.apply(
        &doc(&format!(r#"{{"avatar":{}}}"#, avatar(&file, &sha))),
        false,
    )
    .await
    .unwrap();

    // A dry run removes nothing.
    let dry = env.apply(&doc(r#"{"avatar":null}"#), true).await.unwrap();
    assert_eq!(paths(&dry), ["profile.avatar"]);
    assert_eq!(avatar_files(&env).len(), 1);

    let report = env.apply(&doc(r#"{"avatar":null}"#), false).await.unwrap();
    assert_eq!(report.changed, dry.changed);
    assert_eq!(report.changed[0].after, serde_json::Value::Null);
    assert_eq!(profile_row(&env).await.11, None);
    assert!(avatar_files(&env).is_empty());

    let again = env.apply(&doc(r#"{"avatar":null}"#), false).await.unwrap();
    assert!(again.changed.is_empty());
}

#[tokio::test]
async fn a_missing_avatar_file_is_replaced() {
    let env = ApplyEnv::new().await;
    let (file, sha) = image(&env, "avatar", JPEG);
    let json = doc(&format!(r#"{{"avatar":{}}}"#, avatar(&file, &sha)));
    env.apply(&json, false).await.unwrap();
    let served = env
        .dir
        .join("avatars")
        .join(format!("{}.jpg", env.admin_id));
    std::fs::remove_file(&served).unwrap();

    let report = env.apply(&json, false).await.unwrap();
    assert_eq!(
        report.changed[0].before,
        serde_json::json!("(file missing)")
    );
    assert_eq!(std::fs::read(&served).unwrap(), JPEG);
}

#[tokio::test]
async fn the_export_is_the_stored_profile_and_applies_as_no_change() {
    let env = ApplyEnv::new().await;
    // A fresh admin: no username canonical issue, no hours, no avatar.
    let exported = env.export().await.unwrap().profile.unwrap();
    assert_eq!(exported.name.as_deref(), Some(ADMIN_EMAIL));
    assert_eq!(exported.timezone.as_deref(), Some("UTC"));
    assert_eq!(exported.weekly_hours, None, "no hours stored: left out");
    assert_eq!(exported.avatar, Some(None));

    let (file, sha) = image(&env, "avatar", JPEG);
    env.apply(&doc(&full_profile(&avatar(&file, &sha))), false)
        .await
        .unwrap();
    let exported = env.export().await.unwrap().profile.unwrap();
    let wanted: Document = apply::parse(&doc(&full_profile(&avatar(&file, &sha)))).unwrap();
    let mut wanted = wanted.profile.unwrap();
    let stored_avatar = exported.avatar.clone().unwrap().unwrap();
    assert_eq!(stored_avatar.sha256, sha);
    assert!(stored_avatar
        .file
        .ends_with(&format!("avatars/{}.jpg", env.admin_id)));
    wanted.avatar = exported.avatar.clone();
    assert_eq!(exported, wanted);

    let round_trip = Document {
        format: apply::FORMAT.to_string(),
        user: ADMIN_EMAIL.to_string(),
        profile: Some(exported),
        site: None,
        sources: None,
        event_types: None,
    };
    let before = env.snapshot().await;
    let report = env
        .apply(&serde_json::to_string(&round_trip).unwrap(), false)
        .await
        .unwrap();
    assert!(report.changed.is_empty(), "{:?}", report.changed);
    assert_eq!(env.snapshot().await, before);
}

#[tokio::test]
async fn the_export_leaves_out_what_the_document_cannot_state() {
    let env = ApplyEnv::new().await;
    // Written outside the UI: an uppercase username, a padded title, a
    // timezone chrono-tz does not know.
    sqlx::query("UPDATE users SET username = 'Admin', title = ' x ', timezone = 'Mars/Olympus' WHERE id = ?")
        .bind(&env.admin_id)
        .execute(&env.pool)
        .await
        .unwrap();
    let exported = env.export().await.unwrap().profile.unwrap();
    assert_eq!(exported.username, None);
    assert_eq!(exported.title, None);
    assert_eq!(exported.timezone, None);
    assert_eq!(exported.name.as_deref(), Some(ADMIN_EMAIL));
}
