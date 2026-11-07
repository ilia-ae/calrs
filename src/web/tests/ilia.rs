//! ilia fork branding: a calendar icon and "ilia.ae" in the dashboard sidebar,
//! no crab on the guest pages. Kept apart from upstream's test files, so
//! rebasing the fork onto a new release does not conflict here.

use super::*;

#[tokio::test]
async fn dashboard_sidebar_shows_a_calendar_and_the_ilia_name() {
    let (app, _, session, _) = setup_test_app().await;
    let response = app
        .oneshot(get_authed("/dashboard", &session))
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let html = body_string(response).await;
    assert!(
        html.contains(r#"<svg class="sidebar-brand-mark""#),
        "{html}"
    );
    assert!(
        html.contains(r#"<span class="sidebar-brand-name">ilia.ae</span>"#),
        "{html}"
    );
    assert!(!html.contains(r#"src="/brand-logo""#), "{html}");
}

#[tokio::test]
async fn guest_pages_have_no_crab() {
    let (app, _, _, _) = setup_test_app().await;
    let response = app.oneshot(get("/auth/login")).await.unwrap();
    let html = body_string(response).await;
    assert!(
        html.contains(r#"<a href="https://cal.rs">calrs</a>"#),
        "{html}"
    );
    assert!(!html.contains('🦀'), "{html}");
}
