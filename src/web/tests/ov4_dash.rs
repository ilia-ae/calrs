//! ilia fork: tests for the fourth overnight review round, group `dash`.

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

/// Only the page's own part (its `dashboard_content` block). Renders the
/// whole page first, so a syntax error anywhere still fails.
fn content(name: &str, ctx: minijinja::Value) -> String {
    render(name, ctx.clone());
    let env = env();
    let tmpl = env.get_template(name).unwrap();
    let mut state = tmpl.eval_to_state(ctx).unwrap();
    state.render_block("dashboard_content").unwrap()
}

fn event_type_form(lang: &str) -> String {
    render(
        "event_type_form.html",
        context! {
            lang => lang, sidebar => context! {}, editing => true, form_slug => "call",
            form_first_slot_only => true, form_frequency_limits => "2:week,5:month:m",
            is_group => true, form_pending => context! { blocks_slot => true },
            teams => vec![context! { id => "t1", name => "T" }],
        },
    )
}

/// Findings 11 and 12: the rows the script builds are on the shared buttons
/// and in the page's language.
#[test]
fn frequency_limit_rows_use_the_shared_button_and_translated_labels() {
    let html = event_type_form("ru");
    // The delete control: the ghost button with ×, named, no emoji.
    assert!(!html.contains("1F5D1"), "the bin emoji is gone");
    assert!(!html.contains('\u{1F5D1}'));
    assert!(html.contains("del.className = 'btn btn-ghost btn-sm btn-auto';"));
    assert!(html.contains("del.setAttribute('aria-label', del.title);"));
    assert!(html.contains("del.innerHTML = '&times;';"));
    assert!(!html.contains("opacity: 0.6"));
    // Labels built by the script, in Russian.
    assert!(!html.contains("Per team member"));
    assert!(html.contains(r#"document.createTextNode(' ' + "Для каждого участника")"#));
    assert!(!html.contains("Member's timezone"));
    assert!(!html.contains("level.charAt(0).toUpperCase()"));
    assert!(html.contains(r#"high: "Высокий""#), "priority labels");
    assert!(html.contains(r#"medium: "Средний""#));
    assert!(html.contains(r#"low: "Низкий""#));
    assert!(html.contains("btn.textContent = priorityLabels[level];"));
    // English for an English page.
    let en = event_type_form("en");
    assert!(en.contains(r#"document.createTextNode(' ' + "Per team member")"#));
}

/// Finding 13: both booking-limit booleans are the shared checkbox card,
/// named by their label, with no hand-built switch or light-only colours.
#[test]
fn booking_limit_toggles_are_named_checkbox_cards() {
    let html = event_type_form("en");
    for (id, help) in [
        ("first-slot-toggle", "first-slot-help"),
        ("freq-limit-toggle", "freq-limit-help"),
    ] {
        let at = html
            .find(&format!(r#"id="{id}""#))
            .unwrap_or_else(|| panic!("{id}"));
        let open = html[..at].rfind("<label").expect("inside a label");
        let label = &html[open..];
        let label = &label[..label.find("</label>").unwrap()];
        assert!(label.contains(r#"type="checkbox""#), "{label}");
        assert!(label.contains("accent-color: var(--accent)"), "{label}");
        assert!(!label.contains("opacity: 0"), "{label}");
        assert!(
            label.contains(&format!(r#"aria-describedby="{help}""#)),
            "{label}"
        );
        assert!(label.contains(&format!(r#"id="{help}""#)), "{label}");
        assert!(
            label.contains("<strong"),
            "the title names the box: {label}"
        );
    }
    assert!(html.contains(r#"name="first_slot_only" id="first-slot-toggle" checked"#));
    for gone in [
        "#71717a",
        "first-slot-track",
        "freq-toggle-track",
        "knob",
        "updateToggleStyle",
    ] {
        assert!(!html.contains(gone), "{gone}");
    }
}

/// Finding 15: each bookable-range label stays with its date field.
#[test]
fn bookable_range_labels_wrap_with_their_fields() {
    let html = event_type_form("ru");
    let pairs: Vec<&str> = html
        .split(r#"<span class="bookable-pair""#)
        .skip(1)
        .collect();
    assert_eq!(pairs.len(), 2);
    let first = &pairs[0][..pairs[0]
        .find(r#"id="bookable_until""#)
        .unwrap_or(pairs[0].len())];
    assert!(first.contains(r#"id="bookable_from""#), "{first}");
    assert!(first.contains("display: inline-flex"));
    assert!(pairs[1].contains(r#"id="bookable_until""#));
    assert!(pairs[1].contains("data-order-error="));
}

/// Finding 14: overrides and embed put the back link above the card.
#[test]
fn overrides_and_embed_put_the_back_link_above_the_card() {
    let overrides = |is_team: bool| {
        content(
            "overrides.html",
            context! {
                lang => "en", sidebar => context! {}, event_type_slug => "x",
                event_type_title => "Call", is_team => is_team,
            },
        )
    };
    let personal = overrides(false);
    assert!(personal.contains(
        "<a href=\"/dashboard/event-types\" class=\"back-link\">&larr; Back to event types</a>\n<div class=\"card\">\n  <h1 style=\"margin-bottom: 0.5rem;\">"
    ), "{personal}");
    let team = overrides(true);
    assert!(
        team.contains("<a href=\"/dashboard/teams\" class=\"back-link\">&larr; ")
            && team.contains("</a>\n<div class=\"card\">"),
        "{team}"
    );
    let embed = content(
        "embed.html",
        context! {
            lang => "en", sidebar => context! {}, event_type_title => "Call",
            event_type_slug => "x", visibility => "public",
        },
    );
    assert!(embed.contains(
        "<a href=\"/dashboard/event-types\" class=\"back-link\">&larr; Back to event types</a>\n<div class=\"card\">\n  <h1 style=\"margin-bottom: 0.5rem;\">"
    ), "{embed}");
    for page in [&personal, &team, &embed] {
        assert!(!page.contains("back-link\" style="), "{page}");
    }
}

/// Finding 17: the tab title separates the page name from the team name.
#[test]
fn team_settings_title_separates_the_team_name() {
    let html = render(
        "team_settings.html",
        context! {
            lang => "ru", sidebar => context! {}, team_id => "t1", team_name => "Команда",
            members => vec![context! { id => "u1", name => "Ann", initials => "A", role => "member", source => "direct" }],
        },
    );
    let title = &html[html.find("<title>").unwrap()..html.find("</title>").unwrap()];
    assert!(title.contains("Настройки — Команда — "), "{title}");
}
