//! ilia fork additions to the web layer, kept in one file so rebasing the
//! fork onto a new upstream release touches upstream files as little as
//! possible.

use minijinja::Environment;

/// An `https://` URL the footer can put into an `href` as is: no whitespace,
/// quotes or angle brackets, at most 512 bytes. Anything else gives no link
/// rather than a broken or unsafe one.
pub(crate) fn https_url_from(raw: Option<&str>) -> Option<String> {
    let url = raw?.trim();
    let safe = url.len() <= 512
        && url.starts_with("https://")
        && url.len() > "https://".len()
        && !url
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "\"'<>`\\".contains(c));
    safe.then(|| url.to_string())
}

/// The maker credited in the footer ("Made by <name>"): trimmed, 1 to 64
/// characters, no control characters. The template escapes it.
pub(crate) fn made_by_name_from(raw: Option<&str>) -> Option<String> {
    let name = raw?.trim();
    let ok = !name.is_empty() && name.chars().count() <= 64 && !name.chars().any(char::is_control);
    ok.then(|| name.to_string())
}

/// What the fork's footer line shows, read once at startup from the
/// deployment's environment:
///
/// - `CALRS_SOURCE_URL`: where guests get the source of the version they are
///   using (AGPL-3.0, section 13), the exact commit of the running build;
/// - `CALRS_MADE_BY_NAME`, `CALRS_MADE_BY_URL`: who runs this instance. Unset,
///   the line credits calrs only.
///
/// They are deployment facts, like the base URL, so they come from the
/// environment rather than the database: no migration, no admin form.
#[derive(Default)]
pub(crate) struct Footer {
    pub(crate) source_url: Option<String>,
    pub(crate) made_by_name: Option<String>,
    pub(crate) made_by_url: Option<String>,
}

impl Footer {
    pub(crate) fn from_env() -> Self {
        let var = |name| std::env::var(name).ok();
        Self {
            source_url: var("CALRS_SOURCE_URL"),
            made_by_name: var("CALRS_MADE_BY_NAME"),
            made_by_url: var("CALRS_MADE_BY_URL"),
        }
    }
}

/// Template globals of the fork: `source_url`, `made_by_name` and
/// `made_by_url` for the footer line of `base.html` ("Made by Ilia · based on
/// calrs · Source code", in English whatever the page language), and
/// `telegram_enabled()`. The source link is on every page, the embedded
/// booking widget and the dashboard included; the made-by part is the footer
/// block, which embed mode hides and the dashboard leaves out.
/// URLs are validated above (no quotes, angle brackets or whitespace), so they
/// go into the `href` as is rather than with every `/` entity-escaped; the
/// name stays a plain value and is escaped.
pub(crate) fn register(env: &mut Environment<'static>, footer: &Footer) {
    if let Some(url) = https_url_from(footer.source_url.as_deref()) {
        env.add_global("source_url", minijinja::Value::from_safe_string(url));
    }
    if let Some(name) = made_by_name_from(footer.made_by_name.as_deref()) {
        env.add_global("made_by_name", name);
        if let Some(url) = https_url_from(footer.made_by_url.as_deref()) {
            env.add_global("made_by_url", minijinja::Value::from_safe_string(url));
        }
    }
    // Whether Telegram is on (`CALRS_TELEGRAM_BOT_TOKEN`), read at render
    // time: Profile & Settings links the Telegram page only then.
    env.add_function("telegram_enabled", crate::telegram::enabled);
}

/// ilia fork (`crate::inbox`): what the "too late to cancel or reschedule"
/// page offers as a contact. In relay mode a `mailto:` to the booking's
/// system reply address (with the `[#token]` subject in bare mode), or
/// nothing when no mailbox is read; otherwise upstream's host address.
pub(crate) fn blocked_contact(uid: &str, host_email: String) -> String {
    if crate::inbox::relay() {
        crate::inbox::reply::guest_mailto(uid)
            .map(|(href, _)| href)
            .unwrap_or_default()
    } else {
        host_email
    }
}

/// The address shown for [`blocked_contact`]'s value.
pub(crate) fn blocked_contact_shown(contact: &str) -> &str {
    let bare = contact.strip_prefix("mailto:").unwrap_or(contact);
    bare.split('?').next().unwrap_or(bare)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_url_from_accepts_only_clean_https_urls() {
        let commit = "https://github.com/ilia-ae/calrs/tree/0123456789abcdef";
        assert_eq!(https_url_from(Some(commit)).as_deref(), Some(commit));
        assert_eq!(
            https_url_from(Some("  https://example.org/src \n")).as_deref(),
            Some("https://example.org/src")
        );
        for bad in [
            "",
            "https://",
            "http://github.com/ilia-ae/calrs",
            "javascript:alert(1)",
            "https://example.org/a b",
            "https://example.org/\"onmouseover=x",
            "https://example.org/<script>",
        ] {
            assert_eq!(https_url_from(Some(bad)), None, "{bad}");
        }
        assert_eq!(https_url_from(None), None);
    }

    #[test]
    fn made_by_name_is_short_plain_text() {
        assert_eq!(made_by_name_from(Some(" Ilia ")).as_deref(), Some("Ilia"));
        assert_eq!(
            made_by_name_from(Some(&"я".repeat(64))).as_deref(),
            Some("я".repeat(64).as_str())
        );
        for bad in ["", "   ", "a\nb", "a\u{7}b", &"x".repeat(65)] {
            assert_eq!(made_by_name_from(Some(bad)), None, "{bad:?}");
        }
        assert_eq!(made_by_name_from(None), None);
    }

    const SOURCE: &str = "https://github.com/ilia-ae/calrs/tree/0123456789abcdef";

    fn footer(source_url: Option<&str>, name: Option<&str>, url: Option<&str>) -> Footer {
        Footer {
            source_url: source_url.map(str::to_string),
            made_by_name: name.map(str::to_string),
            made_by_url: url.map(str::to_string),
        }
    }

    fn render(template: &str, footer: &Footer, ctx: minijinja::Value) -> String {
        let mut env = Environment::new();
        env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
        env.set_loader(minijinja::path_loader("templates"));
        crate::i18n::register(&mut env);
        register(&mut env, footer);
        env.get_template(template).unwrap().render(ctx).unwrap()
    }

    fn login_page(footer: &Footer, lang: &str) -> String {
        render(
            "auth/login.html",
            footer,
            minijinja::context! { lang => lang },
        )
    }

    /// The `<div class="powered">` line of a page.
    fn footer_line(html: &str) -> &str {
        let start = html
            .find(r#"<div class="powered">"#)
            .expect("no footer line");
        let end = start + html[start..].find("</div>").unwrap() + "</div>".len();
        &html[start..end]
    }

    #[test]
    fn footer_credits_the_maker_calrs_and_links_the_source() {
        let html = login_page(
            &footer(Some(SOURCE), Some("Ilia"), Some("https://ilia.ae")),
            "en",
        );
        assert_eq!(
            footer_line(&html),
            format!(
                r#"<div class="powered"><span class="made-by">Made by <a href="https://ilia.ae" target="_blank" rel="noopener noreferrer">Ilia</a> · based on <a href="https://cal.rs" target="_blank" rel="noopener noreferrer">calrs</a> · </span><a class="source-link" href="{SOURCE}" target="_blank" rel="noopener noreferrer">Source code</a></div>"#
            )
        );
    }

    #[test]
    fn footer_is_english_in_every_language() {
        let f = footer(Some(SOURCE), Some("Ilia"), Some("https://ilia.ae"));
        let en = footer_line(&login_page(&f, "en")).to_string();
        for lang in ["ru", "de", "fr"] {
            assert_eq!(footer_line(&login_page(&f, lang)), en, "{lang}");
        }
    }

    #[test]
    fn footer_without_a_maker_credits_calrs_only() {
        // No name: the URL alone is ignored.
        let html = login_page(&footer(Some(SOURCE), None, Some("https://ilia.ae")), "en");
        let line = footer_line(&html);
        assert!(
            line.starts_with(
                r#"<div class="powered"><span class="made-by">based on <a href="https://cal.rs""#
            ),
            "{line}"
        );
        assert!(!line.contains("Made by"), "{line}");
        assert!(!line.contains("ilia.ae"), "{line}");
    }

    #[test]
    fn maker_without_a_safe_url_is_plain_escaped_text() {
        let html = login_page(
            &footer(None, Some("<b>Ilia</b>"), Some("javascript:alert(1)")),
            "en",
        );
        let line = footer_line(&html);
        assert_eq!(
            line,
            r#"<div class="powered"><span class="made-by">Made by &lt;b&gt;Ilia&lt;&#x2f;b&gt; · based on <a href="https://cal.rs" target="_blank" rel="noopener noreferrer">calrs</a></span></div>"#
        );
    }

    #[test]
    fn no_source_url_no_link() {
        let html = login_page(&Footer::default(), "en");
        assert!(!html.contains("source-link"), "{html}");
        assert!(!html.contains("Source code"), "{html}");
    }

    /// Embedded, the made-by part is hidden like the logo, but the source link
    /// (AGPL section 13) is not.
    #[test]
    fn embed_hides_the_made_by_part_not_the_source_link() {
        let html = render(
            "auth/login.html",
            &footer(Some(SOURCE), Some("Ilia"), Some("https://ilia.ae")),
            minijinja::context! { lang => "en", embed => true },
        );
        let embed_css = &html[html.find("/* Embed mode").expect("no embed CSS")..];
        let hidden = embed_css
            .lines()
            .find(|line| line.contains("display: none !important"))
            .expect("no hidden elements in embed mode");
        assert!(hidden.contains(".made-by"), "{hidden}");
        assert!(!hidden.contains(".powered"), "{hidden}");
        assert!(
            footer_line(&html).contains(&format!(r#"<a class="source-link" href="{SOURCE}""#)),
            "{html}"
        );
    }
}
