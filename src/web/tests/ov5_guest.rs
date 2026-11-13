//! ilia fork: tests for the fifth overnight review round, group `guest`.

use super::*;
use crate::web::availability::{common_timezone_keys, timezone_options};

const BASE: &str = include_str!("../../../templates/base.html");

const LOCALES: [&str; 10] = ["de", "en", "es", "et", "fr", "it", "nl", "pl", "pt", "ru"];

/// The label of the option `value` in a `timezone_options` result.
fn tz_option_label(groups: &[minijinja::Value], value: &str) -> String {
    groups
        .iter()
        .flat_map(|g| g.get_attr("options").unwrap().try_iter().unwrap())
        .find(|o| o.get_attr("value").unwrap().as_str() == Some(value))
        .unwrap_or_else(|| panic!("no option {value}"))
        .get_attr("label")
        .unwrap()
        .to_string()
}

/// Finding 0: the popular zones read in the page language, with the same
/// offset suffix; the long IANA tail stays as it is.
#[test]
fn popular_timezones_are_translated() {
    let ru = timezone_options("Europe/Moscow", "ru");
    assert!(
        tz_option_label(&ru, "Europe/Moscow").starts_with("Москва (UTC+3"),
        "{}",
        tz_option_label(&ru, "Europe/Moscow")
    );
    let paris = tz_option_label(&ru, "Europe/Paris");
    assert!(paris.starts_with("Париж, Брюссель (UTC+"), "{paris}");
    assert_eq!(tz_option_label(&ru, "UTC"), "UTC");
    // A zone outside the popular ones keeps its English location name.
    assert!(tz_option_label(&ru, "Asia/Almaty").starts_with("Almaty (UTC+"));

    let en = timezone_options("Europe/Moscow", "en");
    assert_eq!(tz_option_label(&en, "Europe/Moscow"), "Moscow (UTC+3)");
    let de = timezone_options("", "de");
    assert!(tz_option_label(&de, "Europe/Moscow").starts_with("Moskau (UTC+3"));
}

/// Every popular zone's key exists in English (the table is not a call
/// site the key scan sees) and is translated in Russian.
#[test]
fn popular_timezone_keys_exist() {
    let keys: Vec<&str> = common_timezone_keys().collect();
    assert_eq!(keys.len(), 20);
    // `translate` falls back to English, so check each locale file itself.
    let files: Vec<(&str, String)> = LOCALES
        .iter()
        .map(|lang| {
            let path = format!("i18n/{lang}/main.ftl");
            (*lang, std::fs::read_to_string(&path).unwrap())
        })
        .collect();
    for key in keys {
        for (lang, ftl) in &files {
            assert!(
                ftl.lines().any(|l| l.starts_with(&format!("{key} = "))),
                "{lang}: {key} missing"
            );
            let text = crate::i18n::translate(lang, key, None);
            assert!(!text.is_empty() && text != key, "{lang}: {key}");
        }
        assert_ne!(
            crate::i18n::translate("ru", key, None),
            crate::i18n::translate("en", key, None),
            "{key}"
        );
    }
}

/// Finding 1: a long unbroken guest e-mail in a detail row wraps inside
/// the card instead of widening it.
#[test]
fn detail_rows_wrap_long_words() {
    assert!(
        BASE.contains(
            ".detail { margin-bottom: 0.625rem; color: var(--text-secondary); font-size: 0.925rem; overflow-wrap: anywhere; }"
        ),
        "the .detail rule"
    );
}

/// Finding 2: on the profile, a short price stays on one line with its
/// label; the long "free now" sentence still wraps.
#[tokio::test]
async fn profile_price_keeps_its_label_and_amount_together() {
    let (app, pool, _, et_id) = setup_test_app().await;
    sqlx::query(
        "INSERT INTO ton_prices (event_type_id, currency, price_usd_cents, updated_at) VALUES (?, 'USD', 4000, '2026-11-09T00:00:00Z')",
    )
    .bind(&et_id)
    .execute(&pool)
    .await
    .unwrap();
    let body = body_string(app.oneshot(get("/u/testuser")).await.unwrap()).await;
    assert!(
        body.contains("<span class=\"et-price\" style=\"color: var(--text-muted); font-size: 0.85rem; white-space: nowrap;\">&middot; "),
        "{body}"
    );
    assert!(body.contains("$40.00"), "{body}");

    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_loader(minijinja::path_loader("templates"));
    crate::i18n::register(&mut env);
    let render = |price: minijinja::Value| {
        env.get_template("profile.html")
            .unwrap()
            .render(context! {
                lang => "ru",
                username => "u",
                event_types => vec![context! { title => "T", slug => "t", duration_min => 30, price => price }],
            })
            .unwrap()
    };
    let free = render(context! { ladder => true, now_free => true });
    assert!(free.contains("class=\"et-price\""), "{free}");
    assert!(
        free.contains(
            "<span class=\"et-price\" style=\"color: var(--text-muted); font-size: 0.85rem;\">"
        ),
        "{free}"
    );
    let now = render(context! { ladder => true, now_free => false, now_ton => "3" });
    assert!(
        now.contains("font-size: 0.85rem; white-space: nowrap;\">"),
        "{now}"
    );
}

/// A router whose saved company link is `https://example.com`.
async fn app_with_company_link() -> (Router, SqlitePool, String) {
    let pool = setup_test_db().await;
    let (_user_id, _account_id, et_id) = seed_test_data(&pool).await;
    let updated = sqlx::query(
        "UPDATE auth_config SET company_link = 'https://example.com' WHERE id = 'singleton'",
    )
    .execute(&pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(updated, 1);
    let data_dir = std::sync::Arc::new(TestDataDir::new());
    let router = create_router(pool.clone(), data_dir.to_path_buf(), [0u8; 32])
        .await
        .layer(axum::Extension(data_dir))
        .layer(local_proxy_peer());
    (router, pool, et_id)
}

const LINKED_LOGO: &str = "<div class=\"logo\"><a href=\"https:&#x2f;&#x2f;example.com\"";

/// Finding 15: the action pages link the logo to the company site, as the
/// confirmed page does, from the real handlers.
#[tokio::test]
async fn action_pages_link_the_logo() {
    let (app, pool, et_id) = app_with_company_link().await;
    let (_, _, cancel_tok) = insert_test_booking(
        &pool,
        &et_id,
        "2026-06-22T10:00:00",
        "2026-06-22T10:30:00",
        "confirmed",
    )
    .await;
    let confirm_tok = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO bookings (id, event_type_id, uid, guest_name, guest_email, guest_timezone, start_at, end_at, status, cancel_token, reschedule_token, confirm_token) VALUES (?, ?, 'uid-ov5-approve', 'Guest', 'guest@test.com', 'UTC', '2026-06-20T10:00:00', '2026-06-20T10:30:00', 'pending', ?, ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&et_id)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&confirm_tok)
        .execute(&pool)
        .await
        .unwrap();

    for (uri, status) in [
        (format!("/booking/cancel/{cancel_tok}"), 200),
        (format!("/booking/approve/{confirm_tok}"), 200),
        (format!("/booking/decline/{confirm_tok}"), 200),
        // The booking error page (booking_action_error.html).
        ("/booking/approve/invalid-token-123".to_string(), 404),
    ] {
        let resp = app.clone().oneshot(get(&uri)).await.unwrap();
        assert_eq!(resp.status().as_u16(), status, "{uri}");
        let body = body_string(resp).await;
        assert!(body.contains(LINKED_LOGO), "{uri}: {body}");
    }
}

/// Every render of a template with the logo line passes `company_link`,
/// so a page cannot show an unlinked logo while the others link it.
#[test]
fn every_render_of_a_logo_template_passes_the_company_link() {
    let logo_templates: Vec<String> = std::fs::read_dir("templates")
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            std::fs::read_to_string(e.path()).is_ok_and(|s| s.contains("{% if company_link %}"))
        })
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(logo_templates
        .iter()
        .any(|t| t == "booking_cancel_form.html"));

    fn sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n != "tests") {
                    sources(&path, out);
                }
            } else if path.extension().is_some_and(|x| x == "rs") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    sources(std::path::Path::new("src/web"), &mut files);

    let mut checked = 0;
    for file in files {
        let src = std::fs::read_to_string(&file).unwrap();
        for name in &logo_templates {
            let needle = format!("get_template(\"{name}\")");
            for (at, _) in src.match_indices(&needle) {
                let rest = &src[at + needle.len()..];
                let end = [rest.find("get_template("), rest.find("\n}\n")]
                    .into_iter()
                    .flatten()
                    .min()
                    .unwrap_or(rest.len());
                assert!(
                    rest[..end].contains("company_link"),
                    "{}: {name} rendered without company_link",
                    file.display()
                );
                checked += 1;
            }
        }
    }
    assert!(checked >= 30, "{checked}");
}
