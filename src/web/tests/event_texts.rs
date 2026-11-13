//! ilia fork: an event type's title and description per language
//! (`web::event_texts`, CLAUDE.md "Event type translations (ilia fork)"):
//! the fallback rules, the guest pages and what guests are sent, the
//! dashboard form, duplicate and delete, and `config apply`.

use super::*;
use crate::web::apply::{self, ApplyError};
use crate::web::event_texts::{self, Text};
use serde_json::{json, Value};

const RU_TITLE: &str = "Консультация CTO";
const RU_DESC: &str = "Пятнадцать минут, чтобы разобраться в ситуации.";

async fn add_text(pool: &SqlitePool, et_id: &str, lang: &str, title: &str, desc: Option<&str>) {
    sqlx::query(
        "INSERT INTO event_type_texts (event_type_id, lang, title, description) VALUES (?, ?, ?, ?)",
    )
    .bind(et_id)
    .bind(lang)
    .bind(title)
    .bind(desc)
    .execute(pool)
    .await
    .unwrap();
}

async fn texts_of(pool: &SqlitePool, et_id: &str) -> Vec<(String, String, Option<String>)> {
    sqlx::query_as(
        "SELECT lang, title, description FROM event_type_texts WHERE event_type_id = ? ORDER BY lang",
    )
    .bind(et_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// A page as a browser reading `accept` gets it.
async fn page(app: &Router, uri: &str, accept: &str) -> String {
    let response = app
        .clone()
        .oneshot(get_with_language(uri, accept))
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{uri}");
    body_string(response).await
}

/// Every part of a captured message decoded (base64, quoted-printable or
/// as sent), headers included, so a test reads what the recipient reads.
fn decoded(raw: &str) -> String {
    use base64::Engine as _;
    let mut out = String::new();
    let mut in_body = false;
    let mut encoding = String::new();
    let mut body: Vec<&str> = Vec::new();
    let flush = |out: &mut String, encoding: &str, body: &mut Vec<&str>| {
        match encoding {
            "base64" => {
                let joined: String = body.concat();
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(joined.trim()) {
                    out.push_str(&String::from_utf8_lossy(&bytes));
                }
            }
            "quoted-printable" => out.push_str(&decode_quoted_printable(&body.join("\r\n"))),
            _ => out.push_str(&body.join("\r\n")),
        }
        out.push('\n');
        body.clear();
    };
    for line in raw.split("\r\n") {
        if line.starts_with("--") {
            flush(&mut out, &encoding, &mut body);
            in_body = false;
            encoding.clear();
            continue;
        }
        if in_body {
            body.push(line);
        } else if line.is_empty() {
            in_body = true;
        } else {
            out.push_str(line);
            out.push('\n');
            if let Some(value) = line
                .to_ascii_lowercase()
                .strip_prefix("content-transfer-encoding:")
            {
                encoding = value.trim().to_string();
            }
        }
    }
    flush(&mut out, &encoding, &mut body);
    out
}

/// The decoded `Subject:` of a captured message (RFC 2047 base64 words).
fn subject(raw: &str) -> String {
    use base64::Engine as _;
    let headers = raw.split("\r\n\r\n").next().unwrap_or(raw);
    let unfolded = headers.replace("\r\n ", " ").replace("\r\n\t", " ");
    let line = unfolded
        .split("\r\n")
        .find_map(|line| line.strip_prefix("Subject: "))
        .expect("subject header");
    line.split(' ')
        .map(|word| {
            match word
                .strip_prefix("=?utf-8?b?")
                .or_else(|| word.strip_prefix("=?UTF-8?B?"))
                .and_then(|w| w.strip_suffix("?="))
            {
                Some(b64) => String::from_utf8(
                    base64::engine::general_purpose::STANDARD
                        .decode(b64)
                        .unwrap(),
                )
                .unwrap(),
                None => format!(" {word} "),
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The raw messages to `email` once the background sends are done.
async fn raw_mails_to(mails: &std::sync::Mutex<Vec<CapturedMail>>, email: &str) -> Vec<String> {
    drain_emails(std::time::Duration::from_secs(10)).await;
    mails
        .lock()
        .unwrap()
        .iter()
        .filter(|m| m.is_to(email))
        .map(|m| m.data.clone())
        .collect()
}

// --- Resolution ---

/// The row for the reader's language, else for its base language, else the
/// base text; a row's description goes with its title.
#[tokio::test]
async fn a_reader_gets_their_language_then_its_base_then_the_base_text() {
    let pool = setup_test_db().await;
    let (user_id, _, et) = seed_test_data(&pool).await;
    add_text(&pool, &et, "de", "Beratung", Some("Kurz")).await;
    add_text(&pool, &et, "ru", RU_TITLE, None).await;

    assert_eq!(
        event_texts::translation(&pool, &et, "ru").await,
        Some((RU_TITLE.to_string(), None))
    );
    assert_eq!(
        event_texts::translation(&pool, &et, "de-formal").await,
        Some(("Beratung".to_string(), Some("Kurz".to_string())))
    );
    assert_eq!(event_texts::translation(&pool, &et, "fr").await, None);
    let base = || ("Test Meeting".to_string(), Some("Base".to_string()));
    assert_eq!(
        event_texts::localize(&pool, &et, "fr", base().0, base().1).await,
        base()
    );
    // The Russian row has no description: none, not the English one.
    assert_eq!(
        event_texts::localize(&pool, &et, "ru", base().0, base().1).await,
        (RU_TITLE.to_string(), None)
    );
    assert_eq!(
        event_texts::localize(&pool, "no-such-type", "ru", base().0, base().1).await,
        base()
    );

    // A row of its own beats the base language's.
    add_text(&pool, &et, "de-formal", "Beratung (Sie)", None).await;
    assert_eq!(
        event_texts::title_in(&pool, &et, "de-formal", "x".into()).await,
        "Beratung (Sie)"
    );
    assert_eq!(
        event_texts::title_in(&pool, &et, "de", "x".into()).await,
        "Beratung"
    );

    // The same rules for a list, in one query.
    let other = insert_personal_et(&pool, &user_id, "other").await;
    add_text(&pool, &other, "de", "Andere", None).await;
    let rows = vec![
        (
            et.clone(),
            "Test Meeting".to_string(),
            Some("Base".to_string()),
        ),
        (
            other.clone(),
            "Other".to_string(),
            Some("Other base".to_string()),
        ),
        ("plain".to_string(), "Plain".to_string(), None),
    ];
    let rows = event_texts::localize_rows(&pool, "de-formal", rows, |r| {
        (r.0.as_str(), &mut r.1, &mut r.2)
    })
    .await;
    assert_eq!(rows[0].1, "Beratung (Sie)");
    assert_eq!(rows[0].2, None);
    assert_eq!(rows[1].1, "Andere");
    assert_eq!(rows[1].2, None);
    assert_eq!(rows[2].1, "Plain");
    let rows = event_texts::localize_rows(
        &pool,
        "fr",
        vec![(et.clone(), "Test Meeting".to_string(), None::<String>)],
        |r| (r.0.as_str(), &mut r.1, &mut r.2),
    )
    .await;
    assert_eq!(rows[0].1, "Test Meeting");
}

/// The guest's copy of an email reads the title in the language the email
/// is written in: the booking's, English when it has none.
#[tokio::test]
async fn a_guest_copy_follows_the_language_of_the_email() {
    let pool = setup_test_db().await;
    let (_, _, et) = seed_test_data(&pool).await;
    add_text(&pool, &et, "ru", RU_TITLE, Some(RU_DESC)).await;
    let id = insert_v1_booking(
        &pool,
        &et,
        "guest@test.com",
        Utc::now() + Duration::days(3),
        30,
        "confirmed",
        &[("language", Some("ru".into()))],
    )
    .await;
    let uid = format!("uid-{id}");
    let details = crate::email::BookingDetails {
        uid: uid.clone(),
        event_title: "Test Meeting".into(),
        guest_language: Some("ru".into()),
        ..Default::default()
    };
    assert_eq!(
        event_texts::for_guest(&pool, &details).await.event_title,
        RU_TITLE
    );
    let english = crate::email::BookingDetails {
        guest_language: None,
        ..details.clone()
    };
    assert_eq!(
        event_texts::for_guest(&pool, &english).await.event_title,
        "Test Meeting"
    );
    // The `.ics` downloads read the booking's stored language.
    assert_eq!(
        event_texts::in_booking_language(&pool, &english)
            .await
            .event_title,
        RU_TITLE
    );
    let cancel = crate::email::CancellationDetails {
        uid,
        event_title: "Test Meeting".into(),
        guest_language: Some("ru".into()),
        ..Default::default()
    };
    assert_eq!(
        event_texts::for_guest(&pool, &cancel).await.event_title,
        RU_TITLE
    );
    assert_eq!(event_texts::email_lang(Some("xx")), "en");
    assert_eq!(event_texts::email_lang(Some("de-formal")), "de-formal");
}

// --- Guest pages ---

/// A Russian reader (the widget's `?lang=ru`, or `Accept-Language: ru`)
/// reads the Russian title and description on the profile, the slot page
/// and the booking form; anyone else the base text, byte for byte as
/// before.
#[tokio::test]
async fn russian_guests_read_the_russian_text_and_others_the_base() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    sqlx::query("UPDATE event_types SET description = 'Base description' WHERE id = ?")
        .bind(&et_id)
        .execute(&pool)
        .await
        .unwrap();
    let form = booking_form_uri(None);
    let before: Vec<String> = {
        let mut pages = Vec::new();
        for uri in ["/u/testuser", "/u/testuser/test-meeting", form.as_str()] {
            pages.push(page(&app, uri, "en-US,en;q=0.9").await);
        }
        pages
    };
    add_text(&pool, &et_id, "ru", RU_TITLE, Some(RU_DESC)).await;

    for (i, uri) in ["/u/testuser", "/u/testuser/test-meeting", form.as_str()]
        .into_iter()
        .enumerate()
    {
        let joiner = if uri.contains('?') { '&' } else { '?' };
        for (uri, accept) in [
            (uri.to_string(), "ru"),
            (format!("{uri}{joiner}embed=1&lang=ru"), "en-US,en;q=0.9"),
        ] {
            let ru = page(&app, &uri, accept).await;
            assert!(ru.contains(RU_TITLE), "{uri}");
            assert!(!ru.contains("Test Meeting"), "{uri}");
            assert!(!ru.contains("Base description"), "{uri}");
            if !uri.contains("/book") {
                assert!(ru.contains(RU_DESC), "{uri}: the description");
            }
        }
        let en = page(&app, uri, "en-US,en;q=0.9").await;
        assert!(en.contains("Test Meeting"), "{uri}");
        assert!(!en.contains(RU_TITLE), "{uri}");
        // Nothing else on the page moved: only the text is the base one.
        assert_eq!(
            en.matches("Test Meeting").count(),
            before[i].matches("Test Meeting").count(),
            "{uri}"
        );
        // French has no row: the base text.
        let fr = page(&app, uri, "fr").await;
        assert!(fr.contains("Test Meeting"), "{uri}");
    }
}

/// A team's profile and slot page read the translation too.
#[tokio::test]
async fn team_pages_read_the_translation() {
    let (app, pool, _, _) = setup_test_app().await;
    let (_, et_id) = seed_team_event_type(&pool).await;
    add_text(&pool, &et_id, "ru", "Синхронизация команды", None).await;
    for uri in ["/team/horizon-team", "/team/horizon-team/team-sync"] {
        let ru = page(&app, uri, "ru").await;
        assert!(ru.contains("Синхронизация команды"), "{uri}");
        assert!(!ru.contains("Team Sync"), "{uri}");
        let en = page(&app, uri, "en").await;
        assert!(en.contains("Team Sync"), "{uri}");
    }
}

/// A booking made in the Russian widget: the confirmation page, the guest's
/// email (subject, body, its `.ics`) and the `.ics` download read the
/// Russian title; the host's email keeps the base title.
#[tokio::test]
async fn a_russian_booking_tells_the_guest_in_russian_and_the_host_as_before() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, port).await;
    add_text(&pool, &et_id, "ru", RU_TITLE, Some(RU_DESC)).await;
    let csrf = "csrf-event-texts";
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/u/testuser/test-meeting/book?embed=1&lang=ru")
                .header("cookie", format!("__Host-calrs_csrf={csrf}"))
                .header("accept-language", "en-US,en;q=0.9")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(book_body(csrf)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let confirmed = body_string(response).await;
    assert!(confirmed.contains(RU_TITLE), "{confirmed}");
    assert!(!confirmed.contains("Test Meeting"));

    let guest = raw_mails_to(&mails, "guest@test.com").await;
    assert_eq!(guest.len(), 1);
    assert!(
        subject(&guest[0]).contains(RU_TITLE),
        "{}",
        subject(&guest[0])
    );
    let body = decoded(&guest[0]);
    assert!(body.contains(RU_TITLE), "{body}");
    assert!(body.contains(&format!("SUMMARY:{RU_TITLE}")), "{body}");
    assert!(!body.contains("Test Meeting"), "{body}");

    let host = raw_mails_to(&mails, "test@example.com").await;
    assert_eq!(host.len(), 1);
    assert!(subject(&host[0]).contains("Test Meeting"));
    let body = decoded(&host[0]);
    assert!(body.contains("Test Meeting"), "{body}");
    assert!(!body.contains(RU_TITLE), "{body}");

    let cancel_token: String = sqlx::query_scalar(
        "SELECT cancel_token FROM bookings WHERE guest_email = 'guest@test.com'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let response = app
        .oneshot(get(&format!("/booking/ics/{cancel_token}")))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let ics = body_string(response).await;
    assert!(ics.contains(&format!("SUMMARY:{RU_TITLE}")), "{ics}");
}

/// The guest token pages (cancel, reschedule) read the booking's language.
#[tokio::test]
async fn the_guest_token_pages_read_the_booking_language() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    add_text(&pool, &et_id, "ru", RU_TITLE, None).await;
    let id = insert_v1_booking(
        &pool,
        &et_id,
        "guest@test.com",
        Utc::now() + Duration::days(3),
        30,
        "confirmed",
        &[("language", Some("ru".into()))],
    )
    .await;
    let (cancel, reschedule): (String, String) =
        sqlx::query_as("SELECT cancel_token, reschedule_token FROM bookings WHERE id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
    for uri in [
        format!("/booking/cancel/{cancel}"),
        format!("/booking/reschedule/{reschedule}"),
    ] {
        let body = page(&app, &uri, "en").await;
        assert!(body.contains(RU_TITLE), "{uri}: {body}");
        assert!(!body.contains("Test Meeting"), "{uri}");
    }
    // An English booking of the same type reads the base title.
    let en = insert_v1_booking(
        &pool,
        &et_id,
        "other@test.com",
        Utc::now() + Duration::days(4),
        30,
        "confirmed",
        &[("language", Some("en".into()))],
    )
    .await;
    let cancel: String = sqlx::query_scalar("SELECT cancel_token FROM bookings WHERE id = ?")
        .bind(&en)
        .fetch_one(&pool)
        .await
        .unwrap();
    let body = page(&app, &format!("/booking/cancel/{cancel}"), "ru").await;
    assert!(body.contains("Test Meeting"), "{body}");
}

/// An approval by the host's token: the guest's confirmation reads the
/// booking's language, the host's own email the base title.
#[tokio::test]
async fn an_approval_tells_the_guest_in_their_language() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, port).await;
    add_text(&pool, &et_id, "fr", "Consultation CTO", None).await;
    insert_pending_french_booking(&pool, &et_id, "uid-texts-approve").await;
    let response = app
        .oneshot(post_bare("/booking/approve/confirm-uid-texts-approve"))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let approved = body_string(response).await;
    assert!(approved.contains("Test Meeting"), "the host's page");

    let guest = raw_mails_to(&mails, "guest@test.com").await;
    assert_eq!(guest.len(), 1);
    let body = decoded(&guest[0]);
    assert!(body.contains("Consultation CTO"), "{body}");
    assert!(!body.contains("Test Meeting"), "{body}");
    let host = raw_mails_to(&mails, "test@example.com").await;
    assert_eq!(host.len(), 1);
    let body = decoded(&host[0]);
    assert!(body.contains("Test Meeting"), "{body}");
    assert!(!body.contains("Consultation CTO"), "{body}");
}

// --- The dashboard form ---

/// The edit form's fields as `et_form_body` in `tests/event_types.rs`,
/// for the fixture's event type, `extra` replacing or adding to them.
fn edit_body(csrf: &str, extra: &[(&str, &str)]) -> String {
    let mut fields: Vec<(&str, &str)> = vec![
        ("_csrf", csrf),
        ("title", "Test Meeting"),
        ("slug", "test-meeting"),
        ("duration_min", "30"),
        ("buffer_before", "0"),
        ("buffer_after", "0"),
        ("min_notice_min", "0"),
        ("location_value", "https://meet.example.com"),
        ("avail_days", "1,2,3,4,5"),
        ("avail_start", "09:00"),
        ("avail_end", "17:00"),
    ];
    for (key, value) in extra {
        match fields.iter_mut().find(|(k, _)| k == key) {
            Some(field) => field.1 = value,
            None => fields.push((key, value)),
        }
    }
    fields
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

async fn post_edit(app: &Router, session: &str, extra: &[(&str, &str)]) -> (u16, String) {
    let csrf = "csrf-texts-form";
    let response = app
        .clone()
        .oneshot(post_form(
            "/dashboard/event-types/test-meeting/edit",
            session,
            csrf,
            &edit_body(csrf, extra),
        ))
        .await
        .unwrap();
    let status = response.status().as_u16();
    (status, body_string(response).await)
}

/// The form shows the section with the stored rows, saves what it posts
/// (rows in any number order, an emptied row removed) and keeps the rows
/// when a form comes without the section (MCP, `config apply`).
#[tokio::test]
async fn the_form_shows_saves_and_keeps_the_translations() {
    let (app, pool, session, et_id) = setup_test_app().await;
    add_text(&pool, &et_id, "ru", RU_TITLE, Some(RU_DESC)).await;
    let form = body_string(
        app.clone()
            .oneshot(get_authed(
                "/dashboard/event-types/test-meeting/edit",
                &session,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert!(form.contains(r#"id="translations-section""#), "{form}");
    assert!(form.contains(r#"name="translations" value="1""#));
    assert!(form.contains(RU_TITLE));
    assert!(form.contains(RU_DESC));
    assert!(form.contains(r#"<option value="ru" selected>Русский</option>"#));
    let new_form = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/event-types/new", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(new_form.contains(r#"id="translations-section""#));

    let (status, _) = post_edit(
        &app,
        &session,
        &[
            ("translations", "1"),
            ("translation_lang_4", "ru"),
            ("translation_title_4", "  Консультация  "),
            ("translation_description_4", ""),
            ("translation_lang_1", "de"),
            ("translation_title_1", "Beratung"),
            ("translation_description_1", "Kurz und **klar**"),
            ("translation_lang_9", "fr"),
            ("translation_title_9", ""),
            ("translation_description_9", ""),
        ],
    )
    .await;
    assert_eq!(status, 303);
    assert_eq!(
        texts_of(&pool, &et_id).await,
        vec![
            (
                "de".to_string(),
                "Beratung".to_string(),
                Some("Kurz und **klar**".to_string())
            ),
            ("ru".to_string(), "Консультация".to_string(), None),
        ]
    );

    // Without the section (the MCP and apply post this form too): kept.
    let (status, _) = post_edit(&app, &session, &[("title", "Renamed")]).await;
    assert_eq!(status, 303);
    assert_eq!(texts_of(&pool, &et_id).await.len(), 2);

    // The section with no row: every translation removed.
    let (status, _) = post_edit(&app, &session, &[("translations", "1")]).await;
    assert_eq!(status, 303);
    assert!(texts_of(&pool, &et_id).await.is_empty());
}

/// Refusals: a language calrs does not ship, a missing or too long title,
/// one language twice. Nothing is saved (the event type neither), and the
/// form comes back with the message and what was typed. Without the CSRF
/// token nothing happens at all.
#[tokio::test]
async fn the_form_refuses_bad_translations() {
    let (app, pool, session, et_id) = setup_test_app().await;
    add_text(&pool, &et_id, "ru", RU_TITLE, None).await;
    let long = "я".repeat(256);
    let cases: Vec<(Vec<(&str, &str)>, &str)> = vec![
        (
            vec![
                ("translation_lang_0", "xx"),
                ("translation_title_0", "Title"),
            ],
            "form-error-translation-language",
        ),
        (
            vec![
                ("translation_lang_0", "ru"),
                ("translation_title_0", " "),
                ("translation_description_0", "Описание"),
            ],
            "form-error-translation-title-required",
        ),
        (
            vec![
                ("translation_lang_0", "ru"),
                ("translation_title_0", long.as_str()),
            ],
            "form-error-translation-title-length",
        ),
        (
            vec![
                ("translation_lang_0", "ru"),
                ("translation_title_0", "Один"),
                ("translation_lang_1", "ru"),
                ("translation_title_1", "Два"),
            ],
            "form-error-translation-duplicate",
        ),
    ];
    for (rows, key) in cases {
        let mut extra = vec![("translations", "1"), ("title", "Changed")];
        extra.extend(rows.iter().copied());
        let (status, body) = post_edit(&app, &session, &extra).await;
        assert_eq!(status, 200, "{key}");
        let message = crate::i18n::translate("en", key, None);
        assert!(body.contains(&message), "{key}: {body}");
        // What was typed is shown again.
        assert!(body.contains(rows[1].1.trim()), "{key}");
        assert_eq!(
            texts_of(&pool, &et_id).await,
            vec![("ru".to_string(), RU_TITLE.to_string(), None)],
            "{key}"
        );
        let title: String = sqlx::query_scalar("SELECT title FROM event_types WHERE id = ?")
            .bind(&et_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(title, "Test Meeting", "{key}: nothing written");
    }
    // 255 characters are fine.
    let ok = "я".repeat(255);
    let (status, _) = post_edit(
        &app,
        &session,
        &[
            ("translations", "1"),
            ("translation_lang_0", "ru"),
            ("translation_title_0", ok.as_str()),
        ],
    )
    .await;
    assert_eq!(status, 303);

    // No CSRF token: refused, nothing written.
    let response = app
        .oneshot(post_form(
            "/dashboard/event-types/test-meeting/edit",
            &session,
            "cookie-token",
            &edit_body("other-token", &[("translations", "1")]),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 403);
    assert_eq!(texts_of(&pool, &et_id).await.len(), 1);
}

/// A new event type (personal and team) saves its translations with it.
#[tokio::test]
async fn a_new_event_type_saves_its_translations() {
    let (app, pool, session, _) = setup_test_app().await;
    let csrf = "csrf-texts-new";
    let body = format!(
        "{}&translations=1&translation_lang_0=ru&translation_title_0={}",
        edit_body(csrf, &[])
            .replace("slug=test-meeting", "slug=fresh")
            .replace("title=Test%20Meeting", "title=Fresh"),
        urlencoding::encode(RU_TITLE)
    );
    let response = app
        .clone()
        .oneshot(post_form(
            "/dashboard/event-types/new",
            &session,
            csrf,
            &body,
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    let et_id: String = sqlx::query_scalar("SELECT id FROM event_types WHERE slug = 'fresh'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        texts_of(&pool, &et_id).await,
        vec![("ru".to_string(), RU_TITLE.to_string(), None)]
    );

    let (team_id, team_et) = seed_team_event_type(&pool).await;
    let body = format!(
        "{}&translations=1&translation_lang_0=de&translation_title_0=Team",
        edit_body(csrf, &[])
            .replace("slug=test-meeting", "slug=team-sync")
            .replace("title=Test%20Meeting", "title=Team%20Sync")
    );
    let response = app
        .oneshot(post_form(
            &format!("/dashboard/group-event-types/{team_id}/team-sync/edit"),
            &session,
            csrf,
            &body,
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection(), "{}", response.status());
    assert_eq!(
        texts_of(&pool, &team_et).await,
        vec![("de".to_string(), "Team".to_string(), None)]
    );
}

/// Duplicating copies the translations; deleting the event type removes
/// them (`ON DELETE CASCADE`).
#[tokio::test]
async fn duplicate_copies_and_delete_removes_the_translations() {
    let (app, pool, session, et_id) = setup_test_app().await;
    add_text(&pool, &et_id, "ru", RU_TITLE, Some(RU_DESC)).await;
    add_text(&pool, &et_id, "de", "Beratung", None).await;
    let csrf = "csrf-texts-dup";
    let response = app
        .clone()
        .oneshot(post_form(
            "/dashboard/event-types/test-meeting/duplicate",
            &session,
            csrf,
            &format!("_csrf={csrf}"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    let copy: String = sqlx::query_scalar(
        "SELECT id FROM event_types WHERE slug != 'test-meeting' AND team_id IS NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(texts_of(&pool, &copy).await, texts_of(&pool, &et_id).await);

    let response = app
        .oneshot(post_form(
            "/dashboard/event-types/test-meeting/delete",
            &session,
            csrf,
            &format!("_csrf={csrf}"),
        ))
        .await
        .unwrap();
    assert!(response.status().is_redirection());
    assert!(texts_of(&pool, &et_id).await.is_empty());
    assert_eq!(
        texts_of(&pool, &copy).await.len(),
        2,
        "the copy keeps its own"
    );
    let left: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM event_type_texts WHERE event_type_id = ?")
            .bind(&et_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(left, 0);
    // Rows go with their event type at the database level too.
    sqlx::query("DELETE FROM event_types WHERE id = ?")
        .bind(&copy)
        .execute(&pool)
        .await
        .unwrap();
    assert!(texts_of(&pool, &copy).await.is_empty());
}

/// The host side keeps the base title: the dashboard list.
#[tokio::test]
async fn the_dashboard_keeps_the_base_title() {
    let (app, pool, session, et_id) = setup_test_app().await;
    add_text(&pool, &et_id, "en", "Guest-facing English", None).await;
    let body = body_string(
        app.oneshot(get_authed("/dashboard/event-types", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(body.contains("Test Meeting"));
    assert!(!body.contains("Guest-facing English"));
}

// --- config apply ---

mod apply_texts {
    use super::super::apply_support::*;
    use super::*;

    /// The role greps a release for this literal before it sends the key
    /// (`apply::event_types::TRANSLATIONS_SHAPE`): it must stay as it is.
    const TRANSLATIONS_SHAPE: &str =
        "translations: {<language>: {title, description}}, the whole set";

    fn doc(item: Value) -> String {
        json!({
            "format": apply::FORMAT,
            "user": ADMIN_EMAIL,
            "event_types": {"items": [item]},
        })
        .to_string()
    }

    fn item(extra: Value) -> Value {
        let mut item = json!({
            "slug": "15min",
            "title": "CTO Consultation",
            "duration_min": 15,
            "location_type": "custom",
            "location_value": "The link comes by email",
        });
        for (key, value) in extra.as_object().unwrap() {
            item[key] = value.clone();
        }
        item
    }

    async fn stored(env: &ApplyEnv) -> Vec<Text> {
        let et_id: String = sqlx::query_scalar("SELECT id FROM event_types WHERE slug = '15min'")
            .fetch_one(&env.pool)
            .await
            .unwrap();
        event_texts::load(&env.pool, &et_id).await
    }

    fn paths(report: &apply::Report) -> Vec<&str> {
        report.changed.iter().map(|c| c.path.as_str()).collect()
    }

    /// Declared, written; a second run changes nothing; a change is one
    /// entry with both sets; `{}` removes all; left out, kept, also when
    /// the run posts the event type form for another key.
    #[tokio::test]
    async fn apply_declares_the_whole_set_and_a_second_run_changes_nothing() {
        let env = ApplyEnv::new().await;
        let ru = json!({"ru": {"title": RU_TITLE, "description": RU_DESC}});
        let json = doc(item(json!({ "translations": ru })));
        let report = env.apply(&json, false).await.unwrap();
        assert_eq!(paths(&report), ["event_types[15min]"]);
        assert_eq!(
            stored(&env).await,
            vec![Text {
                lang: "ru".into(),
                title: RU_TITLE.into(),
                description: Some(RU_DESC.into()),
            }]
        );
        let again = env.apply(&json, false).await.unwrap();
        assert_eq!(again.changed, []);

        let both = json!({
            "de": {"title": "Beratung"},
            "ru": {"title": "Консультация", "description": null},
        });
        let report = env
            .apply(&doc(item(json!({ "translations": both }))), false)
            .await
            .unwrap();
        assert_eq!(paths(&report), ["event_types[15min].translations"]);
        assert_eq!(
            report.changed[0].before,
            json!({"ru": {"title": RU_TITLE, "description": RU_DESC}})
        );
        assert_eq!(
            report.changed[0].after,
            json!({
                "de": {"title": "Beratung", "description": null},
                "ru": {"title": "Консультация", "description": null},
            })
        );
        assert_eq!(stored(&env).await.len(), 2);

        // Left out while the form is posted for the title: kept.
        let report = env
            .apply(&doc(item(json!({"title": "Renamed"}))), false)
            .await
            .unwrap();
        assert_eq!(paths(&report), ["event_types[15min].title"]);
        assert_eq!(stored(&env).await.len(), 2);

        let report = env
            .apply(
                &doc(item(json!({"title": "Renamed", "translations": {}}))),
                false,
            )
            .await
            .unwrap();
        assert_eq!(paths(&report), ["event_types[15min].translations"]);
        assert!(stored(&env).await.is_empty());
    }

    /// The export gives the stored set (`{}` for none), and applies back
    /// as no change; a dry run writes nothing.
    #[tokio::test]
    async fn the_export_round_trips() {
        let env = ApplyEnv::new().await;
        let texts = json!({"de-formal": {"title": "Beratung (Sie)"}, "ru": {"title": RU_TITLE}});
        env.apply(&doc(item(json!({ "translations": texts }))), false)
            .await
            .unwrap();
        let exported = serde_json::to_value(env.export().await.unwrap()).unwrap();
        let item = &exported["event_types"]["items"][0];
        assert_eq!(
            item["translations"],
            json!({
                "de-formal": {"title": "Beratung (Sie)", "description": null},
                "ru": {"title": RU_TITLE, "description": null},
            })
        );
        let snapshot = env.snapshot().await;
        let report = env.apply(&exported.to_string(), false).await.unwrap();
        assert_eq!(report.changed, []);
        assert_eq!(env.snapshot().await, snapshot);

        let dry = env
            .apply(
                &doc(super::apply_texts::item(json!({"translations": {}}))),
                true,
            )
            .await
            .unwrap();
        assert_eq!(paths(&dry), ["event_types[15min].translations"]);
        assert_eq!(env.snapshot().await, snapshot);

        // An event type with none exports `{}`.
        env.apply(
            &doc(super::apply_texts::item(json!({"translations": {}}))),
            false,
        )
        .await
        .unwrap();
        let exported = serde_json::to_value(env.export().await.unwrap()).unwrap();
        assert_eq!(
            exported["event_types"]["items"][0]["translations"],
            json!({})
        );
    }

    /// Refused before anything is written, under the key's path: a
    /// language calrs does not ship (or spelled otherwise), a title empty,
    /// padded or too long, a description past MCP's limit or with a
    /// script link, a language given twice.
    #[tokio::test]
    async fn apply_refuses_bad_translations() {
        let env = ApplyEnv::new().await;
        let long = "я".repeat(256);
        let cases = [
            (
                json!({"xx": {"title": "T"}}),
                "event_types[15min].translations.xx",
            ),
            (
                json!({"RU": {"title": "T"}}),
                "event_types[15min].translations.RU",
            ),
            (
                json!({"ru": {"title": ""}}),
                "event_types[15min].translations.ru.title",
            ),
            (
                json!({"ru": {"title": " T"}}),
                "event_types[15min].translations.ru.title",
            ),
            (
                json!({"ru": {"title": long}}),
                "event_types[15min].translations.ru.title",
            ),
            (
                json!({"ru": {"title": "T", "description": "x".repeat(4001)}}),
                "event_types[15min].translations.ru.description",
            ),
            (
                json!({"ru": {"title": "T", "description": "[x](javascript:alert(1))"}}),
                "event_types[15min].translations.ru.description",
            ),
        ];
        let before = env.snapshot().await;
        for (texts, want) in cases {
            let error = env
                .apply(&doc(item(json!({ "translations": texts }))), false)
                .await
                .unwrap_err();
            assert!(
                matches!(&error, ApplyError::Invalid { path, .. } if path == want),
                "{want}: {error}"
            );
            assert_eq!(env.snapshot().await, before, "{want}");
        }
        // The shape's marker is in the refusal.
        let error = env
            .apply(
                &doc(item(json!({"translations": {"xx": {"title": "T"}}}))),
                false,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains(TRANSLATIONS_SHAPE), "{error}");
        // One language twice: the JSON object itself is refused.
        let twice = doc(item(json!({}))).replace(
            r#""slug":"15min""#,
            r#""slug":"15min","translations":{"ru":{"title":"A"},"ru":{"title":"B"}}"#,
        );
        assert!(twice.contains(r#""ru":{"title":"A"},"ru""#), "{twice}");
        let error = apply::parse(&twice).unwrap_err();
        assert!(error.to_string().contains("given twice"), "{error}");
        // An unknown key in a language's object.
        let error = apply::parse(&doc(item(
            json!({"translations": {"ru": {"title": "T", "summary": "S"}}}),
        )))
        .unwrap_err();
        assert!(error.to_string().contains("summary"), "{error}");
    }
}
