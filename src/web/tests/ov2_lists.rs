//! ilia fork: tests for the second overnight review round, group `lists`.

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

/// Only the page's own part (its `dashboard_content` block), without the
/// shell's sidebar and styles. Renders the whole page first, so a syntax
/// error anywhere still fails.
fn content(name: &str, ctx: minijinja::Value) -> String {
    render(name, ctx.clone());
    let env = env();
    let tmpl = env.get_template(name).unwrap();
    let mut state = tmpl.eval_to_state(ctx).unwrap();
    state.render_block("dashboard_content").unwrap()
}

/// The page's inline `<script>` blocks, joined.
fn scripts(page: &str) -> String {
    page.split("<script>")
        .skip(1)
        .filter_map(|s| s.split_once("</script>").map(|(js, _)| js))
        .collect::<Vec<_>>()
        .join("\n")
}

const TRICKY: &str = "O'Brien & \"R&D\" \\";

fn team_settings_ctx(lang: &str) -> minijinja::Value {
    let user = context! {
        id => "u2", name => TRICKY, email => "ob@example.com",
        initials => "O'", has_avatar => false, is_member => false,
    };
    let member = context! {
        id => "u1", name => TRICKY, initials => "O'", has_avatar => false,
        role => "admin", source => "direct",
    };
    let group = |id: &str, count: i64| {
        context! {
            id => id, name => TRICKY, member_count => count, linked => false,
            members => vec![context! { id => "u3", name => TRICKY, initials => "O'", has_avatar => false }],
        }
    };
    context! {
        lang => lang, sidebar => context! {},
        team_id => "t-1", team_name => "Team", team_slug => "team",
        visibility => "public", success => "Saved.",
        members => vec![member],
        linked_groups => vec![group("g9", 1)],
        all_users => vec![user],
        oidc_groups => vec![group("g1", 5), group("g2", 21), group("g3", 1)],
    }
}

#[test]
fn team_settings_hands_names_to_its_script_as_json() {
    let page = content("team_settings.html", team_settings_ctx("en"));
    let js = scripts(&page);
    // A quote, ampersand or trailing backslash stays one valid JS string.
    let json = r#""O\u0027Brien \u0026 \"R\u0026D\" \\""#;
    assert!(js.contains(&format!("name: {json}")), "{js}");
    assert!(js.contains(r#"currentMembers["u1"] = true;"#), "{js}");
    assert!(js.contains(r#"adminMembers["u1"] = true;"#), "{js}");
    assert!(
        !js.contains("&#x27;"),
        "HTML escapes inside the script: {js}"
    );
    assert!(
        !js.contains("&amp;"),
        "HTML escapes inside the script: {js}"
    );
}

#[test]
fn group_member_counts_take_the_plural_of_their_language() {
    for (tpl, ctx) in [
        ("team_settings.html", team_settings_ctx("ru")),
        (
            "team_form.html",
            context! {
                lang => "ru", sidebar => context! {},
                users => Vec::<minijinja::Value>::new(),
                oidc_groups => vec![
                    context! { id => "g1", name => "A", member_count => 5, members => Vec::<minijinja::Value>::new() },
                    context! { id => "g2", name => "B", member_count => 21, members => Vec::<minijinja::Value>::new() },
                ],
            },
        ),
    ] {
        let js = scripts(&render(tpl, ctx));
        assert!(js.contains(r#"countLabel: "5 участников""#), "{tpl}: {js}");
        assert!(js.contains(r#"countLabel: "21 участник""#), "{tpl}: {js}");
        assert!(js.contains("' (' + g.countLabel + ')'"), "{tpl}");
        assert!(!js.contains("MEMBER_COUNT"), "{tpl}");
    }
}

#[test]
fn team_settings_uses_the_shared_controls() {
    let page = content("team_settings.html", team_settings_ctx("ru"));
    assert!(!page.contains("slot-btn"), "{page}");
    assert!(!page.contains("#2d7a2d") && !page.contains("#e6f9e6"));
    assert!(page.contains(r#"<div class="success" role="status">Saved.</div>"#));
    // The avatar picker is the shared, translated file picker.
    assert!(page.contains(r#"class="btn btn-secondary btn-sm btn-auto file-pick">Выбрать файл<input type="file" name="avatar""#), "{page}");
    assert!(page.contains(r#"class="file-pick-input""#));
    assert!(page.contains("data-file-name"));
    // Remove buttons keep the script's hooks; the delete button is a danger button.
    assert!(page.contains(r#"class="btn btn-ghost btn-sm btn-auto remove-member-btn""#));
    assert!(page.contains(r#"class="btn btn-ghost btn-sm btn-auto remove-group-btn""#));
    assert!(
        page.contains("removeBtn.className = 'btn btn-ghost btn-sm btn-auto remove-member-btn'")
    );
    assert!(page.contains(r#"<button type="button" class="btn btn-danger btn-auto" data-confirm="#));
    // The copy row wraps on a phone instead of overflowing.
    assert!(page.contains(r#"id="public-url" readonly value="" aria-label="#));
    assert!(page.contains("flex: 1 1 12rem; min-width: 0;"));
    assert!(page.contains(r#"id="copy-public-btn" class="btn btn-secondary btn-sm btn-auto""#));
    // The slug placeholder is translated, as on the new-team form.
    assert!(!page.contains(r#"placeholder="engineering""#));
    // The search input is a styled form field.
    assert!(
        page.contains(r#"<div class="form-group" style="position: relative; margin-bottom: 0;">"#)
    );
    assert!(page.contains(r#"aria-label="Роль: O&#x27;Brien"#), "{page}");
}

#[test]
fn description_toolbars_draw_the_link_icon() {
    for tpl in ["team_settings.html", "team_form.html"] {
        let page = render(tpl, team_settings_ctx("en"));
        assert!(
            !page.contains("&#128279;") && !page.contains('\u{1F517}'),
            "{tpl}"
        );
        let link = page
            .split(r#"data-action="link""#)
            .nth(1)
            .and_then(|s| s.split_once("</button>"))
            .map(|(b, _)| b)
            .unwrap_or_else(|| panic!("{tpl}: no link button"));
        assert!(
            link.contains(r#"aria-label="Insert link""#),
            "{tpl}: {link}"
        );
        assert!(link.contains(r#"<svg class="icon""#), "{tpl}: {link}");
        // The bar and its buttons take their look from `.md-toolbar`
        // (dashboard_base.html), not inline styles.
        assert!(
            !page.contains("border: 1px solid transparent; border-radius: 3px"),
            "{tpl}"
        );
        assert!(
            page.contains(r#"<div class="md-toolbar" data-target="description">"#),
            "{tpl}"
        );
        let toggle = page
            .split(r#"class="md-preview-toggle""#)
            .nth(1)
            .and_then(|s| s.split_once('>'))
            .map(|(attrs, _)| attrs)
            .unwrap_or_else(|| panic!("{tpl}: no preview toggle"));
        // An inline colour would beat the shared `[aria-pressed="true"]` accent.
        assert!(!toggle.contains("style="), "{tpl}: {toggle}");
        assert!(!page.contains(r#"<div style="flex: 1;"></div>"#), "{tpl}");
    }
}

#[test]
fn list_pages_use_the_shared_new_button() {
    for (tpl, href) in [
        ("dashboard_teams.html", "/dashboard/teams/new"),
        ("dashboard_sources.html", "/dashboard/sources/new"),
        (
            "dashboard_internal.html",
            "/dashboard/event-types/new?preset=internal",
        ),
    ] {
        let page = render(
            tpl,
            context! { lang => "en", sidebar => context! {}, is_admin => true },
        );
        assert!(
            page.contains(&format!(
                r#"<a href="{href}" class="btn btn-sm btn-auto">+ "#
            )),
            "{tpl}: {page}"
        );
    }
}

#[test]
fn empty_states_draw_an_icon_not_an_emoji() {
    for (tpl, emoji) in [
        ("dashboard_teams.html", "&#128101;"),
        ("dashboard_internal.html", "&#127970;"),
        ("dashboard_bookings.html", "&#128203;"),
        ("invite_form.html", "&#9993;"),
    ] {
        let page = render(tpl, context! { lang => "en", sidebar => context! {} });
        assert!(!page.contains(emoji), "{tpl}");
        assert!(
            page.contains(r#"<div class="empty-icon"><svg viewBox="0 0 24 24""#),
            "{tpl}: {page}"
        );
    }
}

#[test]
fn bookings_claim_and_error_follow_the_theme() {
    let row = context! {
        id => "b1", event_title => "Call", guest_name => "Ann",
        guest_email => "a@example.com", start_at => "2026-03-16 10:00",
        team_name => "Sales", claim_token => "tok",
    };
    let page = content(
        "dashboard_bookings.html",
        context! {
            lang => "en", sidebar => context! {},
            claimable_bookings => vec![row], error_message => "Nope.",
        },
    );
    assert!(!page.contains("#3b82f6"), "{page}");
    assert!(page.contains(r#"<span class="badge badge-info">Sales</span>"#));
    assert!(page
        .contains(r#"<div class="error" role="alert" style="margin-bottom: 1rem;">Nope.</div>"#));
}

#[test]
fn quick_invite_link_survives_a_refused_clipboard() {
    let inv = context! {
        id => "i1", guest_name => "Ann", guest_email => "a@example.com",
        invite_url => "https://x.test/i", created_by => "Me", used_count => 0, max_uses => 1,
    };
    let page = content(
        "invite_form.html",
        context! {
            lang => "ru", sidebar => context! {}, event_type_id => "et1",
            invites => vec![inv], bulk_result => context! { sent_count => 2 },
        },
    );
    // A hidden manual-copy field next to the button and in the active row.
    let manual = r#"<label class="invite-manual" hidden"#;
    assert_eq!(page.matches(manual).count(), 2, "{page}");
    let js = scripts(&page);
    assert!(js.contains("if (!r.ok || r.redirected) throw"), "{js}");
    assert!(js.contains("if (!data || !data.url) throw"), "{js}");
    assert!(js.contains(": Promise.reject(new Error('No clipboard'))"));
    assert!(
        !js.contains("navigator.clipboard.writeText(data.url).then"),
        "{js}"
    );
    assert!(
        !js.contains("navigator.clipboard.writeText(url).then"),
        "{js}"
    );
    assert!(page.contains(r#"<a href="/dashboard/event-types" class="back-link">&larr; "#));
    // The sent notice is the shared green box.
    assert!(
        page.contains(r#"<div class="success" role="status"><strong>"#),
        "{page}"
    );
    assert!(!page.contains("rgba(34,197,94"));
}

#[test]
fn booking_messages_use_the_shared_controls() {
    let page = content(
        "ilia/booking_messages.html",
        context! {
            lang => "en", sidebar => context! {}, open => true, booking_id => "b1",
            draft => "Hello", max_chars => 2000,
            messages => vec![context! { who => "x", not_guest => true, text => "hi" }],
        },
    );
    assert!(
        page.contains(r#"<a href="/dashboard/bookings" class="back-link">&larr; "#),
        "{page}"
    );
    assert!(page.contains(r#"<button type="submit" class="btn btn-auto">"#));
    assert!(page.contains(r#"<div class="form-group" style="margin-bottom: 0.375rem;">"#));
    assert!(page.contains(">Hello</textarea>"));
    assert!(
        page.contains(r#"class="msg-not-guest" style="color: var(--warn);""#),
        "{page}"
    );
    assert!(!page.contains("--danger"));
    assert!(!page.contains("slot-btn"));
}

#[test]
fn troubleshoot_draws_past_time_and_matches_its_legend() {
    let block = |status: &str| {
        context! { status => status, start => "09:00", end => "10:00", label => "x", left_pct => 0, width_pct => 10 }
    };
    let page = content(
        "troubleshoot.html",
        context! {
            lang => "en", sidebar => context! {}, et_title => "Call",
            prev_date => "2026-03-15", next_date => "2026-03-17", selected_slug => "call",
            blocks => vec![block("past"), block("outside")],
        },
    );
    // Past time has its own shade and legend entry, apart from outside hours.
    assert!(
        page.contains("background: #94a3b8; opacity: 0.55;"),
        "{page}"
    );
    assert!(page.contains("background: #94a3b8; opacity: 0.2;"));
    assert!(
        page.contains(
            "opacity: 0.55; vertical-align: middle; margin-right: 3px;\"></span> Past</span>"
        ),
        "{page}"
    );
    // Legend swatches carry their block's opacity.
    assert!(page.contains("#22c55e; opacity: 0.45; vertical-align"));
    assert!(page.contains("#ef4444; opacity: 0.7; vertical-align"));
    // The day arrows are shared buttons, still hooked by the script.
    assert!(!page.contains("slot-btn"), "{page}");
    assert_eq!(
        page.matches("btn btn-secondary btn-sm btn-auto tl-nav")
            .count(),
        2
    );
    assert!(page.contains(r#"aria-label="Previous day""#));
    let js = scripts(&page);
    assert!(js.contains("#troubleshoot-form .tl-nav[href]"));
    assert!(js.contains("closest('a.tl-nav')"));
}
