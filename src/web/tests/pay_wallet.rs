//! ilia fork: the payment page shows one QR code; «Another wallet» opens the
//! wallet (a plain `ton://` link) and, with JavaScript, swaps the code to the
//! `ton://` one, as BTCPay's checkout switches its one QR code; a quiet
//! switch under the caption swaps it either way. Without JavaScript the
//! `ton://` code waits behind the disclosure.

use super::*;

const TON_PAY: &str = include_str!("../../../templates/ton_pay.html");
const KEEPER_SVG: &str = "<svg viewBox=\"0 0 1 1\" id=\"svg-keeper\"></svg>";
const TON_SVG: &str = "<svg viewBox=\"0 0 1 1\" id=\"svg-ton\"></svg>";

fn env() -> minijinja::Environment<'static> {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_loader(minijinja::path_loader("templates"));
    crate::i18n::register(&mut env);
    env
}

fn render(lang: &str, keeper_qr: &str, ton_qr: &str) -> String {
    env()
        .get_template("ton_pay.html")
        .unwrap()
        .render(context! {
            lang => lang,
            event_title => "Test Meeting",
            pay_state => "waiting",
            can_pay => true,
            live => true,
            amount_ton => "20.00",
            held_until => "23:58",
            zone => "UTC",
            renewals => 0,
            renewals_max => 0,
            seconds_left => 600,
            address => "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI",
            comment => "ABCD-1234",
            ton_link => "ton://transfer/UQCD?amount=1&text=ABCD-1234",
            keeper_link => "https://app.tonkeeper.com/transfer/UQCD?amount=1&text=ABCD-1234&asset_id=ton/mainnet/coin",
            keeper_qr_svg => keeper_qr,
            qr_svg => ton_qr,
        })
        .unwrap()
}

fn page() -> String {
    render("en", KEEPER_SVG, TON_SVG)
}

/// The opening tag of the element with `id`.
fn tag<'a>(page: &'a str, id: &str) -> &'a str {
    let at = page
        .find(&format!("id=\"{id}\""))
        .unwrap_or_else(|| panic!("no #{id}"));
    let open = page[..at].rfind('<').unwrap();
    let close = at + page[at..].find('>').unwrap();
    &page[open..=close]
}

/// Whitespace-normalised: a reformatted rule or statement still matches.
fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The inline script that switches the code (it names #ton-qr).
fn swap_script(page: &str) -> &str {
    let at = page.find("getElementById('ton-qr')").expect("swap script");
    let open = page[..at].rfind("<script>").unwrap();
    let close = at + page[at..].find("</script>").unwrap();
    &page[open..close]
}

/// One QR box is rendered, visible, with Tonkeeper's code in it and its
/// label and caption.
#[test]
fn one_qr_box_shows_tonkeeper_code() {
    let page = page();
    assert_eq!(page.matches("id=\"ton-qr\"").count(), 1);
    let scan = tag(&page, "ton-scan");
    assert!(scan.contains("data-shown=\"keeper\""), "{scan}");
    assert!(!scan.contains("hidden"), "{scan}");
    let qr = tag(&page, "ton-qr");
    assert!(qr.contains("role=\"img\""), "{qr}");
    let keeper_alt = crate::i18n::translate("en", "ton-pay-qr-alt-tonkeeper", None);
    assert!(qr.contains(&format!("aria-label=\"{keeper_alt}\"")), "{qr}");
    // Tonkeeper's code sits in the box, shown.
    let in_box =
        &page[page.find("id=\"ton-qr\"").unwrap()..page.find("id=\"ton-qr-caption\"").unwrap()];
    assert!(
        in_box.contains(&format!("<div data-qr=\"keeper\">{KEEPER_SVG}</div>")),
        "{in_box}"
    );
    assert!(!in_box.contains("svg-ton"), "{in_box}");
    // The caption is a polite live region: the swap reads out its new text.
    let caption = tag(&page, "ton-qr-caption");
    assert!(caption.contains("aria-live=\"polite\""), "{caption}");
}

/// Both codes reach the page, once each: the script moves the `ton://` one
/// into the box rather than drawing a third copy.
#[test]
fn both_codes_are_in_the_page_once() {
    let page = page();
    assert_eq!(page.matches("id=\"svg-keeper\"").count(), 1);
    assert_eq!(page.matches("id=\"svg-ton\"").count(), 1);
    assert!(page.contains(&format!("<div data-qr=\"other\">{TON_SVG}</div>")));
    let script = swap_script(&page);
    assert!(script.contains("[data-qr=\"other\"]"), "{script}");
    assert!(script.contains("[data-qr=\"keeper\"]"), "{script}");
    assert!(script.contains(".appendChild("), "{script}");
}

/// «Another wallet» stays a real link to the `ton://` URI (a tap opens the
/// wallet, also without JavaScript); it is the first `ton:` href, which the
/// Molecule probe reads (ton_probe.py: `href="(ton:[^"]*)"`).
#[test]
fn another_wallet_is_the_ton_link() {
    let page = page();
    let other = tag(&page, "ton-open-other");
    assert!(
        other.starts_with("<a class=\"btn btn-secondary\""),
        "{other}"
    );
    assert!(other.contains("href=\"ton:"), "{other}");
    assert!(!other.contains("asset_id"), "{other}");
    let first = page.find("href=\"ton:").unwrap();
    assert!(page[..first].ends_with("id=\"ton-open-other\" "), "{other}");
    let keeper = tag(&page, "ton-open-keeper");
    assert!(keeper.contains("href=\"https:"), "{keeper}");
    // The script never stops the handoff: no preventDefault.
    assert!(!swap_script(&page).contains("preventDefault"));
}

/// Without JavaScript the disclosure is rendered open to use, with the
/// `ton://` code; the script right after it hides it, so with JavaScript
/// no disclosure is left and «Another wallet» is the only other-wallet link.
#[test]
fn disclosure_is_the_no_js_fallback() {
    let page = page();
    assert_eq!(page.matches("<details").count(), 1);
    assert_eq!(page.matches("id=\"ton-open-other\"").count(), 1);
    let details = tag(&page, "ton-other");
    assert_eq!(details, "<details class=\"ton-other\" id=\"ton-other\">");
    let body = &page[page.find("id=\"ton-other\"").unwrap()..page.find("</details>").unwrap()];
    assert!(body.contains("svg-ton"), "{body}");
    let alt = crate::i18n::translate("en", "ton-pay-qr-alt", None);
    assert!(body.contains(&format!("aria-label=\"{alt}\"")), "{body}");
    // The script follows the disclosure directly, so it never paints.
    let after = page[page.find("</details>").unwrap() + "</details>".len()..].trim_start();
    assert!(after.starts_with("<script>"), "{after:.200}");
    let script = swap_script(&page);
    assert!(script.contains("getElementById('ton-other')"), "{script}");
    assert!(
        squash(script).contains(".hidden = true;"),
        "the script hides the disclosure: {script}"
    );
}

/// Every element the script reaches by id is in the page; the switch under
/// the caption starts hidden (no use without JavaScript) and offers the
/// other code; the label, caption and switch text of both codes reach the
/// script translated.
#[test]
fn swap_targets_exist() {
    let page = page();
    let script = swap_script(&page);
    let ids: Vec<&str> = script
        .split("getElementById('")
        .skip(1)
        .map(|s| &s[..s.find('\'').unwrap()])
        .collect();
    assert!(ids.len() >= 7, "{ids:?}");
    for id in ids {
        assert!(page.contains(&format!("id=\"{id}\"")), "#{id} missing");
    }
    let toggle = tag(&page, "ton-qr-switch");
    assert!(toggle.contains("type=\"button\""), "{toggle}");
    assert!(toggle.contains(" hidden"), "{toggle}");
    let text = crate::i18n::translate("en", "ton-pay-qr-other", None);
    assert!(page.contains(&format!("{toggle}{text}</button>")), "{page}");
    // Russian: no apostrophes, which `tojson` escapes.
    let ru = render("ru", KEEPER_SVG, TON_SVG);
    let ru_script = swap_script(&ru);
    for key in [
        "ton-pay-qr-alt-tonkeeper",
        "ton-pay-qr-alt",
        "ton-pay-scan-tonkeeper",
        "ton-pay-scan",
        "ton-pay-qr-other",
        "ton-pay-qr-tonkeeper",
    ] {
        let s = crate::i18n::translate("ru", key, None);
        assert!(
            ru_script.contains(&format!("\"{s}\"")),
            "{key}: {ru_script}"
        );
    }
    assert!(script.contains("aria-label"), "{script}");
}

/// The choice survives the status poll's reload, per tab, and a browser
/// without storage still gets the page (every access in try).
#[test]
fn the_choice_survives_a_reload() {
    let page = page();
    let script = squash(swap_script(&page));
    assert!(script.contains("sessionStorage.getItem("), "{script}");
    assert!(script.contains("sessionStorage.setItem("), "{script}");
    assert_eq!(
        script.matches("sessionStorage.").count(),
        script.matches("try { sessionStorage.").count()
            + script.matches("try { if (sessionStorage.").count(),
        "{script}"
    );
    assert!(!script.contains("localStorage"), "{script}");
}

/// A click or tap leaves no focus ring on a wallet button; a key press
/// brings the keyboard ring back; the switch keeps its own ring and stays
/// in place, so no script moves focus.
#[test]
fn a_click_leaves_no_ring_on_the_wallet_buttons() {
    let css = squash(&TON_PAY[TON_PAY.find("<style>").unwrap()..TON_PAY.find("</style>").unwrap()]);
    assert!(
        css.contains(".ton-wallets .btn[data-pointer]:focus-visible { outline: none; }"),
        "{css}"
    );
    assert!(
        css.contains(".ton-qr-switch:focus-visible { outline: 2px solid var(--accent);"),
        "{css}"
    );
    let page = page();
    let script = swap_script(&page);
    assert!(script.contains("'pointerdown'"), "{script}");
    assert!(
        script.contains("removeAttribute('data-pointer')"),
        "{script}"
    );
    assert!(!script.contains(".focus("), "{script}");
}

/// Without the `ton://` code there is no switch and no disclosure; without
/// Tonkeeper's the disclosure stays the way to the other one.
#[test]
fn a_missing_code_drops_its_switch() {
    let page = render("en", KEEPER_SVG, "");
    assert!(!page.contains("id=\"ton-qr-switch\""));
    assert!(!page.contains("<details"));
    assert!(page.contains("id=\"ton-qr\""));
    let page = render("en", "", TON_SVG);
    assert!(!page.contains("id=\"ton-qr\""));
    assert!(page.contains("<details class=\"ton-other\" id=\"ton-other\">"));
}

/// The way back is translated everywhere; Russian is formal and short.
#[test]
fn way_back_is_translated() {
    let en = crate::i18n::translate("en", "ton-pay-qr-tonkeeper", None);
    for lang in ["de", "en", "es", "et", "fr", "it", "nl", "pl", "pt", "ru"] {
        let text = crate::i18n::translate(lang, "ton-pay-qr-tonkeeper", None);
        assert!(text.contains("Tonkeeper"), "{lang}: {text}");
        assert!(text.contains("QR"), "{lang}: {text}");
        if lang != "en" {
            assert_ne!(text, en, "{lang}");
        }
    }
    assert_eq!(
        crate::i18n::translate("ru", "ton-pay-qr-tonkeeper", None),
        "QR-код для Tonkeeper"
    );
    let page = render("ru", KEEPER_SVG, TON_SVG);
    assert!(page.contains("\"QR-код для Tonkeeper\""), "{page}");
}
