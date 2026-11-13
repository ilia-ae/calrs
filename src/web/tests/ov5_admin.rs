//! ilia fork: tests for the fifth overnight review round, group `admin`.

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

/// The opening tag `<... id="{id}" ...>` of an element.
fn tag_with_id<'a>(html: &'a str, id: &str) -> &'a str {
    let at = html
        .find(&format!(r#"id="{id}""#))
        .unwrap_or_else(|| panic!("{id} renders"));
    let start = html[..at].rfind('<').unwrap();
    let end = at + html[at..].find('>').unwrap();
    &html[start..=end]
}

/// The TON settings page with one field group whose field has an error.
fn ton_page(error: &str) -> String {
    render(
        "ton_settings.html",
        context! {
            lang => "ru", sidebar => context! {},
            debug => context! { telegram => context! { value => "" } },
            status => context! { on => true, network => "testnet", address => "EQ...", address_raw => "0:abc" },
            groups => vec![context! {
                title => "Prices",
                fields => vec![context! {
                    field => "price_usd", value => "x", kind => "usd", default => "1",
                    source => "panel", error => error,
                }],
            }],
        },
    )
}

/// Audit 5 #3: the debug Telegram field shows no real handle as a grey
/// placeholder; empty means "fill nothing in".
#[test]
fn debug_telegram_field_has_no_handle_placeholder() {
    let page = ton_page("");
    let input = tag_with_id(&page, "debug-telegram-input");
    assert!(!input.contains("placeholder"), "{input}");
    assert!(!page.contains("getmonolith"), "{page}");
}

/// Audit 5 #4: a field error is a small red line tied to its input, not a
/// second block alert under the page summary.
#[test]
fn ton_field_error_is_a_line_not_an_alert() {
    let page = ton_page("bad value");
    let span = tag_with_id(&page, "ton-price_usd-error");
    assert!(!span.contains("class="), "{span}");
    assert!(span.contains("color: var(--error-text)"), "{span}");
    assert!(page.contains(">bad value</span>"), "{page}");
    let input = tag_with_id(&page, "ton-price_usd");
    assert!(input.contains(r#"aria-invalid="true""#), "{input}");
    assert!(
        input.contains(r#"aria-describedby="ton-price_usd-error""#),
        "{input}"
    );

    let clean = ton_page("");
    assert!(!clean.contains("ton-price_usd-error"));
    assert!(!clean.contains("aria-invalid"));
}

/// Audit 5 #5: label and value cells of the status table share one padding
/// rule, so a value is not 3px above its label.
#[test]
fn ton_status_cells_share_one_padding() {
    let page = ton_page("");
    let rule = page
        .find("#ton-status td { padding: 0.25rem 0.5rem 0.25rem 0; }")
        .expect("one padding rule for both cells");
    let table = page.find(r#"id="ton-status""#).unwrap();
    assert!(rule < table);
    let end = table + page[table..].find("</table>").unwrap();
    assert!(
        !page[table..end].contains("padding: 0.25rem 0.5rem 0.25rem 0"),
        "no inline padding left on the label cells"
    );
}

/// The admin page with the SMS gateway card's per-provider fields.
fn admin_page() -> String {
    render(
        "admin.html",
        context! {
            lang => "ru", sidebar => context! {}, current_user_id => "u1", local_admin => true,
            users => vec![context! { id => "u2", name => "Bob", email => "bob@example.com", role => "user", enabled => true, auth_provider => "local" }],
            user_count => 1,
            custom_colors => context! { accent => "#111111", accent_hover => "#222222", bg => "#333333", surface => "#444444", text => "#555555" },
            current_theme => "custom",
            sms_configured => true, sms_provider => "twilio",
            sms_providers => crate::sms::PROVIDER_SPECS,
        },
    )
}

/// Audit 5 #6 and #7: the sender and base-URL hints sit under their fields,
/// not inside the label, and the docs link keeps a gap to the next label.
#[test]
fn sms_hints_sit_under_the_fields() {
    let page = admin_page();
    let sender = page
        .find(r#"name="sender""#)
        .expect("the sender field renders");
    let group_start = page[..sender].rfind(r#"<div class="form-group">"#).unwrap();
    let label = &page[group_start..sender];
    assert!(!label.contains("hint"), "{label}");
    let after = &page[sender..sender + page[sender..].find("</div>").unwrap()];
    assert!(after.contains(r#"<span class="hint">"#), "{after}");

    let base = page.find(r#"name="base_url""#).expect("base URL renders");
    let group_start = page[..base].rfind(r#"<div class="form-group">"#).unwrap();
    assert!(!page[group_start..base].contains("hint"));
    let after = &page[base..base + page[base..].find("</div>").unwrap()];
    assert!(after.contains(r#"<span class="hint">"#), "{after}");

    assert!(
        page.contains(r#"font-size: 0.8rem; margin: -0.25rem 0 1.25rem;"><a href="#),
        "docs link keeps a form-group gap below it"
    );
    assert!(!page.contains("font-size: 0.8rem; margin-top: -0.25rem;\"><a"));
}

/// Audit 5 #9: user actions right-align on desktop only; on a phone the
/// wrapped buttons line up under the name.
#[test]
fn user_actions_left_align_on_a_phone() {
    let page = admin_page();
    let actions = page
        .find(r#"<div class="user-actions btn-row""#)
        .expect("user actions render");
    let tag = &page[actions..actions + page[actions..].find('>').unwrap()];
    assert!(!tag.contains("justify-content"), "{tag}");
    let desktop = page
        .find(".user-item .user-actions { opacity: 0;")
        .expect("desktop rule");
    let media = desktop + page[desktop..].find("@media (max-width: 768px)").unwrap();
    let desktop_rule = &page[desktop..desktop + page[desktop..].find('}').unwrap()];
    assert!(desktop_rule.contains("justify-content: flex-end;"));
    let phone = media + page[media..].find(".user-item .user-actions {").unwrap();
    let phone_rule = &page[phone..phone + page[phone..].find('}').unwrap()];
    assert!(
        phone_rule.contains("justify-content: flex-start;"),
        "{phone_rule}"
    );
}

/// Audit 5 #8: the test note keeps a gap above the submit button, and the
/// own-chat checkbox and timing radios sit on their label's first line.
#[test]
fn debug_form_spacing_and_box_alignment() {
    let page = render(
        "ilia/debug_booking.html",
        context! {
            lang => "ru", sidebar => context! {},
            types => vec![context! { slug => "x", title => "X", priced => true }],
            selected => context! { slug => "x", priced => true },
            has_username => true, ton_on => true, timing => "normal",
            fast => context! { window => 10, renewals => 1, renew => 5, grace => 2, margin => 1 },
            timing_bounds => context! { window_min => 5, window_max => 60, renew_max => 60, renewals_max => 3 },
        },
    );
    let note = crate::i18n::translate("ru", "dbg-test-note", None);
    let head: String = note.chars().take(15).collect();
    let at = page.find(&head).expect("test note renders");
    let p = &page[page[..at].rfind("<p").unwrap()..at];
    assert!(p.contains("margin: 0 0 1rem;"), "{p}");
    assert!(page.contains(
        "#dbg-own-chat, #dbg-timing input[type=radio] { margin-top: 0.18rem; flex-shrink: 0; }"
    ));
    assert!(page.contains(r#"id="dbg-timing-normal""#));
}
