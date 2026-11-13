//! ilia fork: tests for the second overnight review round, group `dashcore`.
//! The overview, event types list, event type form, overrides and embed
//! pages on the shared button and link system, without emoji icons, with
//! user names written into scripts as JSON and dates in the user's language.

use super::*;

/// GET `uri` as the seeded user and return the page.
async fn page(app: &Router, session: &str, uri: &str) -> String {
    let response = app.clone().oneshot(get_authed(uri, session)).await.unwrap();
    assert_eq!(response.status().as_u16(), 200, "{uri}");
    body_string(response).await
}

/// The dashboard's main column, without the shared shell around it.
fn main_of(html: &str) -> &str {
    let at = html.find(r#"<main class="dashboard-main">"#);
    &html[at.unwrap_or_else(|| panic!("no dashboard main in {html}"))..]
}

/// A colour emoji, literal (the pictographic planes, U+1F000 and up) or as
/// a five-digit numeric entity (&#127760; and the like).
fn has_emoji(s: &str) -> bool {
    s.chars().any(|c| c as u32 >= 0x1F000)
        || s.match_indices("&#1").any(|(i, _)| {
            let digits: String = s[i + 2..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            digits.parse::<u32>().is_ok_and(|n| n >= 0x1F000)
        })
}

const INLINE_LINK_STYLE: &str = "color: var(--accent); text-decoration: none";

async fn set_language(pool: &SqlitePool, lang: &str) {
    sqlx::query("UPDATE users SET language = ? WHERE email = 'test@example.com'")
        .bind(lang)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn overview_pending_actions_and_links_use_the_shared_system() {
    let (app, pool, session, et_id) = setup_test_app().await;
    insert_test_booking(
        &pool,
        &et_id,
        "2030-06-15T10:00:00",
        "2030-06-15T10:30:00",
        "pending",
    )
    .await;
    let html = page(&app, &session, "/dashboard").await;
    let main = main_of(&html);
    assert!(!main.contains("slot-btn"), "{main}");
    assert!(!main.contains(INLINE_LINK_STYLE), "{main}");
    assert!(
        main.contains(r#"<button type="submit" class="btn btn-sm btn-auto">Confirm</button>"#),
        "{main}"
    );
    assert!(
        main.contains(
            r#"<button type="submit" class="btn btn-secondary btn-sm btn-auto">Decline</button>"#
        ),
        "{main}"
    );
    assert!(
        main.contains(r#"<a href="/u/testuser" target="_blank" rel="noopener">/u/testuser</a>"#),
        "{main}"
    );
}

/// The dashboard handlers pass the user's language to the booking dates:
/// the overview, the pending list and the upcoming list.
#[tokio::test]
async fn dashboard_booking_dates_follow_the_user_language() {
    let (app, pool, session, et_id) = setup_test_app().await;
    insert_test_booking(
        &pool,
        &et_id,
        "2030-06-16T10:00:00",
        "2030-06-16T10:30:00",
        "confirmed",
    )
    .await;
    insert_test_booking(
        &pool,
        &et_id,
        "2030-06-15T10:00:00",
        "2030-06-15T10:30:00",
        "pending",
    )
    .await;
    set_language(&pool, "ru").await;

    // The overview lists the pending request.
    let overview = page(&app, &session, "/dashboard").await;
    assert!(
        overview.contains("15 июня 2030, 10:00 — 10:30"),
        "{overview}"
    );
    assert!(!overview.contains("10:00 AM"), "{overview}");

    let bookings = page(&app, &session, "/dashboard/bookings").await;
    assert!(
        bookings.contains("15 июня 2030, 10:00 — 10:30"),
        "{bookings}"
    );
    assert!(
        bookings.contains("16 июня 2030, 10:00 — 10:30"),
        "{bookings}"
    );
    assert!(!bookings.contains("Jun 15"), "{bookings}");
    assert!(!bookings.contains(" AM —"), "{bookings}");
}

#[tokio::test]
async fn event_types_links_carry_no_inline_colour() {
    let (app, pool, session, et_id) = setup_test_app().await;
    sqlx::query("UPDATE event_types SET visibility = 'private' WHERE id = ?")
        .bind(&et_id)
        .execute(&pool)
        .await
        .unwrap();
    let html = page(&app, &session, "/dashboard/event-types").await;
    let main = main_of(&html);
    assert!(!main.contains(INLINE_LINK_STYLE), "{main}");
    assert!(
        main.contains(&format!(
            r#"<a href="/dashboard/invites/{et_id}" style="font-size: 0.75rem; margin-left: 0.25rem;">"#
        )),
        "{main}"
    );

    sqlx::query("DELETE FROM event_types")
        .execute(&pool)
        .await
        .unwrap();
    let html = page(&app, &session, "/dashboard/event-types").await;
    let main = main_of(&html);
    assert!(
        main.contains(r#"<a href="/dashboard/event-types/new">"#),
        "{main}"
    );
    assert!(!main.contains(INLINE_LINK_STYLE), "{main}");
}

#[tokio::test]
async fn overrides_page_is_on_the_shared_system_and_dated_in_the_user_language() {
    let (app, pool, session, et_id) = setup_test_app().await;

    // Empty: a drawn calendar, no emoji; the back link above the card, as on the embed page.
    let html = page(
        &app,
        &session,
        "/dashboard/event-types/test-meeting/overrides",
    )
    .await;
    let main = main_of(&html);
    assert!(!has_emoji(main), "{main}");
    assert!(main.contains(r#"<div class="empty-icon"><svg "#), "{main}");
    assert!(
        main.contains(
            r#"<a href="/dashboard/event-types" class="back-link">&larr; Back to event types</a>
<div class="card">"#
        ),
        "{main}"
    );
    assert!(!main.contains("slot-btn"), "{main}");

    sqlx::query(
        "INSERT INTO availability_overrides (id, event_type_id, date, is_blocked) VALUES (?, ?, '2026-07-01', 1)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&et_id)
    .execute(&pool)
    .await
    .unwrap();
    set_language(&pool, "ru").await;

    let html = page(
        &app,
        &session,
        "/dashboard/event-types/test-meeting/overrides",
    )
    .await;
    let main = main_of(&html);
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let ru = crate::i18n::format_long_date(date, "ru");
    assert_ne!(ru, crate::i18n::format_long_date(date, "en"));
    assert!(main.contains(&ru), "{ru} in {main}");
    assert!(!main.contains("Wednesday, July 1, 2026"), "{main}");
    assert!(!main.contains("slot-btn"), "{main}");
    assert!(
        main.contains(
            r#"<button type="submit" class="btn btn-danger btn-sm btn-auto" data-confirm="#
        ),
        "{main}"
    );
}

#[tokio::test]
async fn event_type_form_uses_shared_buttons_and_no_emoji() {
    let (app, pool, session, _) = setup_test_app().await;
    sqlx::query("UPDATE event_types SET enabled = 0 WHERE slug = 'test-meeting'")
        .execute(&pool)
        .await
        .unwrap();
    let html = page(
        &app,
        &session,
        "/dashboard/event-types/test-meeting/edit?saved=1",
    )
    .await;
    let main = main_of(&html);
    assert!(!main.contains("slot-btn"), "{main}");
    assert!(!has_emoji(main), "{main}");
    assert!(
        !main.contains("&#10003;") && !main.contains("&#9888;"),
        "{main}"
    );
    assert!(!main.contains(INLINE_LINK_STYLE), "{main}");
    assert!(!main.contains(r#"style="color: var(--accent);""#), "{main}");
    // Saved notice and the disabled banner.
    assert!(
        main.contains(r#"<div class="success" role="status">"#),
        "{main}"
    );
    assert!(
        main.contains(r#"<button type="submit" class="btn btn-sm btn-auto">Enable</button>"#),
        "{main}"
    );
    // No light-only colours left for the warnings.
    assert!(
        !main.contains("#b45309") && !main.contains("#eab308"),
        "{main}"
    );
    assert!(!main.contains("#22c55e"), "{main}");
    // Timezone select styled by the base rule, like the others.
    assert!(
        main.contains(r#"<select id="event_type_timezone" name="timezone">"#),
        "{main}"
    );
    for id in ["reset-to-default-btn", "copy-to-all-btn"] {
        assert!(
            main.contains(&format!(
                r#"id="{id}" class="btn btn-ghost btn-sm btn-auto""#
            )),
            "{id}"
        );
    }
    assert!(
        main.contains(r#"id="add-freq-limit" class="btn btn-secondary btn-sm btn-auto""#),
        "{main}"
    );
    assert!(
        main.contains(r#"id="dg-copy-btn" class="btn btn-secondary btn-sm btn-auto""#),
        "{main}"
    );
    // The markdown toolbar: classed, the link button draws its icon.
    // No inline styles: .md-toolbar in dashboard_base styles the bar and its
    // buttons, and an inline colour would hide the preview's pressed state.
    assert!(main.contains(r#"<div class="md-toolbar" data-target="description">"#));
    let bar = &main[main.find(r#"<div class="md-toolbar""#).unwrap()..];
    let bar = &bar[..bar.find("</div>").unwrap()];
    assert!(!bar.contains("style="), "{bar}");
    let link = &main[main.find(r#"data-action="link""#).unwrap()..];
    let link = &link[..link.find("</button>").unwrap()];
    assert!(link.contains(r#"aria-label="Insert link""#), "{link}");
    assert!(link.contains("<svg class=\"icon\""), "{link}");
    // The availability editor's buttons are on the shared system too.
    assert!(main.contains("addBtn.className = 'btn btn-secondary btn-sm btn-auto';"));
    assert!(main.contains("rmBtn.className = 'btn btn-ghost btn-sm btn-auto';"));
    assert!(!main.contains("width:80px"), "{main}");
}

#[tokio::test]
async fn event_type_form_presets_draw_icons() {
    let (app, _pool, session, _) = setup_test_app().await;
    for preset in ["public", "private", "internal", "team"] {
        let html = page(
            &app,
            &session,
            &format!("/dashboard/event-types/new?preset={preset}"),
        )
        .await;
        let main = main_of(&html);
        assert!(!has_emoji(main), "{preset}: {main}");
        // The preset banner is the first thing in the form.
        let banner = &main[main.find(r#"<form method="POST">"#).unwrap()..];
        let banner = &banner[..banner.find("</div>").unwrap()];
        assert!(
            banner.contains(r#"<svg class="icon""#),
            "{preset}: {banner}"
        );
        assert!(banner.contains("<span>Creating "), "{preset}: {banner}");
    }
}

/// Names reach the form's scripts as JSON: an apostrophe is not
/// HTML-escaped there and a trailing backslash does not end the string.
#[tokio::test]
async fn event_type_form_writes_user_names_as_json() {
    let (app, pool, session, _) = setup_test_app().await;
    let ann = insert_role_user(&pool, "ann@example.com", "user").await;
    sqlx::query("UPDATE users SET name = ?, allow_dynamic_group = 1 WHERE id = ?")
        .bind("Ann O'Brien \\")
        .bind(&ann)
        .execute(&pool)
        .await
        .unwrap();
    let test_user: String =
        sqlx::query_scalar("SELECT id FROM users WHERE email = 'test@example.com'")
            .fetch_one(&pool)
            .await
            .unwrap();
    insert_team_with_et(
        &pool,
        "crew",
        "crew-call",
        &[(&test_user, "admin"), (&ann, "member")],
    )
    .await;

    // Edit: the dynamic group picker (allUsers, owner, slug).
    let html = page(&app, &session, "/dashboard/event-types/test-meeting/edit").await;
    assert!(!html.contains("Ann O&#x27;Brien"), "{html}");
    assert!(html.contains("var owner = \"testuser\";"), "{html}");
    assert!(html.contains("var slug = \"test-meeting\";"), "{html}");
    // tojson writes the apostrophe as \u0027 and doubles the backslash.
    assert!(html.contains(r#"username: "ann", name: "Ann O"#), "{html}");
    assert!(html.contains(r#"Brien \\", hasAvatar"#), "{html}");

    // New: the team members of the creation flow.
    let html = page(&app, &session, "/dashboard/event-types/new").await;
    let script = &html[html.find("var teamMembers = {};").unwrap()..];
    let script = &script[..script.find("(function").unwrap_or(script.len())];
    assert!(!script.contains("&#x27;"), "{script}");
    assert!(script.contains(r#"Brien \\", hasAvatar"#), "{script}");
    assert!(script.contains("timezone: \""), "{script}");
}

#[tokio::test]
async fn embed_button_background_has_its_own_hex_field() {
    let (app, _pool, session, _) = setup_test_app().await;
    let html = page(&app, &session, "/dashboard/event-types/test-meeting/embed").await;
    let bg = html.find(r#"id="cfg-button-bg" "#).unwrap();
    let bg_hex = html.find(r#"id="cfg-button-bg-hex""#).unwrap();
    let fg = html.find(r#"id="cfg-button-fg" "#).unwrap();
    let fg_hex = html.find(r#"id="cfg-button-fg-hex""#).unwrap();
    assert!(bg < bg_hex && bg_hex < fg && fg < fg_hex, "{html}");
    assert!(html.contains("$('cfg-button-bg-hex').addEventListener('input'"));
    assert!(html.contains("setVal('cfg-button-bg-hex', state.buttonBg);"));
    assert!(!main_of(&html).contains(r#"style="color: var(--accent);""#));
}

/// The bookings list's row actions are the same buttons as the overview's:
/// no `.slot-btn` with inline green, red or accent colours.
#[tokio::test]
async fn bookings_row_actions_use_the_shared_buttons() {
    let (app, pool, session, et_id) = setup_test_app().await;
    insert_test_booking(
        &pool,
        &et_id,
        "2030-06-15T10:00:00",
        "2030-06-15T10:30:00",
        "pending",
    )
    .await;
    insert_test_booking(
        &pool,
        &et_id,
        "2030-06-16T10:00:00",
        "2030-06-16T10:30:00",
        "confirmed",
    )
    .await;
    let html = page(&app, &session, "/dashboard/bookings").await;
    let main = main_of(&html);
    assert!(!main.contains("slot-btn"), "{main}");
    assert!(!main.contains("color: var(--success)"), "{main}");
    assert!(!main.contains("color: var(--error-text); cursor"), "{main}");
    assert!(
        main.contains(r#"<button type="submit" class="btn btn-sm btn-auto">Confirm</button>"#),
        "{main}"
    );
    assert!(
        main.contains(r#"class="btn btn-secondary btn-sm btn-auto booking-action-btn""#),
        "{main}"
    );
    assert!(main.contains(r#"class="btn btn-danger btn-sm""#), "{main}");
}
