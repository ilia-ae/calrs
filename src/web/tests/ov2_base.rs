//! ilia fork: tests for the second overnight review round, group `base`:
//! the shared tokens, notices, file picker, button states, focus rings,
//! result discs, motion and the dashboard shell in base.html and
//! dashboard_base.html.

use super::*;

const BASE: &str = include_str!("../../../templates/base.html");
const DASHBOARD_BASE: &str = include_str!("../../../templates/dashboard_base.html");

/// The main `<style>` block of base.html.
fn base_css() -> &'static str {
    let start = BASE.find("<style>").unwrap() + "<style>".len();
    &BASE[start..start + BASE[start..].find("</style>").unwrap()]
}

/// The declarations of the first rule of `css` whose selector list is
/// exactly `selector` (alone on its line, or after a `}`).
fn rule<'a>(css: &'a str, selector: &str) -> &'a str {
    let mut from = 0;
    while let Some(at) = css[from..].find(selector) {
        let at = from + at;
        let after = css[at + selector.len()..].trim_start();
        let line_start = css[..at].rfind(['}', '\n']).map_or(0, |i| i + 1);
        if after.starts_with('{') && css[line_start..at].trim().is_empty() {
            let open = at + selector.len() + css[at + selector.len()..].find('{').unwrap();
            let close = open + css[open..].find('}').unwrap();
            return &css[open + 1..close];
        }
        from = at + selector.len();
    }
    panic!("no rule `{selector}`")
}

fn base_rule(selector: &str) -> &'static str {
    rule(base_css(), selector)
}

#[test]
fn status_tokens_are_defined_in_both_themes() {
    let light = base_rule(":root");
    let dark = base_rule("html.dark");
    for token in [
        "--success:",
        "--success-bg:",
        "--success-border:",
        "--success-text:",
        "--warning:",
        "--warning-bg:",
        "--warning-text:",
        "--error-text:",
        "--warn:",
        "--warn-bg:",
        "--warn-border:",
        "--field-border:",
    ] {
        assert!(light.contains(token), "{token} in :root");
    }
    // The aliases follow --warn, which html.dark redefines; the others have
    // a dark value of their own.
    for token in [
        "--success:",
        "--success-bg:",
        "--success-border:",
        "--success-text:",
        "--error-text:",
        "--warn:",
        "--warn-bg:",
        "--warn-border:",
        "--field-border:",
    ] {
        assert!(dark.contains(token), "{token} in html.dark");
    }
    assert!(light.contains("--warning: var(--warn);"));
    assert!(light.contains("--warning-text: var(--warn);"));
    assert!(light.contains("--warning-bg: var(--warn-bg);"));
    assert!(dark.contains("--field-border: var(--border-hover);"));
}

#[test]
fn success_notice_is_a_shared_rule_on_the_tokens() {
    let ok = base_rule(".success");
    for decl in [
        "color: var(--success-text);",
        "background: var(--success-bg);",
        "border: 1px solid var(--success-border);",
    ] {
        assert!(ok.contains(decl), "{decl} in {ok}");
    }
    // No literal light-only fallback inside the var().
    assert!(!ok.contains("var(--success-bg,"), "{ok}");
}

#[test]
fn text_fields_use_the_field_border_and_a_visible_focus_ring() {
    let fields = base_rule(".form-group input, .form-group textarea, .form-group > select");
    assert!(
        fields.contains("border: 1px solid var(--field-border);"),
        "{fields}"
    );
    let focus = base_rule(
        ".form-group input:focus, .form-group textarea:focus, .form-group > select:focus",
    );
    assert!(
        focus.contains("box-shadow: 0 0 0 3px var(--accent-border);"),
        "{focus}"
    );
    assert!(base_rule(".iti").contains("--iti-border-color: var(--field-border);"));
}

#[test]
fn checkboxes_and_radios_get_their_keyboard_outline_back() {
    let css = base_css();
    let sel = ".form-group input[type=checkbox]:focus-visible, .form-group input[type=radio]:focus-visible";
    assert!(base_rule(sel).contains("outline: 2px solid var(--accent);"));
    // After the rule that removes the text field's ring from them.
    let reset = css
        .find(".form-group input[type=checkbox], .form-group input[type=radio] {")
        .unwrap();
    assert!(css.find(sel).unwrap() > reset);
}

#[test]
fn disabled_buttons_keep_their_variant_and_ignore_hover() {
    let css = base_css();
    // The old rule painted every disabled variant with the primary's muted fill.
    assert!(!css.contains(".btn:disabled { background"), "{css}");
    let gate = ":where(:not(:disabled):not([aria-disabled=\"true\"]))";
    for hover in [
        ".btn:hover",
        ".btn-secondary:hover",
        ".btn-ghost:hover",
        ".btn-danger:hover",
        ".btn-telegram:hover",
    ] {
        let gated = format!("{hover}{gate}");
        assert!(css.contains(&format!("{gated} {{")), "{gated}");
        assert!(!css.contains(&format!("{hover} {{")), "ungated {hover}");
        assert!(!css.contains(&format!("{hover},")), "ungated {hover}");
    }
    let disabled = base_rule(".btn:disabled, .btn[aria-disabled=\"true\"]");
    assert!(disabled.contains("opacity: 0.5;") && disabled.contains("cursor: not-allowed;"));
    assert!(
        base_rule(".btn:disabled:active, .btn[aria-disabled=\"true\"]:active")
            .contains("transform: none;")
    );
    // The fade comes after every variant, the danger fill stays red.
    assert!(
        css.find(".btn:disabled, .btn[aria-disabled").unwrap() > css.find(".btn-danger {").unwrap()
    );
    assert!(base_rule(".btn-danger").contains("background: #dc2626;"));
}

#[test]
fn telegram_button_and_file_picker_are_shared() {
    let tg = base_rule(".btn-telegram");
    assert!(tg.contains("background: #229ED9;") && tg.contains("color: #fff;"));
    assert!(base_rule(".file-pick").contains("position: relative;"));
    assert!(base_rule(".file-pick-input").contains("opacity: 0;"));
    assert!(base_rule(".file-pick:focus-within").contains("outline: 2px solid var(--accent);"));
    assert!(base_rule(".file-pick-name").contains("color: var(--text-muted);"));
    assert!(DASHBOARD_BASE.contains("document.querySelectorAll('.file-pick-input')"));
}

#[test]
fn result_discs_are_readable_in_both_themes() {
    let muted = base_rule(".result-icon.muted");
    assert!(
        muted.contains("background: var(--text-secondary);"),
        "{muted}"
    );
    assert!(muted.contains("color: var(--surface);"), "{muted}");
    // Every other disc keeps a white glyph on a fill dark enough for it.
    assert!(base_rule(".result-icon").contains("color: #fff;"));
    assert!(base_rule("html.dark .result-icon.ok").contains("background: #16a34a;"));
    assert!(base_rule("html.dark .result-icon.bad").contains("background: #dc2626;"));
    assert!(base_rule(".empty-icon svg").contains("width: 2.5rem;"));
}

#[test]
fn theme_pill_uses_the_neutral_tokens() {
    let start = BASE.find(".theme-pill {").unwrap();
    let pill = &BASE[start..start + BASE[start..].find("</style>").unwrap()];
    for slate in ["#374151", "#4b5563", "#94a3b8"] {
        assert!(!pill.contains(slate), "{slate}");
    }
    assert!(pill.contains("html.dark .theme-pill-track { background: var(--border-hover); }"));
    assert!(pill.contains("prefers-reduced-motion: reduce"));
}

#[test]
fn reduced_motion_stops_the_lifts_and_slides() {
    let css = base_css();
    let lift = css.find("transform: translateY(-2px);").unwrap();
    let reduce = lift
        + css[lift..]
            .find("@media (prefers-reduced-motion: reduce)")
            .unwrap();
    assert!(css[reduce..]
        .starts_with("@media (prefers-reduced-motion: reduce) {\n      a.card, .card.card-link"));
    // The check pop only runs when motion is welcome.
    let check = css.find(".check { animation: checkPop").unwrap();
    assert!(css[..check]
        .trim_end()
        .ends_with("@media (prefers-reduced-motion: no-preference) {"));
    assert!(DASHBOARD_BASE.contains(".sidebar, .sidebar.open { transition: none; }"));
}

#[test]
fn dashboard_shell_rules() {
    // A closed phone sidebar leaves the Tab order.
    let start = DASHBOARD_BASE.find("@media (max-width: 768px)").unwrap();
    let phone = &DASHBOARD_BASE[start..];
    assert!(rule(phone, ".sidebar").contains("visibility: hidden;"));
    assert!(rule(phone, ".sidebar.open").contains("visibility: visible;"));
    // Stacked two-column grids do not add their gap to the fields' margin.
    let start = DASHBOARD_BASE.find("@media (max-width: 480px)").unwrap();
    assert!(rule(&DASHBOARD_BASE[start..], ".form-grid-2").contains("row-gap: 0;"));
    // The markdown toolbar is styled by class, not by script.
    assert!(rule(DASHBOARD_BASE, ".md-toolbar").contains("background: var(--surface-hover);"));
    assert!(rule(DASHBOARD_BASE, ".md-toolbar button").contains("border-radius: 3px;"));
    assert!(!DASHBOARD_BASE.contains("mouseenter"));
}

#[tokio::test]
async fn shared_styles_reach_public_and_dashboard_pages() {
    let (app, _, session, _) = setup_test_app().await;
    let response = app.clone().oneshot(get("/auth/login")).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let public = body_string(response).await;
    for needle in [
        ".success {",
        ".empty-icon svg {",
        ".btn-telegram {",
        ".file-pick {",
        "--success-text:",
        "--field-border:",
    ] {
        assert!(public.contains(needle), "{needle} on a public page");
    }
    // The dashboard-only parts stay off the public pages.
    assert!(!public.contains(".md-toolbar {"));

    for uri in ["/dashboard", "/dashboard/settings"] {
        let response = app
            .clone()
            .oneshot(get_authed(uri, &session))
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 200, "{uri}");
        let html = body_string(response).await;
        assert!(html.contains(".md-toolbar {"), "{uri}");
        assert!(
            html.contains("querySelectorAll('.file-pick-input')"),
            "{uri}"
        );
        assert!(html.contains(".success {"), "{uri}");
    }
}
