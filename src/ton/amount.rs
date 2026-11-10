//! Amounts: nanotons (1 TON = 10^9) as `u128` in Rust, `i64` in SQLite.
//! Integer arithmetic only, no floating point anywhere on the money path.

pub const NANO_PER_TON: u128 = 1_000_000_000;
/// Invoices are rounded up to 0.01 TON so the amount reads and types easily.
pub const INVOICE_STEP_NANO: u128 = 10_000_000;

/// The invoice amount for a price in US cents at `rate_usd_micro` (USD per
/// TON × 10^6): `ceil(cents × 10^13 / rate)`, then up to a multiple of
/// 0.01 TON. `None` for a zero rate or a result that does not fit in `i64`.
pub fn cents_to_nano(cents: u64, rate_usd_micro: u64) -> Option<u128> {
    if rate_usd_micro == 0 || cents == 0 {
        return None;
    }
    let num = u128::from(cents).checked_mul(10_000_000_000_000)?;
    let rate = u128::from(rate_usd_micro);
    let exact = num.div_ceil(rate);
    let stepped = exact
        .div_ceil(INVOICE_STEP_NANO)
        .checked_mul(INVOICE_STEP_NANO)?;
    to_i64(stepped)?;
    Some(stepped)
}

/// The USD value in cents of `nano` at a rate, rounded to the nearest cent.
pub fn nano_to_cents(nano: u128, rate_usd_micro: u64) -> u128 {
    // cents = nano × rate / 10^13
    (nano * u128::from(rate_usd_micro) + 5_000_000_000_000) / 10_000_000_000_000
}

/// Range check before any write to SQLite.
pub fn to_i64(nano: u128) -> Option<i64> {
    i64::try_from(nano).ok()
}

/// From a SQLite column; a negative value (never written) reads as 0.
pub fn from_i64(v: i64) -> u128 {
    u128::try_from(v).unwrap_or(0)
}

/// Exact decimal TON: at least two decimals, trailing zeros trimmed after
/// that ("12.35", "12.30", "0.000000001").
pub fn format_ton(nano: u128) -> String {
    let whole = nano / NANO_PER_TON;
    let frac = format!("{:09}", nano % NANO_PER_TON);
    let trimmed = frac.trim_end_matches('0');
    let shown = if trimmed.len() < 2 {
        &frac[..2]
    } else {
        trimmed
    };
    format!("{whole}.{shown}")
}

/// TON rounded half up to `decimals` places, for an approximate amount.
pub fn format_ton_approx(nano: u128, decimals: u32) -> String {
    let decimals = decimals.min(9);
    let unit = 10u128.pow(9 - decimals);
    let rounded = (nano + unit / 2) / unit;
    if decimals == 0 {
        return rounded.to_string();
    }
    let scale = 10u128.pow(decimals);
    format!(
        "{}.{:0width$}",
        rounded / scale,
        rounded % scale,
        width = decimals as usize
    )
}

/// USD cents as "40.00".
pub fn format_usd_cents(cents: u128) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

/// Parse a decimal TON amount ("12.35", "12", "0.5") into nanotons. At most
/// nine decimals, digits only, no sign, no exponent.
pub fn parse_ton(s: &str) -> Option<u128> {
    let (whole, frac) = match s.trim().split_once('.') {
        Some((w, f)) => (w, f),
        None => (s.trim(), ""),
    };
    if whole.is_empty() && frac.is_empty() {
        return None;
    }
    if !whole.bytes().all(|b| b.is_ascii_digit())
        || !frac.bytes().all(|b| b.is_ascii_digit())
        || frac.len() > 9
        || whole.len() > 12
    {
        return None;
    }
    let whole: u128 = if whole.is_empty() {
        0
    } else {
        whole.parse().ok()?
    };
    let frac_nano: u128 = if frac.is_empty() {
        0
    } else {
        format!("{frac:0<9}").parse().ok()?
    };
    whole.checked_mul(NANO_PER_TON)?.checked_add(frac_nano)
}

/// Cheapest and dearest TON price (the `ton_prices` CHECK): 1 to 100000 TON.
pub const MIN_PRICE_NANO: u128 = NANO_PER_TON;
pub const MAX_PRICE_NANO: u128 = 100_000 * NANO_PER_TON;
/// What a new price starts at: the owner's 100 TON.
pub const DEFAULT_PRICE_TON: &str = "100";

/// A TON price ("100", "12.5"): 1 to 100000 TON in steps of 0.01 TON, the
/// invoice step, so the price is exactly the amount asked.
pub fn parse_ton_price(s: &str) -> Option<u128> {
    parse_ton(s)
        .filter(|n| n % INVOICE_STEP_NANO == 0 && (MIN_PRICE_NANO..=MAX_PRICE_NANO).contains(n))
}

/// Parse US dollars ("40", "40.5", "40.00") into cents. At most 2 decimals.
pub fn parse_usd_cents(s: &str) -> Option<u64> {
    let s = s.trim();
    let (whole, frac) = s.split_once('.').unwrap_or((s, ""));
    if whole.is_empty()
        || whole.len() > 9
        || frac.len() > 2
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let whole: u64 = whole.parse().ok()?;
    let frac: u64 = if frac.is_empty() {
        0
    } else {
        format!("{frac:0<2}").parse().ok()?
    };
    whole.checked_mul(100)?.checked_add(frac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ton_prices() {
        assert_eq!(parse_ton_price("100"), Some(100 * NANO_PER_TON));
        assert_eq!(parse_ton_price(" 12.5 "), Some(12_500_000_000));
        assert_eq!(parse_ton_price("1"), Some(NANO_PER_TON));
        assert_eq!(parse_ton_price("100000"), Some(MAX_PRICE_NANO));
        for bad in ["", "0.99", "100000.01", "12.345", "-1", "1e3", "abc", "0"] {
            assert_eq!(parse_ton_price(bad), None, "{bad:?}");
        }
        assert_eq!(
            format_ton(parse_ton_price(DEFAULT_PRICE_TON).unwrap()),
            "100.00"
        );
    }

    #[test]
    fn forty_dollars_at_the_doc_rate() {
        // $40.00 at $3.2412 → 12.341… → 12.35 TON (docs/ton.md).
        assert_eq!(cents_to_nano(4000, 3_241_200), Some(12_350_000_000));
    }

    #[test]
    fn exact_multiples_are_not_bumped() {
        // $10 at $1.00 = exactly 10 TON.
        assert_eq!(cents_to_nano(1000, 1_000_000), Some(10_000_000_000));
        // $1 at $2 = 0.5 TON.
        assert_eq!(cents_to_nano(100, 2_000_000), Some(500_000_000));
        // One nano over a step rounds up a whole step.
        assert_eq!(cents_to_nano(100, 3_000_000), Some(340_000_000));
    }

    #[test]
    fn extremes_do_not_overflow() {
        assert_eq!(cents_to_nano(100, 0), None);
        assert_eq!(cents_to_nano(0, 1_000_000), None);
        // A $10 000 price at 1 micro-USD per TON does not fit in i64.
        assert_eq!(cents_to_nano(1_000_000, 1), None);
        // The largest price at a sane rate fits.
        let n = cents_to_nano(1_000_000, 1_000_000).unwrap();
        assert_eq!(n, 10_000 * NANO_PER_TON);
        assert!(to_i64(n).is_some());
        assert_eq!(cents_to_nano(u64::MAX, 1), None);
    }

    #[test]
    fn formatting() {
        assert_eq!(format_ton(12_350_000_000), "12.35");
        assert_eq!(format_ton(12_300_000_000), "12.30");
        assert_eq!(format_ton(12_000_000_000), "12.00");
        assert_eq!(format_ton(1), "0.000000001");
        assert_eq!(format_ton(2_433_333), "0.002433333");
        assert_eq!(format_ton_approx(38_649_999_999, 1), "38.6");
        assert_eq!(format_ton_approx(38_650_000_000, 1), "38.7");
        assert_eq!(format_ton_approx(38_650_000_000, 0), "39");
        assert_eq!(format_usd_cents(4000), "40.00");
        assert_eq!(format_usd_cents(5), "0.05");
        assert_eq!(nano_to_cents(12_350_000_000, 3_241_200), 4003);
    }

    #[test]
    fn parsing() {
        assert_eq!(parse_ton("12.35"), Some(12_350_000_000));
        assert_eq!(parse_ton("12"), Some(12 * NANO_PER_TON));
        assert_eq!(parse_ton(".5"), Some(500_000_000));
        assert_eq!(parse_ton("0.000000001"), Some(1));
        for bad in ["", ".", "-1", "1e3", "1.0000000001", "12,35", " 1 2", "+1"] {
            assert_eq!(parse_ton(bad), None, "{bad:?}");
        }
        assert_eq!(parse_usd_cents("40"), Some(4000));
        assert_eq!(parse_usd_cents("40.5"), Some(4050));
        assert_eq!(parse_usd_cents("0.99"), Some(99));
        for bad in ["", "40.505", "-1", "$40", "1e2", ".5"] {
            assert_eq!(parse_usd_cents(bad), None, "{bad:?}");
        }
    }
}
