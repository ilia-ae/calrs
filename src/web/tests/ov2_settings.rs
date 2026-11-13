//! ilia fork: tests for the second overnight review round, group `settings`.
//! Each test renders a touched template (minijinja reports syntax errors
//! only at render) and checks the markup the fix settled on. The shared
//! classes (`.success`, `.file-pick`, `.btn-telegram`, `.md-toolbar`) live
//! in base.html / dashboard_base.html; the pages only use them.

use super::*;

async fn page(app: &Router, uri: &str, session: &str) -> String {
    let resp = app.clone().oneshot(get_authed(uri, session)).await.unwrap();
    assert_eq!(resp.status(), 200, "{uri}");
    body_string(resp).await
}

/// The text minijinja escaped back to what `t()` gave (it escapes `/` too).
fn text(html: &str) -> String {
    html.replace("&#x2f;", "/")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Profile & Settings: the saved box is the shared `.success`, the warning
/// uses the theme's `--warn`, the picker has no page copy of its CSS and
/// script, the link button is a stroke icon, the add-interval column fits
/// a phone, and "Copy to all days" is an outlined button.
#[tokio::test]
async fn the_settings_page_uses_the_shared_pieces() {
    let (app, _, session, _) = setup_test_app().await;
    let csrf = "ov2-settings";
    let resp = app
        .clone()
        .oneshot(post_form(
            "/dashboard/settings",
            &session,
            csrf,
            &format!("_csrf={csrf}&name=Test+User&booking_email=&timezone=UTC"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = body_string(resp).await;
    let saved = crate::i18n::translate("en", "settings-saved", None);
    assert!(
        body.contains(&format!(
            "<div class=\"success\" role=\"status\">{saved}</div>"
        )),
        "{body}"
    );
    for gone in [
        "#e6f9e6",
        "#2d7a2d",
        "#b08800",
        "--warning,",
        "&#128279;",
        "theme-icon",
        "width:80px",
    ] {
        assert!(!body.contains(gone), "{gone}: {body}");
    }
    assert!(
        body.contains("color: var(--warn); display: block;"),
        "{body}"
    );
    // At most the shared copy (dashboard_base.html), none on the page.
    assert!(body.matches(".file-pick-input { position").count() <= 1);
    assert!(
        body.matches("querySelectorAll('.file-pick-input')").count() <= 1,
        "{body}"
    );
    // The toolbar: no inline button look, a stroke link icon.
    let toolbar = body
        .split_once("<div class=\"md-toolbar\" data-target=\"bio\"")
        .expect("bio toolbar")
        .1
        .split_once("</div>\n        <textarea")
        .expect("toolbar end")
        .0;
    assert!(!toolbar.contains("border-radius: 3px"), "{toolbar}");
    // The look (and the preview toggle's pressed accent) is dashboard_base's
    // `.md-toolbar`; only strike and code keep a glyph style inline.
    assert!(
        !toolbar.contains("background: var(--surface-hover)"),
        "{toolbar}"
    );
    assert!(!toolbar.contains("background: none"), "{toolbar}");
    assert!(!toolbar.contains("color: var(--text-muted)"), "{toolbar}");
    assert!(body.contains("<div class=\"md-toolbar\" data-target=\"bio\">"));
    assert!(
        toolbar.contains("data-action=\"link\" title=\"Insert link (Ctrl+K)\" aria-label=\"Insert link (Ctrl+K)\"><svg class=\"icon\" viewBox=\"0 0 24 24\" width=\"14\" height=\"14\""),
        "{toolbar}"
    );
    assert!(body.contains("label.style.cssText = 'width:2.5rem;"));
    assert!(body.contains("id=\"copy-to-all-btn\" class=\"btn btn-secondary btn-sm btn-auto\""));
}

/// Admin: small pickers like the profile's, the company link in the
/// shared field look, the SMS country a `.form-group` select, the 2FA and
/// TLS texts at the card sizes, warnings in `--warn`, and no page copy of
/// the picker's CSS and script.
#[tokio::test]
async fn the_admin_page_uses_the_shared_pieces() {
    let (app, _, session, _) = setup_test_app().await;
    let body = page(&app, "/dashboard/admin", &session).await;
    for field in ["logo", "favicon"] {
        assert!(
            body.contains(&format!(
                "<label class=\"btn btn-secondary btn-sm btn-auto file-pick\">Choose file<input type=\"file\" name=\"{field}\""
            )),
            "{field}"
        );
    }
    assert_eq!(
        body.matches("<button type=\"submit\" class=\"btn btn-sm btn-auto\">Upload</button>")
            .count(),
        2,
        "{body}"
    );
    assert!(body.contains(
        "<div class=\"form-group\" style=\"flex: 1; min-width: 12rem; margin: 0;\"><input type=\"url\" id=\"company_link\" name=\"company_link\" value=\"\" placeholder=\"https://yourcompany.com\"></div>"
    ), "{body}");
    assert!(body.matches(".file-pick-input { position").count() <= 1);
    assert!(
        body.contains(&format!(
            "<div class=\"form-group\">\n      <label>{} <span class=\"hint\">",
            crate::i18n::translate("en", "admin-sms-country", None)
        )),
        "{body}"
    );
    assert!(body.contains("<select name=\"default_country_code\">"));
    assert!(body.contains("<select name=\"provider\" id=\"sms-provider\">"));
    assert!(body.contains(&format!(
        "<p style=\"font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 0.75rem;\">{}</p>",
        crate::i18n::translate("en", "mfa-policy-help", None)
    )), "{body}");
    assert!(body.contains("<p class=\"hint\" style=\"font-size: 0.8rem; margin-top: 0.35rem;\">"));
    // base.html defines the token; the page itself uses `--warn` with no light-only fallback.
    assert!(!body.contains("var(--warning-text"), "{body}");
}

/// The SMS gateway labels and hints follow the page language: none of the
/// English sentences on a Russian page, and never a raw message id.
#[tokio::test]
async fn the_sms_gateway_hints_are_translated() {
    let (app, pool, session, _) = setup_test_app().await;
    let en = page(&app, "/dashboard/admin", &session).await;
    assert!(!en.contains(">admin-sms-"), "a raw message id: {en}");
    let english = crate::i18n::translate("en", "admin-sms-twilio-sender-hint", None);
    assert!(en.contains(&english), "{en}");
    assert!(en.contains(
        r#"calrs POSTs {&quot;to&quot;, &quot;text&quot;, &quot;sender&quot;} here and expects 2xx back."#
    ), "{en}");

    sqlx::query("UPDATE users SET language = 'ru' WHERE email = 'test@example.com'")
        .execute(&pool)
        .await
        .unwrap();
    let ru = text(&page(&app, "/dashboard/admin", &session).await);
    assert!(!ru.contains(">admin-sms-"), "a raw message id: {ru}");
    for spec in crate::sms::PROVIDER_SPECS {
        for key in [spec.sender_label_key, spec.sender_hint]
            .into_iter()
            .chain(spec.base_url_label_key)
            .chain(spec.base_url_label.map(|_| spec.base_url_hint))
        {
            let russian = crate::i18n::translate("ru", key, None);
            assert!(ru.contains(&russian), "{key}: {russian}");
        }
    }
    for english in [
        "A Twilio number in E.164 format",
        "Optional. Defaults to",
        "Up to 11 alphanumeric characters",
        "here and expects 2xx back",
        ">Sender <span",
        ">API base URL <span",
        ">Webhook URL <span",
    ] {
        assert!(!ru.contains(english), "{english}");
    }
}

/// Telegram settings: the shared small back link above the card, no page
/// copy of `.btn-telegram`.
#[tokio::test]
async fn the_telegram_settings_page_has_the_shared_back_link() {
    let (app, _, session, _) = setup_test_app().await;
    let body = page(&app, "/dashboard/settings/telegram", &session).await;
    let back = format!(
        "<a href=\"/dashboard/settings\" class=\"back-link\">&larr; {}</a>\n<div class=\"card\">",
        crate::i18n::translate("en", "settings-heading", None).replace('&', "&amp;")
    );
    assert!(body.contains(&back), "{body}");
    assert!(
        !body.contains("<p><a href=\"/dashboard/settings\">"),
        "{body}"
    );
    assert!(body.matches(".btn-telegram:hover").count() <= 1, "{body}");
}
