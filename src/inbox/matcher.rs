//! Which booking a reply is about.
//!
//! Candidate tokens, in this order:
//! 1. a recipient `<local>+<token>@<our domain>` (plus mode);
//! 2. a Message-ID of ours in In-Reply-To or References (`calrs.<token>.`);
//! 3. the subject tag `[#<token>]`.
//!
//! (1) and (2) are proofs that the sender received our mail, not that the
//! sender is the guest: anyone the guest forwarded or copied a message to
//! knows them too. So any From is accepted, and the match says who wrote
//! ([`Sender`]): the guest, an additional attendee, or another address. The
//! thread, Telegram and MCP show the last two as such, never as the guest.
//! A subject tag alone is accepted only from the guest's or an attendee's
//! address.
//!
//! A shared mailbox (`CALRS_INBOX_SHARED`) is also a person's own, and that
//! person writes from it too: anyone they answer or copy in carries our
//! Message-ID on in References, so there no proof is enough from another
//! address, and only the guest's or an attendee's From is accepted
//! (`known_only` in [`find`]). Mail from anyone else is the person's own and
//! stays untouched.
//!
//! Everything read here is in the headers (recipients, In-Reply-To and
//! References, Subject, From), never the body: a shared mailbox is matched
//! on its headers alone before any message is read in full (`poller`).
//!
//! Candidates are the pending or confirmed bookings that ended at most a
//! week ago (the thread's own rule): a few hundred rows at most, each token
//! recomputed and compared in constant time.

use sqlx::SqlitePool;
use subtle::ConstantTimeEq;

use super::mime::Inbound;
use super::reply::{token_shaped, token_with, tokens_in_ids, TOKEN_LEN};

#[derive(Debug, Clone)]
pub(crate) struct Candidate {
    pub(crate) id: String,
    pub(crate) uid: String,
    /// The guest's address first, then the additional attendees',
    /// lowercased.
    pub(crate) addresses: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Proof {
    Recipient,
    Reference,
    Subject,
}

impl Proof {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Proof::Recipient => "recipient",
            Proof::Reference => "reference",
            Proof::Subject => "subject",
        }
    }
}

/// Who an email reply is from, by its From address (not authenticated:
/// what a reply claims, compared with the booking's addresses).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Sender {
    /// The booking's guest address.
    Guest,
    /// One of the booking's additional attendees.
    Attendee,
    /// Neither: someone the thread reached (a forward, a CC, a list).
    Other,
}

impl Sender {
    /// The `booking_messages.sender` value.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Sender::Guest => "guest",
            Sender::Attendee => "attendee",
            Sender::Other => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Matched {
    pub(crate) booking_id: String,
    pub(crate) proof: Proof,
    pub(crate) sender: Sender,
}

/// The bookings a reply can still be about.
pub(crate) async fn candidates(pool: &SqlitePool) -> Vec<Candidate> {
    let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT b.id, b.uid, b.guest_email,
                (SELECT group_concat(a.email, char(10)) FROM booking_attendees a
                  WHERE a.booking_id = b.id)
           FROM bookings b
          WHERE b.status IN ('pending', 'confirmed')
            AND datetime(b.end_at) > datetime('now', '-7 days')",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(target: "calrs::inbox", error = %e, "could not list bookings for inbound mail");
        Vec::new()
    });
    rows.into_iter()
        .map(|(id, uid, guest, attendees)| {
            let mut addresses = vec![guest.trim().to_ascii_lowercase()];
            addresses.extend(
                attendees
                    .unwrap_or_default()
                    .lines()
                    .map(|a| a.trim().to_ascii_lowercase())
                    .filter(|a| !a.is_empty()),
            );
            Candidate { id, uid, addresses }
        })
        .collect()
}

/// The tokens of `m` by proof, each list in order of appearance.
fn tokens(m: &Inbound, domain: &str) -> Vec<(Proof, String)> {
    let mut out = Vec::new();
    for r in &m.recipients {
        if let Some((local, d)) = r.rsplit_once('@') {
            if d.eq_ignore_ascii_case(domain) {
                if let Some((_, tag)) = local.split_once('+') {
                    let tag = tag.to_ascii_lowercase();
                    if token_shaped(&tag) {
                        out.push((Proof::Recipient, tag));
                    }
                }
            }
        }
    }
    for t in tokens_in_ids(&m.references) {
        out.push((Proof::Reference, t));
    }
    let subject = &m.subject;
    for (at, _) in subject.match_indices("[#") {
        let rest = &subject[at + 2..];
        if rest.len() > TOKEN_LEN && rest.as_bytes()[TOKEN_LEN] == b']' {
            let tag = rest[..TOKEN_LEN].to_ascii_lowercase();
            if token_shaped(&tag) {
                out.push((Proof::Subject, tag));
            }
        }
    }
    out
}

/// Whether `m` carries anything token-shaped at all (a plus recipient at
/// our domain, a `calrs.` reference, a subject tag): without one, [`find`]
/// matches nothing and the bookings need not be read.
pub(crate) fn may_match(m: &Inbound, domain: &str) -> bool {
    !tokens(m, domain).is_empty()
}

/// The booking `m` is about, if any. With `known_only` (shared mode) only
/// the guest's or an attendee's From is accepted, whatever the proof.
pub(crate) fn find(
    m: &Inbound,
    candidates: &[Candidate],
    key: &[u8; 32],
    domain: &str,
    known_only: bool,
) -> Option<Matched> {
    let wanted = tokens(m, domain);
    if wanted.is_empty() {
        return None;
    }
    let from = m.from.clone().unwrap_or_default();
    let computed: Vec<(String, &Candidate)> = candidates
        .iter()
        .map(|c| (token_with(key, &c.uid), c))
        .collect();
    for (proof, token) in &wanted {
        let mut hit = None;
        for (t, c) in &computed {
            if bool::from(t.as_bytes().ct_eq(token.as_bytes())) {
                hit = Some(*c);
            }
        }
        let Some(c) = hit else {
            continue;
        };
        let sender = match c.addresses.iter().position(|a| *a == from) {
            Some(0) => Sender::Guest,
            Some(_) => Sender::Attendee,
            None => Sender::Other,
        };
        if sender == Sender::Other && (known_only || *proof == Proof::Subject) {
            continue;
        }
        return Some(Matched {
            booking_id: c.id.clone(),
            proof: *proof,
            sender,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [9u8; 32];

    fn cands() -> Vec<Candidate> {
        vec![
            Candidate {
                id: "b1".into(),
                uid: "uid-1".into(),
                addresses: vec!["guest@x.example".into(), "friend@y.example".into()],
            },
            Candidate {
                id: "b2".into(),
                uid: "uid-2".into(),
                addresses: vec!["other@z.example".into()],
            },
        ]
    }

    fn from(addr: &str) -> Inbound {
        Inbound {
            from: Some(addr.into()),
            ..Default::default()
        }
    }

    #[test]
    fn each_proof_finds_its_booking() {
        let t1 = token_with(&KEY, "uid-1");
        let t2 = token_with(&KEY, "uid-2");
        let mut m = from("stranger@q.example");
        m.recipients = vec![format!("booking+{t2}@ilia.ae")];
        let hit = find(&m, &cands(), &KEY, "ilia.ae", false).unwrap();
        assert_eq!(
            (hit.booking_id.as_str(), hit.proof, hit.sender),
            ("b2", Proof::Recipient, Sender::Other)
        );
        // Another domain's plus address is not ours.
        assert_eq!(find(&m, &cands(), &KEY, "other.example", false), None);

        let mut m = from("guest@x.example");
        m.references = format!("<a@gmail.com> <calrs.{t1}.00aa@ilia.ae>");
        let hit = find(&m, &cands(), &KEY, "ilia.ae", false).unwrap();
        assert_eq!(
            (hit.booking_id.as_str(), hit.proof, hit.sender),
            ("b1", Proof::Reference, Sender::Guest)
        );

        // An attendee is matched as an attendee, not as the guest.
        let mut m = from("friend@y.example");
        m.subject = format!("Re: Message from Ilia [#{t1}]");
        let hit = find(&m, &cands(), &KEY, "ilia.ae", false).unwrap();
        assert_eq!((hit.proof, hit.sender), (Proof::Subject, Sender::Attendee));
        // A forward's recipient replying with our Message-ID: matched, but
        // as another address.
        let mut m = from("colleague@corp.example");
        m.references = format!("<calrs.{t1}.00aa@ilia.ae>");
        assert_eq!(
            find(&m, &cands(), &KEY, "ilia.ae", false).unwrap().sender,
            Sender::Other
        );
    }

    #[test]
    fn a_subject_tag_from_a_stranger_or_a_wrong_token_matches_nothing() {
        let t1 = token_with(&KEY, "uid-1");
        let mut m = from("stranger@q.example");
        m.subject = format!("Re: [#{t1}]");
        assert_eq!(find(&m, &cands(), &KEY, "ilia.ae", false), None);
        let mut m = from("guest@x.example");
        m.subject = "Re: [#aaaaaaaaaaaaaaaa]".into();
        m.recipients = vec!["booking+bbbbbbbbbbbbbbbb@ilia.ae".into()];
        assert_eq!(find(&m, &cands(), &KEY, "ilia.ae", false), None);
        // Under another key the right uid is a wrong token.
        let mut m = from("guest@x.example");
        m.references = format!("<calrs.{t1}.00@ilia.ae>");
        assert_eq!(find(&m, &cands(), &[1u8; 32], "ilia.ae", false), None);
    }

    #[test]
    fn known_only_accepts_the_guest_and_attendees_alone() {
        let t1 = token_with(&KEY, "uid-1");
        let refs = format!("<calrs.{t1}.00aa@ilia.ae> <owner-reply@ilia.ae>");
        // Someone the mailbox's owner answered or copied in, replying to
        // the owner with our Message-ID still in References; a forward that
        // keeps it; a plus address: none is accepted from another address.
        let mut m = from("partner@p.example");
        m.references = refs.clone();
        assert!(find(&m, &cands(), &KEY, "ilia.ae", false).is_some());
        assert_eq!(find(&m, &cands(), &KEY, "ilia.ae", true), None);
        let mut m = from("partner@p.example");
        m.recipients = vec![format!("booking+{t1}@ilia.ae")];
        assert_eq!(find(&m, &cands(), &KEY, "ilia.ae", true), None);
        // The guest and an attendee still match, by any proof.
        let mut m = from("guest@x.example");
        m.references = refs;
        let hit = find(&m, &cands(), &KEY, "ilia.ae", true).unwrap();
        assert_eq!(
            (hit.booking_id.as_str(), hit.proof, hit.sender),
            ("b1", Proof::Reference, Sender::Guest)
        );
        let mut m = from("friend@y.example");
        m.subject = format!("Re: [#{t1}]");
        let hit = find(&m, &cands(), &KEY, "ilia.ae", true).unwrap();
        assert_eq!((hit.proof, hit.sender), (Proof::Subject, Sender::Attendee));
    }
}
