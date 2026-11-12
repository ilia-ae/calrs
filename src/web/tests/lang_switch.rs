//! ilia fork: the language switcher of public pages (`web::lang_switch`).

use super::*;
use crate::web::lang_switch::{cookie, entries, keep, same_origin_target, with_page, Keep};

fn request(uri: &str, cookie: Option<&str>, accept: Option<&str>) -> axum::http::Request<Body> {
    let mut builder = axum::http::Request::builder().uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    if let Some(accept) = accept {
        builder = builder.header("accept-language", accept);
    }
    builder.body(Body::empty()).unwrap()
}

fn headers_with(pairs: &[(&'static str, &str)]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        headers.append(*name, value.parse().unwrap());
    }
    headers
}

/// The `<html lang>` of a page.
fn html_lang(body: &str) -> &str {
    let start = body.find("<html lang=\"").expect("no <html lang>") + "<html lang=\"".len();
    &body[start..start + body[start..].find('"').unwrap()]
}

const SWITCHER: &str = r#"<details class="lang-switch">"#;

async fn page(app: &Router, uri: &str, cookie: Option<&str>, accept: Option<&str>) -> String {
    let response = app
        .clone()
        .oneshot(request(uri, cookie, accept))
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{uri}");
    body_string(response).await
}

async fn set_page_mode(pool: &SqlitePool, mode: &str) {
    sqlx::query("UPDATE users SET public_page_language = ? WHERE email = 'test@example.com'")
        .bind(mode)
        .execute(pool)
        .await
        .unwrap();
}

// --- The cookie ---

#[test]
fn the_cookie_is_read_only_when_it_names_a_bundled_language() {
    let chosen =
        |cookie: &str| crate::i18n::chosen_from_headers(&headers_with(&[("cookie", cookie)]));
    assert_eq!(chosen("calrs_lang=ru"), Some("ru"));
    assert_eq!(
        chosen("__Host-calrs_csrf=x; calrs_lang=pt; other=1"),
        Some("pt")
    );
    assert_eq!(chosen("calrs_lang=de-formal"), Some("de-formal"));
    for bad in [
        "calrs_lang=xx",
        "calrs_lang=",
        "calrs_lang=RU",
        "calrs_lang=\"ru\"",
        "calrs_lang=ru-RU",
        "calrs_lang=../en",
        "calrs_lang=<script>",
        "xcalrs_lang=ru",
        "calrs_language=ru",
    ] {
        assert_eq!(chosen(bad), None, "{bad}");
    }
    assert_eq!(
        crate::i18n::chosen_from_headers(&HeaderMap::new()),
        None,
        "no cookie"
    );
    // Two Cookie headers: the first `calrs_lang` counts.
    let two = headers_with(&[
        ("cookie", "a=1; calrs_lang=fr"),
        ("cookie", "calrs_lang=ru"),
    ]);
    assert_eq!(crate::i18n::chosen_from_headers(&two), Some("fr"));
}

/// The switcher offers one German: on an instance addressing visitors as
/// "Sie" a picked `de` is formal, as a browser's German is.
#[test]
fn a_picked_german_follows_the_instance_register() {
    let formal = headers_with(&[
        ("cookie", "calrs_lang=de"),
        (crate::i18n::GERMAN_FORMAL_HEADER, "1"),
    ]);
    assert_eq!(crate::i18n::chosen_from_headers(&formal), Some("de-formal"));
    assert_eq!(crate::i18n::detect_from_headers(&formal), "de-formal");
    let informal = headers_with(&[("cookie", "calrs_lang=de")]);
    assert_eq!(crate::i18n::detect_from_headers(&informal), "de");
}

#[test]
fn resolution_order_pick_then_stored_then_browser() {
    let browser_fr = headers_with(&[("accept-language", "fr-FR,fr;q=0.9")]);
    let picked_ru = headers_with(&[
        ("accept-language", "fr-FR,fr;q=0.9"),
        ("cookie", "calrs_lang=ru"),
    ]);
    let bad_pick = headers_with(&[
        ("accept-language", "fr-FR,fr;q=0.9"),
        ("cookie", "calrs_lang=zz"),
    ]);
    // Visitor pages.
    assert_eq!(crate::i18n::detect_from_headers(&browser_fr), "fr");
    assert_eq!(crate::i18n::detect_from_headers(&picked_ru), "ru");
    assert_eq!(crate::i18n::detect_from_headers(&bad_pick), "fr");
    // Pages with a language of their own (booking, host).
    assert_eq!(crate::i18n::page_lang(Some("de"), &browser_fr), "de");
    assert_eq!(crate::i18n::page_lang(Some("de"), &picked_ru), "ru");
    assert_eq!(crate::i18n::page_lang(Some("de"), &bad_pick), "de");
    assert_eq!(crate::i18n::page_lang(Some("ja"), &browser_fr), "fr");
    assert_eq!(crate::i18n::page_lang(None, &picked_ru), "ru");
    // The dashboard: a saved language beats the pick.
    assert_eq!(crate::i18n::resolve(Some("de"), &picked_ru), "de");
    assert_eq!(crate::i18n::resolve(None, &picked_ru), "ru");
}

#[test]
fn cookie_attributes() {
    assert_eq!(
        cookie("ru", true, true),
        "calrs_lang=ru; Path=/; Max-Age=31536000; SameSite=Lax; Secure"
    );
    assert_eq!(
        cookie("ru", false, true),
        "calrs_lang=ru; Path=/; Max-Age=31536000; SameSite=Lax"
    );
    // A link from elsewhere: for the browser session only.
    assert_eq!(
        cookie("ru", true, false),
        "calrs_lang=ru; Path=/; SameSite=Lax; Secure"
    );
    assert_eq!(
        cookie("ru", false, false),
        "calrs_lang=ru; Path=/; SameSite=Lax"
    );
}

// --- The redirect target ---

#[test]
fn redirect_target_stays_on_this_origin() {
    assert_eq!(same_origin_target("/u/ilia", Some("lang=ru")), "/u/ilia");
    assert_eq!(
        same_origin_target(
            "/u/ilia/intro",
            Some("month=2026-12&lang=ru&tz=Asia%2FDubai")
        ),
        "/u/ilia/intro?month=2026-12&tz=Asia%2FDubai"
    );
    // Every `lang`, with or without a value, goes; lookalikes stay.
    assert_eq!(
        same_origin_target("/x", Some("lang&lang=en&language=1&flang=2&&a=")),
        "/x?language=1&flang=2&a="
    );
    // Never protocol-relative, never a backslash a browser reads as `/`.
    assert_eq!(
        same_origin_target("//evil.example/x", Some("lang=ru")),
        "/evil.example/x"
    );
    assert_eq!(same_origin_target("/\\evil.example", None), "/evil.example");
    assert_eq!(
        same_origin_target("///\\/evil.example", None),
        "/evil.example"
    );
    assert_eq!(same_origin_target("", None), "/");
    // Nothing that could split a header or leave printable ASCII.
    assert_eq!(same_origin_target("/a\r\nSet-Cookie:x=1", None), "/");
    assert_eq!(same_origin_target("/a", Some("b=\r\nX: 1")), "/");
    assert_eq!(same_origin_target("/a b", None), "/");
    assert_eq!(same_origin_target("/a\\b", None), "/");
    assert_eq!(same_origin_target("/я", None), "/");
    // Percent-encoding is kept as it arrived, not decoded.
    assert_eq!(
        same_origin_target("/u/%2F%2Fevil", Some("next=%0D%0A")),
        "/u/%2F%2Fevil?next=%0D%0A"
    );
}

#[tokio::test]
async fn lang_param_sets_the_cookie_and_redirects_to_the_clean_page() {
    let (app, _, _, _) = setup_test_app().await;
    let response = app
        .clone()
        .oneshot(get("/u/testuser?lang=ru&month=2026-12"))
        .await
        .unwrap();
    assert_eq!(response.status(), 303);
    assert_eq!(response.headers()["location"], "/u/testuser?month=2026-12");
    assert_eq!(response.headers()["cache-control"], "no-store");
    let cookies: Vec<&str> = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap())
        .collect();
    assert!(
        cookies
            .iter()
            .any(|c| c.starts_with("calrs_lang=ru; Path=/; Max-Age=31536000; SameSite=Lax")),
        "{cookies:?}"
    );

    // The page it leads to, with the cookie, is Russian.
    let body = page(
        &app,
        "/u/testuser?month=2026-12",
        Some("calrs_lang=ru"),
        Some("en"),
    )
    .await;
    assert_eq!(html_lang(&body), "ru");
}

#[tokio::test]
async fn an_unknown_lang_sets_nothing_and_is_not_echoed() {
    let (app, _, _, _) = setup_test_app().await;
    for value in [
        "xx",
        "%3Cscript%3E",
        "ru%0D%0ASet-Cookie:%20a=1",
        "https://evil.example",
    ] {
        let response = app
            .clone()
            .oneshot(get(&format!("/u/testuser?lang={value}&a=1")))
            .await
            .unwrap();
        assert_eq!(response.status(), 303, "{value}");
        assert_eq!(response.headers()["location"], "/u/testuser?a=1", "{value}");
        assert!(
            !response
                .headers()
                .get_all("set-cookie")
                .iter()
                .any(|c| c.to_str().unwrap().starts_with("calrs_lang=")),
            "{value}"
        );
        let body = body_string(response).await;
        assert!(!body.contains("evil"), "{value}: {body}");
        assert!(!body.contains("script"), "{value}: {body}");
    }
}

#[tokio::test]
async fn a_protocol_relative_path_redirects_to_this_origin() {
    let (app, _, _, _) = setup_test_app().await;
    let response = app.oneshot(get("//evil.example/u?lang=ru")).await.unwrap();
    assert_eq!(response.status(), 303);
    assert_eq!(response.headers()["location"], "/evil.example/u");
}

/// A form post never changes language: only GET links do.
#[tokio::test]
async fn lang_on_a_post_is_not_a_switch() {
    let (app, _, _, _) = setup_test_app().await;
    let response = app
        .oneshot(post_form_unauthed(
            "/auth/login?lang=ru",
            "csrf-x",
            "_csrf=csrf-x",
        ))
        .await
        .unwrap();
    assert_ne!(response.status(), 303);
    assert!(!response
        .headers()
        .get_all("set-cookie")
        .iter()
        .any(|c| c.to_str().unwrap().starts_with("calrs_lang=")));
}

// --- Which language the pages take ---

#[tokio::test]
async fn the_pick_beats_the_browser_on_public_pages() {
    let (app, _, _, _) = setup_test_app().await;
    for uri in ["/u/testuser", "/u/testuser/test-meeting", "/auth/login"] {
        let body = page(&app, uri, None, Some("fr-FR,fr;q=0.9")).await;
        assert_eq!(html_lang(&body), "fr", "{uri}");
        let body = page(&app, uri, Some("calrs_lang=ru"), Some("fr-FR,fr;q=0.9")).await;
        assert_eq!(html_lang(&body), "ru", "{uri}");
        let body = page(&app, uri, Some("calrs_lang=zz"), Some("fr-FR,fr;q=0.9")).await;
        assert_eq!(html_lang(&body), "fr", "{uri}");
    }
}

/// "Always my language" gives the host's language until the visitor picks
/// one: they asked for it.
#[tokio::test]
async fn the_pick_beats_a_host_who_chose_always_my_language() {
    let (app, pool, _, _) = setup_test_app().await;
    set_user_language(&pool, "test@example.com", "de").await;
    set_page_mode(&pool, "host").await;
    let body = page(&app, "/u/testuser/test-meeting", None, Some("fr")).await;
    assert_eq!(html_lang(&body), "de");
    let body = page(
        &app,
        "/u/testuser/test-meeting",
        Some("calrs_lang=ru"),
        Some("fr"),
    )
    .await;
    assert_eq!(html_lang(&body), "ru");
}

/// The dashboard keeps the saved language; a user without one gets the pick.
#[tokio::test]
async fn the_dashboard_follows_the_pick_only_without_a_saved_language() {
    let (app, pool, session, _) = setup_test_app().await;
    let cookie = format!("__Host-calrs_session={session}; calrs_lang=ru");
    sqlx::query("UPDATE users SET language = NULL WHERE email = 'test@example.com'")
        .execute(&pool)
        .await
        .unwrap();
    let body = page(&app, "/dashboard", Some(cookie.as_str()), Some("fr")).await;
    assert_eq!(html_lang(&body), "ru");
    assert!(!body.contains(SWITCHER), "no switcher on the dashboard");

    set_user_language(&pool, "test@example.com", "de").await;
    let body = page(&app, "/dashboard", Some(cookie.as_str()), Some("fr")).await;
    assert_eq!(html_lang(&body), "de");
}

/// A guest token page has the booking's language, unless the guest picks
/// another there.
#[tokio::test]
async fn the_pick_beats_the_booking_language_on_its_guest_pages() {
    let (app, pool, _, et_id) = setup_test_app().await;
    let cancel_tok = uuid::Uuid::new_v4().to_string();
    let resched_tok = uuid::Uuid::new_v4().to_string();
    let booking_id = insert_booking_at(
        &pool,
        &et_id,
        &cancel_tok,
        &resched_tok,
        chrono::Duration::hours(48),
    )
    .await;
    sqlx::query("UPDATE bookings SET language = 'fr' WHERE id = ?")
        .bind(&booking_id)
        .execute(&pool)
        .await
        .unwrap();
    let uri = format!("/booking/cancel/{cancel_tok}");
    let body = page(&app, &uri, None, Some("en")).await;
    assert_eq!(html_lang(&body), "fr");
    let body = page(&app, &uri, Some("calrs_lang=ru"), Some("en")).await;
    assert_eq!(html_lang(&body), "ru");
    assert!(body.contains("Отмена записи"), "{body}");
    assert!(body.contains(SWITCHER));
}

// --- The menu ---

/// Every bundled language renders the menu, with its own entry current and
/// the others plain; German is listed once.
#[tokio::test]
async fn every_language_renders_the_switcher_with_its_entry_current() {
    let (app, _, _, _) = setup_test_app().await;
    let langs: Vec<(&str, &str)> = crate::i18n::supported_with_labels()
        .filter(|(code, _)| *code != "de-formal")
        .collect();
    assert_eq!(langs.len(), 10);
    for (code, label) in &langs {
        let cookie = format!("calrs_lang={code}");
        let body = page(
            &app,
            "/auth/login?next=%2Fdashboard",
            Some(cookie.as_str()),
            None,
        )
        .await;
        let start = body
            .find(SWITCHER)
            .unwrap_or_else(|| panic!("{code}: no switcher"));
        let menu = &body[start..start + body[start..].find("</details>").unwrap()];
        assert_eq!(menu.matches("<li>").count(), 10, "{code}: {menu}");
        assert_eq!(menu.matches("aria-current").count(), 1, "{code}");
        assert!(
            menu.contains(&format!(r#"aria-current="true">{label}</a>"#)),
            "{code}: {menu}"
        );
        assert!(
            // The closed menu names the current language, as BTCPay's footer does.
            menu.contains(&format!("<span>{label}</span>")),
            "{code}: {menu}"
        );
        assert!(!menu.contains("Deutsch (Sie)"), "{code}");
        for (other, other_label) in &langs {
            assert!(
                menu.contains(&format!(
                    r#"href="/auth/login?next=%2Fdashboard&amp;lang={other}""#
                )),
                "{code} → {other}: {menu}"
            );
            assert!(menu.contains(&format!(">{other_label}</a>")), "{other}");
        }
        assert!(
            menu.contains(&format!(
                r#"aria-label="{}""#,
                crate::i18n::translate(code, "base-language-choose", None)
            )),
            "{code}"
        );
    }
}

/// Native names, sorted: Latin script first, Cyrillic last.
#[test]
fn entries_are_sorted_native_names() {
    let labels: Vec<String> = entries(Some("/u/x"), "en")
        .iter()
        .map(|e| e.get_attr("label").unwrap().to_string())
        .collect();
    assert_eq!(
        labels,
        [
            "Deutsch",
            "Eesti",
            "English",
            "Español",
            "Français",
            "Italiano",
            "Nederlands",
            "Polski",
            "Português (Brasil)",
            "Русский"
        ]
    );
    // German in the formal register marks the one German entry.
    let current: Vec<String> = entries(Some("/u/x"), "de-formal")
        .iter()
        .filter(|e| e.get_attr("current").unwrap().is_true())
        .map(|e| e.get_attr("code").unwrap().to_string())
        .collect();
    assert_eq!(current, ["de"]);
    assert!(entries(None, "en").is_empty());
}

/// The page's own URL is escaped for the attribute it goes into.
#[test]
fn hrefs_are_attribute_escaped() {
    let e = entries(Some("/u/x?a=\"'<>"), "en");
    let href = e[0].get_attr("href").unwrap().to_string();
    assert_eq!(href, "/u/x?a=&quot;&#39;&lt;&gt;&amp;lang=de");
}

fn render_login(ctx: minijinja::Value) -> String {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_loader(minijinja::path_loader("templates"));
    crate::i18n::register(&mut env);
    crate::web::ilia::register(&mut env, &Default::default());
    with_page("/auth/login", || {
        env.get_template("auth/login.html")
            .unwrap()
            .render(ctx)
            .unwrap()
    })
}

#[test]
fn embedded_pages_have_no_switcher() {
    assert!(render_login(context! { lang => "ru" }).contains(SWITCHER));
    let embedded = render_login(context! { lang => "ru", embed => true });
    assert!(!embedded.contains(SWITCHER), "{embedded}");
    // The embed stylesheet hides it too, should a page render it anyway.
    let css = &embedded[embedded.find("/* Embed mode").unwrap()..];
    let hidden = css
        .lines()
        .find(|l| l.contains("display: none !important"))
        .unwrap();
    assert!(hidden.contains(".lang-switch"), "{hidden}");
}

/// Embedded, `?lang=` is not a switch (a third-party iframe keeps no
/// `SameSite=Lax` cookie) and the widget still renders.
#[tokio::test]
async fn embedded_booking_pages_ignore_lang_and_show_no_switcher() {
    let (app, _, _, _) = setup_test_app().await;
    for uri in [
        "/u/testuser/test-meeting?embed=1",
        "/u/testuser/test-meeting?embed=1&lang=ru",
    ] {
        let body = page(&app, uri, None, Some("en")).await;
        assert!(!body.contains(SWITCHER), "{uri}");
        assert_eq!(html_lang(&body), "en", "{uri}");
    }
}

/// A page answering a form POST has no URL to come back to.
#[tokio::test]
async fn a_post_answer_has_no_switcher() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, _, _, _) = setup_test_app().await;
    let csrf = "csrf-switch-post";
    let response = app
        .oneshot(post_form_unauthed(
            "/u/testuser/test-meeting/book",
            csrf,
            &book_body(csrf),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = body_string(response).await;
    assert!(body.contains("<html"), "{body}");
    assert!(!body.contains(SWITCHER));
}

// --- The booking language ---

/// A booking stores the language the guest picked, and its emails follow;
/// on a host's "always my language" page too.
#[tokio::test]
async fn a_booking_stores_the_picked_language() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    for mode in ["visitor", "host"] {
        let (app, pool, _, _) = setup_test_app().await;
        let (port, mails) = spawn_capturing_mta().await;
        use_capturing_mta(&pool, port).await;
        set_user_language(&pool, "test@example.com", "fr").await;
        set_page_mode(&pool, mode).await;
        let csrf = "csrf-switch-book";
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/u/testuser/test-meeting/book")
                    .header("cookie", format!("__Host-calrs_csrf={csrf}; calrs_lang=ru"))
                    .header("accept-language", "en-US,en;q=0.9")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(book_body(csrf)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200, "{mode}");
        assert_eq!(html_lang(&body_string(response).await), "ru", "{mode}");
        let stored: Option<String> = sqlx::query_scalar(
            "SELECT language FROM bookings WHERE guest_email = 'guest@test.com'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(stored.as_deref(), Some("ru"), "{mode}");
        let guest = mail_to(&mails, "guest@test.com").await;
        assert!(guest.contains("Здравствуйте, Guest!"), "{mode}: {guest}");
        assert!(guest.contains("<html lang=\"ru\">"), "{mode}");
        // The host still reads theirs in their own language.
        let host = mail_to(&mails, "test@example.com").await;
        assert!(host.contains("<html lang=\"fr\">"), "{mode}: {host}");
    }
}

/// A guest who picks a language on the reschedule page gets the update in
/// it, and the booking keeps it for the reminders; without a pick the
/// stored language stays.
#[tokio::test]
async fn a_reschedule_with_a_pick_moves_the_booking_language() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    for (pick, expected) in [(Some("ru"), "ru"), (None, "fr")] {
        let (app, pool, _, et_id) = setup_test_app().await;
        let (port, mails) = spawn_capturing_mta().await;
        use_capturing_mta(&pool, port).await;
        let cancel_tok = uuid::Uuid::new_v4().to_string();
        let resched_tok = uuid::Uuid::new_v4().to_string();
        let booking_id = insert_booking_at(
            &pool,
            &et_id,
            &cancel_tok,
            &resched_tok,
            chrono::Duration::hours(48),
        )
        .await;
        sqlx::query("UPDATE bookings SET language = 'fr' WHERE id = ?")
            .bind(&booking_id)
            .execute(&pool)
            .await
            .unwrap();
        let csrf = "csrf-switch-resched";
        let cookie = match pick {
            Some(code) => format!("__Host-calrs_csrf={csrf}; calrs_lang={code}"),
            None => format!("__Host-calrs_csrf={csrf}"),
        };
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/booking/reschedule/{resched_tok}"))
                    .header("cookie", cookie)
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(format!(
                        "_csrf={csrf}&date={}&time=10:00&tz=UTC",
                        far_future_monday()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status() == 200 || response.status().is_redirection());
        let stored: Option<String> =
            sqlx::query_scalar("SELECT language FROM bookings WHERE id = ?")
                .bind(&booking_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored.as_deref(), Some(expected), "{pick:?}");
        let guest = mail_to(&mails, "guest@test.com").await;
        assert!(
            guest.contains(&format!("<html lang=\"{expected}\">")),
            "{pick:?}: {guest}"
        );
    }
}

// --- `?lang=` links from elsewhere ---

/// Only a link followed from this site (or one on its domain) keeps the
/// language for a year; one from elsewhere keeps it for the session, and
/// never replaces the visitor's own pick.
#[test]
fn how_long_a_lang_link_is_kept_depends_on_where_it_came_from() {
    let site = |value: &str| headers_with(&[("sec-fetch-site", value)]);
    assert_eq!(keep(&HeaderMap::new()), Keep::Year, "no Sec-Fetch-Site");
    assert_eq!(keep(&site("same-origin")), Keep::Year);
    assert_eq!(keep(&site("same-site")), Keep::Year);
    assert_eq!(keep(&site("cross-site")), Keep::Session);
    assert_eq!(keep(&site("none")), Keep::Session, "a chat or mail app");
    assert_eq!(
        keep(&site("Same-Origin")),
        Keep::Session,
        "exact values only"
    );
    let picked =
        |value: &str| headers_with(&[("sec-fetch-site", value), ("cookie", "calrs_lang=ru")]);
    assert_eq!(keep(&picked("same-origin")), Keep::Year, "a new pick");
    assert_eq!(keep(&picked("cross-site")), Keep::Not);
    assert_eq!(keep(&picked("none")), Keep::Not);
    // A cookie calrs does not ship is no pick to protect.
    let bad = headers_with(&[
        ("sec-fetch-site", "cross-site"),
        ("cookie", "calrs_lang=zz"),
    ]);
    assert_eq!(keep(&bad), Keep::Session);
}

fn lang_cookies(response: &axum::response::Response) -> Vec<String> {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .filter(|c| c.starts_with("calrs_lang="))
        .collect()
}

/// Someone posts `/u/testuser?lang=pl` in a chat: the visitor who opens it
/// sees that visit in Polish, but a later visit (and a booking from it) is
/// not Polish for a year, and a visitor who already picked keeps theirs.
#[tokio::test]
async fn a_lang_link_from_elsewhere_does_not_stick_for_a_year() {
    let (app, _, _, _) = setup_test_app().await;
    let follow = |site: Option<&'static str>, cookie: Option<&'static str>| {
        let mut builder = axum::http::Request::builder().uri("/u/testuser?lang=pl&a=1");
        if let Some(site) = site {
            builder = builder.header("sec-fetch-site", site);
        }
        if let Some(cookie) = cookie {
            builder = builder.header("cookie", cookie);
        }
        builder.body(Body::empty()).unwrap()
    };
    for (site, cookie, expected) in [
        (Some("same-origin"), None, Some(true)),
        (Some("same-site"), None, Some(true)),
        (None, None, Some(true)),
        (Some("cross-site"), None, Some(false)),
        (Some("none"), None, Some(false)),
        (Some("same-origin"), Some("calrs_lang=ru"), Some(true)),
        (Some("cross-site"), Some("calrs_lang=ru"), None),
        (Some("none"), Some("calrs_lang=ru"), None),
    ] {
        let response = app.clone().oneshot(follow(site, cookie)).await.unwrap();
        assert_eq!(response.status(), 303, "{site:?} {cookie:?}");
        assert_eq!(response.headers()["location"], "/u/testuser?a=1");
        let cookies = lang_cookies(&response);
        match expected {
            None => assert!(cookies.is_empty(), "{site:?} {cookie:?}: {cookies:?}"),
            Some(year) => {
                assert_eq!(cookies.len(), 1, "{site:?} {cookie:?}: {cookies:?}");
                assert!(
                    cookies[0].starts_with("calrs_lang=pl; Path=/"),
                    "{cookies:?}"
                );
                assert_eq!(
                    cookies[0].contains("Max-Age=31536000"),
                    year,
                    "{site:?} {cookie:?}: {cookies:?}"
                );
            }
        }
    }
}

// --- The menu's script ---

/// The menu closes on Escape and on a click or tap elsewhere, and its links
/// follow the URL a page changed in place (the slots page's month arrows use
/// `history.pushState`), without ever leaving this origin. The plain links
/// stay, for a browser without JavaScript.
#[test]
fn the_menu_script_closes_it_and_follows_the_current_url() {
    let body = render_login(context! { lang => "ru" });
    let start = body.find(SWITCHER).unwrap();
    let rest = &body[start..];
    let script = &rest[rest.find("<script>").expect("no menu script")..];
    let script = &script[..script.find("</script>").unwrap()];
    assert!(script.contains("document.querySelector('details.lang-switch')"));
    assert!(script.contains("e.key === 'Escape'"), "{script}");
    assert!(
        script.contains("addEventListener('pointerdown'"),
        "{script}"
    );
    assert!(script.contains("!menu.contains(e.target)"), "{script}");
    assert!(script.contains("menu.open = false"), "{script}");
    // Relinked on opening, from the URL shown now.
    assert!(script.contains("addEventListener('toggle'"), "{script}");
    assert!(
        script.contains("new URLSearchParams(location.search)"),
        "{script}"
    );
    assert!(script.contains("query.set('lang', a.getAttribute('data-lang'))"));
    assert!(
        script.contains(r"location.pathname.replace(/^[\/\\]+/, '')"),
        "leading slashes collapse: {script}"
    );
    // Every link carries its bundled code for the script, and keeps a
    // working href without it.
    for (code, _) in crate::i18n::supported_with_labels().filter(|(c, _)| *c != "de-formal") {
        assert!(
            rest.contains(&format!(
                r#"href="/auth/login?lang={code}" data-lang="{code}""#
            )),
            "{code}"
        );
    }
}

/// The slots page, whose month arrows push a new URL without a reload, has
/// the menu with its script.
#[tokio::test]
async fn the_slots_page_has_the_menu_script() {
    assert!(include_str!("../../../templates/slots.html").contains("history.pushState("));
    let (app, _, _, _) = setup_test_app().await;
    let body = page(
        &app,
        "/u/testuser/test-meeting?month=2026-12",
        None,
        Some("en"),
    )
    .await;
    let rest = &body[body.find(SWITCHER).expect("no switcher")..];
    assert!(
        rest.contains(
            r#"href="/u/testuser/test-meeting?month=2026-12&amp;lang=ru" data-lang="ru""#
        ),
        "{rest}"
    );
    let script = &rest[rest.find("<script>").unwrap()..];
    assert!(
        script[..script.find("</script>").unwrap()].contains("function pointLinks()"),
        "{script}"
    );
}

// --- Pages that get no menu ---

/// A cancellation sent from the guest's cancel page follows the language
/// the guest picked there, not the one the booking was made in.
#[tokio::test]
async fn a_guest_cancellation_follows_the_pick() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    for (pick, expected) in [(Some("ru"), "ru"), (None, "fr")] {
        let (app, pool, _, et_id) = setup_test_app().await;
        let (port, mails) = spawn_capturing_mta().await;
        use_capturing_mta(&pool, port).await;
        let cancel_tok = uuid::Uuid::new_v4().to_string();
        let resched_tok = uuid::Uuid::new_v4().to_string();
        let booking_id = insert_booking_at(
            &pool,
            &et_id,
            &cancel_tok,
            &resched_tok,
            chrono::Duration::hours(48),
        )
        .await;
        sqlx::query("UPDATE bookings SET language = 'fr' WHERE id = ?")
            .bind(&booking_id)
            .execute(&pool)
            .await
            .unwrap();
        let csrf = "csrf-switch-cancel";
        let cookie = match pick {
            Some(code) => format!("__Host-calrs_csrf={csrf}; calrs_lang={code}"),
            None => format!("__Host-calrs_csrf={csrf}"),
        };
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/booking/cancel/{cancel_tok}"))
                    .header("cookie", cookie)
                    .header("accept-language", "en-US,en;q=0.9")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(format!("_csrf={csrf}")))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            response.status().is_success() || response.status().is_redirection(),
            "{pick:?}: {}",
            response.status()
        );
        let status: String = sqlx::query_scalar("SELECT status FROM bookings WHERE id = ?")
            .bind(&booking_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "cancelled", "{pick:?}");
        let guest = mail_to(&mails, "guest@test.com").await;
        assert!(
            guest.contains(&format!("<html lang=\"{expected}\">")),
            "{pick:?}: {guest}"
        );
    }
}

/// A page under /dashboard that extends base.html itself (not
/// dashboard_base.html, which empties the menu's block) still shows no menu:
/// the dashboard has its own language setting, which the menu cannot change.
#[tokio::test]
async fn a_dashboard_page_on_the_public_layout_has_no_switcher() {
    let (app, pool, session, _) = setup_test_app().await;
    let account_id: String = sqlx::query_scalar(
        "SELECT a.id FROM accounts a JOIN users u ON u.id = a.user_id
         WHERE u.email = 'test@example.com'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    seed_synced_event(
        &pool,
        &account_id,
        "20260618T100000",
        "20260618T103000",
        "UTC",
        "OPAQUE",
    )
    .await;
    let source_id: String =
        sqlx::query_scalar("SELECT id FROM caldav_sources WHERE account_id = ? AND name = 'Src'")
            .bind(&account_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let cookie = format!("__Host-calrs_session={session}; calrs_lang=ru");
    let uri = format!("/dashboard/sources/{source_id}/setup-write");
    let body = page(&app, &uri, Some(cookie.as_str()), Some("en")).await;
    // Rendered from base.html, right where the menu would sit.
    assert!(include_str!("../../../templates/source_write_setup.html")
        .starts_with(r#"{% extends "base.html" %}"#));
    assert!(body.contains(r#"id="theme-toggle""#), "{body}");
    assert!(!body.contains(SWITCHER), "{body}");
}

/// What `middleware` itself does, apart from any template: an embedded
/// booking page and the dashboard give `lang_switcher()` no page, and
/// `?lang=` on an embedded page is no switch.
#[tokio::test]
async fn the_middleware_gives_no_page_to_embedded_and_dashboard_requests() {
    let app = axum::Router::new()
        .fallback(|| async {
            crate::web::lang_switch::current_page().unwrap_or_else(|| "none".to_string())
        })
        .layer(axum::middleware::from_fn(
            crate::web::lang_switch::middleware,
        ));
    let answer = |method: &'static str, uri: &'static str| {
        let app = app.clone();
        async move {
            let response = app
                .oneshot(
                    axum::http::Request::builder()
                        .method(method)
                        .uri(uri)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            (status, body_string(response).await)
        }
    };
    for (method, uri, expected) in [
        (
            "GET",
            "/u/testuser/test-meeting?a=1",
            "/u/testuser/test-meeting?a=1",
        ),
        ("GET", "/auth/login", "/auth/login"),
        ("GET", "/u/testuser/test-meeting?embed=1", "none"),
        ("GET", "/u/testuser/test-meeting?embed=1&lang=ru", "none"),
        ("GET", "/team/t/m?embed=true", "none"),
        ("GET", "/dashboard", "none"),
        ("GET", "/dashboard/bookings/x/reschedule", "none"),
        ("POST", "/u/testuser/test-meeting/book", "none"),
        ("POST", "/u/testuser/test-meeting/book?lang=ru", "none"),
    ] {
        let (status, body) = answer(method, uri).await;
        assert_eq!(status, 200, "{method} {uri}");
        assert_eq!(body, expected, "{method} {uri}");
    }
    // Not embedded, `?lang=` is a switch.
    let (status, _) = answer("GET", "/u/testuser/test-meeting?lang=ru").await;
    assert_eq!(status, 303);
}
