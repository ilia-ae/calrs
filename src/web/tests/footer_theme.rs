//! ilia fork: the theme button in the footer, next to the language menu
//! (`.footer-controls` in templates/base.html), in place of the pill that
//! was fixed in the top-right corner of every public page.

use super::ov2_base::rule;
use super::*;

const BASE: &str = include_str!("../../../templates/base.html");
const ROW: &str = r#"<div class="footer-controls">"#;
const SWITCHER: &str = r#"<details class="lang-switch">"#;
const BUTTON: &str = r#"<button id="theme-toggle" type="button" class="theme-btn""#;

async fn fetch(app: &Router, uri: &str, cookie: Option<&str>) -> (u16, String) {
    let mut builder = axum::http::Request::builder()
        .uri(uri)
        .header("accept-language", "en");
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status().as_u16();
    (status, body_string(response).await)
}

/// The footer row of a page, up to its closing tag (neither the language
/// menu nor the button holds a `<div>`).
fn row(html: &str) -> &str {
    let start = html
        .find(ROW)
        .unwrap_or_else(|| panic!("no footer row: {html}"));
    let end = start + html[start..].find("</div>").unwrap();
    &html[start..end]
}

/// Neither the old pill nor its styles are left.
fn assert_no_pill(html: &str) {
    for gone in ["theme-pill", "theme-pill-track", "theme-pill-knob"] {
        assert!(!html.contains(gone), "{gone} left: {html}");
    }
}

#[tokio::test]
async fn public_pages_have_the_language_menu_then_the_theme_button() {
    let (app, _, _, _) = setup_test_app().await;
    for uri in ["/u/testuser", "/u/testuser/test-meeting", "/auth/login"] {
        let (status, html) = fetch(&app, uri, None).await;
        assert_eq!(status, 200, "{uri}");
        let row = row(&html);
        let menu = row.find(SWITCHER).unwrap_or_else(|| panic!("{uri}: {row}"));
        let button = row.find(BUTTON).unwrap_or_else(|| panic!("{uri}: {row}"));
        assert!(menu < button, "{uri}: the menu comes first: {row}");
        // Under the made-by/source line.
        assert!(html.find(r#"<div class="powered">"#).unwrap() < html.find(ROW).unwrap());
        assert_eq!(html.matches(r#"id="theme-toggle""#).count(), 1, "{uri}");
        assert_no_pill(&html);
    }
}

#[tokio::test]
async fn the_button_is_named_by_the_toggle_key_in_the_page_language() {
    let (app, _, _, _) = setup_test_app().await;
    let (_, html) = fetch(&app, "/u/testuser", Some("calrs_lang=ru")).await;
    let row = row(&html);
    let start = row.find(BUTTON).unwrap_or_else(|| panic!("{row}"));
    let button = &row[start..start + row[start..].find("</button>").unwrap()];
    let label = crate::i18n::translate("ru", "base-theme-toggle", None);
    assert_eq!(label, "Сменить тему");
    assert!(button.contains(&format!(r#"title="{label}""#)), "{button}");
    assert!(
        button.contains(&format!(r#"aria-label="{label}""#)),
        "{button}"
    );
    // Both icons are decoration; the label names the button.
    assert_eq!(button.matches("<svg").count(), 2, "{button}");
    assert_eq!(
        button.matches(r#"aria-hidden="true""#).count(),
        2,
        "{button}"
    );
}

/// The error page renders outside the language menu's page scope: the row
/// holds the theme button alone.
#[tokio::test]
async fn the_error_page_keeps_the_theme_button_without_the_menu() {
    let (app, _, _, _) = setup_test_app().await;
    let (status, html) = fetch(&app, "/u/nobody", None).await;
    assert_eq!(status, 404);
    let row = row(&html);
    assert!(!row.contains(SWITCHER), "{row}");
    assert!(row.contains(BUTTON), "{row}");
    assert_no_pill(&html);
}

/// A page answering a form POST has no menu, and still the theme button.
#[tokio::test]
async fn a_post_answer_keeps_the_theme_button_without_the_menu() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, _, _, _) = setup_test_app().await;
    let csrf = "csrf-footer-theme";
    let response = app
        .oneshot(post_form_unauthed(
            "/u/testuser/test-meeting/book",
            csrf,
            &book_body(csrf),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let html = body_string(response).await;
    let row = row(&html);
    assert!(!row.contains(SWITCHER), "{row}");
    assert!(row.contains(BUTTON), "{row}");
    assert_no_pill(&html);
}

/// Embedded, the parent sets the theme (`?theme=`) and the page has neither
/// the menu nor the button; the early script still applies the parent's theme.
#[tokio::test]
async fn embedded_pages_have_neither_control() {
    let (app, _, _, _) = setup_test_app().await;
    for (uri, status) in [
        ("/u/testuser/test-meeting?embed=1&theme=dark", 200),
        ("/u/nobody?embed=1&theme=dark", 404),
    ] {
        let (got, html) = fetch(&app, uri, None).await;
        assert_eq!(got, status, "{uri}");
        assert!(!html.contains(ROW), "{uri}");
        assert!(!html.contains(SWITCHER), "{uri}");
        assert!(!html.contains(r#"id="theme-toggle""#), "{uri}");
        assert!(html.contains(r#"var embedTheme = "dark";"#), "{uri}");
        assert_no_pill(&html);
    }
}

/// The dashboard empties both blocks and keeps its own theme setting: no
/// footer button, and the empty row is hidden.
#[tokio::test]
async fn the_dashboard_has_no_footer_theme_button() {
    let (app, _, session, _) = setup_test_app().await;
    let cookie = format!("__Host-calrs_session={session}");
    let (status, html) = fetch(&app, "/dashboard", Some(cookie.as_str())).await;
    assert_eq!(status, 200);
    assert!(html.contains(&format!("{ROW}</div>")), "{html}");
    assert!(!html.contains(r#"id="theme-toggle""#), "{html}");
    assert!(!html.contains(SWITCHER), "{html}");
    assert!(BASE.contains(".footer-controls:empty { display: none; }"));
    assert_no_pill(&html);
}

/// The button's size and the sun/moon switch; tokens only and nothing fixed
/// to the window. The colour, hover and focus tokens are pinned in
/// `ov2_base::theme_button_uses_the_neutral_tokens`.
#[test]
fn the_button_styles() {
    let start = BASE.find(".theme-btn {").unwrap();
    let css = &BASE[start..start + BASE[start..].find("</style>").unwrap()];
    assert!(!css.contains('#'), "literal colours: {css}");
    assert!(!css.contains("position: fixed"), "{css}");
    let button = rule(css, ".theme-btn");
    for decl in ["width: 1.75rem;", "height: 1.75rem;", "border-radius: 8px;"] {
        assert!(button.contains(decl), "{decl}: {button}");
    }
    let icon = rule(css, ".theme-btn svg");
    assert!(
        icon.contains("width: 14px;") && icon.contains("height: 14px;"),
        "{icon}"
    );
    let hidden = rule(css, ".theme-btn-moon, html.dark .theme-btn-sun");
    assert!(hidden.contains("display: none;"), "{hidden}");
    assert!(rule(css, "html.dark .theme-btn-moon").contains("display: block;"));
    let row = rule(BASE, ".footer-controls");
    assert!(
        row.contains("display: flex;") && row.contains("flex-wrap: wrap;"),
        "{row}"
    );
}

/// The behaviour stays: the no-flash head script and the toggle read the same
/// `localStorage` key, and the toggle is wired only outside embeds.
#[test]
fn the_toggle_behaviour_is_unchanged() {
    let head = &BASE[..BASE.find("</head>").unwrap()];
    assert!(head.contains("localStorage.getItem('calrs_theme')"));
    assert!(BASE.contains("var btn = document.getElementById('theme-toggle');"));
    assert!(BASE.contains("if (btn && !embed) {"));
    assert!(BASE.contains("localStorage.setItem('calrs_theme', next);"));
}
