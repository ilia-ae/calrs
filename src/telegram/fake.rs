//! A Bot API that never leaves the process, for tests: it records what would
//! be sent and answers from a script. Also the lock and guard that install a
//! runtime over it as the process-wide one.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use sqlx::SqlitePool;
use tokio::task::JoinHandle;

use super::api::{BotApi, Outgoing, TgError, User};
use super::worker::{self, Job, RetryPolicy};
use super::Runtime;

/// The chat id `Installed::flush` sends its marker to.
const FLUSH_CHAT: i64 = -424242;

#[derive(Default)]
pub(crate) struct FakeApi {
    /// Every accepted message, in order.
    pub(crate) sent: Mutex<Vec<(i64, Outgoing)>>,
    /// The message id of each accepted message, in the order of `sent`.
    pub(crate) ids: Mutex<Vec<i64>>,
    /// Every send attempt, accepted or not.
    pub(crate) attempts: Mutex<Vec<i64>>,
    /// Answers to the next send attempts; empty means accept.
    pub(crate) send_results: Mutex<VecDeque<Result<(), TgError>>>,
    /// Answers to getMe; empty means a bot called `ilia_test_bot`.
    pub(crate) get_me_results: Mutex<VecDeque<Result<User, TgError>>>,
    /// Answers to getUpdates; empty means wait forever (a quiet long poll).
    pub(crate) updates: Mutex<VecDeque<Result<Vec<serde_json::Value>, TgError>>>,
    /// The offsets getUpdates was called with.
    pub(crate) offsets: Mutex<Vec<i64>>,
    /// setMyCommands calls: (language, command names).
    pub(crate) commands: Mutex<Vec<(Option<String>, Vec<String>)>>,
    /// setMyDescription/setMyShortDescription calls: (language, description,
    /// short description).
    pub(crate) descriptions: Mutex<Vec<(Option<String>, String, String)>>,
    /// setMyCommands answers 500 while set.
    pub(crate) fail_menu: std::sync::atomic::AtomicBool,
}

impl FakeApi {
    pub(crate) fn script_send(&self, results: Vec<Result<(), TgError>>) {
        self.send_results.lock().unwrap().extend(results);
    }

    pub(crate) fn sent(&self) -> Vec<(i64, Outgoing)> {
        self.sent.lock().unwrap().clone()
    }

    /// The texts sent to one chat.
    pub(crate) fn texts_to(&self, chat_id: i64) -> Vec<String> {
        self.sent()
            .into_iter()
            .filter(|(c, _)| *c == chat_id)
            .map(|(_, m)| m.text)
            .collect()
    }

    /// The id of the last message sent to `chat_id`.
    pub(crate) fn last_id_to(&self, chat_id: i64) -> i64 {
        let sent = self.sent();
        let ids = self.ids.lock().unwrap().clone();
        sent.iter()
            .zip(ids)
            .filter(|((c, _), _)| *c == chat_id)
            .map(|(_, id)| id)
            .next_back()
            .unwrap_or_else(|| panic!("nothing sent to {chat_id}"))
    }

    /// Wait (at most 5 s) until a message to `chat_id` satisfies `pred`.
    pub(crate) async fn wait_for(
        &self,
        chat_id: i64,
        pred: impl Fn(&Outgoing) -> bool,
    ) -> Outgoing {
        for _ in 0..500 {
            if let Some((_, m)) = self
                .sent()
                .into_iter()
                .find(|(c, m)| *c == chat_id && pred(m))
            {
                return m;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("no such message to {chat_id}; sent: {:#?}", self.sent());
    }
}

#[async_trait]
impl BotApi for FakeApi {
    async fn get_me(&self) -> Result<User, TgError> {
        let next = self.get_me_results.lock().unwrap().pop_front();
        next.unwrap_or_else(|| {
            Ok(User {
                id: 123456789,
                is_bot: true,
                first_name: "Ilia".into(),
                username: Some("ilia_test_bot".into()),
                language_code: None,
            })
        })
    }

    async fn get_updates(&self, offset: i64) -> Result<Vec<serde_json::Value>, TgError> {
        self.offsets.lock().unwrap().push(offset);
        let next = self.updates.lock().unwrap().pop_front();
        match next {
            Some(result) => result,
            None => std::future::pending().await,
        }
    }

    async fn send_message(&self, chat_id: i64, msg: &Outgoing) -> Result<i64, TgError> {
        self.attempts.lock().unwrap().push(chat_id);
        let next = self.send_results.lock().unwrap().pop_front();
        next.unwrap_or(Ok(()))?;
        // Ids in order, under the lock of `sent`, so `ids` lines up with it.
        let mut sent = self.sent.lock().unwrap();
        let id = 1000 + sent.len() as i64;
        sent.push((chat_id, msg.clone()));
        self.ids.lock().unwrap().push(id);
        Ok(id)
    }

    async fn set_my_commands(
        &self,
        commands: &[(String, String)],
        language_code: Option<&str>,
    ) -> Result<(), TgError> {
        self.commands.lock().unwrap().push((
            language_code.map(str::to_string),
            commands.iter().map(|(c, _)| c.clone()).collect(),
        ));
        if self.fail_menu.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(TgError::Server(500));
        }
        Ok(())
    }

    async fn set_my_description(
        &self,
        description: &str,
        short_description: &str,
        language_code: Option<&str>,
    ) -> Result<(), TgError> {
        self.descriptions.lock().unwrap().push((
            language_code.map(str::to_string),
            description.to_string(),
            short_description.to_string(),
        ));
        Ok(())
    }
}

/// Retries in milliseconds rather than seconds.
pub(crate) fn fast_retry() -> RetryPolicy {
    RetryPolicy {
        delays: [Duration::from_millis(10), Duration::from_millis(10)],
        retry_after_unit: Duration::from_millis(10),
        max_retry_after: Duration::from_secs(1),
    }
}

/// Serialises the tests that install the process-wide runtime.
pub(crate) static TELEGRAM_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// A fake runtime installed as the process-wide one, with its worker running
/// on the test's runtime. Uninstalled when dropped.
pub(crate) struct Installed {
    pub(crate) api: Arc<FakeApi>,
    pub(crate) rt: Arc<Runtime>,
    worker: JoinHandle<()>,
    _lock: tokio::sync::MutexGuard<'static, ()>,
}

impl Installed {
    /// Wait until everything queued so far has been handled.
    pub(crate) async fn flush(&self, pool: &SqlitePool) {
        let before = self.api.texts_to(FLUSH_CHAT).len();
        self.rt.enqueue(Job::Reply {
            pool: pool.clone(),
            chat_id: FLUSH_CHAT,
            msg: Outgoing {
                text: "flush".into(),
                buttons: Vec::new(),
            },
            what: "flush",
            attempts: 1,
        });
        for _ in 0..500 {
            if self.api.texts_to(FLUSH_CHAT).len() > before {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("the telegram worker did not drain");
    }

    /// Messages sent, without the flush markers.
    pub(crate) fn sent(&self) -> Vec<(i64, Outgoing)> {
        self.api
            .sent()
            .into_iter()
            .filter(|(c, _)| *c != FLUSH_CHAT)
            .collect()
    }
}

impl Drop for Installed {
    fn drop(&mut self) {
        super::install(None);
        self.worker.abort();
    }
}

/// Install a fake bot called `bot` as the process-wide runtime.
pub(crate) async fn install_fake(bot: &str) -> Installed {
    let lock = TELEGRAM_LOCK.lock().await;
    let api = Arc::new(FakeApi::default());
    let (rt, rx) = Runtime::new(api.clone(), Some(bot.to_string()), fast_retry());
    super::install(Some(rt.clone()));
    let worker = tokio::spawn(worker::run(rt.clone(), rx));
    Installed {
        api,
        rt,
        worker,
        _lock: lock,
    }
}

/// An in-memory database with every migration applied.
pub(crate) async fn memory_pool() -> SqlitePool {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            SqliteConnectOptions::from_str("sqlite::memory:")
                .unwrap()
                .foreign_keys(true),
        )
        .await
        .unwrap();
    crate::db::migrate(&pool).await.unwrap();
    pool
}

/// A private-chat text from a person, replying to message `reply_to` of the
/// bot (`None`: not a reply).
pub(crate) fn private_message(
    update_id: i64,
    chat_id: i64,
    text: &str,
    reply_to: Option<i64>,
) -> serde_json::Value {
    let mut update = private_text(update_id, chat_id, None, text);
    let message = &mut update["message"];
    message.as_object_mut().unwrap().remove("entities");
    if let Some(id) = reply_to {
        message["reply_to_message"] = serde_json::json!({
            "message_id": id,
            "from": {"id": 123456789, "is_bot": true, "first_name": "Ilia"},
            "chat": {"id": chat_id, "type": "private"},
            "date": 1762700000,
            "text": "earlier"
        });
    }
    update
}

/// A private-chat update from a person, as Telegram sends it.
pub(crate) fn private_text(
    update_id: i64,
    chat_id: i64,
    username: Option<&str>,
    text: &str,
) -> serde_json::Value {
    let mut from = serde_json::json!({
        "id": chat_id, "is_bot": false, "first_name": "Guest", "language_code": "en"
    });
    if let Some(u) = username {
        from["username"] = serde_json::json!(u);
    }
    serde_json::json!({
        "update_id": update_id,
        "message": {
            "message_id": update_id * 10,
            "from": from,
            "chat": {"id": chat_id, "first_name": "Guest", "type": "private"},
            "date": 1762700000,
            "text": text,
            "entities": [{"offset": 0, "length": 6, "type": "bot_command"}]
        }
    })
}
