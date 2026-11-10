-- ilia fork: adaptive re-payment (ilia-booking docs/ton.md, «Срок»). A
-- payment window that ends unpaid while the meeting is still far away is
-- renewed instead of releasing the time: the invoice stays open with the
-- same comment code and payment link, and only its deadlines move forward.
-- renewals_max is the number of renewals allowed when the invoice was
-- opened (CALRS_TON_MAX_RENEWALS then); renewal_notified the renewals the
-- guest was already told about (claimed once each).
ALTER TABLE ton_invoices ADD COLUMN renewals INTEGER NOT NULL DEFAULT 0 CHECK (renewals >= 0);
ALTER TABLE ton_invoices ADD COLUMN renewals_max INTEGER NOT NULL DEFAULT 0 CHECK (renewals_max >= 0);
ALTER TABLE ton_invoices ADD COLUMN renewal_notified INTEGER NOT NULL DEFAULT 0;
ALTER TABLE ton_invoices ADD COLUMN renewed_at TEXT;
