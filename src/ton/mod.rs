//! ilia fork: TON payments, the pure core (ilia-booking docs/ton.md).
//!
//! No web and no database here: addresses, payment codes, amounts, the rate
//! rules, invoice deadlines, the toncenter v3 client and parsers behind the
//! [`toncenter::TonIndexer`] trait, and the decision table that says what one
//! incoming transaction means. The watcher, invoices and their SQL live in
//! `crate::web::ton_pay`, which can reach the web handlers.
//!
//! The server never holds a wallet key: it only reads what arrived at the
//! owner's address. Nothing in this module signs or sends to the chain.

pub mod address;
pub mod amount;
pub mod config;
pub mod deadlines;
pub mod decide;
pub mod link;
pub mod qr;
pub mod rate;
pub mod reference;
pub mod toncenter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "mainnet" => Some(Self::Mainnet),
            "testnet" => Some(Self::Testnet),
            _ => None,
        }
    }
}
