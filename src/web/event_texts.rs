//! ilia fork: an event type's title and description per language (owner's
//! request 2026-11-13: the widget on the Russian page of ilia.ae spoke
//! Russian, but the event type itself still said "CTO Consultation").
//!
//! calrs stores one title and one description per event type. The table
//! `event_type_texts` (migration `ilia_015_event_type_texts.sql`) adds, per
//! shipped locale (`i18n::supported_code`), a title and an optional
//! description. What a reader of language `lang` sees ([`localize`]):
//!
//! 1. the row for `lang` itself;
//! 2. else the row for its base language (`de-formal` reads `de`,
//!    [`base_lang`]);
//! 3. else the event type's own `title` and `description`.
//!
//! A row is one text: its description, NULL included, goes with its title
//! (a translated title is never followed by the base description in another
//! language). Without rows nothing changes anywhere: every caller passes the
//! base text in and gets it back as it was.
//!
//! Guest surfaces read it in the page language (profile and team listings,
//! slot pages, the booking form and its answer, the guest token pages, the
//! TON payment page) or in the booking's stored language (guest emails and
//! their `.ics`, SMS, Telegram: [`for_guest`]). Host surfaces keep the base
//! title. The dashboard form edits the rows ([`TextsForm`], the
//! "Translations" section, `templates/partials/ilia_event_texts_form.html`),
//! `config apply` declares them (`event_types[].translations`,
//! `src/web/apply/event_types.rs`), and a duplicate copies them
//! (`COPIED_CHILD_TABLES`).

use minijinja::{context, Value};
use serde::Deserialize;
use sqlx::{SqliteConnection, SqlitePool};
use std::collections::{BTreeMap, HashMap};

/// The form field that marks the section: present when the form carried
/// it. Without it a save leaves the stored rows as they are (MCP and
/// `config apply` post the event type form without the section).
const MARKER: &str = "translations";
/// The fields of row `n`: `translation_lang_<n>` and so on.
const LANG_FIELD: &str = "translation_lang_";
const TITLE_FIELD: &str = "translation_title_";
const DESCRIPTION_FIELD: &str = "translation_description_";

/// One stored translation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Text {
    pub(crate) lang: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
}

// --- Resolution ---

/// The language a locale falls back to before the base text: `de-formal`
/// reads `de`. `None` for a locale without one.
pub(crate) fn base_lang(lang: &str) -> Option<&'static str> {
    lang.split_once('-')
        .and_then(|(base, _)| crate::i18n::supported_code(base))
}

/// The translation a reader of `lang` sees for event type `et_id`: the row
/// for `lang`, else for its base language; `None` when neither exists.
pub(crate) async fn translation(
    pool: &SqlitePool,
    et_id: &str,
    lang: &str,
) -> Option<(String, Option<String>)> {
    let base = base_lang(lang).unwrap_or(lang);
    sqlx::query_as(
        "SELECT title, description FROM event_type_texts
         WHERE event_type_id = ? AND lang IN (?, ?)
         ORDER BY lang = ? DESC LIMIT 1",
    )
    .bind(et_id)
    .bind(lang)
    .bind(base)
    .bind(lang)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "cannot read event type translations");
        None
    })
}

/// The title and description a reader of `lang` sees: the translation
/// ([`translation`]), else `title` and `description` as they are.
pub(crate) async fn localize(
    pool: &SqlitePool,
    et_id: &str,
    lang: &str,
    title: String,
    description: Option<String>,
) -> (String, Option<String>) {
    match translation(pool, et_id, lang).await {
        Some(text) => text,
        None => (title, description),
    }
}

/// [`localize`] for the title alone.
pub(crate) async fn title_in(pool: &SqlitePool, et_id: &str, lang: &str, title: String) -> String {
    match translation(pool, et_id, lang).await {
        Some((title, _)) => title,
        None => title,
    }
}

/// [`localize`] for a list of event types in one query: `parts` gives a
/// row's event type id, title and description, which are replaced in place.
pub(crate) async fn localize_rows<T>(
    pool: &SqlitePool,
    lang: &str,
    mut rows: Vec<T>,
    parts: impl Fn(&mut T) -> (&str, &mut String, &mut Option<String>),
) -> Vec<T> {
    if rows.is_empty() {
        return rows;
    }
    let base = base_lang(lang).unwrap_or(lang);
    let ids: Vec<String> = rows
        .iter_mut()
        .map(|row| parts(row).0.to_string())
        .collect();
    let ids_json = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());
    let found: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT event_type_id, lang, title, description FROM event_type_texts
         WHERE lang IN (?, ?) AND event_type_id IN (SELECT value FROM json_each(?))",
    )
    .bind(lang)
    .bind(base)
    .bind(&ids_json)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "cannot read event type translations");
        Vec::new()
    });
    let mut texts: HashMap<String, (bool, String, Option<String>)> = HashMap::new();
    for (et_id, row_lang, title, description) in found {
        let exact = row_lang == lang;
        match texts.get(&et_id) {
            Some((true, _, _)) => {}
            Some(_) if !exact => {}
            _ => {
                texts.insert(et_id, (exact, title, description));
            }
        }
    }
    for row in rows.iter_mut() {
        let (et_id, title, description) = parts(row);
        if let Some((_, t, d)) = texts.get(et_id) {
            *title = t.clone();
            *description = d.clone();
        }
    }
    rows
}

// --- What guests are sent ---

/// Email details a guest reads: the booking they are about, the language
/// they are written in, the event title and (`user_texts`) the host's name
/// in them.
pub(crate) trait GuestCopy: Clone {
    fn booking_uid(&self) -> &str;
    fn reader_lang(&self) -> Option<&str>;
    fn event_title_mut(&mut self) -> &mut String;
    fn host_name_mut(&mut self) -> &mut String;
}

impl GuestCopy for crate::email::BookingDetails {
    fn booking_uid(&self) -> &str {
        &self.uid
    }
    fn reader_lang(&self) -> Option<&str> {
        self.guest_language.as_deref()
    }
    fn event_title_mut(&mut self) -> &mut String {
        &mut self.event_title
    }
    fn host_name_mut(&mut self) -> &mut String {
        &mut self.host_name
    }
}

impl GuestCopy for crate::email::CancellationDetails {
    fn booking_uid(&self) -> &str {
        &self.uid
    }
    fn reader_lang(&self) -> Option<&str> {
        self.guest_language.as_deref()
    }
    fn event_title_mut(&mut self) -> &mut String {
        &mut self.event_title
    }
    fn host_name_mut(&mut self) -> &mut String {
        &mut self.host_name
    }
}

impl GuestCopy for crate::email::RescheduleDetails {
    fn booking_uid(&self) -> &str {
        &self.uid
    }
    fn reader_lang(&self) -> Option<&str> {
        self.guest_language.as_deref()
    }
    fn event_title_mut(&mut self) -> &mut String {
        &mut self.event_title
    }
    fn host_name_mut(&mut self) -> &mut String {
        &mut self.host_name
    }
}

/// The language a guest email is written in: the stored booking language
/// when calrs ships it, else English (as `email::guest_lang`).
pub(crate) fn email_lang(stored: Option<&str>) -> &'static str {
    stored.and_then(crate::i18n::supported_code).unwrap_or("en")
}

/// The copy of `details` for the guest (their email and its `.ics`, the
/// copies to additional attendees, the SMS): the event title of the
/// booking's event type in the language those are written in, and the
/// host's name too (`user_texts::host_for_guest`). Host emails keep
/// `details` as it is. Unchanged when there is no translation.
pub(crate) async fn for_guest<D: GuestCopy>(pool: &SqlitePool, details: &D) -> D {
    let mut copy = details.clone();
    let et_id: Option<String> =
        sqlx::query_scalar("SELECT event_type_id FROM bookings WHERE uid = ?")
            .bind(details.booking_uid())
            .fetch_optional(pool)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "cannot read a booking's event type");
                None
            });
    if let Some(et_id) = et_id {
        let lang = email_lang(details.reader_lang());
        if let Some((title, _)) = translation(pool, &et_id, lang).await {
            *copy.event_title_mut() = title;
        }
    }
    super::user_texts::host_for_guest(pool, copy).await
}

/// [`for_guest`] for details built without the guest's language (the
/// guest `.ics` downloads): the booking's stored language, as its emails,
/// for the title and the host's name.
pub(crate) async fn in_booking_language<D: GuestCopy>(pool: &SqlitePool, details: &D) -> D {
    let mut copy = details.clone();
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT event_type_id, language FROM bookings WHERE uid = ?")
            .bind(details.booking_uid())
            .fetch_optional(pool)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "cannot read a booking's event type");
                None
            });
    if let Some((et_id, language)) = row {
        let lang = email_lang(language.as_deref());
        if let Some((title, _)) = translation(pool, &et_id, lang).await {
            *copy.event_title_mut() = title;
        }
        // ilia fork (`user_texts`): the host's name in that language too.
        let shown = std::mem::take(copy.host_name_mut());
        *copy.host_name_mut() =
            super::user_texts::booking_host_name(pool, details.booking_uid(), lang, shown).await;
    }
    copy
}

/// The token a guest page was opened with.
pub(crate) enum Token<'a> {
    Cancel(&'a str),
    Reschedule(&'a str),
}

/// The event title a guest page opened with `token` shows in `lang`
/// (`base` when there is no translation, or no such booking).
pub(crate) async fn token_title(
    pool: &SqlitePool,
    token: Token<'_>,
    lang: &str,
    base: String,
) -> String {
    let (sql, value) = match token {
        Token::Cancel(t) => (
            "SELECT event_type_id FROM bookings WHERE cancel_token = ?",
            t,
        ),
        Token::Reschedule(t) => (
            "SELECT event_type_id FROM bookings WHERE reschedule_token = ?",
            t,
        ),
    };
    let et_id: Option<String> = sqlx::query_scalar(sql)
        .bind(value)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
    match et_id {
        Some(et_id) => title_in(pool, &et_id, lang, base).await,
        None => base,
    }
}

// --- Storage ---

/// Every stored translation of `et_id`, by language.
pub(crate) async fn stored(pool: &SqlitePool, et_id: &str) -> sqlx::Result<Vec<Text>> {
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT lang, title, description FROM event_type_texts
         WHERE event_type_id = ? ORDER BY lang",
    )
    .bind(et_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(lang, title, description)| Text {
            lang,
            title,
            description,
        })
        .collect())
}

/// [`stored`] for a page: a failed read shows none (and is logged).
pub(crate) async fn load(pool: &SqlitePool, et_id: &str) -> Vec<Text> {
    stored(pool, et_id).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "cannot read event type translations");
        Vec::new()
    })
}

/// Replace every translation of `et_id` with `texts`, on `conn` (the
/// caller's transaction, or `config apply`'s connection).
pub(crate) async fn replace(
    conn: &mut SqliteConnection,
    et_id: &str,
    texts: &[Text],
) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM event_type_texts WHERE event_type_id = ?")
        .bind(et_id)
        .execute(&mut *conn)
        .await?;
    for text in texts {
        sqlx::query(
            "INSERT INTO event_type_texts (event_type_id, lang, title, description)
             VALUES (?, ?, ?, ?)",
        )
        .bind(et_id)
        .bind(&text.lang)
        .bind(&text.title)
        .bind(&text.description)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

// --- The event type form ---

/// The "Translations" section of the event type form, flattened into
/// `EventTypeForm`: the marker and every field the form struct does not
/// name, of which the rows' `translation_lang_<n>`, `translation_title_<n>`
/// and `translation_description_<n>` are read.
#[derive(Default, Deserialize)]
pub(crate) struct TextsForm {
    translations: Option<String>,
    #[serde(flatten)]
    fields: HashMap<String, String>,
}

impl TextsForm {
    /// Whether the form carried the section.
    pub(crate) fn given(&self) -> bool {
        self.translations.is_some()
    }

    /// The rows as posted, by their number: (language, title, description).
    fn rows(&self) -> BTreeMap<u32, (String, String, String)> {
        let mut rows: BTreeMap<u32, (String, String, String)> = BTreeMap::new();
        for (key, value) in &self.fields {
            for (prefix, slot) in [(LANG_FIELD, 0), (TITLE_FIELD, 1), (DESCRIPTION_FIELD, 2)] {
                let Some(n) = key.strip_prefix(prefix).and_then(|n| n.parse::<u32>().ok()) else {
                    continue;
                };
                let row = rows.entry(n).or_default();
                match slot {
                    0 => row.0 = value.clone(),
                    1 => row.1 = value.clone(),
                    _ => row.2 = value.clone(),
                }
            }
        }
        rows
    }
}

/// The rows of a posted section, checked as the base title and description
/// are (`validate_event_type_form`): a title is required (trimmed, at most
/// `MAX_EVENT_TYPE_TITLE` characters), a blank description is none. A row
/// left with neither is dropped (the way to remove one without JavaScript).
/// The language must be a shipped locale, each at most once. `Ok(None)`
/// when the form did not carry the section. Errors are Fluent keys.
pub(crate) fn parse_form(form: &TextsForm) -> Result<Option<Vec<Text>>, &'static str> {
    if !form.given() {
        return Ok(None);
    }
    let mut texts: Vec<Text> = Vec::new();
    for (lang, title, description) in form.rows().into_values() {
        let title = title.trim();
        let description = Some(description).filter(|d| !d.trim().is_empty());
        if title.is_empty() && description.is_none() {
            continue;
        }
        let Some(lang) = crate::i18n::supported_code(&lang) else {
            return Err("form-error-translation-language");
        };
        if title.is_empty() {
            return Err("form-error-translation-title-required");
        }
        if title.chars().count() > super::MAX_EVENT_TYPE_TITLE {
            return Err("form-error-translation-title-length");
        }
        if texts.iter().any(|t| t.lang == lang) {
            return Err("form-error-translation-duplicate");
        }
        texts.push(Text {
            lang: lang.to_string(),
            title: title.to_string(),
            description,
        });
    }
    texts.sort_by(|a, b| a.lang.cmp(&b.lang));
    Ok(Some(texts))
}

/// Store what the form's section says for `et_id`, in one transaction.
/// Nothing when the form did not carry it. The handler checked it first
/// (`validate_event_type_form`), so a refusal here is only logged.
pub(crate) async fn save_form(pool: &SqlitePool, et_id: &str, form: &TextsForm) {
    let texts = match parse_form(form) {
        Ok(Some(texts)) => texts,
        Ok(None) => return,
        Err(key) => {
            tracing::warn!(key, "event type translations not saved");
            return;
        }
    };
    let saved = async {
        let mut tx = pool.begin().await?;
        replace(&mut tx, et_id, &texts).await?;
        tx.commit().await
    }
    .await;
    if let Err(e) = saved {
        tracing::error!(error = %e, et_id, "cannot save event type translations");
    }
}

/// One row of the section, as the template shows it.
fn row_ctx(n: usize, lang: &str, title: &str, description: &str) -> Value {
    context! { n => n, lang => lang, title => title, description => description }
}

/// The section's template context (`event_texts`): the rows (stored for
/// `et_id`, none for a new event type) and every shipped language by its
/// own name.
pub(crate) async fn form_ctx(pool: &SqlitePool, et_id: Option<&str>) -> Value {
    let texts = match et_id {
        Some(et_id) => load(pool, et_id).await,
        None => Vec::new(),
    };
    let rows: Vec<Value> = texts
        .iter()
        .enumerate()
        .map(|(n, t)| row_ctx(n, &t.lang, &t.title, t.description.as_deref().unwrap_or("")))
        .collect();
    section_ctx(rows)
}

/// The section of a form shown again after a refusal: the rows as posted
/// when the form carried the section (so nothing typed is lost), else what
/// [`form_ctx`] shows.
pub(crate) async fn posted_ctx(pool: &SqlitePool, form: &TextsForm, et_id: Option<&str>) -> Value {
    if !form.given() {
        return form_ctx(pool, et_id).await;
    }
    let rows: Vec<Value> = form
        .rows()
        .into_values()
        .enumerate()
        .map(|(n, (lang, title, description))| row_ctx(n, &lang, &title, &description))
        .collect();
    section_ctx(rows)
}

fn section_ctx(rows: Vec<Value>) -> Value {
    let languages: Vec<Value> = crate::i18n::supported_with_labels()
        .map(|(code, label)| context! { code => code, label => label })
        .collect();
    context! { rows => rows, languages => languages }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The section as `axum::Form` parses a posted form.
    async fn form(pairs: &[(&str, &str)]) -> TextsForm {
        let fields: Vec<(&str, String)> = pairs.iter().map(|(k, v)| (*k, v.to_string())).collect();
        let (_, axum::Form(form)) = crate::web::apply::form::<TextsForm>(&fields).await.unwrap();
        form
    }

    #[test]
    fn the_base_language_of_a_register_is_its_language() {
        assert_eq!(base_lang("de-formal"), Some("de"));
        assert_eq!(base_lang("de"), None);
        assert_eq!(base_lang("ru"), None);
        assert_eq!(base_lang("xx-yy"), None);
    }

    #[tokio::test]
    async fn a_form_without_the_section_changes_nothing() {
        assert_eq!(parse_form(&form(&[("title", "X")]).await), Ok(None));
    }

    #[tokio::test]
    async fn rows_are_read_in_order_trimmed_and_sorted_by_language() {
        let posted = form(&[
            ("translations", "1"),
            ("translation_lang_3", "ru"),
            ("translation_title_3", "  Консультация  "),
            ("translation_description_3", "Пятнадцать минут"),
            ("translation_lang_0", "de"),
            ("translation_title_0", "Beratung"),
            ("translation_description_0", "  "),
            // Left empty: removed.
            ("translation_lang_7", "fr"),
            ("translation_title_7", ""),
        ])
        .await;
        assert_eq!(
            parse_form(&posted),
            Ok(Some(vec![
                Text {
                    lang: "de".into(),
                    title: "Beratung".into(),
                    description: None,
                },
                Text {
                    lang: "ru".into(),
                    title: "Консультация".into(),
                    description: Some("Пятнадцать минут".into()),
                },
            ]))
        );
        assert_eq!(
            parse_form(&form(&[("translations", "1")]).await),
            Ok(Some(vec![]))
        );
    }

    #[tokio::test]
    async fn bad_rows_are_refused_with_their_key() {
        async fn one(lang: &str, title: &str, description: &str) -> Result<usize, &'static str> {
            let posted = form(&[
                ("translations", "1"),
                ("translation_lang_0", lang),
                ("translation_title_0", title),
                ("translation_description_0", description),
            ])
            .await;
            parse_form(&posted).map(|texts| texts.map_or(0, |t| t.len()))
        }
        assert_eq!(
            one("xx", "T", "").await,
            Err("form-error-translation-language")
        );
        assert_eq!(
            one("", "T", "").await,
            Err("form-error-translation-language")
        );
        assert_eq!(
            one("ru", "", "D").await,
            Err("form-error-translation-title-required")
        );
        assert_eq!(
            one("ru", &"я".repeat(256), "").await,
            Err("form-error-translation-title-length")
        );
        assert_eq!(one("ru", &"я".repeat(255), "").await, Ok(1));
        let twice = form(&[
            ("translations", "1"),
            ("translation_lang_0", "ru"),
            ("translation_title_0", "A"),
            ("translation_lang_1", "ru"),
            ("translation_title_1", "B"),
        ])
        .await;
        assert_eq!(parse_form(&twice), Err("form-error-translation-duplicate"));
    }
}
