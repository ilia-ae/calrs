//! ilia fork: the dashboard shell. Monochrome sidebar icons that follow the
//! text and active colour, named icon-only controls, and the overview and
//! event-type pages on the shared button system. Kept apart from upstream's
//! test files, so rebasing the fork onto a new release does not conflict here.

use super::*;

async fn dashboard_page(uri: &str) -> String {
    let (app, _, session, _) = setup_test_app().await;
    let response = app.oneshot(get_authed(uri, &session)).await.unwrap();
    assert_eq!(response.status().as_u16(), 200, "{uri}");
    body_string(response).await
}

/// A colour emoji (the pictographic planes, U+1F000 and up).
fn has_colour_emoji(s: &str) -> bool {
    s.chars().any(|c| c as u32 >= 0x1F000)
}

#[tokio::test]
async fn sidebar_icons_are_monochrome_svgs() {
    let html = dashboard_page("/dashboard").await;
    let start = html.find(r#"<nav class="sidebar" id="dashboard-sidebar">"#);
    let start = start.expect("sidebar nav with its id");
    // The links only: the sign-out arrow below them is a glyph of its own.
    let start = start + html[start..].find(r#"<div class="sidebar-nav">"#).unwrap();
    let nav = &html[start
        ..start
            + html[start..]
                .find(r#"<div class="sidebar-bottom">"#)
                .unwrap()];

    let links = nav.matches(r#"class="sidebar-link"#).count();
    assert!(links >= 10, "{nav}");
    assert_eq!(nav.matches(r#"<svg class="icon""#).count(), links, "{nav}");
    assert!(!nav.contains(r#"<span class="icon">"#), "{nav}");
    assert!(!nav.contains("&#"), "{nav}");
    assert!(!has_colour_emoji(nav), "{nav}");
}

#[tokio::test]
async fn hamburger_has_a_name_and_an_expanded_state() {
    let html = dashboard_page("/dashboard").await;
    assert!(
        html.contains(
            r#"class="hamburger" aria-label="Menu" aria-controls="dashboard-sidebar" aria-expanded="false""#
        ),
        "{html}"
    );
}

#[tokio::test]
async fn timezone_banner_uses_the_shared_buttons() {
    let html = dashboard_page("/dashboard").await;
    assert!(
        html.contains(r#"<button id="tz-update" type="button" class="btn btn-sm btn-auto">"#),
        "{html}"
    );
    assert!(
        html.contains(
            r#"<button id="tz-dismiss" type="button" class="btn btn-ghost btn-sm btn-auto">"#
        ),
        "{html}"
    );
}

#[tokio::test]
async fn overview_has_no_emoji_and_names_its_disabled_actions() {
    let html = dashboard_page("/dashboard").await;
    let main = &html[html.find(r#"<main class="dashboard-main">"#).unwrap()..];

    // The stat tiles lost their watermark, the action cards their emoji.
    assert!(!main.contains("opacity: 0.04"), "{main}");
    assert_eq!(main.matches(r#"class="card stat-tile""#).count(), 4);
    assert!(!has_colour_emoji(main), "{main}");
    // The seeded user has no calendar source, so all four actions wait for one.
    assert_eq!(
        main.matches(r#" aria-disabled="true" tabindex="-1""#)
            .count(),
        4,
        "{main}"
    );
    assert!(
        main.contains(
            r#"<div id="avail-banner" class="card" style="border-left: 3px solid var(--accent);" hidden>"#
        ),
        "{main}"
    );
    assert!(
        !main.contains("border-top: 2px solid var(--accent)"),
        "{main}"
    );
    assert!(
        main.contains(r#"id="avail-banner-dismiss" class="btn btn-ghost btn-sm btn-auto" aria-label="Dismiss""#),
        "{main}"
    );
}

#[tokio::test]
async fn event_types_use_the_shared_buttons_and_name_the_menu() {
    let html = dashboard_page("/dashboard/event-types").await;
    assert!(!html.contains(r#"class="slot-btn"#), "{html}");
    assert!(
        html.contains(r#"<a href="/dashboard/event-types/new" class="btn btn-sm btn-auto">+ "#),
        "{html}"
    );
    assert!(
        html.contains(r#"class="btn btn-secondary btn-sm btn-auto copy-link-btn""#),
        "{html}"
    );
    assert!(
        html.contains(
            r#"class="btn btn-secondary btn-sm btn-auto action-menu-toggle" aria-label="More actions" aria-expanded="false""#
        ),
        "{html}"
    );
}
