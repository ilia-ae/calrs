-- ilia fork, Inbox stage (ilia-booking docs/decisions.md, 2026-11-10): the
-- system mailbox. A guest's email reply is stored in the booking's thread as
-- via = 'email'; `inbound_hash` (SHA-256 of the normalised Message-ID) keeps
-- a message polled twice, or delivered to two aliases, from being stored
-- twice. SQLite cannot change a CHECK in place, so the table is rebuilt; no
-- foreign key points at it.
CREATE TABLE booking_messages_new (
    id              TEXT PRIMARY KEY,
    booking_id      TEXT NOT NULL REFERENCES bookings(id) ON DELETE CASCADE,
    direction       TEXT NOT NULL CHECK (direction IN ('host', 'guest')),
    -- The user who wrote a host message; NULL for guest messages.
    author_user_id  TEXT REFERENCES users(id) ON DELETE SET NULL,
    via             TEXT NOT NULL CHECK (via IN ('dashboard', 'mcp', 'telegram', 'email')),
    body            TEXT NOT NULL CHECK (length(body) BETWEEN 1 AND 2000),
    email_status    TEXT NOT NULL DEFAULT 'none'
                    CHECK (email_status IN ('none', 'queued', 'sent', 'failed')),
    telegram_status TEXT NOT NULL DEFAULT 'none'
                    CHECK (telegram_status IN ('none', 'queued', 'sent', 'failed')),
    -- UTC 'YYYY-MM-DD HH:MM:SS', as every Telegram timestamp.
    created_at      TEXT NOT NULL,
    inbound_hash    TEXT,
    -- Who sent an email reply (via = 'email'): the booking's guest, one of
    -- its additional attendees, or another address that knew the booking's
    -- reply token (a forward, a CC). NULL for every other row.
    sender          TEXT CHECK (sender IN ('guest', 'attendee', 'other')),
    -- The From address of an attendee's or another sender's email, shown to
    -- the host so a forwarded thread is never taken for the guest.
    sender_email    TEXT
);
INSERT INTO booking_messages_new
       (id, booking_id, direction, author_user_id, via, body, email_status,
        telegram_status, created_at)
SELECT id, booking_id, direction, author_user_id, via, body, email_status,
       telegram_status, created_at
  FROM booking_messages;
DROP TABLE booking_messages;
ALTER TABLE booking_messages_new RENAME TO booking_messages;
CREATE INDEX idx_booking_messages_booking ON booking_messages (booking_id, created_at);
CREATE INDEX idx_booking_messages_author ON booking_messages (author_user_id, created_at)
    WHERE author_user_id IS NOT NULL;
CREATE UNIQUE INDEX idx_booking_messages_inbound ON booking_messages (booking_id, inbound_hash)
    WHERE inbound_hash IS NOT NULL;

-- Where the poller stopped in each folder: UIDs are only comparable within
-- one UIDVALIDITY.
CREATE TABLE inbox_state (
    folder      TEXT PRIMARY KEY,
    uidvalidity INTEGER NOT NULL,
    last_uid    INTEGER NOT NULL,
    updated_at  TEXT NOT NULL
);

-- The ORGANIZER of the guest's invites, kept for the booking's life:
-- 'host' for bookings that existed before guest mail went through the
-- system (their invites named the host, and a later update or CANCEL must
-- name the same organizer, or Outlook and Gmail ignore it or file it as a
-- second event); NULL for bookings created since (the system address).
ALTER TABLE bookings ADD COLUMN guest_organizer TEXT;
UPDATE bookings SET guest_organizer = 'host';
