//! Unit tests of the Telegram feature. Nothing here talks to Telegram: HTTP
//! is covered by `parse_response` over the response shapes documented at
//! core.telegram.org/bots/api, and everything else by `fake::FakeApi`.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::NaiveDate;
use serde_json::{json, Value};

use super::api::{self, parse_response, Button, Outgoing, SendMessage, TgError, Update, User};
use super::commands::handle_update;
use super::config::{self, redact, BotConfig};
use super::events::TelegramEvent;
use super::fake::{self, FakeApi};
use super::message::{self, GuestLinks, GuestView, HostView};
use super::poller::{self, PollPolicy};
use super::tokens::{self, Kind};
use super::username;
use super::worker::{self, Job};
use super::{store, text, Runtime};

const TOKEN: &str = "123456789:AAHdqTcvCH1vGWJxfSeofSAs0K5PALDsawQ";
const SECRET: &str = "AAHdqTcvCH1vGWJxfSeofSAs0K5PALDsawQ";

// --- Log capture ---

#[derive(Clone, Default)]
struct Log(Arc<Mutex<Vec<String>>>);

impl Log {
    fn capture() -> (Self, tracing::subscriber::DefaultGuard) {
        use tracing_subscriber::layer::SubscriberExt;
        let log = Self::default();
        let guard =
            crate::test_support::capture_default(tracing_subscriber::registry().with(log.clone()));
        (log, guard)
    }

    fn all(&self) -> String {
        self.0.lock().unwrap().join("\n")
    }
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Log {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        struct Fields(String);
        impl tracing::field::Visit for Fields {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                use std::fmt::Write;
                let _ = write!(self.0, "{}={:?} ", field.name(), value);
            }
        }
        let mut fields = Fields(String::new());
        event.record(&mut fields);
        self.0.lock().unwrap().push(fields.0);
    }
}

// --- 1. Parsing the documented shapes ---

#[test]
fn get_me_parses() {
    let body = r#"{"ok":true,"result":{"id":123456789,"is_bot":true,"first_name":"Ilia",
        "username":"ilia_booking_bot","can_join_groups":false,"can_read_all_group_messages":false,
        "supports_inline_queries":false,"can_connect_to_business":false,"has_main_web_app":false}}"#;
    let me: User = parse_response(200, body).unwrap();
    assert_eq!(me.id, 123456789);
    assert!(me.is_bot);
    assert_eq!(me.username.as_deref(), Some("ilia_booking_bot"));
}

fn sample_updates() -> Value {
    json!({"ok": true, "result": [
        {"update_id": 900000001, "message": {"message_id": 1,
            "from": {"id": 42, "is_bot": false, "first_name": "Guest", "username": "RealName",
                     "language_code": "ru"},
            "chat": {"id": 42, "first_name": "Guest", "username": "RealName", "type": "private"},
            "date": 1762700000, "text": "/start b_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "entities": [{"offset": 0, "length": 6, "type": "bot_command"}]}},
        {"update_id": 900000002, "message": {"message_id": 2,
            "from": {"id": 43, "is_bot": false, "first_name": "Member"},
            "chat": {"id": -1001234567890_i64, "title": "Team", "type": "supergroup"},
            "date": 1762700001, "text": "/start@ilia_booking_bot x"}},
        {"update_id": 900000003, "channel_post": {"message_id": 3,
            "chat": {"id": -1009876543210_i64, "title": "News", "type": "channel"},
            "date": 1762700002, "text": "hello"}},
        {"update_id": 900000004, "edited_message": {"message_id": 1,
            "from": {"id": 42, "is_bot": false, "first_name": "Guest"},
            "chat": {"id": 42, "type": "private"}, "date": 1762700000, "edit_date": 1762700100,
            "text": "/stop"}},
        {"update_id": 900000005, "my_chat_member": {"chat": {"id": 42, "type": "private"},
            "from": {"id": 42, "is_bot": false, "first_name": "Guest"}, "date": 1762700003,
            "old_chat_member": {"status": "member", "user": {"id": 1, "is_bot": true, "first_name": "B"}},
            "new_chat_member": {"status": "kicked", "user": {"id": 1, "is_bot": true, "first_name": "B"},
                                "until_date": 0}}},
        {"update_id": 900000006, "message": {"message_id": 4,
            "from": {"id": 42, "is_bot": false, "first_name": "Guest"},
            "chat": {"id": 42, "type": "private"}, "date": 1762700004,
            "sticker": {"file_id": "x", "file_unique_id": "y", "type": "regular", "width": 512,
                        "height": 512, "is_animated": false, "is_video": false}}},
        {"update_id": 900000007, "message": {"message_id": 5,
            "from": {"id": 42, "is_bot": false, "first_name": "Guest", "is_premium": true,
                     "added_to_attachment_menu": true, "some_future_field": {"a": 1}},
            "chat": {"id": 42, "type": "private", "future": [1, 2]}, "date": 1762700005,
            "text": "/help", "link_preview_options": {"is_disabled": true}}}
    ]})
}

#[test]
fn get_updates_parses_every_kind_and_ignores_unknown_fields() {
    let raw: Vec<Value> = parse_response(200, &sample_updates().to_string()).unwrap();
    assert_eq!(raw.len(), 7);
    let updates: Vec<Update> = raw
        .iter()
        .map(|u| serde_json::from_value(u.clone()).unwrap())
        .collect();
    let start = updates[0].message.as_ref().unwrap();
    assert_eq!(start.chat.kind, "private");
    assert_eq!(
        start.from.as_ref().unwrap().username.as_deref(),
        Some("RealName")
    );
    assert_eq!(
        updates[1].message.as_ref().unwrap().chat.id,
        -1001234567890_i64
    );
    assert_eq!(updates[1].message.as_ref().unwrap().chat.kind, "supergroup");
    // channel_post, edited_message, my_chat_member: no `message`.
    for u in &updates[2..5] {
        assert!(u.message.is_none(), "{u:?}");
    }
    assert!(updates[5].message.as_ref().unwrap().text.is_none());
    assert_eq!(
        updates[6].message.as_ref().unwrap().text.as_deref(),
        Some("/help")
    );
}

#[test]
fn send_message_answer_parses() {
    let body = r#"{"ok":true,"result":{"message_id":77,"from":{"id":123456789,"is_bot":true,
        "first_name":"Ilia","username":"ilia_booking_bot"},"chat":{"id":42,"first_name":"Guest",
        "type":"private"},"date":1762700010,"text":"Booking confirmed"}}"#;
    let message: api::Message = parse_response(200, body).unwrap();
    assert_eq!(message.message_id, 77);
    assert_eq!(message.chat.id, 42);
}

#[test]
fn errors_map_to_their_kind() {
    let cases: Vec<(u16, &str, TgError)> = vec![
        (
            401,
            r#"{"ok":false,"error_code":401,"description":"Unauthorized"}"#,
            TgError::Unauthorized,
        ),
        (
            404,
            r#"{"ok":false,"error_code":404,"description":"Not Found"}"#,
            TgError::Unauthorized,
        ),
        (
            409,
            r#"{"ok":false,"error_code":409,"description":"Conflict: terminated by other getUpdates request; make sure that only one bot instance is running"}"#,
            TgError::Conflict("Conflict: terminated by other getUpdates request; make sure that only one bot instance is running".into()),
        ),
        (
            409,
            r#"{"ok":false,"error_code":409,"description":"Conflict: can't use getUpdates method while webhook is active; use deleteWebhook to delete the webhook first"}"#,
            TgError::Conflict("Conflict: can't use getUpdates method while webhook is active; use deleteWebhook to delete the webhook first".into()),
        ),
        (
            429,
            r#"{"ok":false,"error_code":429,"description":"Too Many Requests: retry after 7","parameters":{"retry_after":7}}"#,
            TgError::RateLimited { retry_after: 7 },
        ),
        (
            400,
            r#"{"ok":false,"error_code":400,"description":"Bad Request: chat not found"}"#,
            TgError::BadRequest("Bad Request: chat not found".into()),
        ),
        (
            403,
            r#"{"ok":false,"error_code":403,"description":"Forbidden: bot was blocked by the user"}"#,
            TgError::Forbidden("Forbidden: bot was blocked by the user".into()),
        ),
        (
            502,
            "<html><body><h1>502 Bad Gateway</h1></body></html>",
            TgError::Server(502),
        ),
        (
            400,
            r#"{"ok":false,"error_code":400,"description":"Bad Request: group chat was upgraded to a supergroup chat","parameters":{"migrate_to_chat_id":-1001234567890}}"#,
            TgError::BadRequest("Bad Request: group chat was upgraded to a supergroup chat".into()),
        ),
    ];
    for (status, body, expected) in cases {
        let got = parse_response::<Value>(status, body).unwrap_err();
        assert_eq!(got, expected, "{body}");
    }
    assert!(matches!(
        parse_response::<Value>(200, "not json"),
        Err(TgError::Other(_))
    ));
    assert!(matches!(
        parse_response::<User>(200, r#"{"ok":true,"result":[1]}"#),
        Err(TgError::Other(_))
    ));
}

// --- 2. Request shape ---

#[test]
fn send_message_is_plain_text_without_previews() {
    let msg = Outgoing {
        text: "<b>hi</b> & bye".into(),
        buttons: vec![
            Button {
                label: "Cancel".into(),
                url: "https://book.example.test/booking/cancel/x".into(),
            },
            Button {
                label: "Local".into(),
                url: "http://localhost:3000/x".into(),
            },
        ],
    };
    let body = serde_json::to_value(SendMessage::new(-1001234567890, &msg)).unwrap();
    assert!(body.get("parse_mode").is_none(), "{body}");
    assert_eq!(body["chat_id"], json!(-1001234567890_i64));
    assert!(body["chat_id"].is_i64());
    assert_eq!(body["text"], "<b>hi</b> & bye");
    assert_eq!(body["link_preview_options"]["is_disabled"], true);
    let rows = body["reply_markup"]["inline_keyboard"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "only https buttons: {body}");
    assert_eq!(
        rows[0][0]["url"],
        "https://book.example.test/booking/cancel/x"
    );

    let plain = Outgoing {
        text: "x".into(),
        buttons: Vec::new(),
    };
    let body = serde_json::to_value(SendMessage::new(42, &plain)).unwrap();
    assert!(body.get("reply_markup").is_none(), "{body}");
}

// --- 3. Redaction ---

#[test]
fn redact_hides_the_token_and_its_secret() {
    let url =
        format!("error sending request for url (https://api.telegram.org/bot{TOKEN}/getUpdates)");
    let out = redact(&url, TOKEN);
    assert!(!out.contains(TOKEN) && !out.contains(SECRET), "{out}");
    assert!(out.contains("<redacted>"), "{out}");
    // The secret half alone, and someone else's token shape.
    assert!(!redact(&format!("x {SECRET} y"), TOKEN).contains(SECRET));
    let other = "bot987654321:ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ/sendMessage";
    let out = redact(other, TOKEN);
    assert!(!out.contains("ZZZZZZZZZZZZZZZZ"), "{out}");
    // Ordinary text is left alone.
    assert_eq!(redact("chat 42 at 12:30", TOKEN), "chat 42 at 12:30");
}

#[test]
fn no_error_displays_the_token() {
    let leak = format!("https://api.telegram.org/bot{TOKEN}/getUpdates");
    let errors = vec![
        TgError::Unauthorized,
        TgError::Conflict(leak.clone()),
        TgError::RateLimited { retry_after: 3 },
        TgError::BadRequest(leak.clone()),
        TgError::Forbidden(leak.clone()),
        TgError::Server(502),
        TgError::Transport(leak.clone()),
        TgError::Other(leak.clone()),
    ];
    for e in errors {
        let e = e.redacted(TOKEN);
        for shown in [e.to_string(), format!("{e:?}")] {
            assert!(!shown.contains(SECRET), "{shown}");
        }
    }
}

#[test]
fn config_debug_hides_the_token() {
    let config = BotConfig::for_tests(TOKEN);
    let shown = format!("{config:?}");
    assert!(!shown.contains(SECRET), "{shown}");
    let shown = format!("{:?}", api::HttpApi::new(config));
    assert!(!shown.contains(SECRET), "{shown}");
}

#[test]
fn token_format_is_checked() {
    assert!(config::valid_token(TOKEN));
    for bad in [
        "",
        "123:AAHdqTcvCH1vGWJxfSeofSAs0K5PALDsawQ",
        "123456789",
        "123456789:short",
        "123456789:AAHdqTcvCH1vGWJxfSeofSAs0K5PALDs wQ",
        "abcdefghi:AAHdqTcvCH1vGWJxfSeofSAs0K5PALDsawQ",
    ] {
        assert!(!config::valid_token(bad), "{bad}");
    }
    assert!(matches!(config::parse(None), Ok(None)));
    assert!(matches!(config::parse(Some("  ")), Ok(None)));
    assert!(config::parse(Some(TOKEN)).unwrap().is_some());
    assert!(config::parse(Some("123456789:bad token")).is_err());
}

#[tokio::test]
async fn a_malformed_token_turns_telegram_off_without_logging_it() {
    let _env = crate::test_support::ENV_LOCK.lock().await;
    let old = std::env::var(config::TOKEN_ENV).ok();
    let malformed = "123456789:AAHdqTcvCH1vGWJxfSeofSAs0K5PALDsaw Q!";
    std::env::set_var(config::TOKEN_ENV, malformed);
    let (log, guard) = Log::capture();
    let result = config::from_env();
    drop(guard);
    match old {
        Some(v) => std::env::set_var(config::TOKEN_ENV, v),
        None => std::env::remove_var(config::TOKEN_ENV),
    }
    assert!(result.is_none());
    let all = log.all();
    assert!(!all.contains("AAHdqTcv"), "{all}");
    if !cfg!(tarpaulin) {
        assert!(all.contains("malformed"), "{all}");
    }
}

// --- 4. Tokens ---

#[test]
fn tokens_have_the_deep_link_shape_and_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for kind in [Kind::Booking, Kind::User] {
        for _ in 0..500 {
            let (plain, hash) = tokens::new_token(kind);
            assert!(plain.len() <= 64 && plain.len() == 45, "{plain}");
            assert!(plain.starts_with(if kind == Kind::Booking { "b_" } else { "u_" }));
            assert!(plain
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'));
            assert_eq!(hash, tokens::hash(&plain));
            assert_eq!(hash.len(), 64);
            assert_eq!(tokens::parse_payload(&plain), Some((kind, hash)));
            assert!(seen.insert(plain));
        }
    }
    assert_eq!(seen.len(), 1000);
}

#[test]
fn malformed_payloads_are_refused() {
    let body = "A".repeat(43);
    assert!(tokens::parse_payload(&format!("b_{body}")).is_some());
    for bad in [
        format!("x_{body}"),
        format!("b_{}", "A".repeat(42)),
        format!("b_{}", "A".repeat(44)),
        format!("b_{}+", "A".repeat(42)),
        format!("b_{}/", "A".repeat(42)),
        format!("b_{}=", "A".repeat(42)),
        format!("b_{} ", "A".repeat(42)),
        format!("b_{}", "A".repeat(63)),
        String::new(),
        "b_".into(),
    ] {
        assert!(tokens::parse_payload(&bad).is_none(), "{bad}");
    }
}

// --- 5. Usernames and modes ---

#[test]
fn usernames_are_normalised() {
    for (raw, want) in [
        ("@Ilia_Ar", "Ilia_Ar"),
        ("ilia_ar", "ilia_ar"),
        ("t.me/ilia_ar", "ilia_ar"),
        ("https://t.me/ilia_ar", "ilia_ar"),
        ("  @abcd ", "abcd"),
    ] {
        assert_eq!(username::normalize(raw).as_deref(), Some(want), "{raw}");
    }
    let long = "a".repeat(33);
    for raw in [
        "ab",
        "1abc",
        "abc_",
        "a b",
        "имя",
        long.as_str(),
        "@",
        "",
        "ab-cd",
    ] {
        assert_eq!(username::normalize(raw), None, "{raw}");
    }
}

#[test]
fn modes_decide_whether_the_field_is_required() {
    use username::{mode, resolve};
    assert_eq!(mode::parse("required"), mode::REQUIRED);
    assert_eq!(mode::parse("bogus"), mode::OFF);
    assert_eq!(resolve("off", Some("not a name!"), "en"), Ok(None));
    assert_eq!(resolve("optional", Some(""), "en"), Ok(None));
    assert_eq!(resolve("optional", None, "en"), Ok(None));
    assert_eq!(
        resolve("optional", Some("@Guest_Name"), "en"),
        Ok(Some("Guest_Name".into()))
    );
    let required = resolve("required", Some(" "), "en").unwrap_err();
    assert_eq!(
        required.1,
        crate::i18n::translate("en", "tg-book-required", None)
    );
    let invalid = resolve("required", Some("no way"), "ru").unwrap_err();
    assert_eq!(
        invalid.0,
        crate::i18n::translate("ru", "tg-book-invalid-title", None)
    );
    assert_eq!(
        invalid.1,
        crate::i18n::translate("ru", "tg-book-invalid", None)
    );
    assert!(invalid.1.contains("Telegram"), "{invalid:?}");
}

// --- 6. Composition ---

fn https_links() -> GuestLinks {
    GuestLinks {
        cancel: Some("https://book.example.test/booking/cancel/CT".into()),
        reschedule: Some("https://book.example.test/booking/reschedule/RT".into()),
        calendar: Some("https://book.example.test/booking/ics/CT".into()),
        pick_time: Some("https://book.example.test/booking/reschedule/RT".into()),
        rebook: Some("https://book.example.test/u/ilia/intro".into()),
    }
}

fn guest_view(lang: &str, title: &str) -> GuestView {
    GuestView {
        event: text::clean(title, text::TITLE_MAX),
        when: message::when(
            lang,
            NaiveDate::from_ymd_opt(2026, 8, 17).unwrap(),
            "14:00",
            "Europe/Paris",
        ),
        host: "Ilia".into(),
        pending: false,
        minutes_to_start: 60,
        location: None,
        links: https_links(),
        payment: None,
    }
}

fn host_view(lang: &str) -> HostView {
    HostView {
        event: "Product demo".into(),
        when: message::when(
            lang,
            NaiveDate::from_ymd_opt(2026, 8, 17).unwrap(),
            "15:00",
            "Asia/Dubai",
        ),
        guest: "Eve".into(),
        email: "eve@example.test".into(),
        guest_linked: false,
        linked_as: Some("RealName".into()),
        entered: Some("Guest_Name".into()),
        bookings_url: Some("https://book.example.test/dashboard/bookings".into()),
        payment: None,
    }
}

#[test]
fn every_event_composes_in_every_locale() {
    for (lang, _) in crate::i18n::supported_with_labels() {
        for event in TelegramEvent::samples() {
            let audience = event.audience();
            let g = message::guest(&event, &guest_view(lang, "Product demo"), lang);
            assert_eq!(g.is_some(), audience.guest, "{lang} {event:?}");
            let h = message::host(&event, &host_view(lang), lang);
            assert_eq!(h.is_some(), audience.hosts, "{lang} {event:?}");
            for (who, msg) in [("guest", g), ("host", h)] {
                let Some(msg) = msg else { continue };
                let what = format!("{lang} {who} {event:?}: {}", msg.text);
                assert!(!msg.text.contains("tg-"), "{what}");
                assert!(msg.text.contains("Product demo"), "{what}");
                assert!(
                    msg.text.contains("14:00") || msg.text.contains("15:00"),
                    "{what}"
                );
                assert!(msg.text.chars().count() <= text::MESSAGE_MAX, "{what}");
                // No "(... (Europe/Paris))": the date already ends with its zone.
                assert!(!msg.text.contains("))"), "{what}");
                for b in &msg.buttons {
                    assert!(!b.label.starts_with("tg-"), "{what}");
                }
            }
        }
    }
}

#[test]
fn exact_english_and_russian_messages() {
    let en = guest_view("en", "Product demo");
    assert_eq!(en.when, "Monday, August 17, 2026, 14:00 (Europe/Paris)");
    let linked = message::guest(&TelegramEvent::GuestLinked, &en, "en").unwrap();
    assert_eq!(
        linked.text,
        format!(
            "Done: notifications about this booking will come here.\n\n\
             Booking confirmed: Product demo, {w}. Host: Ilia.",
            w = en.when
        )
    );
    let labels: Vec<&str> = linked.buttons.iter().map(|b| b.label.as_str()).collect();
    assert_eq!(labels, ["Add to calendar", "Reschedule", "Cancel booking"]);
    let cancelled = message::guest(
        &TelegramEvent::Cancelled {
            by_host: false,
            reason: Some("secret reason".into()),
        },
        &en,
        "en",
    )
    .unwrap();
    assert_eq!(
        cancelled.text,
        format!("You cancelled your booking: Product demo, {}.", en.when)
    );
    let reminder = message::guest(&TelegramEvent::Reminder, &en, "en").unwrap();
    assert_eq!(
        reminder.text,
        format!("Reminder: Product demo starts in 60 minutes, {}.", en.when)
    );

    let ru = guest_view("ru", "Знакомство");
    assert!(
        ru.when.contains("17 августа 2026, 14:00 (Europe/Paris)"),
        "{}",
        ru.when
    );
    let linked = message::guest(&TelegramEvent::GuestLinked, &ru, "ru").unwrap();
    assert_eq!(
        linked.text,
        format!(
            "Готово: уведомления об этой записи будут приходить сюда.\n\n\
             Запись подтверждена: Знакомство, {w}. Организатор: Ilia.",
            w = ru.when
        )
    );
    let cancelled = message::guest(
        &TelegramEvent::Cancelled {
            by_host: false,
            reason: None,
        },
        &ru,
        "ru",
    )
    .unwrap();
    assert_eq!(
        cancelled.text,
        format!("Вы отменили запись: Знакомство, {}.", ru.when)
    );
    let mut soon = ru.clone();
    soon.minutes_to_start = 21;
    let reminder = message::guest(&TelegramEvent::Reminder, &soon, "ru").unwrap();
    assert_eq!(
        reminder.text,
        format!(
            "Напоминание: Знакомство начнётся через 21 минуту, {}.",
            ru.when
        )
    );
    let cancel = &reminder.buttons.last().unwrap();
    assert_eq!(cancel.label, "Отменить запись");
}

#[test]
fn russian_host_messages_name_the_guest_once_without_nested_brackets() {
    let v = host_view("ru");
    let cancelled = message::host(
        &TelegramEvent::Cancelled {
            by_host: false,
            reason: None,
        },
        &v,
        "ru",
    )
    .unwrap();
    assert_eq!(
        cancelled.text,
        format!("Отмена записи: Product demo, {}. Гость: Eve.", v.when)
    );
    let moved = message::host(&TelegramEvent::Rescheduled { pending: false }, &v, "ru").unwrap();
    assert_eq!(
        moved.text,
        format!(
            "Перенос записи: Product demo, теперь {}. Гость: Eve.",
            v.when
        )
    );
    let linked = message::host(&TelegramEvent::GuestLinked, &v, "ru").unwrap();
    assert!(
        linked.text.starts_with(&format!(
            "Eve будет получать уведомления о встрече «Product demo», {}, в Telegram: @RealName.",
            v.when
        )),
        "{}",
        linked.text
    );
    for (lang, _) in crate::i18n::supported_with_labels() {
        let v = host_view(lang);
        for event in TelegramEvent::samples() {
            if let Some(msg) = message::host(&event, &v, lang) {
                assert!(!msg.text.contains("))"), "{lang} {event:?}: {}", msg.text);
            }
        }
    }
}

#[test]
fn confirmed_messages_and_the_reminder_say_where_the_meeting_is() {
    let mut call = guest_view("en", "Demo");
    call.location = Some("https://meet.example.test/abc-def".into());
    for event in [
        TelegramEvent::GuestLinked,
        TelegramEvent::Approved { reschedule: false },
        TelegramEvent::Rescheduled { pending: false },
        TelegramEvent::Reminder,
    ] {
        let msg = message::guest(&event, &call, "en").unwrap();
        assert_eq!(msg.buttons[0].label, "Join", "{event:?}");
        assert_eq!(msg.buttons[0].url, "https://meet.example.test/abc-def");
    }
    let mut room = guest_view("ru", "Demo");
    room.location = Some("Офис, 3 этаж".into());
    let msg = message::guest(&TelegramEvent::Reminder, &room, "ru").unwrap();
    assert!(msg.text.ends_with("\nМесто: Офис, 3 этаж"), "{}", msg.text);
    assert!(msg.buttons.iter().all(|b| b.label != "Подключиться"));
    // A request does not reveal the place, as the pending email does not.
    let mut pending = call.clone();
    pending.pending = true;
    let msg = message::guest(&TelegramEvent::GuestLinked, &pending, "en").unwrap();
    assert!(msg.buttons.iter().all(|b| b.label != "Join"), "{msg:?}");
    // Messages about an ended booking carry no place either.
    let msg = message::guest(&TelegramEvent::Expired, &call, "en").unwrap();
    assert!(msg.buttons.iter().all(|b| b.label != "Join"), "{msg:?}");
}

#[test]
fn the_bot_describes_itself_in_every_locale() {
    for (lang, _) in crate::i18n::supported_with_labels() {
        let (long, short) = message::description(lang, "book.example.test");
        assert!(
            long.contains("book.example.test") && long.contains("/stop"),
            "{lang}: {long}"
        );
        assert!(
            long.chars().count() <= 512 && short.chars().count() <= 120,
            "{lang}"
        );
        assert!(
            !long.starts_with("tg-") && !short.starts_with("tg-"),
            "{lang}"
        );
    }
}

#[test]
fn remaining_time_takes_the_plural_of_each_locale() {
    for (minutes, want) in [
        (1, "через 1 минуту"),
        (2, "через 2 минуты"),
        (5, "через 5 минут"),
        (11, "через 11 минут"),
        (21, "через 21 минуту"),
        (22, "через 22 минуты"),
        (25, "через 25 минут"),
        (90, "через 90 минут"),
        (180, "через 3 часа"),
        (300, "через 5 часов"),
        (1440, "через 24 часа"),
        (2880, "через 2 дня"),
        (7200, "через 5 дней"),
    ] {
        assert_eq!(message::remaining(minutes, "ru"), want, "{minutes}");
    }
    assert_eq!(message::remaining(1, "en"), "in 1 minute");
    assert_eq!(message::remaining(2, "en"), "in 2 minutes");
    assert_eq!(message::remaining(60, "en"), "in 60 minutes");
    assert_eq!(message::remaining(120, "en"), "in 2 hours");
    assert_eq!(message::remaining(1440 * 3, "en"), "in 3 days");
    assert_eq!(message::remaining(5, "pl"), "za 5 minut");
    assert_eq!(message::remaining(0, "en"), "in 1 minute");
}

#[test]
fn http_links_go_into_the_text() {
    let mut view = guest_view("en", "Demo");
    view.links = GuestLinks {
        cancel: Some("http://localhost:3000/booking/cancel/CT".into()),
        ..Default::default()
    };
    let msg = message::guest(&TelegramEvent::Reminder, &view, "en").unwrap();
    assert!(msg.buttons.is_empty());
    assert!(
        msg.text
            .ends_with("Cancel booking: http://localhost:3000/booking/cancel/CT"),
        "{}",
        msg.text
    );
}

#[test]
fn the_host_sees_the_linked_username_and_the_typed_one() {
    let v = host_view("en");
    let msg = message::host(&TelegramEvent::GuestLinked, &v, "en").unwrap();
    assert!(msg.text.contains(": @RealName."), "{}", msg.text);
    assert!(
        msg.text.ends_with("Entered on the form: @Guest_Name."),
        "{}",
        msg.text
    );
    let same = HostView {
        entered: Some("realname".into()),
        ..v.clone()
    };
    let msg = message::host(&TelegramEvent::GuestLinked, &same, "en").unwrap();
    assert!(!msg.text.contains("Entered"), "{}", msg.text);
    let none = HostView {
        linked_as: None,
        entered: None,
        ..v
    };
    let msg = message::host(&TelegramEvent::GuestLinked, &none, "ru").unwrap();
    assert!(msg.text.contains("без имени пользователя"), "{}", msg.text);
}

// --- 7. Untrusted text ---

#[test]
fn guest_text_is_cleaned_and_shown_literally() {
    let name = "Eve\u{202E}lirpA\n\u{200B}<b>&amp;</b>\u{E0041}";
    let cleaned = text::clean(name, text::NAME_MAX);
    assert_eq!(cleaned, "EvelirpA <b>&amp;</b>");
    let view = HostView {
        guest: cleaned,
        ..host_view("en")
    };
    let msg = message::host(
        &TelegramEvent::BookingCreated { pending: false },
        &view,
        "en",
    )
    .unwrap();
    assert!(
        msg.text.contains("Guest: EvelirpA <b>&amp;</b>, eve@"),
        "{}",
        msg.text
    );
    assert!(!msg.text.contains('\u{202E}') && !msg.text.contains('\n'));

    let long = "T".repeat(500);
    let view = guest_view("en", &long);
    assert!(view.event.ends_with('…') && view.event.chars().count() == text::TITLE_MAX);
    let msg = message::guest(&TelegramEvent::Reminder, &view, "en").unwrap();
    assert!(msg.text.contains("14:00"), "{}", msg.text);

    let reason = Some("Line one\nLine two\u{202E}".to_string());
    let msg = message::guest(
        &TelegramEvent::Declined { reason },
        &guest_view("en", "Demo"),
        "en",
    )
    .unwrap();
    assert!(
        msg.text.ends_with("\nReason: Line one Line two"),
        "{}",
        msg.text
    );
}

#[test]
fn guest_text_cannot_pose_as_a_link_in_host_messages() {
    assert_eq!(
        text::defang("Payment failed, re-verify at https://ilia-pay.example/login"),
        "Payment failed, re-verify at https[:]//ilia-pay[.]example/login"
    );
    assert_eq!(
        text::defang("contact @support_ilia_bot or /start now"),
        "contact [@]support_ilia_bot or [/]start now"
    );
    assert_eq!(text::defang("www.evil.test"), "www[.]evil[.]test");
    // Ordinary names are left alone.
    for name in [
        "Jean-Luc Picard",
        "J. R. R. Tolkien",
        "O'Brien",
        "Анна Петрова",
    ] {
        assert_eq!(text::defang(name), name);
    }
    let view = HostView {
        guest: text::defang(&text::clean(
            "Payment failed, re-verify at https://ilia-pay.example/login @support_bot",
            text::GUEST_NAME_MAX,
        )),
        ..host_view("en")
    };
    for event in TelegramEvent::samples() {
        if let Some(msg) = message::host(&event, &view, "en") {
            assert!(!msg.text.contains("://"), "{event:?}: {}", msg.text);
            assert!(!msg.text.contains("ilia-pay.example"), "{}", msg.text);
            assert!(!msg.text.contains(" @support_bot"), "{}", msg.text);
        }
    }
    // A guest's cancellation reason is guest text too.
    let msg = message::host(
        &TelegramEvent::Cancelled {
            by_host: false,
            reason: Some("see www.pay.example or @helpdesk_bot".into()),
        },
        &host_view("en"),
        "en",
    )
    .unwrap();
    assert!(
        msg.text
            .ends_with("Reason: see www[.]pay[.]example or [@]helpdesk_bot"),
        "{}",
        msg.text
    );
}

#[test]
fn host_messages_never_carry_booking_tokens() {
    let tokened = [
        "/booking/cancel/",
        "/booking/reschedule/",
        "/booking/approve/",
        "/booking/ics/",
    ];
    let mut guest_has_them = false;
    for (lang, _) in crate::i18n::supported_with_labels() {
        for event in TelegramEvent::samples() {
            if let Some(msg) = message::host(&event, &host_view(lang), lang) {
                let all = format!("{} {:?}", msg.text, msg.buttons);
                for t in tokened {
                    assert!(!all.contains(t), "{lang} {event:?}: {all}");
                }
            }
            if let Some(msg) = message::guest(&event, &guest_view(lang, "Demo"), lang) {
                let all = format!("{} {:?}", msg.text, msg.buttons);
                guest_has_them |= tokened.iter().any(|t| all.contains(t));
            }
        }
    }
    assert!(guest_has_them);
}

// --- 8. Update filters ---

fn runtime(api: Arc<FakeApi>) -> (Arc<Runtime>, tokio::sync::mpsc::Receiver<Job>) {
    Runtime::new(api, Some("ilia_test_bot".into()), fake::fast_retry())
}

/// The replies queued so far, as (chat, text).
fn replies(rx: &mut tokio::sync::mpsc::Receiver<Job>) -> Vec<(i64, String)> {
    let mut out = Vec::new();
    while let Ok(job) = rx.try_recv() {
        match job {
            Job::Reply { chat_id, msg, .. } => out.push((chat_id, msg.text)),
            Job::Event {
                booking_id, event, ..
            } => out.push((0, format!("event {booking_id} {}", event.kind()))),
            Job::Shutdown => {}
        }
    }
    out
}

async fn db_snapshot(pool: &sqlx::SqlitePool) -> String {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT COALESCE(booking_id, '') || '|' || COALESCE(chat_id, '') FROM telegram_guest_links
         UNION ALL SELECT user_id || '|' || chat_id FROM telegram_user_chats",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    format!("{rows:?}")
}

#[tokio::test]
async fn groups_channels_bots_and_edits_are_ignored() {
    let pool = fake::memory_pool().await;
    let (rt, mut rx) = runtime(Arc::new(FakeApi::default()));
    let before = db_snapshot(&pool).await;
    let raw: Vec<Value> = parse_response(200, &sample_updates().to_string()).unwrap();
    // The group /start, the channel post, the edited /stop, the member update.
    for u in &raw[1..5] {
        handle_update(&rt, &pool, u).await;
    }
    let mut bot = fake::private_text(1, 42, None, "/help");
    bot["message"]["from"]["is_bot"] = json!(true);
    handle_update(&rt, &pool, &bot).await;
    let mut relayed = fake::private_text(2, 42, None, "/help");
    relayed["message"]["chat"]["id"] = json!(43);
    handle_update(&rt, &pool, &relayed).await;
    let mut group = fake::private_text(3, -100, None, "/help");
    group["message"]["chat"]["type"] = json!("group");
    handle_update(&rt, &pool, &group).await;
    let mut no_from = fake::private_text(4, 42, None, "/help");
    no_from["message"].as_object_mut().unwrap().remove("from");
    handle_update(&rt, &pool, &no_from).await;
    // The sticker has no text.
    handle_update(&rt, &pool, &raw[5]).await;
    handle_update(&rt, &pool, &json!({"update_id": 9, "message": "nonsense"})).await;
    assert!(replies(&mut rx).is_empty());
    assert_eq!(db_snapshot(&pool).await, before);
}

#[tokio::test]
async fn free_text_gets_the_help_once_in_ten_minutes() {
    let pool = fake::memory_pool().await;
    let (rt, mut rx) = runtime(Arc::new(FakeApi::default()));
    for i in 0..3 {
        handle_update(
            &rt,
            &pool,
            &fake::private_text(i, 42, None, "hello, <b>bot</b>"),
        )
        .await;
    }
    let got = replies(&mut rx);
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(
        got[0].1.contains("/stop") && !got[0].1.contains("<b>"),
        "{got:?}"
    );
    // /help, a bare /start and an idle /stop say nothing new either: the
    // same window. A command for another bot never gets an answer.
    for (i, text) in ["/help", "/HELP@Ilia_Test_Bot", "/start", "/stop", "/nope"]
        .into_iter()
        .enumerate()
    {
        handle_update(
            &rt,
            &pool,
            &fake::private_text(5 + i as i64, 42, None, text),
        )
        .await;
    }
    let payload = format!("b_{}", "A".repeat(43));
    handle_update(
        &rt,
        &pool,
        &fake::private_text(11, 42, None, &format!("/start@other_bot {payload}")),
    )
    .await;
    assert!(replies(&mut rx).is_empty());
    // Another chat gets its own answer.
    handle_update(
        &rt,
        &pool,
        &fake::private_text(12, 43, None, "/HELP@Ilia_Test_Bot"),
    )
    .await;
    assert_eq!(replies(&mut rx).len(), 1);
}

#[tokio::test]
async fn replies_to_strangers_have_a_global_budget_and_never_fill_the_outbox() {
    let pool = fake::memory_pool().await;
    let (rt, mut rx) = runtime(Arc::new(FakeApi::default()));
    // 80 chats each send /help once: 60 replies a minute in all.
    for chat in 0..80 {
        handle_update(
            &rt,
            &pool,
            &fake::private_text(chat, 5000 + chat, None, "/help"),
        )
        .await;
    }
    assert_eq!(replies(&mut rx).len(), 60);

    // With the outbox half full of notifications, replies are dropped and
    // notifications still go in.
    let (rt, mut rx) = runtime(Arc::new(FakeApi::default()));
    for i in 0..(worker::QUEUE_CAPACITY - worker::REPLY_RESERVE) {
        assert!(rt.enqueue(Job::Event {
            pool: pool.clone(),
            booking_id: format!("b-{i}"),
            event: TelegramEvent::Reminder,
        }));
    }
    handle_update(&rt, &pool, &fake::private_text(1, 42, None, "/help")).await;
    assert!(rt.enqueue(Job::Event {
        pool: pool.clone(),
        booking_id: "last".into(),
        event: TelegramEvent::Reminder,
    }));
    let queued = replies(&mut rx);
    assert_eq!(
        queued.len(),
        worker::QUEUE_CAPACITY - worker::REPLY_RESERVE + 1
    );
    assert!(queued.iter().all(|(chat, _)| *chat == 0), "a reply got in");
}

#[tokio::test]
async fn stop_says_whether_anything_was_linked() {
    let pool = fake::memory_pool().await;
    let (rt, mut rx) = runtime(Arc::new(FakeApi::default()));
    handle_update(&rt, &pool, &fake::private_text(1, 42, None, "/stop")).await;
    assert_eq!(
        replies(&mut rx)[0].1,
        crate::i18n::translate("en", "tg-bot-not-linked", None)
    );
    // A host chat: /stop removes it and always says so.
    let (user_id, _) = seed_user(&pool).await;
    sqlx::query("INSERT INTO telegram_user_chats (user_id, chat_id) VALUES (?, 42)")
        .bind(&user_id)
        .execute(&pool)
        .await
        .unwrap();
    handle_update(&rt, &pool, &fake::private_text(2, 42, None, "/stop")).await;
    assert_eq!(
        replies(&mut rx),
        [(42, crate::i18n::translate("en", "tg-bot-stopped", None))]
    );
    assert!(store::user_chat(&pool, &user_id).await.is_none());
}

#[tokio::test]
async fn an_expired_host_link_gets_the_host_wording() {
    let pool = fake::memory_pool().await;
    let (rt, mut rx) = runtime(Arc::new(FakeApi::default()));
    let (user_id, _) = seed_user(&pool).await;
    let (token, hash) = tokens::new_token(Kind::User);
    store::new_user_token(&pool, &user_id, &hash).await.unwrap();
    sqlx::query("UPDATE telegram_user_link_tokens SET expires_at = '2000-01-01 00:00:00'")
        .execute(&pool)
        .await
        .unwrap();
    handle_update(
        &rt,
        &pool,
        &fake::private_text(1, 42, None, &format!("/start {token}")),
    )
    .await;
    let (bogus, _) = tokens::new_token(Kind::Booking);
    handle_update(
        &rt,
        &pool,
        &fake::private_text(2, 42, None, &format!("/start {bogus}")),
    )
    .await;
    assert_eq!(
        replies(&mut rx),
        [
            (
                42,
                crate::i18n::translate("en", "tg-bot-link-invalid-host", None)
            ),
            (
                42,
                crate::i18n::translate("en", "tg-bot-link-invalid", None)
            ),
        ]
    );
}

// --- 9. Rate limits ---

#[tokio::test]
async fn failed_links_are_rate_limited_per_chat() {
    let pool = fake::memory_pool().await;
    let (rt, mut rx) = runtime(Arc::new(FakeApi::default()));
    let invalid = crate::i18n::translate("en", "tg-bot-link-invalid", None);
    let limited = crate::i18n::translate("en", "tg-bot-rate-limited", None);
    for i in 0..7 {
        let payload = format!("b_{:A>43}", i);
        handle_update(
            &rt,
            &pool,
            &fake::private_text(i, 42, None, &format!("/start {payload}")),
        )
        .await;
    }
    let got: Vec<String> = replies(&mut rx).into_iter().map(|(_, t)| t).collect();
    let mut want = vec![invalid.clone(); 5];
    want.push(limited);
    assert_eq!(got, want);
    // Another chat is not affected.
    handle_update(
        &rt,
        &pool,
        &fake::private_text(8, 77, None, "/start garbage"),
    )
    .await;
    assert_eq!(replies(&mut rx), vec![(77, invalid)]);
}

#[tokio::test]
async fn a_valid_token_waits_out_the_limit() {
    let pool = fake::memory_pool().await;
    let (rt, mut rx) = runtime(Arc::new(FakeApi::default()));
    let (user_id, _) = seed_user(&pool).await;
    let (token, hash) = tokens::new_token(Kind::User);
    store::new_user_token(&pool, &user_id, &hash).await.unwrap();
    for i in 0..6 {
        handle_update(&rt, &pool, &fake::private_text(i, 42, None, "/start nope")).await;
    }
    replies(&mut rx);
    handle_update(
        &rt,
        &pool,
        &fake::private_text(9, 42, None, &format!("/start {token}")),
    )
    .await;
    assert!(replies(&mut rx).is_empty());
    assert!(store::user_chat(&pool, &user_id).await.is_none());
    // From another chat it still works: the token was not burnt.
    handle_update(
        &rt,
        &pool,
        &fake::private_text(10, 77, Some("Host_Name"), &format!("/start {token}")),
    )
    .await;
    let got = replies(&mut rx);
    assert!(got[0].1.contains("Host One"), "{got:?}");
    assert_eq!(
        store::user_chat(&pool, &user_id)
            .await
            .unwrap()
            .0
            .as_deref(),
        Some("Host_Name")
    );
}

#[tokio::test]
async fn junk_from_many_chats_never_switches_linking_off() {
    let pool = fake::memory_pool().await;
    let (rt, mut rx) = runtime(Arc::new(FakeApi::default()));
    let (log, guard) = Log::capture();
    // Twenty accounts, five junk payloads each (the per-chat maximum), half
    // of them token-shaped: far past the old shared cap of 60 an hour.
    for chat in 0..20 {
        for i in 0..5 {
            let payload = if i % 2 == 0 {
                "x".to_string()
            } else {
                tokens::new_token(Kind::Booking).0
            };
            handle_update(
                &rt,
                &pool,
                &fake::private_text(i, 1000 + chat, None, &format!("/start {payload}")),
            )
            .await;
        }
    }
    drop(guard);
    // Only token-shaped payloads count toward the alert.
    assert_eq!(rt.limits.global_failures.count(&()), 40);
    replies(&mut rx);
    // A real host token still links, from a chat that never failed.
    let (user_id, _) = seed_user(&pool).await;
    let (token, hash) = tokens::new_token(Kind::User);
    store::new_user_token(&pool, &user_id, &hash).await.unwrap();
    handle_update(
        &rt,
        &pool,
        &fake::private_text(99, 77, Some("Host_Name"), &format!("/start {token}")),
    )
    .await;
    assert!(store::user_chat(&pool, &user_id).await.is_some());
    let got = replies(&mut rx);
    assert!(
        got.iter().any(|(c, t)| *c == 77 && t.contains("Host One")),
        "{got:?}"
    );
    // Past 60 token-shaped misses, one warning; still nothing refused.
    let (log2, guard) = Log::capture();
    for i in 0..30 {
        let (junk, _) = tokens::new_token(Kind::Booking);
        handle_update(
            &rt,
            &pool,
            &fake::private_text(i, 3000 + i, None, &format!("/start {junk}")),
        )
        .await;
    }
    drop(guard);
    if !cfg!(tarpaulin) {
        let all = format!("{}{}", log.all(), log2.all());
        assert_eq!(
            all.matches("many unknown Telegram links").count(),
            1,
            "{all}"
        );
    }
}

async fn seed_user(pool: &sqlx::SqlitePool) -> (String, String) {
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, name, role, auth_provider, username, enabled, language)
         VALUES (?, 'host@example.test', 'Host One', 'user', 'local', 'hostone', 1, 'ru')",
    )
    .bind(&user_id)
    .execute(pool)
    .await
    .unwrap();
    (user_id, "hostone".into())
}

// --- 10. Worker ---

fn msg(text: &str) -> Outgoing {
    Outgoing {
        text: text.into(),
        buttons: Vec::new(),
    }
}

#[tokio::test]
async fn rate_limited_sends_wait_and_succeed() {
    let pool = fake::memory_pool().await;
    let api = Arc::new(FakeApi::default());
    api.script_send(vec![Err(TgError::RateLimited { retry_after: 1 })]);
    let (rt, _rx) = runtime(api.clone());
    assert!(worker::deliver(&rt, &pool, 42, &msg("hi"), "test", None, 3).await);
    assert_eq!(api.attempts.lock().unwrap().len(), 2);
    assert_eq!(api.texts_to(42), ["hi"]);
}

#[tokio::test]
async fn a_blocked_bot_unlinks_the_chat() {
    let pool = fake::memory_pool().await;
    let (user_id, _) = seed_user(&pool).await;
    sqlx::query("INSERT INTO telegram_user_chats (user_id, chat_id) VALUES (?, 42)")
        .bind(&user_id)
        .execute(&pool)
        .await
        .unwrap();
    let api = Arc::new(FakeApi::default());
    api.script_send(vec![Err(TgError::Forbidden(
        "Forbidden: bot was blocked by the user".into(),
    ))]);
    let (rt, _rx) = runtime(api.clone());
    assert!(!worker::deliver(&rt, &pool, 42, &msg("hi"), "test", None, 3).await);
    assert_eq!(api.attempts.lock().unwrap().len(), 1);
    assert!(store::user_chat(&pool, &user_id).await.is_none());
}

#[tokio::test]
async fn a_bad_request_is_not_retried() {
    let pool = fake::memory_pool().await;
    let api = Arc::new(FakeApi::default());
    api.script_send(vec![Err(TgError::BadRequest(
        "Bad Request: chat not found".into(),
    ))]);
    let (rt, _rx) = runtime(api.clone());
    assert!(!worker::deliver(&rt, &pool, 42, &msg("hi"), "test", None, 3).await);
    assert_eq!(api.attempts.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn transport_failures_give_up_after_three_and_log_no_token() {
    let pool = fake::memory_pool().await;
    let api = Arc::new(FakeApi::default());
    let leak = format!(
        "connect: error sending request for url (https://api.telegram.org/bot{TOKEN}/sendMessage)"
    );
    api.script_send(vec![
        Err(TgError::Transport(leak.clone()).redacted(TOKEN)),
        Err(TgError::Transport(leak.clone()).redacted(TOKEN)),
        Err(TgError::Server(502)),
    ]);
    let (rt, _rx) = runtime(api.clone());
    let (log, guard) = Log::capture();
    let ok = worker::deliver(&rt, &pool, 42, &msg("secret text"), "test", Some("b-1"), 3).await;
    drop(guard);
    assert!(!ok);
    assert_eq!(api.attempts.lock().unwrap().len(), 3);
    let all = log.all();
    assert!(
        !all.contains(SECRET) && !all.contains("secret text") && !all.contains("42"),
        "{all}"
    );
    if !cfg!(tarpaulin) {
        assert!(all.contains("telegram message not delivered"), "{all}");
    }
}

#[tokio::test]
async fn a_reply_is_tried_once_and_never_waits() {
    let pool = fake::memory_pool().await;
    let api = Arc::new(FakeApi::default());
    api.script_send(vec![
        Err(TgError::RateLimited { retry_after: 60 }),
        Err(TgError::Server(502)),
    ]);
    let (rt, _rx) = runtime(api.clone());
    let started = std::time::Instant::now();
    assert!(!worker::deliver(&rt, &pool, 42, &msg("hi"), "help", None, 1).await);
    assert!(!worker::deliver(&rt, &pool, 42, &msg("hi"), "help", None, 1).await);
    assert!(started.elapsed() < Duration::from_millis(500));
    assert_eq!(api.attempts.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn a_full_queue_drops_the_job() {
    let pool = fake::memory_pool().await;
    let (rt, mut rx) = runtime(Arc::new(FakeApi::default()));
    for i in 0..(worker::QUEUE_CAPACITY + 5) {
        rt.enqueue(Job::Reply {
            pool: pool.clone(),
            chat_id: i as i64,
            msg: msg("x"),
            what: "test",
            attempts: 1,
        });
    }
    assert_eq!(replies(&mut rx).len(), worker::QUEUE_CAPACITY);
}

#[tokio::test]
async fn shutdown_drains_what_was_queued() {
    let pool = fake::memory_pool().await;
    let api = Arc::new(FakeApi::default());
    let (rt, rx) = runtime(api.clone());
    for i in 0..3 {
        rt.enqueue(Job::Reply {
            pool: pool.clone(),
            chat_id: i,
            msg: msg("x"),
            what: "test",
            attempts: 1,
        });
    }
    let handle = super::TelegramHandle {
        rt: Some(rt.clone()),
        tasks: vec![tokio::spawn(worker::run(rt.clone(), rx))],
    };
    let started = std::time::Instant::now();
    handle.finish(Duration::from_secs(5)).await;
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(api.sent().len(), 3);
}

// --- 11. Poller ---

#[tokio::test]
async fn the_poller_persists_its_offset_and_stops_on_request() {
    let pool = fake::memory_pool().await;
    let api = Arc::new(FakeApi::default());
    let batch: Vec<Value> = vec![
        fake::private_text(500, 42, None, "/help"),
        json!({"update_id": 502, "channel_post": {"message_id": 1,
               "chat": {"id": -100, "type": "channel"}, "date": 1}}),
        json!({"update_id": 501, "message": {"broken": true}}),
    ];
    api.updates.lock().unwrap().extend([
        Ok(batch),
        Err(TgError::Conflict(
            "Conflict: terminated by other getUpdates request".into(),
        )),
    ]);
    let (rt, mut rx) = Runtime::new(api.clone(), None, fake::fast_retry());
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let policy = PollPolicy {
        conflict: Duration::from_millis(20),
        ..PollPolicy::default()
    };
    let task = tokio::spawn(poller::run(rt.clone(), pool.clone(), stop_rx, policy));
    // getMe, the batch, the 409, then a long poll that never answers.
    for _ in 0..500 {
        if api.offsets.lock().unwrap().len() >= 3 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(*api.offsets.lock().unwrap(), [0, 503, 503]);
    assert_eq!(store::load_offset(&pool).await, 503);
    assert_eq!(rt.bot_username().as_deref(), Some("ilia_test_bot"));
    assert_eq!(
        store::cached_bot_username(&pool).await.as_deref(),
        Some("ilia_test_bot")
    );
    // The menu and the description: the default, then every two-letter
    // shipped language (de-formal is served by de).
    let langs: Vec<Option<String>> = api
        .commands
        .lock()
        .unwrap()
        .iter()
        .map(|(l, _)| l.clone())
        .collect();
    let want: Vec<Option<String>> = std::iter::once(None)
        .chain(
            ["fr", "es", "pl", "de", "it", "et", "pt", "nl", "ru"]
                .into_iter()
                .map(|l| Some(l.to_string())),
        )
        .collect();
    let mut sorted = langs.clone();
    sorted.sort();
    let mut want_sorted = want;
    want_sorted.sort();
    assert_eq!(sorted, want_sorted);
    let descriptions = api.descriptions.lock().unwrap().clone();
    assert_eq!(descriptions.len(), langs.len());
    let ru = descriptions
        .iter()
        .find(|(l, _, _)| l.as_deref() == Some("ru"))
        .unwrap();
    assert!(ru.1.starts_with("Этот бот присылает"), "{}", ru.1);
    assert_eq!(replies(&mut rx).len(), 1);
    stop_tx.send(true).unwrap();
    tokio::time::timeout(Duration::from_secs(1), task)
        .await
        .expect("the poller stops promptly")
        .unwrap();
}

#[test]
fn bot_usernames_are_checked() {
    for good in ["ilia_booking_bot", "IliaBot", "abcbot"] {
        assert!(poller::valid_bot_username(good), "{good}");
    }
    for bad in ["ab", "1abot", "ilia_booking", "ilia-bot", "bot", "имя_bot"] {
        assert!(!poller::valid_bot_username(bad), "{bad}");
    }
}

#[tokio::test]
async fn a_token_for_another_bot_starts_the_poller_state_over() {
    let pool = fake::memory_pool().await;
    // First start with a token: what is stored is kept, the id recorded.
    store::save_offset(&pool, 900_000_123).await;
    store::save_bot_username(&pool, "old_ilia_bot").await;
    assert!(!store::claim_bot(&pool, 111111).await);
    assert_eq!(store::load_offset(&pool).await, 900_000_123);
    // Same bot again: nothing changes.
    assert!(!store::claim_bot(&pool, 111111).await);
    assert_eq!(store::load_offset(&pool).await, 900_000_123);
    // Another bot: its updates start low, so the old offset would hide
    // them all; and links must not name the old bot.
    assert!(store::claim_bot(&pool, 222222).await);
    assert_eq!(store::load_offset(&pool).await, 0);
    assert_eq!(store::cached_bot_username(&pool).await, None);
    let config = config::parse(Some(TOKEN)).unwrap().unwrap();
    assert_eq!(config.bot_id(), Some(123456789));
}

// --- Retention ---

#[tokio::test]
async fn old_links_and_tokens_are_purged() {
    let pool = fake::memory_pool().await;
    let (user_id, _) = seed_user(&pool).await;
    sqlx::query(
        "INSERT INTO telegram_user_link_tokens (token_hash, user_id, expires_at)
         VALUES ('old', ?, '2000-01-01 00:00:00'), ('new', ?, '2999-01-01 00:00:00')",
    )
    .bind(&user_id)
    .bind(&user_id)
    .execute(&pool)
    .await
    .unwrap();
    store::purge(&pool, chrono::Utc::now()).await;
    let left: Vec<String> = sqlx::query_scalar("SELECT token_hash FROM telegram_user_link_tokens")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(left, ["new"]);
}

// --- TON payments (ilia-booking docs/ton.md) ---

const PAY_URL: &str = "https://book.example.test/booking/pay/0123456789abcdef0123456789abcdef";

fn guest_payment(lang: &str, open: bool, partly_paid: bool) -> message::GuestPayment {
    message::GuestPayment {
        amount: "20.00".into(),
        paid: if partly_paid { "5.00" } else { "0.00" }.into(),
        rest: if partly_paid { "15.00" } else { "20.00" }.into(),
        usd: Some("40.00".into()),
        deadline: message::when(
            lang,
            NaiveDate::from_ymd_opt(2026, 8, 10).unwrap(),
            "14:30",
            "Europe/Paris",
        ),
        open,
        partly_paid,
        pay_url: Some(PAY_URL.into()),
    }
}

fn host_payment() -> message::HostPayment {
    message::HostPayment {
        amount: "20.00".into(),
        paid: "20.00".into(),
        usd: Some("40.00".into()),
    }
}

#[test]
fn every_payment_event_composes_in_every_locale() {
    for (lang, _) in crate::i18n::supported_with_labels() {
        for event in TelegramEvent::payment_samples() {
            let audience = event.audience();
            let mut gv = guest_view(lang, "Product demo");
            gv.payment = Some(guest_payment(lang, true, false));
            let g = message::guest(&event, &gv, lang);
            assert_eq!(g.is_some(), audience.guest, "{lang} {event:?}");
            let mut hv = host_view(lang);
            hv.payment = Some(host_payment());
            let h = message::host(&event, &hv, lang);
            assert_eq!(h.is_some(), audience.hosts, "{lang} {event:?}");
            for (who, msg) in [("guest", g), ("host", h)] {
                let Some(msg) = msg else { continue };
                let what = format!("{lang} {who} {event:?}: {}", msg.text);
                assert!(!msg.text.contains("tg-"), "{what}");
                assert!(!msg.text.contains("ton-"), "{what}");
                assert!(msg.text.contains("Product demo"), "{what}");
                assert!(
                    msg.text.contains("14:00") || msg.text.contains("15:00"),
                    "{what}"
                );
                assert!(!msg.text.contains("))"), "{what}");
                assert!(msg.text.chars().count() <= text::MESSAGE_MAX, "{what}");
                for b in &msg.buttons {
                    assert!(!b.label.starts_with("tg-"), "{what}");
                }
                if who == "host" {
                    // Amounts only: never the payment page.
                    assert!(!msg.text.contains("/booking/pay/"), "{what}");
                    for b in &msg.buttons {
                        assert!(!b.url.contains("/booking/pay/"), "{what}");
                    }
                }
            }
        }
    }
}

/// ilia fork: a price in TON says no dollars, to the guest or the hosts,
/// in any locale.
#[test]
fn a_ton_price_shows_no_dollars() {
    for (lang, _) in crate::i18n::supported_with_labels() {
        let mut gv = guest_view(lang, "Product demo");
        let mut p = guest_payment(lang, true, false);
        p.usd = None;
        gv.payment = Some(p);
        let mut hv = host_view(lang);
        hv.payment = Some(message::HostPayment {
            usd: None,
            ..host_payment()
        });
        for event in TelegramEvent::payment_samples() {
            for msg in [
                message::guest(&event, &gv, lang),
                message::host(&event, &hv, lang),
            ]
            .into_iter()
            .flatten()
            {
                assert!(!msg.text.contains('$'), "{lang} {event:?}: {}", msg.text);
                assert!(!msg.text.contains("tg-"), "{lang} {event:?}: {}", msg.text);
            }
        }
        let due = message::guest(&TelegramEvent::PaymentPartial, &gv, lang);
        assert!(due.is_none() || due.unwrap().text.contains("20.00"));
        let received = message::host(&TelegramEvent::PaymentReceived, &hv, lang).unwrap();
        assert!(received.text.contains("20.00"), "{lang}: {}", received.text);
    }
}

#[test]
fn the_payment_link_goes_only_into_open_payment_messages_to_the_guest() {
    let lang = "en";
    let mut v = guest_view(lang, "Product demo");
    v.payment = Some(guest_payment(lang, true, false));
    for event in TelegramEvent::payment_samples() {
        let carries = message::guest(&event, &v, lang).is_some_and(|m| {
            m.text.contains(PAY_URL) || m.buttons.iter().any(|b| b.url == PAY_URL)
        });
        assert_eq!(
            carries,
            matches!(
                event,
                TelegramEvent::PaymentDue
                    | TelegramEvent::PaymentPartial
                    | TelegramEvent::PaymentRenewed { .. }
            ),
            "{event:?}"
        );
    }
    // Once the invoice is no longer open, nobody is asked to pay.
    v.payment = Some(guest_payment(lang, false, false));
    assert!(message::guest(&TelegramEvent::PaymentPartial, &v, lang).is_none());
    assert!(message::guest(&TelegramEvent::PaymentRenewed { renewal: 1 }, &v, lang).is_none());
    let linked = message::guest(&TelegramEvent::PaymentDue, &v, lang).unwrap();
    assert_eq!(
        linked.text,
        message::guest(&TelegramEvent::GuestLinked, &v, lang)
            .unwrap()
            .text
    );
    assert!(linked.buttons.iter().all(|b| b.url != PAY_URL));
}

#[test]
fn exact_english_and_russian_payment_messages() {
    let mut en = guest_view("en", "Product demo");
    en.payment = Some(guest_payment("en", true, false));
    let deadline = en.payment.as_ref().unwrap().deadline.clone();
    assert_eq!(deadline, "Monday, August 10, 2026, 14:30 (Europe/Paris)");
    let due = message::guest(&TelegramEvent::PaymentDue, &en, "en").unwrap();
    assert_eq!(
        due.text,
        format!(
            "Done: notifications about this booking will come here.\n\n\
             Your booking awaits payment: Product demo, {w}. Amount: 20.00 TON (≈ $40.00). \
             Pay by {deadline}, otherwise the time is released.",
            w = en.when
        )
    );
    let labels: Vec<&str> = due.buttons.iter().map(|b| b.label.as_str()).collect();
    assert_eq!(labels, ["Pay", "Cancel booking"]);
    en.payment = Some(guest_payment("en", true, true));
    let partial = message::guest(&TelegramEvent::PaymentPartial, &en, "en").unwrap();
    assert_eq!(
        partial.text,
        format!(
            "Part of the payment for Product demo, {w} has arrived: 5.00 of 20.00 TON. \
             Pay the remaining 15.00 TON by {deadline}, otherwise the time is released.",
            w = en.when
        )
    );
    let expired = message::guest(
        &TelegramEvent::PaymentExpired {
            why: crate::telegram::PaymentRelease::Expired,
            partly_paid: true,
        },
        &en,
        "en",
    )
    .unwrap();
    assert_eq!(
        expired.text,
        format!(
            "The time to pay for Product demo, {} has run out, so the time is released. \
             You can book another time. Part of the payment had already arrived: Ilia will \
             contact you about a refund.",
            en.when
        )
    );

    let mut ru = guest_view("ru", "Знакомство");
    ru.payment = Some(guest_payment("ru", true, false));
    let deadline = ru.payment.as_ref().unwrap().deadline.clone();
    let due = message::guest(&TelegramEvent::PaymentDue, &ru, "ru").unwrap();
    assert_eq!(
        due.text,
        format!(
            "Готово: уведомления об этой записи будут приходить сюда.\n\n\
             Запись ждёт оплаты: Знакомство, {w}. К оплате: 20.00 TON (≈ $40.00). \
             Срок оплаты — {deadline}; если оплата не поступит, время освободится.",
            w = ru.when
        )
    );
    assert_eq!(due.buttons[0].label, "Оплатить");
    let received = message::guest(&TelegramEvent::PaymentReceived, &ru, "ru").unwrap();
    assert_eq!(
        received.text,
        format!(
            "Оплата получена, запись подтверждена: Знакомство, {}. Организатор: Ilia.",
            ru.when
        )
    );

    let mut host = host_view("ru");
    host.payment = Some(host_payment());
    let paid = message::host(&TelegramEvent::PaymentReceived, &host, "ru").unwrap();
    assert_eq!(
        paid.text,
        format!(
            "Оплачено: Product demo, {}. Гость: Eve, eve@example.test. \
             Получено 20.00 TON (≈ $40.00), запись подтверждена.",
            host.when
        )
    );
    let busy = message::host(
        &TelegramEvent::PaymentProblem {
            problem: "host_busy".into(),
        },
        &host,
        "ru",
    )
    .unwrap();
    assert_eq!(
        busy.text,
        format!(
            "Оплата требует внимания: Product demo, {}. Гость: Eve, eve@example.test.\n\
             Получено 20.00 из 20.00 TON.\n{}",
            host.when,
            crate::i18n::translate("ru", "ton-problem-host-busy", None)
        )
    );
    // The guest of that booking hears it too, without the host's options.
    let held = message::guest(
        &TelegramEvent::PaymentProblem {
            problem: "host_busy".into(),
        },
        &ru,
        "ru",
    )
    .unwrap();
    assert_eq!(
        held.text,
        format!(
            "Мы получили оплату встречи «Знакомство», {}, но подтвердить запись \
             автоматически не удалось. Ilia свяжется с вами.",
            ru.when
        )
    );
    // An unknown problem says nothing rather than a raw key.
    assert!(message::host(
        &TelegramEvent::PaymentProblem {
            problem: "made_up".into()
        },
        &host,
        "ru"
    )
    .is_none());
}

// --- 12. Integration with TON: review fixes ---

/// A guest's email address is guest text too: calrs accepts anything with
/// an `@` and a dot in the domain, URLs and spaces included.
#[test]
fn a_guest_email_cannot_pose_as_a_link_in_host_payment_messages() {
    for plain in [
        "eve@example.test",
        "first.last+tag@mail.example.co.uk",
        "o_brien-99%x@sub-domain.example",
        "анна@пример.рф",
    ] {
        assert_eq!(text::guest_email(plain), plain);
    }
    let phish =
        "Payment failed, re-verify wallet at https://tonkeeper-verify.example/connect a@x.co";
    let shown = text::guest_email(phish);
    assert_eq!(
        shown,
        "Payment failed, re-verify wallet at https[:]//tonkeeper-verify[.]example/connect a@x[.]co"
    );
    for odd in [
        "https://evil.example@x.co",
        "www.evil.example a@x.co",
        "a@b@x.co",
        "a@x..co",
        "@x.co",
    ] {
        let shown = text::guest_email(odd);
        assert!(!shown.contains("://"), "{shown}");
        assert!(!shown.contains("evil.example"), "{shown}");
        assert!(!shown.contains("x.co"), "{shown}");
    }
    let mut view = host_view("en");
    view.email = shown;
    view.payment = Some(host_payment());
    let mut events = vec![TelegramEvent::PaymentReceived];
    events.extend(
        TelegramEvent::payment_samples()
            .into_iter()
            .filter(|e| e.audience().hosts),
    );
    for event in events {
        let msg = message::host(&event, &view, "en").unwrap();
        assert!(!msg.text.contains("://"), "{event:?}: {}", msg.text);
        assert!(
            !msg.text.contains("tonkeeper-verify.example"),
            "{event:?}: {}",
            msg.text
        );
    }
}

/// A priced booking's hosts hear nothing when the guest links a chat (it
/// awaits its payment), so "paid" says who holds the booking's chat, and
/// whether that is the username the guest typed.
#[test]
fn the_hosts_paid_message_names_the_guests_telegram() {
    let mut v = host_view("en");
    v.payment = Some(host_payment());
    let paid = |v: &HostView, lang: &str| {
        message::host(&TelegramEvent::PaymentReceived, v, lang)
            .unwrap()
            .text
    };
    // No chat linked: nothing about Telegram.
    assert!(!paid(&v, "en").contains("Telegram"), "{}", paid(&v, "en"));
    v.guest_linked = true;
    assert!(
        paid(&v, "en").ends_with(
            "the booking is confirmed.\nGuest's Telegram: @RealName. Entered on the form: @Guest_Name."
        ),
        "{}",
        paid(&v, "en")
    );
    let same = HostView {
        entered: Some("realname".into()),
        ..v.clone()
    };
    assert!(
        paid(&same, "en").ends_with("\nGuest's Telegram: @RealName."),
        "{}",
        paid(&same, "en")
    );
    let no_username = HostView {
        linked_as: None,
        ..v.clone()
    };
    assert!(
        paid(&no_username, "ru")
            .ends_with("\nTelegram гостя: без имени пользователя. В форме указано: @Guest_Name."),
        "{}",
        paid(&no_username, "ru")
    );
    for (lang, _) in crate::i18n::supported_with_labels() {
        let text = paid(&v, lang);
        assert!(!text.contains("tg-"), "{lang}: {text}");
        assert!(text.contains("@RealName"), "{lang}: {text}");
    }
}

/// A late transfer from a sender other than the first payer reaches the
/// hosts only.
#[test]
fn a_late_transfer_from_a_stranger_is_for_the_hosts_only() {
    let stranger = TelegramEvent::PaymentLate {
        value: "1.5".into(),
        to_guest: false,
    };
    let a = stranger.audience();
    assert!(!a.guest && a.hosts);
    assert!(message::guest(&stranger, &guest_view("en", "Product demo"), "en").is_none());
    assert!(message::host(&stranger, &host_view("en"), "en").is_some());
    let own = TelegramEvent::PaymentLate {
        value: "1.5".into(),
        to_guest: true,
    };
    assert!(own.audience().guest);
}

// --- ilia fork: messages with guests ---

#[test]
fn message_events_compose_in_every_locale_for_their_side_only() {
    for (lang, _) in crate::i18n::supported_with_labels() {
        for event in TelegramEvent::message_samples() {
            let audience = event.audience();
            let g = message::guest(&event, &guest_view(lang, "Product demo"), lang);
            let h = message::host(&event, &host_view(lang), lang);
            assert_eq!(g.is_some(), audience.guest, "{lang} {event:?}");
            assert_eq!(h.is_some(), audience.hosts, "{lang} {event:?}");
            let msg = g.or(h).unwrap();
            let what = format!("{lang} {event:?}: {}", msg.text);
            assert!(!msg.text.contains("tg-"), "{what}");
            assert!(msg.text.contains("Product demo"), "{what}");
            assert!(!msg.text.contains("))"), "{what}");
            // The text itself, and a line on how to answer after it.
            let body = match &event {
                TelegramEvent::HostMessage { .. } => "See you on Monday.\nBring the slides.",
                _ => "Can we talk about pricing?",
            };
            assert!(msg.text.contains(&format!("\n\n{body}\n\n")), "{what}");
            // Nothing that carries a booking token, in either direction.
            let all = format!("{} {:?}", msg.text, msg.buttons);
            for t in [
                "/booking/cancel/",
                "/booking/reschedule/",
                "/booking/ics/",
                "/booking/pay/",
            ] {
                assert!(!all.contains(t), "{what}");
            }
        }
    }
}

#[test]
fn exact_english_and_russian_thread_messages() {
    let host = TelegramEvent::HostMessage {
        id: "m".into(),
        text: "See you.".into(),
    };
    let guest = TelegramEvent::GuestMessage {
        id: "m".into(),
        text: "Hi".into(),
        email: None,
    };
    let en = message::guest(&host, &guest_view("en", "Product demo"), "en").unwrap();
    assert_eq!(
        en.text,
        "Message from Ilia about Product demo, Monday, August 17, 2026, 14:00 (Europe/Paris):\n\n\
         See you.\n\nTo answer, write your reply here."
    );
    assert!(en.buttons.is_empty());
    let ru = message::host(&guest, &host_view("ru"), "ru").unwrap();
    assert!(
        ru.text
            .starts_with("Сообщение от гостя Eve (eve@example.test) по записи: Product demo, "),
        "{}",
        ru.text
    );
    assert!(
        ru.text
            .ends_with("\n\nHi\n\nЧтобы ответить гостю, ответьте на это сообщение."),
        "{}",
        ru.text
    );
}

#[test]
fn a_guest_text_is_one_defanged_line_in_the_hosts_chat() {
    let event = TelegramEvent::GuestMessage {
        id: "m".into(),
        text: "Visit https://pay.example/login\n\u{202E}now /start @bot".into(),
        email: None,
    };
    let msg = message::host(&event, &host_view("en"), "en").unwrap();
    assert!(
        msg.text
            .contains("\n\nVisit https[:]//pay[.]example/login now [/]start [@]bot\n\n"),
        "{}",
        msg.text
    );
    assert!(!msg.text.contains('\u{202E}'));
}

#[test]
fn host_text_keeps_its_lines_and_loses_invisible_characters() {
    assert_eq!(
        text::clean_lines("  Hello\r\n\r\n\r\n\u{200B}see\tyou\u{202E}\n\n", 100),
        "Hello\n\nsee you"
    );
    assert_eq!(text::clean_lines("\n \n", 100), "");
    assert_eq!(text::clean_lines("abcdef", 4), "abc…");
}

#[test]
fn a_reply_carries_the_id_of_the_message_it_answers() {
    let update: Update =
        serde_json::from_value(fake::private_message(5, 42, "Yes", Some(1003))).unwrap();
    let message = update.message.unwrap();
    assert_eq!(message.reply_to_message.map(|r| r.message_id), Some(1003));
    let plain: Update = serde_json::from_value(fake::private_message(6, 42, "Yes", None)).unwrap();
    assert!(plain.message.unwrap().reply_to_message.is_none());
}

// --- ilia fork, Optimize: the menu is sent only when it changed ---

/// Start the poller once over `api`, until its first long poll, and stop it.
async fn start_once(pool: &sqlx::SqlitePool, api: Arc<FakeApi>) {
    let (rt, _rx) = Runtime::new(api.clone(), None, fake::fast_retry());
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let task = tokio::spawn(poller::run(
        rt,
        pool.clone(),
        stop_rx,
        PollPolicy::default(),
    ));
    for _ in 0..500 {
        if !api.offsets.lock().unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(!api.offsets.lock().unwrap().is_empty(), "no long poll");
    stop_tx.send(true).unwrap();
    tokio::time::timeout(Duration::from_secs(1), task)
        .await
        .unwrap()
        .unwrap();
}

fn menu_calls(api: &FakeApi) -> (usize, usize) {
    (
        api.commands.lock().unwrap().len(),
        api.descriptions.lock().unwrap().len(),
    )
}

#[tokio::test]
async fn the_menu_is_sent_once_per_text_change_and_bot() {
    let pool = fake::memory_pool().await;
    let langs = poller::menus().len();
    assert_eq!(langs, 10, "the default and nine two-letter languages");
    store::claim_bot(&pool, 123456789).await;

    // First start: 10 setMyCommands and 10 setMyDescription (each two calls
    // on the wire, the long and the short text).
    let api = Arc::new(FakeApi::default());
    start_once(&pool, api.clone()).await;
    assert_eq!(menu_calls(&api), (langs, langs));
    assert_eq!(
        store::menu_hash(&pool).await.as_deref(),
        Some(poller::menu_hash(&poller::menus()).as_str())
    );

    // A restart with the same texts: getMe only.
    let api = Arc::new(FakeApi::default());
    start_once(&pool, api.clone()).await;
    assert_eq!(menu_calls(&api), (0, 0));

    // Changed texts (a new release): sent again. A failed call keeps the
    // old hash, so the next start retries.
    sqlx::query("UPDATE telegram_state SET commands_hash = 'older-release'")
        .execute(&pool)
        .await
        .unwrap();
    let api = Arc::new(FakeApi::default());
    api.fail_menu
        .store(true, std::sync::atomic::Ordering::SeqCst);
    start_once(&pool, api.clone()).await;
    assert_eq!(menu_calls(&api), (langs, langs));
    assert_eq!(
        store::menu_hash(&pool).await.as_deref(),
        Some("older-release")
    );
    let api = Arc::new(FakeApi::default());
    start_once(&pool, api.clone()).await;
    assert_eq!(menu_calls(&api), (langs, langs));

    // Another bot's token starts over.
    assert!(store::claim_bot(&pool, 987654321).await);
    assert_eq!(store::menu_hash(&pool).await, None);
    let api = Arc::new(FakeApi::default());
    start_once(&pool, api.clone()).await;
    assert_eq!(menu_calls(&api), (langs, langs));
}

#[test]
fn the_menu_hash_follows_every_text() {
    let menus = poller::menus();
    let base = poller::menu_hash(&menus);
    assert_eq!(base.len(), 64);
    let mut changed = poller::menus();
    changed[3].short.push('!');
    assert_ne!(poller::menu_hash(&changed), base);
    let mut changed = poller::menus();
    changed[0].commands[0].1.push('!');
    assert_ne!(poller::menu_hash(&changed), base);
    // Moving a character between two fields changes it too.
    let mut a = poller::menus();
    a[1].description.push('x');
    let mut b = poller::menus();
    b[1].short.insert(0, 'x');
    assert_ne!(poller::menu_hash(&a), poller::menu_hash(&b));
}
