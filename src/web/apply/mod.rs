//! ilia fork: `calrs config apply`, the settings as code
//! (`docs/apply-schema.md`).
//!
//! One JSON document (`calrs-apply-v1`) declares the admin's profile, the
//! site settings, the sources' calendars and the admin's personal event
//! types. Each section posts the form a browser would post (the stored
//! values with the document's on top) to the web handler itself, so the
//! checks are the web's own, then everything is read back and compared.
//!
//! The run is one SQLite transaction. The state is built on a pool of
//! exactly one connection (`open_pool`), `BEGIN IMMEDIATE` is issued on it
//! (`Txn`), and every statement of every handler runs on that connection,
//! inside the transaction: sqlx 0.7 only pings a connection it takes back
//! (`return_to_pool`), it never resets it. `Txn::check` proves the
//! transaction is still the one begun (a temporary table created inside it)
//! after each section and before the commit. `--dry-run` runs the same
//! handlers and rolls back, so its diff is the real run's.
//!
//! Rules for the section modules:
//! - Never hold a `PoolConnection` or a `Transaction` of `ctx.pool()` across
//!   a handler call: the pool has one connection, the handler would wait
//!   for it until `ACQUIRE_TIMEOUT` and fail. Write with one statement at a
//!   time on `ctx.pool()`; never call `pool.begin()` (SQLite refuses a
//!   nested `BEGIN`).
//! - No network I/O: the write lock is held for the whole run.
//! - `check` reads only. `apply` posts forms and records the changes.
//!   `verify` reads back and fails on any difference. `export` reads the
//!   stored state in the document's shape.

mod event_types;
mod export;
mod files;
mod profile;
mod site;
mod sources;
mod threecx;
mod ton;

use crate::auth::{AdminUser, AuthUser};
use crate::models::User;
use axum::body::Body;
use axum::extract::{Form, FromRequest};
use axum::http::{header, HeaderMap, HeaderValue, Method, Request};
use axum::response::Response;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::AppState;

/// The document and report format. The release guards of the role grep for
/// this string in the source and in the binary: keep it a literal here.
pub(crate) const FORMAT: &str = "calrs-apply-v1";

/// A secret column in a diff: its value is never shown.
pub(crate) const SECRET_SET: &str = "(secret set)";

/// Longest document read from stdin.
pub(crate) const DOCUMENT_MAX_BYTES: u64 = 1024 * 1024;

/// How long a statement waits for the one connection. Only a section that
/// holds the connection across a handler call waits this long.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(10);

/// How long `BEGIN IMMEDIATE` waits for a running `calrs serve` to finish
/// its write.
const BUSY_TIMEOUT: Duration = Duration::from_secs(10);

/// The temporary table that marks the open transaction (`Txn`).
const TXN_MARK: &str = "calrs_apply_txn";

// --- Keywords ---

/// A closed set of keywords, spelled in the document as in the database and
/// the web forms.
macro_rules! keyword {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $key:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub(crate) enum $name {
            $(#[serde(rename = $key)] $variant),+
        }

        impl $name {
            /// The keyword as the document, the database and the forms spell it.
            pub(crate) fn key(self) -> &'static str {
                match self {
                    $(Self::$variant => $key),+
                }
            }

            /// The keyword for a stored value, if it is one.
            pub(crate) fn from_key(key: &str) -> Option<Self> {
                match key {
                    $($key => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

keyword!(
    /// A day of the week.
    Day { Mon => "mon", Tue => "tue", Wed => "wed", Thu => "thu", Fri => "fri", Sat => "sat", Sun => "sun" }
);
keyword!(
    /// `users.public_page_language`.
    PublicPageLanguage { Visitor => "visitor", Host => "host" }
);
keyword!(
    /// `auth_config.theme`.
    Theme {
        Default => "default",
        Nord => "nord",
        Dracula => "dracula",
        Gruvbox => "gruvbox",
        Solarized => "solarized",
        TokyoNight => "tokyo-night",
        Vates => "vates",
        Custom => "custom",
    }
);
keyword!(
    /// `auth_config.mfa_required`: 1 is `required`.
    MfaPolicy { Required => "required", Optional => "optional" }
);
keyword!(
    /// A feature v1 can only switch off (all its fields and secret cleared).
    Off { Off => "off" }
);
keyword!(
    /// What happens to the admin's personal event types the document leaves out.
    Prune { None => "none", Disable => "disable", Delete => "delete" }
);
keyword!(
    /// `telegram_event_types.guest_mode`.
    GuestMode { Off => "off", Optional => "optional", Required => "required" }
);
keyword!(
    /// `event_types.visibility`.
    Visibility { Public => "public", Internal => "internal", Private => "private" }
);
keyword!(
    /// `event_types.location_type`.
    LocationType {
        Link => "link",
        Phone => "phone",
        InPerson => "in_person",
        Custom => "custom",
        JitsiAuto => "jitsi_auto",
        WebhookAuto => "webhook_auto",
        GoogleMeet => "google_meet",
        // ilia fork: a meeting on the site's 3CX PBX (`web::threecx`).
        ThreeCxAuto => "threecx_auto",
    }
);
keyword!(
    /// `event_types.default_calendar_view`.
    CalendarView { Month => "month", Week => "week", Column => "column" }
);
keyword!(
    /// `event_types.sms_phone_mode`.
    PhoneMode { Off => "off", Optional => "optional", Required => "required" }
);
keyword!(
    /// `booking_frequency_limits.period`.
    Period { Day => "day", Week => "week", Month => "month", Year => "year" }
);

impl Day {
    /// The MCP tools' day, for their checks (`ops_write::weekly_rules`).
    pub(crate) fn to_mcp(self) -> super::agent::dto::Day {
        use super::agent::dto::Day as D;
        match self {
            Day::Mon => D::Mon,
            Day::Tue => D::Tue,
            Day::Wed => D::Wed,
            Day::Thu => D::Thu,
            Day::Fri => D::Fri,
            Day::Sat => D::Sat,
            Day::Sun => D::Sun,
        }
    }

    /// `availability_rules.day_of_week`: 0 is Sunday.
    pub(crate) fn day_of_week(self) -> i32 {
        self.to_mcp().day_of_week()
    }
}

// --- How a key may be given ---

/// A key that may be left out (leave as is) but not be null.
fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

/// A key that may be left out (leave as is) or be null (clear, or inherit).
fn nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// A key that must be given and may be null.
fn given<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

// --- The document (docs/apply-schema.md) ---

/// The desired state. A key left out leaves the setting as it is.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Document {
    /// Always `FORMAT`.
    pub(crate) format: String,
    /// Email of the enabled admin the profile and event types belong to.
    pub(crate) user: String,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) profile: Option<Profile>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) site: Option<Site>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) sources: Option<Vec<Source>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) event_types: Option<EventTypes>,
}

/// Settings → Profile of `Document::user`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Profile {
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) name: Option<String>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) username: Option<String>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) title: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) bio: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) booking_email: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) timezone: Option<String>,
    /// null follows the browser (Accept-Language).
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) language: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) public_page_language: Option<PublicPageLanguage>,
    /// null inherits the site's.
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) week_start: Option<Option<Day>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) allow_dynamic_group: Option<bool>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) lend_resource_write: Option<bool>,
    /// The default availability (`user_availability_rules`).
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) weekly_hours: Option<Vec<DayHours>>,
    /// null removes the avatar.
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) avatar: Option<Option<Avatar>>,
}

/// The bookable hours of one weekday (the MCP `DayHoursIn` shape).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DayHours {
    pub(crate) day: Day,
    pub(crate) windows: Vec<Window>,
}

/// `HH:MM` to `HH:MM`, 24-hour.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Window {
    pub(crate) start: String,
    pub(crate) end: String,
}

impl DayHours {
    /// The days in the MCP tools' shape, for `ops_write::weekly_rules`.
    pub(crate) fn to_mcp(days: &[DayHours]) -> Vec<super::agent::dto::DayHoursIn> {
        days.iter()
            .map(|day| super::agent::dto::DayHoursIn {
                day: day.day.to_mcp(),
                windows: day
                    .windows
                    .iter()
                    .map(|w| super::agent::dto::WindowIn {
                        start: w.start.clone(),
                        end: w.end.clone(),
                    })
                    .collect(),
            })
            .collect()
    }
}

/// An image already on the server, and its SHA-256 (lowercase hex).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Avatar {
    pub(crate) file: String,
    pub(crate) sha256: String,
}

/// Admin settings (`auth_config`). `base_url` and `allow_private_hosts`
/// belong to the environment and are not keys here.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Site {
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) theme: Option<Theme>,
    /// Required when `theme` is `custom`.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) custom: Option<CustomColors>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) week_start: Option<Day>,
    /// true addresses German speakers as "Sie".
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) german_formal: Option<bool>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) registration: Option<bool>,
    /// Comma list, lowercase; null allows every domain.
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) allowed_email_domains: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) mfa_policy: Option<MfaPolicy>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) company_link: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) jitsi: Option<Jitsi>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) oidc: Option<Off>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) google_oauth2: Option<Off>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) captcha: Option<Off>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) meeting_webhook: Option<Off>,
    /// ilia fork: the SMS gateway card (`sms_config`), only `"off"`: the
    /// stored gateway deleted, as the card's Clear button does
    /// (`site::SMS_OFF_SHAPE`). A gateway in the `CALRS_SMS_*` environment
    /// is not the document's.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) sms: Option<Off>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) sms_allow_all_users: Option<bool>,
    /// ilia fork: the TON settings the admin page also edits
    /// (`site.ton`, `apply/ton.rs`): left out, nothing changes; given, its
    /// keys are owned by `config apply` and the ones it owned and leaves
    /// out go back to the page.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) ton: Option<TonSection>,
    /// ilia fork: the 3CX card (`apply/threecx.rs`): `"off"`, or the
    /// address, extension, label and (null keeps it) the key.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) threecx: Option<threecx::ThreeCx>,
}

/// `site.ton`: the `crate::ton::settings::SPECS` fields, each a value or
/// null (the default, still owned).
pub(crate) type TonSection = std::collections::BTreeMap<String, Value>;

/// The five colours of the custom theme, each `#rrggbb`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CustomColors {
    pub(crate) accent: String,
    pub(crate) accent_hover: String,
    pub(crate) bg: String,
    pub(crate) surface: String,
    pub(crate) text: String,
}

/// `"off"`, or the Jitsi settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Jitsi {
    Off(Off),
    On(JitsiSettings),
}

/// Every key must be given; `pattern` null uses `{event}-{random}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct JitsiSettings {
    pub(crate) base_url: String,
    #[serde(deserialize_with = "given")]
    pub(crate) pattern: Option<String>,
    #[serde(deserialize_with = "given")]
    pub(crate) display_name: Option<String>,
}

/// A calendar source that already exists, matched on `url` and `username`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {
    pub(crate) url: String,
    pub(crate) username: String,
    /// ilia fork: the source's name (Sources → Edit), 1..255 bytes, not
    /// padded. Written without connecting to the calendar server.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) name: Option<String>,
    /// Display name of the calendar bookings are written to; null writes none.
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) write_calendar: Option<Option<String>>,
    /// Display names of the calendars synced; every other one is not.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) sync_calendars: Option<Vec<String>>,
}

/// The admin's personal event types, keyed by slug.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EventTypes {
    /// Left out: `none`.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) prune: Option<Prune>,
    pub(crate) items: Vec<EventTypeItem>,
}

impl EventTypes {
    pub(crate) fn prune(&self) -> Prune {
        self.prune.unwrap_or(Prune::None)
    }
}

/// One event type. Field names, values and ranges are the MCP
/// `EventTypePatch` ones; a key left out keeps the stored value, or the new
/// event type form's default on creation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EventTypeItem {
    pub(crate) slug: String,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) title: Option<String>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) description: Option<Option<String>>,
    /// ilia fork: the title and description per shipped language
    /// (`event_type_texts`, `web::event_texts`), keyed by locale code; the
    /// whole set: `{}` removes every one. A language may appear once.
    #[serde(
        default,
        deserialize_with = "unique_texts",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) translations: Option<std::collections::BTreeMap<String, TextItem>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) duration_min: Option<i64>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) slot_interval_min: Option<Option<i64>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) buffer_before: Option<i64>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) buffer_after: Option<i64>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) min_notice_min: Option<i64>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) requires_confirmation: Option<bool>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) pending_blocks_slot: Option<bool>,
    /// null: pending bookings never expire.
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) pending_expiry_hours: Option<Option<i64>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) visibility: Option<Visibility>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) location_type: Option<LocationType>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) location_value: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) meeting_pattern_override: Option<Option<String>>,
    /// null follows the profile's timezone.
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) timezone: Option<Option<String>>,
    /// null uses the profile's default hours.
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) weekly_hours: Option<Option<Vec<DayHours>>>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) booking_horizon_days: Option<Option<i64>>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) bookable_from: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) bookable_until: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) reminder_minutes: Option<Option<i64>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) max_additional_guests: Option<i64>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) first_slot_only: Option<bool>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) default_calendar_view: Option<CalendarView>,
    /// null inherits the profile's, then the site's.
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) week_start: Option<Option<Day>>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) cancel_notice_min: Option<Option<i64>>,
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) reschedule_notice_min: Option<Option<i64>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) sms_phone_mode: Option<PhoneMode>,
    /// The whole list; `[]` removes every limit.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) frequency_limits: Option<Vec<FrequencyLimit>>,
    /// ilia fork: the calendars whose events block this type's slots, by
    /// display name; null: every synced calendar (the form's default, no
    /// calendar chosen).
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) busy_calendars: Option<Option<Vec<String>>>,
    /// ilia fork: the date overrides (days off, custom hours) from today on,
    /// the whole list; `[]` removes every one from today on. Past rows are
    /// never touched.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) overrides: Option<Vec<DateOverride>>,
    /// The price section of the event type form; null: free (no price).
    #[serde(
        default,
        deserialize_with = "nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) ton_price: Option<Option<TonPrice>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) telegram_guest_mode: Option<GuestMode>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) enabled: Option<bool>,
}

/// ilia fork: one language's title and description of an event type
/// (`translations`); `description` null or left out: none in that language.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TextItem {
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) description: Option<String>,
}

/// ilia fork: `translations`, given (it may not be null): an object whose
/// keys are languages, each at most once (a JSON object read into a map
/// would otherwise keep the last of two silently).
fn unique_texts<'de, D>(
    deserializer: D,
) -> Result<Option<std::collections::BTreeMap<String, TextItem>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Texts;
    impl<'de> serde::de::Visitor<'de> for Texts {
        type Value = std::collections::BTreeMap<String, TextItem>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an object of languages, each {title, description}")
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = std::collections::BTreeMap::new();
            while let Some(lang) = map.next_key::<String>()? {
                let text: TextItem = map.next_value()?;
                if out.insert(lang.clone(), text).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "translations: language {lang:?} given twice"
                    )));
                }
            }
            Ok(out)
        }
    }
    deserializer.deserialize_map(Texts).map(Some)
}

/// At most `count` bookings per `period` (per booker email is not a thing:
/// per event type, as the web form's `N:period`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FrequencyLimit {
    pub(crate) count: i64,
    pub(crate) period: Period,
}

/// ilia fork: one date override of an event type (`availability_overrides`):
/// `{date, blocked: true}` (the day off) or `{date, start, end}` (custom
/// hours that day, `HH:MM`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DateOverride {
    pub(crate) date: String,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) blocked: Option<bool>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) start: Option<String>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) end: Option<String>,
}

/// The TON price of an event type (`ton_prices`), the event type form's
/// price section: exactly one of `ton` (the invoice amount itself), `usd`
/// (converted at the rate of the moment the guest books), spelled as the
/// form shows a stored price ("100.00", "500.00"), and `ladder: true` (ilia
/// fork: the site's demand ladder, in TON). Always the whole
/// price: `window_min` and `enabled` left out take their defaults.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TonPrice {
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) ton: Option<String>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) usd: Option<String>,
    /// ilia fork: `true` instead of `ton`/`usd`: the price follows the
    /// site's demand ladder (TON). Only `true` is accepted; leave it out
    /// for a fixed price.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) ladder: Option<bool>,
    /// Minutes the guest has to pay; null: the server's default
    /// (`CALRS_TON_WINDOW_MIN`).
    #[serde(default)]
    pub(crate) window_min: Option<i64>,
    /// false keeps the price switched off: the type books free.
    #[serde(default = "switched_on")]
    pub(crate) enabled: bool,
}

fn switched_on() -> bool {
    true
}

// --- The report ---

/// The one line `config apply` prints.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    pub(crate) format: String,
    pub(crate) dry_run: bool,
    pub(crate) changed: Vec<Change>,
    /// A value `calrs serve` caches at startup changed: restart it.
    pub(crate) restart: bool,
    pub(crate) unmanaged: Unmanaged,
}

/// One setting that changed. Paths: `profile.title`, `site.custom.accent`,
/// `sources[0].write_calendar`, `event_types[15min].duration_min`, and
/// `event_types[15min]` for a created (before null, after "created") or
/// deleted (before "present", after null) event type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Change {
    pub(crate) path: String,
    pub(crate) before: Value,
    pub(crate) after: Value,
}

/// What the document does not manage but exists.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Unmanaged {
    /// Slugs of the admin's personal event types the document leaves out,
    /// as they were before pruning.
    pub(crate) event_types: Vec<String>,
}

// --- Errors ---

/// Why a run stopped. Every one rolls back, except `AfterCommit`.
#[derive(Debug)]
pub(crate) enum ApplyError {
    /// The document is not valid or not canonical. Exit 2.
    Invalid { path: String, message: String },
    /// A handler refused the form, or stored something else than asked.
    /// Exit 2.
    Refused { path: String, message: String },
    /// Anything else. Exit 1.
    Internal(anyhow::Error),
    /// The transaction committed, then a staged file could not be put in
    /// place (`files::finish`). Exit 1. The settings are written; the next
    /// run sees the file missing and stages it again. `report` is the run's
    /// report (set by `run`): `cli` prints it on stdout as on success, so
    /// the caller still learns what was written and whether `calrs serve`
    /// must restart, which the next run cannot tell any more.
    AfterCommit {
        error: anyhow::Error,
        report: Option<Box<Report>>,
    },
}

impl ApplyError {
    pub(crate) fn invalid(path: impl Into<String>, message: impl fmt::Display) -> Self {
        Self::Invalid {
            path: path.into(),
            message: message.to_string(),
        }
    }

    pub(crate) fn refused(path: impl Into<String>, message: impl fmt::Display) -> Self {
        Self::Refused {
            path: path.into(),
            message: message.to_string(),
        }
    }

    pub(crate) fn internal(context: &str, error: impl fmt::Display) -> Self {
        Self::Internal(anyhow::anyhow!("{context}: {error}"))
    }

    /// A failure after the commit (`AfterCommit`).
    pub(crate) fn after_commit(context: &str, error: impl fmt::Display) -> Self {
        Self::AfterCommit {
            error: anyhow::anyhow!("{context}: {error}"),
            report: None,
        }
    }

    /// The report of a run that committed before failing (`AfterCommit`).
    pub(crate) fn report(&self) -> Option<&Report> {
        match self {
            Self::AfterCommit { report, .. } => report.as_deref(),
            _ => None,
        }
    }

    /// An MCP check's error (`ops_write`) under the document's path.
    pub(crate) fn from_tool(
        path: impl Into<String>,
        error: &super::agent::error::ToolError,
    ) -> Self {
        Self::invalid(path, error.message())
    }

    /// The handler accepted the form but the read-back differs.
    pub(crate) fn stored_differs(path: impl Into<String>, wanted: &Value, stored: &Value) -> Self {
        Self::refused(
            path,
            format!("the form was accepted but {stored} is stored instead of {wanted}"),
        )
    }

    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            Self::Invalid { .. } | Self::Refused { .. } => 2,
            Self::Internal(_) | Self::AfterCommit { .. } => 1,
        }
    }
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid { path, message } => write!(f, "invalid {path}: {message}"),
            Self::Refused { path, message } => {
                write!(f, "refused {path}: {message}; nothing was written")
            }
            Self::Internal(error) => write!(f, "internal error: {error:#}; nothing was written"),
            Self::AfterCommit { error, .. } => write!(
                f,
                "internal error after the commit: {error:#}; the settings were written, run again to put the file in place"
            ),
        }
    }
}

impl From<sqlx::Error> for ApplyError {
    fn from(error: sqlx::Error) -> Self {
        Self::Internal(anyhow::Error::new(error).context("database"))
    }
}

// --- Canonical input (docs/apply-schema.md, "Canonical values") ---

/// Text the handler would store as given: not empty, nothing `trim` drops,
/// at most `max_bytes` when given.
pub(crate) fn check_text(
    path: &str,
    value: &str,
    max_bytes: Option<usize>,
) -> Result<(), ApplyError> {
    if value.is_empty() {
        return Err(ApplyError::invalid(
            path,
            "must not be empty (use null where it may be cleared)",
        ));
    }
    if value.trim() != value {
        return Err(ApplyError::invalid(
            path,
            "must not start or end with whitespace",
        ));
    }
    if let Some(max) = max_bytes {
        if value.len() > max {
            return Err(ApplyError::invalid(
                path,
                format!("longer than {max} bytes"),
            ));
        }
    }
    Ok(())
}

/// A nullable text: null, or `check_text`. `""` is refused: null is the one
/// way to say empty.
pub(crate) fn check_opt_text(
    path: &str,
    value: Option<&str>,
    max_bytes: Option<usize>,
) -> Result<(), ApplyError> {
    value.map_or(Ok(()), |v| check_text(path, v, max_bytes))
}

/// An IANA timezone as `chrono_tz` spells it.
pub(crate) fn check_timezone(path: &str, value: &str) -> Result<chrono_tz::Tz, ApplyError> {
    check_text(path, value, None)?;
    value
        .parse::<chrono_tz::Tz>()
        .map_err(|_| ApplyError::invalid(path, format!("{value:?} is not an IANA timezone")))
}

// --- Posting to the web handlers ---

/// The form a browser would post: `fields` after a CSRF token, with the
/// double-submit cookie in the headers, parsed the way axum parses one.
pub(crate) async fn form<T: DeserializeOwned>(
    fields: &[(&str, String)],
) -> Result<(HeaderMap, Form<T>), ApplyError> {
    let token = super::generate_csrf_token();
    let cookie = HeaderValue::from_str(&format!("{}={token}", super::CSRF_COOKIE_NAME))
        .map_err(|e| ApplyError::internal("build the form cookie", e))?;
    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, cookie);
    let body = std::iter::once(("_csrf", token.as_str()))
        .chain(fields.iter().map(|(name, value)| (*name, value.as_str())))
        .map(|(name, value)| {
            format!(
                "{}={}",
                urlencoding::encode(name),
                urlencoding::encode(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&");
    let request = Request::builder()
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .map_err(|e| ApplyError::internal("build the form request", e))?;
    let form = Form::<T>::from_request(request, &())
        .await
        .map_err(|e| ApplyError::internal("parse the form", e))?;
    Ok((headers, form))
}

/// Whether a form handler saved: it answers a saved form with a redirect
/// (the admin cards, the event type form) or with its page again and no
/// error on it (Settings → Profile). A refusal is its page with an error
/// (`class="error"`), an error page or any other status, and comes back
/// under `path` with the page's message. A handler can also answer as if it
/// saved when it did not (some swallow database errors, `create_event_type`
/// redirects when the user has no account), so sections read back anyway.
pub(crate) async fn answered(path: &str, response: Response) -> Result<(), ApplyError> {
    let status = response.status();
    if status.is_redirection() {
        return Ok(());
    }
    let body = axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default();
    let error = error_text(&body);
    if status.is_success() && error.is_none() {
        return Ok(());
    }
    let message = error.unwrap_or_else(|| format!("HTTP {status}"));
    Err(ApplyError::refused(
        path,
        format!("the web form refused it: {message}"),
    ))
}

/// The text of the first `class="error"` element of a page.
fn error_text(page: &str) -> Option<String> {
    let start = page.find("class=\"error\"")?;
    let open_end = start + page[start..].find('>')? + 1;
    let close = open_end + page[open_end..].find("</")?;
    let text = page[open_end..close]
        .replace("&#x27;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

// --- One run ---

/// What the sections share: the state the handlers run on (one connection,
/// inside the transaction), the admin, and the report being built.
pub(crate) struct Ctx {
    pub(crate) state: Arc<AppState>,
    /// The admin as stored now; `reload_me` after a section changes it.
    pub(crate) me: User,
    pub(crate) dry_run: bool,
    changes: Vec<Change>,
    restart: bool,
    pub(crate) unmanaged_event_types: Vec<String>,
    /// Files to move into place once the transaction commits.
    pub(crate) staged: Vec<files::Staged>,
    /// ilia fork: `site.ton` changes, logged (`calrs::audit`) once the
    /// transaction commits; their durable trail is `ton_settings_audit`.
    pub(crate) ton_audit: Vec<crate::web::ton_pay::settings::Change>,
}

impl Ctx {
    pub(crate) fn new(state: Arc<AppState>, me: User, dry_run: bool) -> Self {
        Self {
            state,
            me,
            dry_run,
            changes: Vec::new(),
            restart: false,
            unmanaged_event_types: Vec::new(),
            staged: Vec::new(),
            ton_audit: Vec::new(),
        }
    }

    /// The one connection's pool, inside the transaction.
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.state.pool
    }

    /// The admin as the `AuthUser` extractor gives it, in English.
    pub(crate) fn auth_user(&self) -> AuthUser {
        AuthUser {
            user: self.me.clone(),
            impersonation: None,
            lang: "en",
        }
    }

    /// The admin as the `AdminUser` extractor gives it, in English.
    pub(crate) fn admin_user(&self) -> AdminUser {
        AdminUser {
            user: self.me.clone(),
            lang: "en",
        }
    }

    pub(crate) async fn reload_me(&mut self) -> Result<(), ApplyError> {
        self.me = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(&self.me.id)
            .fetch_one(self.pool())
            .await?;
        Ok(())
    }

    /// Note a setting's change; nothing when `before` and `after` are equal
    /// as JSON. True when it changed.
    pub(crate) fn record(
        &mut self,
        path: impl Into<String>,
        before: impl Serialize,
        after: impl Serialize,
    ) -> bool {
        let before = serde_json::to_value(before).unwrap_or(Value::Null);
        let after = serde_json::to_value(after).unwrap_or(Value::Null);
        if before == after {
            return false;
        }
        self.changes.push(Change {
            path: path.into(),
            before,
            after,
        });
        true
    }

    /// `record`, for a value `calrs serve` caches at startup.
    pub(crate) fn record_restart(
        &mut self,
        path: impl Into<String>,
        before: impl Serialize,
        after: impl Serialize,
    ) -> bool {
        let changed = self.record(path, before, after);
        self.restart |= changed;
        changed
    }

    pub(crate) fn changes(&self) -> &[Change] {
        &self.changes
    }

    fn report(&self) -> Report {
        Report {
            format: FORMAT.to_string(),
            dry_run: self.dry_run,
            changed: self.changes.clone(),
            restart: self.restart,
            unmanaged: Unmanaged {
                event_types: self.unmanaged_event_types.clone(),
            },
        }
    }
}

/// The run's transaction on the one connection. Not a guard: `commit` or
/// `rollback` it explicitly (an async rollback cannot run in `Drop`; a
/// dropped pool closes the connection, and SQLite rolls back then).
pub(crate) struct Txn(());

impl Txn {
    pub(crate) async fn begin(pool: &SqlitePool) -> Result<Self, ApplyError> {
        if pool.options().get_max_connections() != 1 {
            return Err(ApplyError::internal(
                "begin",
                "the apply pool must have exactly one connection (open_pool)",
            ));
        }
        sqlx::raw_sql("BEGIN IMMEDIATE").execute(pool).await?;
        let mark = sqlx::raw_sql(&format!("CREATE TEMP TABLE {TXN_MARK} (id INTEGER)"))
            .execute(pool)
            .await;
        if let Err(error) = mark {
            let _ = sqlx::raw_sql("ROLLBACK").execute(pool).await;
            return Err(error.into());
        }
        Ok(Self(()))
    }

    /// The transaction begun is still open on the same connection: the
    /// table created inside it exists. SQLite drops it on any rollback
    /// (including its own, after a disk-full error), and a new connection
    /// never has it.
    pub(crate) async fn check(&self, pool: &SqlitePool) -> Result<(), ApplyError> {
        sqlx::raw_sql(&format!("SELECT COUNT(*) FROM temp.{TXN_MARK}"))
            .execute(pool)
            .await
            .map(|_| ())
            .map_err(|e| ApplyError::internal("the apply transaction was lost", e))
    }

    pub(crate) async fn commit(self, pool: &SqlitePool) -> Result<(), ApplyError> {
        self.check(pool).await?;
        sqlx::raw_sql(&format!("DROP TABLE temp.{TXN_MARK}"))
            .execute(pool)
            .await?;
        sqlx::raw_sql("COMMIT").execute(pool).await?;
        Ok(())
    }

    pub(crate) async fn rollback(self, pool: &SqlitePool) -> Result<(), ApplyError> {
        sqlx::raw_sql("ROLLBACK").execute(pool).await?;
        Ok(())
    }
}

/// The run's pool: one connection to `db_path`, never reaped while the run
/// holds its transaction, and `mode=ro` for `read_only`.
pub(crate) async fn open_pool(db_path: &Path, read_only: bool) -> Result<SqlitePool, ApplyError> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false)
        .read_only(read_only)
        .foreign_keys(true)
        // As `db::connect`: deleted content is zeroed.
        .pragma("secure_delete", "on")
        .busy_timeout(BUSY_TIMEOUT);
    SqlitePoolOptions::new()
        .max_connections(1)
        .min_connections(0)
        .idle_timeout(None)
        .max_lifetime(None)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_with(options)
        .await
        .map_err(|e| ApplyError::internal("open the database", e))
}

/// Parse a document: the format first, so a newer document is named as
/// one, then every key, strictly.
pub(crate) fn parse(input: &str) -> Result<Document, ApplyError> {
    let value: Value = serde_json::from_str(input)
        .map_err(|e| ApplyError::invalid("(document)", format!("not JSON: {e}")))?;
    match value.get("format").and_then(Value::as_str) {
        Some(FORMAT) => {}
        Some(other) => {
            return Err(ApplyError::invalid(
                "format",
                format!("{other:?} is not supported; this calrs reads {FORMAT:?}"),
            ))
        }
        None => return Err(ApplyError::invalid("format", format!("must be {FORMAT:?}"))),
    }
    serde_json::from_str(input).map_err(|e| ApplyError::invalid("(document)", e))
}

/// The enabled admin `email` names, or the only enabled admin when `None`.
pub(crate) async fn admin(pool: &SqlitePool, email: Option<&str>) -> Result<User, ApplyError> {
    match email {
        Some(email) => {
            check_text("user", email, None)?;
            sqlx::query_as::<_, User>(
                "SELECT * FROM users WHERE email = ? AND role = 'admin' AND enabled = 1",
            )
            .bind(email)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ApplyError::invalid("user", format!("{email} is not an enabled admin")))
        }
        None => {
            let mut admins = sqlx::query_as::<_, User>(
                "SELECT * FROM users WHERE role = 'admin' AND enabled = 1 LIMIT 2",
            )
            .fetch_all(pool)
            .await?;
            match (admins.pop(), admins.is_empty()) {
                (Some(admin), true) => Ok(admin),
                (Some(_), false) => Err(ApplyError::invalid(
                    "user",
                    "more than one enabled admin: name one with --user",
                )),
                (None, _) => Err(ApplyError::invalid("user", "there is no enabled admin")),
            }
        }
    }
}

/// Apply `doc` on `pool` (from `open_pool(_, false)`), in one transaction,
/// committed unless `dry_run`.
pub(crate) async fn run(
    pool: SqlitePool,
    data_dir: PathBuf,
    secret_key: [u8; 32],
    doc: &Document,
    dry_run: bool,
) -> Result<Report, ApplyError> {
    let state = super::build_state(pool.clone(), data_dir, secret_key).await;
    let txn = Txn::begin(&pool).await?;
    let me = match admin(&pool, Some(&doc.user)).await {
        Ok(me) => me,
        Err(error) => {
            txn.rollback(&pool).await?;
            return Err(error);
        }
    };
    let mut ctx = Ctx::new(state, me, dry_run);
    let outcome = match sections(&mut ctx, &txn, doc).await {
        Ok(()) if dry_run => txn.rollback(&pool).await,
        Ok(()) => txn.commit(&pool).await,
        Err(error) => {
            if let Err(rollback) = txn.rollback(&pool).await {
                tracing::error!(error = %rollback, "config apply: rollback failed");
            }
            Err(error)
        }
    };
    let staged = std::mem::take(&mut ctx.staged);
    let committed = outcome.is_ok() && !dry_run;
    if committed {
        crate::web::ton_pay::settings::log_changes(
            &ctx.ton_audit,
            Some(&ctx.me.id),
            crate::ton::settings::Owner::Apply,
        );
    }
    let finished = files::finish(staged, committed);
    outcome?;
    let report = ctx.report();
    match finished {
        Ok(()) => Ok(report),
        Err(ApplyError::AfterCommit { error, .. }) => Err(ApplyError::AfterCommit {
            error,
            report: Some(Box::new(report)),
        }),
        Err(error) => Err(error),
    }
}

/// Every section: all checks first (reads only), then each section's
/// writes, then every read-back.
async fn sections(ctx: &mut Ctx, txn: &Txn, doc: &Document) -> Result<(), ApplyError> {
    if let Some(profile) = &doc.profile {
        profile::check(ctx, profile).await?;
    }
    if let Some(site) = &doc.site {
        site::check(ctx, site).await?;
    }
    if let Some(sources) = &doc.sources {
        sources::check(ctx, sources).await?;
    }
    if let Some(event_types) = &doc.event_types {
        event_types::check(ctx, event_types).await?;
    }

    // Event types last: their form reads the profile (timezone, default
    // hours) and the site (Jitsi for `jitsi_auto`).
    if let Some(profile) = &doc.profile {
        profile::apply(ctx, profile).await?;
        ctx.reload_me().await?;
        txn.check(ctx.pool()).await?;
    }
    if let Some(site) = &doc.site {
        site::apply(ctx, site).await?;
        txn.check(ctx.pool()).await?;
    }
    if let Some(sources) = &doc.sources {
        sources::apply(ctx, sources).await?;
        txn.check(ctx.pool()).await?;
    }
    if let Some(event_types) = &doc.event_types {
        event_types::apply(ctx, event_types).await?;
        txn.check(ctx.pool()).await?;
    }

    if let Some(profile) = &doc.profile {
        profile::verify(ctx, profile).await?;
    }
    if let Some(site) = &doc.site {
        site::verify(ctx, site).await?;
    }
    if let Some(sources) = &doc.sources {
        sources::verify(ctx, sources).await?;
    }
    if let Some(event_types) = &doc.event_types {
        event_types::verify(ctx, event_types).await?;
    }
    Ok(())
}

/// The stored state as a document (`--export`), on `pool` (from
/// `open_pool(_, true)`): it reads only.
pub(crate) async fn export_document(
    pool: SqlitePool,
    data_dir: PathBuf,
    secret_key: [u8; 32],
    user: Option<&str>,
) -> Result<Document, ApplyError> {
    let me = admin(&pool, user).await?;
    let state = super::build_state(pool, data_dir, secret_key).await;
    let ctx = Ctx::new(state, me, true);
    export::export(&ctx).await
}

// --- The command ---

/// `calrs config apply`.
pub(crate) enum Command {
    /// `--stdin [--dry-run]`.
    Apply { dry_run: bool },
    /// `--export [--user EMAIL]`.
    Export { user: Option<String> },
}

/// Run `command` on the database `main_pool` is connected to. Prints one
/// JSON line on stdout; on an error prints it on stderr and exits 2
/// (invalid or refused) or 1 (internal), having written nothing. The one
/// exception is `AfterCommit` (exit 1): the settings are written, and the
/// report line is printed on stdout before the error, `restart` included.
pub(crate) async fn cli(
    main_pool: &SqlitePool,
    secret_key: &[u8; 32],
    command: Command,
) -> anyhow::Result<()> {
    let db_path = (*main_pool.connect_options())
        .clone()
        .get_filename()
        .into_owned();
    let data_dir = db_path.parent().map(Path::to_path_buf).unwrap_or_default();
    let line = match command {
        Command::Apply { dry_run } => apply_stdin(&db_path, data_dir, *secret_key, dry_run).await,
        Command::Export { user } => {
            export_cli(&db_path, data_dir, *secret_key, user.as_deref()).await
        }
    };
    match line {
        Ok(line) => {
            println!("{line}");
            Ok(())
        }
        Err(error) => {
            if let Some(line) = error.report().and_then(|r| serde_json::to_string(r).ok()) {
                println!("{line}");
            }
            eprintln!("calrs config apply: {error}");
            main_pool.close().await;
            std::process::exit(error.exit_code());
        }
    }
}

async fn apply_stdin(
    db_path: &Path,
    data_dir: PathBuf,
    secret_key: [u8; 32],
    dry_run: bool,
) -> Result<String, ApplyError> {
    let input = read_document(std::io::stdin().lock())?;
    let doc = parse(&input)?;
    let pool = open_pool(db_path, false).await?;
    let report = run(pool.clone(), data_dir, secret_key, &doc, dry_run).await;
    pool.close().await;
    serde_json::to_string(&report?).map_err(|e| ApplyError::internal("print the report", e))
}

async fn export_cli(
    db_path: &Path,
    data_dir: PathBuf,
    secret_key: [u8; 32],
    user: Option<&str>,
) -> Result<String, ApplyError> {
    let pool = open_pool(db_path, true).await?;
    let doc = export_document(pool.clone(), data_dir, secret_key, user).await;
    pool.close().await;
    serde_json::to_string(&doc?).map_err(|e| ApplyError::internal("print the document", e))
}

/// The document, at most `DOCUMENT_MAX_BYTES` of UTF-8.
pub(crate) fn read_document(input: impl std::io::Read) -> Result<String, ApplyError> {
    use std::io::Read;
    let mut text = String::new();
    input
        .take(DOCUMENT_MAX_BYTES + 1)
        .read_to_string(&mut text)
        .map_err(|e| ApplyError::invalid("(document)", format!("unreadable: {e}")))?;
    if text.len() as u64 > DOCUMENT_MAX_BYTES {
        return Err(ApplyError::invalid(
            "(document)",
            format!("longer than {DOCUMENT_MAX_BYTES} bytes"),
        ));
    }
    Ok(text)
}
