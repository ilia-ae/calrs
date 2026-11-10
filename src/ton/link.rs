//! Links: the standard `ton://transfer` deep link (docs.ton.org, "Deep
//! links") a wallet opens with the amount and comment filled in, and the
//! explorer URL of a transaction for the owner's emails.

use super::Network;

/// `ton://transfer/<address>?amount=<nano>&text=<comment>&exp=<unix>`.
/// `exp` limits when the payer's wallet may send; wallets are not obliged
/// to honour it, so it is a hint, not a promise.
pub fn transfer_uri(address_friendly: &str, nano: u128, text: &str, exp_unix: i64) -> String {
    format!(
        "ton://transfer/{}?amount={nano}&text={}&exp={exp_unix}",
        urlencoding::encode(address_friendly),
        urlencoding::encode(text)
    )
}

/// Tonkeeper's universal link with the same fields as [`transfer_uri`]
/// (github.com/tonkeeper/wallet-api, "Deeplinking"): an `https://` URL,
/// so it also works where a browser refuses the `ton://` scheme, opening
/// the app when installed and Tonkeeper's web page otherwise.
pub fn tonkeeper_uri(address_friendly: &str, nano: u128, text: &str, exp_unix: i64) -> String {
    format!(
        "https://app.tonkeeper.com/transfer/{}?amount={nano}&text={}&exp={exp_unix}",
        urlencoding::encode(address_friendly),
        urlencoding::encode(text)
    )
}

/// The transaction on tonviewer, by its hex hash.
pub fn tonviewer_tx(network: Network, tx_hex: &str) -> String {
    let host = match network {
        Network::Mainnet => "tonviewer.com",
        Network::Testnet => "testnet.tonviewer.com",
    };
    format!("https://{host}/transaction/{tx_hex}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_link() {
        assert_eq!(
            transfer_uri(
                "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI",
                12_350_000_000,
                "B-7KQ3M9XD2F",
                1_794_300_000
            ),
            "ton://transfer/UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI?amount=12350000000&text=B-7KQ3M9XD2F&exp=1794300000"
        );
        assert!(transfer_uri("x", 1, "a b&c", 0).contains("text=a%20b%26c"));
    }

    #[test]
    fn tonkeeper_link() {
        assert_eq!(
            tonkeeper_uri("UQCD", 5, "B-7KQ3M9XD2F", 9),
            "https://app.tonkeeper.com/transfer/UQCD?amount=5&text=B-7KQ3M9XD2F&exp=9"
        );
    }

    #[test]
    fn explorer_links() {
        assert_eq!(
            tonviewer_tx(Network::Testnet, "ab"),
            "https://testnet.tonviewer.com/transaction/ab"
        );
        assert!(tonviewer_tx(Network::Mainnet, "ab").starts_with("https://tonviewer.com/"));
    }
}
