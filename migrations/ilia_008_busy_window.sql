-- ilia fork, Boundaries stage (ilia-booking docs/decisions.md, 2026-11-10).
--
-- The CalDAV resource each cached event came from, so a delta deletion is
-- matched by href (Kerio's hrefs are `<uuid>-00000001.eml`, not `{uid}.ics`)
-- and the daily resync asks only for resources whose etag changed. `etag`
-- exists since 001 and was never written; it is now. NULL for EWS and for
-- rows synced before this migration (they get one on their next fetch).
ALTER TABLE events ADD COLUMN href TEXT;
CREATE INDEX idx_events_calendar_href ON events (calendar_id, href) WHERE href IS NOT NULL;

-- UTC `YYYYMMDDTHHMMSSZ` up to which this calendar's cache is complete (the
-- end of the last full fetch's time range). An unchanged calendar is skipped
-- only while this still covers the horizon, so events sliding into it are
-- read even when the ctag does not move.
ALTER TABLE calendars ADD COLUMN synced_until TEXT;

-- The TZID parameter of a row's RECURRENCE-ID (the value is stored bare in
-- `recurrence_id`). A moved instance stored as a resource of its own (Kerio)
-- excludes its master's original slot by this value, and the master may be
-- in another zone; NULL: none, or synced before this migration.
ALTER TABLE events ADD COLUMN recurrence_tzid TEXT;
