//! What one incoming transaction means for us: the decision table of
//! docs/ton.md ("Решение по каждой транзакции"), as pure functions.
//!
//! [`classify`] covers rows 1–6 from the transaction alone; for a candidate
//! (a native TON transfer with a text comment or none), [`against_invoice`]
//! covers rows 7–12 once the invoice the comment names has been looked up.
//! Deduplication by transaction hash (row 1's second half) is the primary
//! key of `ton_payments`; this module covers the `lt` half.
//!
//! [`parse_tx`] never panics on any JSON: no indexing, no unwrap.

use serde_json::Value;

use super::toncenter::{hash_to_hex, value_u128, value_u64, BookEntry, TxPage};

/// The finality of a transaction, as a string (live answers) or a number
/// (the published spec: 0 pending, 1 confirmed, 2 finalized).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finality {
    Finalized,
    NotYet,
    /// No field: decided by `mc_block_seqno`.
    Missing,
    /// Anything else: wait, and alert if the cursor stays stuck on it.
    Unknown(String),
}

impl Finality {
    fn from_value(v: Option<&Value>) -> Self {
        match v {
            None | Some(Value::Null) => Self::Missing,
            Some(Value::String(s)) => match s.as_str() {
                "finalized" => Self::Finalized,
                "pending" | "confirmed" => Self::NotYet,
                other => Self::Unknown(other.chars().take(32).collect()),
            },
            Some(Value::Number(n)) => match n.as_u64() {
                Some(2) => Self::Finalized,
                Some(0 | 1) => Self::NotYet,
                _ => Self::Unknown(n.to_string()),
            },
            Some(other) => Self::Unknown(other.to_string().chars().take(32).collect()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InMsg {
    pub source: Option<String>,
    pub destination: Option<String>,
    pub value_nano: u128,
    pub bounced: bool,
    /// `None`: no opcode (an empty body).
    pub opcode: Option<u32>,
    /// `message_content.decoded.type`, when decoded.
    pub decoded_type: Option<String>,
    /// The text comment, when the body is one.
    pub comment: Option<String>,
}

/// The fields of a transaction we decide on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedTx {
    pub account: String,
    pub hash_hex: String,
    pub lt: u64,
    pub now: u64,
    pub mc_block_seqno: Option<u64>,
    pub finality: Finality,
    pub emulated: bool,
    pub desc_type: Option<String>,
    pub aborted: bool,
    pub compute_skipped: bool,
    pub compute_success: bool,
    pub action_success: Option<bool>,
    pub in_msg: Option<InMsg>,
}

impl ParsedTx {
    /// In a masterchain block AND finalized (or no finality field at all).
    pub fn is_final(&self) -> bool {
        if self.emulated || self.mc_block_seqno.is_none() {
            return false;
        }
        matches!(self.finality, Finality::Finalized | Finality::Missing)
    }

    pub fn value_nano(&self) -> u128 {
        self.in_msg.as_ref().map_or(0, |m| m.value_nano)
    }

    pub fn source(&self) -> Option<&str> {
        self.in_msg.as_ref().and_then(|m| m.source.as_deref())
    }
}

/// A transaction that cannot be read. `lt` and `hash_hex` are set when at
/// least those parsed, so it can still be recorded and passed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseIssue {
    pub what: &'static str,
    pub lt: Option<u64>,
    pub hash_hex: Option<String>,
    pub final_: bool,
}

fn opcode_of(v: Option<&Value>) -> Result<Option<u32>, ()> {
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => n
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or(()),
        Some(Value::String(s)) => {
            let s = s.trim();
            let parsed = match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                Some(hex) => u32::from_str_radix(hex, 16),
                None => s.parse::<u32>(),
            };
            parsed.map(Some).map_err(|_| ())
        }
        Some(_) => Err(()),
    }
}

fn opt_raw(v: Option<&Value>) -> Result<Option<String>, ()> {
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => super::address::normalize_raw(s).map(Some).ok_or(()),
        Some(_) => Err(()),
    }
}

fn bool_or(v: Option<&Value>, default: bool) -> bool {
    v.and_then(Value::as_bool).unwrap_or(default)
}

pub fn parse_tx(v: &Value) -> Result<ParsedTx, ParseIssue> {
    let lt = v.get("lt").and_then(value_u64);
    let hash_hex = v.get("hash").and_then(Value::as_str).and_then(hash_to_hex);
    let mc_block_seqno = v.get("mc_block_seqno").and_then(value_u64);
    let finality = Finality::from_value(v.get("finality"));
    let emulated = bool_or(v.get("emulated"), false);
    let final_ = !emulated
        && mc_block_seqno.is_some()
        && matches!(finality, Finality::Finalized | Finality::Missing);
    let issue = |what: &'static str| ParseIssue {
        what,
        lt,
        hash_hex: hash_hex.clone(),
        final_,
    };
    let (Some(lt), Some(hash)) = (lt, hash_hex.clone()) else {
        return Err(issue("lt_or_hash"));
    };
    let account = v
        .get("account")
        .and_then(Value::as_str)
        .and_then(super::address::normalize_raw)
        .ok_or_else(|| issue("account"))?;
    let now = v
        .get("now")
        .and_then(value_u64)
        .ok_or_else(|| issue("now"))?;
    let desc = v.get("description");
    let desc_type = desc
        .and_then(|d| d.get("type"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let aborted = bool_or(desc.and_then(|d| d.get("aborted")), true);
    let compute = desc.and_then(|d| d.get("compute_ph"));
    let compute_skipped = bool_or(compute.and_then(|c| c.get("skipped")), true);
    let compute_success = bool_or(compute.and_then(|c| c.get("success")), false);
    let action_success = desc
        .and_then(|d| d.get("action"))
        .filter(|a| !a.is_null())
        .map(|a| bool_or(a.get("success"), false));
    let in_msg = match v.get("in_msg") {
        None | Some(Value::Null) => None,
        Some(m) if m.is_object() => {
            let source = opt_raw(m.get("source")).map_err(|_| issue("in_msg.source"))?;
            let destination =
                opt_raw(m.get("destination")).map_err(|_| issue("in_msg.destination"))?;
            let value_nano = match m.get("value") {
                None | Some(Value::Null) if source.is_none() => 0,
                Some(val) => value_u128(val).ok_or_else(|| issue("in_msg.value"))?,
                None => return Err(issue("in_msg.value")),
            };
            let opcode = opcode_of(m.get("opcode")).map_err(|_| issue("in_msg.opcode"))?;
            let decoded = m.get("message_content").and_then(|c| c.get("decoded"));
            let decoded_type = decoded
                .and_then(|d| d.get("type").or_else(|| d.get("@type")))
                .and_then(Value::as_str)
                .map(str::to_string);
            let comment = if decoded_type.as_deref() == Some("text_comment") {
                decoded
                    .and_then(|d| d.get("comment"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            } else {
                None
            };
            Some(InMsg {
                source,
                destination,
                value_nano,
                bounced: bool_or(m.get("bounced"), false),
                opcode,
                decoded_type,
                comment,
            })
        }
        Some(_) => return Err(issue("in_msg")),
    };
    Ok(ParsedTx {
        account,
        hash_hex: hash,
        lt,
        now,
        mc_block_seqno,
        finality,
        emulated,
        desc_type,
        aborted,
        compute_skipped,
        compute_success,
        action_success,
        in_msg,
    })
}

/// Why an incoming transfer does not count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IgnoreReason {
    /// Our own outgoing transfer bounced back.
    Bounced,
    /// Aborted, or compute/action phase failed.
    Failed,
    /// Not a plain TON transfer with a text comment (jettons, NFTs, an
    /// encrypted comment).
    NotText,
    /// Dated more than a minute before the invoice it names.
    BeforeInvoice,
    /// toncenter answered something we cannot read.
    Unparsed,
}

impl IgnoreReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bounced => "bounced",
            Self::Failed => "failed",
            Self::NotText => "not_text",
            Self::BeforeInvoice => "before_invoice",
            Self::Unparsed => "unparsed",
        }
    }
}

/// Rows 1–6 (and 7's "is it a candidate at all").
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Class {
    /// Row 1: at or below the cursor, a repeat. Nothing at all.
    SkipSilent,
    /// Row 2: not final (or emulated). The cursor stops here.
    Stop,
    /// Row 3: not an incoming internal message to us. The cursor passes.
    Pass,
    /// Rows 4–6.
    Ignored(IgnoreReason),
    /// A native TON transfer; `reference` is the normalised code when the
    /// whole comment is one.
    Candidate { reference: Option<String> },
}

pub fn classify(tx: &ParsedTx, our_raw: &str, cursor_lt: u64) -> Class {
    // 1
    if tx.lt <= cursor_lt {
        return Class::SkipSilent;
    }
    // 2
    if !tx.is_final() {
        return Class::Stop;
    }
    // 3
    let Some(msg) = &tx.in_msg else {
        return Class::Pass;
    };
    if tx.account != our_raw
        || tx.desc_type.as_deref() != Some("ord")
        || msg.source.is_none()
        || msg.destination.as_deref() != Some(our_raw)
    {
        return Class::Pass;
    }
    // 4
    if msg.bounced {
        return Class::Ignored(IgnoreReason::Bounced);
    }
    // 5
    if tx.aborted || tx.compute_skipped || !tx.compute_success || tx.action_success == Some(false) {
        return Class::Ignored(IgnoreReason::Failed);
    }
    // 6: op = 0 with a decoded text comment, or no body at all (a plain
    // transfer without a comment: money arrived, nothing names an invoice).
    match (msg.opcode, msg.decoded_type.as_deref()) {
        (Some(0), Some("text_comment")) => Class::Candidate {
            reference: msg.comment.as_deref().and_then(super::reference::normalize),
        },
        (None, None) => Class::Candidate { reference: None },
        _ => Class::Ignored(IgnoreReason::NotText),
    }
}

/// The invoice a code names, as the decision needs it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvoiceView {
    pub network: super::Network,
    pub address_raw: String,
    pub status: InvoiceStatus,
    pub created_unix: i64,
    pub settle_by_unix: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvoiceStatus {
    Open,
    Paid,
    Confirmed,
    Expired,
    Closed,
    Waived,
}

impl InvoiceStatus {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "open" => Self::Open,
            "paid" => Self::Paid,
            "confirmed" => Self::Confirmed,
            "expired" => Self::Expired,
            "closed" => Self::Closed,
            "waived" => Self::Waived,
            _ => return None,
        })
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Paid => "paid",
            Self::Confirmed => "confirmed",
            Self::Expired => "expired",
            Self::Closed => "closed",
            Self::Waived => "waived",
        }
    }
}

/// Rows 7–12.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// 7, 8: no invoice for the code (or none in the comment), or the code
    /// of an invoice on another address or network.
    Unmatched { matched_elsewhere: bool },
    /// 9
    BeforeInvoice,
    /// 10: the invoice is already settled; all of it is excess.
    Overpaid,
    /// 11: the invoice is closed or expired, or sent after `settle_by`.
    Late,
    /// 12: `paid_nano += value`.
    Credited,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unmatched { .. } => "unmatched",
            Self::BeforeInvoice => "ignored",
            Self::Overpaid => "overpaid",
            Self::Late => "late",
            Self::Credited => "credited",
        }
    }
}

/// A transfer is "before the invoice" only when dated over a minute before
/// it (clock and block-time slack).
const BEFORE_INVOICE_SLACK_SECS: i64 = 60;

pub fn against_invoice(
    tx: &ParsedTx,
    network: super::Network,
    invoice: Option<&InvoiceView>,
) -> Outcome {
    // 7
    let Some(inv) = invoice else {
        return Outcome::Unmatched {
            matched_elsewhere: false,
        };
    };
    // 8
    if inv.address_raw != tx.account || inv.network != network {
        return Outcome::Unmatched {
            matched_elsewhere: true,
        };
    }
    let now = i64::try_from(tx.now).unwrap_or(i64::MAX);
    // 9
    if now < inv.created_unix - BEFORE_INVOICE_SLACK_SECS {
        return Outcome::BeforeInvoice;
    }
    match inv.status {
        // 10
        InvoiceStatus::Paid | InvoiceStatus::Confirmed | InvoiceStatus::Waived => Outcome::Overpaid,
        // 11
        InvoiceStatus::Expired | InvoiceStatus::Closed => Outcome::Late,
        InvoiceStatus::Open if now > inv.settle_by_unix => Outcome::Late,
        // 12
        InvoiceStatus::Open => Outcome::Credited,
    }
}

/// The sender as stored and shown to the owner: friendly form from the
/// address book, and whether it is a wallet.
pub fn sender_info(tx: &ParsedTx, page: &TxPage) -> (Option<String>, Option<bool>) {
    let Some(src) = tx.source() else {
        return (None, None);
    };
    match page.address_book.get(src) {
        Some(BookEntry {
            user_friendly,
            interfaces,
        }) => (
            user_friendly.clone(),
            Some(interfaces.iter().any(|i| i.starts_with("wallet"))),
        ),
        None => (None, None),
    }
}

/// A sender's comment as stored: control characters dropped, at most 128
/// characters. Still sender text: never put in an email or a guest page.
pub fn clean_comment(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(128)
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::toncenter::{parse_tx_page, tests::fixture};
    use super::super::Network;
    use super::*;
    use serde_json::json;

    pub(crate) const TF: &str =
        "0:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8";

    /// The recorded transactions of the TON Foundation address, ascending:
    /// [0] aborted no_gas + bounce nofunds text comment, [1] jetton notify
    /// (aborted no_gas), [2] a normal text comment transfer.
    pub(crate) fn recorded() -> Vec<Value> {
        parse_tx_page(&fixture("mainnet-transactions-tf.json"))
            .unwrap()
            .transactions
    }

    /// A derived transaction: the recorded normal transfer with `edit`
    /// applied. Every derived case starts from a real answer.
    pub(crate) fn builder(edit: impl FnOnce(&mut Value)) -> Value {
        let mut tx = recorded()[2].clone();
        edit(&mut tx);
        tx
    }

    /// The normal transfer carrying `comment` and `value` at `lt`.
    pub(crate) fn payment(lt: u64, now: u64, value: u128, comment: &str) -> Value {
        builder(|t| {
            t["lt"] = json!(lt.to_string());
            t["now"] = json!(now);
            t["hash"] = json!(base64_hash(lt));
            t["in_msg"]["value"] = json!(value.to_string());
            t["in_msg"]["message_content"]["decoded"]["comment"] = json!(comment);
        })
    }

    pub(crate) fn base64_hash(seed: u64) -> String {
        use base64::Engine;
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&seed.to_be_bytes());
        bytes[31] = 0xAB;
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    fn class_of(v: &Value) -> Class {
        classify(&parse_tx(v).unwrap(), TF, 0)
    }

    #[test]
    fn the_recorded_live_transactions() {
        let txs = recorded();
        let parsed: Vec<ParsedTx> = txs.iter().map(|t| parse_tx(t).unwrap()).collect();
        // `now` is an integer and `lt` a string in live answers.
        assert!(txs[2]["now"].is_u64() && txs[2]["lt"].is_string());
        assert!(parsed.iter().all(ParsedTx::is_final));
        // A tiny comment transfer that ran out of gas: failed (row 5).
        assert_eq!(
            classify(&parsed[0], TF, 0),
            Class::Ignored(IgnoreReason::Failed)
        );
        assert!(parsed[0].aborted && parsed[0].compute_skipped);
        // A jetton notify (aborted no_gas here too): row 5 before row 6.
        assert_eq!(
            classify(&parsed[1], TF, 0),
            Class::Ignored(IgnoreReason::Failed)
        );
        assert_eq!(parsed[1].in_msg.as_ref().unwrap().opcode, Some(0x7362_d09c));
        // A real text comment: a candidate whose comment is no code.
        assert_eq!(
            classify(&parsed[2], TF, 0),
            Class::Candidate { reference: None }
        );
        assert_eq!(
            parsed[2].in_msg.as_ref().unwrap().comment.as_deref(),
            Some("NordHash Whale Verification")
        );
        assert_eq!(parsed[2].value_nano(), 2_433_333);
        assert_eq!(parsed[2].hash_hex.len(), 64);
    }

    #[test]
    fn a_comment_with_a_code_is_a_candidate_with_it() {
        let tx = payment(200, 1_794_222_938, 12_350_000_000, "b-7kq3 m9xd-2f");
        assert_eq!(
            class_of(&tx),
            Class::Candidate {
                reference: Some("B7KQ3M9XD2F".into())
            }
        );
        let tx = payment(200, 1_794_222_938, 12_350_000_000, "B-7KQ3M9XD2F thanks");
        assert_eq!(class_of(&tx), Class::Candidate { reference: None });
    }

    #[test]
    fn row_1_repeats_at_or_below_the_cursor() {
        let tx = parse_tx(&payment(500, 1, 1, "B-7KQ3M9XD2F")).unwrap();
        assert_eq!(classify(&tx, TF, 500), Class::SkipSilent);
        assert_eq!(classify(&tx, TF, 501), Class::SkipSilent);
        assert_ne!(classify(&tx, TF, 499), Class::SkipSilent);
    }

    #[test]
    fn row_2_finality_both_shapes() {
        for (finality, want_stop) in [
            (json!("finalized"), false),
            (json!(2), false),
            (json!("confirmed"), true),
            (json!("pending"), true),
            (json!(1), true),
            (json!(0), true),
            (json!("weird"), true),
            (json!(7), true),
            (json!(true), true),
        ] {
            let tx = builder(|t| t["finality"] = finality.clone());
            assert_eq!(class_of(&tx) == Class::Stop, want_stop, "{finality}");
        }
        // No field: decided by mc_block_seqno.
        let tx = builder(|t| {
            t.as_object_mut().unwrap().remove("finality");
        });
        assert_ne!(class_of(&tx), Class::Stop);
        let tx = builder(|t| {
            t.as_object_mut().unwrap().remove("finality");
            t["mc_block_seqno"] = Value::Null;
        });
        assert_eq!(class_of(&tx), Class::Stop);
        // finalized but not in a masterchain block: wait.
        let tx = builder(|t| t["mc_block_seqno"] = Value::Null);
        assert_eq!(class_of(&tx), Class::Stop);
        let tx = builder(|t| t["emulated"] = json!(true));
        assert_eq!(class_of(&tx), Class::Stop);
        assert_eq!(
            parse_tx(&builder(|t| t["finality"] = json!("weird")))
                .unwrap()
                .finality,
            Finality::Unknown("weird".into())
        );
    }

    #[test]
    fn row_3_not_an_incoming_internal_message() {
        // External in (no source): the wallet's own outgoing transaction.
        let ext = builder(|t| t["in_msg"]["source"] = Value::Null);
        assert_eq!(class_of(&ext), Class::Pass);
        let foreign = builder(|t| {
            t["in_msg"]["destination"] =
                json!("0:077F69B02858C4A6BC33E5F052E64E5BAD13AB95DB61442CFE0958B97959BDE4")
        });
        assert_eq!(class_of(&foreign), Class::Pass);
        let tick = builder(|t| t["description"]["type"] = json!("tick_tock"));
        assert_eq!(class_of(&tick), Class::Pass);
        let other_account = builder(|t| {
            t["account"] =
                json!("0:077F69B02858C4A6BC33E5F052E64E5BAD13AB95DB61442CFE0958B97959BDE4")
        });
        assert_eq!(class_of(&other_account), Class::Pass);
        let no_msg = builder(|t| t["in_msg"] = Value::Null);
        assert_eq!(class_of(&no_msg), Class::Pass);
        // Destination in lower case still matches.
        let lower = builder(|t| t["in_msg"]["destination"] = json!(TF.to_lowercase()));
        assert!(matches!(class_of(&lower), Class::Candidate { .. }));
    }

    #[test]
    fn rows_4_to_6() {
        let bounced = builder(|t| t["in_msg"]["bounced"] = json!(true));
        assert_eq!(class_of(&bounced), Class::Ignored(IgnoreReason::Bounced));
        for edit in [
            |t: &mut Value| t["description"]["aborted"] = json!(true),
            |t: &mut Value| t["description"]["compute_ph"]["skipped"] = json!(true),
            |t: &mut Value| t["description"]["compute_ph"]["success"] = json!(false),
            |t: &mut Value| t["description"]["action"]["success"] = json!(false),
            |t: &mut Value| {
                t["description"].as_object_mut().unwrap().remove("aborted");
            },
        ] {
            assert_eq!(
                class_of(&builder(edit)),
                Class::Ignored(IgnoreReason::Failed)
            );
        }
        // An action phase may be absent on a transfer that sends nothing.
        let no_action = builder(|t| {
            t["description"].as_object_mut().unwrap().remove("action");
        });
        assert!(matches!(class_of(&no_action), Class::Candidate { .. }));
        // Encrypted comment: another opcode.
        let encrypted = builder(|t| {
            t["in_msg"]["opcode"] = json!("0x2167da4b");
            t["in_msg"]["decoded_opcode"] = json!("encrypted_comment");
            t["in_msg"]["message_content"]["decoded"] = Value::Null;
        });
        assert_eq!(class_of(&encrypted), Class::Ignored(IgnoreReason::NotText));
        // A jetton notify that carries "B-…" text is still not TON.
        let jetton = builder(|t| {
            t["in_msg"]["opcode"] = json!("0x7362d09c");
            t["in_msg"]["message_content"]["decoded"] = json!({"@type": "jetton_notify", "type": "jetton_notify", "comment": "B-7KQ3M9XD2F"});
        });
        assert_eq!(class_of(&jetton), Class::Ignored(IgnoreReason::NotText));
        // op = 0 that toncenter did not decode as text.
        let undecoded = builder(|t| t["in_msg"]["message_content"]["decoded"] = Value::Null);
        assert_eq!(class_of(&undecoded), Class::Ignored(IgnoreReason::NotText));
        // Numeric opcode 0 works like "0x00000000".
        let numeric = builder(|t| t["in_msg"]["opcode"] = json!(0));
        assert!(matches!(class_of(&numeric), Class::Candidate { .. }));
        // No body at all: a plain transfer without a comment.
        let plain = builder(|t| {
            t["in_msg"]["opcode"] = Value::Null;
            t["in_msg"]["message_content"]["decoded"] = Value::Null;
        });
        assert_eq!(class_of(&plain), Class::Candidate { reference: None });
    }

    fn view(status: InvoiceStatus) -> InvoiceView {
        InvoiceView {
            network: Network::Mainnet,
            address_raw: TF.into(),
            status,
            created_unix: 1_000_000,
            settle_by_unix: 1_000_000 + 45 * 60,
        }
    }

    #[test]
    fn rows_7_to_12() {
        let at = |now: u64| parse_tx(&payment(9, now, 5, "B-7KQ3M9XD2F")).unwrap();
        let net = Network::Mainnet;
        // 7
        assert_eq!(
            against_invoice(&at(1_000_100), net, None),
            Outcome::Unmatched {
                matched_elsewhere: false
            }
        );
        // 8: another address, another network.
        let mut other = view(InvoiceStatus::Open);
        other.address_raw =
            "0:077F69B02858C4A6BC33E5F052E64E5BAD13AB95DB61442CFE0958B97959BDE4".into();
        assert_eq!(
            against_invoice(&at(1_000_100), net, Some(&other)),
            Outcome::Unmatched {
                matched_elsewhere: true
            }
        );
        let mut testnet = view(InvoiceStatus::Open);
        testnet.network = Network::Testnet;
        assert_eq!(
            against_invoice(&at(1_000_100), net, Some(&testnet)),
            Outcome::Unmatched {
                matched_elsewhere: true
            }
        );
        // 9: more than 60 s before the invoice; 60 s exactly is fine.
        let open = view(InvoiceStatus::Open);
        assert_eq!(
            against_invoice(&at(1_000_000 - 61), net, Some(&open)),
            Outcome::BeforeInvoice
        );
        assert_eq!(
            against_invoice(&at(1_000_000 - 60), net, Some(&open)),
            Outcome::Credited
        );
        // 10
        for s in [
            InvoiceStatus::Paid,
            InvoiceStatus::Confirmed,
            InvoiceStatus::Waived,
        ] {
            assert_eq!(
                against_invoice(&at(1_000_100), net, Some(&view(s))),
                Outcome::Overpaid
            );
        }
        // 11
        for s in [InvoiceStatus::Expired, InvoiceStatus::Closed] {
            assert_eq!(
                against_invoice(&at(1_000_100), net, Some(&view(s))),
                Outcome::Late
            );
        }
        assert_eq!(
            against_invoice(&at(1_000_000 + 45 * 60 + 1), net, Some(&open)),
            Outcome::Late
        );
        // 12: up to settle_by inclusive.
        assert_eq!(
            against_invoice(&at(1_000_000 + 45 * 60), net, Some(&open)),
            Outcome::Credited
        );
    }

    #[test]
    fn parse_never_panics_on_mutated_json() {
        let base = serde_json::to_vec(&recorded()[2]).unwrap();
        // Truncations.
        for cut in (0..base.len()).step_by(7) {
            if let Ok(v) = serde_json::from_slice::<Value>(&base[..cut]) {
                let _ = parse_tx(&v);
            }
        }
        // Byte mutations from a fixed seed (xorshift).
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..3000 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let mut bytes = base.clone();
            let pos = (x as usize) % bytes.len();
            bytes[pos] = (x >> 32) as u8;
            if let Ok(v) = serde_json::from_slice::<Value>(&bytes) {
                if let Ok(tx) = parse_tx(&v) {
                    let _ = classify(&tx, TF, 0);
                }
            }
        }
        // Wrong types everywhere.
        for v in [
            json!(null),
            json!([]),
            json!("tx"),
            json!({"lt": [], "hash": 5}),
            json!({"lt": "1", "hash": base64_hash(1), "account": 5, "now": 1}),
            json!({"lt": "1", "hash": base64_hash(1), "account": TF, "now": "x"}),
            json!({"lt": "1", "hash": base64_hash(1), "account": TF, "now": 1, "in_msg": 3}),
            json!({"lt": "1", "hash": base64_hash(1), "account": TF, "now": 1,
                   "in_msg": {"source": TF, "value": "-5"}}),
            json!({"lt": "1", "hash": base64_hash(1), "account": TF, "now": 1,
                   "in_msg": {"source": TF, "value": "5", "opcode": "0xZZ"}}),
        ] {
            assert!(parse_tx(&v).is_err(), "{v}");
        }
        // Shuffled keys parse the same.
        let mut obj: Vec<(String, Value)> = recorded()[2]
            .as_object()
            .unwrap()
            .clone()
            .into_iter()
            .collect();
        obj.reverse();
        let shuffled = Value::Object(obj.into_iter().collect());
        assert_eq!(parse_tx(&shuffled), parse_tx(&recorded()[2]));
    }

    #[test]
    fn a_parse_issue_keeps_lt_hash_and_finality_when_it_can() {
        let v = builder(|t| t["in_msg"]["value"] = json!("lots"));
        let issue = parse_tx(&v).unwrap_err();
        assert_eq!(issue.what, "in_msg.value");
        assert!(issue.lt.is_some() && issue.hash_hex.is_some() && issue.final_);
        let v = builder(|t| {
            t["in_msg"]["value"] = json!("lots");
            t["finality"] = json!("pending");
        });
        assert!(!parse_tx(&v).unwrap_err().final_);
    }

    #[test]
    fn the_testnet_pages_parse_whole() {
        for name in [
            "testnet-transactions-page1.json",
            "testnet-transactions-page2.json",
        ] {
            let page = parse_tx_page(&fixture(name)).unwrap();
            let our = "0:CCA3DC4922DEAA32160983996E8482B5F179112552943DE3A0AC0D0E449A1381";
            for v in &page.transactions {
                let tx = parse_tx(v).unwrap();
                // External-in and jetton excesses: nothing for us to credit.
                assert!(
                    !matches!(
                        classify(&tx, our, 0),
                        Class::Candidate { reference: Some(_) }
                    ),
                    "{v}"
                );
            }
        }
    }

    #[test]
    fn comments_are_cleaned_and_cut() {
        assert_eq!(clean_comment(" hi\u{7}\n there "), "hi there");
        assert_eq!(clean_comment(&"x".repeat(500)).len(), 128);
    }
}
