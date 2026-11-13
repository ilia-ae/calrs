-- ilia fork: 3CX video meetings for bookings (src/web/threecx.rs).
--
-- The site's 3CX settings, edited on the admin page (or by `calrs config
-- apply`): the PBX's base URL, the video conference API key (AES-256-GCM,
-- like the other secrets of auth_config), the extension that organises
-- the meetings, and the label guests see.
ALTER TABLE auth_config ADD COLUMN threecx_base_url TEXT;
ALTER TABLE auth_config ADD COLUMN threecx_api_key_enc TEXT;
ALTER TABLE auth_config ADD COLUMN threecx_extension TEXT;
ALTER TABLE auth_config ADD COLUMN threecx_display_name TEXT;

-- One 3CX meeting per booking. No foreign key on purpose: when a booking
-- row goes (a deleted user or event type), the sweep still finds its
-- meeting and deletes it on the PBX. `host_link_enc` is the organizer's
-- join link (host rights), encrypted; guests get the meeting's open link
-- in bookings.meeting_url. `start_at` (RFC 3339 UTC) and `duration_min`
-- are what the PBX holds, so the sweep can tell a moved booking.
CREATE TABLE ilia_threecx_meetings (
    booking_id TEXT PRIMARY KEY,
    meeting_id TEXT NOT NULL,
    host_link_enc TEXT NOT NULL,
    start_at TEXT NOT NULL,
    duration_min INTEGER NOT NULL,
    subject TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    attempts INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TEXT,
    last_error TEXT
);
