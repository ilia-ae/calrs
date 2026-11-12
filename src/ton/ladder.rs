//! ilia fork: the demand ladder (ilia-booking pricing design §1.2). One
//! site-wide list of steps, `"free,20,40,60,80,100,200,300,400,500"`: with
//! `n` of the owner's live bookings ahead, the price is step `n`, and the
//! last step is the ceiling. The same text format is read from the
//! environment, the database and `config apply`, so one parser covers all
//! three. All arithmetic is in integer nanotons.

use super::amount::{self, NANO_PER_TON};

/// The ladder when nothing sets one.
pub const DEFAULT: &str = "free,20,40,60,80,100,200,300,400,500";
/// At most this many steps.
pub const MAX_STEPS: usize = 30;

/// One step: free (the owner approves such a booking) or a price in TON.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    Free,
    /// Nanotons, a valid `ton_prices` amount (1 to 100000 TON, 0.01 steps).
    Ton(u128),
}

impl Step {
    /// The value a booking form carries as `quoted_nano`: `0` for free,
    /// else the nanotons.
    pub fn quote_value(self) -> String {
        match self {
            Self::Free => "0".to_string(),
            Self::Ton(n) => n.to_string(),
        }
    }

    /// The form's `quoted_nano` back: `None` for anything that is not a
    /// whole number (a missing or garbled quote is never trusted).
    pub fn from_quote(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw.is_empty() || raw.len() > 20 || !raw.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        match raw.parse::<u128>().ok()? {
            0 => Some(Self::Free),
            n => Some(Self::Ton(n)),
        }
    }

    /// "20" or "12.5": the canonical text (no trailing zeros), "free".
    pub fn text(self) -> String {
        match self {
            Self::Free => "free".to_string(),
            Self::Ton(n) => ton_text(n),
        }
    }

    pub fn nano(self) -> Option<u128> {
        match self {
            Self::Free => None,
            Self::Ton(n) => Some(n),
        }
    }

    pub fn is_free(self) -> bool {
        self == Self::Free
    }
}

/// May a booking quoted at `quoted` be taken at `actual`? Free only for a
/// free quote; a price only for a priced quote at least as high. A lower
/// price is charged without asking; a switch between free and priced, in
/// either direction, or a higher price is asked again.
pub fn accepts(quoted: Step, actual: Step) -> bool {
    match (quoted, actual) {
        (Step::Free, Step::Free) => true,
        (Step::Ton(q), Step::Ton(a)) => a <= q,
        _ => false,
    }
}

/// TON without trailing zeros: "20", "12.5", "0.01".
pub fn ton_text(nano: u128) -> String {
    let whole = nano / NANO_PER_TON;
    let frac = format!("{:09}", nano % NANO_PER_TON);
    let frac = frac.trim_end_matches('0');
    if frac.is_empty() {
        whole.to_string()
    } else {
        format!("{whole}.{frac}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ladder(Vec<Step>);

impl Default for Ladder {
    fn default() -> Self {
        Self::parse(DEFAULT).unwrap_or_else(|_| Self(vec![Step::Free]))
    }
}

impl Ladder {
    /// Parse `"free,20,40"`: 1 to 30 comma-separated steps; `free` only as
    /// a run at the start; every other step a TON price (1 to 100000, at
    /// most two decimals); never decreasing. `Err` says what is wrong,
    /// without echoing the value.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err("the ladder is empty".into());
        }
        let mut steps = Vec::new();
        for (i, part) in raw.split(',').map(str::trim).enumerate() {
            if steps.len() == MAX_STEPS {
                return Err(format!("the ladder has more than {MAX_STEPS} steps"));
            }
            let step = if part.eq_ignore_ascii_case("free") {
                Step::Free
            } else {
                let decimals = part.split_once('.').map_or(0, |(_, f)| f.len());
                match amount::parse_ton_price(part) {
                    Some(n) if decimals <= 2 => Step::Ton(n),
                    _ => {
                        return Err(format!(
                            "step {}: a price in TON from 1 to 100000 with at most two decimals, or free",
                            i + 1
                        ))
                    }
                }
            };
            match (steps.last(), step) {
                (Some(Step::Ton(_)), Step::Free) => {
                    return Err(format!(
                        "step {}: free may only come first, before every price",
                        i + 1
                    ))
                }
                (Some(Step::Ton(prev)), Step::Ton(n)) if n < *prev => {
                    return Err(format!(
                        "step {}: the price must not go down along the ladder",
                        i + 1
                    ))
                }
                _ => {}
            }
            steps.push(step);
        }
        Ok(Self(steps))
    }

    /// The price at demand `n`: step `n`, capped at the last (the ceiling).
    pub fn price(&self, n: i64) -> Step {
        let i = usize::try_from(n.max(0)).unwrap_or(usize::MAX);
        self.0
            .get(i)
            .or(self.0.last())
            .copied()
            .unwrap_or(Step::Free)
    }

    pub fn steps(&self) -> &[Step] {
        &self.0
    }

    /// The canonical text, the format it is stored and exported in.
    pub fn text(&self) -> String {
        self.0
            .iter()
            .map(|s| s.text())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// The ceiling (the last step).
    pub fn ceiling(&self) -> Step {
        self.0.last().copied().unwrap_or(Step::Free)
    }

    /// The first step with a price, if any.
    pub fn lowest_price(&self) -> Option<u128> {
        self.0.iter().find_map(|s| s.nano())
    }

    pub fn starts_free(&self) -> bool {
        self.0.first() == Some(&Step::Free)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: u128 = NANO_PER_TON;

    #[test]
    fn the_default_parses_and_round_trips() {
        let l = Ladder::parse(DEFAULT).unwrap();
        assert_eq!(l.text(), DEFAULT);
        assert_eq!(l.steps().len(), 10);
        assert_eq!(Ladder::default(), l);
        assert_eq!(
            Ladder::parse(" FREE , 20 ,40.50, 40.5,100.00 ")
                .unwrap()
                .text(),
            "free,20,40.5,40.5,100"
        );
        assert_eq!(Ladder::parse("12.5").unwrap().text(), "12.5");
        let text = Ladder::parse("free,free,1,1.01").unwrap().text();
        assert_eq!(Ladder::parse(&text).unwrap().text(), text);
    }

    #[test]
    fn the_price_is_step_n_capped_at_the_ceiling() {
        let l = Ladder::parse(DEFAULT).unwrap();
        assert_eq!(l.price(0), Step::Free);
        assert_eq!(l.price(-3), Step::Free);
        assert_eq!(l.price(1), Step::Ton(20 * T));
        assert_eq!(l.price(5), Step::Ton(100 * T));
        assert_eq!(l.price(9), Step::Ton(500 * T));
        assert_eq!(l.price(10), Step::Ton(500 * T));
        assert_eq!(l.price(i64::MAX), Step::Ton(500 * T));
        assert_eq!(l.ceiling(), Step::Ton(500 * T));
        assert_eq!(l.lowest_price(), Some(20 * T));
        assert!(l.starts_free());
        let priced = Ladder::parse("30,60").unwrap();
        assert_eq!(priced.price(0), Step::Ton(30 * T));
        assert!(!priced.starts_free());
    }

    #[test]
    fn bad_ladders_are_refused() {
        let thirty_one = vec!["1"; 31].join(",");
        let thirty = vec!["1"; 30].join(",");
        assert!(Ladder::parse(&thirty).is_ok());
        for bad in [
            "",
            "  ",
            "20,free",
            "free,20,free",
            "40,20",
            "free,20,19.99",
            "12.345",
            "12.3456",
            "0.5",
            "0",
            "100001",
            "-5",
            "1e3",
            "twenty",
            "free,,20",
            "20,",
            thirty_one.as_str(),
        ] {
            assert!(Ladder::parse(bad).is_err(), "{bad:?}");
        }
        // The error never echoes the value.
        let e = Ladder::parse("free,SECRET").unwrap_err();
        assert!(!e.contains("SECRET"), "{e}");
    }

    #[test]
    fn quotes_round_trip_and_compare() {
        for s in [Step::Free, Step::Ton(20 * T), Step::Ton(1)] {
            assert_eq!(Step::from_quote(&s.quote_value()), Some(s));
        }
        for bad in ["", " ", "-1", "1.5", "abc", "999999999999999999999"] {
            assert_eq!(Step::from_quote(bad), None, "{bad:?}");
        }
        let (free, twenty, forty) = (Step::Free, Step::Ton(20 * T), Step::Ton(40 * T));
        assert!(accepts(free, free));
        assert!(accepts(forty, twenty), "a lower price is charged");
        assert!(accepts(twenty, twenty));
        assert!(!accepts(twenty, forty), "a higher price is asked again");
        assert!(!accepts(free, twenty), "free to priced is asked again");
        assert!(!accepts(twenty, free), "priced to free is asked again");
        assert_eq!(ton_text(12_500_000_000), "12.5");
        assert_eq!(ton_text(20 * T), "20");
    }
}
