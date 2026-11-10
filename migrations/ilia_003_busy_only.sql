-- ilia fork: busy-only calendar storage (src/busy_only.rs). The event cache
-- keeps when the host is busy, never what about: no titles, descriptions,
-- locations, attendees or raw iCal. Recurring rows keep a reduced raw_ical
-- (their EXDATEs and RECURRENCE-IDs only), which busy_only::scrub_stored
-- writes at startup, since SQL cannot parse iCal. With CALRS_BUSY_ONLY=0 the
-- content comes back on each source's next daily full resync.
UPDATE events SET summary = NULL, description = NULL, location = NULL
 WHERE summary IS NOT NULL OR description IS NOT NULL OR location IS NOT NULL;
UPDATE events SET raw_ical = NULL
 WHERE raw_ical IS NOT NULL AND (rrule IS NULL OR rrule = '');
UPDATE resource_events SET summary = NULL WHERE summary IS NOT NULL;
UPDATE resource_events SET raw_ical = NULL
 WHERE raw_ical IS NOT NULL AND (rrule IS NULL OR rrule = '');
