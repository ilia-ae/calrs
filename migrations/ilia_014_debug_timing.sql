-- ilia fork: a test booking's own invoice timing (the admin's debug booking
-- page, src/web/debug_booking.rs; "Timing for this test booking").
-- `debug_timing` is NULL for every invoice but one opened by that page with
-- a fast or custom timing: then it holds `crate::ton::deadlines::DebugTiming`
-- as text (`window=3,renew_window=3,renewals=1,grace=1,margin=1,fuse=10`),
-- which replaces the site's window, grace, fuse, renewal rules and expiry
-- margin for that invoice only. Only that page writes it (never a guest's
-- form), and the watcher honours it only while the booking is marked in
-- `debug_bookings`. Everything else about the invoice is ordinary.
ALTER TABLE ton_invoices ADD COLUMN debug_timing TEXT;
