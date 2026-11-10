//! Text from guests, hosts and Telegram users on its way into a message.
//!
//! Messages are sent as plain text (no `parse_mode`), so nothing is markup and
//! nothing needs escaping. What is left to defend against is text that looks
//! like something else: control characters and newlines that fake extra lines,
//! bidirectional overrides that reorder what is shown, invisible characters
//! that hide a lookalike, and sheer length that pushes the date out.

/// Characters removed outright: bidi controls, zero-width characters, the
/// word joiner and invisible operators, BOM, and Unicode tag characters.
fn invisible(c: char) -> bool {
    matches!(c,
        '\u{061C}' | '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}'
        | '\u{2060}'..='\u{2069}' | '\u{FEFF}' | '\u{E0000}'..='\u{E007F}'
        | '\u{00AD}' | '\u{180E}'
    )
}

/// `value` on one line: controls become spaces, invisible characters go,
/// whitespace collapses, and the result is cut to `max` characters with `…`.
pub(crate) fn clean(value: &str, max: usize) -> String {
    let mut out = String::with_capacity(value.len().min(max * 4));
    let mut space = false;
    for c in value.chars() {
        if invisible(c) {
            continue;
        }
        if c.is_control() || c.is_whitespace() {
            space = true;
            continue;
        }
        if space && !out.is_empty() {
            out.push(' ');
        }
        space = false;
        out.push(c);
    }
    shorten(&out, max)
}

/// A host's message (ilia fork, `web::messages`): like [`clean`], but the
/// line breaks stay. Each line is cleaned on its own (controls become
/// spaces, invisible characters go), more than one empty line in a row
/// becomes one, and the whole is trimmed and cut to `max` characters.
/// Host-written: it is not defanged.
pub(crate) fn clean_lines(value: &str, max: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in value.replace("\r\n", "\n").replace('\r', "\n").split('\n') {
        let line = clean(line, max);
        if line.is_empty() && lines.last().is_none_or(|l| l.is_empty()) {
            continue;
        }
        lines.push(line);
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    shorten(&lines.join("\n"), max)
}

/// Cut to `max` characters, ending with `…` when anything was cut.
pub(crate) fn shorten(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    let kept: String = value.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
}

/// Guest-typed text on its way into a host's message, after [`clean`].
///
/// Plain text is not markup, but Telegram clients still turn URLs, bare
/// domains, `@mentions` and `/commands` into tappable entities. A guest name
/// like "Payment failed, re-verify at https://pay.example/login" would then
/// arrive as a live link inside the owner's own bot chat. So the shapes a
/// client links are broken with visible characters (`clean` has already
/// removed invisible ones): `://` becomes `[:]//`, a dot between two letters
/// or digits `[.]`, and an `@` or `/` that starts a word `[@]`/`[/]`.
pub(crate) fn defang(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let mut out = String::with_capacity(value.len() + 8);
    let word = |c: Option<&char>| c.is_some_and(|c| c.is_alphanumeric());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let prev = i.checked_sub(1).and_then(|p| chars.get(p));
        let next = chars.get(i + 1);
        match c {
            ':' if next == Some(&'/') && chars.get(i + 2) == Some(&'/') => {
                out.push_str("[:]//");
                i += 3;
                continue;
            }
            '.' if word(prev) && word(next) => out.push_str("[.]"),
            '@' | '/' if !word(prev) && prev != Some(&'/') => {
                out.push('[');
                out.push(c);
                out.push(']');
            }
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

/// A guest's email address on its way into a host's message: as typed
/// when it is a plain address, else defanged like a name ([`defang`]).
/// calrs only checks for an `@` and a dot in the domain, so "Payment
/// failed, re-verify at https://… a@x.co" is a valid guest email; it must
/// not arrive in the owner's bot chat as a tappable link.
pub(crate) fn guest_email(value: &str) -> String {
    let value = clean(value, EMAIL_MAX);
    if plain_email(&value) {
        value
    } else {
        defang(&value)
    }
}

/// One `@`, a local part of letters, digits and `._%+-`, and a domain of
/// letters, digits, `-` and dots, with a dot inside it.
fn plain_email(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    let local_ok = !local.is_empty()
        && local
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-'));
    let domain_ok = domain.contains('.')
        && !domain.starts_with(['.', '-'])
        && !domain.ends_with(['.', '-'])
        && !domain.contains("..")
        && domain
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '.' | '-'));
    local_ok && domain_ok
}

/// Longest title, name and reason shown.
pub(crate) const TITLE_MAX: usize = 100;
pub(crate) const NAME_MAX: usize = 100;
/// Longest guest name in a host's message (a name, not a paragraph).
pub(crate) const GUEST_NAME_MAX: usize = 64;
pub(crate) const EMAIL_MAX: usize = 254;
pub(crate) const REASON_MAX: usize = 500;
pub(crate) const LOCATION_MAX: usize = 300;
/// Longest host message and guest message of a booking's thread.
pub(crate) const HOST_TEXT_MAX: usize = 2000;
pub(crate) const GUEST_TEXT_MAX: usize = 1000;
/// Telegram's limit for a message text.
pub(crate) const MESSAGE_MAX: usize = 4096;
