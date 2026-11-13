//! ilia fork: `config apply`, `site.threecx`: the admin page's 3CX card
//! (`web::threecx`).
//!
//! `"off"` removes the settings, the key with them. An object sets the PBX
//! address, the extension and the label; its `api_key` is the key to store,
//! or null to keep the stored one, so the key can be the admin page's to
//! change (owner's request 2026-11-13: "changing it in the web must work
//! too"). The key is never exported and is reported as `"(secret set)"`.
//! `calrs serve` caches the settings, so a change asks for a restart.

use super::{ApplyError, Ctx, SECRET_SET};
use crate::web::threecx;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `site.threecx`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum ThreeCx {
    Off(super::Off),
    On(ThreeCxSettings),
}

/// Every key but `api_key` must be given; `display_name` null shows the
/// generic label, `api_key` null (or left out) keeps the stored key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ThreeCxSettings {
    pub(crate) base_url: String,
    pub(crate) extension: String,
    #[serde(deserialize_with = "super::given")]
    pub(crate) display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) api_key: Option<String>,
}

const PATH: &str = "site.threecx";

/// The marker of a calrs whose `config apply` knows `site.threecx`, part of
/// every refusal of the section (so it is in the binary). The ilia-booking
/// role greps the commit and the release for it before it sends the key.
pub(crate) const THREECX_SHAPE: &str =
    r#"site.threecx: "off" or {base_url, extension, display_name, api_key|null}"#;

fn invalid(field: &str, reason: impl std::fmt::Display) -> ApplyError {
    ApplyError::invalid(
        format!("{PATH}.{field}"),
        format!("{reason} ({THREECX_SHAPE})"),
    )
}

/// The stored settings in the document's shape (the key as whether one is
/// stored, and whether it equals `wanted_key`).
struct Stored {
    shown: threecx::Shown,
    key: Option<String>,
}

async fn stored(ctx: &Ctx) -> Result<Stored, ApplyError> {
    let shown = threecx::shown(ctx.pool()).await;
    let enc: Option<String> =
        sqlx::query_scalar("SELECT threecx_api_key_enc FROM auth_config WHERE id = 'singleton'")
            .fetch_optional(ctx.pool())
            .await?
            .flatten();
    let key = enc.and_then(|e| crate::crypto::decrypt_value(&ctx.state.secret_key, &e).ok());
    Ok(Stored { shown, key })
}

/// Reading only: each value as the card would store it.
pub(super) async fn check(_ctx: &Ctx, section: &ThreeCx) -> Result<(), ApplyError> {
    let ThreeCx::On(s) = section else {
        return Ok(());
    };
    let canonical = threecx::check_base_url(&s.base_url).map_err(|r| invalid("base_url", r))?;
    if canonical != s.base_url {
        return Err(invalid(
            "base_url",
            format!("must be written as stored: {canonical:?}"),
        ));
    }
    let ext = threecx::check_extension(&s.extension).map_err(|r| invalid("extension", r))?;
    if ext != s.extension {
        return Err(invalid("extension", "must be digits only, without spaces"));
    }
    if let Some(name) = &s.display_name {
        let checked = threecx::check_display_name(name).map_err(|r| invalid("display_name", r))?;
        if checked.as_deref() != Some(name.as_str()) {
            return Err(invalid(
                "display_name",
                "must not be empty or padded (null shows the generic label)",
            ));
        }
    }
    if let Some(key) = &s.api_key {
        let checked = threecx::check_api_key(key).map_err(|r| invalid("api_key", r))?;
        if &checked != key {
            return Err(invalid("api_key", "must not be padded"));
        }
    }
    Ok(())
}

fn doc_of(stored: &Stored) -> Value {
    if stored.shown.base_url.is_empty() && !stored.shown.key_set {
        return Value::String("off".to_string());
    }
    serde_json::json!({
        "base_url": stored.shown.base_url,
        "extension": stored.shown.extension,
        "display_name": (!stored.shown.display_name.is_empty()).then(|| stored.shown.display_name.clone()),
        "api_key": stored.shown.key_set.then_some(SECRET_SET),
    })
}

fn differs(section: &ThreeCx, stored: &Stored) -> bool {
    match section {
        ThreeCx::Off(_) => !stored.shown.base_url.is_empty() || stored.shown.key_set,
        ThreeCx::On(s) => {
            stored.shown.base_url != s.base_url
                || stored.shown.extension != s.extension
                || stored.shown.display_name != s.display_name.clone().unwrap_or_default()
                || s.api_key
                    .as_ref()
                    .is_some_and(|k| stored.key.as_ref() != Some(k))
        }
    }
}

fn wanted_doc(section: &ThreeCx, stored: &Stored) -> Value {
    match section {
        ThreeCx::Off(_) => Value::String("off".to_string()),
        ThreeCx::On(s) => serde_json::json!({
            "base_url": s.base_url,
            "extension": s.extension,
            "display_name": s.display_name,
            "api_key": (s.api_key.is_some() || stored.shown.key_set).then_some(SECRET_SET),
        }),
    }
}

/// Store the section when it differs, and record the change.
pub(super) async fn apply(ctx: &mut Ctx, section: &ThreeCx) -> Result<(), ApplyError> {
    let before = stored(ctx).await?;
    if !differs(section, &before) {
        return Ok(());
    }
    let update = match section {
        ThreeCx::Off(_) => threecx::Update {
            base_url: None,
            extension: None,
            display_name: None,
            api_key: None,
            clear: true,
        },
        ThreeCx::On(s) => threecx::Update {
            base_url: Some(s.base_url.clone()),
            extension: Some(s.extension.clone()),
            display_name: s.display_name.clone(),
            api_key: s.api_key.clone(),
            clear: false,
        },
    };
    match threecx::store(ctx.pool(), &ctx.state.secret_key, update).await {
        Ok(()) => {}
        Err(threecx::StoreError::Refused(r)) => return Err(invalid(r.field, r.reason)),
        Err(threecx::StoreError::Internal(e)) => {
            return Err(ApplyError::internal("store the 3CX settings", e))
        }
    }
    let wanted = wanted_doc(section, &before);
    ctx.record_restart(PATH, doc_of(&before), wanted);
    Ok(())
}

/// Read back: the stored settings are the section's.
pub(super) async fn verify(ctx: &Ctx, section: &ThreeCx) -> Result<(), ApplyError> {
    let have = stored(ctx).await?;
    if differs(section, &have) {
        return Err(ApplyError::stored_differs(
            PATH,
            &wanted_doc(section, &have),
            &doc_of(&have),
        ));
    }
    Ok(())
}

/// The stored settings without the key (`api_key` left out keeps it), or
/// `"off"`; nothing when what is stored is not something the checks accept.
pub(super) async fn export(ctx: &Ctx) -> Result<Option<ThreeCx>, ApplyError> {
    let s = stored(ctx).await?;
    if s.shown.base_url.is_empty() && s.shown.extension.is_empty() && !s.shown.key_set {
        return Ok(Some(ThreeCx::Off(super::Off::Off)));
    }
    let section = ThreeCx::On(ThreeCxSettings {
        base_url: s.shown.base_url.clone(),
        extension: s.shown.extension.clone(),
        display_name: (!s.shown.display_name.is_empty()).then(|| s.shown.display_name.clone()),
        api_key: None,
    });
    Ok(check(ctx, &section).await.is_ok().then_some(section))
}
