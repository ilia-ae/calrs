-- ilia fork, Optimize stage (ilia-booking docs/operations.md).
--
-- The Telegram bot sets its command menu and descriptions (30 Bot API calls)
-- only when their texts changed: the hash of what this bot last took in full.
-- Reset with the bot (store::claim_bot).
ALTER TABLE telegram_state ADD COLUMN commands_hash TEXT;

-- Hot-path indexes, each kept because EXPLAIN QUERY PLAN on the seeded
-- database (src/web/tests/ton_perf.rs) showed a full SCAN of a growing table
-- turn into a SEARCH, and the timing improved.
--
-- The watcher, every step (10 s while an invoice is open): the open and paid
-- invoices (watched addresses, confirmations, expiry by created_at, the
-- reconcile pass) instead of every invoice ever issued.
CREATE INDEX idx_ton_invoices_status ON ton_invoices (status, created_at);
-- Its alert passes: invoice problems not yet told, and paid invoices whose
-- "payment received" is not sent yet (both nearly always empty; the second
-- is named by store::due_received_notices, INDEXED BY).
CREATE INDEX idx_ton_invoices_problem ON ton_invoices (created_at) WHERE problem IS NOT NULL;
CREATE INDEX idx_ton_invoices_paid_notice ON ton_invoices (created_at)
    WHERE status = 'confirmed' AND paid_notice_sent_at IS NULL;
-- The slot page's recurring busy query: only the recurring masters, instead
-- of every event that starts before the window ends (18 months of history).
-- An (calendar_id, end_at) index for the single events was tried and not
-- chosen by the planner, so it is not here.
CREATE INDEX idx_events_recurring ON events (calendar_id) WHERE rrule IS NOT NULL AND rrule != '';
