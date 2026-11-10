//! ilia fork: a host's message to the guest of a booking (`web::messages`),
//! sent from the system address under the host's name. The guest answers by
//! replying: Reply-To is the system mailbox (`crate::inbox`), which stores
//! the answer in the thread; with no mailbox replies are not read and the
//! footer says so. In the guest's language; the text as the host wrote it,
//! line breaks kept.

use super::*;

const ACCENT_MESSAGE: &str = "#2563eb";

/// One host message, with what the guest needs to place it.
#[derive(Debug, Clone, Default)]
pub struct HostMessageMail {
    /// The guest's language (a calrs language code).
    pub lang: String,
    pub guest_name: String,
    pub guest_email: String,
    /// Who wrote it: the name, and (outside relay mode only, upstream's
    /// tests) the address replies go to.
    pub host_name: String,
    pub host_email: String,
    /// The booking's uid: its reply token and subject tag.
    pub booking_uid: String,
    pub event_title: String,
    /// The meeting's date and time in the guest's zone, written out.
    pub date: String,
    pub time: String,
    /// The host's text, cleaned (`telegram::text::clean_lines`).
    pub body: String,
}

/// Send it to the guest.
pub async fn send_guest_host_message(config: &SmtpConfig, mail: &HostMessageMail) -> Result<()> {
    let to = named_mailbox(&mail.guest_name, &mail.guest_email)?;
    let lang = lang_or_en(Some(&mail.lang));
    let sender = Sender::Host {
        name: &mail.host_name,
        email: &mail.host_email,
        booking: Some(&mail.booking_uid),
        lang,
    };
    let composed = compose_guest_host_message(mail);
    send_composed(config, sender, to, composed, None).await
}

/// Subject, plain text and HTML of a host message.
pub(super) fn compose_guest_host_message(mail: &HostMessageMail) -> ComposedEmail {
    let lang = lang_or_en(Some(&mail.lang));
    let mut subject = ta(
        lang,
        "msg-email-subject",
        [
            ("host", &mail.host_name),
            ("event", &mail.event_title),
            ("date", &mail.date),
        ],
    );
    // The tag finds the booking again when a client drops the headers.
    if let Some(tag) = crate::inbox::reply::subject_tag(&mail.booking_uid) {
        subject.push_str(&tag);
    }
    let heading = ta(lang, "email-confirm-greeting", [("name", &mail.guest_name)]);
    let intro = ta(
        lang,
        "msg-email-intro",
        [("host", &mail.host_name), ("event", &mail.event_title)],
    );
    // Replies are read only when the system mailbox is (relay mode without
    // one: the reply would reach an unread From).
    let footer_key = if crate::inbox::relay() && !crate::inbox::enabled() {
        "msg-email-footer-no-reply"
    } else {
        "msg-email-footer"
    };
    let footer = ta(lang, footer_key, [("host", &mail.host_name)]);
    let rows = vec![
        row(t(lang, "confirmed-detail-event"), &mail.event_title),
        row(t(lang, "confirmed-detail-date"), &mail.date),
        row(t(lang, "confirmed-detail-time"), &mail.time),
        row(t(lang, "confirmed-detail-with"), &mail.host_name),
    ];
    let mut plain = format!("{heading}\n\n{intro}\n\n{}\n\n", mail.body);
    for r in &rows {
        plain.push_str(&format!("{} {}\n", r.label, r.value));
    }
    plain.push('\n');
    plain.push_str(&footer);
    plain.push_str("\n\n");
    plain.push_str(&t(lang, "email-signature"));
    // The text keeps its lines: escaped first, then each break a <br>.
    let message = format!(
        "{}<br><br><span style=\"font-weight:400;\">{}</span>",
        h(&intro),
        h(&mail.body).replace('\n', "<br>")
    );
    let html = render_html_email_in(
        lang,
        &h(&heading),
        &message,
        ACCENT_MESSAGE,
        &rows,
        Some(&footer),
        &[],
    );
    ComposedEmail {
        subject,
        plain,
        html,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mail(lang: &str) -> HostMessageMail {
        HostMessageMail {
            lang: lang.into(),
            guest_name: "Eve".into(),
            guest_email: "eve@example.test".into(),
            host_name: "Ilia".into(),
            host_email: "ilia@example.test".into(),
            booking_uid: "uid-msg-1".into(),
            event_title: "Product demo".into(),
            date: "Monday, 17 August 2026".into(),
            time: "14:00 (Europe/Paris)".into(),
            body: "Hello Eve,\nplease bring the <slides>.".into(),
        }
    }

    #[test]
    fn the_text_keeps_its_lines_and_is_escaped() {
        let m = compose_guest_host_message(&mail("en"));
        assert_eq!(
            m.subject,
            "Message from Ilia: Product demo, Monday, 17 August 2026"
        );
        assert!(
            m.plain.contains("Hello Eve,\nplease bring the <slides>."),
            "{}",
            m.plain
        );
        assert!(m.plain.contains("reply to this email"), "{}", m.plain);
        assert!(
            m.html
                .contains("Hello Eve,<br>please bring the &lt;slides&gt;."),
            "{}",
            m.html
        );
        assert!(!m.html.contains("<slides>"));
    }

    #[test]
    fn every_locale_writes_it() {
        for (lang, _) in crate::i18n::supported_with_labels() {
            let m = compose_guest_host_message(&mail(lang));
            for text in [&m.subject, &m.plain] {
                assert!(!text.contains("msg-"), "{lang}: {text}");
                assert!(text.contains("Ilia"), "{lang}: {text}");
            }
            assert!(m.plain.contains("please bring"), "{lang}");
        }
    }
}
