//! ilia fork: tests for the second overnight review round, group `auth`.

use super::*;

/// The template environment `build_state` sets up, without a database.
fn env() -> minijinja::Environment<'static> {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_loader(minijinja::path_loader("templates"));
    crate::i18n::register(&mut env);
    crate::web::ilia::register(&mut env, &crate::web::ilia::Footer::from_env());
    crate::brand::register(&mut env, crate::brand::configured());
    env
}

fn mfa(values: minijinja::Value) -> String {
    env()
        .get_template("auth/mfa.html")
        .expect("auth/mfa.html should load")
        .render(minijinja::context! { lang => "en", csrf_token => "t", ..values })
        .expect("auth/mfa.html should render")
}

/// Links in running text keep base.html's classless link rule (accent and
/// underline): the MFA style no longer overrides it.
#[test]
fn mfa_style_leaves_links_to_the_base_rule() {
    let html = mfa(minijinja::context! { mode => "recovery", codes => vec!["aaaa-bbbb"] });
    assert!(!html.contains(".ilia-mfa a:not(.btn)"), "no link override");
    // A button right after a field sits on the field's margin, as on login.
    assert!(html.contains(".ilia-mfa .form-group ~ .btn { margin-top: 0; }"));
}

/// The way on from the recovery codes is a primary button, and the copy
/// result goes into an always-present, initially empty live region.
#[test]
fn recovery_codes_continue_button_and_live_status() {
    let html =
        mfa(minijinja::context! { mode => "recovery", codes => vec!["aaaa-bbbb", "cccc-dddd"] });
    assert!(html.contains(r#"<a class="btn" href="/dashboard">Continue"#));
    assert!(!html.contains(r#"<p class="mfa-back"><a href="/dashboard">"#));
    assert!(html.contains(
        r#"<p class="passkey-meta" id="recovery-copy-manual" role="status" data-manual=""#
    ));
    assert!(html.contains(r#""></p>"#));
    assert!(!html.contains(r#"role="status" hidden"#), "never hidden");
    assert!(html.contains("manual.textContent = button.dataset.copied;"));
    assert!(html.contains("manual.textContent = manual.dataset.manual;"));
    assert!(html.contains(r#"manual.textContent = "";"#));
}

/// The code views (sign-in and enrolment, with or without a passkey on
/// offer) space the restart link like the passkey view does.
#[test]
fn code_views_space_the_restart_link() {
    for values in [
        minijinja::context! { mode => "login", method => "totp" },
        minijinja::context! { mode => "login", method => "totp", passkey_available => true, has_totp => true },
        minijinja::context! { mode => "login", method => "recovery" },
        minijinja::context! { mode => "enroll", method => "totp", qr => "", secret => "S" },
    ] {
        let html = mfa(values);
        assert!(
            html.contains(r#"<p class="mfa-back"><a href="/auth/login">"#),
            "restart link spaced: {html}"
        );
        assert!(html.contains(".ilia-mfa .mfa-back { margin: 1.25rem 0 0; }"));
        assert!(!html.contains(r#"<p><a href="/auth/login">"#));
    }
}

/// The footer links of login and registration carry no inline colour, so
/// the base rule underlines them.
#[tokio::test]
async fn login_and_register_links_are_underlined_by_the_base_rule() {
    let (app, _, _, _) = setup_test_app().await;
    let login = body_string(app.clone().oneshot(get("/auth/login")).await.unwrap()).await;
    assert!(login.contains(r#"<a href="/auth/register">"#), "{login}");
    let register = body_string(app.oneshot(get("/auth/register")).await.unwrap()).await;
    assert!(register.contains(r#"<a href="/auth/login">"#), "{register}");
    for page in [login, register] {
        assert!(!page.contains("style=\"color: var(--accent); text-decoration: none;\""));
    }
}
