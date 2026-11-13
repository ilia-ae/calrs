//! ilia fork: 3CX video meetings for bookings (owner's request 2026-11-13:
//! "we use 3CX at https://sip.ilia.ae, not Jitsi").
//!
//! An event type with `location_type = "threecx_auto"` gets, when a booking
//! is confirmed, its own scheduled meeting on the owner's 3CX PBX through the
//! 3CX Video Conference REST API (`/webmeeting/api/v1`, header `3CX-ApiKey`,
//! the key generated in the 3CX admin console; not the XAPI key):
//!
//! - `POST /scheduled` with the time, the duration, the subject and the
//!   organizing extension, and **no participants**: 3CX would otherwise email
//!   the guest itself, from its own address, next to calrs's emails (owner's
//!   decision 2026-11-13: "important not to double the emails"). Its answer
//!   carries the meeting id and the organizer's join link (host rights).
//! - `GET /scheduled/{id}` for the meeting's open link, which the guest gets:
//!   `bookings.meeting_url`, so every guest email, page, `.ics` and Telegram
//!   message shows it as for any other provider.
//! - The organizer link is kept encrypted in `ilia_threecx_meetings` and goes
//!   only to the host: the host's calendar copy (`writeback`) and the host
//!   emails ([`for_host`]).
//! - A moved booking moves its meeting (`PATCH /scheduled/{id}`: the links
//!   stay the same, checked against the owner's PBX on 2026-11-13), and a
//!   booking that ends (cancelled, declined, expired, deleted) deletes it
//!   (`DELETE /scheduled/{id}`). Both are done by [`sweep`], every minute
//!   from the reminder loop, from what the database says: every cancel and
//!   reschedule path of calrs (dashboard, guest links, MCP agent, CLI,
//!   calendar orphans, TON expiry) is covered without a hook in each, and a
//!   failed call is retried with a back-off.
//!
//! The PBX locks out an address after repeated failed logins, so a refused
//! key stops every call for an hour ([`auth_blocked`]); the admin page's test
//! button tries again at once. The key never reaches a log line, an error
//! message or the page (`ThreeCxConfig` prints `<redacted>`).
//!
//! The key may come from the deployment instead of the page: `CALRS_THREECX_API_KEY`
//! (a systemd credential, `crate::credentials`; owner's decision 2026-11-13:
//! "all secrets in the vault"). When it is set it is the key, the stored one
//! is never used, and the card and `config apply` refuse a key of their own
//! ([`env_key`]).

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// `event_types.location_type` of a 3CX meeting.
pub const LOCATION_TYPE: &str = "threecx_auto";

/// Request timeout of one API call.
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
/// How long a refused key stops every call.
const AUTH_BACKOFF: std::time::Duration = std::time::Duration::from_secs(3600);
/// Rows one sweep pass looks at.
const SWEEP_BATCH: i64 = 50;
/// A meeting whose booking ended this long ago is forgotten (the PBX keeps
/// its own history; nothing is deleted there).
const FORGET_AFTER_DAYS: i64 = 2;
/// Longest field the PBX gets from calrs.
const SUBJECT_MAX: usize = 100;

/// The site's 3CX settings (`auth_config.threecx_*`).
#[derive(Clone)]
pub struct ThreeCxConfig {
    /// `https://host[:port]`, no path.
    pub base_url: String,
    pub api_key: String,
    /// The extension that organizes the meetings, digits only.
    pub extension: String,
    /// What guests see as the place, e.g. "3CX"; `None` = the generic label.
    pub display_name: Option<String>,
}

impl std::fmt::Debug for ThreeCxConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreeCxConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &"<redacted>")
            .field("extension", &self.extension)
            .field("display_name", &self.display_name)
            .finish()
    }
}

// --- Checks (admin form and config apply) ---

/// The base URL as stored: `https://host[:port]`, lower-cased host, no path,
/// query, fragment or user info; plain `http://` only for `localhost` and
/// `127.0.0.1` (development and the tests' mock PBX).
pub fn check_base_url(raw: &str) -> Result<String, &'static str> {
    let raw = raw.trim().trim_end_matches('/');
    let url = reqwest::Url::parse(raw).map_err(|_| "not a URL")?;
    let host = url.host_str().ok_or("no host")?.to_ascii_lowercase();
    let local = host == "localhost" || host == "127.0.0.1";
    match url.scheme() {
        "https" => {}
        "http" if local => {}
        _ => return Err("must be https:// (http:// only for localhost)"),
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("must not carry a user name or password");
    }
    if url.path() != "/" && !url.path().is_empty() {
        return Err("must be the PBX address only, without a path");
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("must not carry a query or fragment");
    }
    let mut out = format!("{}://{}", url.scheme(), host);
    if let Some(port) = url.port() {
        out.push_str(&format!(":{port}"));
    }
    Ok(out)
}

/// An extension: 1 to 10 digits.
pub fn check_extension(raw: &str) -> Result<String, &'static str> {
    let ext = raw.trim();
    if ext.is_empty() || ext.len() > 10 || !ext.bytes().all(|b| b.is_ascii_digit()) {
        return Err("must be 1 to 10 digits");
    }
    Ok(ext.to_string())
}

/// An API key: 16 to 256 printable ASCII characters without spaces.
pub fn check_api_key(raw: &str) -> Result<String, &'static str> {
    let key = raw.trim();
    if key.len() < 16 || key.len() > 256 || !key.bytes().all(|b| b.is_ascii_graphic()) {
        return Err("must be 16 to 256 printable characters without spaces");
    }
    Ok(key.to_string())
}

/// The label guests see: at most 100 characters, no control characters.
pub fn check_display_name(raw: &str) -> Result<Option<String>, &'static str> {
    let name = raw.trim();
    if name.is_empty() {
        return Ok(None);
    }
    if name.chars().count() > 100 || name.chars().any(char::is_control) {
        return Err("must be at most 100 characters, without control characters");
    }
    Ok(Some(name.to_string()))
}

// --- The key from the deployment ---

/// The environment variable that carries the key (also a name in
/// `crate::credentials::NAMES`).
pub const ENV_KEY: &str = "CALRS_THREECX_API_KEY";

/// What `CALRS_THREECX_API_KEY` says.
#[derive(Clone, PartialEq, Eq)]
pub enum EnvKey {
    /// Not set (or empty): the key is the stored one.
    Unset,
    /// The key.
    Set(String),
    /// Set but not a key: 3CX stays off rather than using another key.
    Malformed,
}

impl std::fmt::Debug for EnvKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            EnvKey::Unset => "Unset",
            EnvKey::Set(_) => "Set(<redacted>)",
            EnvKey::Malformed => "Malformed",
        })
    }
}

impl EnvKey {
    /// The variable is set, valid or not: the key is the deployment's.
    pub fn is_set(&self) -> bool {
        !matches!(self, EnvKey::Unset)
    }
}

/// Pure: the variable's value read as a key. Empty is unset, so the role can
/// always define the credential.
pub fn parse_env_key(value: Option<&str>) -> EnvKey {
    match value.map(str::trim) {
        None | Some("") => EnvKey::Unset,
        Some(raw) => match check_api_key(raw) {
            Ok(key) => EnvKey::Set(key),
            Err(_) => EnvKey::Malformed,
        },
    }
}

#[cfg(not(test))]
static ENV: std::sync::OnceLock<EnvKey> = std::sync::OnceLock::new();

/// `CALRS_THREECX_API_KEY`, read once. A malformed value logs its name, never
/// the value. Tests read [`test_env_key`]'s value instead, never the process
/// environment.
pub fn env_key() -> EnvKey {
    #[cfg(not(test))]
    {
        ENV.get_or_init(|| {
            let parsed = parse_env_key(std::env::var(ENV_KEY).ok().as_deref());
            if parsed == EnvKey::Malformed {
                tracing::warn!("{ENV_KEY} is malformed (16 to 256 printable characters); 3CX off");
            }
            parsed
        })
        .clone()
    }
    #[cfg(test)]
    {
        ENV_OVERRIDE.with(|c| c.borrow().clone())
    }
}

#[cfg(test)]
thread_local! {
    /// Tests only: the variable for this thread (`#[tokio::test]` is one
    /// thread, the app under `oneshot` included).
    static ENV_OVERRIDE: std::cell::RefCell<EnvKey> = const { std::cell::RefCell::new(EnvKey::Unset) };
}

/// Tests only: act as if `CALRS_THREECX_API_KEY` were `value` until the guard
/// is dropped.
#[cfg(test)]
pub fn test_env_key(value: Option<&str>) -> EnvKeyGuard {
    EnvKeyGuard(ENV_OVERRIDE.with(|c| c.replace(parse_env_key(value))))
}

#[cfg(test)]
pub struct EnvKeyGuard(EnvKey);

#[cfg(test)]
impl Drop for EnvKeyGuard {
    fn drop(&mut self) {
        let old = std::mem::replace(&mut self.0, EnvKey::Unset);
        ENV_OVERRIDE.with(|c| *c.borrow_mut() = old);
    }
}

// --- Stored settings ---

/// The settings, `None` unless base URL, extension and key are all there and
/// valid. The key is `CALRS_THREECX_API_KEY` when that is set (none when it is
/// malformed), else the stored one (a key that does not decrypt counts as
/// missing).
pub async fn load(pool: &SqlitePool, key: &[u8; 32]) -> Option<ThreeCxConfig> {
    let row: Option<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT threecx_base_url, threecx_api_key_enc, threecx_extension, threecx_display_name \
             FROM auth_config WHERE id = 'singleton'",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let (base_url, key_enc, extension, display_name) = row?;
    let base_url = check_base_url(base_url.as_deref()?).ok()?;
    let extension = check_extension(extension.as_deref()?).ok()?;
    let api_key = match env_key() {
        EnvKey::Set(env) => env,
        EnvKey::Malformed => return None,
        EnvKey::Unset => {
            let stored = crate::crypto::decrypt_value(key, key_enc.as_deref()?).ok()?;
            check_api_key(&stored).ok()?
        }
    };
    Some(ThreeCxConfig {
        base_url,
        api_key,
        extension,
        display_name: display_name.and_then(|n| check_display_name(&n).ok().flatten()),
    })
}

/// What the admin page shows: the stored values except the key, and whether
/// there is one (the deployment's, or a stored one when the deployment sets
/// none).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Shown {
    pub base_url: String,
    pub extension: String,
    pub display_name: String,
    pub key_set: bool,
    /// `CALRS_THREECX_API_KEY` is set: the key is the deployment's.
    pub key_from_env: bool,
    /// It is set but not a key.
    pub key_env_bad: bool,
}

pub async fn shown(pool: &SqlitePool) -> Shown {
    let row: Option<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT threecx_base_url, threecx_api_key_enc, threecx_extension, threecx_display_name \
             FROM auth_config WHERE id = 'singleton'",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let env = env_key();
    let (base_url, key_enc, extension, display_name) = row.unwrap_or_default();
    Shown {
        base_url: base_url.unwrap_or_default(),
        extension: extension.unwrap_or_default(),
        display_name: display_name.unwrap_or_default(),
        key_set: match &env {
            EnvKey::Set(_) => true,
            EnvKey::Malformed => false,
            EnvKey::Unset => key_enc.is_some_and(|k| !k.is_empty()),
        },
        key_from_env: env.is_set(),
        key_env_bad: env == EnvKey::Malformed,
    }
}

/// A change to the stored settings. `api_key: None` keeps the stored key
/// (the form's empty password field); `clear` removes everything, the key
/// with it. While `CALRS_THREECX_API_KEY` is set a key is refused, and every
/// save removes a stored key, which is never used then.
pub struct Update {
    pub base_url: Option<String>,
    pub extension: Option<String>,
    pub display_name: Option<String>,
    pub api_key: Option<String>,
    pub clear: bool,
}

/// The refusal of a key while the deployment sets it.
pub const KEY_FROM_ENV: &str = "is set by the deployment (CALRS_THREECX_API_KEY); change it there";

/// Why a change was refused, with the field it concerns.
#[derive(Debug, PartialEq, Eq)]
pub struct Refused {
    pub field: &'static str,
    pub reason: &'static str,
}

/// Check and store a change in one statement. Base URL and extension go
/// together (both or neither); a key needs both. Saving the settings clears
/// a refused-key pause, since the key may be the new one.
pub async fn store(pool: &SqlitePool, key: &[u8; 32], update: Update) -> Result<(), StoreError> {
    if update.clear {
        sqlx::query(
            "UPDATE auth_config SET threecx_base_url = NULL, threecx_api_key_enc = NULL, \
             threecx_extension = NULL, threecx_display_name = NULL, updated_at = datetime('now') \
             WHERE id = 'singleton'",
        )
        .execute(pool)
        .await?;
        clear_auth_block();
        return Ok(());
    }
    let refused = |field, reason| StoreError::Refused(Refused { field, reason });
    let base_url = match update.base_url.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(raw) => Some(check_base_url(raw).map_err(|r| refused("base_url", r))?),
    };
    let extension = match update.extension.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(raw) => Some(check_extension(raw).map_err(|r| refused("extension", r))?),
    };
    if base_url.is_some() != extension.is_some() {
        return Err(refused(
            if base_url.is_none() {
                "base_url"
            } else {
                "extension"
            },
            "the PBX address and the extension go together",
        ));
    }
    let display_name = check_display_name(update.display_name.as_deref().unwrap_or(""))
        .map_err(|r| refused("display_name", r))?;
    let from_env = env_key().is_set();
    let new_key = match update.api_key.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(_) if from_env => return Err(refused("api_key", KEY_FROM_ENV)),
        Some(raw) => Some(check_api_key(raw).map_err(|r| refused("api_key", r))?),
    };
    if new_key.is_some() && base_url.is_none() {
        return Err(refused(
            "api_key",
            "set the PBX address and the extension too",
        ));
    }
    match new_key {
        Some(api_key) => {
            let enc = crate::crypto::encrypt_value(key, &api_key)
                .map_err(|e| StoreError::Internal(e.to_string()))?;
            sqlx::query(
                "UPDATE auth_config SET threecx_base_url = ?, threecx_api_key_enc = ?, \
                 threecx_extension = ?, threecx_display_name = ?, updated_at = datetime('now') \
                 WHERE id = 'singleton'",
            )
            .bind(&base_url)
            .bind(&enc)
            .bind(&extension)
            .bind(&display_name)
            .execute(pool)
            .await?;
        }
        None => {
            // Without an address there is nothing to call: the stored key
            // goes with it, so a later address is never used with an old key.
            // With the deployment's key a stored one is never used: it goes.
            let keep_key = base_url.is_some() && !from_env;
            sqlx::query(
                "UPDATE auth_config SET threecx_base_url = ?, threecx_extension = ?, \
                 threecx_display_name = ?, \
                 threecx_api_key_enc = CASE WHEN ? THEN threecx_api_key_enc ELSE NULL END, \
                 updated_at = datetime('now') WHERE id = 'singleton'",
            )
            .bind(&base_url)
            .bind(&extension)
            .bind(&display_name)
            .bind(keep_key)
            .execute(pool)
            .await?;
        }
    }
    clear_auth_block();
    Ok(())
}

#[derive(Debug)]
pub enum StoreError {
    Refused(Refused),
    Internal(String),
}

impl From<sqlx::Error> for StoreError {
    fn from(e: sqlx::Error) -> Self {
        StoreError::Internal(e.to_string())
    }
}

// --- The API ---

/// Why a call failed. `Service` carries the PBX's `errorInformation`,
/// cleaned and cut to 200 characters; nothing else from the answer is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// The key was refused (or a call was skipped while a refusal pauses
    /// every call).
    Auth,
    /// The meeting does not exist (or there are none).
    NotFound,
    /// No answer, a timeout, or one that is not the API's JSON.
    Transport,
    /// The PBX answered with an error.
    Service(String),
    /// The PBX answered with a link calrs will not hand out.
    BadLink,
}

impl ApiError {
    /// The short form stored in `last_error` and logged.
    pub fn class(&self) -> String {
        match self {
            ApiError::Auth => "auth".to_string(),
            ApiError::NotFound => "not found".to_string(),
            ApiError::Transport => "transport".to_string(),
            ApiError::Service(msg) => format!("service: {msg}"),
            ApiError::BadLink => "bad link".to_string(),
        }
    }
}

/// Until when a refused key pauses the calls, per PBX address.
static AUTH_BLOCK: Mutex<Option<HashMap<String, Instant>>> = Mutex::new(None);

/// Whether a refused key pauses every call to this PBX right now.
pub fn auth_blocked(base_url: &str) -> bool {
    let mut guard = AUTH_BLOCK.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    match map.get(base_url) {
        Some(until) if Instant::now() < *until => true,
        Some(_) => {
            map.remove(base_url);
            false
        }
        None => false,
    }
}

fn note_auth_refused(base_url: &str) {
    let mut guard = AUTH_BLOCK.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .get_or_insert_with(HashMap::new)
        .insert(base_url.to_string(), Instant::now() + AUTH_BACKOFF);
}

/// End every pause (the settings were saved: the key may be the new one).
fn clear_auth_block() {
    let mut guard = AUTH_BLOCK.lock().unwrap_or_else(|e| e.into_inner());
    *guard = None;
}

fn clear_auth_block_for(base_url: &str) {
    let mut guard = AUTH_BLOCK.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(map) = guard.as_mut() {
        map.remove(base_url);
    }
}

/// The API's answer, read from the body: `{"status": "success", "result":
/// ...}` or `{"status": "error", "errorInformation": "..."}`. Pure, so the
/// shapes the PBX answers with are tested without a server.
pub fn parse_answer(http_status: u16, body: &[u8]) -> Result<Value, ApiError> {
    let Ok(json) = serde_json::from_slice::<Value>(body) else {
        return Err(if http_status == 401 || http_status == 403 {
            ApiError::Auth
        } else {
            ApiError::Transport
        });
    };
    if json.get("status").and_then(Value::as_str) == Some("success") && http_status < 300 {
        return Ok(json.get("result").cloned().unwrap_or(Value::Null));
    }
    let info = json
        .get("errorInformation")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let lower = info.to_ascii_lowercase();
    if lower.contains("authentication") || http_status == 401 || http_status == 403 {
        return Err(ApiError::Auth);
    }
    if lower.contains("not found") || lower.contains("no meetings") || http_status == 404 {
        return Err(ApiError::NotFound);
    }
    let cleaned: String = info.chars().filter(|c| !c.is_control()).take(200).collect();
    Err(ApiError::Service(if cleaned.is_empty() {
        format!("HTTP {http_status}")
    } else {
        cleaned
    }))
}

/// A link calrs hands out: `https` (or the configured `http` of a local
/// PBX) on the configured host and port, under `/meet/`, nothing that is not
/// a URL character.
pub fn check_link(cfg: &ThreeCxConfig, raw: &str) -> Result<String, ApiError> {
    let raw = raw.trim();
    let url = reqwest::Url::parse(raw).map_err(|_| ApiError::BadLink)?;
    let base = reqwest::Url::parse(&cfg.base_url).map_err(|_| ApiError::BadLink)?;
    let same_host =
        url.host_str().map(str::to_ascii_lowercase) == base.host_str().map(str::to_ascii_lowercase);
    let same_scheme = url.scheme() == base.scheme();
    let same_port = url.port_or_known_default() == base.port_or_known_default();
    let printable = raw.bytes().all(|b| b.is_ascii_graphic());
    if !(same_host && same_scheme && same_port && printable)
        || !url.path().starts_with("/meet/")
        || url.path().len() <= "/meet/".len()
        || !url.username().is_empty()
    {
        return Err(ApiError::BadLink);
    }
    Ok(raw.to_string())
}

async fn call(
    cfg: &ThreeCxConfig,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
) -> Result<Value, ApiError> {
    if auth_blocked(&cfg.base_url) {
        return Err(ApiError::Auth);
    }
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(TIMEOUT)
        .build()
        .map_err(|_| ApiError::Transport)?;
    let url = format!("{}/webmeeting/api/v1{}", cfg.base_url, path);
    let mut request = client
        .request(method, &url)
        .header("3CX-ApiKey", &cfg.api_key)
        .header("accept", "application/json")
        .header("user-agent", "calrs-3cx/1");
    if let Some(body) = body {
        request = request.json(&body);
    }
    // reqwest's error names the URL only, never the headers; it is still
    // not logged here, only its kind.
    let response = request.send().await.map_err(|e| {
        tracing::warn!(
            timeout = e.is_timeout(),
            connect = e.is_connect(),
            "3cx: request failed"
        );
        ApiError::Transport
    })?;
    let status = response.status().as_u16();
    let bytes = response.bytes().await.map_err(|_| ApiError::Transport)?;
    let answer = parse_answer(status, &bytes);
    if answer == Err(ApiError::Auth) {
        note_auth_refused(&cfg.base_url);
        tracing::error!("3cx: the API key was refused; every call pauses for an hour");
    }
    answer
}

/// A meeting just made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Created {
    pub meeting_id: String,
    /// The meeting's open link, for the guest.
    pub open_link: String,
    /// The organizer's join link, for the host.
    pub host_link: String,
}

fn meeting_body(start: DateTime<Utc>, duration_min: i64, subject: &str) -> Value {
    json!({
        // Whoever comes first opens the room: the host's link is in the
        // host's calendar and emails, but a host who opens the guest's link
        // must not wait for himself.
        "hostJoinFirst": false,
        "hideParticipants": false,
        "selfModeration": true,
        "audioOnStart": true,
        "videoOnStart": true,
        "chatOnStart": true,
        "dateTime": start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "duration": duration_min,
        "subject": subject,
        "description": "",
    })
}

/// Make a meeting: `POST /scheduled` without participants, then `GET` it
/// for the open link. A meeting whose links cannot be used is deleted again.
pub async fn create(
    cfg: &ThreeCxConfig,
    start: DateTime<Utc>,
    duration_min: i64,
    subject: &str,
) -> Result<Created, ApiError> {
    let mut body = meeting_body(start, duration_min, subject);
    body["extension"] = json!(cfg.extension);
    body["type"] = json!("Webmeeting");
    body["participants"] = json!([]);
    let result = call(cfg, reqwest::Method::POST, "/scheduled", Some(body)).await?;
    let meeting_id = result
        .get("meetingid")
        .and_then(Value::as_str)
        .filter(|id| {
            !id.is_empty() && id.len() <= 100 && id.bytes().all(|b| b.is_ascii_alphanumeric())
        })
        .ok_or(ApiError::Service("no meeting id in the answer".to_string()))?
        .to_string();
    let host_link = result
        .pointer("/organizer/join")
        .and_then(Value::as_str)
        .ok_or(ApiError::BadLink)
        .and_then(|l| check_link(cfg, l));
    let open_link = match &host_link {
        Ok(_) => detail_open_link(cfg, &meeting_id).await,
        Err(e) => Err(e.clone()),
    };
    match (open_link, host_link) {
        (Ok(open_link), Ok(host_link)) => Ok(Created {
            meeting_id,
            open_link,
            host_link,
        }),
        (Err(e), _) | (_, Err(e)) => {
            if let Err(cleanup) = delete(cfg, &meeting_id).await {
                tracing::warn!(error = %cleanup.class(), "3cx: a meeting made without usable links stays on the PBX");
            }
            Err(e)
        }
    }
}

async fn detail_open_link(cfg: &ThreeCxConfig, meeting_id: &str) -> Result<String, ApiError> {
    let detail = call(
        cfg,
        reqwest::Method::GET,
        &format!("/scheduled/{meeting_id}"),
        None,
    )
    .await?;
    let link = detail
        .get("openlink")
        .and_then(Value::as_str)
        .ok_or(ApiError::BadLink)?;
    check_link(cfg, link)
}

/// Move a meeting (`PATCH /scheduled/{id}`); its links stay.
pub async fn reschedule(
    cfg: &ThreeCxConfig,
    meeting_id: &str,
    start: DateTime<Utc>,
    duration_min: i64,
    subject: &str,
) -> Result<(), ApiError> {
    call(
        cfg,
        reqwest::Method::PATCH,
        &format!("/scheduled/{meeting_id}"),
        Some(meeting_body(start, duration_min, subject)),
    )
    .await
    .map(|_| ())
}

/// Delete a meeting; one that is already gone counts as deleted.
pub async fn delete(cfg: &ThreeCxConfig, meeting_id: &str) -> Result<(), ApiError> {
    match call(
        cfg,
        reqwest::Method::DELETE,
        &format!("/scheduled/{meeting_id}"),
        None,
    )
    .await
    {
        Ok(_) | Err(ApiError::NotFound) => Ok(()),
        Err(e) => Err(e),
    }
}

/// The admin page's test: list the meetings (`GET /meetings/list`; "No
/// Meetings" is a working key). A test always calls, even while a refused
/// key pauses the others, and a working key ends the pause.
pub async fn check(cfg: &ThreeCxConfig) -> Result<(), ApiError> {
    clear_auth_block_for(&cfg.base_url);
    match call(cfg, reqwest::Method::GET, "/meetings/list", None).await {
        Ok(_) | Err(ApiError::NotFound) => Ok(()),
        Err(e) => Err(e),
    }
}

// --- Bookings ---

/// A booking's start (UTC) and length in minutes, from its stored times:
/// a version-1 row is RFC 3339 UTC, a legacy row a wall clock in the event
/// type's zone.
fn booking_span(start: &str, end: &str, event_tz: chrono_tz::Tz) -> Option<(DateTime<Utc>, i64)> {
    let to_utc = |v: &str| -> Option<DateTime<Utc>> {
        let naive: NaiveDateTime = crate::booking_time::local(v, event_tz, chrono_tz::UTC)?;
        Some(naive.and_utc())
    };
    let start = to_utc(start)?;
    let end = to_utc(end)?;
    let minutes = (end - start).num_minutes();
    (minutes > 0).then_some((start, minutes))
}

fn subject(title: &str, guest_name: &str) -> String {
    let clean = |s: &str| -> String { s.chars().filter(|c| !c.is_control()).collect::<String>() };
    let full = format!("{} — {}", clean(title).trim(), clean(guest_name).trim());
    full.chars().take(SUBJECT_MAX).collect()
}

/// Make the meeting of a booking just confirmed and remember it; returns
/// the guest's link. `None` when the call fails or the booking already has
/// one (a second confirm path racing: its meeting is deleted again).
pub async fn create_for_booking(
    pool: &SqlitePool,
    key: &[u8; 32],
    cfg: &ThreeCxConfig,
    booking_id: &str,
) -> Option<String> {
    let row: Option<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT b.start_at, b.end_at, b.guest_name, et.title, et.id \
         FROM bookings b JOIN event_types et ON et.id = b.event_type_id WHERE b.id = ?",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let (start_at, end_at, guest_name, title, event_type_id) = row?;
    let tz = crate::booking_time::event_timezone(pool, &event_type_id)
        .await
        .ok()?;
    let (start, minutes) = booking_span(&start_at, &end_at, tz)?;
    let subject = subject(&title, &guest_name);
    let created = match create(cfg, start, minutes, &subject).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(%booking_id, error = %e.class(), "3cx: could not make the meeting");
            return None;
        }
    };
    let host_enc = match crate::crypto::encrypt_value(key, &created.host_link) {
        Ok(enc) => enc,
        Err(_) => {
            let _ = delete(cfg, &created.meeting_id).await;
            return None;
        }
    };
    let inserted = sqlx::query(
        "INSERT INTO ilia_threecx_meetings (booking_id, meeting_id, host_link_enc, start_at, duration_min, subject) \
         VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT(booking_id) DO NOTHING",
    )
    .bind(booking_id)
    .bind(&created.meeting_id)
    .bind(&host_enc)
    .bind(start.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    .bind(minutes)
    .bind(&subject)
    .execute(pool)
    .await;
    match inserted {
        Ok(r) if r.rows_affected() == 1 => {
            tracing::info!(%booking_id, "3cx: meeting made");
            Some(created.open_link)
        }
        other => {
            if let Err(e) = other {
                tracing::error!(%booking_id, error = %e, "3cx: could not remember the meeting");
            }
            let _ = delete(cfg, &created.meeting_id).await;
            None
        }
    }
}

/// The organizer's join link of a booking's meeting, by booking uid.
pub async fn host_link(pool: &SqlitePool, key: &[u8; 32], booking_uid: &str) -> Option<String> {
    let enc: Option<String> = sqlx::query_scalar(
        "SELECT m.host_link_enc FROM ilia_threecx_meetings m \
         JOIN bookings b ON b.id = m.booking_id WHERE b.uid = ?",
    )
    .bind(booking_uid)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    crate::crypto::decrypt_value(key, &enc?).ok()
}

/// `details` for a host: the location swapped for the organizer's link when
/// the booking has a 3CX meeting, unchanged otherwise.
pub async fn for_host(
    pool: &SqlitePool,
    key: &[u8; 32],
    details: &crate::email::BookingDetails,
) -> crate::email::BookingDetails {
    let mut details = details.clone();
    if let Some(link) = host_link(pool, key, &details.uid).await {
        details.location = Some(link);
    }
    details
}

/// What one sweep pass did, for the tests and the log.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Swept {
    pub moved: u32,
    pub deleted: u32,
    pub forgotten: u32,
    pub failed: u32,
}

fn backoff_minutes(attempts: i64) -> i64 {
    (1i64 << attempts.clamp(0, 6)).min(60)
}

/// Keep the PBX in step with the bookings: delete the meeting of a booking
/// that ended or is gone, move the meeting of a booking whose time changed,
/// forget meetings of bookings over two days past. Called every minute from
/// the reminder loop; does nothing without settings or while a refused key
/// pauses the calls.
pub async fn sweep(pool: &SqlitePool, cfg: Option<&ThreeCxConfig>, now: DateTime<Utc>) -> Swept {
    let mut swept = Swept::default();
    let Some(cfg) = cfg else {
        return swept;
    };
    if auth_blocked(&cfg.base_url) {
        return swept;
    }
    let now_s = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    #[allow(clippy::type_complexity)]
    let rows: Vec<(
        String,
        String,
        String,
        i64,
        String,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = match sqlx::query_as(
        "SELECT m.booking_id, m.meeting_id, m.start_at, m.duration_min, m.subject, m.attempts, \
                b.status, b.start_at, b.end_at, b.event_type_id \
         FROM ilia_threecx_meetings m LEFT JOIN bookings b ON b.id = m.booking_id \
         WHERE m.next_attempt_at IS NULL OR m.next_attempt_at <= ? \
         ORDER BY m.created_at LIMIT ?",
    )
    .bind(&now_s)
    .bind(SWEEP_BATCH)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!(error = %e, "3cx: could not read the meetings");
            return swept;
        }
    };
    for (
        booking_id,
        meeting_id,
        held_start,
        held_minutes,
        subject,
        attempts,
        status,
        start,
        end,
        et_id,
    ) in rows
    {
        let live = matches!(status.as_deref(), Some("confirmed") | Some("pending"));
        let span = match (live, start.as_deref(), end.as_deref(), et_id.as_deref()) {
            (true, Some(s), Some(e), Some(et)) => {
                match crate::booking_time::event_timezone(pool, et).await {
                    Ok(tz) => booking_span(s, e, tz),
                    Err(_) => None,
                }
            }
            _ => None,
        };
        let outcome = if !live {
            delete(cfg, &meeting_id).await.map(|_| Action::Deleted)
        } else if let Some((start, minutes)) = span {
            if start + Duration::minutes(minutes) < now - Duration::days(FORGET_AFTER_DAYS) {
                Ok(Action::Forgotten)
            } else {
                let start_s = start.format("%Y-%m-%dT%H:%M:%SZ").to_string();
                if start_s == held_start && minutes == held_minutes {
                    Ok(Action::Same)
                } else {
                    reschedule(cfg, &meeting_id, start, minutes, &subject)
                        .await
                        .map(|_| Action::Moved(start_s, minutes))
                }
            }
        } else {
            Ok(Action::Same)
        };
        match outcome {
            Ok(Action::Same) => {}
            Ok(Action::Deleted) => {
                forget(pool, &booking_id).await;
                swept.deleted += 1;
                tracing::info!(%booking_id, "3cx: meeting deleted");
            }
            Ok(Action::Forgotten) => {
                forget(pool, &booking_id).await;
                swept.forgotten += 1;
            }
            Ok(Action::Moved(start_s, minutes)) => {
                let _ = sqlx::query(
                    "UPDATE ilia_threecx_meetings SET start_at = ?, duration_min = ?, attempts = 0, \
                     next_attempt_at = NULL, last_error = NULL WHERE booking_id = ?",
                )
                .bind(&start_s)
                .bind(minutes)
                .bind(&booking_id)
                .execute(pool)
                .await;
                swept.moved += 1;
                tracing::info!(%booking_id, "3cx: meeting moved");
            }
            Err(e) => {
                swept.failed += 1;
                let next = now + Duration::minutes(backoff_minutes(attempts));
                let _ = sqlx::query(
                    "UPDATE ilia_threecx_meetings SET attempts = attempts + 1, next_attempt_at = ?, \
                     last_error = ? WHERE booking_id = ?",
                )
                .bind(next.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .bind(e.class())
                .bind(&booking_id)
                .execute(pool)
                .await;
                tracing::warn!(%booking_id, error = %e.class(), "3cx: meeting not in step, retried later");
                if e == ApiError::Auth {
                    break;
                }
            }
        }
    }
    swept
}

async fn forget(pool: &SqlitePool, booking_id: &str) {
    if let Err(e) = sqlx::query("DELETE FROM ilia_threecx_meetings WHERE booking_id = ?")
        .bind(booking_id)
        .execute(pool)
        .await
    {
        tracing::error!(%booking_id, error = %e, "3cx: could not forget a meeting");
    }
}

enum Action {
    Same,
    Deleted,
    Forgotten,
    Moved(String, i64),
}

// --- The admin page's 3CX card ---

/// The card's form: `action` is `save` (the default), `test` or `clear`.
#[derive(serde::Deserialize)]
pub(super) struct AdminForm {
    _csrf: Option<String>,
    action: Option<String>,
    threecx_base_url: Option<String>,
    threecx_extension: Option<String>,
    threecx_display_name: Option<String>,
    threecx_api_key: Option<String>,
}

/// What the card says after a post (`/dashboard/admin?threecx=<notice>`):
/// only these words are read back, so nothing the PBX said reaches the page.
pub(super) const NOTICES: &[&str] = &[
    "saved",
    "cleared",
    "ok",
    "auth",
    "transport",
    "service",
    "bad-link",
    "not-set",
    "bad-base-url",
    "bad-extension",
    "bad-api-key",
    "bad-display-name",
    "bad-pair",
    "key-env",
];

fn notice_for(error: &ApiError) -> &'static str {
    match error {
        ApiError::Auth => "auth",
        ApiError::Transport => "transport",
        ApiError::NotFound | ApiError::Service(_) => "service",
        ApiError::BadLink => "bad-link",
    }
}

fn refused_notice(refused: &Refused) -> &'static str {
    match refused.field {
        "base_url" if refused.reason.contains("together") => "bad-pair",
        "extension" if refused.reason.contains("together") => "bad-pair",
        "api_key" if refused.reason == KEY_FROM_ENV => "key-env",
        "api_key" if refused.reason.contains("too") => "bad-pair",
        "base_url" => "bad-base-url",
        "extension" => "bad-extension",
        "api_key" => "bad-api-key",
        _ => "bad-display-name",
    }
}

fn back(notice: &str) -> axum::response::Response {
    use axum::response::IntoResponse;
    axum::response::Redirect::to(&format!("/dashboard/admin?threecx={notice}#threecx"))
        .into_response()
}

/// `POST /dashboard/admin/threecx`: save, test or remove the settings.
/// Admins only, CSRF-checked; the key is write-only (an empty field keeps
/// the stored one) and never shown back.
pub(super) async fn admin_save(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<super::AppState>>,
    admin: crate::auth::AdminUser,
    headers: axum::http::HeaderMap,
    axum::Form(form): axum::Form<AdminForm>,
) -> axum::response::Response {
    if let Err(resp) = super::verify_csrf_token(&headers, &form._csrf) {
        return resp;
    }
    let action = form.action.as_deref().unwrap_or("save");
    if action == "test" {
        // The stored settings, or the form's with the stored key when the
        // field is empty: an admin can test before saving.
        let stored = load(&state.pool, &state.secret_key).await;
        let typed_url = form
            .threecx_base_url
            .as_deref()
            .and_then(|u| check_base_url(u).ok());
        let typed_ext = form
            .threecx_extension
            .as_deref()
            .and_then(|e| check_extension(e).ok());
        // The deployment's key is the key: a typed one is not tried.
        let typed_key = match env_key() {
            EnvKey::Set(env) => Some(env),
            EnvKey::Malformed => None,
            EnvKey::Unset => form
                .threecx_api_key
                .as_deref()
                .filter(|k| !k.trim().is_empty())
                .and_then(|k| check_api_key(k).ok()),
        };
        let cfg = match (typed_url, typed_ext, typed_key, stored) {
            (Some(base_url), Some(extension), Some(api_key), _) => Some(ThreeCxConfig {
                base_url,
                api_key,
                extension,
                display_name: None,
            }),
            (Some(base_url), Some(extension), None, Some(s)) => Some(ThreeCxConfig {
                base_url,
                extension,
                ..s
            }),
            (_, _, _, s) => s,
        };
        let Some(cfg) = cfg else {
            return back("not-set");
        };
        let notice = match check(&cfg).await {
            Ok(()) => "ok",
            Err(e) => notice_for(&e),
        };
        tracing::info!(admin = %admin.user.email, outcome = notice, "admin: 3cx tested");
        return back(notice);
    }
    let update = Update {
        base_url: form.threecx_base_url,
        extension: form.threecx_extension,
        display_name: form.threecx_display_name,
        api_key: form.threecx_api_key,
        clear: action == "clear",
    };
    let notice = match store(&state.pool, &state.secret_key, update).await {
        Ok(()) if action == "clear" => "cleared",
        Ok(()) => "saved",
        Err(StoreError::Refused(r)) => refused_notice(&r),
        Err(StoreError::Internal(e)) => {
            tracing::error!(error = %e, "admin: could not store the 3cx settings");
            return super::internal_error_response("store 3cx settings", &e);
        }
    };
    *state.meeting_config.write().await =
        super::meeting::load_config(&state.pool, &state.secret_key).await;
    tracing::info!(admin = %admin.user.email, outcome = notice, "admin: 3cx settings");
    back(notice)
}
