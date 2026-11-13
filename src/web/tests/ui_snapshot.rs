//! ilia fork: a snapshot of every page, for a visual review in a browser
//! (open the files directly, or screenshot them with a headless browser in
//! both themes and at a desktop and a phone width). `#[ignore]`: run it on
//! purpose with
//! `CALRS_UI_SNAPSHOT_DIR=<dir> cargo test --locked ui_snapshot -- --ignored`.
//! Each page is written as `<name>.html` with ilia.ae's custom theme inlined
//! in place of `/accent.css`, plus `index.tsv` (name, status, path); a page
//! that answers neither 200 nor 404 is listed with its status and not written.

use super::*;

const PAGES: &[(&str, &str, bool)] = &[
    // Guest side.
    ("public-profile", "/u/testuser", false),
    ("public-slots", "/u/testuser/test-meeting", false),
    (
        "public-book-form",
        "/u/testuser/test-meeting/book?date={DATE}&time=11:00",
        false,
    ),
    ("public-pay-waiting", "/booking/pay/{PAY}", false),
    ("public-pay-confirmed", "/booking/pay/{PAID}", false),
    ("public-cancel", "/booking/cancel/{CANCEL}", false),
    (
        "public-reschedule",
        "/booking/reschedule/{RESCHEDULE}",
        false,
    ),
    ("host-approve", "/booking/approve/{CONFIRM}", false),
    ("host-decline", "/booking/decline/{CONFIRM}", false),
    ("public-not-found", "/u/testuser/no-such-meeting", false),
    ("auth-login", "/auth/login", false),
    ("auth-register", "/auth/register", false),
    // Host side.
    ("dash-overview", "/dashboard", true),
    ("dash-event-types", "/dashboard/event-types", true),
    ("dash-event-type-new", "/dashboard/event-types/new", true),
    (
        "dash-event-type-edit",
        "/dashboard/event-types/test-meeting/edit",
        true,
    ),
    (
        "dash-event-type-overrides",
        "/dashboard/event-types/test-meeting/overrides",
        true,
    ),
    (
        "dash-event-type-embed",
        "/dashboard/event-types/test-meeting/embed",
        true,
    ),
    ("dash-bookings", "/dashboard/bookings", true),
    ("dash-directory", "/dashboard/directory", true),
    ("dash-invite-links", "/dashboard/invite-links", true),
    ("dash-sources", "/dashboard/sources", true),
    ("dash-source-new", "/dashboard/sources/new", true),
    ("dash-teams", "/dashboard/teams", true),
    ("dash-team-new", "/dashboard/teams/new", true),
    (
        "dash-team-settings",
        "/dashboard/teams/{TEAM}/settings",
        true,
    ),
    ("dash-invites", "/dashboard/invites/{ET}", true),
    ("dash-troubleshoot", "/dashboard/troubleshoot", true),
    ("dash-settings", "/dashboard/settings", true),
    ("dash-settings-mfa", "/dashboard/settings/mfa", true),
    (
        "dash-settings-telegram",
        "/dashboard/settings/telegram",
        true,
    ),
    ("dash-admin", "/dashboard/admin", true),
    ("dash-admin-ton", "/dashboard/admin/ton", true),
    ("dash-debug-booking", "/dashboard/admin/debug-booking", true),
];

/// A 120x40 grey mark standing in for the owner's logo in the snapshots.
const SNAPSHOT_LOGO: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='120' height='40'%3E%3Crect width='120' height='40' rx='8' fill='%23888'/%3E%3Ccircle cx='20' cy='20' r='9' fill='%23fff'/%3E%3C/svg%3E";

#[tokio::test]
#[ignore]
async fn ui_snapshot() {
    let Ok(dir) = std::env::var("CALRS_UI_SNAPSHOT_DIR") else {
        panic!("set CALRS_UI_SNAPSHOT_DIR");
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let e = env().await;
    sqlx::query(
        "INSERT OR REPLACE INTO ton_settings (key, value, owner, updated_at) VALUES ('CALRS_DEBUG_BOOKING', '1', 'panel', '2026-11-12T00:00:00Z')",
    )
    .execute(&e.pool)
    .await
    .unwrap();
    sqlx::query("UPDATE auth_config SET registration_enabled = 1")
        .execute(&e.pool)
        .await
        .unwrap();
    // One booking waiting for its payment, one paid and confirmed.
    let (open, pay) = e.book_http("guest@test.com", "10:00", "198.51.100.1").await;
    let confirm = e
        .booking_col(&open, "confirm_token")
        .await
        .unwrap_or_default();
    let (paid_bid, paid) = e.book_http("paid@test.com", "14:00", "198.51.100.2").await;
    let inv = e.inv(&paid_bid).await;
    e.pay(AMOUNT, &code(&inv.reference));
    e.step(e.now).await;
    let cancel = e.booking_col(&paid_bid, "cancel_token").await.unwrap();
    let reschedule = e.booking_col(&paid_bid, "reschedule_token").await.unwrap();
    let date: String =
        sqlx::query_scalar("SELECT substr(start_at, 1, 10) FROM bookings WHERE id = ?")
            .bind(&paid_bid)
            .fetch_one(&e.pool)
            .await
            .unwrap();
    let session = e.session().await;
    // A public team the host administers, with a second member whose long
    // name shows how a member row wraps on a phone.
    let team = uuid::Uuid::new_v4().to_string();
    let (other, _) = insert_test_user_with_session(
        &e.pool,
        "Анна-Мария Константинопольская-Длинная",
        "anna@example.com",
        "anna",
    )
    .await;
    sqlx::query("INSERT INTO teams (id, name, slug, visibility, created_by) SELECT ?, 'Команда', 'komanda', 'public', id FROM users WHERE username = 'testuser'")
        .bind(&team)
        .execute(&e.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id, role, source) SELECT ?, id, 'admin', 'direct' FROM users WHERE username = 'testuser'")
        .bind(&team)
        .execute(&e.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id, role, source) VALUES (?, ?, 'member', 'direct')")
        .bind(&team)
        .bind(&other)
        .execute(&e.pool)
        .await
        .unwrap();
    let theme = crate::web::branding::custom_theme_css(
        "#313131", "#111111", "#f6f6f6", "#ffffff", "#313131",
    );

    // A private meeting, for the invites page (an invite exists only for a
    // private or internal type; private keeps the invite-links list empty).
    let private_et = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO event_types (id, account_id, slug, title, duration_min, buffer_before, buffer_after, min_notice_min, enabled, visibility) VALUES (?, ?, 'private-call', 'Личная встреча', 30, 0, 0, 0, 0, 'private')")
        .bind(&private_et)
        .bind(&e.account_id)
        .execute(&e.pool)
        .await
        .unwrap();

    let mut index = String::new();
    for (name, path, authed) in PAGES {
        let uri = path
            .replace("{DATE}", &date)
            .replace("{PAY}", &pay)
            .replace("{PAID}", &paid)
            .replace("{CANCEL}", &cancel)
            .replace("{RESCHEDULE}", &reschedule)
            .replace("{CONFIRM}", &confirm)
            .replace("{TEAM}", &team)
            .replace("{ET}", &private_et);
        let mut req = axum::http::Request::builder().uri(&uri);
        let mut cookie = String::from("calrs_lang=ru");
        if *authed {
            cookie.push_str(&format!("; __Host-calrs_session={session}"));
        }
        req = req.header("cookie", cookie);
        let resp = e
            .app()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = resp.status().as_u16();
        index.push_str(&format!("{name}\t{status}\t{uri}\n"));
        // The 404 page is designed too.
        if status != 200 && status != 404 {
            continue;
        }
        // A stand-in logo (prod sets one), inlined since the saved file
        // has no server behind `/logo`.
        let page = body_string(resp)
            .await
            .replace(
                "<link rel=\"stylesheet\" href=\"/accent.css\">",
                &format!("<style>{theme}</style>"),
            )
            .replace(
                "<img src=\"/logo\"",
                &format!("<img src=\"{SNAPSHOT_LOGO}\""),
            );
        std::fs::write(dir.join(format!("{name}.html")), page).unwrap();
    }
    std::fs::write(dir.join("index.tsv"), index).unwrap();
}
