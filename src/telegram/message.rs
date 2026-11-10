//! Telegram message bodies, composed from the `tg-*` Fluent keys in the
//! reader's language: the booking's for the guest, the user's saved one for a
//! host. Pure functions over views the worker builds from the database, so a
//! test reads exactly what a recipient gets.
//!
//! Plain text only (no `parse_mode`): every guest- or host-written value has
//! already passed `text::clean`. Links that carry a booking token (cancel,
//! reschedule, calendar, pick a new time) are only ever put in guest
//! messages, which only go to the chat that proved that booking's token. Host
//! messages link the bookings dashboard and nothing else.

use fluent_bundle::{FluentArgs, FluentValue};

use super::api::{Button, Outgoing};
use super::events::{PaymentRelease, TelegramEvent};
use super::text::{self, MESSAGE_MAX};

fn ta(lang: &str, key: &str, args: &[(&str, &str)]) -> String {
    let mut fa = FluentArgs::new();
    for (k, v) in args {
        fa.set(*k, FluentValue::from(*v));
    }
    crate::i18n::translate(lang, key, Some(&fa))
}

/// Links a guest message may carry, each already absolute.
#[derive(Debug, Clone, Default)]
pub(crate) struct GuestLinks {
    pub(crate) cancel: Option<String>,
    pub(crate) reschedule: Option<String>,
    pub(crate) calendar: Option<String>,
    pub(crate) pick_time: Option<String>,
    pub(crate) rebook: Option<String>,
}

/// One booking as its guest sees it.
#[derive(Debug, Clone, Default)]
pub(crate) struct GuestView {
    /// Event type title, cleaned.
    pub(crate) event: String,
    /// "date, time (zone)" in the guest's zone and language.
    pub(crate) when: String,
    /// Host name(s), cleaned.
    pub(crate) host: String,
    /// The booking awaits the host's approval.
    pub(crate) pending: bool,
    /// Minutes from now to the start, for the reminder.
    pub(crate) minutes_to_start: i64,
    /// Where the meeting is: a call link or a place, cleaned. Confirmed
    /// bookings only.
    pub(crate) location: Option<String>,
    pub(crate) links: GuestLinks,
    /// The booking's TON payment, when its type is priced.
    pub(crate) payment: Option<GuestPayment>,
}

/// A booking's TON payment as its guest sees it (ilia-booking docs/ton.md).
#[derive(Debug, Clone, Default)]
pub(crate) struct GuestPayment {
    /// The invoice amount, what arrived so far and what is still due, in
    /// TON ("12.35").
    pub(crate) amount: String,
    pub(crate) paid: String,
    pub(crate) rest: String,
    /// The price in USD ("40.00").
    pub(crate) usd: String,
    /// Until when the time is held, in the guest's zone and language.
    pub(crate) deadline: String,
    /// The invoice is open: the guest can still pay.
    pub(crate) open: bool,
    /// Something arrived already.
    pub(crate) partly_paid: bool,
    /// The payment page (it carries the payment token): only ever in a
    /// guest message, which only goes to the chat linked by the booking's
    /// token. Never the comment code.
    pub(crate) pay_url: Option<String>,
}

/// One booking as a host sees it.
#[derive(Debug, Clone, Default)]
pub(crate) struct HostView {
    pub(crate) event: String,
    /// In the host's zone and language.
    pub(crate) when: String,
    /// Guest name and address, cleaned: the guest typed them. The name is
    /// also defanged (`text::defang`), so it cannot pose as a link.
    pub(crate) guest: String,
    /// Shown as typed when it is a plain address, else defanged too
    /// (`text::guest_email`).
    pub(crate) email: String,
    /// A guest's chat is linked to the booking.
    pub(crate) guest_linked: bool,
    /// The Telegram username the guest linked with, without `@`.
    pub(crate) linked_as: Option<String>,
    /// The username typed on the form, without `@`.
    pub(crate) entered: Option<String>,
    /// `{base}/dashboard/bookings`.
    pub(crate) bookings_url: Option<String>,
    /// The booking's TON payment, when its type is priced.
    pub(crate) payment: Option<HostPayment>,
}

/// A booking's TON payment as a host sees it: amounts only. There is no
/// field for the payment link or the comment code, so no host message can
/// carry them.
#[derive(Debug, Clone, Default)]
pub(crate) struct HostPayment {
    /// The invoice amount and what arrived, in TON ("12.35").
    pub(crate) amount: String,
    pub(crate) paid: String,
    /// The price in USD ("40.00").
    pub(crate) usd: String,
}

/// "Monday, 17 August 2026, 14:00 (Europe/Paris)".
pub(crate) fn when(lang: &str, date: chrono::NaiveDate, time: &str, tz: &str) -> String {
    let date = crate::i18n::format_long_date(date, lang);
    ta(
        lang,
        "tg-when",
        &[("date", &date), ("time", time), ("tz", tz)],
    )
}

/// "in 5 minutes" / "in 3 hours" / "in 2 days", with the locale's plural.
pub(crate) fn remaining(minutes: i64, lang: &str) -> String {
    let minutes = minutes.max(1);
    let mut args = FluentArgs::new();
    if minutes < 120 {
        args.set("count", crate::i18n::number(minutes as f64));
        crate::i18n::translate(lang, "tg-in-minutes", Some(&args))
    } else if minutes < 48 * 60 {
        args.set("count", crate::i18n::number(((minutes + 30) / 60) as f64));
        crate::i18n::translate(lang, "tg-in-hours", Some(&args))
    } else {
        args.set(
            "count",
            crate::i18n::number(((minutes + 720) / 1440) as f64),
        );
        crate::i18n::translate(lang, "tg-in-days", Some(&args))
    }
}

/// `@name`, or the localised "no username".
fn at(lang: &str, username: Option<&str>) -> String {
    match username {
        Some(name) => format!("@{name}"),
        None => ta(lang, "tg-host-no-username", &[]),
    }
}

/// `text`, and "Entered on the form: @…" when the guest typed another
/// username than the chat that linked has (or that chat has none).
fn with_entered(lang: &str, mut text: String, v: &HostView) -> String {
    let differs = match (&v.entered, &v.linked_as) {
        (Some(entered), Some(actual)) => !entered.eq_ignore_ascii_case(actual),
        (Some(_), None) => true,
        _ => false,
    };
    if let (true, Some(entered)) = (differs, &v.entered) {
        text.push(' ');
        text.push_str(&ta(
            lang,
            "tg-host-guest-linked-differs",
            &[("entered", &format!("@{entered}"))],
        ));
    }
    text
}

/// The message with its links: `https://` links become buttons, any other
/// (a dev instance on `http://localhost`) a `label: url` line, since Telegram
/// refuses non-HTTPS keyboard buttons. Capped at Telegram's 4096 characters.
fn finish(lang: &str, mut text: String, links: Vec<(String, Option<&String>)>) -> Outgoing {
    let mut buttons = Vec::new();
    for (label, url) in links {
        let Some(url) = url else { continue };
        if url.starts_with("https://") {
            buttons.push(Button {
                label,
                url: url.clone(),
            });
        } else {
            text.push('\n');
            text.push_str(&ta(
                lang,
                "tg-link-line",
                &[("label", &label), ("url", url)],
            ));
        }
    }
    Outgoing {
        text: text::shorten(text.trim(), MESSAGE_MAX),
        buttons,
    }
}

/// The text with a "Reason: …" line. `untrusted`: a guest wrote it and a
/// host reads it, so it is defanged like the guest's name.
fn with_reason(lang: &str, text: String, reason: Option<&String>, untrusted: bool) -> String {
    match reason
        .map(|r| text::clean(r, text::REASON_MAX))
        .map(|r| if untrusted { text::defang(&r) } else { r })
        .filter(|r| !r.is_empty())
    {
        Some(reason) => format!("{text}\n{}", ta(lang, "tg-reason", &[("reason", &reason)])),
        None => text,
    }
}

/// Where the meeting is: an `https://` call link as a "Join" button (first),
/// anything else (a place, a phone number) as a "Location:" line.
fn with_location<'a>(
    lang: &str,
    mut text: String,
    location: Option<&'a String>,
    links: &mut Vec<(String, Option<&'a String>)>,
) -> String {
    let Some(location) = location.filter(|l| !l.is_empty()) else {
        return text;
    };
    if location.starts_with("https://") && !location.contains(char::is_whitespace) {
        links.insert(0, (ta(lang, "tg-button-join", &[]), Some(location)));
    } else {
        text.push('\n');
        text.push_str(&ta(lang, "confirmed-detail-location", &[]));
        text.push(' ');
        text.push_str(location);
    }
    text
}

/// What the guest reads, or `None` when the event is not for them.
pub(crate) fn guest(event: &TelegramEvent, v: &GuestView, lang: &str) -> Option<Outgoing> {
    use TelegramEvent::*;
    let ev = v.event.as_str();
    let wh = v.when.as_str();
    let base = [("event", ev), ("when", wh)];
    let l = &v.links;
    let cancel = || (ta(lang, "tg-button-cancel", &[]), l.cancel.as_ref());
    let reschedule = || (ta(lang, "tg-button-reschedule", &[]), l.reschedule.as_ref());
    let calendar = || (ta(lang, "tg-button-calendar", &[]), l.calendar.as_ref());
    let rebook = || (ta(lang, "tg-button-rebook", &[]), l.rebook.as_ref());
    let place = v.location.as_ref().filter(|_| !v.pending);
    let msg = match event {
        BookingCreated { .. } => return None,
        GuestLinked => {
            let status = if v.pending {
                ta(lang, "tg-guest-pending", &base)
            } else {
                ta(
                    lang,
                    "tg-guest-confirmed",
                    &[("event", ev), ("when", wh), ("host", &v.host)],
                )
            };
            let text = format!("{}\n\n{status}", ta(lang, "tg-bot-linked-guest", &[]));
            let mut links = Vec::new();
            if !v.pending {
                links.push(calendar());
            }
            links.push(reschedule());
            links.push(cancel());
            let text = with_location(lang, text, place, &mut links);
            finish(lang, text, links)
        }
        Approved { reschedule: moved } => {
            let text = if *moved {
                ta(lang, "tg-guest-reschedule-approved", &base)
            } else {
                ta(lang, "tg-guest-approved", &base)
            };
            let mut links = vec![calendar(), reschedule(), cancel()];
            let text = with_location(lang, text, place, &mut links);
            finish(lang, text, links)
        }
        Declined { reason } => {
            let text = with_reason(
                lang,
                ta(lang, "tg-guest-declined", &base),
                reason.as_ref(),
                false,
            );
            finish(lang, text, vec![rebook()])
        }
        Cancelled {
            by_host: true,
            reason,
        } => {
            let text = with_reason(
                lang,
                ta(lang, "tg-guest-cancelled-by-host", &base),
                reason.as_ref(),
                false,
            );
            finish(lang, text, vec![rebook()])
        }
        // The guest wrote the reason themselves: not echoed back.
        Cancelled { by_host: false, .. } => finish(
            lang,
            ta(lang, "tg-guest-cancelled-by-guest", &base),
            vec![rebook()],
        ),
        Rescheduled { pending: false } => {
            let mut links = vec![calendar(), reschedule(), cancel()];
            let text = with_location(
                lang,
                ta(lang, "tg-guest-rescheduled", &base),
                place,
                &mut links,
            );
            finish(lang, text, links)
        }
        Rescheduled { pending: true } => finish(
            lang,
            ta(lang, "tg-guest-reschedule-pending", &base),
            vec![cancel()],
        ),
        RescheduleRequested => finish(
            lang,
            ta(lang, "tg-guest-reschedule-requested", &base),
            vec![
                (ta(lang, "tg-button-pick-time", &[]), l.pick_time.as_ref()),
                cancel(),
            ],
        ),
        Reminder => {
            let left = remaining(v.minutes_to_start, lang);
            let mut links = vec![reschedule(), cancel()];
            let text = ta(
                lang,
                "tg-guest-reminder",
                &[("event", ev), ("when", wh), ("in", &left)],
            );
            let text = with_location(lang, text, place, &mut links);
            finish(lang, text, links)
        }
        Expired => finish(lang, ta(lang, "tg-guest-expired", &base), vec![rebook()]),
        PaymentDue | PaymentPartial => {
            let Some(p) = v.payment.as_ref().filter(|p| p.open) else {
                // Paid or released meanwhile: the linked guest still gets
                // the welcome with the booking's state; a partial notice
                // has nothing left to say.
                return match event {
                    PaymentDue => guest(&GuestLinked, v, lang),
                    _ => None,
                };
            };
            let due = payment_due(lang, ev, wh, p);
            let text = match event {
                PaymentDue => format!("{}\n\n{due}", ta(lang, "tg-bot-linked-guest", &[])),
                _ => due,
            };
            let pay = (ta(lang, "tg-button-pay", &[]), p.pay_url.as_ref());
            finish(lang, text, vec![pay, cancel()])
        }
        PaymentReceived => {
            let text = ta(
                lang,
                "tg-guest-payment-received",
                &[("event", ev), ("when", wh), ("host", &v.host)],
            );
            let mut links = vec![calendar(), reschedule(), cancel()];
            let text = with_location(lang, text, place, &mut links);
            finish(lang, text, links)
        }
        PaymentLate {
            to_guest: false, ..
        } => return None,
        PaymentLate { value, .. } => finish(
            lang,
            ta(
                lang,
                "tg-guest-payment-late",
                &[
                    ("event", ev),
                    ("when", wh),
                    ("value", value),
                    ("host", &v.host),
                ],
            ),
            vec![rebook()],
        ),
        PaymentExtra { .. } => return None,
        PaymentExpired { why, partly_paid } => {
            let key = match why {
                PaymentRelease::Expired => "tg-guest-payment-expired",
                PaymentRelease::Taken => "tg-guest-payment-taken",
                PaymentRelease::Refund => "tg-guest-payment-refund",
            };
            let mut text = ta(lang, key, &[("event", ev), ("when", wh), ("host", &v.host)]);
            if *partly_paid && *why != PaymentRelease::Refund {
                text.push(' ');
                text.push_str(&ta(
                    lang,
                    "tg-guest-payment-refund-part",
                    &[("host", &v.host)],
                ));
            }
            finish(lang, text, vec![rebook()])
        }
        PaymentProblem { problem } if TelegramEvent::problem_for_guest(problem) => finish(
            lang,
            ta(
                lang,
                "tg-guest-payment-held",
                &[("event", ev), ("when", wh), ("host", &v.host)],
            ),
            Vec::new(),
        ),
        PaymentProblem { .. } => return None,
    };
    Some(msg)
}

/// "Your booking awaits payment …", or with part of it paid, "… the rest".
fn payment_due(lang: &str, event: &str, when: &str, p: &GuestPayment) -> String {
    if p.partly_paid {
        ta(
            lang,
            "tg-guest-payment-partial",
            &[
                ("event", event),
                ("when", when),
                ("paid", &p.paid),
                ("amount", &p.amount),
                ("rest", &p.rest),
                ("deadline", &p.deadline),
            ],
        )
    } else {
        ta(
            lang,
            "tg-guest-payment-due",
            &[
                ("event", event),
                ("when", when),
                ("amount", &p.amount),
                ("usd", &p.usd),
                ("deadline", &p.deadline),
            ],
        )
    }
}

/// What a host reads, or `None` when the event is not for hosts.
pub(crate) fn host(event: &TelegramEvent, v: &HostView, lang: &str) -> Option<Outgoing> {
    use TelegramEvent::*;
    let args = [
        ("event", v.event.as_str()),
        ("when", v.when.as_str()),
        ("guest", v.guest.as_str()),
        ("email", v.email.as_str()),
    ];
    let text = match event {
        BookingCreated { pending: false } => ta(lang, "tg-host-new-booking", &args),
        BookingCreated { pending: true } => ta(lang, "tg-host-approval-request", &args),
        Cancelled {
            by_host: false,
            reason,
        } => with_reason(
            lang,
            ta(lang, "tg-host-cancelled", &args),
            reason.as_ref(),
            true,
        ),
        Rescheduled { pending: false } => ta(lang, "tg-host-rescheduled", &args),
        Rescheduled { pending: true } => ta(lang, "tg-host-reschedule-approval", &args),
        GuestLinked => {
            let actual = at(lang, v.linked_as.as_deref());
            let text = ta(
                lang,
                "tg-host-guest-linked",
                &[
                    ("event", v.event.as_str()),
                    ("when", v.when.as_str()),
                    ("guest", v.guest.as_str()),
                    ("username", actual.as_str()),
                ],
            );
            with_entered(lang, text, v)
        }
        PaymentReceived => {
            let p = v.payment.as_ref()?;
            let mut text = ta(
                lang,
                "tg-host-payment-received",
                &[
                    ("event", v.event.as_str()),
                    ("when", v.when.as_str()),
                    ("guest", v.guest.as_str()),
                    ("email", v.email.as_str()),
                    ("paid", p.paid.as_str()),
                    ("usd", p.usd.as_str()),
                ],
            );
            // A priced booking's hosts were not told when the guest linked
            // a chat (it awaited its payment): who holds the booking's
            // chat, and whether that is who the guest said, comes here.
            if v.guest_linked {
                text.push('\n');
                let linked = ta(
                    lang,
                    "tg-host-payment-guest-telegram",
                    &[("username", at(lang, v.linked_as.as_deref()).as_str())],
                );
                text.push_str(&with_entered(lang, linked, v));
            }
            text
        }
        PaymentLate { value, .. } | PaymentExtra { value } => {
            let body = match event {
                PaymentLate { .. } => "ton-alert-late-body",
                _ => "ton-alert-overpaid-body",
            };
            format!(
                "{}\n{}\n{}",
                ta(lang, "tg-host-payment-problem", &args),
                ta(lang, "tg-host-payment-transfer", &[("value", value)]),
                ta(lang, body, &[]),
            )
        }
        PaymentProblem { problem } => {
            if !crate::web::ton_pay::telegram::PROBLEMS.contains(&problem.as_str()) {
                return None;
            }
            let mut text = ta(lang, "tg-host-payment-problem", &args);
            if let Some(p) = &v.payment {
                text.push('\n');
                text.push_str(&ta(
                    lang,
                    "tg-host-payment-amounts",
                    &[("paid", &p.paid), ("amount", &p.amount)],
                ));
            }
            text.push('\n');
            text.push_str(&ta(
                lang,
                &format!("ton-problem-{}", problem.replace('_', "-")),
                &[],
            ));
            text
        }
        Cancelled { by_host: true, .. }
        | Approved { .. }
        | Declined { .. }
        | RescheduleRequested
        | Reminder
        | Expired
        | PaymentDue
        | PaymentPartial
        | PaymentExpired { .. } => return None,
    };
    let bookings = ta(lang, "tg-button-bookings", &[]);
    Some(finish(
        lang,
        text,
        vec![(bookings, v.bookings_url.as_ref())],
    ))
}

/// Replies of the bot to commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reply {
    Help,
    /// A guest's booking link that no longer works.
    LinkInvalid,
    /// A priced booking's link from before the server's secret key changed:
    /// the payment page has the new one.
    LinkReplaced,
    /// A host's Profile & Settings link that no longer works.
    HostLinkInvalid,
    /// The guest opened their booking link again from the linked chat.
    AlreadyLinked,
    RateLimited,
    Stopped,
    NotLinked,
}

/// A bot reply in `lang`. `site` names this calrs (its host name).
pub(crate) fn bot_reply(reply: Reply, lang: &str, site: &str) -> Outgoing {
    let text = match reply {
        Reply::Help => ta(lang, "tg-bot-help", &[("site", site)]),
        Reply::LinkInvalid => ta(lang, "tg-bot-link-invalid", &[]),
        Reply::LinkReplaced => ta(lang, "tg-bot-link-replaced", &[]),
        Reply::HostLinkInvalid => ta(lang, "tg-bot-link-invalid-host", &[]),
        Reply::AlreadyLinked => ta(lang, "tg-bot-already-linked", &[]),
        Reply::RateLimited => ta(lang, "tg-bot-rate-limited", &[]),
        Reply::Stopped => ta(lang, "tg-bot-stopped", &[]),
        Reply::NotLinked => ta(lang, "tg-bot-not-linked", &[]),
    };
    Outgoing {
        text,
        buttons: Vec::new(),
    }
}

/// The reply to a host's `/start u_…`.
pub(crate) fn host_linked(lang: &str, name: &str) -> Outgoing {
    let name = text::clean(name, text::NAME_MAX);
    Outgoing {
        text: ta(lang, "tg-bot-linked-host", &[("name", &name)]),
        buttons: Vec::new(),
    }
}

/// The bot's profile texts in `lang` (`setMyDescription`, shown above the
/// Start button before the first message, at most 512 characters; and
/// `setMyShortDescription`, on the bot's profile, at most 120).
pub(crate) fn description(lang: &str, site: &str) -> (String, String) {
    (
        text::shorten(&ta(lang, "tg-bot-description", &[("site", site)]), 512),
        text::shorten(
            &ta(lang, "tg-bot-short-description", &[("site", site)]),
            120,
        ),
    )
}

/// The bot's command list (`setMyCommands`) in `lang`.
pub(crate) fn commands(lang: &str) -> Vec<(String, String)> {
    vec![
        ("help".to_string(), ta(lang, "tg-cmd-help", &[])),
        ("stop".to_string(), ta(lang, "tg-cmd-stop", &[])),
    ]
}
