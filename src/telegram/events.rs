//! What happened to a booking, and who hears about it.

/// What happened to a booking. One entry point, `crate::telegram::notify`,
/// serves every call site; who hears about it is decided by [`audience`],
/// and what they read by `message::guest` and `message::host`.
///
/// Payment events (TON, ilia-booking docs/ton.md) are the `Payment*`
/// variants, queued by `crate::web::ton_pay::telegram`. They carry owned,
/// non-secret data only: the amounts and the payment link are read from the
/// invoice when the message is written, and the link goes to the guest's own
/// chat only, never to a host (nor does the comment code).
///
/// [`audience`]: TelegramEvent::audience
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TelegramEvent {
    /// Just submitted. Hosts: "new booking" or "request to approve". The
    /// guest has not linked a chat yet.
    BookingCreated { pending: bool },
    /// A host approved a request (`reschedule`: a guest's new time).
    Approved { reschedule: bool },
    /// A host declined a request. The reason is host-written, for the guest.
    Declined { reason: Option<String> },
    /// Cancelled by a host (dashboard, agent, calendar deletion) or by the
    /// guest. A guest's reason is shown to the hosts, cleaned.
    Cancelled {
        by_host: bool,
        reason: Option<String>,
    },
    /// The guest moved it; `pending` when the new time awaits approval.
    Rescheduled { pending: bool },
    /// A host asks the guest to pick a new time.
    RescheduleRequested,
    /// The reminder loop, at the event type's `reminder_minutes`.
    Reminder,
    /// A pending request passed its deadline.
    Expired,
    /// The guest linked a chat (`/start` with the booking's token). Guest:
    /// the booking's current state; hosts: who linked, as Telegram reports.
    /// A booking that awaits its TON payment answers [`PaymentDue`] instead.
    ///
    /// [`PaymentDue`]: TelegramEvent::PaymentDue
    GuestLinked,
    /// TON: the guest linked a chat to a booking that awaits its payment.
    /// Guest: the amount, the deadline and the payment page. Hosts: nothing
    /// (they hear about a priced booking once it is paid).
    PaymentDue,
    /// TON: part of the amount arrived. Guest: what is left, and by when.
    PaymentPartial,
    /// TON: paid in full and confirmed. Guest: confirmed; hosts: paid.
    PaymentReceived,
    /// TON: a transfer of `value` TON arrived after the payment window.
    /// Guest (`to_guest`: it came from the wallet that paid first, see
    /// `ton_pay::telegram::from_first_payer`): it did not confirm the
    /// booking; hosts: refund it.
    PaymentLate { value: String, to_guest: bool },
    /// TON: a transfer of `value` TON for a booking already paid. Hosts.
    PaymentExtra { value: String },
    /// TON: the expiry sweep released a priced booking that was not
    /// confirmed. `partly_paid`: some of it arrived and goes back.
    PaymentExpired {
        why: PaymentRelease,
        partly_paid: bool,
    },
    /// TON: the invoice's `problem` (`ton_invoices.problem`), for the hosts.
    /// The guest hears about the two that keep a paid booking unconfirmed
    /// (`host_busy`, `confirm_failed`).
    PaymentProblem { problem: String },
    /// TON (ilia fork): the payment window ended unpaid while the meeting
    /// is far away, so it was renewed (the `renewal`th time). Guest: what is
    /// still due and the new deadline. Hosts: nothing.
    PaymentRenewed { renewal: u8 },
    /// ilia fork (messages): a host wrote to the guest (`booking_messages`
    /// row `id`). Guest: the text. `text` is host-written, already cleaned.
    HostMessage { id: String, text: String },
    /// ilia fork (messages): the guest wrote in their linked chat (row
    /// `id`). Hosts: the text, cleaned and defanged when composed, with a
    /// "reply to answer" line. `text` is guest-written: never logged.
    /// `email`: it came as an email reply to the system mailbox
    /// (`crate::inbox`), not in the guest's chat, and who sent it.
    GuestMessage {
        id: String,
        text: String,
        email: Option<MailFrom>,
    },
}

/// ilia fork (inbox): who sent an email reply. `address` is the From of an
/// attendee or another sender (unverified, shown to hosts only, never
/// logged); empty for the guest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MailFrom {
    pub(crate) sender: crate::inbox::matcher::Sender,
    pub(crate) address: String,
}

/// Why the expiry sweep released a priced booking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaymentRelease {
    /// The payment window closed without the full amount.
    Expired,
    /// The host's time was taken before the booking was paid.
    Taken,
    /// Paid, but it could not be confirmed: the host refunds.
    Refund,
}

impl TelegramEvent {
    /// A TON payment event: the hosts of a booking that awaits its payment
    /// hear these and nothing else.
    pub(crate) fn is_payment(&self) -> bool {
        use TelegramEvent::*;
        matches!(
            self,
            PaymentDue
                | PaymentPartial
                | PaymentReceived
                | PaymentLate { .. }
                | PaymentExtra { .. }
                | PaymentExpired { .. }
                | PaymentProblem { .. }
                | PaymentRenewed { .. }
        )
    }

    /// The guest hears about this invoice problem too.
    pub(crate) fn problem_for_guest(problem: &str) -> bool {
        matches!(problem, "host_busy" | "confirm_failed")
    }
}

/// Who an event is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Audience {
    pub(crate) guest: bool,
    pub(crate) hosts: bool,
}

impl TelegramEvent {
    pub(crate) fn audience(&self) -> Audience {
        use TelegramEvent::*;
        let (guest, hosts) = match self {
            BookingCreated { .. } => (false, true),
            Approved { .. } => (true, false),
            Declined { .. } => (true, false),
            Cancelled { by_host, .. } => (true, !by_host),
            Rescheduled { .. } => (true, true),
            RescheduleRequested => (true, false),
            Reminder => (true, false),
            Expired => (true, false),
            GuestLinked => (true, true),
            PaymentDue | PaymentPartial => (true, false),
            PaymentReceived => (true, true),
            PaymentLate { to_guest, .. } => (*to_guest, true),
            PaymentExtra { .. } => (false, true),
            PaymentExpired { .. } => (true, false),
            PaymentProblem { problem } => (Self::problem_for_guest(problem), true),
            PaymentRenewed { .. } => (true, false),
            HostMessage { .. } => (true, false),
            GuestMessage { .. } => (false, true),
        };
        Audience { guest, hosts }
    }

    /// Short label for logs.
    pub(crate) fn kind(&self) -> &'static str {
        use TelegramEvent::*;
        match self {
            BookingCreated { pending: false } => "booking_created",
            BookingCreated { pending: true } => "booking_requested",
            Approved { reschedule: false } => "approved",
            Approved { reschedule: true } => "reschedule_approved",
            Declined { .. } => "declined",
            Cancelled { by_host: true, .. } => "cancelled_by_host",
            Cancelled { by_host: false, .. } => "cancelled_by_guest",
            Rescheduled { pending: false } => "rescheduled",
            Rescheduled { pending: true } => "reschedule_requested_by_guest",
            RescheduleRequested => "pick_new_time",
            Reminder => "reminder",
            Expired => "expired",
            GuestLinked => "guest_linked",
            PaymentDue => "payment_due",
            PaymentPartial => "payment_partial",
            PaymentReceived => "payment_received",
            PaymentLate { .. } => "payment_late",
            PaymentExtra { .. } => "payment_extra",
            PaymentExpired {
                why: PaymentRelease::Expired,
                ..
            } => "payment_expired",
            PaymentExpired {
                why: PaymentRelease::Taken,
                ..
            } => "payment_released_taken",
            PaymentExpired {
                why: PaymentRelease::Refund,
                ..
            } => "payment_released_refund",
            PaymentProblem { .. } => "payment_problem",
            PaymentRenewed { .. } => "payment_renewed",
            HostMessage { .. } => "host_message",
            GuestMessage { .. } => "guest_message",
        }
    }

    /// Every kind of event, one value each, for tests.
    #[cfg(test)]
    pub(crate) fn samples() -> Vec<TelegramEvent> {
        use TelegramEvent::*;
        vec![
            BookingCreated { pending: false },
            BookingCreated { pending: true },
            Approved { reschedule: false },
            Approved { reschedule: true },
            Declined {
                reason: Some("Away that week".into()),
            },
            Declined { reason: None },
            Cancelled {
                by_host: true,
                reason: Some("Sick".into()),
            },
            Cancelled {
                by_host: false,
                reason: Some("Conflict".into()),
            },
            Rescheduled { pending: false },
            Rescheduled { pending: true },
            RescheduleRequested,
            Reminder,
            Expired,
            GuestLinked,
        ]
    }

    /// The message events (a guest text by Telegram and by email), for tests.
    #[cfg(test)]
    pub(crate) fn message_samples() -> Vec<TelegramEvent> {
        vec![
            TelegramEvent::HostMessage {
                id: "m1".into(),
                text: "See you on Monday.\nBring the slides.".into(),
            },
            TelegramEvent::GuestMessage {
                id: "m2".into(),
                text: "Can we talk about pricing?".into(),
                email: None,
            },
            TelegramEvent::GuestMessage {
                id: "m3".into(),
                text: "Can we talk about pricing?".into(),
                email: Some(MailFrom {
                    sender: crate::inbox::matcher::Sender::Guest,
                    address: String::new(),
                }),
            },
            TelegramEvent::GuestMessage {
                id: "m4".into(),
                text: "Can we talk about pricing?".into(),
                email: Some(MailFrom {
                    sender: crate::inbox::matcher::Sender::Attendee,
                    address: "friend@example.test".into(),
                }),
            },
            TelegramEvent::GuestMessage {
                id: "m5".into(),
                text: "Can we talk about pricing?".into(),
                email: Some(MailFrom {
                    sender: crate::inbox::matcher::Sender::Other,
                    address: "colleague@example.test".into(),
                }),
            },
        ]
    }

    /// Every kind of payment event, one value each (each problem), for tests.
    #[cfg(test)]
    pub(crate) fn payment_samples() -> Vec<TelegramEvent> {
        use TelegramEvent::*;
        let mut all = vec![
            PaymentDue,
            PaymentPartial,
            PaymentRenewed { renewal: 1 },
            PaymentReceived,
            PaymentLate {
                value: "1.5".into(),
                to_guest: true,
            },
            PaymentLate {
                value: "1.5".into(),
                to_guest: false,
            },
            PaymentExtra {
                value: "2.25".into(),
            },
        ];
        for why in [
            PaymentRelease::Expired,
            PaymentRelease::Taken,
            PaymentRelease::Refund,
        ] {
            for partly_paid in [false, true] {
                all.push(PaymentExpired { why, partly_paid });
            }
        }
        for problem in crate::web::ton_pay::telegram::PROBLEMS {
            all.push(PaymentProblem {
                problem: problem.to_string(),
            });
        }
        all
    }
}
