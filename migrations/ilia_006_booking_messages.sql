-- ilia fork: host <-> guest messages, one thread per booking (ilia-booking
-- docs/operations.md, «Сообщения гостям»). The host writes from the booking
-- in the dashboard, through the MCP agent, or by replying in Telegram; the
-- guest gets it by email (Reply-To: the host) and in their linked Telegram
-- chat. A guest's text in their linked chat is stored here and relayed to
-- the host's chat. The thread goes with its booking (ON DELETE CASCADE).
CREATE TABLE booking_messages (
    id              TEXT PRIMARY KEY,
    booking_id      TEXT NOT NULL REFERENCES bookings(id) ON DELETE CASCADE,
    direction       TEXT NOT NULL CHECK (direction IN ('host', 'guest')),
    -- The user who wrote a host message; NULL for guest messages.
    author_user_id  TEXT REFERENCES users(id) ON DELETE SET NULL,
    via             TEXT NOT NULL CHECK (via IN ('dashboard', 'mcp', 'telegram')),
    body            TEXT NOT NULL CHECK (length(body) BETWEEN 1 AND 2000),
    email_status    TEXT NOT NULL DEFAULT 'none'
                    CHECK (email_status IN ('none', 'queued', 'sent', 'failed')),
    telegram_status TEXT NOT NULL DEFAULT 'none'
                    CHECK (telegram_status IN ('none', 'queued', 'sent', 'failed')),
    -- UTC 'YYYY-MM-DD HH:MM:SS', as every Telegram timestamp.
    created_at      TEXT NOT NULL
);
-- The thread of a booking, and the per-booking limits.
CREATE INDEX idx_booking_messages_booking ON booking_messages (booking_id, created_at);
-- The per-user limit on host messages.
CREATE INDEX idx_booking_messages_author ON booking_messages (author_user_id, created_at)
    WHERE author_user_id IS NOT NULL;

-- Which booking a bot message in a chat is about, so that a reply to it
-- picks the booking: a guest's reply in their chat, or a host's reply in
-- theirs (side). chat_id is never shown, logged or given to the agent.
-- Purged after 30 days, and with the chat's links (/stop, a block, unlink).
CREATE TABLE telegram_message_map (
    chat_id    INTEGER NOT NULL,
    message_id INTEGER NOT NULL,
    booking_id TEXT NOT NULL REFERENCES bookings(id) ON DELETE CASCADE,
    side       TEXT NOT NULL CHECK (side IN ('guest', 'host')),
    created_at TEXT NOT NULL,
    PRIMARY KEY (chat_id, message_id)
);
CREATE INDEX idx_telegram_message_map_created ON telegram_message_map (created_at);
CREATE INDEX idx_telegram_message_map_booking ON telegram_message_map (booking_id);
