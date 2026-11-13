//! ilia fork: a user's public profile (name, title, bio) per language
//! (owner's report 2026-11-13: the widget on the Russian page of ilia.ae
//! spoke Russian, the event type too, but its header still read "Ilia
//! Arestov" / "Fractional CTO"; the owner's Russian page calls him «Илья
//! Арестов», «Технический директор»). The sibling of `event_texts`.
//!
//! calrs stores one `name`, `title` and `bio` per user. The table
//! `user_texts` (migration `ilia_017_user_texts.sql`) adds, per shipped
//! locale (`i18n::supported_code`), an optional name, title and bio. **Each
//! field falls back on its own** ([`translation`]): what a reader of `lang`
//! sees as a field is
//!
//! 1. the value of the row for `lang` itself, when it has one;
//! 2. else the value of the row for its base language (`de-formal` reads
//!    `de`, `event_texts::base_lang`), when it has one;
//! 3. else the user's own value.
//!
//! Unlike an event type's translation, whose description goes with its
//! title, the three are independent: a Russian name and title with no
//! Russian bio show the base bio (a person's bio in another language is
//! still about them; an event type's description under a translated title
//! would describe it twice). Without rows nothing changes anywhere: every
//! caller passes the base value in and gets it back as it was.
//!
//! Guest pages read it in the page language (the profile, the slot pages
//! and booking forms, the confirmation page, the guest token pages, the TON
//! payment page, team members on team pages); what is sent to the guest, in
//! the booking's stored language (guest emails and their `.ics`, Telegram):
//! [`host_for_guest`], which `event_texts::for_guest` calls. Only a booking
//! with one host is translated ([`booking_host_name`]): a multi-host booking
//! (collective team, dynamic group) names its stored organizer roster
//! (#161), which the `.ics` ORGANIZER and every later email must repeat.
//! Host surfaces keep the base values. The dashboard edits the rows in
//! Profile & Settings ([`TextsForm`], the "Translations" section,
//! `templates/partials/ilia_profile_texts_form.html`), `config apply`
//! declares them (`profile.translations`, `src/web/apply/profile.rs`).

use super::event_texts::{base_lang, email_lang, GuestCopy, Token};
use minijinja::{context, Value};
use serde::Deserialize;
use sqlx::{SqliteConnection, SqlitePool};
use std::collections::{BTreeMap, HashMap};

/// The form field that marks the section. Without it a save leaves the
/// stored rows as they are (`config apply` posts the settings form without
/// the section).
const MARKER: &str = "profile_translations";
/// The fields of row `n`: `profile_translation_lang_<n>` and so on.
const LANG_FIELD: &str = "profile_translation_lang_";
const NAME_FIELD: &str = "profile_translation_name_";
const TITLE_FIELD: &str = "profile_translation_title_";
const BIO_FIELD: &str = "profile_translation_bio_";
/// The base name's limit (`dashboard::settings_save`: 1..=255 bytes).
pub(crate) const NAME_MAX_BYTES: usize = 255;

/// One stored translation: at least one field is `Some`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Text {
    pub(crate) lang: String,
    pub(crate) name: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) bio: Option<String>,
}

// --- Resolution ---

/// What a reader of one language sees in place of each field: `Some`
/// where a translation applies (the language's row, else its base
/// language's), `None` where the base value shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Shown {
    pub(crate) name: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) bio: Option<String>,
}

type Row = (String, Option<String>, Option<String>, Option<String>);

/// The rows of `user_id` for `lang` and its base, as [`Shown`].
fn resolve(lang: &str, rows: Vec<Row>) -> Shown {
    let mut exact = Shown::default();
    let mut base = Shown::default();
    for (row_lang, name, title, bio) in rows {
        let into = if row_lang == lang {
            &mut exact
        } else {
            &mut base
        };
        *into = Shown { name, title, bio };
    }
    Shown {
        name: exact.name.or(base.name),
        title: exact.title.or(base.title),
        bio: exact.bio.or(base.bio),
    }
}

/// The translation a reader of `lang` sees for `user_id`, field by field.
pub(crate) async fn translation(pool: &SqlitePool, user_id: &str, lang: &str) -> Shown {
    let base = base_lang(lang).unwrap_or(lang);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT lang, name, title, bio FROM user_texts
         WHERE user_id = ? AND lang IN (?, ?)",
    )
    .bind(user_id)
    .bind(lang)
    .bind(base)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "cannot read profile translations");
        Vec::new()
    });
    resolve(lang, rows)
}

/// The name, title and bio a reader of `lang` sees: each field's
/// translation, else the value passed in.
pub(crate) async fn localize(
    pool: &SqlitePool,
    user_id: &str,
    lang: &str,
    name: String,
    title: Option<String>,
    bio: Option<String>,
) -> (String, Option<String>, Option<String>) {
    let shown = translation(pool, user_id, lang).await;
    (
        shown.name.unwrap_or(name),
        shown.title.or(title),
        shown.bio.or(bio),
    )
}

/// [`localize`] for the name and the title.
pub(crate) async fn name_title_in(
    pool: &SqlitePool,
    user_id: &str,
    lang: &str,
    name: String,
    title: Option<String>,
) -> (String, Option<String>) {
    let shown = translation(pool, user_id, lang).await;
    (shown.name.unwrap_or(name), shown.title.or(title))
}

/// [`localize`] for the name alone.
pub(crate) async fn name_in(pool: &SqlitePool, user_id: &str, lang: &str, name: String) -> String {
    translation(pool, user_id, lang).await.name.unwrap_or(name)
}

/// [`name_in`] for the user called `username`.
pub(crate) async fn name_in_for_username(
    pool: &SqlitePool,
    username: &str,
    lang: &str,
    name: String,
) -> String {
    match user_id_where(pool, "SELECT id FROM users WHERE username = ?", username).await {
        Some(user_id) => name_if_theirs(pool, &user_id, lang, name).await,
        None => name,
    }
}

/// [`name_in`] for the owner of event type `et_id` (the legacy single-user
/// pages, which name the host by `accounts.name`): unchanged unless `name`
/// is that user's own name.
pub(crate) async fn owner_name_in(
    pool: &SqlitePool,
    et_id: &str,
    lang: &str,
    name: String,
) -> String {
    let owner = user_id_where(
        pool,
        "SELECT a.user_id FROM accounts a JOIN event_types et ON et.account_id = a.id
         WHERE et.id = ?",
        et_id,
    )
    .await;
    match owner {
        Some(user_id) => name_if_theirs(pool, &user_id, lang, name).await,
        None => name,
    }
}

async fn user_id_where(pool: &SqlitePool, sql: &str, value: &str) -> Option<String> {
    sqlx::query_scalar::<_, Option<String>>(sql)
        .bind(value)
        .fetch_optional(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "cannot read a host for profile translations");
            None
        })
        .flatten()
}

/// The names of a list of people in one query: `parts` gives a row's user
/// id and name, which is replaced in place.
pub(crate) async fn localize_names<T>(
    pool: &SqlitePool,
    lang: &str,
    mut rows: Vec<T>,
    parts: impl Fn(&mut T) -> (&str, &mut String),
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
    let found: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT user_id, lang, name FROM user_texts
         WHERE name IS NOT NULL AND lang IN (?, ?)
           AND user_id IN (SELECT value FROM json_each(?))",
    )
    .bind(lang)
    .bind(base)
    .bind(&ids_json)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "cannot read profile translations");
        Vec::new()
    });
    let mut names: HashMap<String, (bool, String)> = HashMap::new();
    for (user_id, row_lang, name) in found {
        let Some(name) = name else { continue };
        let exact = row_lang == lang;
        match names.get(&user_id) {
            Some((true, _)) => {}
            Some(_) if !exact => {}
            _ => {
                names.insert(user_id, (exact, name));
            }
        }
    }
    for row in rows.iter_mut() {
        let (user_id, name) = parts(row);
        if let Some((_, shown)) = names.get(user_id) {
            *name = shown.clone();
        }
    }
    rows
}

/// `name` in `lang` when it is `user_id`'s own name, else as it is.
async fn name_if_theirs(pool: &SqlitePool, user_id: &str, lang: &str, name: String) -> String {
    let own: Option<String> = sqlx::query_scalar("SELECT name FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
    if own.as_deref() == Some(name.as_str()) {
        name_in(pool, user_id, lang, name).await
    } else {
        name
    }
}

// --- Bookings: who the guest sees as the host ---

/// The one user the guest of booking `uid` sees as its host: the assigned
/// member, else the owner of a personal event type. `None` for a booking
/// with a stored organizer roster (a collective team or a dynamic group
/// link, #161) or a team booking nobody is assigned to.
async fn single_host(pool: &SqlitePool, uid: &str) -> Option<String> {
    #[allow(clippy::type_complexity)]
    let row: Option<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT b.organizer_name, b.assigned_user_id, et.team_id, a.user_id
             FROM bookings b
             JOIN event_types et ON et.id = b.event_type_id
             LEFT JOIN accounts a ON a.id = et.account_id
             WHERE b.uid = ?",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "cannot read a booking's host");
        None
    });
    let (organizer, assigned, team, owner) = row?;
    if organizer.is_some() {
        return None;
    }
    if assigned.is_some() {
        return assigned;
    }
    if team.is_some() {
        return None;
    }
    owner
}

/// The host's name as the guest of booking `uid` reads it in `lang`:
/// `shown` (the name the caller resolved, `booking_host_identity`'s or the
/// page's) translated when it is the one host's own name, else unchanged.
pub(crate) async fn booking_host_name(
    pool: &SqlitePool,
    uid: &str,
    lang: &str,
    shown: String,
) -> String {
    match single_host(pool, uid).await {
        Some(user_id) => name_if_theirs(pool, &user_id, lang, shown).await,
        None => shown,
    }
}

/// [`booking_host_name`] for a guest page opened with `token`.
pub(crate) async fn token_host_name(
    pool: &SqlitePool,
    token: Token<'_>,
    lang: &str,
    shown: String,
) -> String {
    let (sql, value) = match token {
        Token::Cancel(t) => ("SELECT uid FROM bookings WHERE cancel_token = ?", t),
        Token::Reschedule(t) => ("SELECT uid FROM bookings WHERE reschedule_token = ?", t),
    };
    let uid: Option<String> = sqlx::query_scalar(sql)
        .bind(value)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
    match uid {
        Some(uid) => booking_host_name(pool, &uid, lang, shown).await,
        None => shown,
    }
}

/// The guest's copy of `details` (their email and its `.ics`, the
/// attendees' copies) with the host named in the language it is written in
/// (the booking's stored language, English when none: `email_lang`): the
/// email's body, its From display name and the `.ics` ORGANIZER's CN (the
/// address stays the system's, `inbox::reply`). Unchanged without a
/// translation and for a multi-host booking.
pub(crate) async fn host_for_guest<D: GuestCopy>(pool: &SqlitePool, mut details: D) -> D {
    let lang = email_lang(details.reader_lang());
    let uid = details.booking_uid().to_string();
    let shown = std::mem::take(details.host_name_mut());
    *details.host_name_mut() = booking_host_name(pool, &uid, lang, shown).await;
    details
}

/// The profile the reschedule picker shows (`hosts::reschedule_host_profile`)
/// in `lang`: the host's name and title, or a roster's member names (the
/// team's own name stays).
pub(super) async fn reschedule_host(
    pool: &SqlitePool,
    mut host: super::hosts::RescheduleHostProfile,
    lang: &str,
) -> super::hosts::RescheduleHostProfile {
    if let Some(team) = host.team.take() {
        let members = localize_names(pool, lang, team.members, |m| (m.0.as_str(), &mut m.1)).await;
        host.team = Some(super::hosts::RescheduleTeamProfile {
            id: team.id,
            members,
        });
        return host;
    }
    if let Some(user_id) = host.user_id.clone().filter(|id| !id.is_empty()) {
        let (name, title) = name_title_in(pool, &user_id, lang, host.name, host.title).await;
        host.name = name;
        host.title = title;
    }
    host
}

// --- Storage ---

/// Every stored translation of `user_id`, by language.
pub(crate) async fn stored(pool: &SqlitePool, user_id: &str) -> sqlx::Result<Vec<Text>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT lang, name, title, bio FROM user_texts WHERE user_id = ? ORDER BY lang",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(lang, name, title, bio)| Text {
            lang,
            name,
            title,
            bio,
        })
        .collect())
}

/// [`stored`] for a page: a failed read shows none (and is logged).
pub(crate) async fn load(pool: &SqlitePool, user_id: &str) -> Vec<Text> {
    stored(pool, user_id).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "cannot read profile translations");
        Vec::new()
    })
}

/// Replace every translation of `user_id` with `texts`, on `conn` (the
/// caller's transaction, or `config apply`'s connection).
pub(crate) async fn replace(
    conn: &mut SqliteConnection,
    user_id: &str,
    texts: &[Text],
) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM user_texts WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *conn)
        .await?;
    for text in texts {
        sqlx::query(
            "INSERT INTO user_texts (user_id, lang, name, title, bio) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(&text.lang)
        .bind(&text.name)
        .bind(&text.title)
        .bind(&text.bio)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

// --- The settings form ---

/// The "Translations" section of Profile & Settings, flattened into
/// `SettingsForm`: the marker and every field the form struct does not
/// name, of which the rows' `profile_translation_{lang,name,title,bio}_<n>`
/// are read.
#[derive(Default, Deserialize)]
pub(crate) struct TextsForm {
    profile_translations: Option<String>,
    #[serde(flatten)]
    fields: HashMap<String, String>,
}

/// One posted row: language, name, title, bio.
type Posted = (String, String, String, String);

impl TextsForm {
    /// Whether the form carried the section.
    pub(crate) fn given(&self) -> bool {
        self.profile_translations.is_some()
    }

    /// The rows as posted, by their number.
    fn rows(&self) -> BTreeMap<u32, Posted> {
        let mut rows: BTreeMap<u32, Posted> = BTreeMap::new();
        for (key, value) in &self.fields {
            for (prefix, slot) in [
                (LANG_FIELD, 0),
                (NAME_FIELD, 1),
                (TITLE_FIELD, 2),
                (BIO_FIELD, 3),
            ] {
                let Some(n) = key.strip_prefix(prefix).and_then(|n| n.parse::<u32>().ok()) else {
                    continue;
                };
                let row = rows.entry(n).or_default();
                match slot {
                    0 => row.0 = value.clone(),
                    1 => row.1 = value.clone(),
                    2 => row.2 = value.clone(),
                    _ => row.3 = value.clone(),
                }
            }
        }
        rows
    }
}

/// A field as `settings_save` stores its base value: trimmed, empty none.
fn field(value: &str) -> Option<String> {
    Some(value.trim())
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

/// The rows of a posted section, checked as `settings_save` checks the
/// base fields: each trimmed, an empty one none (that language shows the
/// base value), a name at most [`NAME_MAX_BYTES`] bytes; the title and the
/// bio have no limit, as the base ones. A row left with no field is
/// dropped (the way to remove one without JavaScript). The language must
/// be a shipped locale, each at most once. `Ok(None)` when the form did not
/// carry the section. Errors are Fluent keys.
pub(crate) fn parse_form(form: &TextsForm) -> Result<Option<Vec<Text>>, &'static str> {
    if !form.given() {
        return Ok(None);
    }
    let mut texts: Vec<Text> = Vec::new();
    for (lang, name, title, bio) in form.rows().into_values() {
        let (name, title, bio) = (field(&name), field(&title), field(&bio));
        if name.is_none() && title.is_none() && bio.is_none() {
            continue;
        }
        let Some(lang) = crate::i18n::supported_code(&lang) else {
            return Err("form-error-translation-language");
        };
        if name.as_ref().is_some_and(|n| n.len() > NAME_MAX_BYTES) {
            return Err("settings-error-translation-name-length");
        }
        if texts.iter().any(|t| t.lang == lang) {
            return Err("form-error-translation-duplicate");
        }
        texts.push(Text {
            lang: lang.to_string(),
            name,
            title,
            bio,
        });
    }
    texts.sort_by(|a, b| a.lang.cmp(&b.lang));
    Ok(Some(texts))
}

/// Store what the form's section says for `user_id`, in one transaction.
/// Nothing when the form did not carry it. The handler checked it first,
/// so a refusal here is only logged.
pub(crate) async fn save_form(pool: &SqlitePool, user_id: &str, form: &TextsForm) {
    let texts = match parse_form(form) {
        Ok(Some(texts)) => texts,
        Ok(None) => return,
        Err(key) => {
            tracing::warn!(key, "profile translations not saved");
            return;
        }
    };
    let saved = async {
        let mut tx = pool.begin().await?;
        replace(&mut tx, user_id, &texts).await?;
        tx.commit().await
    }
    .await;
    if let Err(e) = saved {
        tracing::error!(error = %e, user_id, "cannot save profile translations");
    }
}

/// One row of the section, as the template shows it.
fn row_ctx(n: usize, lang: &str, name: &str, title: &str, bio: &str) -> Value {
    context! { n => n, lang => lang, name => name, title => title, bio => bio }
}

/// The section's template context (`profile_texts`): the stored rows and
/// every shipped language by its own name.
pub(crate) async fn form_ctx(pool: &SqlitePool, user_id: &str) -> Value {
    let rows: Vec<Value> = load(pool, user_id)
        .await
        .iter()
        .enumerate()
        .map(|(n, t)| {
            row_ctx(
                n,
                &t.lang,
                t.name.as_deref().unwrap_or(""),
                t.title.as_deref().unwrap_or(""),
                t.bio.as_deref().unwrap_or(""),
            )
        })
        .collect();
    section_ctx(rows)
}

/// The section of a form shown again after a refusal: the rows as posted
/// when the form carried the section (so nothing typed is lost), else what
/// [`form_ctx`] shows.
pub(crate) async fn posted_ctx(pool: &SqlitePool, form: &TextsForm, user_id: &str) -> Value {
    if !form.given() {
        return form_ctx(pool, user_id).await;
    }
    let rows: Vec<Value> = form
        .rows()
        .into_values()
        .enumerate()
        .map(|(n, (lang, name, title, bio))| row_ctx(n, &lang, &name, &title, &bio))
        .collect();
    section_ctx(rows)
}

fn section_ctx(rows: Vec<Value>) -> Value {
    let languages: Vec<Value> = crate::i18n::supported_with_labels()
        .map(|(code, label)| context! { code => code, label => label })
        .collect();
    context! { rows => rows, languages => languages, marker => MARKER }
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

    fn row(lang: &str, rows: &[(&str, Option<&str>, Option<&str>, Option<&str>)]) -> Vec<Row> {
        rows.iter()
            .map(|(l, n, t, b)| {
                (
                    l.to_string(),
                    n.map(str::to_string),
                    t.map(str::to_string),
                    b.map(str::to_string),
                )
            })
            .filter(|r| r.0 == lang || Some(r.0.as_str()) == base_lang(lang))
            .collect()
    }

    /// Each field on its own: the language's value, else its base
    /// language's, else none (the caller's base value).
    #[test]
    fn every_field_falls_back_on_its_own() {
        let stored = [
            ("de", Some("Ilja"), Some("CTO"), Some("Bio de")),
            ("de-formal", None, Some("Technischer Leiter"), None),
            ("ru", Some("Илья"), None, None),
        ];
        assert_eq!(
            resolve("de-formal", row("de-formal", &stored)),
            Shown {
                name: Some("Ilja".into()),
                title: Some("Technischer Leiter".into()),
                bio: Some("Bio de".into()),
            }
        );
        assert_eq!(
            resolve("ru", row("ru", &stored)),
            Shown {
                name: Some("Илья".into()),
                title: None,
                bio: None,
            }
        );
        assert_eq!(resolve("fr", row("fr", &stored)), Shown::default());
    }

    #[tokio::test]
    async fn a_form_without_the_section_changes_nothing() {
        assert_eq!(parse_form(&form(&[("name", "X")]).await), Ok(None));
    }

    #[tokio::test]
    async fn rows_are_read_trimmed_and_sorted_and_empty_ones_dropped() {
        let posted = form(&[
            ("profile_translations", "1"),
            ("profile_translation_lang_3", "ru"),
            ("profile_translation_name_3", "  Илья Арестов "),
            ("profile_translation_title_3", "Технический директор"),
            ("profile_translation_bio_3", "  "),
            ("profile_translation_lang_0", "de"),
            ("profile_translation_bio_0", "Über mich"),
            // Left empty: removed.
            ("profile_translation_lang_7", "fr"),
            ("profile_translation_name_7", ""),
        ])
        .await;
        assert_eq!(
            parse_form(&posted),
            Ok(Some(vec![
                Text {
                    lang: "de".into(),
                    name: None,
                    title: None,
                    bio: Some("Über mich".into()),
                },
                Text {
                    lang: "ru".into(),
                    name: Some("Илья Арестов".into()),
                    title: Some("Технический директор".into()),
                    bio: None,
                },
            ]))
        );
        assert_eq!(
            parse_form(&form(&[("profile_translations", "1")]).await),
            Ok(Some(vec![]))
        );
    }

    #[tokio::test]
    async fn bad_rows_are_refused_with_their_key() {
        async fn one(lang: &str, name: &str) -> Result<usize, &'static str> {
            let posted = form(&[
                ("profile_translations", "1"),
                ("profile_translation_lang_0", lang),
                ("profile_translation_name_0", name),
            ])
            .await;
            parse_form(&posted).map(|texts| texts.map_or(0, |t| t.len()))
        }
        assert_eq!(one("xx", "N").await, Err("form-error-translation-language"));
        assert_eq!(one("", "N").await, Err("form-error-translation-language"));
        // The base name's limit: bytes, as `settings_save` counts them.
        assert_eq!(
            one("ru", &"я".repeat(128)).await,
            Err("settings-error-translation-name-length")
        );
        assert_eq!(one("ru", &"я".repeat(127)).await, Ok(1));
        assert_eq!(one("en", &"n".repeat(255)).await, Ok(1));
        let twice = form(&[
            ("profile_translations", "1"),
            ("profile_translation_lang_0", "ru"),
            ("profile_translation_name_0", "A"),
            ("profile_translation_lang_1", "ru"),
            ("profile_translation_title_1", "B"),
        ])
        .await;
        assert_eq!(parse_form(&twice), Err("form-error-translation-duplicate"));
    }
}
