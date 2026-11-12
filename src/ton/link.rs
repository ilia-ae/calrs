//! Links: the standard `ton://transfer` deep link (docs.ton.org, "Deep
//! links") a wallet opens with the amount and comment filled in, Tonkeeper's
//! own universal link, and the explorer URL of a transaction for the owner's
//! emails.
//!
//! Why two transfer links: a Tonkeeper "multichain" wallet (the current
//! iOS app, `MainCoordinator+Deeplinks.swift`) opens a `transfer/` link that
//! names no asset as a plain send to the address: it asks for the token and
//! drops the amount. With `asset_id=ton/<net>/coin` it fills in amount and
//! comment and goes to the confirmation. MyTonWallet, on the other hand,
//! refuses a link carrying any parameter outside `amount, text, bin, jetton,
//! nft, init, stateInit, exp`, so `asset_id` goes only on the Tonkeeper link.

use super::Network;

/// `ton://transfer/<address>?amount=<nano>&text=<comment>&exp=<unix>`.
/// `exp` limits when the payer's wallet may send; wallets are not obliged
/// to honour it, so it is a hint, not a promise. Only parameters every
/// wallet accepts: one unknown parameter makes MyTonWallet and Tonkeeper
/// reject the whole link.
pub fn transfer_uri(address_friendly: &str, nano: u128, text: &str, exp_unix: i64) -> String {
    format!(
        "ton://transfer/{}?amount={nano}&text={}&exp={exp_unix}",
        urlencoding::encode(address_friendly),
        urlencoding::encode(text)
    )
}

/// Tonkeeper's universal link (github.com/tonkeeper/wallet-api,
/// "Deeplinking"), [`transfer_uri`]'s fields plus `asset_id` pinning
/// Toncoin of `network`: an `https://` URL, so it also works where a
/// browser refuses the `ton://` scheme and when scanned with the phone's
/// camera, opening the app when installed and Tonkeeper's web page
/// otherwise. Tonkeeper's in-app scanner reads it too. `asset_id` is
/// written as Tonkeeper's own tests write it, slashes unescaped.
pub fn tonkeeper_uri(
    network: Network,
    address_friendly: &str,
    nano: u128,
    text: &str,
    exp_unix: i64,
) -> String {
    format!(
        "https://app.tonkeeper.com/transfer/{}?amount={nano}&text={}&exp={exp_unix}&asset_id={}",
        urlencoding::encode(address_friendly),
        urlencoding::encode(text),
        toncoin_asset_id(network)
    )
}

/// Tonkeeper's asset id of the network's own coin (`<chain>/<net>/coin`).
fn toncoin_asset_id(network: Network) -> &'static str {
    match network {
        Network::Mainnet => "ton/mainnet/coin",
        Network::Testnet => "ton/testnet/coin",
    }
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
    fn the_plain_link_carries_only_what_every_wallet_accepts() {
        let uri = transfer_uri("UQCD", 5, "B-7KQ3M9XD2F", 9);
        let query = uri.split_once('?').unwrap().1;
        let names: Vec<&str> = query
            .split('&')
            .map(|p| p.split_once('=').unwrap().0)
            .collect();
        // MyTonWallet's accepted list is wider; `asset_id` is not in it.
        assert_eq!(names, ["amount", "text", "exp"]);
    }

    #[test]
    fn tonkeeper_link_pins_toncoin() {
        assert_eq!(
            tonkeeper_uri(
                Network::Mainnet,
                "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI",
                12_350_000_000,
                "B-7KQ3M9XD2F",
                1_794_300_000
            ),
            "https://app.tonkeeper.com/transfer/UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI?amount=12350000000&text=B-7KQ3M9XD2F&exp=1794300000&asset_id=ton/mainnet/coin"
        );
        assert_eq!(
            tonkeeper_uri(Network::Testnet, "0QCD", 5, "B-7KQ3M9XD2F", 9),
            "https://app.tonkeeper.com/transfer/0QCD?amount=5&text=B-7KQ3M9XD2F&exp=9&asset_id=ton/testnet/coin"
        );
        assert!(tonkeeper_uri(Network::Mainnet, "x", 1, "a b&c", 0).contains("text=a%20b%26c&"));
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
