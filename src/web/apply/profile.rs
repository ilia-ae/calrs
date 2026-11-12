//! ilia fork: `config apply`, the `profile` section: Settings → Profile
//! (`dashboard::settings_save`) and the avatar.
//!
//! The profile form is posted whole, as the settings page posts it: every
//! field holds the stored value, or the document's where it gives one, so a
//! key left out keeps what is stored. It is posted only when one of its
//! fields differs from the document.
//!
//! The avatar has no form apply can post inside the transaction: the upload
//! handler writes the file at once. So apply reuses the upload's own checks
//! (`detect_image_ext`, `AVATAR_MAX_BYTES`) and its one UPDATE, writes the
//! image under a temporary name and stages it (`files::Staged`): it replaces
//! `avatars/<user id>.<ext>` only after the commit, and the old file with
//! another extension is removed then.

use super::files::Staged;
use super::{
    check_opt_text, check_text, check_timezone, ApplyError, Avatar, Ctx, Day, DayHours, Profile,
    PublicPageLanguage, Window,
};
use crate::web::agent::error::ToolError;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Component, Path, PathBuf};

/// `settings_save`: the name is 1..=255 bytes.
const NAME_MAX: usize = 255;
/// `settings_save`: a username of at least 2 characters, cut to 100.
const USERNAME_MIN: usize = 2;
const USERNAME_MAX: usize = 100;
/// `settings_save`: the booking email is at most 255 bytes.
const BOOKING_EMAIL_MAX: usize = 255;
/// The diff value of an avatar whose stored file cannot be read.
const AVATAR_UNREADABLE: &str = "(file missing)";
/// The weekdays in the order an export lists them.
const WEEK: [Day; 7] = [
    Day::Mon,
    Day::Tue,
    Day::Wed,
    Day::Thu,
    Day::Fri,
    Day::Sat,
    Day::Sun,
];

/// An `availability_rules` row: day of week (0 is Sunday), start, end.
type Rule = (i32, String, String);

/// The form fields of Settings → Profile as stored, in the document's terms.
#[derive(Clone, Debug)]
struct Stored {
    name: String,
    username: Option<String>,
    title: Option<String>,
    bio: Option<String>,
    booking_email: Option<String>,
    timezone: String,
    language: Option<String>,
    public_page_language: PublicPageLanguage,
    week_start: Option<Day>,
    allow_dynamic_group: bool,
    lend_resource_write: bool,
    /// Sorted.
    rules: Vec<Rule>,
}

type StoredRow = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
    bool,
    Option<String>,
);

/// The stored profile and `users.avatar_path`.
async fn read(ctx: &Ctx) -> Result<(Stored, Option<String>), ApplyError> {
    let row: StoredRow = sqlx::query_as(
        "SELECT name, username, title, bio, booking_email, timezone, language, \
         public_page_language, week_start, allow_dynamic_group, lend_resource_write, avatar_path \
         FROM users WHERE id = ?",
    )
    .bind(&ctx.me.id)
    .fetch_one(ctx.pool())
    .await?;
    let rules: Vec<Rule> = sqlx::query_as(
        "SELECT day_of_week, start_time, end_time FROM user_availability_rules \
         WHERE user_id = ? ORDER BY day_of_week, start_time, end_time",
    )
    .bind(&ctx.me.id)
    .fetch_all(ctx.pool())
    .await?;
    let (
        name,
        username,
        title,
        bio,
        booking_email,
        timezone,
        language,
        public_page_language,
        week_start,
        allow_dynamic_group,
        lend_resource_write,
        avatar_path,
    ) = row;
    // As the settings page reads them: an unsupported language follows the
    // browser, anything but 'host' is 'visitor'.
    let public_page_language = public_page_language
        .as_deref()
        .and_then(crate::web::dashboard::normalize_public_page_language)
        .and_then(PublicPageLanguage::from_key)
        .unwrap_or(PublicPageLanguage::Visitor);
    let stored = Stored {
        name,
        username,
        title,
        bio,
        booking_email,
        timezone,
        language: language.filter(|code| crate::i18n::is_supported(code)),
        public_page_language,
        week_start: week_start.as_deref().and_then(Day::from_key),
        allow_dynamic_group,
        lend_resource_write,
        rules,
    };
    Ok((stored, avatar_path))
}

// --- Canonical input ---

fn check_name(path: &str, name: &str) -> Result<(), ApplyError> {
    check_text(path, name, Some(NAME_MAX))
}

/// What `settings_save` keeps of a username unchanged: lowercase ASCII
/// letters, digits and `-`, 2 to 100 of them.
fn check_username(path: &str, username: &str) -> Result<(), ApplyError> {
    check_text(path, username, None)?;
    if !username
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ApplyError::invalid(
            path,
            "only lowercase letters a-z, digits and '-' (the form would rewrite it)",
        ));
    }
    if !(USERNAME_MIN..=USERNAME_MAX).contains(&username.len()) {
        return Err(ApplyError::invalid(
            path,
            format!("{USERNAME_MIN} to {USERNAME_MAX} characters"),
        ));
    }
    Ok(())
}

/// Markdown whose every link the public page keeps (http, https, mailto or
/// relative): it is rendered by the page's own renderer and refused when
/// that drops a link.
fn check_bio(path: &str, bio: Option<&str>) -> Result<(), ApplyError> {
    check_opt_text(path, bio, None)?;
    let Some(bio) = bio else {
        return Ok(());
    };
    use pulldown_cmark::{Event, Options, Parser, Tag};
    let links = Parser::new_ext(bio, Options::ENABLE_STRIKETHROUGH)
        .filter(|event| matches!(event, Event::Start(Tag::Link { .. })))
        .count();
    let kept = crate::utils::render_inline_markdown(bio)
        .matches("<a target=\"_blank\" rel=\"noopener noreferrer\" href=")
        .count();
    if kept < links {
        return Err(ApplyError::invalid(
            path,
            "links must be http, https, mailto or relative (the page drops any other)",
        ));
    }
    Ok(())
}

fn check_booking_email(path: &str, email: Option<&str>) -> Result<(), ApplyError> {
    check_opt_text(path, email, Some(BOOKING_EMAIL_MAX))
}

/// A language the form stores as given: a code calrs ships.
fn check_language(path: &str, language: Option<&str>) -> Result<(), ApplyError> {
    let Some(code) = language else {
        return Ok(());
    };
    check_text(path, code, None)?;
    if !crate::i18n::is_supported(code) {
        let codes = crate::i18n::supported_with_labels()
            .map(|(code, _)| code)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ApplyError::invalid(
            path,
            format!("{code:?} is not a language calrs ships ({codes}); null follows the browser"),
        ));
    }
    Ok(())
}

/// The week as stored rules: `ops_write::weekly_rules` (each day once, at
/// most 10 windows, HH:MM, start before end, no overlap, not empty), with
/// no whitespace and the windows of a day in order.
fn desired_rules(path: &str, days: &[DayHours]) -> Result<Vec<Rule>, ApplyError> {
    for (i, day) in days.iter().enumerate() {
        for (j, window) in day.windows.iter().enumerate() {
            let at = format!("{path}[{i}].windows[{j}]");
            check_text(&format!("{at}.start"), &window.start, None)?;
            check_text(&format!("{at}.end"), &window.end, None)?;
        }
    }
    let rules = crate::web::agent::ops_write::weekly_rules(path, &DayHours::to_mcp(days))
        .map_err(|e| tool_error(path, &e))?;
    for (i, day) in days.iter().enumerate() {
        if day
            .windows
            .windows(2)
            .any(|pair| pair[0].start > pair[1].start)
        {
            return Err(ApplyError::invalid(
                format!("{path}[{i}].windows"),
                "list the windows of a day in order of their start",
            ));
        }
    }
    Ok(rules)
}

/// An `ops_write` check's error under the field it names.
fn tool_error(path: &str, error: &ToolError) -> ApplyError {
    match error
        .message()
        .strip_prefix("invalid ")
        .and_then(|rest| rest.split_once(": "))
    {
        Some((field, why)) => ApplyError::invalid(field, why),
        None => ApplyError::from_tool(path, error),
    }
}

/// Stored rules in the document's shape, Monday first, days without hours
/// left out.
fn hours_of(rules: &[Rule]) -> Vec<DayHours> {
    WEEK.iter()
        .filter_map(|&day| {
            let windows: Vec<Window> = rules
                .iter()
                .filter(|(d, _, _)| *d == day.day_of_week())
                .map(|(_, start, end)| Window {
                    start: start.clone(),
                    end: end.clone(),
                })
                .collect();
            (!windows.is_empty()).then_some(DayHours { day, windows })
        })
        .collect()
}

// --- The avatar ---

/// The image `avatar` names, checked as the upload checks one: at most
/// `AVATAR_MAX_BYTES`, a PNG, JPEG, GIF or WebP by its first bytes, and the
/// SHA-256 the document gives. Its bytes, extension and SHA-256.
async fn load_image(avatar: &Avatar) -> Result<(Vec<u8>, &'static str), ApplyError> {
    const FILE: &str = "profile.avatar.file";
    const SHA: &str = "profile.avatar.sha256";
    let max = crate::web::branding::AVATAR_MAX_BYTES;
    check_text(FILE, &avatar.file, None)?;
    if !Path::new(&avatar.file).is_absolute() {
        return Err(ApplyError::invalid(
            FILE,
            "must be an absolute path on the server",
        ));
    }
    if avatar.sha256.len() != 64
        || !avatar
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(ApplyError::invalid(SHA, "must be 64 lowercase hex digits"));
    }
    let unreadable =
        |e: std::io::Error| ApplyError::invalid(FILE, format!("cannot read {}: {e}", avatar.file));
    let meta = tokio::fs::metadata(&avatar.file)
        .await
        .map_err(unreadable)?;
    if !meta.is_file() {
        return Err(ApplyError::invalid(FILE, "not a regular file"));
    }
    let too_big = |len: u64| {
        ApplyError::invalid(
            FILE,
            format!("{len} bytes; an avatar is at most {max} bytes"),
        )
    };
    if meta.len() > max as u64 {
        return Err(too_big(meta.len()));
    }
    let bytes = tokio::fs::read(&avatar.file).await.map_err(unreadable)?;
    if bytes.len() > max {
        return Err(too_big(bytes.len() as u64));
    }
    let Some(ext) = crate::web::dashboard::detect_image_ext(&bytes) else {
        return Err(ApplyError::invalid(
            FILE,
            "not a PNG, JPEG, GIF or WebP image (SVG is not accepted)",
        ));
    };
    let sha256 = sha256_hex(&bytes);
    if sha256 != avatar.sha256 {
        return Err(ApplyError::invalid(
            SHA,
            format!("the file's SHA-256 is {sha256}"),
        ));
    }
    Ok((bytes, ext))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn avatars_dir(ctx: &Ctx) -> PathBuf {
    ctx.state.data_dir.join("avatars")
}

/// The file `users.avatar_path` names, when it is a plain file name.
fn avatar_file(ctx: &Ctx, name: &str) -> Option<PathBuf> {
    let mut components = Path::new(name).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Some(avatars_dir(ctx).join(name)),
        _ => None,
    }
}

/// The SHA-256 of the stored avatar's file, if it can be read.
async fn stored_sha256(ctx: &Ctx, name: &str) -> Option<String> {
    let file = avatar_file(ctx, name)?;
    tokio::fs::read(file).await.ok().map(|b| sha256_hex(&b))
}

/// An avatar in the diff: `sha256:<hex>`, or null.
fn avatar_value(name: Option<&str>, sha256: Option<&str>) -> Value {
    match (name, sha256) {
        (None, _) => Value::Null,
        (Some(_), Some(sha256)) => json!(format!("sha256:{sha256}")),
        (Some(_), None) => json!(AVATAR_UNREADABLE),
    }
}

async fn apply_avatar(
    ctx: &mut Ctx,
    stored: Option<&str>,
    desired: Option<&Avatar>,
) -> Result<(), ApplyError> {
    let stored_sha = match stored {
        Some(name) => stored_sha256(ctx, name).await,
        None => None,
    };
    let before = avatar_value(stored, stored_sha.as_deref());
    let Some(avatar) = desired else {
        let Some(old) = stored else {
            return Ok(());
        };
        // `delete_avatar`'s UPDATE; the file goes after the commit.
        sqlx::query(
            "UPDATE users SET avatar_path = NULL, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(&ctx.me.id)
        .execute(ctx.pool())
        .await?;
        if let Some(file) = avatar_file(ctx, old) {
            ctx.staged.push(Staged::remove(file));
        }
        ctx.record("profile.avatar", before, Value::Null);
        return Ok(());
    };
    if stored_sha.as_deref() == Some(avatar.sha256.as_str()) {
        return Ok(());
    }
    // Read again, so what is installed is what was checked.
    let (bytes, ext) = load_image(avatar).await?;
    let name = format!("{}.{ext}", ctx.me.id);
    let dest = avatars_dir(ctx).join(&name);
    let temp = ctx.state.data_dir.join(format!(
        ".calrs-apply-avatar-{}.{ext}",
        uuid::Uuid::new_v4()
    ));
    ctx.staged.push(Staged::place(
        temp.clone(),
        dest.clone(),
        avatar.sha256.clone(),
    ));
    tokio::fs::write(&temp, &bytes)
        .await
        .map_err(|e| ApplyError::internal("stage the avatar", e))?;
    // `upload_avatar`'s UPDATE.
    sqlx::query("UPDATE users SET avatar_path = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(&name)
        .bind(&ctx.me.id)
        .execute(ctx.pool())
        .await?;
    if let Some(old) = stored.filter(|old| *old != name) {
        if let Some(file) = avatar_file(ctx, old) {
            ctx.staged.push(Staged::remove(file));
        }
    }
    ctx.record(
        "profile.avatar",
        before,
        avatar_value(Some(&name), Some(&avatar.sha256)),
    );
    Ok(())
}

async fn verify_avatar(ctx: &Ctx, desired: Option<&Avatar>) -> Result<(), ApplyError> {
    let stored: Option<String> = sqlx::query_scalar("SELECT avatar_path FROM users WHERE id = ?")
        .bind(&ctx.me.id)
        .fetch_one(ctx.pool())
        .await?;
    let sha = match stored.as_deref() {
        None => None,
        Some(name) => match avatar_file(ctx, name) {
            None => None,
            Some(file) => match Staged::placed_at(&ctx.staged, &file) {
                Some(sha) => Some(sha.to_string()),
                None => stored_sha256(ctx, name).await,
            },
        },
    };
    let stored_value = avatar_value(stored.as_deref(), sha.as_deref());
    let wanted = desired.map_or(Value::Null, |a| json!(format!("sha256:{}", a.sha256)));
    if stored_value != wanted {
        return Err(ApplyError::stored_differs(
            "profile.avatar",
            &wanted,
            &stored_value,
        ));
    }
    Ok(())
}

// --- The form ---

/// The stored profile with the document's values on top.
fn merge(stored: &Stored, section: &Profile) -> Result<Stored, ApplyError> {
    let mut w = stored.clone();
    if let Some(v) = &section.name {
        w.name = v.clone();
    }
    if let Some(v) = &section.username {
        w.username = Some(v.clone());
    }
    if let Some(v) = &section.title {
        w.title = v.clone();
    }
    if let Some(v) = &section.bio {
        w.bio = v.clone();
    }
    if let Some(v) = &section.booking_email {
        w.booking_email = v.clone();
    }
    if let Some(v) = &section.timezone {
        w.timezone = v.clone();
    }
    if let Some(v) = &section.language {
        w.language = v.clone();
    }
    if let Some(v) = section.public_page_language {
        w.public_page_language = v;
    }
    if let Some(v) = section.week_start {
        w.week_start = v;
    }
    if let Some(v) = section.allow_dynamic_group {
        w.allow_dynamic_group = v;
    }
    if let Some(v) = section.lend_resource_write {
        w.lend_resource_write = v;
    }
    if let Some(days) = &section.weekly_hours {
        w.rules = desired_rules("profile.weekly_hours", days)?;
    }
    Ok(w)
}

/// The form fields as the document spells them, in its order.
fn values(s: &Stored) -> [(&'static str, Value); 12] {
    [
        ("name", json!(s.name)),
        ("username", json!(s.username)),
        ("title", json!(s.title)),
        ("bio", json!(s.bio)),
        ("booking_email", json!(s.booking_email)),
        ("timezone", json!(s.timezone)),
        ("language", json!(s.language)),
        ("public_page_language", json!(s.public_page_language)),
        ("week_start", json!(s.week_start)),
        ("allow_dynamic_group", json!(s.allow_dynamic_group)),
        ("lend_resource_write", json!(s.lend_resource_write)),
        ("weekly_hours", json!(hours_of(&s.rules))),
    ]
}

/// The fields whose value differs: (key, stored, wanted).
fn differences(stored: &Stored, wanted: &Stored) -> Vec<(&'static str, Value, Value)> {
    values(stored)
        .into_iter()
        .zip(values(wanted))
        .filter(|((_, s), (_, w))| s != w)
        .map(|((key, s), (_, w))| (key, s, w))
        .collect()
}

/// Settings → Profile as the page posts it, holding `w`. An empty username
/// keeps the stored one (there is none then); an empty week start inherits
/// the site's; a checkbox is posted only when ticked.
fn form_fields(w: &Stored) -> Vec<(&'static str, String)> {
    let mut fields = vec![
        ("name", w.name.clone()),
        ("username", w.username.clone().unwrap_or_default()),
        ("title", w.title.clone().unwrap_or_default()),
        ("bio", w.bio.clone().unwrap_or_default()),
        ("booking_email", w.booking_email.clone().unwrap_or_default()),
        ("timezone", w.timezone.clone()),
        ("language", w.language.clone().unwrap_or_default()),
        (
            "public_page_language",
            w.public_page_language.key().to_string(),
        ),
        (
            "week_start",
            w.week_start.map(Day::key).unwrap_or_default().to_string(),
        ),
    ];
    if w.allow_dynamic_group {
        fields.push(("allow_dynamic_group", "on".to_string()));
    }
    if w.lend_resource_write {
        fields.push(("lend_resource_write", "on".to_string()));
    }
    fields.push((
        "avail_schedule",
        crate::web::agent::ops_write::schedule_string(&w.rules),
    ));
    fields
}

async fn post(ctx: &Ctx, wanted: &Stored) -> Result<(), ApplyError> {
    let (headers, form) = super::form(&form_fields(wanted)).await?;
    let response = crate::web::dashboard::settings_save(
        State(ctx.state.clone()),
        ctx.auth_user(),
        headers,
        form,
    )
    .await
    .into_response();
    super::answered("profile", response).await
}

// --- The section ---

/// Every check of the section, reading only (pipeline step 1). The rest
/// (a username taken, a booking email the form refuses) is the form's.
pub(super) async fn check(_ctx: &Ctx, section: &Profile) -> Result<(), ApplyError> {
    if let Some(name) = &section.name {
        check_name("profile.name", name)?;
    }
    if let Some(username) = &section.username {
        check_username("profile.username", username)?;
    }
    if let Some(title) = &section.title {
        check_opt_text("profile.title", title.as_deref(), None)?;
    }
    if let Some(bio) = &section.bio {
        check_bio("profile.bio", bio.as_deref())?;
    }
    if let Some(email) = &section.booking_email {
        check_booking_email("profile.booking_email", email.as_deref())?;
    }
    if let Some(timezone) = &section.timezone {
        check_timezone("profile.timezone", timezone)?;
    }
    if let Some(language) = &section.language {
        check_language("profile.language", language.as_deref())?;
    }
    if let Some(days) = &section.weekly_hours {
        desired_rules("profile.weekly_hours", days)?;
    }
    if let Some(Some(avatar)) = &section.avatar {
        load_image(avatar).await?;
    }
    Ok(())
}

/// Post the form when a field differs, stage the avatar when it differs,
/// and record the changes (pipeline step 3).
pub(super) async fn apply(ctx: &mut Ctx, section: &Profile) -> Result<(), ApplyError> {
    let (stored, avatar_path) = read(ctx).await?;
    let wanted = merge(&stored, section)?;
    let differences = differences(&stored, &wanted);
    for (key, before, after) in &differences {
        ctx.record(format!("profile.{key}"), before, after);
    }
    if let Some(avatar) = &section.avatar {
        apply_avatar(ctx, avatar_path.as_deref(), avatar.as_ref()).await?;
    }
    if !differences.is_empty() {
        post(ctx, &wanted).await?;
    }
    Ok(())
}

/// Read back and compare with the document (pipeline step 4).
pub(super) async fn verify(ctx: &Ctx, section: &Profile) -> Result<(), ApplyError> {
    let (stored, _) = read(ctx).await?;
    let wanted = merge(&stored, section)?;
    if let Some((key, stored, wanted)) = differences(&stored, &wanted).into_iter().next() {
        return Err(ApplyError::stored_differs(
            format!("profile.{key}"),
            &wanted,
            &stored,
        ));
    }
    if let Some(avatar) = &section.avatar {
        verify_avatar(ctx, avatar.as_ref()).await?;
    }
    Ok(())
}

/// The stored state in the document's shape (`--export`), reading only. A
/// stored value the document cannot state canonically (one the UI never
/// writes) is left out, so applying the export leaves it as it is.
pub(super) async fn export(ctx: &Ctx) -> Result<Option<Profile>, ApplyError> {
    let (s, avatar_path) = read(ctx).await?;
    let ok = |r: Result<(), ApplyError>| r.is_ok();
    let hours = hours_of(&s.rules);
    let avatar = match avatar_path.as_deref() {
        None => Some(None),
        Some(name) => export_avatar(ctx, name).await.map(Some),
    };
    Ok(Some(Profile {
        name: ok(check_name("", &s.name)).then(|| s.name.clone()),
        username: s.username.clone().filter(|u| ok(check_username("", u))),
        title: ok(check_opt_text("", s.title.as_deref(), None)).then(|| s.title.clone()),
        bio: ok(check_bio("", s.bio.as_deref())).then(|| s.bio.clone()),
        booking_email: ok(check_booking_email("", s.booking_email.as_deref()))
            .then(|| s.booking_email.clone()),
        timezone: check_timezone("", &s.timezone)
            .is_ok()
            .then(|| s.timezone.clone()),
        language: Some(s.language.clone()),
        public_page_language: Some(s.public_page_language),
        week_start: Some(s.week_start),
        allow_dynamic_group: Some(s.allow_dynamic_group),
        lend_resource_write: Some(s.lend_resource_write),
        weekly_hours: (!hours.is_empty() && desired_rules("", &hours).is_ok()).then_some(hours),
        avatar,
    }))
}

/// The stored avatar as the document gives one, if its file is a valid one.
async fn export_avatar(ctx: &Ctx, name: &str) -> Option<Avatar> {
    let file = avatar_file(ctx, name)?;
    let file = tokio::fs::canonicalize(file).await.ok()?;
    let sha256 = stored_sha256(ctx, name).await?;
    let avatar = Avatar {
        file: file.to_str()?.to_string(),
        sha256,
    };
    load_image(&avatar).await.ok()?;
    Some(avatar)
}
