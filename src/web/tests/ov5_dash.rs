//! ilia fork: tests for the fifth overnight review round, group `dash`.

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

/// A pending booking of the test event type, with a TON invoice in
/// `invoice` status (none when `None`). Returns the booking id.
async fn pending_booking(
    pool: &SqlitePool,
    et_id: &str,
    hour: u32,
    invoice: Option<&str>,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let start = format!("2099-06-15T{hour:02}:00:00");
    let end = format!("2099-06-15T{hour:02}:30:00");
    sqlx::query("INSERT INTO bookings (id, event_type_id, uid, guest_name, guest_email, guest_timezone, start_at, end_at, status, cancel_token, reschedule_token) VALUES (?, ?, ?, 'Guest', 'guest@test.com', 'UTC', ?, ?, 'pending', ?, ?)")
        .bind(&id)
        .bind(et_id)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&start)
        .bind(&end)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(uuid::Uuid::new_v4().to_string())
        .execute(pool)
        .await
        .unwrap();
    if let Some(status) = invoice {
        let stamp = "2099-06-14T10:00:00Z";
        sqlx::query(
            "INSERT INTO ton_invoices (id, booking_id, event_type_id, pay_token, reference, network,
                 address_raw, event_title, start_at_utc, guest_email_norm, price_usd_cents,
                 rate_usd_micro, rate_sources, rate_at, amount_nano, status, created_at,
                 expires_at, settle_by, hard_deadline)
             VALUES (?, ?, ?, ?, ?, 'testnet', '0:00', 'Test Meeting', ?, 'guest@test.com', 100,
                 1000000, 'test=1', ?, 20000000000, ?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&id)
        .bind(et_id)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(format!("B{}", &uuid::Uuid::new_v4().simple().to_string()[..10]))
        .bind(&start)
        .bind(stamp)
        .bind(status)
        .bind(stamp)
        .bind(stamp)
        .bind(stamp)
        .bind(stamp)
        .execute(pool)
        .await
        .unwrap();
    }
    id
}

/// Finding 10: an unpaid TON booking on the overview gets the bookings
/// page's actions (confirm without payment, decline) and its payment badge,
/// never a plain Confirm that the server would refuse. A paid one and a
/// free one keep Confirm.
#[tokio::test]
async fn overview_offers_unpaid_ton_bookings_the_bookings_page_actions() {
    let (app, pool, session, et_id) = setup_test_app().await;
    let unpaid = pending_booking(&pool, &et_id, 10, Some("open")).await;
    let paid = pending_booking(&pool, &et_id, 11, Some("paid")).await;
    let free = pending_booking(&pool, &et_id, 12, None).await;
    let resp = app
        .oneshot(get_authed("/dashboard", &session))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    // The waive link is a template value, so its slashes come escaped.
    let body = body_string(resp).await.replace("&#x2f;", "/");

    assert!(
        !body.contains(&format!("/dashboard/bookings/{unpaid}/confirm")),
        "no plain Confirm for an unpaid booking"
    );
    assert!(body.contains(&format!(
        "<a href=\"/dashboard/ton/waive/{unpaid}\" class=\"btn btn-sm btn-auto\">"
    )));
    assert!(body.contains(&crate::i18n::translate("en", "ton-dash-waive", None)));
    assert!(body.contains(&format!("/dashboard/bookings/{unpaid}/cancel")));
    assert!(body.contains("badge badge-warning"), "the payment badge");

    for id in [&paid, &free] {
        assert!(body.contains(&format!("/dashboard/bookings/{id}/confirm")));
        assert!(!body.contains(&format!("/dashboard/ton/waive/{id}")));
    }
    // The free booking keeps the plain "pending" badge; the paid one shows
    // its payment state in its place.
    assert_eq!(body.matches("badge badge-pending").count(), 1);
    assert!(body.contains("badge badge-success"));
}

fn priority_form(weights: &[i64]) -> String {
    let members: Vec<minijinja::Value> = weights
        .iter()
        .enumerate()
        .map(|(i, w)| {
            context! {
                user_id => format!("u{i}"), name => "Анна-Мария Константинопольская-Длинная",
                initials => "АК", weight => w, timezone => "Europe/Tallinn",
            }
        })
        .collect();
    render(
        "event_type_form.html",
        context! {
            lang => "ru", sidebar => context! {}, editing => true, form_slug => "call",
            is_group => true, priority_members => members,
            form_pending => context! { blocks_slot => true },
        },
    )
}

/// Finding 11: the selected priority is readable in dark (no white on the
/// light grey fill), a selected Low has a visible border, and every button
/// says whether it is pressed, in the markup and in the script.
#[test]
fn priority_buttons_are_readable_and_expose_their_state() {
    let html = priority_form(&[3, 2, 1]);
    assert!(html.contains(
        ".priority-btn.active-medium { background: var(--text-muted); color: var(--surface); border-color: var(--text-muted); }"
    ));
    assert!(html.contains(
        ".priority-btn.active-low { background: var(--surface-hover); color: var(--text); border-color: var(--border-hover); font-style: italic; }"
    ));
    assert!(!html.contains("var(--text-muted); color: #fff"));
    // Three rows, one pressed button each.
    assert_eq!(
        html.matches("aria-pressed=\"true\" onclick=\"setPriority")
            .count(),
        3
    );
    assert_eq!(
        html.matches("aria-pressed=\"false\" onclick=\"setPriority")
            .count(),
        6
    );
    assert!(html.contains(
        "class=\"priority-btn active-medium\" aria-pressed=\"true\" onclick=\"setPriority(this, 'u1', 'medium')\""
    ));
    // The script keeps the state in step: setPriority, toggleMember and the
    // rows it builds.
    assert!(html.contains("btn.setAttribute('aria-pressed', 'true');"));
    assert!(html.contains("low.setAttribute('aria-pressed', 'true');"));
    assert!(
        html.contains("btn.setAttribute('aria-pressed', level === current ? 'true' : 'false');")
    );
    assert_eq!(
        html.matches("b.className = 'priority-btn'; b.setAttribute('aria-pressed', 'false');")
            .count(),
        3
    );
}

/// Finding 12: on a phone the priority buttons drop under the name, and a
/// long name breaks, in the server-rendered rows and in the script's.
#[test]
fn priority_rows_wrap_on_a_phone() {
    let html = priority_form(&[2]);
    assert!(html.contains(
        "data-row-uid=\"u0\" style=\"display: flex; flex-wrap: wrap; align-items: center;"
    ));
    assert!(html.contains("gap: 0.5rem; flex: 1 1 8rem; min-width: 0; transition: opacity 0.15s;\" data-member-info=\"u0\""));
    assert!(html.contains("font-size: 0.875rem; overflow-wrap: anywhere;\">Анна-Мария"));
    assert!(html.contains(
        "data-priority-btns=\"u0\" style=\"display: flex; gap: 0.25rem; flex-shrink: 0; margin-left: auto;\""
    ));
    // The rows the script builds.
    assert!(
        html.contains("row.style.cssText = 'display: flex; flex-wrap: wrap; align-items: center;")
    );
    assert!(html.contains("nameWrap.style.cssText = 'display: flex; align-items: center; gap: 0.5rem; flex: 1 1 8rem;"));
    assert!(html.contains(
        "name.style.cssText = 'font-weight: 500; font-size: 0.875rem; overflow-wrap: anywhere;';"
    ));
    assert!(html.contains(
        "btns.style.cssText = 'display: flex; gap: 0.25rem; flex-shrink: 0; margin-left: auto;';"
    ));
    assert!(!html.contains("flex: 1; min-width: 0; transition"));
}

/// Finding 13: the favourite star is outlined like the button next to it.
#[test]
fn directory_star_uses_the_field_border() {
    let html = render(
        "partials/directory_entries.html",
        context! {
            lang => "ru", favorites_only => false,
            directory_entries => vec![context! {
                id => "e1", title => "Звонок", favorite => true, owner_name => "Анна",
            }],
        },
    );
    assert!(html.contains(
        ".directory-star { font-size: 1.125rem; line-height: 1; padding: 0 0.6rem; border-color: var(--field-border); }"
    ));
    assert!(html.contains("directory-star\" aria-pressed=\"true\""));
}

/// Finding 14: a wrapped "allow several bookings" label keeps the box on
/// its first line.
#[test]
fn invite_checkbox_label_keeps_the_box_on_the_first_line() {
    let html = render(
        "invite_form.html",
        context! { lang => "ru", sidebar => context! {}, event_type_title => "Звонок", event_type_slug => "call" },
    );
    assert!(html.contains(
        "<label style=\"display: flex; align-items: flex-start; gap: 0.5rem; cursor: pointer;\">\n          <input type=\"checkbox\" name=\"single_use\" style=\"accent-color: var(--accent); width: 1rem; height: 1rem; margin-top: 0.18rem; flex-shrink: 0;\">"
    ));
}
