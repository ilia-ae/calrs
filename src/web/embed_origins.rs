//! ilia fork: which sites may frame the embedded booking pages.
//!
//! Upstream answers `?embed=1` on a booking page with `frame-ancestors *`:
//! any site can put the booking widget in an iframe. `CALRS_EMBED_ORIGINS`
//! narrows that to the sites the operator names:
//!
//! - unset: any site (`*`), as upstream;
//! - `*`: any site, said explicitly;
//! - `none`: no other site; the pages keep `frame-ancestors 'self'` and
//!   `X-Frame-Options: SAMEORIGIN`, so only calrs itself (the embed code
//!   generator's preview) frames them;
//! - a comma or space separated list of origins (`https://example.org,
//!   https://www.example.org`): `frame-ancestors 'self'` plus exactly those.
//!
//! An origin is `https://host[:port]`: no path, query, user info or wildcard,
//! since `frame-ancestors` would read a path or `*.` as a wider match than
//! the operator meant. Plain `http://` is accepted only for `localhost`,
//! `127.0.0.1` and `.onion` hosts (a local test page; an onion service,
//! whose transport is already encrypted). An unusable value, an empty one
//! included, turns embedding off and logs an error: failing open would let
//! every site frame the booking form.
//!
//! It is a deployment fact, like the base URL, so it comes from the
//! environment rather than the database (`calrs config apply` does not
//! carry it).

/// How many origins a list may name, which keeps the CSP header short.
const MAX_ORIGINS: usize = 16;

/// The parents an embedded booking page may have (`AppState.embed_origins`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EmbedOrigins {
    /// Any site: `frame-ancestors *` (upstream behaviour).
    Any,
    /// No other site: the page stays `frame-ancestors 'self'`.
    Off,
    /// `'self'` and these origins, normalised (lower case, no default port
    /// removal, no duplicates), in the order given.
    Only(Vec<String>),
}

impl EmbedOrigins {
    /// Parses a `CALRS_EMBED_ORIGINS` value; the error names the entry.
    pub(crate) fn parse(raw: &str) -> Result<Self, String> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err("empty value (use none to turn embedding off, * for any site)".into());
        }
        if raw.eq_ignore_ascii_case("none") {
            return Ok(Self::Off);
        }
        if raw == "*" {
            return Ok(Self::Any);
        }
        let mut origins: Vec<String> = Vec::new();
        for entry in raw
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|e| !e.is_empty())
        {
            let origin = parse_origin(entry)?;
            if !origins.contains(&origin) {
                origins.push(origin);
            }
        }
        if origins.is_empty() {
            return Err("no origin (use none to turn embedding off)".into());
        }
        if origins.len() > MAX_ORIGINS {
            return Err(format!("more than {MAX_ORIGINS} origins"));
        }
        Ok(Self::Only(origins))
    }

    /// `CALRS_EMBED_ORIGINS`: unset is [`EmbedOrigins::Any`]; a value that
    /// does not parse is [`EmbedOrigins::Off`], with the reason logged.
    pub(crate) fn from_env() -> Self {
        Self::from_var(std::env::var("CALRS_EMBED_ORIGINS").ok().as_deref())
    }

    pub(crate) fn from_var(raw: Option<&str>) -> Self {
        let Some(raw) = raw else { return Self::Any };
        Self::parse(raw).unwrap_or_else(|reason| {
            tracing::error!(
                %reason,
                "CALRS_EMBED_ORIGINS refused; embedded booking pages may not be framed by other sites"
            );
            Self::Off
        })
    }

    /// The `frame-ancestors` directive of an embedded booking page, or
    /// `None` when other sites may not frame it (the strict CSP stays).
    pub(crate) fn frame_ancestors(&self) -> Option<String> {
        match self {
            Self::Any => Some("frame-ancestors *".to_string()),
            Self::Off => None,
            Self::Only(origins) => Some(format!("frame-ancestors 'self' {}", origins.join(" "))),
        }
    }

    /// The policy as the startup log shows it.
    pub(crate) fn describe(&self) -> String {
        match self {
            Self::Any => "*".to_string(),
            Self::Off => "none".to_string(),
            Self::Only(origins) => origins.join(" "),
        }
    }
}

/// One origin, `scheme://host[:port]`, lower-cased; the error names it.
fn parse_origin(entry: &str) -> Result<String, String> {
    let bad = |why: &str| Err(format!("{entry:?}: {why}"));
    let lower = entry.to_ascii_lowercase();
    let (scheme, rest) = match lower.split_once("://") {
        Some((s @ ("https" | "http"), rest)) => (s, rest),
        _ => return bad("not an https:// origin"),
    };
    if rest.contains(['/', '?', '#', '@', '\\', '*', '\'', '"', ';']) {
        return bad("an origin is scheme://host[:port], with no path, wildcard or quotes");
    }
    let (host, port) = match rest.rsplit_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (rest, None),
    };
    if !is_host_name(host) {
        return bad("not a host name");
    }
    if let Some(port) = port {
        let ok = !port.is_empty()
            && port.len() <= 5
            && port.bytes().all(|b| b.is_ascii_digit())
            && port.parse::<u16>().is_ok_and(|p| p > 0);
        if !ok {
            return bad("not a port");
        }
    }
    if scheme == "http" && !(host == "localhost" || host == "127.0.0.1" || host.ends_with(".onion"))
    {
        return bad("http:// only for localhost, 127.0.0.1 and .onion; use https://");
    }
    Ok(lower)
}

/// A DNS name (or a dotted IPv4 address): labels of letters, digits and
/// inner hyphens, 1 to 63 bytes each, at most 253 bytes in all.
fn is_host_name(host: &str) -> bool {
    host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn only(origins: &[&str]) -> EmbedOrigins {
        EmbedOrigins::Only(origins.iter().map(|o| o.to_string()).collect())
    }

    #[test]
    fn unset_is_upstream_any_site() {
        assert_eq!(EmbedOrigins::from_var(None), EmbedOrigins::Any);
        assert_eq!(
            EmbedOrigins::Any.frame_ancestors().as_deref(),
            Some("frame-ancestors *")
        );
    }

    #[test]
    fn star_and_none_are_keywords() {
        assert_eq!(EmbedOrigins::parse(" * "), Ok(EmbedOrigins::Any));
        assert_eq!(EmbedOrigins::parse("none"), Ok(EmbedOrigins::Off));
        assert_eq!(EmbedOrigins::parse("NONE"), Ok(EmbedOrigins::Off));
        assert_eq!(EmbedOrigins::Off.frame_ancestors(), None);
    }

    #[test]
    fn a_list_frames_self_and_exactly_those_origins() {
        let parsed =
            EmbedOrigins::parse("https://ilia.ae, HTTPS://WWW.Ilia.ae\thttps://ilia.ae").unwrap();
        assert_eq!(parsed, only(&["https://ilia.ae", "https://www.ilia.ae"]));
        assert_eq!(
            parsed.frame_ancestors().as_deref(),
            Some("frame-ancestors 'self' https://ilia.ae https://www.ilia.ae")
        );
        assert_eq!(parsed.describe(), "https://ilia.ae https://www.ilia.ae");
        assert_eq!(
            EmbedOrigins::parse("https://example.org:8443"),
            Ok(only(&["https://example.org:8443"]))
        );
    }

    #[test]
    fn plain_http_only_for_local_and_onion_hosts() {
        for ok in [
            "http://localhost:8080",
            "http://127.0.0.1",
            "http://ilia26sowyfexylopfirhb5fway6b5in7rrdhpfj33brqjcnzja2seqd.onion",
        ] {
            assert_eq!(EmbedOrigins::parse(ok), Ok(only(&[ok])), "{ok}");
        }
        assert!(EmbedOrigins::parse("http://ilia.ae").is_err());
    }

    #[test]
    fn anything_wider_or_malformed_is_refused() {
        for bad in [
            "",
            "   ",
            ", ,",
            "ilia.ae",
            "*.ilia.ae",
            "https://*.ilia.ae",
            "https://*",
            "https://ilia.ae/",
            "https://ilia.ae/contacts",
            "https://ilia.ae?x=1",
            "https://user@ilia.ae",
            "https:",
            "https://",
            "https://:443",
            "https://ilia.ae:",
            "https://ilia.ae:0",
            "https://ilia.ae:65536",
            "https://ilia.ae:+1",
            "https://-ilia.ae",
            "https://ilia..ae",
            "https://ilia.ae.",
            "https://[::1]",
            "https://ilia.ae;script-src",
            "https://ilia.ae'",
            "'self'",
            "data:",
            "javascript://ilia.ae",
            "wss://ilia.ae",
            "https://ilia.ae *",
            "* https://ilia.ae",
            "none https://ilia.ae",
        ] {
            assert!(EmbedOrigins::parse(bad).is_err(), "{bad:?}");
        }
        let many = (0..=MAX_ORIGINS)
            .map(|i| format!("https://s{i}.example.org"))
            .collect::<Vec<_>>()
            .join(" ");
        assert!(EmbedOrigins::parse(&many).is_err());
    }

    #[test]
    fn a_refused_value_turns_embedding_off() {
        assert_eq!(EmbedOrigins::from_var(Some("")), EmbedOrigins::Off);
        assert_eq!(
            EmbedOrigins::from_var(Some("https://*.ilia.ae")),
            EmbedOrigins::Off
        );
        assert_eq!(
            EmbedOrigins::from_var(Some("https://ilia.ae")),
            only(&["https://ilia.ae"])
        );
    }
}
