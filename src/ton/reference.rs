//! The payment code a guest puts in the transfer comment: `B-` and 10
//! Crockford base32 characters (50 random bits), e.g. `B-7KQ3M9XD2F`. It is a
//! random code rather than the booking id: the chain is public and permanent.
//!
//! Stored normalised (`B7KQ3M9XD2F`). A comment matches only when the WHOLE
//! comment normalises to a code: "B-7KQ3M9XD2F thanks" does not, so a spam
//! comment listing many codes can never touch an invoice.

use rand::RngCore;

/// Crockford base32: no I, L, O, U.
pub const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const BODY_LEN: usize = 10;
/// Longest comment considered at all; anything longer is not a code.
const MAX_COMMENT_LEN: usize = 64;

/// A fresh code, in its display form (`B-XXXXXXXXXX`).
pub fn generate(rng: &mut impl RngCore) -> String {
    let mut out = String::with_capacity(2 + BODY_LEN);
    out.push_str("B-");
    for _ in 0..BODY_LEN {
        // 256 is a multiple of 32: masking the low 5 bits is unbiased.
        let idx = (rng.next_u32() & 31) as usize;
        out.push(ALPHABET[idx] as char);
    }
    out
}

/// The comment reduced to a stored code (`B` + 10 characters), or `None`
/// when the whole comment is not one. Trims, upper-cases, drops spaces and
/// dashes, and maps the Crockford look-alikes `O`→`0`, `I`/`L`→`1`.
pub fn normalize(comment: &str) -> Option<String> {
    let comment = comment.trim();
    if comment.is_empty() || comment.len() > MAX_COMMENT_LEN || !comment.is_ascii() {
        return None;
    }
    let mut chars = comment
        .chars()
        .filter(|c| !c.is_ascii_whitespace() && *c != '-')
        .map(|c| c.to_ascii_uppercase());
    if chars.next() != Some('B') {
        return None;
    }
    let mut out = String::with_capacity(1 + BODY_LEN);
    out.push('B');
    for c in chars {
        let c = match c {
            'O' => '0',
            'I' | 'L' => '1',
            other => other,
        };
        if !ALPHABET.contains(&(c as u8)) {
            return None;
        }
        out.push(c);
    }
    (out.len() == 1 + BODY_LEN).then_some(out)
}

/// The display form of a stored code: `B7KQ3M9XD2F` → `B-7KQ3M9XD2F`.
pub fn display(stored: &str) -> String {
    match stored.strip_prefix('B') {
        Some(body) => format!("B-{body}"),
        None => stored.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_codes_have_the_format_and_normalise_to_themselves() {
        let mut rng = rand::rngs::OsRng;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..10_000 {
            let code = generate(&mut rng);
            assert_eq!(code.len(), 12);
            assert!(code.starts_with("B-"));
            let stored = normalize(&code).unwrap();
            assert_eq!(display(&stored), code);
            assert!(seen.insert(stored), "duplicate code {code}");
        }
    }

    #[test]
    fn crockford_look_alikes_case_and_spacing_are_forgiven() {
        assert_eq!(normalize("B-7KQ3M9XD2F").as_deref(), Some("B7KQ3M9XD2F"));
        assert_eq!(normalize("b-7kq3 m9xd-2f").as_deref(), Some("B7KQ3M9XD2F"));
        assert_eq!(normalize("  B7KQ3M9XD2F\n").as_deref(), Some("B7KQ3M9XD2F"));
        assert_eq!(normalize("B-OIL0000000").as_deref(), Some("B0110000000"));
    }

    #[test]
    fn only_a_whole_comment_matches() {
        for bad in [
            "",
            "B-7KQ3M9XD2F thanks",
            "pay B-7KQ3M9XD2F",
            "B-7KQ3M9XD2",
            "B-7KQ3M9XD2FF",
            "B-7KQ3M9XDUF",
            "C-7KQ3M9XD2F",
            "B-7KQ3M9XD2F!",
            "B-7KQ3M9XD2Ф",
            "B-7KQ3M9XD2F B-7KQ3M9XD2F",
        ] {
            assert_eq!(normalize(bad), None, "{bad:?}");
        }
    }
}
