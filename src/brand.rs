//! ilia fork: the name this instance shows for itself, `CALRS_BRAND_NAME`
//! ("ilia.ae"), in place of "calrs" where a visitor sees the product name:
//! browser tab titles, the sign-in and registration pages, the signature and
//! footer of emails and the authenticator app entry of a new TOTP set-up.
//!
//! Like `CALRS_MADE_BY_NAME` it is a deployment fact read once at startup,
//! so it comes from the environment: no migration, no admin form. Unset,
//! empty or invalid, every one of those places stays upstream's "calrs".
//! Texts about the software itself ("calrs syncs these calendars", the
//! "based on calrs" footer credit) keep the software's name.

use std::sync::OnceLock;

/// The longest name accepted, in characters.
const MAX_CHARS: usize = 64;

/// A brand name as the environment gives it: trimmed, 1 to 64 characters,
/// no control characters. Anything else is no brand. Every place that shows
/// it escapes it (templates by autoescape, emails with `h`).
pub(crate) fn name_from(raw: Option<&str>) -> Option<String> {
    let name = raw?.trim();
    let ok = !name.is_empty()
        && name.chars().count() <= MAX_CHARS
        && !name.chars().any(char::is_control);
    ok.then(|| name.to_string())
}

/// The configured brand, `None` for upstream's "calrs".
pub(crate) fn configured() -> Option<&'static str> {
    static BRAND: OnceLock<Option<String>> = OnceLock::new();
    BRAND
        .get_or_init(|| name_from(std::env::var("CALRS_BRAND_NAME").ok().as_deref()))
        .as_deref()
}

/// The name to show: the brand, or "calrs".
pub(crate) fn name() -> &'static str {
    configured().unwrap_or("calrs")
}

/// The template global `brand_name`, only when a brand is set. Templates test
/// it and fall back to upstream's text without it, so an environment that
/// does not register it (unit tests) renders exactly like upstream.
pub(crate) fn register(env: &mut minijinja::Environment<'static>, brand: Option<&str>) {
    if let Some(brand) = brand {
        env.add_global("brand_name", brand.to_string());
    }
}

/// The last line of every plain-text email: upstream's "— calrs" in every
/// locale, so "— <brand>" whatever the language.
pub(crate) fn email_signature(brand: &str) -> String {
    format!("— {brand}")
}

/// What follows "Sent by" at the bottom of an HTML email: upstream's link to
/// cal.rs, or the brand as plain text (escaped with `h`).
pub(crate) fn email_credit_html(brand: Option<&str>, h: fn(&str) -> String) -> String {
    match brand {
        Some(brand) => format!(
            r#"<span style="font-size:12px;color:#6b7280;font-weight:600;">{}</span>"#,
            h(brand)
        ),
        None => r#"<a href="https://cal.rs" style="font-size:12px;color:#6b7280;font-weight:600;text-decoration:none;">calrs</a>"#.to_string(),
    }
}

/// The issuer of a TOTP set-up, the name an authenticator app lists the
/// account under. The otpauth URI does not allow `:` in it, so it becomes `_`
/// as in the account name. It only labels new set-ups: codes do not depend
/// on it, and apps that already hold the secret keep their label.
pub(crate) fn totp_issuer() -> String {
    name().replace(':', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brand_is_short_plain_text() {
        assert_eq!(name_from(Some(" ilia.ae ")).as_deref(), Some("ilia.ae"));
        let longest = "я".repeat(MAX_CHARS);
        assert_eq!(name_from(Some(&longest)).as_deref(), Some(longest.as_str()));
        for bad in [
            "",
            "   ",
            "a\nb",
            "a\tb",
            "a\u{7}b",
            &"x".repeat(MAX_CHARS + 1),
        ] {
            assert_eq!(name_from(Some(bad)), None, "{bad:?}");
        }
        assert_eq!(name_from(None), None);
    }

    #[test]
    fn email_signature_and_credit() {
        assert_eq!(email_signature("ilia.ae"), "— ilia.ae");
        let h = crate::email::h;
        assert_eq!(
            email_credit_html(Some("<b>ilia</b> & co"), h),
            r#"<span style="font-size:12px;color:#6b7280;font-weight:600;">&lt;b&gt;ilia&lt;/b&gt; &amp; co</span>"#
        );
        assert!(email_credit_html(None, h).starts_with(r#"<a href="https://cal.rs""#));
    }

    /// The upstream signature this replaces is the same in every locale.
    #[test]
    fn upstream_signature_is_calrs_everywhere() {
        for (lang, _) in crate::i18n::supported_with_labels() {
            assert_eq!(
                crate::i18n::translate(lang, "email-signature", None),
                email_signature("calrs"),
                "{lang}"
            );
        }
    }
}
