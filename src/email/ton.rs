//! ilia fork: the TON payment emails (ilia-booking docs/ton.md, "Письма").
//!
//! Guests get two, both under the host's name: the link to their payment
//! page (only when they ask for it, and with no amount, address or coin in
//! it: a server mailing "pay in crypto" to any address typed into a form
//! would be a spam channel), and "not booked" when the booking is released
//! ([`TonReleased`]: the payment did not arrive in time, or the time was
//! taken before it was paid, only when they asked for the link or paid
//! part of it; or it was paid but could not be confirmed, always). The paid
//! booking's confirmation is upstream's own, sent by the approval path.
//!
//! The owner gets one per payment and one per problem, from the instance.
//! No email ever carries the comment a sender wrote on chain: it is
//! anybody's text (a phishing channel), shown only in the records.

use super::*;

/// A guest email about a payment, already written for the guest: the date
/// and time in their zone, in `lang`.
#[derive(Clone, Default)]
pub struct TonGuestMail {
    pub lang: String,
    pub guest_name: String,
    pub guest_email: String,
    pub host_name: String,
    pub host_email: String,
    pub event_title: String,
    /// The long date ("Tuesday, November 17, 2026").
    pub date: String,
    /// "14:00 (Asia/Dubai)".
    pub time: String,
    /// When the time stops being held, "14:30 (Asia/Dubai)"; empty when it
    /// does not apply.
    pub pay_by: String,
}

/// Why a priced booking was released without being confirmed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TonReleased {
    /// The payment did not arrive in time.
    Expired,
    /// The host's time was taken before the booking was paid.
    Taken,
    /// Paid, but the time could not be confirmed: the host refunds.
    Refund,
}

/// An owner email about a payment or an invoice, in the owner's language
/// and zone. Fields that do not apply stay empty.
#[derive(Clone, Default)]
pub struct TonOwnerMail {
    pub lang: String,
    pub to_name: String,
    pub to_email: String,
    /// Empty for a payment that matched no booking.
    pub event_title: String,
    pub date: String,
    pub time: String,
    pub guest_name: String,
    pub guest_email: String,
    /// The invoice amount, "12.35".
    pub amount_ton: String,
    /// The price, "40.00".
    pub price_usd: String,
    /// The invoice's rate, "3.2412".
    pub rate_usd: String,
    /// Received in total, "12.35".
    pub paid_ton: String,
    /// Over the amount, "" when none.
    pub excess_ton: String,
    /// This one transfer, for payment alerts.
    pub value_ton: String,
    /// (hash, explorer URL) of the transfers concerned.
    pub txs: Vec<(String, String)>,
    /// The sender's user-friendly address, when known.
    pub sender: Option<String>,
    pub sender_is_wallet: Option<bool>,
    /// The bookings dashboard, when the base URL is known.
    pub dashboard_url: Option<String>,
}

const ACCENT_GUEST: &str = "#2563eb";
const ACCENT_OK: &str = "#16a34a";
const ACCENT_WARN: &str = "#d97706";
const ACCENT_BAD: &str = "#dc2626";

// --- guest emails -----------------------------------------------------------

/// The guest asked for their payment page by email (once per invoice).
pub async fn send_guest_ton_pay_link(
    config: &SmtpConfig,
    mail: &TonGuestMail,
    pay_url: &str,
    cancel_url: Option<&str>,
) -> Result<()> {
    let to = named_mailbox(&mail.guest_name, &mail.guest_email)?;
    let lang = lang_or_en(Some(&mail.lang));
    let sender = Sender::Host {
        name: &mail.host_name,
        email: &mail.host_email,
        lang,
    };
    let composed = compose_guest_ton_pay_link(mail, pay_url, cancel_url);
    send_composed(config, sender, to, composed, None).await
}

pub(super) fn compose_guest_ton_pay_link(
    mail: &TonGuestMail,
    pay_url: &str,
    cancel_url: Option<&str>,
) -> ComposedEmail {
    let lang = lang_or_en(Some(&mail.lang));
    let mut plain_lines = vec![ta(lang, "ton-email-link-line", [("url", pay_url)])];
    let mut actions = vec![EmailAction {
        label: t(lang, "ton-email-link-action"),
        url: pay_url.to_string(),
        color: ACCENT_GUEST.to_string(),
    }];
    if let Some(url) = cancel_url {
        plain_lines.push(ta(lang, "email-confirm-need-to-cancel", [("url", url)]));
        actions.push(EmailAction {
            label: t(lang, "email-action-cancel-booking"),
            url: url.to_string(),
            color: ACCENT_BAD.to_string(),
        });
    }
    EmailParts {
        lang,
        subject: ta(
            lang,
            "ton-email-link-subject",
            [("event", &mail.event_title)],
        ),
        heading: ta(lang, "email-confirm-greeting", [("name", &mail.guest_name)]),
        message: ta(
            lang,
            "ton-email-link-message",
            [("event", &mail.event_title), ("host", &mail.host_name)],
        ),
        accent: ACCENT_GUEST,
        rows: {
            let mut rows = guest_rows(lang, mail);
            if !mail.pay_by.is_empty() {
                rows.push(row(t(lang, "ton-email-row-pay-by"), &mail.pay_by));
            }
            rows
        },
        plain_lines,
        footer_plain: None,
        footer_html: None,
        actions,
    }
    .compose()
}

/// The booking was released unconfirmed, for `reason`.
pub async fn send_guest_ton_released(
    config: &SmtpConfig,
    mail: &TonGuestMail,
    reason: TonReleased,
    rebook_url: Option<&str>,
    partial: bool,
) -> Result<()> {
    let to = named_mailbox(&mail.guest_name, &mail.guest_email)?;
    let lang = lang_or_en(Some(&mail.lang));
    let sender = Sender::Host {
        name: &mail.host_name,
        email: &mail.host_email,
        lang,
    };
    let composed = compose_guest_ton_released(mail, reason, rebook_url, partial);
    send_composed(config, sender, to, composed, None).await
}

pub(super) fn compose_guest_ton_released(
    mail: &TonGuestMail,
    reason: TonReleased,
    rebook_url: Option<&str>,
    partial: bool,
) -> ComposedEmail {
    let lang = lang_or_en(Some(&mail.lang));
    // A refund notice already says the money goes back.
    let footer =
        (partial && reason != TonReleased::Refund).then(|| t(lang, "ton-email-expired-partial"));
    let message_key = match reason {
        TonReleased::Expired => "ton-email-expired-message",
        TonReleased::Taken => "ton-email-taken-message",
        TonReleased::Refund => "ton-email-refund-message",
    };
    let plain_lines = rebook_url
        .map(|url| ta(lang, "ton-email-rebook-line", [("url", url)]))
        .into_iter()
        .collect();
    let actions = rebook_url
        .map(|url| EmailAction {
            label: t(lang, "email-pending-expired-action"),
            url: url.to_string(),
            color: "#6366f1".to_string(),
        })
        .into_iter()
        .collect();
    EmailParts {
        lang,
        subject: ta(
            lang,
            "ton-email-expired-subject",
            [("event", &mail.event_title), ("date", &mail.date)],
        ),
        heading: ta(lang, "email-confirm-greeting", [("name", &mail.guest_name)]),
        message: ta(
            lang,
            message_key,
            [("event", &mail.event_title), ("host", &mail.host_name)],
        ),
        accent: ACCENT_WARN,
        rows: guest_rows(lang, mail),
        plain_lines,
        footer_plain: footer.clone(),
        footer_html: footer,
        actions,
    }
    .compose()
}

fn guest_rows(lang: &str, mail: &TonGuestMail) -> Vec<EmailRow> {
    vec![
        row(t(lang, "confirmed-detail-event"), &mail.event_title),
        row(t(lang, "confirmed-detail-date"), &mail.date),
        row(t(lang, "confirmed-detail-time"), &mail.time),
        row(t(lang, "confirmed-detail-with"), &mail.host_name),
    ]
}

// --- owner emails -----------------------------------------------------------

/// A booking was paid in full and confirmed.
pub async fn send_host_ton_payment_received(
    config: &SmtpConfig,
    mail: &TonOwnerMail,
) -> Result<()> {
    let to = named_mailbox(&mail.to_name, &mail.to_email)?;
    let composed = compose_host_ton_payment_received(mail);
    send_composed(config, Sender::Instance, to, composed, None).await
}

pub(super) fn compose_host_ton_payment_received(mail: &TonOwnerMail) -> ComposedEmail {
    let lang = lang_or_en(Some(&mail.lang));
    let mut rows = booking_rows(lang, mail);
    rows.push(row(
        t(lang, "ton-email-row-amount"),
        amount_value(lang, mail),
    ));
    if !mail.excess_ton.is_empty() {
        rows.push(row(
            t(lang, "ton-email-row-excess"),
            format!("{} TON", mail.excess_ton),
        ));
    }
    push_tx_rows(lang, &mut rows, mail);
    let footer = (!mail.excess_ton.is_empty()).then(|| t(lang, "ton-email-refund-rule"));
    EmailParts {
        lang,
        subject: ta(
            lang,
            "ton-email-received-subject",
            [
                ("event", &mail.event_title),
                ("guest", &mail.guest_name),
                ("date", &mail.date),
            ],
        ),
        heading: t(lang, "ton-email-received-heading"),
        message: ta(
            lang,
            "ton-email-received-message",
            [("guest", &mail.guest_name), ("amount", &mail.paid_ton)],
        ),
        accent: ACCENT_OK,
        rows,
        plain_lines: dashboard_lines(lang, mail),
        footer_plain: footer.clone(),
        footer_html: footer,
        actions: dashboard_actions(lang, mail),
    }
    .compose()
}

/// One incoming transfer needs the owner: a partial payment, an
/// overpayment, a late or unrecognised one, a failed or unreadable one.
pub async fn send_host_ton_payment_alert(
    config: &SmtpConfig,
    mail: &TonOwnerMail,
    kind: &str,
) -> Result<()> {
    let to = named_mailbox(&mail.to_name, &mail.to_email)?;
    let composed = compose_host_ton_payment_alert(mail, kind);
    send_composed(config, Sender::Instance, to, composed, None).await
}

/// The short name and the explanation of a payment alert.
fn payment_alert_text(lang: &str, kind: &str) -> (String, String) {
    match kind {
        "partial" => (
            t(lang, "ton-alert-partial"),
            t(lang, "ton-alert-partial-body"),
        ),
        "overpaid" => (
            t(lang, "ton-alert-overpaid"),
            t(lang, "ton-alert-overpaid-body"),
        ),
        "late" => (t(lang, "ton-alert-late"), t(lang, "ton-alert-late-body")),
        "unmatched" => (
            t(lang, "ton-alert-unmatched"),
            t(lang, "ton-alert-unmatched-body"),
        ),
        "failed" => (
            t(lang, "ton-alert-failed"),
            t(lang, "ton-alert-failed-body"),
        ),
        "not_text" => (
            t(lang, "ton-alert-not-text"),
            t(lang, "ton-alert-not-text-body"),
        ),
        _ => (t(lang, "ton-alert-other"), t(lang, "ton-alert-other-body")),
    }
}

/// Whether money may have to go back for an alert of this kind.
fn refund_kind(kind: &str) -> bool {
    matches!(
        kind,
        "partial"
            | "overpaid"
            | "late"
            | "unmatched"
            | "refund_due"
            | "host_busy"
            | "host_busy_unpaid"
            | "confirm_failed"
    )
}

pub(super) fn compose_host_ton_payment_alert(mail: &TonOwnerMail, kind: &str) -> ComposedEmail {
    let lang = lang_or_en(Some(&mail.lang));
    let (what, body) = payment_alert_text(lang, kind);
    let mut rows = Vec::new();
    if !mail.value_ton.is_empty() {
        rows.push(row(
            t(lang, "ton-email-row-transfer"),
            format!("{} TON", mail.value_ton),
        ));
    }
    if let Some(sender) = &mail.sender {
        rows.push(row(
            t(lang, "ton-email-row-sender"),
            sender_value(lang, mail, sender),
        ));
    }
    push_tx_rows(lang, &mut rows, mail);
    if !mail.event_title.is_empty() {
        rows.extend(booking_rows(lang, mail));
        if !mail.amount_ton.is_empty() {
            rows.push(row(
                t(lang, "ton-email-row-amount"),
                amount_value(lang, mail),
            ));
            rows.push(row(
                t(lang, "ton-email-row-paid"),
                format!("{} TON", mail.paid_ton),
            ));
        }
    }
    let footer = refund_kind(kind).then(|| t(lang, "ton-email-refund-rule"));
    EmailParts {
        lang,
        subject: ta(lang, "ton-email-alert-subject", [("what", &what)]),
        heading: t(lang, "ton-email-alert-heading"),
        message: body,
        accent: ACCENT_WARN,
        rows,
        plain_lines: dashboard_lines(lang, mail),
        footer_plain: footer.clone(),
        footer_html: footer,
        actions: dashboard_actions(lang, mail),
    }
    .compose()
}

/// An invoice got a problem: the time is no longer free after payment, a
/// partial payment at the deadline, an overpayment, a refund due, an
/// unpaid booking found confirmed, a revoked address.
pub async fn send_host_ton_invoice_problem(
    config: &SmtpConfig,
    mail: &TonOwnerMail,
    kind: &str,
) -> Result<()> {
    let to = named_mailbox(&mail.to_name, &mail.to_email)?;
    let composed = compose_host_ton_invoice_problem(mail, kind);
    send_composed(config, Sender::Instance, to, composed, None).await
}

pub(super) fn compose_host_ton_invoice_problem(mail: &TonOwnerMail, kind: &str) -> ComposedEmail {
    let lang = lang_or_en(Some(&mail.lang));
    let body = match kind {
        "host_busy" => t(lang, "ton-problem-host-busy"),
        "partial" => t(lang, "ton-problem-partial"),
        "overpaid" => t(lang, "ton-problem-overpaid"),
        "refund_due" => t(lang, "ton-problem-refund-due"),
        "confirmed_unpaid" => t(lang, "ton-problem-confirmed-unpaid"),
        "address_revoked" => t(lang, "ton-problem-address-revoked"),
        "host_busy_unpaid" => t(lang, "ton-problem-host-busy-unpaid"),
        "confirm_failed" => t(lang, "ton-problem-confirm-failed"),
        _ => t(lang, "ton-alert-other-body"),
    };
    let subject_args = [("event", mail.event_title.as_str()), ("date", &mail.date)];
    let urgent = matches!(kind, "host_busy" | "confirm_failed");
    let subject = if urgent {
        ta(lang, "ton-email-problem-urgent-subject", subject_args)
    } else {
        ta(lang, "ton-email-problem-subject", subject_args)
    };
    let mut rows = booking_rows(lang, mail);
    rows.push(row(
        t(lang, "ton-email-row-amount"),
        amount_value(lang, mail),
    ));
    rows.push(row(
        t(lang, "ton-email-row-paid"),
        format!("{} TON", mail.paid_ton),
    ));
    if !mail.excess_ton.is_empty() {
        rows.push(row(
            t(lang, "ton-email-row-excess"),
            format!("{} TON", mail.excess_ton),
        ));
    }
    push_tx_rows(lang, &mut rows, mail);
    let footer = refund_kind(kind).then(|| t(lang, "ton-email-refund-rule"));
    EmailParts {
        lang,
        subject,
        heading: t(lang, "ton-email-problem-heading"),
        message: body,
        accent: if urgent { ACCENT_BAD } else { ACCENT_WARN },
        rows,
        plain_lines: dashboard_lines(lang, mail),
        footer_plain: footer.clone(),
        footer_html: footer,
        actions: dashboard_actions(lang, mail),
    }
    .compose()
}

/// The watcher, the rate or the limits need the operator (at most hourly
/// per kind). `detail` is a class or a count, never a key or a URL.
pub async fn send_host_ton_health(
    config: &SmtpConfig,
    to_name: &str,
    to_email: &str,
    lang: &str,
    kind: &str,
    detail: &str,
) -> Result<()> {
    let to = named_mailbox(to_name, to_email)?;
    let composed = compose_host_ton_health(lang, kind, detail);
    send_composed(config, Sender::Instance, to, composed, None).await
}

pub(super) fn compose_host_ton_health(lang: &str, kind: &str, detail: &str) -> ComposedEmail {
    let lang = lang_or_en(Some(lang));
    let body = match kind {
        "indexer_down" => t(lang, "ton-health-indexer-down"),
        "auth_failed" => t(lang, "ton-health-auth-failed"),
        "wrong_network" => t(lang, "ton-health-wrong-network"),
        "wallet_inactive" => t(lang, "ton-health-wallet-inactive"),
        "lagging" => t(lang, "ton-health-lagging"),
        "stuck_finality" => t(lang, "ton-health-stuck-finality"),
        "watcher_crashed" => t(lang, "ton-health-watcher-crashed"),
        "disabled_with_open" => t(lang, "ton-health-disabled-with-open"),
        "other_network_open" => t(lang, "ton-health-other-network-open"),
        "rate_unavailable" => t(lang, "ton-health-rate-unavailable"),
        "capacity" => t(lang, "ton-health-capacity"),
        "capacity_near" => t(lang, "ton-health-capacity-near"),
        "payment_alerts_held" => t(lang, "ton-health-payment-alerts-held"),
        _ => t(lang, "ton-health-other"),
    };
    let rows = if detail.is_empty() {
        vec![]
    } else {
        vec![row(t(lang, "ton-email-row-detail"), detail)]
    };
    EmailParts {
        lang,
        subject: ta(lang, "ton-email-health-subject", [("kind", kind)]),
        heading: t(lang, "ton-email-health-heading"),
        message: body,
        accent: ACCENT_BAD,
        rows,
        plain_lines: vec![],
        footer_plain: None,
        footer_html: None,
        actions: vec![],
    }
    .compose()
}

fn booking_rows(lang: &str, mail: &TonOwnerMail) -> Vec<EmailRow> {
    host_rows(
        lang,
        &mail.event_title,
        &mail.date,
        &mail.time,
        &mail.guest_name,
        &mail.guest_email,
    )
}

/// "12.35 TON ≈ $40.00 (1 TON = $3.2412)".
fn amount_value(lang: &str, mail: &TonOwnerMail) -> String {
    ta(
        lang,
        "ton-email-amount-value",
        [
            ("amount", &mail.amount_ton),
            ("usd", &mail.price_usd),
            ("rate", &mail.rate_usd),
        ],
    )
}

fn sender_value(lang: &str, mail: &TonOwnerMail, sender: &str) -> String {
    match mail.sender_is_wallet {
        Some(true) => ta(lang, "ton-email-sender-wallet", [("address", sender)]),
        Some(false) => ta(lang, "ton-email-sender-other", [("address", sender)]),
        None => sender.to_string(),
    }
}

fn push_tx_rows(lang: &str, rows: &mut Vec<EmailRow>, mail: &TonOwnerMail) {
    for (hash, url) in &mail.txs {
        rows.push(row(t(lang, "ton-email-row-tx"), format!("{hash} {url}")));
    }
}

fn dashboard_lines(lang: &str, mail: &TonOwnerMail) -> Vec<String> {
    mail.dashboard_url
        .as_deref()
        .map(|url| ta(lang, "ton-email-dashboard-line", [("url", url)]))
        .into_iter()
        .collect()
}

fn dashboard_actions(lang: &str, mail: &TonOwnerMail) -> Vec<EmailAction> {
    mail.dashboard_url
        .as_deref()
        .map(|url| EmailAction {
            label: t(lang, "ton-email-dashboard-action"),
            url: url.to_string(),
            color: ACCENT_GUEST.to_string(),
        })
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guest(lang: &str) -> TonGuestMail {
        TonGuestMail {
            lang: lang.into(),
            guest_name: "Ann".into(),
            guest_email: "ann@example.com".into(),
            host_name: "Ilia".into(),
            host_email: "ilia@example.com".into(),
            event_title: "Consultation".into(),
            date: "Tuesday, November 17, 2026".into(),
            time: "14:00 (Asia/Dubai)".into(),
            pay_by: String::new(),
        }
    }

    fn owner(lang: &str) -> TonOwnerMail {
        TonOwnerMail {
            lang: lang.into(),
            to_name: "Ilia".into(),
            to_email: "ilia@example.com".into(),
            event_title: "Consultation".into(),
            date: "Tuesday, November 17, 2026".into(),
            time: "14:00 (Asia/Dubai)".into(),
            guest_name: "Ann".into(),
            guest_email: "ann@example.com".into(),
            amount_ton: "12.35".into(),
            price_usd: "40.00".into(),
            rate_usd: "3.2412".into(),
            paid_ton: "12.35".into(),
            excess_ton: String::new(),
            value_ton: "12.35".into(),
            txs: vec![(
                "ab".repeat(32),
                format!("https://tonviewer.com/transaction/{}", "ab".repeat(32)),
            )],
            sender: Some("UQAHf2mwKFjEprwz5fBS5k5brROrldthRCz-CVi5eVm95EuN".into()),
            sender_is_wallet: Some(true),
            dashboard_url: Some("https://cal.example.com/dashboard/bookings".into()),
        }
    }

    fn all_text(m: &ComposedEmail) -> String {
        format!("{}\n{}\n{}", m.subject, m.plain, m.html)
    }

    #[test]
    fn the_link_email_says_nothing_about_money_or_coins() {
        for lang in ["en", "ru", "de", "fr"] {
            let m = compose_guest_ton_pay_link(
                &guest(lang),
                "https://cal.example.com/booking/pay/0123",
                Some("https://cal.example.com/booking/cancel/9"),
            );
            let text = all_text(&m);
            assert!(
                text.contains("https://cal.example.com/booking/pay/0123"),
                "{lang}"
            );
            assert!(text.contains("/booking/cancel/9"), "{lang}");
            assert!(text.contains("Consultation"));
            for word in ["TON", "12.35", "UQ", "crypto", "крипт", "$"] {
                assert!(!text.contains(word), "{lang}: {word:?} in {text}");
            }
        }
    }

    #[test]
    fn the_expiry_email_offers_another_time_and_mentions_a_partial_payment() {
        let expired = TonReleased::Expired;
        let m = compose_guest_ton_released(&guest("en"), expired, Some("https://x/u/i/c"), false);
        assert!(m.plain.contains("https://x/u/i/c"));
        let partial =
            compose_guest_ton_released(&guest("en"), expired, Some("https://x/u/i/c"), true);
        assert!(partial.plain.len() > m.plain.len());
        let ru = compose_guest_ton_released(&guest("ru"), expired, None, true);
        assert!(ru.subject.contains("Consultation"));
        assert!(!ru.plain.contains("https://x"));
    }

    #[test]
    fn each_release_reason_says_what_happened() {
        let mut texts = std::collections::HashSet::new();
        for reason in [
            TonReleased::Expired,
            TonReleased::Taken,
            TonReleased::Refund,
        ] {
            let m = compose_guest_ton_released(&guest("en"), reason, None, false);
            texts.insert(m.plain.clone());
        }
        assert_eq!(texts.len(), 3);
        let refund = compose_guest_ton_released(&guest("en"), TonReleased::Refund, None, true);
        assert!(refund.plain.contains("refund"));
        assert!(refund.plain.contains("Ilia"));
        assert!(!refund.plain.contains(&t("en", "ton-email-expired-partial")));
    }

    #[test]
    fn the_link_email_says_until_when_the_time_is_held() {
        let mut mail = guest("en");
        mail.pay_by = "14:30 (Asia/Dubai)".into();
        let m = compose_guest_ton_pay_link(&mail, "https://cal.example.com/booking/pay/0123", None);
        assert!(m.plain.contains("14:30 (Asia/Dubai)"), "{}", m.plain);
        assert!(m.plain.contains(&t("en", "ton-email-row-pay-by")));
    }

    #[test]
    fn owner_emails_carry_amounts_transactions_and_the_refund_rule() {
        let mut mail = owner("en");
        let m = compose_host_ton_payment_received(&mail);
        assert!(m.subject.contains("Consultation") && m.subject.contains("Ann"));
        assert!(m.plain.contains("12.35 TON"));
        assert!(m.plain.contains("tonviewer.com/transaction/"));
        assert!(m.plain.contains("/dashboard/bookings"));
        let rule = t("en", "ton-email-refund-rule");
        assert!(!m.plain.contains(&rule));
        mail.excess_ton = "1.00".into();
        assert!(compose_host_ton_payment_received(&mail)
            .plain
            .contains(&rule));

        for kind in [
            "partial",
            "overpaid",
            "late",
            "unmatched",
            "failed",
            "not_text",
            "bounced",
        ] {
            let m = compose_host_ton_payment_alert(&owner("ru"), kind);
            assert!(
                !m.subject.is_empty() && m.plain.contains("12.35 TON"),
                "{kind}"
            );
            assert!(m.plain.contains("UQAHf2mw"), "{kind}");
        }
        for kind in [
            "host_busy",
            "partial",
            "overpaid",
            "refund_due",
            "confirmed_unpaid",
            "address_revoked",
            "host_busy_unpaid",
            "confirm_failed",
        ] {
            let m = compose_host_ton_invoice_problem(&owner("en"), kind);
            assert!(m.subject.contains("Consultation"), "{kind}");
            assert!(
                !m.plain.contains(&t("en", "ton-alert-other-body")),
                "{kind} has its own explanation"
            );
        }
        let urgent = compose_host_ton_invoice_problem(&owner("en"), "host_busy");
        let normal = compose_host_ton_invoice_problem(&owner("en"), "refund_due");
        assert_ne!(urgent.subject, normal.subject);
    }

    #[test]
    fn each_health_kind_has_its_own_explanation() {
        let kinds = [
            "indexer_down",
            "auth_failed",
            "wrong_network",
            "wallet_inactive",
            "lagging",
            "stuck_finality",
            "watcher_crashed",
            "disabled_with_open",
            "other_network_open",
            "rate_unavailable",
            "capacity",
            "capacity_near",
            "payment_alerts_held",
        ];
        let mut bodies = std::collections::HashSet::new();
        for kind in kinds {
            let m = compose_host_ton_health("en", kind, "http_403");
            assert!(m.subject.contains(kind));
            assert!(m.plain.contains("http_403"));
            bodies.insert(m.plain);
        }
        assert_eq!(bodies.len(), kinds.len());
    }
}
