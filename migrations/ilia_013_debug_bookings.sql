-- ilia fork: bookings made from the admin's debug booking page
-- (/dashboard/admin/debug-booking, src/web/debug_booking.rs; on only while
-- its switch is on: the Debug block of /dashboard/admin/ton, or
-- CALRS_DEBUG_BOOKING). Each row marks one real booking, made through
-- the public booking path with the admin's own details, as a test, so the
-- page can list them and cancel them all at once. The booking itself is an
-- ordinary row of `bookings`: nothing here changes how it is priced, paid,
-- held or confirmed. `telegram_own_chat` records whether its guest messages
-- were routed to the admin's own linked chat (never the chat id).
CREATE TABLE debug_bookings (
    booking_id        TEXT PRIMARY KEY REFERENCES bookings(id) ON DELETE CASCADE,
    user_id           TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    telegram_own_chat INTEGER NOT NULL DEFAULT 0 CHECK (telegram_own_chat IN (0, 1)),
    created_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_debug_bookings_user ON debug_bookings (user_id, created_at);
