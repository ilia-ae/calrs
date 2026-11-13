//! ilia fork: tests for the third overnight review round, group `admin`.

use super::*;

async fn admin_page(app: Router, session: &str) -> String {
    let response = app
        .oneshot(get_authed("/dashboard/admin", session))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    body_string(response).await
}

/// The theme swatches are ringed with a theme token (visible on a dark card),
/// and the theme radios stay focusable, with the focus ring on the card.
#[tokio::test]
async fn theme_picker_is_keyboard_reachable_and_swatches_use_a_token() {
    let (app, _, session, _) = setup_test_app().await;
    let body = admin_page(app, &session).await;
    assert!(!body.contains("rgba(0,0,0,0.1)"), "{body}");
    assert!(body.contains("border: 1px solid var(--border-hover);\"></span>"));
    let radio = body
        .split("<input type=\"radio\" name=\"theme\"")
        .nth(1)
        .expect("theme radio")
        .split_once('>')
        .unwrap()
        .0;
    assert!(!radio.contains("display: none"), "{radio}");
    assert!(radio.contains("opacity: 0"), "{radio}");
    assert!(body.contains(
        "input[name=theme]:focus-visible + .theme-card { outline: 2px solid var(--accent); outline-offset: 2px; }"
    ));
}

/// The webhook gateway's name and secret label are translated; the vendors'
/// own terms are not. The "keep current" hint shows only on the active
/// gateway, whose secret is the one stored.
#[tokio::test]
async fn sms_webhook_labels_are_translated_and_the_keep_hint_is_only_on_the_active_gateway() {
    let (app, pool, session, _) = setup_test_app().await;
    let en = admin_page(app.clone(), &session).await;
    assert!(
        en.contains("<option value=\"webhook\" >Generic webhook</option>"),
        "{en}"
    );
    // Nothing is stored, so the form preselects Twilio: only its block has
    // the hint.
    assert!(en.contains("<label>HMAC secret</label>"), "{en}");
    assert!(en.contains("<label>API token</label>"), "{en}");
    assert!(
        en.contains("<label>Auth token <span class=\"hint\">"),
        "{en}"
    );

    set_user_language(&pool, "test@example.com", "ru").await;
    let ru = admin_page(app, &session).await;
    assert!(ru.contains(">Свой вебхук</option>"), "{ru}");
    assert!(ru.contains("<label>Секрет HMAC</label>"), "{ru}");
    assert!(!ru.contains("Generic webhook"), "{ru}");
    assert!(!ru.contains("HMAC secret"), "{ru}");
    assert!(ru.contains("<label>API key</label>"), "{ru}");
}

/// With a gateway stored, only that gateway's secret says "leave empty to
/// keep current"; the others say it is required on switching.
#[tokio::test]
async fn sms_keep_hint_marks_the_stored_gateway_only() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let (app, pool, session, _) = setup_test_app().await;
    sqlx::query(
        "INSERT INTO sms_config (id, provider, api_key, api_secret_enc, sender, enabled) \
         VALUES ('cfg', 'sevenio', NULL, 'x', 'Calrs', 1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let body = admin_page(app, &session).await;
    assert!(
        body.contains(
            "<label>API key <span class=\"hint\">(leave empty to keep current)</span></label>"
        ),
        "{body}"
    );
    assert!(body.contains("<label>Auth token</label>"), "{body}");
    assert!(body.contains("<label>HMAC secret</label>"), "{body}");
}

/// Every calling code names its ISO 3166-1 region(s), which the admin page
/// renames in the page language on the client.
#[tokio::test]
async fn country_codes_carry_regions_for_localised_names() {
    for (code, regions, label) in crate::sms::phone::COUNTRY_CODES {
        let parts: Vec<&str> = regions.split(' ').collect();
        assert!(!parts.is_empty(), "{code}");
        for r in &parts {
            assert!(
                r.len() == 2 && r.chars().all(|c| c.is_ascii_uppercase()),
                "{code} {label}: {r:?}"
            );
        }
        // One region per name, except +672 ("Australian External
        // Territories"), named after its two places: Norfolk Island and
        // the Antarctic bases.
        let names = if *code == "+672" {
            2
        } else {
            label.matches(" / ").count() + 1
        };
        assert_eq!(parts.len(), names, "{code} {label}: one region per name");
    }
    let (app, _, session, _) = setup_test_app().await;
    let body = admin_page(app, &session).await;
    assert!(
        body.contains("<option value=\"+1\" data-regions=\"US CA\" >"),
        "{body}"
    );
    assert!(body.contains(
        "<option value=\"+971\" data-regions=\"AE\" >United Arab Emirates (+971)</option>"
    ));
    assert!(body.contains("<select name=\"default_country_code\">"));
    assert!(body.contains("new Intl.DisplayNames([lang], { type: 'region' })"));
}

/// The impersonation banner keeps its amber bar but uses the shared button.
#[tokio::test]
async fn impersonation_banner_uses_the_shared_button() {
    let (app, pool, session, _) = setup_test_app().await;
    let target = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO users (id, email, name, role, auth_provider, username, enabled) VALUES (?, 'target@test.com', 'Target User', 'user', 'local', 'target', 1)")
        .bind(&target)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO accounts (id, name, email, timezone, user_id) VALUES (?, 'Target User', 'target@test.com', 'UTC', ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&target)
    .execute(&pool)
    .await
    .unwrap();
    let request = axum::http::Request::builder()
        .uri("/dashboard")
        .header(
            "cookie",
            format!("__Host-calrs_session={session}; __Host-calrs_impersonate={target}"),
        )
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), 200);
    let body = body_string(response).await;
    assert!(body.contains("<strong>Target User</strong>"), "{body}");
    assert!(
        body.contains("<button type=\"submit\" class=\"btn btn-secondary btn-sm btn-auto\">Stop impersonating</button>"),
        "{body}"
    );
    assert!(!body.contains("background: #000"), "{body}");
    // 0.25rem + the 2rem .btn-sm + 0.25rem fills the 2.5rem spacer.
    assert!(body.contains("padding: 0.25rem 1rem;"));
    assert!(body.contains("<div style=\"height: 2.5rem;\"></div>"));
}

/// The Russian e-mail hint on the debug booking page says «адрес», as the
/// rest of the Russian UI does, not the English word.
#[test]
fn russian_debug_email_hint_has_no_english_word() {
    let text = crate::i18n::translate("ru", "dbg-email-help", None);
    assert!(!text.to_lowercase().contains("email"), "{text}");
    assert!(text.contains("адрес для входа"), "{text}");
}
