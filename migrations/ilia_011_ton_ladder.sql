-- ilia fork: the demand ladder (ilia-booking pricing design §1). A personal
-- event type is free (no row), at a fixed price (TON or USD, as before), or
-- priced by the site-wide ladder: the price is step n of the ladder, n the
-- owner's live bookings ahead. A ladder row is always TON and stores no
-- amount; `enabled` keeps its meaning (0: the type books free).
--
-- ton_prices is rebuilt as in ilia_004 (nothing references it); every
-- existing price stays a fixed one.
CREATE TABLE ton_prices_new (
    event_type_id   TEXT PRIMARY KEY REFERENCES event_types(id) ON DELETE CASCADE,
    mode            TEXT NOT NULL DEFAULT 'fixed' CHECK (mode IN ('fixed', 'ladder')),
    currency        TEXT NOT NULL DEFAULT 'TON' CHECK (currency IN ('TON', 'USD')),
    price_nano      INTEGER CHECK (price_nano IS NULL
                                   OR (price_nano BETWEEN 1000000000 AND 100000000000000
                                       AND price_nano % 10000000 = 0)),
    price_usd_cents INTEGER CHECK (price_usd_cents IS NULL OR price_usd_cents BETWEEN 100 AND 1000000),
    window_min      INTEGER CHECK (window_min IS NULL OR window_min BETWEEN 10 AND 240),
    enabled         INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    updated_at      TEXT NOT NULL,
    CHECK (mode = 'ladder' OR ((currency = 'TON') = (price_nano IS NOT NULL))),
    CHECK (mode = 'ladder' OR ((currency = 'USD') = (price_usd_cents IS NOT NULL))),
    CHECK (mode = 'fixed' OR (currency = 'TON' AND price_nano IS NULL AND price_usd_cents IS NULL))
);
INSERT INTO ton_prices_new (event_type_id, mode, currency, price_nano, price_usd_cents, window_min, enabled, updated_at)
    SELECT event_type_id, 'fixed', currency, price_nano, price_usd_cents, window_min, enabled, updated_at FROM ton_prices;
DROP TABLE ton_prices;
ALTER TABLE ton_prices_new RENAME TO ton_prices;

-- What an invoice was priced by: the ladder's n when it opened, and the
-- price the guest was shown (never charged: the amount is computed on the
-- server, and is at most this).
ALTER TABLE ton_invoices ADD COLUMN pricing_mode TEXT NOT NULL DEFAULT 'fixed'
    CHECK (pricing_mode IN ('fixed', 'ladder'));
ALTER TABLE ton_invoices ADD COLUMN demand_n INTEGER;
ALTER TABLE ton_invoices ADD COLUMN quoted_nano INTEGER;

-- TON settings that may come from the database (src/ton/settings.rs): the
-- environment wins, then this one value per key with who wrote it, then
-- the default. Values are in the environment's text format; keys are the
-- variable names. IF NOT EXISTS: the TON settings page uses the same table.
CREATE TABLE IF NOT EXISTS ton_settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    owner      TEXT NOT NULL DEFAULT 'panel' CHECK (owner IN ('panel', 'apply')),
    updated_at TEXT NOT NULL,
    updated_by TEXT
);

-- Free ladder steps taken, for the one-per-guest limit: by email, by the
-- typed Telegram username and by client network (the invoices' HMAC tag).
-- A cancelled free booking keeps its row, so cancelling and booking again
-- does not keep a free step. Rows go after the longest limit (365 days).
CREATE TABLE ton_free_bookings (
    booking_id     TEXT PRIMARY KEY,
    event_type_id  TEXT NOT NULL,
    email_norm     TEXT NOT NULL,
    telegram_norm  TEXT,
    client_tag     TEXT,
    demand_n       INTEGER NOT NULL,
    created_at     TEXT NOT NULL
);
CREATE INDEX idx_ton_free_bookings_created ON ton_free_bookings (created_at);
