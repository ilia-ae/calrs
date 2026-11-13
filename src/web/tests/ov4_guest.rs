//! ilia fork: tests for the fourth overnight review round, group `guest`.

use super::*;

const BASE: &str = include_str!("../../../templates/base.html");
const SLOTS: &str = include_str!("../../../templates/slots.html");
const TON_PAY: &str = include_str!("../../../templates/ton_pay.html");

fn env() -> minijinja::Environment<'static> {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_loader(minijinja::path_loader("templates"));
    crate::i18n::register(&mut env);
    env
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

/// The outlined buttons and every time slot take the field outline token,
/// which is --border in light and the stronger --border-hover in dark, so
/// they keep an edge on a dark card.
#[test]
fn outlined_buttons_and_time_slots_take_the_field_border() {
    assert!(
        rule(BASE, ".btn-secondary").contains("border-color: var(--field-border);"),
        "{}",
        rule(BASE, ".btn-secondary")
    );
    assert!(rule(BASE, ".slot-btn").contains("border: 1px solid var(--field-border);"));
    for selector in [".slot-pill", ".week-slot-pill", ".column-slot-pill"] {
        let r = rule(SLOTS, selector);
        assert!(
            r.contains("border: 1px solid var(--field-border);"),
            "{selector}: {r}"
        );
        assert!(!r.contains("var(--border)"), "{selector}: {r}");
    }
}

/// The payment page's state box, its deadline divider and the copy rows sit
/// on a --surface-hover fill, the colour of --border in dark: they take the
/// field outline token. The emailed note is green text, not the fill green.
#[test]
fn payment_box_outlines_take_the_field_border_and_the_note_the_text_green() {
    let css = &TON_PAY[TON_PAY.find("<style>").unwrap()..TON_PAY.find("</style>").unwrap()];
    assert!(rule(css, ".ton-status").contains("border: 1px solid var(--field-border);"));
    assert!(rule(css, ".ton-deadline").contains("border-top: 1px solid var(--field-border);"));
    assert!(rule(css, ".ton-field").contains("border: 1px solid var(--field-border);"));
    assert!(rule(css, ".ton-note.ok").contains("color: var(--success-text);"));

    let page = env()
        .get_template("ton_pay.html")
        .unwrap()
        .render(context! {
            lang => "ru",
            event_title => "Test Meeting",
            pay_state => "waiting",
            can_pay => true,
            live => true,
            emailed => true,
            amount_ton => "20.00",
            held_until => "23:58",
            zone => "UTC",
            renewals => 0,
            renewals_max => 0,
            seconds_left => 600,
        })
        .unwrap();
    assert!(page.contains("<p class=\"ton-note ok\">"), "{page}");
}

/// The new time on the reschedule confirmation is green text in the text
/// token (AA on white), not the fill green.
#[test]
fn reschedule_confirm_new_time_uses_the_text_green() {
    let html = env()
        .get_template("booking_reschedule_confirm.html")
        .unwrap()
        .render(context! { lang => "ru", new_start_time => "10:00", new_end_time => "10:30" })
        .unwrap();
    assert!(
        html.contains("<span style=\"color: var(--success-text); font-weight: 600;\">"),
        "{html}"
    );
    assert!(
        !html.contains("<span style=\"color: var(--success);"),
        "{html}"
    );
}

/// A text field directly in a .form-group gets the select's height floor, so
/// a field and a select side by side end on the same line. The select keeps
/// its own rule.
#[test]
fn form_group_text_field_gets_the_select_floor() {
    assert!(rule(BASE, ".form-group > select").contains("min-height: 2.5rem;"));
    assert!(rule(
        BASE,
        ".form-group > input:not([type=checkbox]):not([type=radio])"
    )
    .contains("min-height: 2.5rem;"));
}

// minijinja escapes the `/` of the link.
const LOGO: &str = r#"<div class="logo"><a href="https:&#x2f;&#x2f;example.com" target="_blank" rel="noopener noreferrer"><img src="/logo" alt="" onerror="this.closest('.logo').style.display='none'"></a></div>"#;

/// The guest and host action pages and their result pages carry the logo
/// inside the narrow column, linked to the company site when one is set,
/// as the confirmed page and the reschedule confirmation do.
#[test]
fn action_and_result_pages_carry_the_logo() {
    let env = env();
    let cases: &[(&str, minijinja::Value)] = &[
        (
            "booking_cancel_form.html",
            context! { lang => "ru", company_link => "https://example.com" },
        ),
        (
            "booking_decline_form.html",
            context! { lang => "ru", company_link => "https://example.com" },
        ),
        (
            "booking_approve_form.html",
            context! { lang => "ru", company_link => "https://example.com" },
        ),
        (
            "booking_action_error.html",
            context! { lang => "ru", title => "T", message => "M", company_link => "https://example.com" },
        ),
        (
            "booking_action_blocked.html",
            context! { lang => "ru", notice_min => 60, company_link => "https://example.com" },
        ),
        (
            "booking_cancelled_guest.html",
            context! { lang => "ru", company_link => "https://example.com" },
        ),
        (
            "booking_approved.html",
            context! { lang => "ru", company_link => "https://example.com" },
        ),
        (
            "booking_declined.html",
            context! { lang => "ru", company_link => "https://example.com" },
        ),
        (
            "booking_claim_form.html",
            context! { lang => "ru", company_link => "https://example.com" },
        ),
        (
            "booking_claimed.html",
            context! { lang => "ru", company_link => "https://example.com" },
        ),
    ];
    for (name, ctx) in cases {
        let html = env
            .get_template(name)
            .unwrap_or_else(|e| panic!("{name}: {e}"))
            .render(ctx)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(
            html.contains(&format!("<div class=\"page-narrow\">\n{LOGO}")),
            "{name}: {html}"
        );
        // Without a company link the logo is there, unlinked.
        let html = env
            .get_template(name)
            .unwrap()
            .render(context! { lang => "ru", title => "T", message => "M", notice_min => 60 })
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(
            html.contains(r#"<div class="logo"><img src="/logo" alt="""#),
            "{name}: {html}"
        );
    }
}

/// The not-found page is the 30rem column of the other result pages, with
/// the logo and the card inside it.
#[tokio::test]
async fn error_page_is_the_narrow_column() {
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
    let narrow = html
        .find("<div class=\"page-narrow\">\n<div class=\"logo\">")
        .expect(&html);
    let card = html.find("<div class=\"card err-card\"").unwrap();
    assert!(narrow < card, "{html}");
}

/// On the profile, the event type row keeps a gap between its text and the
/// arrow, so on a phone the arrow does not touch the price.
#[tokio::test]
async fn profile_event_row_keeps_a_gap_before_the_arrow() {
    let (app, _, _, _) = setup_test_app().await;
    let req = axum::http::Request::builder()
        .uri("/u/testuser")
        .header("accept-language", "en")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let html = body_string(resp).await;
    assert!(
        html.contains("<div style=\"display: flex; justify-content: space-between; align-items: center; gap: 1rem;\">\n      <div style=\"min-width: 0;\">"),
        "{html}"
    );
    assert!(
        html.contains("<span aria-hidden=\"true\" style=\"flex: none; color: var(--accent);"),
        "{html}"
    );
}

/// The claim and payment side pages carry the logo line too, inside the
/// narrow column (finding 1 of the fourth round, the pages outside its list).
#[test]
fn claim_and_payment_side_pages_carry_the_logo() {
    for name in [
        "booking_already_claimed.html",
        "ton_quote.html",
        "ton_pay_exit.html",
        "ton_already_open.html",
    ] {
        let path = format!("{}/templates/{name}", env!("CARGO_MANIFEST_DIR"));
        let src = std::fs::read_to_string(&path).expect("template");
        let at = src.find("<div class=\"page-narrow\">\n<div class=\"logo\">");
        assert!(at.is_some(), "{name}: no logo as the column's first child");
    }
}
