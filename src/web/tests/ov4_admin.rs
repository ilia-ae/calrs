//! ilia fork: tests for the fourth overnight review round, group `admin`.

use super::*;
use minijinja::context;

fn env() -> minijinja::Environment<'static> {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_loader(minijinja::path_loader("templates"));
    crate::i18n::register(&mut env);
    env
}

fn render(name: &str, ctx: minijinja::Value) -> String {
    env()
        .get_template(name)
        .unwrap_or_else(|e| panic!("{name} loads: {e}"))
        .render(ctx)
        .unwrap_or_else(|e| panic!("{name} renders: {e}"))
}

/// The opening tags `<{tag} ...>` of a page, attributes included.
fn tags<'a>(html: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}");
    html.match_indices(&open)
        .filter_map(|(i, _)| html[i..].split_once('>').map(|(t, _)| t))
        .filter(|t| {
            t[open.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_whitespace())
        })
        .collect()
}

/// The admin page with every conditional field reached: a user, a group
/// with members, a resource, teams and the SMS gateway.
fn admin_page() -> String {
    let member = |id: &str| context! { user_id => id, name => "Ann", email => "ann@example.com", weight => 1 };
    render(
        "admin.html",
        context! {
            lang => "en", sidebar => context! {}, current_user_id => "u1", local_admin => true,
            users => vec![context! { id => "u2", name => "Bob", email => "bob@example.com", role => "user", enabled => true, auth_provider => "local" }],
            user_count => 1,
            groups => vec![context! { id => "g1", name => "Staff", members => vec![member("u1"), member("u2")] }],
            group_count => 1,
            custom_colors => context! { accent => "#111111", accent_hover => "#222222", bg => "#333333", surface => "#444444", text => "#555555" },
            current_theme => "custom",
            resources => vec![context! { id => "r1", name => "Room", feed_url => "https://x.example/", allowed_team_ids => "t1" }],
            resource_teams_all => vec![context! { id => "t1", name => "T" }],
            oidc_enabled => true, smtp_configured => true, sms_configured => true,
        },
    )
}

/// Audit 4 #6: the admin page's own fields (filters, custom colour hex
/// inputs, group weight) use the stronger field edge, and the filters' blur
/// handler puts that edge back, not the faint card border.
#[test]
fn admin_fields_use_the_field_border() {
    let page = admin_page();
    let mut seen = 0;
    for tag in ["input", "select", "textarea"] {
        for t in tags(&page, tag) {
            assert!(!t.contains("var(--border)"), "{t}");
            seen += usize::from(t.contains("var(--field-border)"));
        }
    }
    assert!(seen >= 8, "filters, five hex inputs and the weight: {seen}");
    for id in ["user-search", "group-search"] {
        let t = tags(&page, "input")
            .into_iter()
            .find(|t| t.contains(&format!(r#"id="{id}""#)))
            .unwrap_or_else(|| panic!("{id} renders"));
        assert!(
            t.contains(r#"onblur="this.style.borderColor='var(--field-border)'"#),
            "{t}"
        );
    }
}

/// Audit 4 #7: field hints on the admin page are the shared block `.hint`,
/// not inline 0.8rem spans that wrap with the parent's loose line height.
#[test]
fn admin_hints_are_block_hints() {
    let page = admin_page();
    for t in tags(&page, "span") {
        // A script shows the hidden warnings; every visible one is a block.
        if t.contains("font-size: 0.8rem") && !t.contains("display: none") {
            assert!(t.contains("display: block"), "{t}");
        }
    }
    assert!(
        page.matches(r#"<span class="hint">"#).count() >= 17,
        "{}",
        page.matches(r#"<span class="hint">"#).count()
    );
}

/// Every clickable checkbox label keeps its box on the first text line.
fn assert_checkbox_labels_top_aligned(page: &str, expected: usize) {
    let mut seen = 0;
    for (i, _) in page.match_indices("<label style=\"display: flex;") {
        let rest = &page[i..];
        let label = &rest[..rest.find("</label>").expect("label closes")];
        let open = label.split_once('>').unwrap().0;
        let Some(input) = tags(label, "input").into_iter().next() else {
            continue;
        };
        if !open.contains("cursor: pointer") || !input.contains(r#"type="checkbox""#) {
            continue;
        }
        seen += 1;
        assert!(open.contains("align-items: flex-start"), "{open}");
        assert!(
            input.contains("margin-top: 0.18rem; flex-shrink: 0;"),
            "{input}"
        );
    }
    assert_eq!(seen, expected, "checkbox labels found");
}

/// Audit 4 #8: checkbox labels on the admin page and Profile & Settings put
/// the box on the first line when the text wraps.
#[tokio::test]
async fn checkbox_labels_align_the_box_with_the_first_line() {
    assert_checkbox_labels_top_aligned(&admin_page(), 6);

    let (app, _, session, _) = setup_test_app().await;
    let resp = app
        .oneshot(get_authed("/dashboard/settings", &session))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_checkbox_labels_top_aligned(&body_string(resp).await, 2);
}

/// Audit 4 #9: the debug page's switch note leads into the link that names
/// the TON page, so the page name is not said twice.
#[test]
fn debug_switch_note_names_the_ton_page_once() {
    for lang in [
        "de",
        "de-formal",
        "en",
        "es",
        "et",
        "fr",
        "it",
        "nl",
        "pl",
        "pt",
        "ru",
    ] {
        let note = crate::i18n::translate(lang, "dbg-switch-note", None);
        let name = crate::i18n::translate(lang, "nav-admin-ton", None);
        assert!(!note.contains(&name), "{lang}: {note}");
        assert!(note.trim_end().ends_with(':'), "{lang}: {note}");
    }
    let page = render(
        "ilia/debug_booking.html",
        context! { lang => "ru", sidebar => context! {} },
    );
    assert!(
        page.contains(
            "переключателем вверху страницы: <a href=\"/dashboard/admin/ton#debug-block\">Оплата TON</a>"
        ),
        "{page}"
    );
}

/// Audit 4 #10: the TON status table keeps labels on the first line of a
/// value that wraps (a long raw address on a phone).
#[test]
fn ton_status_cells_align_to_the_top() {
    let page = render(
        "ton_settings.html",
        context! {
            lang => "ru", sidebar => context! {},
            debug => context! { telegram => context! {} },
            status => context! { on => true, network => "testnet", address => "EQ...", address_raw => "0:abc" },
        },
    );
    let rule = page
        .find("#ton-status td { vertical-align: top; }")
        .expect("status cells align to the top");
    assert!(rule < page.find(r#"id="ton-status""#).unwrap());
    assert!(page.contains("<code>0:abc</code>"), "{page}");
}
