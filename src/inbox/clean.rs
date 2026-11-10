//! What the guest wrote, without the quoted history and the signature.
//!
//! As far as is reasonable with plain text: cut at the first reply header
//! (Gmail, Apple Mail, Thunderbird and Outlook in the shipped languages,
//! Kerio webmail's "Original Message" line), drop `>` lines, cut at the
//! signature delimiter, drop trailing "Sent from my …" lines. Then the
//! Telegram cleaning (controls, bidi, invisible characters) and the cap.

use crate::telegram::text::{clean_lines, GUEST_TEXT_MAX};

/// A line that starts the quoted history (looking at the next line too, for
/// headers a client wraps).
fn quote_header(line: &str, next: Option<&str>) -> bool {
    let l = line.trim();
    let lower = l.to_lowercase();
    let next_lower = next.map(|n| n.trim().to_lowercase()).unwrap_or_default();
    // "On Mon, 10 Nov 2026 at 10:00, Ilia <x@y> wrote:" (maybe wrapped).
    let wrote = |s: &str| {
        s.ends_with("wrote:")
            || s.ends_with("a écrit :")
            || s.ends_with("a écrit:")
            || s.ends_with("escribió:")
            || s.ends_with("ha scritto:")
            || s.ends_with("schreef:")
            || s.ends_with("napisał(a):")
            || s.ends_with("napisał:")
            || s.ends_with("escreveu:")
            || s.ends_with("kirjutas:")
            || s.ends_with("написал(а):")
            || s.ends_with("написал:")
            || s.ends_with("написала:")
            || s.ends_with("пишет:")
            || (s.starts_with("am ") && s.contains("schrieb"))
    };
    if wrote(&lower) {
        return true;
    }
    let opener = ["on ", "le ", "el ", "il ", "op ", "w dniu ", "em ", "am "];
    if opener.iter().any(|o| lower.starts_with(o)) && wrote(&next_lower) && next_lower.len() < 40 {
        return true;
    }
    // Gmail in Russian: "пн, 10 нояб. 2026 г. в 10:00, Ilia <x@y>:".
    if lower.ends_with(">:") && lower.contains('@') {
        return true;
    }
    // "-----Original Message-----", Kerio's "-------- Исходное сообщение --------".
    if l.starts_with("--")
        && (lower.contains("original message")
            || lower.contains("исходное сообщение")
            || lower.contains("пересылаемое сообщение")
            || lower.contains("forwarded message")
            || lower.contains("ursprüngliche nachricht")
            || lower.contains("message d'origine"))
    {
        return true;
    }
    false
}

/// An Outlook header block: `From:`/`От:` with `Sent:`/`Date:` right after.
fn outlook_block(lines: &[&str], i: usize) -> bool {
    let starts = |l: &str, keys: &[&str]| {
        let l = l.trim_start_matches(['*', ' ']).to_lowercase();
        keys.iter().any(|k| l.starts_with(k))
    };
    let line = lines[i];
    let is_rule = line.trim().len() >= 10 && line.trim().chars().all(|c| c == '_');
    let from_at = if is_rule { i + 1 } else { i };
    let Some(from) = lines.get(from_at) else {
        return false;
    };
    if !starts(from, &["from:", "от:", "von:", "de:", "da:", "od:", "van:"]) {
        return false;
    }
    lines[from_at + 1..].iter().take(4).any(|l| {
        starts(
            l,
            &[
                "sent:",
                "date:",
                "отправлено:",
                "дата:",
                "gesendet:",
                "envoyé :",
                "envoyé:",
                "enviado:",
                "inviato:",
                "wysłano:",
                "verzonden:",
            ],
        )
    })
}

/// A trailing client signature line.
fn sent_from(line: &str) -> bool {
    let lower = line.trim().to_lowercase();
    [
        "sent from my",
        "sent from ",
        "get outlook for",
        "отправлено с ",
        "отправлено из ",
        "отправлено через",
        "von meinem",
        "envoyé de mon",
        "enviado desde mi",
        "inviato da",
        "wysłane z",
    ]
    .iter()
    .any(|p| lower.starts_with(p))
}

/// The reply part of `text`, uncapped.
pub(crate) fn reply_text(text: &str) -> String {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = text.split('\n').collect();
    let mut kept: Vec<&str> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_end() == "--" || *line == "-- " {
            break;
        }
        if quote_header(line, lines.get(i + 1).copied()) || outlook_block(&lines, i) {
            break;
        }
        if line.trim_start().starts_with('>') {
            continue;
        }
        kept.push(line);
    }
    while kept
        .last()
        .is_some_and(|l| l.trim().is_empty() || sent_from(l))
    {
        kept.pop();
    }
    kept.join("\n")
}

/// The stored text: the reply, cleaned and capped, and whether it was cut.
pub(crate) fn finish(text: &str) -> (String, bool) {
    let reply = reply_text(text);
    let full = clean_lines(&reply, 1_000_000);
    let body = clean_lines(&reply, GUEST_TEXT_MAX);
    let shortened = full.chars().count() > GUEST_TEXT_MAX;
    (body, shortened)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmail_quotes_are_cut() {
        let t = "Yes, Tuesday works.\n\nOn Mon, 10 Nov 2026 at 10:00, Ilia <booking@ilia.ae>\nwrote:\n> Can we move it?\n";
        assert_eq!(reply_text(t), "Yes, Tuesday works.");
        let t = "Sure\nOn Mon, 10 Nov 2026, Ilia via calrs <booking@ilia.ae> wrote:\n> old";
        assert_eq!(reply_text(t), "Sure");
    }

    #[test]
    fn a_russian_outlook_quote_and_signature_are_cut() {
        let t = "Добрый день!\nПодтверждаю встречу.\n\n--\nИван Петров\nООО «Ромашка»\n\
________________________________\nОт: Ilia <booking@ilia.ae>\nОтправлено: 10 ноября 2026 г. 10:00\n\
Кому: Иван\nТема: Сообщение\n\nСтарый текст";
        assert_eq!(reply_text(t), "Добрый день!\nПодтверждаю встречу.");
        let t = "Спасибо\n\nпн, 10 нояб. 2026 г. в 10:00, Ilia <booking@ilia.ae>:\n> текст";
        assert_eq!(reply_text(t), "Спасибо");
        let t = "Ок\n\n-------- Исходное сообщение --------\nОт: booking@ilia.ae";
        assert_eq!(reply_text(t), "Ок");
        let t = "Буду вовремя.\n\nОтправлено с iPhone";
        assert_eq!(reply_text(t), "Буду вовремя.");
    }

    #[test]
    fn long_text_is_shortened_and_controls_go() {
        let (body, cut) = finish(&"а".repeat(1500));
        assert!(cut);
        assert_eq!(body.chars().count(), GUEST_TEXT_MAX);
        let (body, cut) = finish("Hi\u{202E}there\u{0007}\n\n\n\nBye");
        assert!(!cut);
        assert_eq!(body, "Hithere\n\nBye");
    }
}
