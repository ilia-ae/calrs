//! ilia fork: `config apply`, the `site` section: the admin settings
//! (`auth_config`) and the MFA policy.
//!
//! Each key of the section is compared with the stored value, and only the
//! admin cards whose keys differ are posted, as a browser posts them: the
//! card's stored values with the document's on top.
//!
//! | Keys                                   | Card (handler)                      |
//! |----------------------------------------|-------------------------------------|
//! | `theme`, `custom`                      | Theme (`admin_update_accent`)       |
//! | `week_start`, `german_formal`          | General (`admin_update_general`)    |
//! | `registration`, `allowed_email_domains`| Authentication (`admin_update_auth`)|
//! | `company_link`                         | Company link                        |
//! | `jitsi`                                | Jitsi (`admin_update_jitsi`)        |
//! | `oidc`, `google_oauth2`, `captcha`, `meeting_webhook` | their cards, all fields empty |
//! | `sms` (ilia fork, only `"off"`)        | SMS gateway, its Clear button (`admin_update_sms_clear`) |
//! | `sms_allow_all_users`                  | SMS policy                          |
//! | `ton` (ilia fork)                      | none: `apply/ton.rs` writes `ton_settings` as the TON page does |
//!
//! Two writes have no form apply can post:
//! - `mfa_policy`: the web form asks for the admin's password and a TOTP
//!   code. apply calls `mfa::store_policy`, the statements `policy_save`
//!   runs once those passed (the policy, and for `required` the end of every
//!   local session that has not passed MFA). The CLI runs as the server's OS
//!   user with the database in hand, which is already more than an admin.
//! - The secrets of the features switched `off`: every card keeps the stored
//!   secret when its field is empty, so apply clears the column itself.
//!
//! The General card also carries `base_url` and `allow_private_hosts`,
//! which belong to the environment and are not keys of the document. apply
//! posts their stored values back and fails when the handler stored
//! anything else.

use super::{
    check_opt_text, check_text, ApplyError, Ctx, CustomColors, Day, Jitsi, JitsiSettings,
    MfaPolicy, Off, Site, Theme, SECRET_SET,
};
use crate::web::admin;
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};
use std::collections::HashSet;

/// ilia fork: the marker of a calrs whose `config apply` knows `site.sms`
/// (part of the refusal below, so it is in the binary). The ilia-booking
/// role greps the commit and the release for it before it sends the key:
/// a raw string, so the source holds the same bytes as the binary (an
/// escaped `\"` would not match `git grep -F` on the commit).
pub(crate) const SMS_OFF_SHAPE: &str = r#"site.sms: "off" deletes the stored SMS gateway"#;

/// Every check of the section, reading only (pipeline step 1). The document
/// alone decides: each value must be in the form the handler stores as is.
pub(super) async fn check(ctx: &Ctx, site: &Site) -> Result<(), ApplyError> {
    if let Some(ton) = &site.ton {
        super::ton::check(ctx, ton).await?;
    }
    if let Some(threecx) = &site.threecx {
        super::threecx::check(ctx, threecx).await?;
    }
    match (site.theme, &site.custom) {
        (Some(Theme::Custom), None) => {
            return Err(ApplyError::invalid(
                "site.custom",
                "theme \"custom\" needs all five colours under site.custom",
            ))
        }
        (Some(Theme::Custom), Some(colours)) => check_colours(colours)?,
        (_, Some(_)) => {
            return Err(ApplyError::invalid(
                "site.custom",
                "is only for theme \"custom\", given in the same document",
            ))
        }
        (_, None) => {}
    }
    if let Some(Some(domains)) = &site.allowed_email_domains {
        check_domains("site.allowed_email_domains", domains)?;
    }
    if let Some(Some(link)) = &site.company_link {
        check_company_link("site.company_link", link)?;
    }
    if let Some(Jitsi::On(jitsi)) = &site.jitsi {
        check_jitsi(jitsi)?;
    }
    if site.sms.is_some() && crate::sms::sms_env_active() {
        // The card's handler refuses to clear (it redirects without a
        // word) while the environment block governs the gateway.
        return Err(ApplyError::refused(
            "site.sms",
            format!(
                "{SMS_OFF_SHAPE}, but the CALRS_SMS_* environment block sets the gateway \
                 (it wins over the database): remove one of the two"
            ),
        ));
    }
    Ok(())
}

/// Post the cards whose keys differ and record the changes (pipeline step
/// 3). A card whose keys all match is not posted.
pub(super) async fn apply(ctx: &mut Ctx, site: &Site) -> Result<(), ApplyError> {
    if let Some(ton) = &site.ton {
        super::ton::apply(ctx, ton).await?;
    }
    if let Some(threecx) = &site.threecx {
        super::threecx::apply(ctx, threecx).await?;
    }
    let before = Stored::load(ctx).await?;
    let pairs = pairs(site, &before);
    let mut cards = HashSet::new();
    for pair in pairs.iter().filter(|p| p.differs()) {
        cards.insert(pair.card);
        if pair.restart {
            ctx.record_restart(&pair.path, &pair.stored, &pair.wanted);
        } else {
            ctx.record(&pair.path, &pair.stored, &pair.wanted);
        }
    }

    if cards.contains(&Card::Theme) {
        post_theme(ctx, site).await?;
    }
    if cards.contains(&Card::General) {
        post_general(ctx, site, &before).await?;
    }
    if cards.contains(&Card::Auth) {
        post_auth(ctx, site, &before).await?;
    }
    if cards.contains(&Card::Mfa) {
        let mandatory = site.mfa_policy == Some(MfaPolicy::Required);
        let mut conn = ctx.pool().acquire().await?;
        crate::mfa::store_policy(&mut conn, mandatory)
            .await
            .map_err(|e| ApplyError::internal("store the MFA policy", e))?;
    }
    if cards.contains(&Card::CompanyLink) {
        let link = site.company_link.clone().flatten().unwrap_or_default();
        let fields = [("company_link", link)];
        let (headers, form) = super::form(&fields).await?;
        let response = admin::admin_update_company_link(
            State(ctx.state.clone()),
            ctx.admin_user(),
            headers,
            form,
        )
        .await
        .into_response();
        posted("site.company_link", response).await?;
    }
    if cards.contains(&Card::Jitsi) {
        let (base_url, pattern, display_name) = match &site.jitsi {
            Some(Jitsi::On(j)) => (
                j.base_url.clone(),
                j.pattern.clone().unwrap_or_default(),
                j.display_name.clone().unwrap_or_default(),
            ),
            _ => Default::default(),
        };
        let fields = [
            ("jitsi_base_url", base_url),
            ("jitsi_pattern", pattern),
            ("jitsi_display_name", display_name),
        ];
        let (headers, form) = super::form(&fields).await?;
        let response =
            admin::admin_update_jitsi(State(ctx.state.clone()), ctx.admin_user(), headers, form)
                .await
                .into_response();
        posted("site.jitsi", response).await?;
    }
    if cards.contains(&Card::Oidc) {
        // Switched off, with the auto-register choice the card shows kept.
        let mut fields = vec![
            ("oidc_issuer_url", String::new()),
            ("oidc_client_id", String::new()),
            ("oidc_client_secret", String::new()),
        ];
        if before.oidc_auto_register {
            fields.push(("oidc_auto_register", "on".to_string()));
        }
        let (headers, form) = super::form(&fields).await?;
        let response =
            admin::admin_update_oidc(State(ctx.state.clone()), ctx.admin_user(), headers, form)
                .await
                .into_response();
        posted("site.oidc", response).await?;
    }
    if cards.contains(&Card::Google) {
        let fields = [
            ("google_oauth2_client_id", String::new()),
            ("google_oauth2_client_secret", String::new()),
        ];
        let (headers, form) = super::form(&fields).await?;
        let response = admin::admin_update_google_oauth2(
            State(ctx.state.clone()),
            ctx.admin_user(),
            headers,
            form,
        )
        .await
        .into_response();
        posted("site.google_oauth2", response).await?;
    }
    if cards.contains(&Card::Captcha) {
        let fields = [
            ("captcha_instance_url", String::new()),
            ("captcha_site_key", String::new()),
            ("captcha_secret", String::new()),
            ("captcha_widget_url", String::new()),
            ("captcha_wasm_url", String::new()),
        ];
        let (headers, form) = super::form(&fields).await?;
        let response =
            admin::admin_update_captcha(State(ctx.state.clone()), ctx.admin_user(), headers, form)
                .await
                .into_response();
        posted("site.captcha", response).await?;
    }
    if cards.contains(&Card::Webhook) {
        let fields = [
            ("meeting_webhook_url", String::new()),
            ("meeting_webhook_auth_mode", String::new()),
            ("meeting_webhook_secret", String::new()),
            ("meeting_webhook_display_name", String::new()),
        ];
        let (headers, form) = super::form(&fields).await?;
        let response = admin::admin_update_meeting_webhook(
            State(ctx.state.clone()),
            ctx.admin_user(),
            headers,
            form,
        )
        .await
        .into_response();
        posted("site.meeting_webhook", response).await?;
    }
    if cards.contains(&Card::Sms) {
        // Every row, the encrypted secret with it: the card's Clear button.
        let (headers, form) = super::form(&[]).await?;
        let response = admin::admin_update_sms_clear(
            State(ctx.state.clone()),
            ctx.admin_user(),
            headers,
            form,
        )
        .await
        .into_response();
        posted("site.sms", response).await?;
    }
    for secret in Secret::ALL {
        if cards.contains(&Card::Secret(secret)) {
            sqlx::query(secret.clear())
                .execute(ctx.pool())
                .await
                .map_err(|e| ApplyError::internal("clear a secret", e))?;
        }
    }
    if cards.contains(&Card::SmsPolicy) {
        let mut fields = Vec::new();
        if site.sms_allow_all_users == Some(true) {
            fields.push(("allow_all_users", "on".to_string()));
        }
        let (headers, form) = super::form(&fields).await?;
        let response = admin::admin_update_sms_policy(
            State(ctx.state.clone()),
            ctx.admin_user(),
            headers,
            form,
        )
        .await
        .into_response();
        posted("site.sms_allow_all_users", response).await?;
    }
    Ok(())
}

/// Read back and compare with the document (pipeline step 4).
pub(super) async fn verify(ctx: &Ctx, site: &Site) -> Result<(), ApplyError> {
    if let Some(ton) = &site.ton {
        super::ton::verify(ctx, ton).await?;
    }
    if let Some(threecx) = &site.threecx {
        super::threecx::verify(ctx, threecx).await?;
    }
    let stored = Stored::load(ctx).await?;
    match pairs(site, &stored).into_iter().find(Pair::differs) {
        Some(pair) => Err(ApplyError::stored_differs(
            pair.path,
            &pair.wanted,
            &pair.stored,
        )),
        None => Ok(()),
    }
}

/// The stored state in the document's shape (`--export`), reading only. A
/// key whose stored value the document cannot say (a value the checks
/// refuse, a feature that is not off) is left out, so applying the export
/// changes nothing.
pub(super) async fn export(ctx: &Ctx) -> Result<Option<Site>, ApplyError> {
    let s = Stored::load(ctx).await?;
    let custom = s.colours().filter(|c| check_colours(c).is_ok());
    let (theme, custom) = match Theme::from_key(&s.theme) {
        Some(Theme::Custom) if custom.is_some() => (Some(Theme::Custom), custom),
        Some(Theme::Custom) | None => (None, None),
        Some(theme) => (Some(theme), None),
    };
    let off = |is_off: bool| is_off.then_some(Off::Off);
    Ok(Some(Site {
        theme,
        custom,
        week_start: Day::from_key(&s.week_start),
        german_formal: Some(s.german_formal),
        registration: Some(s.registration_enabled),
        allowed_email_domains: match &s.allowed_email_domains {
            None => Some(None),
            Some(d) => check_domains("", d).is_ok().then(|| Some(d.clone())),
        },
        mfa_policy: Some(if s.mfa_required {
            MfaPolicy::Required
        } else {
            MfaPolicy::Optional
        }),
        company_link: match &s.company_link {
            None => Some(None),
            Some(l) => check_company_link("", l).is_ok().then(|| Some(l.clone())),
        },
        jitsi: s.jitsi(),
        oidc: off(s.oidc_is_off()),
        google_oauth2: off(s.google_is_off()),
        captcha: off(s.captcha_is_off()),
        meeting_webhook: off(s.webhook_is_off()),
        // A gateway in the environment is not the document's to clear.
        sms: off(s.sms.is_none() && !crate::sms::sms_env_active()),
        sms_allow_all_users: Some(s.sms_allow_all_users),
        ton: super::ton::export(ctx).await?,
        threecx: super::threecx::export(ctx).await?,
    }))
}

// --- Canonical values ---

/// `#` and six lowercase hex digits, as the colour picker posts it.
fn check_colours(colours: &CustomColors) -> Result<(), ApplyError> {
    for (name, value) in [
        ("accent", &colours.accent),
        ("accent_hover", &colours.accent_hover),
        ("bg", &colours.bg),
        ("surface", &colours.surface),
        ("text", &colours.text),
    ] {
        let hex = value.len() == 7
            && value.starts_with('#')
            && value[1..]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !hex {
            return Err(ApplyError::invalid(
                format!("site.custom.{name}"),
                format!("{value:?} is not a colour \"#rrggbb\" in lowercase hex"),
            ));
        }
    }
    Ok(())
}

/// A comma list of lowercase domain names, no spaces, no repeats: what
/// `is_email_allowed` compares with, stored as given.
fn check_domains(path: &str, value: &str) -> Result<(), ApplyError> {
    check_text(path, value, None)?;
    let mut seen = HashSet::new();
    for domain in value.split(',') {
        let label_ok = |label: &str| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        };
        if domain.len() > 253 || !domain.contains('.') || !domain.split('.').all(label_ok) {
            return Err(ApplyError::invalid(
                path,
                format!(
                    "{domain:?} is not a lowercase domain name; write a comma list without \
                     spaces, e.g. \"example.com,example.org\""
                ),
            ));
        }
        if !seen.insert(domain) {
            return Err(ApplyError::invalid(
                path,
                format!("{domain:?} is listed twice"),
            ));
        }
    }
    Ok(())
}

/// An absolute `http(s)` URL with a host, nothing `trim` drops: the web's
/// `is_safe_company_link`, and a URL a browser can open.
fn check_company_link(path: &str, value: &str) -> Result<(), ApplyError> {
    check_text(path, value, None)?;
    if !admin::is_safe_company_link(value) {
        return Err(ApplyError::invalid(
            path,
            "must start with http:// or https://",
        ));
    }
    http_url(path, value).map(|_| ())
}

/// The Jitsi server: an `http(s)` URL with a host and no query, fragment or
/// credentials, since a room name is joined to it with `/`.
fn check_jitsi(jitsi: &JitsiSettings) -> Result<(), ApplyError> {
    let path = "site.jitsi.base_url";
    check_text(path, &jitsi.base_url, None)?;
    let url = http_url(path, &jitsi.base_url)?;
    if url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(ApplyError::invalid(
            path,
            "must be the server's address only: no query, fragment or credentials",
        ));
    }
    check_opt_text("site.jitsi.pattern", jitsi.pattern.as_deref(), None)?;
    check_opt_text(
        "site.jitsi.display_name",
        jitsi.display_name.as_deref(),
        None,
    )
}

fn http_url(path: &str, value: &str) -> Result<reqwest::Url, ApplyError> {
    match reqwest::Url::parse(value) {
        Ok(url) if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() => Ok(url),
        _ => Err(ApplyError::invalid(
            path,
            format!("{value:?} is not an absolute http(s) URL with a host"),
        )),
    }
}

// --- What is stored ---

/// The columns of `auth_config` the section reads.
#[derive(sqlx::FromRow)]
struct Stored {
    theme: String,
    custom_accent: Option<String>,
    custom_accent_hover: Option<String>,
    custom_bg: Option<String>,
    custom_surface: Option<String>,
    custom_text: Option<String>,
    week_start: String,
    german_formal: bool,
    registration_enabled: bool,
    allowed_email_domains: Option<String>,
    mfa_required: bool,
    company_link: Option<String>,
    jitsi_base_url: Option<String>,
    jitsi_pattern: Option<String>,
    jitsi_display_name: Option<String>,
    oidc_enabled: bool,
    oidc_issuer_url: Option<String>,
    oidc_client_id: Option<String>,
    oidc_client_secret: Option<String>,
    oidc_auto_register: bool,
    google_oauth2_client_id: Option<String>,
    google_oauth2_client_secret: Option<String>,
    captcha_instance_url: Option<String>,
    captcha_site_key: Option<String>,
    captcha_secret: Option<String>,
    captcha_widget_url: Option<String>,
    captcha_wasm_url: Option<String>,
    meeting_webhook_url: Option<String>,
    meeting_webhook_auth_mode: Option<String>,
    meeting_webhook_secret: Option<String>,
    meeting_webhook_display_name: Option<String>,
    sms_allow_all_users: bool,
    base_url: Option<String>,
    allow_private_hosts: Option<String>,
    /// ilia fork: the stored SMS gateway (`sms_config`), the enabled row
    /// first; `None` when there is none.
    #[sqlx(skip)]
    sms: Option<SmsRow>,
}

/// The columns of an `sms_config` row the report names (`site.sms.*`).
#[derive(sqlx::FromRow)]
struct SmsRow {
    provider: String,
    api_key: Option<String>,
    api_secret_enc: Option<String>,
    sender: String,
    base_url: Option<String>,
}

impl Stored {
    async fn load(ctx: &Ctx) -> Result<Self, ApplyError> {
        let mut stored = sqlx::query_as::<_, Stored>(
            "SELECT theme, custom_accent, custom_accent_hover, custom_bg, custom_surface, \
             custom_text, week_start, german_formal, registration_enabled, \
             allowed_email_domains, mfa_required, company_link, jitsi_base_url, jitsi_pattern, \
             jitsi_display_name, oidc_enabled, oidc_issuer_url, oidc_client_id, \
             oidc_client_secret, oidc_auto_register, google_oauth2_client_id, \
             google_oauth2_client_secret, captcha_instance_url, captcha_site_key, \
             captcha_secret, captcha_widget_url, captcha_wasm_url, meeting_webhook_url, \
             meeting_webhook_auth_mode, meeting_webhook_secret, meeting_webhook_display_name, \
             sms_allow_all_users, base_url, allow_private_hosts \
             FROM auth_config WHERE id = 'singleton'",
        )
        .fetch_one(ctx.pool())
        .await?;
        stored.sms = sqlx::query_as::<_, SmsRow>(
            "SELECT provider, api_key, api_secret_enc, sender, base_url FROM sms_config \
             ORDER BY enabled DESC, id LIMIT 1",
        )
        .fetch_optional(ctx.pool())
        .await?;
        Ok(stored)
    }

    fn colours(&self) -> Option<CustomColors> {
        Some(CustomColors {
            accent: self.custom_accent.clone()?,
            accent_hover: self.custom_accent_hover.clone()?,
            bg: self.custom_bg.clone()?,
            surface: self.custom_surface.clone()?,
            text: self.custom_text.clone()?,
        })
    }

    /// Jitsi as the document says it, when it can.
    fn jitsi(&self) -> Option<Jitsi> {
        match (
            &self.jitsi_base_url,
            &self.jitsi_pattern,
            &self.jitsi_display_name,
        ) {
            (None, None, None) => Some(Jitsi::Off(Off::Off)),
            (Some(base_url), pattern, display_name) => {
                let jitsi = JitsiSettings {
                    base_url: base_url.clone(),
                    pattern: pattern.clone(),
                    display_name: display_name.clone(),
                };
                check_jitsi(&jitsi).is_ok().then_some(Jitsi::On(jitsi))
            }
            _ => None,
        }
    }

    /// Jitsi in the document's shape, or the raw columns.
    fn jitsi_value(&self) -> Value {
        if self.jitsi_base_url.is_none()
            && self.jitsi_pattern.is_none()
            && self.jitsi_display_name.is_none()
        {
            return json!("off");
        }
        json!({
            "base_url": self.jitsi_base_url,
            "pattern": self.jitsi_pattern,
            "display_name": self.jitsi_display_name,
        })
    }

    fn oidc_is_off(&self) -> bool {
        !self.oidc_enabled
            && self.oidc_issuer_url.is_none()
            && self.oidc_client_id.is_none()
            && self.oidc_client_secret.is_none()
    }

    fn google_is_off(&self) -> bool {
        self.google_oauth2_client_id.is_none() && self.google_oauth2_client_secret.is_none()
    }

    fn captcha_is_off(&self) -> bool {
        self.captcha_instance_url.is_none()
            && self.captcha_site_key.is_none()
            && self.captcha_secret.is_none()
            && self.captcha_widget_url.is_none()
            && self.captcha_wasm_url.is_none()
    }

    fn webhook_is_off(&self) -> bool {
        self.meeting_webhook_url.is_none()
            && self.meeting_webhook_auth_mode.is_none()
            && self.meeting_webhook_secret.is_none()
            && self.meeting_webhook_display_name.is_none()
    }
}

/// A secret column is never shown: `SECRET_SET`, or null.
fn secret(value: &Option<String>) -> Value {
    match value {
        Some(_) => json!(SECRET_SET),
        None => Value::Null,
    }
}

// --- Desired against stored ---

/// What posts a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Card {
    Theme,
    General,
    Auth,
    Mfa,
    CompanyLink,
    Jitsi,
    Oidc,
    Google,
    Captcha,
    Webhook,
    /// ilia fork: the SMS gateway's Clear button (row and secret at once).
    Sms,
    /// No card clears a secret: a statement does.
    Secret(Secret),
    SmsPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Secret {
    Oidc,
    Google,
    Captcha,
    Webhook,
}

impl Secret {
    const ALL: [Secret; 4] = [
        Secret::Oidc,
        Secret::Google,
        Secret::Captcha,
        Secret::Webhook,
    ];

    fn clear(self) -> &'static str {
        match self {
            Secret::Oidc => "UPDATE auth_config SET oidc_client_secret = NULL, updated_at = datetime('now') WHERE id = 'singleton'",
            Secret::Google => "UPDATE auth_config SET google_oauth2_client_secret = NULL, updated_at = datetime('now') WHERE id = 'singleton'",
            Secret::Captcha => "UPDATE auth_config SET captcha_secret = NULL, updated_at = datetime('now') WHERE id = 'singleton'",
            Secret::Webhook => "UPDATE auth_config SET meeting_webhook_secret = NULL, updated_at = datetime('now') WHERE id = 'singleton'",
        }
    }
}

/// One stored value against the document's, in the document's shape.
struct Pair {
    path: String,
    wanted: Value,
    stored: Value,
    card: Card,
    /// `calrs serve` caches it at startup.
    restart: bool,
}

impl Pair {
    fn differs(&self) -> bool {
        self.wanted != self.stored
    }
}

/// Every key the section gives, in document order. A feature switched `off`
/// is one pair per column, so a cleared secret reads `SECRET_SET` to null.
fn pairs(site: &Site, s: &Stored) -> Vec<Pair> {
    let mut out = Vec::new();
    let mut pair = |path: &str, wanted: Value, stored: Value, card: Card, restart: bool| {
        out.push(Pair {
            path: format!("site.{path}"),
            wanted,
            stored,
            card,
            restart,
        })
    };
    if let Some(theme) = site.theme {
        pair(
            "theme",
            json!(theme.key()),
            json!(s.theme),
            Card::Theme,
            true,
        );
    }
    if let Some(c) = &site.custom {
        for (name, wanted, stored) in [
            ("accent", &c.accent, &s.custom_accent),
            ("accent_hover", &c.accent_hover, &s.custom_accent_hover),
            ("bg", &c.bg, &s.custom_bg),
            ("surface", &c.surface, &s.custom_surface),
            ("text", &c.text, &s.custom_text),
        ] {
            let path = format!("custom.{name}");
            pair(&path, json!(wanted), json!(stored), Card::Theme, true);
        }
    }
    if let Some(day) = site.week_start {
        pair(
            "week_start",
            json!(day.key()),
            json!(s.week_start),
            Card::General,
            false,
        );
    }
    if let Some(formal) = site.german_formal {
        pair(
            "german_formal",
            json!(formal),
            json!(s.german_formal),
            Card::General,
            true,
        );
    }
    if let Some(open) = site.registration {
        pair(
            "registration",
            json!(open),
            json!(s.registration_enabled),
            Card::Auth,
            false,
        );
    }
    if let Some(domains) = &site.allowed_email_domains {
        pair(
            "allowed_email_domains",
            json!(domains),
            json!(s.allowed_email_domains),
            Card::Auth,
            false,
        );
    }
    if let Some(policy) = site.mfa_policy {
        let stored = if s.mfa_required {
            MfaPolicy::Required
        } else {
            MfaPolicy::Optional
        };
        pair(
            "mfa_policy",
            json!(policy.key()),
            json!(stored.key()),
            Card::Mfa,
            false,
        );
    }
    if let Some(link) = &site.company_link {
        pair(
            "company_link",
            json!(link),
            json!(s.company_link),
            Card::CompanyLink,
            true,
        );
    }
    if let Some(jitsi) = &site.jitsi {
        let wanted = serde_json::to_value(jitsi).unwrap_or(Value::Null);
        pair("jitsi", wanted, s.jitsi_value(), Card::Jitsi, true);
    }
    let null = || Value::Null;
    if site.oidc.is_some() {
        pair(
            "oidc.enabled",
            json!(false),
            json!(s.oidc_enabled),
            Card::Oidc,
            false,
        );
        for (name, stored) in [
            ("issuer_url", &s.oidc_issuer_url),
            ("client_id", &s.oidc_client_id),
        ] {
            let path = format!("oidc.{name}");
            pair(&path, null(), json!(stored), Card::Oidc, false);
        }
        pair(
            "oidc.client_secret",
            null(),
            secret(&s.oidc_client_secret),
            Card::Secret(Secret::Oidc),
            false,
        );
    }
    if site.google_oauth2.is_some() {
        pair(
            "google_oauth2.client_id",
            null(),
            json!(s.google_oauth2_client_id),
            Card::Google,
            false,
        );
        pair(
            "google_oauth2.client_secret",
            null(),
            secret(&s.google_oauth2_client_secret),
            Card::Secret(Secret::Google),
            false,
        );
    }
    if site.captcha.is_some() {
        for (name, stored) in [
            ("instance_url", &s.captcha_instance_url),
            ("site_key", &s.captcha_site_key),
            ("widget_url", &s.captcha_widget_url),
            ("wasm_url", &s.captcha_wasm_url),
        ] {
            let path = format!("captcha.{name}");
            pair(&path, null(), json!(stored), Card::Captcha, true);
        }
        pair(
            "captcha.secret",
            null(),
            secret(&s.captcha_secret),
            Card::Secret(Secret::Captcha),
            true,
        );
    }
    if site.meeting_webhook.is_some() {
        for (name, stored) in [
            ("url", &s.meeting_webhook_url),
            ("auth_mode", &s.meeting_webhook_auth_mode),
            ("display_name", &s.meeting_webhook_display_name),
        ] {
            let path = format!("meeting_webhook.{name}");
            pair(&path, null(), json!(stored), Card::Webhook, true);
        }
        pair(
            "meeting_webhook.secret",
            null(),
            secret(&s.meeting_webhook_secret),
            Card::Secret(Secret::Webhook),
            true,
        );
    }
    if site.sms.is_some() {
        // One pair per column the card shows; the row goes as a whole.
        let row = s.sms.as_ref();
        let text = |v: Option<&String>| match v {
            Some(v) if !v.is_empty() => json!(v),
            _ => Value::Null,
        };
        for (name, stored) in [
            ("provider", text(row.map(|r| &r.provider))),
            ("api_key", text(row.and_then(|r| r.api_key.as_ref()))),
            ("sender", text(row.map(|r| &r.sender))),
            ("base_url", text(row.and_then(|r| r.base_url.as_ref()))),
            (
                "api_secret",
                row.map_or(Value::Null, |r| secret(&r.api_secret_enc)),
            ),
        ] {
            let path = format!("sms.{name}");
            pair(&path, null(), stored, Card::Sms, false);
        }
    }
    if let Some(all) = site.sms_allow_all_users {
        pair(
            "sms_allow_all_users",
            json!(all),
            json!(s.sms_allow_all_users),
            Card::SmsPolicy,
            false,
        );
    }
    out
}

// --- Posting the cards ---

async fn post_theme(ctx: &Ctx, site: &Site) -> Result<(), ApplyError> {
    // `check` makes `theme` given whenever this card is posted, and `custom`
    // given exactly when the theme is custom.
    let theme = site.theme.unwrap_or(Theme::Default);
    let mut fields = vec![("theme", theme.key().to_string())];
    if let Some(c) = &site.custom {
        fields.extend([
            ("custom_accent", c.accent.clone()),
            ("custom_accent_hover", c.accent_hover.clone()),
            ("custom_bg", c.bg.clone()),
            ("custom_surface", c.surface.clone()),
            ("custom_text", c.text.clone()),
        ]);
    }
    let (headers, form) = super::form(&fields).await?;
    let response =
        admin::admin_update_accent(State(ctx.state.clone()), ctx.admin_user(), headers, form)
            .await
            .into_response();
    posted("site.theme", response).await
}

/// The General card, with the stored `base_url` and `allow_private_hosts`
/// posted back as the page shows them. The handler skips a field the
/// environment sets; otherwise it would store a normalised copy, and a
/// changed one is refused here, because those two belong to the
/// environment and `calrs config general`, not to the document.
async fn post_general(ctx: &Ctx, site: &Site, before: &Stored) -> Result<(), ApplyError> {
    let formal = site.german_formal.unwrap_or(before.german_formal);
    let week_start = site
        .week_start
        .map(|d| d.key().to_string())
        .unwrap_or_else(|| before.week_start.clone());
    let fields = [
        ("base_url", before.base_url.clone().unwrap_or_default()),
        (
            "allow_private_hosts",
            before.allow_private_hosts.clone().unwrap_or_default(),
        ),
        (
            "german_formal",
            if formal { "sie" } else { "du" }.to_string(),
        ),
        ("week_start", week_start),
    ];
    let path = if site
        .week_start
        .is_some_and(|d| d.key() != before.week_start)
    {
        "site.week_start"
    } else {
        "site.german_formal"
    };
    let (headers, form) = super::form(&fields).await?;
    let response =
        admin::admin_update_general(State(ctx.state.clone()), ctx.admin_user(), headers, form)
            .await
            .into_response();
    posted(path, response).await?;

    let after = Stored::load(ctx).await?;
    for (name, was, is) in [
        ("base_url", &before.base_url, &after.base_url),
        (
            "allow_private_hosts",
            &before.allow_private_hosts,
            &after.allow_private_hosts,
        ),
    ] {
        if was != is {
            return Err(ApplyError::refused(
                path,
                format!(
                    "saving the General card would also change the stored {name} from {} to \
                     {}; set it with the environment or `calrs config general` first",
                    json!(was),
                    json!(is)
                ),
            ));
        }
    }
    Ok(())
}

async fn post_auth(ctx: &Ctx, site: &Site, before: &Stored) -> Result<(), ApplyError> {
    let open = site.registration.unwrap_or(before.registration_enabled);
    let domains = match &site.allowed_email_domains {
        Some(wanted) => wanted.clone(),
        None => before.allowed_email_domains.clone(),
    };
    let mut fields = vec![("allowed_email_domains", domains.unwrap_or_default())];
    if open {
        fields.push(("registration_enabled", "on".to_string()));
    }
    let path = if site
        .registration
        .is_some_and(|r| r != before.registration_enabled)
    {
        "site.registration"
    } else {
        "site.allowed_email_domains"
    };
    let (headers, form) = super::form(&fields).await?;
    let response =
        admin::admin_update_auth(State(ctx.state.clone()), ctx.admin_user(), headers, form)
            .await
            .into_response();
    posted(path, response).await
}

/// `answered`, and also a refusal the admin cards give as a redirect back
/// to the page with `?error=<message>` (an invalid base URL, company link
/// or captcha WASM URL).
async fn posted(path: &str, response: Response) -> Result<(), ApplyError> {
    if response.status().is_redirection() {
        let location = response
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok());
        if let Some(message) = location.and_then(redirect_error) {
            return Err(ApplyError::refused(
                path,
                format!("the web form refused it: {message}"),
            ));
        }
    }
    super::answered(path, response).await
}

/// The `error` query parameter of a redirect target, decoded.
fn redirect_error(location: &str) -> Option<String> {
    let (_, query) = location.split_once('?')?;
    let raw = query.split('&').find_map(|p| p.strip_prefix("error="))?;
    let message = urlencoding::decode(&raw.replace('+', " "))
        .map(|m| m.into_owned())
        .unwrap_or_else(|_| raw.to_string());
    Some(if message.trim().is_empty() {
        "an error without a message".to_string()
    } else {
        message
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_redirect_with_an_error_is_a_refusal() {
        assert_eq!(
            redirect_error("/dashboard/admin?error=Invalid%20base%20URL").as_deref(),
            Some("Invalid base URL")
        );
        assert_eq!(
            redirect_error("/dashboard/admin?tab=x&error=a+b").as_deref(),
            Some("a b")
        );
        assert_eq!(redirect_error("/dashboard/admin"), None);
        assert_eq!(redirect_error("/dashboard/admin?resource_notice=ok"), None);
    }

    /// The ilia-booking role greps for this exact text
    /// (`calrs_apply_sms_off_mark` in its vars/main.yml): change both or
    /// neither.
    #[test]
    fn the_sms_off_marker_is_the_one_the_role_greps_for() {
        assert_eq!(
            SMS_OFF_SHAPE,
            "site.sms: \"off\" deletes the stored SMS gateway"
        );
        // resolve.yml greps the commit's source with -F for the plain text.
        let source = include_str!("site.rs");
        assert!(source.contains(&format!("r#\"{SMS_OFF_SHAPE}\"#")));
    }

    #[test]
    fn domains_must_be_a_canonical_comma_list() {
        for good in [
            "ilia.ae",
            "ilia.ae,example.org",
            "xn--80ak6aa92e.com",
            "a-b.co",
        ] {
            assert!(check_domains("p", good).is_ok(), "{good}");
        }
        for bad in [
            "Ilia.ae",
            "ilia.ae, example.org",
            "ilia.ae,,example.org",
            "ilia.ae,",
            "localhost",
            "-a.com",
            "a..com",
            "ilia.ae;example.org",
            "ilia.ae,ilia.ae",
            " ilia.ae",
            "",
        ] {
            assert!(check_domains("p", bad).is_err(), "{bad}");
        }
    }
}
