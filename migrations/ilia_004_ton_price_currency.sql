-- ilia fork: prices in TON (the default) or in USD (ilia-booking docs/ton.md).
-- A TON price is the invoice amount itself: no rate, no rate request. A USD
-- price is converted at the rate of the moment the guest books, as before.
-- Every existing price was in USD and stays so.
--
-- ton_prices is rebuilt (nothing references it); ton_invoices only gains a
-- column, since ton_payments references it. A TON invoice stores the neutral
-- price_usd_cents = 0, rate_usd_micro = 1, rate_sources = '', rate_at =
-- created_at, which nothing reads when currency = 'TON'.
CREATE TABLE ton_prices_new (
    event_type_id   TEXT PRIMARY KEY REFERENCES event_types(id) ON DELETE CASCADE,
    currency        TEXT NOT NULL DEFAULT 'TON' CHECK (currency IN ('TON', 'USD')),
    price_nano      INTEGER CHECK (price_nano IS NULL
                                   OR (price_nano BETWEEN 1000000000 AND 100000000000000
                                       AND price_nano % 10000000 = 0)),
    price_usd_cents INTEGER CHECK (price_usd_cents IS NULL OR price_usd_cents BETWEEN 100 AND 1000000),
    window_min      INTEGER CHECK (window_min IS NULL OR window_min BETWEEN 10 AND 240),
    enabled         INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    updated_at      TEXT NOT NULL,
    CHECK ((currency = 'TON') = (price_nano IS NOT NULL)),
    CHECK ((currency = 'USD') = (price_usd_cents IS NOT NULL))
);
INSERT INTO ton_prices_new (event_type_id, currency, price_nano, price_usd_cents, window_min, enabled, updated_at)
    SELECT event_type_id, 'USD', NULL, price_usd_cents, window_min, enabled, updated_at FROM ton_prices;
DROP TABLE ton_prices;
ALTER TABLE ton_prices_new RENAME TO ton_prices;

ALTER TABLE ton_invoices ADD COLUMN currency TEXT NOT NULL DEFAULT 'USD'
    CHECK (currency IN ('USD', 'TON'));

-- Priced types ask for the guest's Telegram unless the owner chose a mode.
-- Only types that take payment: a price switched off books for free.
INSERT OR IGNORE INTO telegram_event_types (event_type_id, guest_mode, updated_at)
    SELECT event_type_id, 'required', datetime('now') FROM ton_prices WHERE enabled = 1;
