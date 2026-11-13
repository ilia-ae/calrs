//! ilia fork: a user's public name, title and bio per language
//! (`web::user_texts`, CLAUDE.md "Profile translations (ilia fork)"): the
//! fallback per field, the guest pages and what guests are sent, the
//! settings form, `config apply`, and the user's deletion.

use super::*;
use crate::web::apply::{self, ApplyError};
use crate::web::user_texts::{self, Text};
use serde_json::{json, Value};

const RU_NAME: &str = "Илья Арестов";
const RU_TITLE: &str = "Технический директор";
const RU_BIO: &str = "Технический директор на аутсорсе.";
const BASE_TITLE: &str = "Fractional CTO";
const BASE_BIO: &str = "Fractional CTO based in Dubai.";

async fn add_text(
    pool: &SqlitePool,
    user_id: &str,
    lang: &str,
    name: Option<&str>,
    title: Option<&str>,
    bio: Option<&str>,
) {
    sqlx::query("INSERT INTO user_texts (user_id, lang, name, title, bio) VALUES (?, ?, ?, ?, ?)")
        .bind(user_id)
        .bind(lang)
        .bind(name)
        .bind(title)
        .bind(bio)
        .execute(pool)
        .await
        .unwrap();
}

type Stored = Vec<(String, Option<String>, Option<String>, Option<String>)>;

async fn texts_of(pool: &SqlitePool, user_id: &str) -> Stored {
    sqlx::query_as("SELECT lang, name, title, bio FROM user_texts WHERE user_id = ? ORDER BY lang")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn user_id(pool: &SqlitePool) -> String {
    sqlx::query_scalar("SELECT id FROM users WHERE username = 'testuser'")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// The fixture's host with a title and a bio, and a Russian translation of
/// all three.
async fn russian_host(pool: &SqlitePool) -> String {
    let id = user_id(pool).await;
    sqlx::query("UPDATE users SET title = ?, bio = ? WHERE id = ?")
        .bind(BASE_TITLE)
        .bind(BASE_BIO)
        .bind(&id)
        .execute(pool)
        .await
        .unwrap();
    id
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

/// RFC 2047 words (`=?utf-8?b?…?=`) of a header line decoded.
fn decode_words(line: &str) -> String {
    use base64::Engine as _;
    line.split(' ')
        .map(|word| {
            let bare = word.trim_matches('"');
            match bare
                .strip_prefix("=?utf-8?b?")
                .or_else(|| bare.strip_prefix("=?UTF-8?B?"))
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

/// The decoded value of the first `name:` header of a captured message.
fn header(raw: &str, name: &str) -> String {
    let headers = raw.split("\r\n\r\n").next().unwrap_or(raw);
    let unfolded = headers.replace("\r\n ", " ").replace("\r\n\t", " ");
    let prefix = format!("{name}: ");
    let line = unfolded
        .split("\r\n")
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .unwrap_or_else(|| panic!("no {name} header"));
    decode_words(line)
}

/// Every part of a captured message decoded (base64, quoted-printable or
/// as sent), so a test reads what the recipient reads.
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
            out.push_str(&decode_words(line));
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
    // Unfold the `.ics` lines, which may break a long name.
    out.replace("\r\n ", "").replace("\n ", "")
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

/// Each field on its own: the reader's language, else its base language,
/// else the user's own value; a language without a row, the base values.
#[tokio::test]
async fn each_field_falls_back_on_its_own() {
    let pool = setup_test_db().await;
    let (user_id, _, _) = seed_test_data(&pool).await;
    add_text(
        &pool,
        &user_id,
        "de",
        Some("Testnutzer"),
        Some("Leiter"),
        Some("Über mich"),
    )
    .await;
    add_text(
        &pool,
        &user_id,
        "de-formal",
        None,
        Some("Technischer Leiter"),
        None,
    )
    .await;
    add_text(&pool, &user_id, "ru", Some(RU_NAME), Some(RU_TITLE), None).await;
    let base = || {
        (
            "Test User".to_string(),
            Some(BASE_TITLE.to_string()),
            Some(BASE_BIO.to_string()),
        )
    };
    let localize = |lang: &'static str| {
        let pool = pool.clone();
        let user_id = user_id.clone();
        async move {
            let (n, t, b) = base();
            user_texts::localize(&pool, &user_id, lang, n, t, b).await
        }
    };
    // A Russian name and title, no Russian bio: the base bio.
    assert_eq!(
        localize("ru").await,
        (
            RU_NAME.to_string(),
            Some(RU_TITLE.to_string()),
            Some(BASE_BIO.to_string())
        )
    );
    // `de-formal`'s own title, the rest from `de`.
    assert_eq!(
        localize("de-formal").await,
        (
            "Testnutzer".to_string(),
            Some("Technischer Leiter".to_string()),
            Some("Über mich".to_string())
        )
    );
    assert_eq!(localize("fr").await, base());
    assert_eq!(localize("en").await, base());
    // A base field that is empty stays empty without a translation, and a
    // translation fills it.
    assert_eq!(
        user_texts::name_title_in(&pool, &user_id, "fr", "Test User".into(), None).await,
        ("Test User".to_string(), None)
    );
    assert_eq!(
        user_texts::name_title_in(&pool, &user_id, "ru", "Test User".into(), None).await,
        (RU_NAME.to_string(), Some(RU_TITLE.to_string()))
    );
    assert_eq!(
        user_texts::name_in(&pool, "no-such-user", "ru", "X".into()).await,
        "X"
    );

    // The same rule for a list of people, in one query.
    let other = insert_role_user(&pool, "other@example.com", "user").await;
    add_text(&pool, &other, "de", Some("Anderer"), None, None).await;
    add_text(&pool, &other, "de-formal", None, Some("Nur Titel"), None).await;
    let rows = vec![
        (user_id.clone(), "Test User".to_string()),
        (other.clone(), "Other".to_string()),
        ("plain".to_string(), "Plain".to_string()),
    ];
    let names = user_texts::localize_names(&pool, "de-formal", rows.clone(), |r| {
        (r.0.as_str(), &mut r.1)
    })
    .await;
    assert_eq!(names[0].1, "Testnutzer");
    assert_eq!(
        names[1].1, "Anderer",
        "a title-only row leaves the base name"
    );
    assert_eq!(names[2].1, "Plain");
    let names = user_texts::localize_names(&pool, "fr", rows, |r| (r.0.as_str(), &mut r.1)).await;
    assert_eq!(names[0].1, "Test User");
}

/// A booking's host is translated only when the guest sees one person: a
/// booking with a stored organizer roster, or a name that is not the
/// host's own, stays as it is.
#[tokio::test]
async fn only_a_single_host_by_their_own_name_is_translated() {
    let pool = setup_test_db().await;
    let (user_id, _, et) = seed_test_data(&pool).await;
    add_text(&pool, &user_id, "ru", Some(RU_NAME), None, None).await;
    let single = insert_v1_booking(
        &pool,
        &et,
        "guest@test.com",
        Utc::now() + Duration::days(3),
        30,
        "confirmed",
        &[],
    )
    .await;
    let uid = format!("uid-{single}");
    assert_eq!(
        user_texts::booking_host_name(&pool, &uid, "ru", "Test User".into()).await,
        RU_NAME
    );
    assert_eq!(
        user_texts::booking_host_name(&pool, &uid, "en", "Test User".into()).await,
        "Test User"
    );
    assert_eq!(
        user_texts::booking_host_name(&pool, &uid, "ru", "Test User & Bob".into()).await,
        "Test User & Bob"
    );
    let roster = insert_v1_booking(
        &pool,
        &et,
        "guest2@test.com",
        Utc::now() + Duration::days(4),
        30,
        "confirmed",
        &[
            ("organizer_user_id", Some(user_id.clone())),
            ("organizer_name", Some("Test User".into())),
            ("organizer_email", Some("test@example.com".into())),
        ],
    )
    .await;
    assert_eq!(
        user_texts::booking_host_name(&pool, &format!("uid-{roster}"), "ru", "Test User".into())
            .await,
        "Test User",
        "the stored organizer is named as stored"
    );

    // The guest's copy of an email: the host in the email's language.
    let details = crate::email::BookingDetails {
        uid: uid.clone(),
        host_name: "Test User".into(),
        guest_language: Some("ru".into()),
        ..Default::default()
    };
    assert_eq!(
        crate::web::event_texts::for_guest(&pool, &details)
            .await
            .host_name,
        RU_NAME
    );
    let english = crate::email::BookingDetails {
        guest_language: None,
        ..details.clone()
    };
    assert_eq!(
        crate::web::event_texts::for_guest(&pool, &english)
            .await
            .host_name,
        "Test User"
    );
}

// --- Guest pages ---

/// A Russian reader (the widget's `?lang=ru`, or `Accept-Language: ru`)
/// reads the Russian name, title and bio on the profile, the slot page and
/// the booking form; an English reader the page exactly as it was without
/// any translation, and a French one the base values.
#[tokio::test]
async fn russian_guests_read_the_russian_profile_and_others_the_base() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, _, _) = setup_test_app().await;
    let id = russian_host(&pool).await;
    let form = booking_form_uri(None);
    let uris = ["/u/testuser", "/u/testuser/test-meeting", form.as_str()];
    let mut before = Vec::new();
    for uri in uris {
        before.push(page(&app, uri, "en-US,en;q=0.9").await);
    }
    add_text(
        &pool,
        &id,
        "ru",
        Some(RU_NAME),
        Some(RU_TITLE),
        Some(RU_BIO),
    )
    .await;

    for (i, uri) in uris.into_iter().enumerate() {
        let joiner = if uri.contains('?') { '&' } else { '?' };
        for (uri, accept) in [
            (uri.to_string(), "ru"),
            (format!("{uri}{joiner}embed=1&lang=ru"), "en-US,en;q=0.9"),
        ] {
            let ru = page(&app, &uri, accept).await;
            assert!(ru.contains(RU_NAME), "{uri}");
            assert!(!ru.contains("Test User"), "{uri}");
            if !uri.contains("/book") {
                assert!(ru.contains(RU_TITLE), "{uri}: the title");
                assert!(!ru.contains(BASE_TITLE), "{uri}");
            }
            if uri.starts_with("/u/testuser?") || uri == "/u/testuser" {
                assert!(ru.contains(RU_BIO), "{uri}: the bio");
                assert!(!ru.contains(BASE_BIO), "{uri}");
                assert!(ru.contains(&format!("<title>{RU_NAME} —")), "{uri}");
                // The initials follow the name shown.
                assert!(ru.contains("ИА"), "{uri}");
            }
        }
        let en = page(&app, uri, "en-US,en;q=0.9").await;
        if uri.ends_with("/test-meeting") {
            // The slot grid moves with the clock: the same host header.
            assert_eq!(
                en.matches("Test User").count(),
                before[i].matches("Test User").count(),
                "{uri}"
            );
            assert!(en.contains(BASE_TITLE), "{uri}");
        } else {
            assert_eq!(en, before[i], "{uri}: an English reader sees no change");
        }
        let fr = page(&app, uri, "fr").await;
        assert!(fr.contains("Test User"), "{uri}");
        assert!(!fr.contains(RU_NAME), "{uri}");
    }
}

/// A field the translation leaves empty shows the base one: a Russian name
/// alone keeps the English title and bio.
#[tokio::test]
async fn an_empty_field_shows_the_base_one() {
    let (app, pool, _, _) = setup_test_app().await;
    let id = russian_host(&pool).await;
    add_text(&pool, &id, "ru", Some(RU_NAME), None, None).await;
    let ru = page(&app, "/u/testuser", "ru").await;
    assert!(ru.contains(RU_NAME));
    assert!(ru.contains(BASE_TITLE));
    assert!(ru.contains(BASE_BIO));
}

/// A team's profile and slot page name each member in the page language.
#[tokio::test]
async fn team_pages_name_their_members_in_the_page_language() {
    let (app, pool, _, _) = setup_test_app().await;
    seed_team_event_type(&pool).await;
    let id = user_id(&pool).await;
    add_text(&pool, &id, "ru", Some(RU_NAME), None, None).await;
    for uri in ["/team/horizon-team", "/team/horizon-team/team-sync"] {
        let ru = page(&app, uri, "ru").await;
        assert!(ru.contains(&format!("title=\"{RU_NAME}\"")), "{uri}");
        assert!(!ru.contains("title=\"Test User\""), "{uri}");
        let en = page(&app, uri, "en").await;
        assert!(en.contains("title=\"Test User\""), "{uri}");
    }
}

/// A booking made in the Russian widget: the confirmation page, the guest's
/// email (body, From display name, the `.ics` ORGANIZER's CN and SUMMARY)
/// and the `.ics` download name the host in Russian; the host's email keeps
/// the base name.
#[tokio::test]
async fn a_russian_booking_names_the_host_in_russian_and_the_host_reads_the_base() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, _, _) = setup_test_app().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, port).await;
    let id = russian_host(&pool).await;
    add_text(&pool, &id, "ru", Some(RU_NAME), Some(RU_TITLE), None).await;
    let csrf = "csrf-user-texts";
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
    assert!(confirmed.contains(RU_NAME), "{confirmed}");
    assert!(!confirmed.contains("Test User"), "{confirmed}");

    let guest = raw_mails_to(&mails, "guest@test.com").await;
    assert_eq!(guest.len(), 1);
    assert!(
        header(&guest[0], "From").contains(RU_NAME),
        "{}",
        header(&guest[0], "From")
    );
    let body = decoded(&guest[0]);
    assert!(body.contains(RU_NAME), "{body}");
    assert!(
        body.contains(&format!("ORGANIZER;CN={RU_NAME}:mailto:")),
        "{body}"
    );
    assert!(body.contains("& Илья"), "the SUMMARY: {body}");
    assert!(!body.contains("Test User"), "{body}");

    let host = raw_mails_to(&mails, "test@example.com").await;
    assert_eq!(host.len(), 1);
    let body = decoded(&host[0]);
    assert!(!body.contains(RU_NAME), "{body}");
    assert!(!body.contains("Илья"), "{body}");

    let cancel_token: String = sqlx::query_scalar(
        "SELECT cancel_token FROM bookings WHERE guest_email = 'guest@test.com'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let response = app
        .clone()
        .oneshot(get(&format!("/booking/ics/{cancel_token}")))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let ics = body_string(response).await;
    assert!(ics.contains(&format!("CN={RU_NAME}")), "{ics}");
    // The guest's cancel page names them in Russian too.
    let cancel = page(&app, &format!("/booking/cancel/{cancel_token}"), "en").await;
    assert!(cancel.contains(RU_NAME), "{cancel}");
}

/// The guest token pages (cancel, the reschedule picker) read the booking's
/// language; an English booking of the same host reads the base name.
#[tokio::test]
async fn the_guest_token_pages_read_the_booking_language() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    let id = russian_host(&pool).await;
    add_text(&pool, &id, "ru", Some(RU_NAME), Some(RU_TITLE), None).await;
    let booking = insert_v1_booking(
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
            .bind(&booking)
            .fetch_one(&pool)
            .await
            .unwrap();
    let body = page(&app, &format!("/booking/cancel/{cancel}"), "en").await;
    assert!(body.contains(RU_NAME), "{body}");
    assert!(!body.contains("Test User"), "{body}");
    let body = page(&app, &format!("/booking/reschedule/{reschedule}"), "en").await;
    assert!(body.contains(RU_NAME), "{body}");
    assert!(body.contains(RU_TITLE), "{body}");
    assert!(!body.contains("Test User"), "{body}");

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
    assert!(body.contains("Test User"), "{body}");
}

/// A guest reschedule in a Russian booking: the confirmation page, the
/// guest's moved invite (body, From, ORGANIZER's CN) name the host in
/// Russian; the host's confirmation keeps the base name.
#[tokio::test]
async fn a_guest_reschedule_names_the_host_in_the_booking_language() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, port).await;
    let id = user_id(&pool).await;
    add_text(&pool, &id, "ru", Some(RU_NAME), None, None).await;
    let booking = insert_v1_booking(
        &pool,
        &et_id,
        "resched@test.com",
        Utc::now() + Duration::days(2),
        30,
        "confirmed",
        &[("language", Some("ru".into()))],
    )
    .await;
    let token: String = sqlx::query_scalar("SELECT reschedule_token FROM bookings WHERE id = ?")
        .bind(&booking)
        .fetch_one(&pool)
        .await
        .unwrap();
    let csrf = "csrf-user-texts-resched";
    let response = app
        .oneshot(post_form_unauthed(
            &format!("/booking/reschedule/{token}"),
            csrf,
            &format!(
                "_csrf={csrf}&date={}&time=10%3A00&tz=UTC",
                far_future_monday()
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let confirmed = body_string(response).await;
    assert!(confirmed.contains(RU_NAME), "{confirmed}");
    assert!(!confirmed.contains("Test User"), "{confirmed}");

    let guest = raw_mails_to(&mails, "resched@test.com").await;
    assert_eq!(guest.len(), 1);
    assert!(header(&guest[0], "From").contains(RU_NAME));
    let body = decoded(&guest[0]);
    assert!(
        body.contains(&format!("ORGANIZER;CN={RU_NAME}:mailto:")),
        "{body}"
    );
    assert!(!body.contains("Test User"), "{body}");
    let host = raw_mails_to(&mails, "test@example.com").await;
    assert_eq!(host.len(), 1);
    assert!(!decoded(&host[0]).contains(RU_NAME));
}

/// The TON guest emails (pay link, renewal, release) name the host in the
/// booking's language.
#[tokio::test]
async fn the_ton_guest_emails_name_the_host_in_the_booking_language() {
    let pool = setup_test_db().await;
    let (user_id, _, et) = seed_test_data(&pool).await;
    add_text(&pool, &user_id, "ru", Some(RU_NAME), None, None).await;
    let ru = insert_v1_booking(
        &pool,
        &et,
        "guest@test.com",
        Utc::now() + Duration::days(3),
        30,
        "pending",
        &[("language", Some("ru".into()))],
    )
    .await;
    let (mail, _) = crate::web::ton_pay::page::guest_mail(&pool, &ru)
        .await
        .unwrap();
    assert_eq!(mail.host_name, RU_NAME);
    let en = insert_v1_booking(
        &pool,
        &et,
        "other@test.com",
        Utc::now() + Duration::days(4),
        30,
        "pending",
        &[("language", Some("en".into()))],
    )
    .await;
    let (mail, _) = crate::web::ton_pay::page::guest_mail(&pool, &en)
        .await
        .unwrap();
    assert_eq!(mail.host_name, "Test User");
}

/// An approval by the host's token: the guest's confirmation names the host
/// in the booking's language, the host's own email in the base name.
#[tokio::test]
async fn an_approval_names_the_host_in_the_guest_language() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, _, et_id) = setup_test_app().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, port).await;
    let id = user_id(&pool).await;
    add_text(&pool, &id, "fr", Some("Utilisateur Test"), None, None).await;
    insert_pending_french_booking(&pool, &et_id, "uid-user-texts-approve").await;
    let response = app
        .oneshot(post_bare("/booking/approve/confirm-uid-user-texts-approve"))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    let guest = raw_mails_to(&mails, "guest@test.com").await;
    assert_eq!(guest.len(), 1);
    let body = decoded(&guest[0]);
    assert!(body.contains("Utilisateur Test"), "{body}");
    assert!(!body.contains("Test User"), "{body}");
    let host = raw_mails_to(&mails, "test@example.com").await;
    assert_eq!(host.len(), 1);
    assert!(!decoded(&host[0]).contains("Utilisateur Test"));
}

/// The guest's reminder names the host in the booking's language; the
/// host's keeps the base name.
#[tokio::test]
async fn the_reminder_names_the_host_in_the_guest_language() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (_app, pool, _, et_id) = setup_test_app().await;
    let (port, mails) = spawn_capturing_mta().await;
    use_capturing_mta(&pool, port).await;
    let id = user_id(&pool).await;
    add_text(&pool, &id, "ru", Some(RU_NAME), None, None).await;
    sqlx::query("UPDATE event_types SET reminder_minutes = 60 WHERE id = ?")
        .bind(&et_id)
        .execute(&pool)
        .await
        .unwrap();
    insert_v1_booking(
        &pool,
        &et_id,
        "guest@test.com",
        Utc::now() + Duration::minutes(30),
        30,
        "confirmed",
        &[("language", Some("ru".into()))],
    )
    .await;
    send_due_reminders(pool.clone(), [0u8; 32]).await;
    let guest = raw_mails_to(&mails, "guest@test.com").await;
    assert_eq!(guest.len(), 1);
    let body = decoded(&guest[0]);
    assert!(body.contains(RU_NAME), "{body}");
    assert!(!body.contains("Test User"), "{body}");
    let host = raw_mails_to(&mails, "test@example.com").await;
    assert_eq!(host.len(), 1);
    assert!(!decoded(&host[0]).contains(RU_NAME));
}

// --- Profile & Settings ---

/// The settings form's fields, `extra` replacing or adding to them.
fn settings_body(csrf: &str, extra: &[(&str, &str)]) -> String {
    let mut fields: Vec<(&str, &str)> = vec![
        ("_csrf", csrf),
        ("name", "Test User"),
        ("booking_email", ""),
        ("timezone", "UTC"),
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

async fn post_settings(app: &Router, session: &str, extra: &[(&str, &str)]) -> (u16, String) {
    let csrf = "csrf-user-texts-settings";
    let response = app
        .clone()
        .oneshot(post_form(
            "/dashboard/settings",
            session,
            csrf,
            &settings_body(csrf, extra),
        ))
        .await
        .unwrap();
    let status = response.status().as_u16();
    (status, body_string(response).await)
}

fn some(value: &str) -> Option<String> {
    Some(value.to_string())
}

/// The page shows the section with the stored rows, saves what it posts
/// (any row numbers, trimmed, an emptied row removed), keeps the rows when
/// the form comes without the section (`config apply`), and removes every
/// one when the section comes with no row.
#[tokio::test]
async fn the_settings_form_shows_saves_and_keeps_the_translations() {
    let (app, pool, session, _) = setup_test_app().await;
    let id = user_id(&pool).await;
    add_text(
        &pool,
        &id,
        "ru",
        Some(RU_NAME),
        Some(RU_TITLE),
        Some(RU_BIO),
    )
    .await;
    let form = body_string(
        app.clone()
            .oneshot(get_authed("/dashboard/settings", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(form.contains(r#"id="profile-translations" open"#), "{form}");
    assert!(form.contains(r#"name="profile_translations" value="1""#));
    assert!(form.contains(RU_NAME));
    assert!(form.contains(RU_TITLE));
    assert!(form.contains(RU_BIO));
    assert!(form.contains(r#"<option value="ru" selected>Русский</option>"#));
    assert!(form.contains(&crate::i18n::translate(
        "en",
        "settings-translations-help",
        None
    )));

    let (status, saved) = post_settings(
        &app,
        &session,
        &[
            ("profile_translations", "1"),
            ("profile_translation_lang_4", "ru"),
            ("profile_translation_name_4", "  Илья  "),
            ("profile_translation_title_4", ""),
            ("profile_translation_bio_4", ""),
            ("profile_translation_lang_1", "de"),
            ("profile_translation_name_1", ""),
            ("profile_translation_title_1", "Technischer Leiter"),
            ("profile_translation_bio_1", "Über **mich**"),
            ("profile_translation_lang_9", "fr"),
            ("profile_translation_name_9", ""),
        ],
    )
    .await;
    assert_eq!(status, 200);
    assert!(saved.contains(&crate::i18n::translate("en", "settings-saved", None)));
    assert_eq!(
        texts_of(&pool, &id).await,
        vec![
            (
                "de".to_string(),
                None,
                some("Technischer Leiter"),
                some("Über **mich**")
            ),
            ("ru".to_string(), some("Илья"), None, None),
        ]
    );
    // The saved page shows the stored rows.
    assert!(saved.contains("Technischer Leiter"));

    // Without the section (`config apply` posts this form too): kept.
    let (status, _) = post_settings(&app, &session, &[("name", "Renamed")]).await;
    assert_eq!(status, 200);
    assert_eq!(texts_of(&pool, &id).await.len(), 2);

    // The section with no row: every translation removed.
    let (status, _) = post_settings(&app, &session, &[("profile_translations", "1")]).await;
    assert_eq!(status, 200);
    assert!(texts_of(&pool, &id).await.is_empty());
}

/// Refusals: a language calrs does not ship, a name past the base name's
/// 255 bytes, one language twice. Nothing is saved (the base fields
/// neither), and the page comes back with the message and what was typed.
/// Without the CSRF token nothing happens at all.
#[tokio::test]
async fn the_settings_form_refuses_bad_translations() {
    let (app, pool, session, _) = setup_test_app().await;
    let id = user_id(&pool).await;
    add_text(&pool, &id, "ru", Some(RU_NAME), None, None).await;
    let long = "я".repeat(128);
    let cases: Vec<(Vec<(&str, &str)>, &str)> = vec![
        (
            vec![
                ("profile_translation_lang_0", "xx"),
                ("profile_translation_name_0", "Typed name"),
            ],
            "form-error-translation-language",
        ),
        (
            vec![
                ("profile_translation_lang_0", "ru"),
                ("profile_translation_name_0", long.as_str()),
            ],
            "settings-error-translation-name-length",
        ),
        (
            vec![
                ("profile_translation_lang_0", "ru"),
                ("profile_translation_name_0", "Один"),
                ("profile_translation_lang_1", "ru"),
                ("profile_translation_title_1", "Два"),
            ],
            "form-error-translation-duplicate",
        ),
    ];
    for (rows, key) in cases {
        let mut extra = vec![("profile_translations", "1"), ("name", "Changed")];
        extra.extend(rows.iter().copied());
        let (status, body) = post_settings(&app, &session, &extra).await;
        assert_eq!(status, 200, "{key}");
        let message = crate::i18n::translate("en", key, None);
        assert!(body.contains(&message), "{key}: {body}");
        assert!(body.contains(rows[1].1), "{key}: what was typed is shown");
        assert_eq!(
            texts_of(&pool, &id).await,
            vec![("ru".to_string(), some(RU_NAME), None, None)],
            "{key}"
        );
        let name: String = sqlx::query_scalar("SELECT name FROM users WHERE id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(name, "Test User", "{key}: nothing written");
    }
    // 255 bytes are fine.
    let ok = "n".repeat(255);
    let (status, _) = post_settings(
        &app,
        &session,
        &[
            ("profile_translations", "1"),
            ("profile_translation_lang_0", "en"),
            ("profile_translation_name_0", ok.as_str()),
        ],
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(
        texts_of(&pool, &id).await[0].1.as_deref(),
        Some(ok.as_str())
    );

    // No CSRF token: refused, nothing written.
    let response = app
        .oneshot(post_form(
            "/dashboard/settings",
            &session,
            "cookie-token",
            &settings_body("other-token", &[("profile_translations", "1")]),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 403);
    assert_eq!(texts_of(&pool, &id).await.len(), 1);
}

/// The dashboard keeps the base name: the sidebar and the settings fields.
#[tokio::test]
async fn the_dashboard_keeps_the_base_name() {
    let (app, pool, session, _) = setup_test_app().await;
    let id = user_id(&pool).await;
    add_text(&pool, &id, "en", Some("Guest-facing Name"), None, None).await;
    let body = body_string(
        app.oneshot(get_authed("/dashboard", &session))
            .await
            .unwrap(),
    )
    .await;
    assert!(body.contains("Test User"));
    assert!(!body.contains("Guest-facing Name"));
}

/// Deleting the user removes their translations (`ON DELETE CASCADE`).
#[tokio::test]
async fn deleting_the_user_removes_the_translations() {
    let pool = setup_test_db().await;
    seed_test_data(&pool).await;
    let other = insert_role_user(&pool, "other@example.com", "user").await;
    add_text(&pool, &other, "ru", Some("Другой"), None, None).await;
    crate::auth::delete_user(&pool, &other, None, None)
        .await
        .unwrap();
    assert!(texts_of(&pool, &other).await.is_empty());
    let third = insert_role_user(&pool, "third@example.com", "user").await;
    add_text(&pool, &third, "ru", Some("Третий"), None, None).await;
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&third)
        .execute(&pool)
        .await
        .unwrap();
    assert!(texts_of(&pool, &third).await.is_empty());
}

// --- config apply ---

mod apply_texts {
    use super::super::apply_support::*;
    use super::*;

    /// The role greps a release for this literal before it sends the key
    /// (`apply::profile::PROFILE_TRANSLATIONS_SHAPE`): it must stay as it is.
    const PROFILE_TRANSLATIONS_SHAPE: &str =
        "profile.translations: {<language>: {name, title, bio}}, the whole set";

    fn doc(profile: Value) -> String {
        json!({
            "format": apply::FORMAT,
            "user": ADMIN_EMAIL,
            "profile": profile,
        })
        .to_string()
    }

    async fn stored(env: &ApplyEnv) -> Vec<Text> {
        user_texts::load(&env.pool, &env.admin_id).await
    }

    fn paths(report: &apply::Report) -> Vec<&str> {
        report.changed.iter().map(|c| c.path.as_str()).collect()
    }

    /// Declared, written; a second run changes nothing; a change is one
    /// entry with both sets; left out, kept, also when the run posts the
    /// settings form for another key; `{}` removes all.
    #[tokio::test]
    async fn apply_declares_the_whole_set_and_a_second_run_changes_nothing() {
        let env = ApplyEnv::new().await;
        let ru = json!({"ru": {"name": RU_NAME, "title": RU_TITLE, "bio": null}});
        let json = doc(json!({ "translations": ru }));
        let report = env.apply(&json, false).await.unwrap();
        assert_eq!(paths(&report), ["profile.translations"]);
        assert_eq!(report.changed[0].before, json!({}));
        assert_eq!(report.changed[0].after, ru);
        assert_eq!(
            stored(&env).await,
            vec![Text {
                lang: "ru".into(),
                name: Some(RU_NAME.into()),
                title: Some(RU_TITLE.into()),
                bio: None,
            }]
        );
        let again = env.apply(&json, false).await.unwrap();
        assert_eq!(again.changed, []);

        // `bio` left out is none, as null.
        let both = json!({
            "de": {"title": "Technischer Leiter"},
            "ru": {"name": RU_NAME, "bio": RU_BIO},
        });
        let report = env
            .apply(&doc(json!({ "translations": both })), false)
            .await
            .unwrap();
        assert_eq!(paths(&report), ["profile.translations"]);
        assert_eq!(report.changed[0].before, ru);
        assert_eq!(
            report.changed[0].after,
            json!({
                "de": {"name": null, "title": "Technischer Leiter", "bio": null},
                "ru": {"name": RU_NAME, "title": null, "bio": RU_BIO},
            })
        );
        assert_eq!(stored(&env).await.len(), 2);

        // Left out while the form is posted for the title: kept.
        let report = env
            .apply(&doc(json!({"title": "Renamed"})), false)
            .await
            .unwrap();
        assert_eq!(paths(&report), ["profile.title"]);
        assert_eq!(stored(&env).await.len(), 2);

        // With a form change in the same run: both, the form first.
        let report = env
            .apply(&doc(json!({"title": "Again", "translations": {}})), false)
            .await
            .unwrap();
        assert_eq!(paths(&report), ["profile.title", "profile.translations"]);
        assert!(stored(&env).await.is_empty());
        let title: Option<String> = sqlx::query_scalar("SELECT title FROM users WHERE id = ?")
            .bind(&env.admin_id)
            .fetch_one(&env.pool)
            .await
            .unwrap();
        assert_eq!(title.as_deref(), Some("Again"));
    }

    /// The export gives the stored set (`{}` for none) and applies back as
    /// no change; a dry run writes nothing; a stored row the document could
    /// not give (a hand edit) leaves the key out.
    #[tokio::test]
    async fn the_export_round_trips() {
        let env = ApplyEnv::new().await;
        let exported = serde_json::to_value(env.export().await.unwrap()).unwrap();
        assert_eq!(exported["profile"]["translations"], json!({}));

        let texts = json!({"de-formal": {"title": "Leiter"}, "ru": {"name": RU_NAME}});
        env.apply(&doc(json!({ "translations": texts })), false)
            .await
            .unwrap();
        let exported = serde_json::to_value(env.export().await.unwrap()).unwrap();
        assert_eq!(
            exported["profile"]["translations"],
            json!({
                "de-formal": {"name": null, "title": "Leiter", "bio": null},
                "ru": {"name": RU_NAME, "title": null, "bio": null},
            })
        );
        let snapshot = env.snapshot().await;
        let report = env.apply(&exported.to_string(), false).await.unwrap();
        assert_eq!(report.changed, []);
        assert_eq!(env.snapshot().await, snapshot);

        let dry = env
            .apply(&doc(json!({"translations": {}})), true)
            .await
            .unwrap();
        assert_eq!(paths(&dry), ["profile.translations"]);
        assert_eq!(env.snapshot().await, snapshot);

        sqlx::query("UPDATE user_texts SET name = ' padded ' WHERE lang = 'ru'")
            .execute(&env.pool)
            .await
            .unwrap();
        let exported = env.export().await.unwrap();
        assert_eq!(exported.profile.unwrap().translations, None);
    }

    /// Refused before anything is written, under the key's path: a language
    /// calrs does not ship (or spelled otherwise), a language with no field,
    /// a name empty, padded or past 255 bytes, a padded title, a bio with a
    /// script link; a language given twice and an unknown key when the
    /// document is read.
    #[tokio::test]
    async fn apply_refuses_bad_translations() {
        let env = ApplyEnv::new().await;
        let long = "я".repeat(128);
        let cases = [
            (json!({"xx": {"name": "N"}}), "profile.translations.xx"),
            (json!({"RU": {"name": "N"}}), "profile.translations.RU"),
            (json!({"ru": {}}), "profile.translations.ru"),
            (
                json!({"ru": {"name": null, "title": null, "bio": null}}),
                "profile.translations.ru",
            ),
            (json!({"ru": {"name": ""}}), "profile.translations.ru.name"),
            (
                json!({"ru": {"name": " N"}}),
                "profile.translations.ru.name",
            ),
            (
                json!({"ru": {"name": long}}),
                "profile.translations.ru.name",
            ),
            (
                json!({"ru": {"title": "T "}}),
                "profile.translations.ru.title",
            ),
            (
                json!({"ru": {"bio": "[x](javascript:alert(1))"}}),
                "profile.translations.ru.bio",
            ),
        ];
        let before = env.snapshot().await;
        for (texts, want) in cases {
            let error = env
                .apply(&doc(json!({ "translations": texts })), false)
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
            .apply(&doc(json!({"translations": {"xx": {"name": "N"}}})), false)
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains(PROFILE_TRANSLATIONS_SHAPE),
            "{error}"
        );
        // One language twice: the JSON object itself is refused.
        let twice = doc(json!({"title": "T"})).replace(
            r#""title":"T""#,
            r#""translations":{"ru":{"name":"A"},"ru":{"name":"B"}}"#,
        );
        assert!(twice.contains(r#""ru":{"name":"A"},"ru""#), "{twice}");
        let error = apply::parse(&twice).unwrap_err();
        assert!(error.to_string().contains("given twice"), "{error}");
        // An unknown key in a language's object.
        let error = apply::parse(&doc(
            json!({"translations": {"ru": {"name": "N", "summary": "S"}}}),
        ))
        .unwrap_err();
        assert!(error.to_string().contains("summary"), "{error}");
        // `translations` may not be null.
        assert!(apply::parse(&doc(json!({ "translations": null }))).is_err());
    }
}
