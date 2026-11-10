-- ilia fork: TON payments (ilia-booking docs/ton.md). Own tables only: the
-- event_types and bookings schemas are not touched.

-- The price of a personal event type: the ONLY thing that makes a type paid.
-- requires_confirmation and pending_blocks_slot are never consulted for it.
CREATE TABLE ton_prices (
    event_type_id   TEXT PRIMARY KEY REFERENCES event_types(id) ON DELETE CASCADE,
    price_usd_cents INTEGER NOT NULL CHECK (price_usd_cents BETWEEN 100 AND 1000000),
    window_min      INTEGER CHECK (window_min IS NULL OR window_min BETWEEN 10 AND 240),
    enabled         INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    updated_at      TEXT NOT NULL
);

-- One invoice per booking. No foreign key to bookings on purpose: deleting an
-- event type (its bookings cascade) must not erase the money record, so what
-- is needed is snapshotted here.
CREATE TABLE ton_invoices (
    id               TEXT PRIMARY KEY,
    booking_id       TEXT NOT NULL UNIQUE,
    event_type_id    TEXT NOT NULL,
    pay_token        TEXT NOT NULL UNIQUE,          -- 128 bits, the payment page link (secret)
    reference        TEXT NOT NULL UNIQUE,          -- normalised code: 'B' + 10 Crockford characters
    network          TEXT NOT NULL CHECK (network IN ('mainnet', 'testnet')),
    address_raw      TEXT NOT NULL,                 -- '0:HEX', the address at invoice time
    event_title      TEXT NOT NULL,
    start_at_utc     TEXT NOT NULL,                 -- snapshot; a reschedule does not change it
    guest_email_norm TEXT NOT NULL,
    client_tag       TEXT,                          -- HMAC(secret key, IPv4 /24 or IPv6 /64), per-network limits; NULL after 48 h
    price_usd_cents  INTEGER NOT NULL,
    rate_usd_micro   INTEGER NOT NULL CHECK (rate_usd_micro > 0),
    rate_sources     TEXT NOT NULL,                 -- 'coingecko=1.038000;binance=1.038000'
    rate_at          TEXT NOT NULL,
    amount_nano      INTEGER NOT NULL CHECK (amount_nano > 0),
    tolerance_nano   INTEGER NOT NULL DEFAULT 0 CHECK (tolerance_nano >= 0),
    paid_nano        INTEGER NOT NULL DEFAULT 0 CHECK (paid_nano >= 0),
    excess_nano      INTEGER NOT NULL DEFAULT 0 CHECK (excess_nano >= 0),
    status           TEXT NOT NULL DEFAULT 'open'
                     CHECK (status IN ('open', 'paid', 'confirmed', 'expired', 'closed', 'waived')),
    problem          TEXT CHECK (problem IS NULL OR problem IN
                     ('host_busy', 'partial', 'overpaid', 'refund_due', 'confirmed_unpaid', 'address_revoked',
                      'host_busy_unpaid', 'confirm_failed')),
    created_at       TEXT NOT NULL,
    expires_at       TEXT NOT NULL,                 -- the page countdown and exp= in the link
    settle_by        TEXT NOT NULL,                 -- expires_at + grace
    hard_deadline    TEXT NOT NULL,                 -- = bookings.pending_expires_at at creation
    link_emailed_at  TEXT,
    guest_expiry_sent_at TEXT,
    paid_at          TEXT,
    confirmed_at     TEXT,
    closed_at        TEXT,
    waived_by        TEXT,                          -- 'dashboard:<user_id>' | 'mcp:<user_id>'
    owner_alerted_problem TEXT,                     -- the last problem the owner was told about
    owner_alerted_at TEXT,
    paid_notice_sent_at TEXT,                       -- "payment received" claimed
    confirm_attempts INTEGER NOT NULL DEFAULT 0     -- paid, confirmation failed for a reason other than a busy host
);
CREATE INDEX idx_ton_invoices_active ON ton_invoices (status) WHERE status IN ('open', 'paid', 'waived');
CREATE INDEX idx_ton_invoices_email ON ton_invoices (guest_email_norm) WHERE status = 'open';
CREATE INDEX idx_ton_invoices_tag ON ton_invoices (client_tag, created_at);
CREATE INDEX idx_ton_invoices_created ON ton_invoices (created_at);

-- Every incoming transaction accounted for. The primary key on the
-- transaction hash is the double-credit guard; (network, address, lt) too.
-- Column names avoid "hash": the hashes are public and are shown.
CREATE TABLE ton_payments (
    tx_hex           TEXT PRIMARY KEY,              -- lowercase hex
    network          TEXT NOT NULL,
    address_raw      TEXT NOT NULL,
    lt               INTEGER NOT NULL,
    utime            INTEGER NOT NULL,
    mc_block_seqno   INTEGER NOT NULL,
    source_raw       TEXT,
    source_friendly  TEXT,
    source_is_wallet INTEGER,
    value_nano       INTEGER NOT NULL CHECK (value_nano >= 0),
    comment          TEXT,                          -- cleaned, <= 128 chars; SENDER TEXT, never in emails or guest pages
    invoice_id       TEXT REFERENCES ton_invoices(id),
    outcome          TEXT NOT NULL CHECK (outcome IN ('credited', 'overpaid', 'late', 'unmatched', 'ignored')),
    reason           TEXT,                          -- bounced, failed, not_text, before_invoice, other_address, unparsed
    linked_by        TEXT CHECK (linked_by IS NULL OR linked_by IN ('auto', 'link_payment')),
    alert_due        INTEGER NOT NULL DEFAULT 0,
    owner_alerted_at TEXT,
    alert_held       INTEGER NOT NULL DEFAULT 0,    -- claimed into the hourly digest instead of its own email
    refund_tx_hex    TEXT,
    refund_to_raw    TEXT,
    refunded_at      TEXT,
    seen_at          TEXT NOT NULL,
    UNIQUE (network, address_raw, lt)
);
CREATE INDEX idx_ton_payments_invoice ON ton_payments (invoice_id);
CREATE INDEX idx_ton_payments_alert ON ton_payments (alert_due) WHERE alert_due = 1 AND owner_alerted_at IS NULL;

-- The cursor and health of each watched (network, address).
CREATE TABLE ton_watch (
    network          TEXT NOT NULL,
    address_raw      TEXT NOT NULL,
    is_current       INTEGER NOT NULL DEFAULT 0,
    cursor_lt        INTEGER NOT NULL,
    cursor_tx_hex    TEXT,
    caught_up_at     TEXT,
    indexed_utime    INTEGER,
    global_id        INTEGER,
    account_status   TEXT,
    account_checked_at TEXT,
    stuck_lt         INTEGER,
    stuck_since      TEXT,
    last_ok_at       TEXT,
    last_error       TEXT,                          -- a class ('rate_limited', 'http_403'), never a URL
    last_error_at    TEXT,
    failures         INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (network, address_raw)
);

-- Throttle of global owner alerts (watcher health, rate, capacity).
CREATE TABLE ton_alerts (
    kind         TEXT PRIMARY KEY,
    last_sent_at TEXT NOT NULL,
    detail       TEXT
);
