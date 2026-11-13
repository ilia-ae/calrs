//! ilia fork: tests for the third overnight review round, group `dash`.

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

fn source(id: &str) -> minijinja::Value {
    context! {
        id => id, name => "Work", url => "https://dav.example.com/", username => "ann",
        enabled => true, last_synced => "never", auth_type => "basic",
        calendars => vec![context! { id => "c1", name => "Cal", sync_enabled => true }],
        synced_count => 1,
    }
}

/// Every page with form fields, rendered with enough data to reach its fields.
fn field_pages() -> Vec<(&'static str, String)> {
    let et = context! {
        lang => "en", sidebar => context! {}, form_requires_confirmation => true, form_pending => context! { blocks_slot => true },
        teams => vec![context! { id => "t1", name => "T" }],
        ton_price => context! { enabled => true, amount => "1", window => 30, default_window => 30 },
    };
    vec![
        ("event_type_form.html", render("event_type_form.html", et)),
        (
            "source_form.html",
            render(
                "source_form.html",
                context! { lang => "en", sidebar => context! {} },
            ),
        ),
        (
            "invite_form.html",
            render(
                "invite_form.html",
                context! { lang => "en", sidebar => context! {}, event_type_id => "et1" },
            ),
        ),
        (
            "overrides.html",
            render(
                "overrides.html",
                context! { lang => "en", sidebar => context! {}, event_type_slug => "x" },
            ),
        ),
        (
            "team_form.html",
            render(
                "team_form.html",
                context! { lang => "en", sidebar => context! {} },
            ),
        ),
        (
            "team_settings.html",
            render(
                "team_settings.html",
                context! {
                    lang => "en", sidebar => context! {}, team_id => "t1",
                    members => vec![context! { id => "u1", name => "Ann", initials => "A", role => "member", source => "direct" }],
                },
            ),
        ),
        (
            "troubleshoot.html",
            render(
                "troubleshoot.html",
                context! { lang => "en", sidebar => context! {}, et_title => "Call" },
            ),
        ),
        (
            "settings.html",
            render(
                "settings.html",
                context! { lang => "en", sidebar => context! {} },
            ),
        ),
        (
            "dashboard_bookings.html",
            render(
                "dashboard_bookings.html",
                context! {
                    lang => "en", sidebar => context! {},
                    upcoming => vec![context! { id => "b1", guest_name => "G", status => "confirmed" }],
                },
            ),
        ),
    ]
}

/// Dark mode: every field edge is the stronger `--field-border`, not the
/// card's `--border`, including the fields the scripts build.
#[test]
fn form_fields_use_the_field_border() {
    for (name, page) in field_pages() {
        let mut seen = 0;
        for tag in ["input", "select", "textarea"] {
            for t in tags(&page, tag) {
                seen += 1;
                assert!(!t.contains("var(--border)"), "{name}: {t}");
            }
        }
        assert!(seen > 0, "{name} rendered no fields");
    }
    let et = &field_pages()[0].1;
    assert!(et.contains("border:1px solid var(--field-border);border-radius:var(--radius);background:var(--surface);color:var(--text);font-family:inherit;font-size:0.85rem;min-width:90px;"));
    assert!(et.contains("input.style.cssText = 'width: 70px; padding: 0.5rem 0.625rem; border: 1px solid var(--field-border);"));
    assert!(et.contains(
        "select.style.cssText = 'padding: 0.5rem 0.75rem; border: 1px solid var(--field-border);"
    ));
    assert!(
        et.contains("id=\"ton_price_mode\""),
        "TON partial not rendered"
    );
    assert!(et.contains("id=\"week_start\""), "week start not rendered");
    // The rich-text editor frames match the fields.
    for name in ["event_type_form.html", "team_form.html", "settings.html"] {
        let page = &field_pages().into_iter().find(|p| p.0 == name).unwrap().1;
        assert!(
            page.contains(r#"<div style="border: 1px solid var(--field-border); border-radius: var(--radius); overflow: hidden;">"#),
            "{name}"
        );
    }
    // Light mode is unchanged: the token falls back to --border there.
    let base = include_str!("../../../templates/base.html");
    assert!(base.contains("--field-border: var(--border)"), "base.html");
}

/// A checkbox card's box sits on its bold title, not the middle of the text.
#[test]
fn checkbox_cards_align_the_box_with_the_title() {
    let label = r#"<label style="display: flex; align-items: flex-start; gap: 0.625rem; cursor: pointer; padding: 0.75rem;"#;
    let input = r#"width: 1.125rem; height: 1.125rem; margin-top: 0.18rem; flex-shrink: 0;"#;
    let pages = field_pages();
    let et = &pages[0].1;
    // TON + confirmation + one slot a day + frequency limits (ov4)
    assert_eq!(et.matches(label).count(), 4, "{et}");
    assert!(et.contains(
        r#"<label style="display: flex; align-items: flex-start; gap: 0.625rem; cursor: pointer;">"#
    ));
    assert_eq!(et.matches(input).count(), 5);
    let src = &pages[1].1;
    assert_eq!(src.matches(label).count(), 1, "{src}");
    assert_eq!(src.matches(input).count(), 1);
}

/// On a phone the role select and Remove wrap together, under the name.
#[test]
fn team_member_controls_wrap_as_one() {
    let page = &field_pages()
        .into_iter()
        .find(|p| p.0 == "team_settings.html")
        .unwrap()
        .1;
    assert!(page.contains(".member-row .member-controls { display: flex; align-items: center; gap: 0.5rem; margin-left: auto; flex: none; }"));
    let row = page
        .split(r#"<div class="member-row" data-id="u1""#)
        .nth(1)
        .unwrap();
    let controls = row
        .split(r#"<div class="member-controls">"#)
        .nth(1)
        .unwrap();
    let select = controls.find(r#"<select class="role-select""#).unwrap();
    let remove = controls.find("remove-member-btn").unwrap();
    let close = controls.find("</div>").unwrap();
    assert!(select < remove && remove < close, "{row}");
    // The script builds the same structure for new members and groups.
    assert_eq!(
        page.matches("controls.className = 'member-controls';")
            .count(),
        2
    );
    assert_eq!(page.matches("controls.appendChild(removeBtn);").count(), 2);
    assert!(page.contains("controls.appendChild(roleSelect);"));
    assert!(page.contains("controls.appendChild(badge);"));
    assert!(!page.contains("row.appendChild(removeBtn);"));
}

/// Row actions are the shared buttons: no time-slot chips, no inline colours.
#[test]
fn list_row_actions_use_the_shared_buttons() {
    let team = context! {
        id => "t1", name => "T", slug => "t", visibility => "public", is_team_admin => true,
    };
    let et = context! { id => "et1", title => "Call", slug => "call" };
    let inv = context! {
        id => "i1", guest_name => "Ann", guest_email => "a@example.com",
        invite_url => "https://x.test/i", created_by => "Me", used_count => 0, max_uses => 1,
    };
    let cal = context! { id => "c1", href => "/c1", name => "Cal", event_count => 0 };
    let pages = [
        (
            "dashboard_teams.html",
            context! { lang => "en", sidebar => context! {}, is_admin => true, teams => vec![team] },
        ),
        (
            "dashboard_sources.html",
            context! { lang => "en", sidebar => context! {}, sources => vec![source("s1")] },
        ),
        (
            "dashboard_internal.html",
            context! { lang => "en", sidebar => context! {}, event_types => vec![et] },
        ),
        (
            "invite_form.html",
            context! { lang => "en", sidebar => context! {}, event_type_id => "et1", invites => vec![inv], can_delete_invites => true },
        ),
        (
            "source_calendars.html",
            context! { lang => "en", sidebar => context! {}, source_id => "s1", source_name => "Work", calendars => vec![cal.clone()] },
        ),
        (
            "source_write_setup.html",
            context! { lang => "en", sidebar => context! {}, source_id => "s1", calendars => vec![cal] },
        ),
    ];
    for (name, ctx) in pages {
        let page = render(name, ctx);
        assert!(!page.contains(r#"class="slot-btn"#), "{name}: {page}");
        for t in tags(&page, "button").into_iter().chain(tags(&page, "a")) {
            if t.contains("class=\"btn") {
                assert!(!t.contains("color:"), "{name}: {t}");
                assert!(!t.contains("border-color"), "{name}: {t}");
                assert!(!t.contains("font-size"), "{name}: {t}");
            }
        }
        match name {
            "dashboard_teams.html" => {
                assert_eq!(
                    page.matches(r#"class="btn btn-secondary btn-sm btn-auto">Settings</a>"#)
                        .count(),
                    1,
                    "{page}"
                );
            }
            "dashboard_sources.html" => {
                assert!(page.contains(r#"<button type="submit" class="btn btn-sm btn-auto">"#));
                assert!(page.contains(
                    r#"class="btn btn-danger btn-sm btn-auto source-remove-btn" data-confirm="#
                ));
                assert!(page.contains(".source-actions .btn { width: 100%; min-height: 44px;"));
                assert!(!page.contains(".source-actions .slot-btn"));
            }
            "dashboard_internal.html" => {
                assert!(
                    page.contains(
                        r#"class="btn btn-sm btn-auto quick-link-btn" data-et-id="et1">"#
                    ),
                    "{page}"
                );
                assert!(!page.contains("btn.style.color"));
            }
            "invite_form.html" => {
                assert!(page.contains(r#"id="quick-link-btn" class="btn btn-sm btn-auto" style="white-space: nowrap;">"#));
                assert!(
                    page.contains(r#"class="btn btn-secondary btn-sm btn-auto copy-invite-btn""#)
                );
                assert!(page.contains(
                    r#"class="btn btn-danger btn-sm btn-auto invite-delete-btn" data-confirm="#
                ));
                assert!(!page.contains("btn.style.color"));
                assert!(!page.contains("btn.style.borderColor"));
            }
            "source_calendars.html" => {
                assert!(page.contains(r#"<button type="submit" class="btn btn-auto">"#));
                assert_eq!(
                    page.matches(r#"<button type="button" class="btn btn-secondary btn-sm btn-auto" onclick="#)
                        .count(),
                    2
                );
            }
            "source_write_setup.html" => {
                assert!(page.contains(r#"<div class="btn-row">"#));
                assert!(page.contains(r#"<a href="/dashboard" class="btn btn-ghost btn-auto">"#));
            }
            _ => unreachable!(),
        }
    }
}

/// The invite form's expiry and checkbox stack on a phone: the class alone
/// sets the two columns, so its phone rule can drop to one.
#[test]
fn invite_form_grid_stacks_on_a_phone() {
    let page = &field_pages()
        .into_iter()
        .find(|p| p.0 == "invite_form.html")
        .unwrap()
        .1;
    assert!(page.contains(r#"<div class="form-grid-2">"#), "{page}");
    assert!(!page.contains(r#"grid-template-columns: 1fr 1fr; gap: 1rem;" class="form-grid-2""#));
    assert!(page.contains(".form-grid-2 { grid-template-columns: 1fr; row-gap: 0; }"));
}

/// The generic preset is translated, so the name it fills in is too.
#[test]
fn other_source_preset_is_translated() {
    let providers = vec![
        context! { id => "nextcloud", name => "Nextcloud", url => "", backend => "caldav" },
        context! { id => "other", name => "Other / Generic CalDAV", url => "", backend => "caldav" },
    ];
    for (lang, label) in [
        ("ru", "Другой сервер CalDAV"),
        ("en", "Other CalDAV server"),
    ] {
        let page = render(
            "source_form.html",
            context! { lang => lang, sidebar => context! {}, providers => providers.clone() },
        );
        assert!(
            page.contains(&format!(
                r#"<option value="other" data-url="" data-backend="caldav" >{label}</option>"#
            )),
            "{page}"
        );
        assert!(page.contains(r#"data-backend="caldav" >Nextcloud</option>"#));
        assert!(!page.contains("Generic CalDAV"));
    }
}

/// The weekday checkbox and label sit on the middle of the 32px time row.
#[test]
fn schedule_day_toggle_is_centred_on_the_time_row() {
    let pages = field_pages();
    for name in ["event_type_form.html", "settings.html"] {
        let page = &pages.iter().find(|p| p.0 == name).unwrap().1;
        assert!(
            page.contains("toggle.style.cssText = 'accent-color:var(--accent);width:1rem;height:1rem;margin-top:0.5rem;"),
            "{name}"
        );
        assert!(
            page.contains("font-size:0.875rem;font-weight:500;padding-top:0.3rem;flex-shrink:0;"),
            "{name}"
        );
    }
}

/// The phone hamburger (12px + 40px) leaves a gap above the first card; the
/// troubleshoot legend reads as one block; the directory checkbox matches the
/// other labels.
#[test]
fn dashboard_spacing_and_label_sizes() {
    let page = render(
        "dashboard_directory.html",
        context! { lang => "en", sidebar => context! {} },
    );
    assert!(page.contains(".dashboard-main { padding: 3.75rem 0.75rem 1.5rem; }"));
    assert!(page.contains(".dashboard-main { padding: 3.75rem 1rem 2rem; }"));
    assert!(!page.contains("padding: 3.25rem 0.75rem"));
    assert!(page.contains(r#"<label style="display: flex; align-items: center; gap: 0.5rem; font-weight: 400; font-size: 0.875rem; cursor: pointer;">"#), "{page}");
    let ts = &field_pages()
        .into_iter()
        .find(|p| p.0 == "troubleshoot.html")
        .unwrap()
        .1;
    assert!(ts.contains(
        r#"<div style="display: flex; gap: 0.5rem 1rem; flex-wrap: wrap; font-size: 0.75rem;"#
    ));
}
