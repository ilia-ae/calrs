-- Telegram notifications (ilia fork; ilia-booking docs/operations.md, «Уведомления в Telegram»).
-- The bot token is never stored: it comes from CALRS_TELEGRAM_BOT_TOKEN only. Deep-link
-- tokens are stored as SHA-256 hashes. chat_id identifies a person's private chat and is
-- never shown anywhere (dashboard, agent, logs).

-- Guest Telegram field per event type: 'off' (default, no field, no link), 'optional', 'required'.
CREATE TABLE telegram_event_types (
    event_type_id TEXT PRIMARY KEY REFERENCES event_types(id) ON DELETE CASCADE,
    guest_mode    TEXT NOT NULL DEFAULT 'off' CHECK (guest_mode IN ('off', 'optional', 'required')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- One row per booking whose event type offers Telegram.
CREATE TABLE telegram_guest_links (
    booking_id        TEXT PRIMARY KEY REFERENCES bookings(id) ON DELETE CASCADE,
    entered_username  TEXT,              -- typed on the form, normalised (no '@'); NULL when left empty
    link_token_hash   TEXT UNIQUE,       -- hex SHA-256 of the 'b_' deep-link token; NULL once the booking ended
    link_expires_at   TEXT NOT NULL,     -- UTC 'YYYY-MM-DD HH:MM:SS': the meeting's end
    chat_id           INTEGER,           -- private chat linked now; NULL until linked and after /stop
    token_chat_id     INTEGER,           -- the one chat that used the token: only it may link again (after /stop)
    telegram_username TEXT,              -- username Telegram reported at link time (may differ, may be NULL)
    linked_at         TEXT,
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    -- TON: the hash this row's derived token had before the secret key changed (the payment page
    -- re-derives it); a /start with it gets "open the payment page again", not "invalid". NULL otherwise.
    replaced_token_hash TEXT
);
CREATE INDEX idx_telegram_guest_links_chat ON telegram_guest_links(chat_id) WHERE chat_id IS NOT NULL;
CREATE INDEX idx_telegram_guest_links_replaced ON telegram_guest_links(replaced_token_hash)
    WHERE replaced_token_hash IS NOT NULL;

-- Host linking from Profile & Settings: single-use, 15 minutes.
CREATE TABLE telegram_user_link_tokens (
    token_hash TEXT PRIMARY KEY,         -- hex SHA-256 of the 'u_' token
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TEXT NOT NULL,            -- UTC 'YYYY-MM-DD HH:MM:SS'
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_telegram_user_link_tokens_user ON telegram_user_link_tokens(user_id);

-- A host's own chat for host notifications; one per user, relinking replaces it.
CREATE TABLE telegram_user_chats (
    user_id           TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    chat_id           INTEGER NOT NULL,
    telegram_username TEXT,
    linked_at         TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_telegram_user_chats_chat ON telegram_user_chats(chat_id);

-- Poller state: the getUpdates offset survives restarts (no update handled twice); bot username cache.
-- Both belong to one bot: bot_id (the token's numeric prefix, public) resets them when the token
-- is for another bot.
CREATE TABLE telegram_state (
    id            TEXT PRIMARY KEY CHECK (id = 'singleton'),
    bot_id        INTEGER,
    update_offset INTEGER NOT NULL DEFAULT 0,
    bot_username  TEXT,
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO telegram_state (id) VALUES ('singleton');
