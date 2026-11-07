//! Text the agent did not write and the owner did not either: guest names and
//! timezones from the public booking form, calendar names from remote
//! servers. It goes to the model only under `guest_supplied` /
//! `calendar_supplied` keys and only through these functions, which strip what
//! could hide or reorder text and cap the length.

/// Clean untrusted text for the model: control characters (C0, C1, DEL)
/// become spaces, bidirectional controls, zero-width characters and Unicode
/// tag characters are removed, runs of whitespace collapse to one space, the
/// ends are trimmed, and the result is cut to `max_chars` characters (the last
/// one an ellipsis when cut).
pub(crate) fn untrusted_text(s: &str, max_chars: usize) -> String {
    let mut out = String::with_capacity(s.len().min(max_chars.saturating_mul(4)));
    let mut space = false;
    for c in s.chars() {
        if is_invisible(c) {
            continue;
        }
        if c.is_control() || c.is_whitespace() {
            space = !out.is_empty();
            continue;
        }
        if space {
            out.push(' ');
            space = false;
        }
        out.push(c);
    }
    cap_chars(out, max_chars)
}

/// Owner-written text (titles, descriptions): kept as written, only capped.
/// Returns the text and whether it was cut.
pub(crate) fn owner_text(s: &str, max_chars: usize) -> (String, bool) {
    if s.chars().count() <= max_chars {
        (s.to_string(), false)
    } else {
        (cap_chars(s.to_string(), max_chars), true)
    }
}

fn cap_chars(s: String, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s;
    }
    if max_chars == 0 {
        return String::new();
    }
    let mut cut: String = s.chars().take(max_chars - 1).collect();
    cut.truncate(cut.trim_end().len());
    cut.push('…');
    cut
}

/// Characters that change how text displays without showing themselves.
/// Owner text the agent writes may not contain them either.
pub(crate) fn is_invisible(c: char) -> bool {
    matches!(
        c,
        // Arabic letter mark, left-to-right and right-to-left marks
        '\u{061C}' | '\u{200E}' | '\u{200F}'
        // embeddings and overrides (LRE, RLE, PDF, LRO, RLO)
        | '\u{202A}'..='\u{202E}'
        // isolates (LRI, RLI, FSI, PDI)
        | '\u{2066}'..='\u{2069}'
        // zero-width space, non-joiner, joiner; word joiner and invisible operators
        | '\u{200B}'..='\u{200D}'
        | '\u{2060}'..='\u{2064}'
        // byte order mark / zero-width no-break space, soft hyphen
        | '\u{FEFF}' | '\u{00AD}'
        // interlinear annotation controls
        | '\u{FFF9}'..='\u{FFFB}'
        // tag characters, used to smuggle hidden text
        | '\u{E0000}'..='\u{E007F}'
    )
}

/// A guest address the owner can recognise without the model getting it:
/// `a***@example.com`. The domain is cleaned like other untrusted text; an
/// address that does not look like one becomes `***`.
pub(crate) fn mask_email(s: &str) -> String {
    let Some((local, domain)) = s.trim().rsplit_once('@') else {
        return "***".to_string();
    };
    let domain = untrusted_text(domain, 64);
    if local.is_empty() || domain.is_empty() || domain.contains(' ') || domain.contains('@') {
        return "***".to_string();
    }
    match local.chars().next().filter(char::is_ascii_alphanumeric) {
        Some(first) => format!("{first}***@{domain}"),
        None => format!("***@{domain}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_become_spaces_and_collapse() {
        assert_eq!(untrusted_text("  Ada\n\tLovelace\r\n ", 80), "Ada Lovelace");
        assert_eq!(
            untrusted_text("a\u{0000}b\u{007F}c\u{0085}d", 80),
            "a b c d"
        );
        assert_eq!(untrusted_text("a\u{00A0}\u{2028}\u{3000}b", 80), "a b");
        assert_eq!(untrusted_text("\n\n", 80), "");
    }

    #[test]
    fn bidi_zero_width_and_tags_are_removed() {
        assert_eq!(untrusted_text("Eve\u{202E}lirpA", 80), "EvelirpA");
        assert_eq!(
            untrusted_text("\u{2066}a\u{2069}\u{200E}b\u{200F}\u{061C}", 80),
            "ab"
        );
        assert_eq!(
            untrusted_text("ig\u{200B}no\u{200D}re\u{FEFF}\u{2060}", 80),
            "ignore"
        );
        let hidden: String = "do this"
            .chars()
            .map(|c| char::from_u32(0xE0000 + c as u32).unwrap())
            .collect();
        assert_eq!(untrusted_text(&format!("Bob{hidden}"), 80), "Bob");
        assert_eq!(untrusted_text("soft\u{00AD}hyphen", 80), "softhyphen");
    }

    #[test]
    fn length_is_capped_in_characters() {
        assert_eq!(untrusted_text("abcdef", 6), "abcdef");
        assert_eq!(untrusted_text("abcdefg", 6), "abcde…");
        assert_eq!(untrusted_text("ééééééé", 3), "éé…");
        assert_eq!(untrusted_text("ab cdef", 4), "ab…");
        assert_eq!(untrusted_text("abc", 0), "");
        assert_eq!(untrusted_text(&"x".repeat(10_000), 80).chars().count(), 80);
    }

    #[test]
    fn owner_text_reports_truncation() {
        assert_eq!(
            owner_text("line\none", 20),
            ("line\none".to_string(), false)
        );
        assert_eq!(owner_text("abcdef", 4), ("abc…".to_string(), true));
    }

    #[test]
    fn emails_are_masked() {
        assert_eq!(mask_email("guest@example.com"), "g***@example.com");
        assert_eq!(mask_email(" Guest@Example.COM "), "G***@Example.COM");
        assert_eq!(mask_email("_x@example.com"), "***@example.com");
        assert_eq!(mask_email("a@b@example.com"), "a***@example.com");
        assert_eq!(mask_email("x@exa\u{202E}mple.com"), "x***@example.com");
        for bad in ["", "guest", "@example.com", "guest@", "guest@exa mple.com"] {
            assert_eq!(mask_email(bad), "***", "{bad:?}");
        }
        let long = format!("x@{}.com", "d".repeat(100));
        assert_eq!(mask_email(&long).chars().count(), 1 + 3 + 1 + 64);
    }
}
