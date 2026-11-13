//! ilia fork: tests for the second overnight review round, group `guest`.

use super::*;

/// Minijinja escapes `/` in HTML; the zone names carry one.
fn html(body: &str) -> String {
    body.replace("&#x2f;", "/")
}

/// A booking stored the current way (UTC, `time_version` 1), 14:00–14:30 in
/// Paris, with every token; each on its own day (one booking per start).
async fn insert_utc_booking(pool: &SqlitePool, et_id: &str, status: &str) -> [String; 4] {
    static DAY: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    let day = DAY.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let id = uuid::Uuid::new_v4().to_string();
    let cancel = uuid::Uuid::new_v4().to_string();
    let resched = uuid::Uuid::new_v4().to_string();
    let confirm = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO bookings (id, event_type_id, uid, guest_name, guest_email, guest_timezone, start_at, end_at, status, cancel_token, reschedule_token, confirm_token, time_version) VALUES (?, ?, ?, 'Guest', 'guest@test.com', 'Europe/Paris', ?, ?, ?, ?, ?, ?, 1)")
        .bind(&id)
        .bind(et_id)
        .bind(format!("uid-{id}"))
        .bind(format!("2030-07-{day:02}T12:00:00Z"))
        .bind(format!("2030-07-{day:02}T12:30:00Z"))
        .bind(status)
        .bind(&cancel)
        .bind(&resched)
        .bind(&confirm)
        .execute(pool)
        .await
        .unwrap();
    [id, cancel, resched, confirm]
}

/// Finding 7: the cancel, approve and decline forms say whose 14:00 it is.
#[tokio::test]
async fn booking_action_forms_name_the_zone_of_their_times() {
    let (app, pool, _, et_id) = setup_test_app().await;
    let [_, cancel, _, _] = insert_utc_booking(&pool, &et_id, "confirmed").await;
    let [_, _, _, confirm] = insert_utc_booking(&pool, &et_id, "pending").await;
    for uri in [
        format!("/booking/cancel/{cancel}"),
        format!("/booking/approve/{confirm}"),
        format!("/booking/decline/{confirm}"),
    ] {
        let resp = app.clone().oneshot(get(&uri)).await.unwrap();
        assert_eq!(resp.status(), 200, "{uri}");
        let body = html(&body_string(resp).await);
        assert!(
            body.contains("14:00 – 14:30 (Europe/Paris)"),
            "{uri}: {body}"
        );
    }
}

/// Finding 7: the result page after the action names the zone too.
#[tokio::test]
async fn booking_action_results_name_the_zone_of_their_times() {
    let (app, pool, _, et_id) = setup_test_app().await;
    let [_, _, _, confirm] = insert_utc_booking(&pool, &et_id, "pending").await;
    let resp = app
        .clone()
        .oneshot(post_bare(&format!("/booking/approve/{confirm}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = html(&body_string(resp).await);
    assert!(body.contains("14:00 – 14:30 (Europe/Paris)"), "{body}");

    let [_, _, _, confirm] = insert_utc_booking(&pool, &et_id, "pending").await;
    let csrf = "test-csrf-ov2-decline";
    let resp = app
        .clone()
        .oneshot(post_form_unauthed(
            &format!("/booking/decline/{confirm}"),
            csrf,
            &format!("_csrf={csrf}&reason=Busy"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = html(&body_string(resp).await);
    assert!(body.contains("14:00 – 14:30 (Europe/Paris)"), "{body}");

    let [_, cancel, _, _] = insert_utc_booking(&pool, &et_id, "confirmed").await;
    let resp = app
        .oneshot(post_form_unauthed(
            &format!("/booking/cancel/{cancel}"),
            csrf,
            &format!("_csrf={csrf}&reason=Busy"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = html(&body_string(resp).await);
    assert!(body.contains("14:00 – 14:30 (Europe/Paris)"), "{body}");
}

/// Finding 7: a legacy row (no zone known on the form) prints no label
/// rather than a false one.
#[tokio::test]
async fn a_legacy_booking_form_prints_no_zone() {
    let (app, pool, _, et_id) = setup_test_app().await;
    let cancel = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO bookings (id, event_type_id, uid, guest_name, guest_email, guest_timezone, start_at, end_at, status, cancel_token, reschedule_token) VALUES (?, ?, 'uid-legacy-zone', 'Guest', 'guest@test.com', 'Europe/Paris', '2030-07-10T10:00:00', '2030-07-10T10:30:00', 'confirmed', ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&et_id)
        .bind(&cancel)
        .bind(uuid::Uuid::new_v4().to_string())
        .execute(&pool)
        .await
        .unwrap();
    let resp = app
        .oneshot(get(&format!("/booking/cancel/{cancel}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = html(&body_string(resp).await);
    assert!(body.contains("10:00 – 10:30</div>"), "{body}");
}

/// Finding 8: the host reschedule page's way back is a ghost button in the
/// button row, not an inline-styled link.
#[tokio::test]
async fn host_reschedule_page_uses_the_button_row() {
    let (app, pool, session, et_id) = setup_test_app().await;
    let [id, _, _, _] = insert_utc_booking(&pool, &et_id, "confirmed").await;
    let resp = app
        .oneshot(get_authed(
            &format!("/dashboard/bookings/{id}/reschedule"),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = html(&body_string(resp).await);
    assert!(body.contains("<div class=\"btn-row\">"), "{body}");
    assert!(
        body.contains("<a class=\"btn btn-ghost\" href=\"/dashboard/bookings\">"),
        "{body}"
    );
    assert!(!body.contains("margin-left: 0.75rem; color: var(--text-muted)"));
}

/// Findings 5 and 74: the payer's invite names no one; the guest's own
/// invite is unchanged.
#[test]
fn the_payer_invite_carries_the_meeting_only() {
    let details = crate::email::BookingDetails {
        event_title: "Consultation".into(),
        date: "2030-07-10".into(),
        start_time: "14:00".into(),
        end_time: "14:30".into(),
        utc_times: Some(("20300710T120000Z".into(), "20300710T123000Z".into())),
        guest_name: "Jane Payersecret".into(),
        guest_email: "jane@guest.test".into(),
        guest_timezone: "Europe/Paris".into(),
        host_name: "Hosty McHost".into(),
        host_email: "host@host.test".into(),
        uid: "uid-payer".into(),
        notes: Some("PRIVATE-NOTE".into()),
        guest_phone: Some("+33 1 23 45 67 89".into()),
        location: Some("https://meet.example/abc".into()),
        additional_attendees: vec!["extra@guest.test".into()],
        sequence: 3,
        ..Default::default()
    };
    let payer = crate::email::generate_ics_payer(&details);
    assert!(payer.contains("METHOD:PUBLISH"), "{payer}");
    assert!(payer.contains("UID:uid-payer"), "{payer}");
    assert!(payer.contains("SEQUENCE:3"), "{payer}");
    assert!(
        payer.contains("SUMMARY:Consultation \u{2014} Hosty\r\n"),
        "{payer}"
    );
    assert!(
        payer.contains("LOCATION:https://meet.example/abc"),
        "{payer}"
    );
    assert!(payer.contains("DTSTART:20300710T120000Z"), "{payer}");
    for private in [
        "Jane",
        "Payersecret",
        "jane@guest.test",
        "extra@guest.test",
        "host@host.test",
        "PRIVATE-NOTE",
        "+33",
        "ATTENDEE",
        "ORGANIZER",
        "DESCRIPTION",
    ] {
        assert!(!payer.contains(private), "{private}: {payer}");
    }
    let guest = crate::email::generate_ics(&details, "PUBLISH");
    assert!(guest.contains("ATTENDEE;CN="), "{guest}");
    assert!(guest.contains("mailto:jane@guest.test"), "{guest}");
    assert!(guest.contains("mailto:extra@guest.test"), "{guest}");
    assert!(guest.contains("PRIVATE-NOTE"), "{guest}");
    assert!(
        guest.contains("SUMMARY:Consultation \u{2014} Jane & Hosty"),
        "{guest}"
    );
}
