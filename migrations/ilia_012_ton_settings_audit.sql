-- ilia fork: who changed which TON setting (src/web/ton_pay/settings.rs):
-- the admin page (/dashboard/admin/ton) and `calrs config apply` (the `ton`
-- section). One row per key changed; `before`/`after` in the environment's
-- text format, NULL for "no stored value" (the default). `owner` is the
-- row's owner after the change (NULL: the row was removed). The network,
-- the address and the API keys are never stored, so never appear here.
-- Rows older than a year are removed at each write.
CREATE TABLE ton_settings_audit (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    at            TEXT NOT NULL,
    actor_user_id TEXT,
    source        TEXT NOT NULL CHECK (source IN ('panel', 'apply')),
    key           TEXT NOT NULL,
    before        TEXT,
    after         TEXT,
    owner         TEXT CHECK (owner IS NULL OR owner IN ('panel', 'apply'))
);
CREATE INDEX idx_ton_settings_audit_at ON ton_settings_audit (at);
