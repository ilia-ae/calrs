//! ilia fork: real statuses and the styled error page (src/web/error_page.rs),
//! and the instance name in titles (src/brand.rs). Kept apart from
//! upstream's test files, so a rebase does not conflict here.

use super::*;

fn page_request(uri: &str, accept_language: &str) -> axum::http::Request<Body> {
    axum::http::Request::builder()
        .uri(uri)
        .header("accept", "text/html,application/xhtml+xml,*/*;q=0.8")
        .header("accept-language", accept_language)
        .body(Body::empty())
        .unwrap()
}

async fn fetch(app: &Router, request: axum::http::Request<Body>) -> (u16, HeaderMap, String) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    (status, headers, body_string(response).await)
}

fn title_of(html: &str) -> &str {
    let start = html.find("<title>").expect("no title") + "<title>".len();
    let end = start + html[start..].find("</title>").unwrap();
    &html[start..end]
}

/// The error page: a whole document with the theme script, the footer, the
/// title for the status and upstream's message.
fn assert_error_page(html: &str, title: &str, message: &str) {
    assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
    assert!(html.contains("calrs_theme"), "no theme script: {html}");
    assert!(
        html.contains(r#"<div class="powered">"#),
        "no footer: {html}"
    );
    assert!(html.contains(r#"<img src="/logo""#), "no logo: {html}");
    assert!(html.contains(&format!("<h1>{title}</h1>")), "{html}");
    assert!(
        html.contains(&format!(r#"<p class="subtitle">{message}</p>"#)),
        "{html}"
    );
    assert_eq!(title_of(html), format!("{title} — calrs"));
}

#[tokio::test]
async fn unknown_user_is_a_404_page_in_the_visitors_language() {
    let (app, _, _, _) = setup_test_app().await;
    let (status, headers, html) = fetch(&app, page_request("/u/nobody", "ru")).await;
    assert_eq!(status, 404);
    assert_eq!(headers["content-type"], "text/html; charset=utf-8");
    assert!(html.contains(r#"<html lang="ru">"#), "{html}");
    assert_error_page(&html, "Не найдено", "Пользователь не найден.");

    let (status, _, html) = fetch(&app, page_request("/u/nobody/test-meeting", "en")).await;
    assert_eq!(status, 404);
    assert_error_page(&html, "Not found", "User not found.");
}

/// Every layer outside still sets its headers on the new page.
#[tokio::test]
async fn the_error_page_keeps_the_security_headers() {
    let (app, _, _, _) = setup_test_app().await;
    let (status, headers, _) = fetch(&app, page_request("/u/nobody", "en")).await;
    assert_eq!(status, 404);
    for name in [
        "content-security-policy",
        "x-frame-options",
        "x-content-type-options",
        "referrer-policy",
        "x-robots-tag",
    ] {
        assert!(headers.contains_key(name), "{name} missing: {headers:?}");
    }
}

#[tokio::test]
async fn unknown_event_type_and_team_are_404_pages() {
    let (app, _, _, _) = setup_test_app().await;
    for (uri, message) in [
        ("/u/testuser/no-such-meeting", "Event type not found."),
        (
            "/u/testuser/no-such-meeting/book?date=2030-01-07&time=10:00",
            "Event type not found.",
        ),
        ("/no-such-meeting", "Event type not found."),
        ("/team/no-such-team", "Team not found."),
        ("/team/no-such-team/x", "Event type not found."),
    ] {
        let (status, _, html) = fetch(&app, page_request(uri, "en")).await;
        assert_eq!(status, 404, "{uri}: {html}");
        assert_error_page(&html, "Not found", message);
    }
}

/// The page echoes nothing from the request.
#[tokio::test]
async fn the_error_page_does_not_echo_the_request() {
    let (app, _, _, _) = setup_test_app().await;
    for uri in [
        "/u/%3Cscript%3Ealert(1)%3C%2Fscript%3E",
        "/u/secret-probe/secret-slug",
        "/no/such/secret-path/here",
    ] {
        let (status, _, html) = fetch(&app, page_request(uri, "en")).await;
        assert_eq!(status, 404, "{uri}");
        for needle in ["<script>alert", "secret"] {
            assert!(!html.contains(needle), "{uri} echoes {needle}: {html}");
        }
    }
}

/// The error page renders outside the language switcher's page scope, so it
/// has no menu, whose links would carry the request's path and query. A real
/// page still has it, so the check below would see one.
#[tokio::test]
async fn the_error_page_has_no_language_switcher() {
    let (app, _, _, _) = setup_test_app().await;
    let (status, _, html) = fetch(&app, page_request("/u/testuser", "en")).await;
    assert_eq!(status, 200);
    assert!(html.contains(r#"<details class="lang-switch">"#), "{html}");
    for uri in [
        "/u/secret-probe/secret-slug?secret=1",
        "/u/nobody",
        "/no/such/secret-path/here",
    ] {
        let (status, _, html) = fetch(&app, page_request(uri, "en")).await;
        assert_eq!(status, 404, "{uri}");
        assert!(
            !html.contains(r#"<details class="lang-switch">"#),
            "{uri}: {html}"
        );
        for needle in ["?lang=", "&amp;lang=", "secret"] {
            assert!(!html.contains(needle), "{uri} has {needle}: {html}");
        }
    }
}

/// HEAD gets the status and headers of GET, and no body.
#[tokio::test]
async fn head_gets_the_status_without_a_body() {
    let (app, _, _, _) = setup_test_app().await;
    for (uri, status) in [
        ("/u/nobody", 404),
        ("/a/b/c/d", 404),
        ("/booking/cancel/no-such-token", 404),
        ("/u/testuser", 200),
    ] {
        let request = axum::http::Request::builder()
            .method("HEAD")
            .uri(uri)
            .header("accept", "text/html,application/xhtml+xml,*/*;q=0.8")
            .header("accept-language", "en")
            .body(Body::empty())
            .unwrap();
        let (got, headers, body) = fetch(&app, request).await;
        assert_eq!(got, status, "{uri}");
        assert_eq!(headers["content-type"], "text/html; charset=utf-8", "{uri}");
        assert_eq!(body, "", "{uri}");
        // The length of the page a GET would get.
        let len: usize = headers["content-length"].to_str().unwrap().parse().unwrap();
        assert!(len > 1000, "{uri}: {len}");
    }
}

/// A bare message answering a form POST gets its status, whatever the
/// method; the booking error page answering a POST keeps upstream's 200.
#[tokio::test]
async fn post_answers_bare_message_status_booking_error_page_200() {
    let (app, _, _, _) = setup_test_app().await;
    let mut request = post_form_unauthed(
        "/u/testuser/no-such-meeting/book",
        "tok",
        "_csrf=tok&date=2030-01-07&time=10:00&tz=UTC&name=Guest&email=guest%40example.com",
    );
    request
        .headers_mut()
        .insert("accept-language", "en".parse().unwrap());
    let (status, _, html) = fetch(&app, request).await;
    assert_eq!(status, 404, "{html}");
    assert_error_page(&html, "Not found", "Event type not found.");

    let mut request = post_form_unauthed("/booking/cancel/no-such-token", "tok", "_csrf=tok");
    request
        .headers_mut()
        .insert("accept-language", "en".parse().unwrap());
    let (status, _, html) = fetch(&app, request).await;
    assert_eq!(status, 200, "{html}");
    assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
    assert!(html.contains("Invalid link"), "{html}");
}

#[tokio::test]
async fn an_unknown_address_is_a_404_page_for_a_browser_only() {
    let (app, _, _, _) = setup_test_app().await;
    let (status, _, html) = fetch(&app, page_request("/a/b/c/d", "fr")).await;
    assert_eq!(status, 404);
    assert_error_page(
        &html,
        "Introuvable",
        "Il n’y a rien à cette adresse. Vérifiez le lien.",
    );

    // An image, a script or fetch(): upstream's empty 404.
    let (status, _, body) = fetch(&app, get("/a/b/c/d")).await;
    assert_eq!(status, 404);
    assert_eq!(body, "");
    let request = axum::http::Request::builder()
        .uri("/avatar/nobody")
        .header("accept", "image/avif,image/webp,*/*")
        .body(Body::empty())
        .unwrap();
    let (status, _, body) = fetch(&app, request).await;
    assert_eq!(status, 404);
    assert_eq!(body, "");
}

#[tokio::test]
async fn real_pages_are_untouched() {
    let (app, _, _, _) = setup_test_app().await;
    for uri in ["/u/testuser", "/u/testuser/test-meeting", "/auth/login"] {
        let (status, _, html) = fetch(&app, page_request(uri, "en")).await;
        assert_eq!(status, 200, "{uri}");
        assert!(!html.contains("<h1>Not found</h1>"), "{uri}");
    }
}

#[tokio::test]
async fn a_dead_booking_link_is_a_404_with_the_booking_error_page() {
    let (app, _, _, _) = setup_test_app().await;
    for uri in [
        "/booking/cancel/no-such-token",
        "/booking/reschedule/no-such-token",
        "/booking/decline/no-such-token",
        "/booking/claim/no-such-booking?token=x",
    ] {
        let (status, _, html) = fetch(&app, page_request(uri, "en")).await;
        assert_eq!(status, 404, "{uri}: {html}");
        assert!(html.starts_with("<!DOCTYPE html>"), "{uri}");
    }
}

#[tokio::test]
async fn a_cancel_link_of_a_cancelled_booking_is_gone() {
    let (app, pool, _, et_id) = setup_test_app().await;
    let start = (Utc::now() + Duration::days(3))
        .format("%Y-%m-%dT10:00:00")
        .to_string();
    let end = (Utc::now() + Duration::days(3))
        .format("%Y-%m-%dT10:30:00")
        .to_string();
    let (_, _, token) = insert_test_booking(&pool, &et_id, &start, &end, "cancelled").await;
    let (status, _, html) = fetch(
        &app,
        page_request(&format!("/booking/cancel/{token}"), "en"),
    )
    .await;
    assert_eq!(status, 410, "{html}");
    assert!(html.contains("Already cancelled"), "{html}");
}

async fn private_with_invite(
    pool: &SqlitePool,
    et_id: &str,
    expires_at: Option<&str>,
    used: i32,
) -> String {
    sqlx::query("UPDATE event_types SET visibility = 'private' WHERE id = ?")
        .bind(et_id)
        .execute(pool)
        .await
        .unwrap();
    let token = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO booking_invites (id, event_type_id, token, guest_name, guest_email, expires_at, max_uses, used_count, created_by_user_id) \
         SELECT ?, ?, ?, 'Bob', 'bob@example.net', ?, 1, ?, id FROM users WHERE username = 'testuser'",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(et_id)
    .bind(&token)
    .bind(expires_at)
    .bind(used)
    .execute(pool)
    .await
    .unwrap();
    token
}

#[tokio::test]
async fn invite_links_get_their_status() {
    let (app, pool, _, et_id) = setup_test_app().await;
    let expired = private_with_invite(&pool, &et_id, Some("2000-01-01 00:00:00"), 0).await;
    let used = private_with_invite(&pool, &et_id, None, 1).await;
    let fresh = private_with_invite(&pool, &et_id, None, 0).await;
    for (query, status, title, message) in [
        (
            "",
            403,
            "Access restricted",
            "This event type requires an invite link.",
        ),
        ("?invite=nope", 404, "Not found", "Invalid invite link."),
        (
            &format!("?invite={expired}") as &str,
            410,
            "Link no longer valid",
            "This invite link has expired.",
        ),
        (
            &format!("?invite={used}"),
            410,
            "Link no longer valid",
            "This invite link has already been used.",
        ),
    ] {
        let uri = format!("/u/testuser/test-meeting{query}");
        let (got, _, html) = fetch(&app, page_request(&uri, "en")).await;
        assert_eq!(got, status, "{uri}: {html}");
        assert_error_page(&html, title, message);
    }
    let uri = format!("/u/testuser/test-meeting?invite={fresh}");
    let (status, _, _) = fetch(&app, page_request(&uri, "en")).await;
    assert_eq!(status, 200);
}

#[tokio::test]
async fn an_invalid_date_is_a_400() {
    let (app, _, _, _) = setup_test_app().await;
    let (status, _, html) = fetch(
        &app,
        page_request(
            "/u/testuser/test-meeting/book?date=not-a-date&time=10:00",
            "en",
        ),
    )
    .await;
    assert_eq!(status, 400, "{html}");
    assert!(html.contains("<h1>Invalid request</h1>"), "{html}");
}

/// Embedded, the page drops its chrome like the booking pages do.
#[tokio::test]
async fn embedded_error_page_follows_the_embed_theme() {
    let (app, _, _, _) = setup_test_app().await;
    let (status, headers, html) =
        fetch(&app, page_request("/u/nobody?embed=1&theme=dark", "en")).await;
    assert_eq!(status, 404);
    assert!(html.contains("/* Embed mode"), "{html}");
    assert!(
        html.contains(r#"<meta name="color-scheme" content="dark">"#),
        "{html}"
    );
    assert!(
        headers["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("frame-ancestors *"),
        "{headers:?}"
    );
}

/// The page links back to the company site when one is set. Without one
/// there is no button (this site's root `/` redirects to the owner's login),
/// and none when embedded.
#[tokio::test]
async fn the_error_page_links_back_to_the_company_site_only() {
    let (app, _, _, _) = setup_test_app().await;
    let (status, _, html) = fetch(&app, page_request("/u/nobody", "en")).await;
    assert_eq!(status, 404);
    assert!(!html.contains("Back to the site"), "{html}");
    assert!(
        !html.contains(r#"class="btn btn-secondary btn-auto""#),
        "{html}"
    );
    assert!(
        html.contains(r#"<span class="result-icon muted""#),
        "{html}"
    );
    let (status, _, html) = fetch(&app, page_request("/u/nobody?embed=1", "en")).await;
    assert_eq!(status, 404);
    assert!(!html.contains("Back to the site"), "{html}");

    let pool = setup_test_db().await;
    seed_test_data(&pool).await;
    sqlx::query("UPDATE auth_config SET company_link = ? WHERE id = 'singleton'")
        .bind("https://example.com/")
        .execute(&pool)
        .await
        .unwrap();
    let data_dir = std::sync::Arc::new(TestDataDir::new());
    let app = create_router(pool, data_dir.to_path_buf(), [0u8; 32])
        .await
        .layer(axum::Extension(data_dir))
        .layer(local_proxy_peer());
    let (status, _, html) = fetch(&app, page_request("/u/nobody", "ru")).await;
    assert_eq!(status, 404);
    // minijinja escapes the slashes of an attribute value (`&#x2f;`).
    let start = html
        .find(r#"<a class="btn btn-secondary btn-auto" href=""#)
        .unwrap_or_else(|| panic!("no way back: {html}"));
    let button = &html[start..start + html[start..].find("</a>").unwrap()];
    assert!(
        button.contains("example.com") && button.ends_with(">На сайт"),
        "{button}"
    );
}

/// Every shipped translation of a mapped message is recognised, with the
/// status of its key.
#[test]
fn every_translation_of_a_mapped_message_is_recognised() {
    for (lang, _) in crate::i18n::supported_with_labels() {
        for (key, status) in crate::web::error_page::BARE {
            let text = crate::i18n::translate(lang, key, None);
            let entry =
                crate::web::error_page::bare_entry(&text).unwrap_or_else(|| panic!("{lang} {key}"));
            assert_eq!(
                crate::web::error_page::BARE[entry].1,
                *status,
                "{lang} {key}"
            );
        }
    }
    assert_eq!(crate::web::error_page::bare_entry("User not found"), None);
    assert_eq!(crate::web::error_page::bare_entry(""), None);
}

/// Titles that mean different statuses must read differently in every
/// language, or the booking error page could get the wrong one.
#[test]
fn booking_error_titles_are_distinct_per_language() {
    for (lang, _) in crate::i18n::supported_with_labels() {
        let mut seen = std::collections::HashMap::new();
        for (key, status) in crate::web::error_page::BOOKING_ERRORS {
            let title = crate::i18n::translate(lang, key, None);
            if let Some(other) = seen.insert(title.clone(), *status) {
                assert_eq!(other, *status, "{lang}: {title}");
            }
        }
    }
}

// --- The instance name (CALRS_BRAND_NAME, src/brand.rs) ---

/// The test router with a brand, as `build_state` registers it when
/// `CALRS_BRAND_NAME` is set (the environment is read once per process, so
/// tests set the global directly).
async fn branded_app(brand: &str) -> Router {
    let pool = setup_test_db().await;
    seed_test_data(&pool).await;
    let data_dir = std::sync::Arc::new(TestDataDir::new());
    let mut state = build_state(pool, data_dir.to_path_buf(), [0u8; 32]).await;
    crate::brand::register(
        &mut Arc::get_mut(&mut state).unwrap().templates,
        Some(brand),
    );
    router(state)
        .layer(axum::Extension(data_dir))
        .layer(local_proxy_peer())
}

#[tokio::test]
async fn the_brand_replaces_calrs_in_titles_and_on_the_login_page() {
    let app = branded_app("ilia.ae").await;
    let (_, _, html) = fetch(&app, page_request("/auth/login", "en")).await;
    assert_eq!(title_of(&html), "Sign in — ilia.ae");
    assert!(
        html.contains(r#"<p class="subtitle">Sign in to your ilia.ae account</p>"#),
        "{html}"
    );
    let (_, _, html) = fetch(&app, page_request("/auth/login", "ru")).await;
    assert!(
        html.contains(r#"<p class="subtitle">Войдите в свой аккаунт ilia.ae</p>"#),
        "{html}"
    );
    // A title with a value of the page in it.
    let (_, _, html) = fetch(&app, page_request("/u/testuser", "en")).await;
    assert_eq!(title_of(&html), "Test User — ilia.ae");
    // The error page too.
    let (_, _, html) = fetch(&app, page_request("/u/nobody", "en")).await;
    assert_eq!(title_of(&html), "Not found — ilia.ae");
}

#[tokio::test]
async fn without_a_brand_titles_stay_upstreams() {
    let (app, _, _, _) = setup_test_app().await;
    let (_, _, html) = fetch(&app, page_request("/auth/login", "en")).await;
    assert_eq!(title_of(&html), "Sign in — calrs");
    assert!(
        html.contains(r#"<p class="subtitle">Sign in to your calrs account</p>"#),
        "{html}"
    );
}

/// The brand and the page's own values are escaped once, not twice.
#[test]
fn brand_titles_escape_once() {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_loader(minijinja::path_loader("templates"));
    crate::i18n::register(&mut env);
    crate::brand::register(&mut env, Some("<i>ilia</i> & co"));
    env.add_template(
        "named.html",
        r#"{% extends "base.html" %}{% block title %}{{ name }} — calrs{% endblock %}"#,
    )
    .unwrap();
    env.add_template("plain.html", r#"{% extends "base.html" %}"#)
        .unwrap();
    let named = env
        .get_template("named.html")
        .unwrap()
        .render(minijinja::context! { name => "O'Brien & <Co>" })
        .unwrap();
    assert_eq!(
        title_of(&named),
        "O&#x27;Brien &amp; &lt;Co&gt; — &lt;i&gt;ilia&lt;&#x2f;i&gt; &amp; co"
    );
    let plain = env
        .get_template("plain.html")
        .unwrap()
        .render(minijinja::context! {})
        .unwrap();
    assert_eq!(title_of(&plain), "&lt;i&gt;ilia&lt;&#x2f;i&gt; &amp; co");
}

/// A title that does not name calrs is left alone.
#[test]
fn brand_leaves_other_titles_alone() {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_loader(minijinja::path_loader("templates"));
    crate::i18n::register(&mut env);
    crate::brand::register(&mut env, Some("ilia.ae"));
    env.add_template(
        "t.html",
        r#"{% extends "base.html" %}{% block title %}Pay with TON{% endblock %}"#,
    )
    .unwrap();
    let html = env.get_template("t.html").unwrap().render(()).unwrap();
    assert_eq!(title_of(&html), "Pay with TON");
}
