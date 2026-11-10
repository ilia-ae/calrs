//! ilia fork additions to the web layer, kept in one file so rebasing the
//! fork onto a new upstream release touches upstream files as little as
//! possible.

use minijinja::Environment;

/// Where guests get the source of the version they are using (AGPL-3.0,
/// section 13): the deployment sets `CALRS_SOURCE_URL` to the exact commit
/// of the running build. Only an `https://` URL with no whitespace, quotes or
/// angle brackets is used; anything else shows no link rather than a broken
/// or unsafe one.
pub(crate) fn source_url_from(raw: Option<&str>) -> Option<String> {
    let url = raw?.trim();
    let safe = url.len() <= 512
        && url.starts_with("https://")
        && url.len() > "https://".len()
        && !url
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "\"'<>`\\".contains(c));
    safe.then(|| url.to_string())
}

/// Template globals of the fork: `source_url`, the AGPL source link that
/// `base.html` shows on every page, the embedded booking widget included, and
/// `telegram_enabled()`.
/// The URL is validated above (no quotes, angle brackets or whitespace), so it
/// goes into the `href` as is rather than with every `/` entity-escaped.
pub(crate) fn register(env: &mut Environment<'static>, source_url: Option<&str>) {
    if let Some(url) = source_url_from(source_url) {
        env.add_global("source_url", minijinja::Value::from_safe_string(url));
    }
    // Whether Telegram is on (`CALRS_TELEGRAM_BOT_TOKEN`), read at render
    // time: Profile & Settings links the Telegram page only then.
    env.add_function("telegram_enabled", crate::telegram::enabled);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_url_accepts_only_clean_https_urls() {
        let commit = "https://github.com/ilia-ae/calrs/tree/0123456789abcdef";
        assert_eq!(source_url_from(Some(commit)).as_deref(), Some(commit));
        assert_eq!(
            source_url_from(Some("  https://example.org/src \n")).as_deref(),
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
            assert_eq!(source_url_from(Some(bad)), None, "{bad}");
        }
        assert_eq!(source_url_from(None), None);
    }

    fn login_page(source_url: Option<&str>) -> String {
        let mut env = Environment::new();
        env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
        env.set_loader(minijinja::path_loader("templates"));
        crate::i18n::register(&mut env);
        register(&mut env, source_url);
        env.get_template("auth/login.html")
            .unwrap()
            .render(minijinja::context! { lang => "en" })
            .unwrap()
    }

    #[test]
    fn pages_link_the_source_of_the_running_version() {
        let url = "https://github.com/ilia-ae/calrs/tree/0123456789abcdef";
        let html = login_page(Some(url));
        assert!(
            html.contains(&format!(r#"<div class="source-link"><a href="{url}""#)),
            "{html}"
        );
        assert!(html.contains(">Source code</a>"), "{html}");
    }

    #[test]
    fn no_source_url_no_link() {
        let html = login_page(None);
        assert!(!html.contains(r#"class="source-link""#), "{html}");
    }
}
