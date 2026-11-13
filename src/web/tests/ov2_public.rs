//! ilia fork: tests for the second overnight review round, group `public`.

use super::*;

/// The template environment of the page tests: lenient, with i18n.
fn ov2_env() -> minijinja::Environment<'static> {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_loader(minijinja::path_loader("templates"));
    crate::i18n::register(&mut env);
    env
}

/// The team profile marks a non-public event type with the dashboard's
/// translated badge, not the raw database value.
#[test]
fn team_profile_badges_are_translated() {
    let env = ov2_env();
    let event_types = vec![
        context! { slug => "a", title => "A", duration_min => 30, visibility => "internal" },
        context! { slug => "b", title => "B", duration_min => 30, visibility => "private" },
        context! { slug => "c", title => "C", duration_min => 30, visibility => "public" },
    ];
    let html = env
        .get_template("team_profile.html")
        .unwrap()
        .render(context! {
            team_name => "Sales",
            team_slug => "sales",
            event_types => event_types,
            lang => "ru",
        })
        .unwrap();
    assert!(
        html.contains(
            r#"<span class="badge badge-info" style="margin-left: 0.5rem;">внутренний</span>"#
        ),
        "{html}"
    );
    assert!(
        html.contains(
            r#"<span class="badge badge-muted" style="margin-left: 0.5rem;">закрытый</span>"#
        ),
        "{html}"
    );
    assert_eq!(html.matches(r#"class="badge "#).count(), 2, "{html}");
    assert!(!html.contains(">internal<") && !html.contains(">private<"));
    assert!(!html.contains("text-transform: uppercase; letter-spacing: 0.04em"));
}

/// The empty state of the user and team profiles draws the stroke calendar
/// icon, not the colour emoji.
#[test]
fn profile_empty_states_draw_an_svg_icon() {
    let env = ov2_env();
    let empty: Vec<minijinja::Value> = Vec::new();
    for name in ["profile.html", "team_profile.html"] {
        let html = env
            .get_template(name)
            .unwrap()
            .render(context! {
                host_name => "Alice",
                username => "alice",
                team_name => "Sales",
                team_slug => "sales",
                event_types => empty.clone(),
                lang => "en",
            })
            .unwrap();
        assert!(
            html.contains(r#"<div class="empty-icon"><svg class="icon" viewBox="0 0 24 24""#),
            "{name}: {html}"
        );
        assert!(
            !html.contains("&#128197;") && !html.contains('\u{1F4C5}'),
            "{name}"
        );
    }
}

/// The slots page's timezone select takes the shared field outline token
/// (the stronger --border-hover in dark mode), with --border as fallback.
#[test]
fn slots_timezone_select_uses_the_field_border_token() {
    let src = std::fs::read_to_string("templates/slots.html").unwrap();
    let start = src.find(".slots-info-setting select {").unwrap();
    let rule = &src[start..start + src[start..].find('}').unwrap()];
    assert!(
        rule.contains("border: 1px solid var(--field-border, var(--border));"),
        "{rule}"
    );
}

/// The not-found card draws the shared muted result icon through the macro
/// and no longer overrides its colours on the page.
#[tokio::test]
async fn error_page_uses_the_shared_result_icon() {
    let (app, _, _, _) = setup_test_app().await;
    let req = axum::http::Request::builder()
        .uri("/u/nobody")
        .header("accept", "text/html,application/xhtml+xml,*/*;q=0.8")
        .header("accept-language", "en")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status().as_u16(), 404);
    let html = body_string(resp).await;
    assert!(
        html.contains(r#"<span class="result-icon muted" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5""#),
        "{html}"
    );
    assert!(!html.contains(".err-card .result-icon.muted"), "{html}");
    // No company link is set: no button, above all none to `/` (the login).
    assert!(!html.contains(r#"href="/""#), "{html}");
    assert!(!html.contains("Back to the site"), "{html}");
}
