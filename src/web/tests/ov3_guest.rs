//! ilia fork: tests for the third overnight review round, group `guest`.

use super::*;

fn env() -> minijinja::Environment<'static> {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_loader(minijinja::path_loader("templates"));
    crate::i18n::register(&mut env);
    env
}

/// The booking form names the zone of the chosen slot, as every later page
/// does, and its TON note no longer asks the guest to pick a time.
#[test]
fn book_form_names_the_zone_and_the_price_note_follows_the_step() {
    let env = env();
    let page = env
        .get_template("book.html")
        .unwrap()
        .render(context! {
            event_type => context! { slug => "intro", title => "Intro Call", duration_min => 30 },
            host_name => "Alice",
            username => "alice",
            date => "2030-06-17",
            date_label => "Monday, June 17, 2030",
            time_start => "11:00",
            time_end => "11:30",
            guest_tz => "Europe/Moscow",
            ton_price => context! { usd => "40.00", ton => "20.00" },
            lang => "en",
        })
        .unwrap();
    assert!(
        page.contains(r#"data-time="11:30">11:30</span> (Europe&#x2f;Moscow)"#),
        "{page}"
    );
    let book_note = crate::i18n::translate("en", "ton-book-price-note", None);
    let slots_note = crate::i18n::translate("en", "ton-slots-price-note", None);
    assert!(page.contains(&book_note), "{page}");
    assert!(!page.contains(&slots_note));

    // A TON price takes the same note.
    let page = env
        .get_template("book.html")
        .unwrap()
        .render(context! {
            event_type => context! { slug => "intro", title => "Intro Call", duration_min => 30 },
            ton_price => context! { ton_exact => "20.00" },
            lang => "ru",
        })
        .unwrap();
    assert!(page.contains("оплата в TON после записи"), "{page}");
    // No zone known: no empty brackets.
    assert!(!page.contains("</span> ()"));
}

/// The slots page keeps its own note; a failed deferred load speaks the
/// page language, in the look of the other empty states.
#[test]
fn slots_page_keeps_its_note_and_translates_the_load_failure() {
    let env = env();
    let page = env
        .get_template("slots.html")
        .unwrap()
        .render(context! {
            event_type => context! { slug => "intro", title => "Intro Call", duration_min => 30 },
            host_name => "Alice",
            username => "alice",
            days => Vec::<minijinja::Value>::new(),
            available_dates => Vec::<String>::new(),
            month_label => "June 2030",
            month_year => "2030-06",
            first_weekday => 0,
            days_in_month => 30,
            today_date => "2030-06-01",
            guest_tz => "UTC",
            ton_price => context! { usd => "40.00" },
            lang => "ru",
        })
        .unwrap();
    assert!(page.contains("оплата в TON после выбора времени"));
    assert!(!page.contains("оплата в TON после записи"));
    assert!(
        page.contains(
            r#""loadFailed": "Не удалось загрузить свободное время. Обновите страницу.""#
        ),
        "the failure message comes from i18n"
    );
    assert!(page.contains("emptyIcon(ICON_BAN) + '<div>' + (i18n.loadFailed"));
    assert!(!page.contains(r#"<div style="color:var(--error-text);">Failed to load"#));
}

/// The tab of an action error names the error, as its heading does.
#[test]
fn action_error_tab_title_is_the_heading() {
    let page = env()
        .get_template("booking_action_error.html")
        .unwrap()
        .render(context! { lang => "ru", title => "Запись не оплачена", message => "M" })
        .unwrap();
    assert!(page.contains("<title>Запись не оплачена</title>"), "{page}");
    // Without a title, the generic one.
    let page = env()
        .get_template("booking_action_error.html")
        .unwrap()
        .render(context! { lang => "ru", message => "M" })
        .unwrap();
    let generic = crate::i18n::translate("ru", "action-error-page-title", None);
    assert!(
        page.contains(&format!("<title>{generic}</title>")),
        "{page}"
    );
}

/// The payment rule names the zone of the deadline, as the state box does.
#[test]
fn payment_deadline_rule_names_the_zone() {
    let page = env()
        .get_template("ton_pay.html")
        .unwrap()
        .render(context! {
            lang => "ru",
            event_title => "Test Meeting",
            pay_state => "waiting",
            can_pay => true,
            live => true,
            amount_ton => "20.00",
            held_until => "23:58",
            zone => "UTC",
            renewals => 0,
            renewals_max => 0,
            seconds_left => 600,
        })
        .unwrap();
    assert!(page.contains("Отправьте платёж до 23:58 (UTC)."), "{page}");
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("time", "23:58");
    args.set("zone", "Asia/Dubai");
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
        let text = crate::i18n::translate(lang, "ton-pay-warn-deadline", Some(&args));
        assert!(
            text.contains("23:58") && text.contains("Asia/Dubai"),
            "{lang}: {text}"
        );
    }
}

/// A paid event type shows its price on the public profile.
#[tokio::test]
async fn profile_lists_the_price_of_a_paid_event_type() {
    let (app, pool, _, et_id) = setup_test_app().await;
    sqlx::query(
        "INSERT INTO ton_prices (event_type_id, currency, price_usd_cents, updated_at) VALUES (?, 'USD', 4000, '2026-11-09T00:00:00Z')",
    )
    .bind(&et_id)
    .execute(&pool)
    .await
    .unwrap();
    let response = app.clone().oneshot(get("/u/testuser")).await.unwrap();
    assert_eq!(response.status(), 200);
    let body = body_string(response).await;
    assert!(body.contains("class=\"et-price\""), "{body}");
    assert!(body.contains("$40.00"), "{body}");
    // Spaced from the duration, as the duration is from the title.
    assert!(!body.contains("</span><span class=\"et-price\""), "{body}");

    // A free type shows none.
    sqlx::query("UPDATE ton_prices SET enabled = 0")
        .execute(&pool)
        .await
        .unwrap();
    let body = body_string(app.oneshot(get("/u/testuser")).await.unwrap()).await;
    assert!(!body.contains("class=\"et-price\""));
}

/// The reschedule banner and confirm step show the current time in the
/// page's zone (the selector's, as the slots), not in the zone the booking
/// was made in.
#[tokio::test]
async fn reschedule_shows_the_current_time_in_the_page_zone() {
    let (app, pool, _, et_id) = setup_test_app().await;
    let tok = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO bookings (id, event_type_id, uid, guest_name, guest_email, guest_timezone, start_at, end_at, status, cancel_token, reschedule_token, language) VALUES (?, ?, 'uid-msk', 'Гость', 'msk@test.com', 'Europe/Moscow', '2030-06-17T10:00:00', '2030-06-17T10:30:00', 'confirmed', ?, ?, 'ru')")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&et_id)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&tok)
        .execute(&pool)
        .await
        .unwrap();
    let long = crate::i18n::format_long_date(NaiveDate::from_ymd_opt(2030, 6, 17).unwrap(), "ru");

    // Dubai is UTC+4.
    let body = body_string(
        app.clone()
            .oneshot(get(&format!("/booking/reschedule/{tok}?tz=Asia%2FDubai")))
            .await
            .unwrap(),
    )
    .await;
    assert!(body.contains(&format!("сейчас: {long} в 14:00")), "{body}");

    // No zone on the URL: the page (selector and slots) is in UTC, so is the
    // banner; not the booking's Moscow (13:00).
    let body = body_string(
        app.clone()
            .oneshot(get(&format!("/booking/reschedule/{tok}")))
            .await
            .unwrap(),
    )
    .await;
    assert!(body.contains(&format!("сейчас: {long} в 10:00")), "{body}");
    assert!(!body.contains(&format!("сейчас: {long} в 13:00")));

    // The confirm step strikes the old time in the same zone.
    let body = body_string(
        app.oneshot(get(&format!(
            "/booking/reschedule/{tok}?date=2030-06-18&time=11:00&tz=Asia%2FDubai"
        )))
        .await
        .unwrap(),
    )
    .await;
    assert!(
        body.contains(" в <span class=\"resched-time\" data-time=\"14:00\">14:00</span>"),
        "{body}"
    );
}
