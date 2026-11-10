//! TON account addresses: the user-friendly form (36 bytes in base64 or
//! base64url, flags + workchain + account id + CRC16-XMODEM) and the raw
//! `workchain:hex` form toncenter uses. Only workchain 0 is accepted for a
//! payment address. See docs.ton.org "Address formats".

use base64::Engine;

use super::Network;

/// A parsed address: workchain and 32-byte account id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TonAddress {
    pub workchain: i8,
    pub account: [u8; 32],
}

/// The flags of a user-friendly address. A raw address carries none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flags {
    pub bounceable: bool,
    pub testnet_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    Empty,
    Length,
    Alphabet,
    Checksum,
    Tag,
    Hex,
    Workchain,
    TestnetOnMainnet,
}

impl std::fmt::Display for AddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Empty => "the address is empty",
            Self::Length => "a user-friendly address has 48 characters",
            Self::Alphabet => "the address is not base64 or base64url",
            Self::Checksum => "the address checksum does not match (a typo?)",
            Self::Tag => "unknown address flags",
            Self::Hex => "a raw address is <workchain>:<64 hex digits>",
            Self::Workchain => "only workchain 0 (basechain) addresses can receive payments",
            Self::TestnetOnMainnet => {
                "a testnet-only address (0Q.../kQ...) cannot be used on mainnet"
            }
        })
    }
}

const TAG_BOUNCEABLE: u8 = 0x11;
const TAG_NON_BOUNCEABLE: u8 = 0x51;
const TAG_TESTNET: u8 = 0x80;

/// CRC16-CCITT, XMODEM variant: polynomial 0x1021, initial value 0, no
/// reflection, no final xor.
pub fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= u16::from(byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

impl TonAddress {
    /// Parse either form. A user-friendly address also returns its flags.
    pub fn parse(input: &str) -> Result<(TonAddress, Option<Flags>), AddressError> {
        let s = input.trim();
        if s.is_empty() {
            return Err(AddressError::Empty);
        }
        if s.contains(':') {
            return Self::parse_raw(s).map(|a| (a, None));
        }
        Self::parse_friendly(s).map(|(a, f)| (a, Some(f)))
    }

    fn parse_raw(s: &str) -> Result<TonAddress, AddressError> {
        let (wc, hex_part) = s.split_once(':').ok_or(AddressError::Hex)?;
        let workchain: i8 = wc.parse().map_err(|_| AddressError::Hex)?;
        if hex_part.len() != 64 || !hex_part.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(AddressError::Hex);
        }
        let mut account = [0u8; 32];
        hex::decode_to_slice(hex_part, &mut account).map_err(|_| AddressError::Hex)?;
        Ok(TonAddress { workchain, account })
    }

    fn parse_friendly(s: &str) -> Result<(TonAddress, Flags), AddressError> {
        if s.len() != 48 {
            return Err(AddressError::Length);
        }
        let url = s.contains('-') || s.contains('_');
        let std = s.contains('+') || s.contains('/');
        if url && std {
            return Err(AddressError::Alphabet);
        }
        let engine = if url {
            &base64::engine::general_purpose::URL_SAFE
        } else {
            &base64::engine::general_purpose::STANDARD
        };
        let bytes = engine.decode(s).map_err(|_| AddressError::Alphabet)?;
        let bytes: [u8; 36] = bytes.try_into().map_err(|_| AddressError::Length)?;
        let crc = crc16_xmodem(&bytes[..34]);
        if crc.to_be_bytes() != [bytes[34], bytes[35]] {
            return Err(AddressError::Checksum);
        }
        let tag = bytes[0];
        let testnet_only = tag & TAG_TESTNET != 0;
        let bounceable = match tag & !TAG_TESTNET {
            TAG_BOUNCEABLE => true,
            TAG_NON_BOUNCEABLE => false,
            _ => return Err(AddressError::Tag),
        };
        let workchain = bytes[1] as i8;
        let mut account = [0u8; 32];
        account.copy_from_slice(&bytes[2..34]);
        Ok((
            TonAddress { workchain, account },
            Flags {
                bounceable,
                testnet_only,
            },
        ))
    }

    /// Parse an address meant to receive payments on `network`: workchain 0,
    /// and no testnet-only flag on mainnet. (A mainnet-form address is fine
    /// on testnet: the raw address is the same on both.)
    pub fn for_network(input: &str, network: Network) -> Result<TonAddress, AddressError> {
        let (addr, flags) = Self::parse(input)?;
        if addr.workchain != 0 {
            return Err(AddressError::Workchain);
        }
        if network == Network::Mainnet && flags.is_some_and(|f| f.testnet_only) {
            return Err(AddressError::TestnetOnMainnet);
        }
        Ok(addr)
    }

    /// `0:HEX`, upper case: the form toncenter answers with.
    pub fn raw(&self) -> String {
        format!("{}:{}", self.workchain, hex::encode_upper(self.account))
    }

    /// The user-friendly form in base64url.
    pub fn friendly(&self, bounceable: bool, testnet: bool) -> String {
        let mut bytes = [0u8; 36];
        let mut tag = if bounceable {
            TAG_BOUNCEABLE
        } else {
            TAG_NON_BOUNCEABLE
        };
        if testnet {
            tag |= TAG_TESTNET;
        }
        bytes[0] = tag;
        bytes[1] = self.workchain as u8;
        bytes[2..34].copy_from_slice(&self.account);
        let crc = crc16_xmodem(&bytes[..34]).to_be_bytes();
        bytes[34] = crc[0];
        bytes[35] = crc[1];
        base64::engine::general_purpose::URL_SAFE.encode(bytes)
    }

    /// The form shown to guests: non-bounceable (`UQ…`, testnet `0Q…`), as
    /// wallets recommend for transfers to a wallet.
    pub fn for_guest(&self, network: Network) -> String {
        self.friendly(false, network == Network::Testnet)
    }
}

/// Normalise a raw address string from an indexer answer (`0:hex`, any case)
/// to `0:HEX`; `None` when it is not a raw address.
pub fn normalize_raw(s: &str) -> Option<String> {
    TonAddress::parse_raw(s.trim()).ok().map(|a| a.raw())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TF_RAW: &str = "0:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8";
    const TF_EQ: &str = "EQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqB2N";
    const TF_UQ: &str = "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI";
    const OTHER_UQ: &str = "UQAHf2mwKFjEprwz5fBS5k5brROrldthRCz-CVi5eVm95EuN";
    const OTHER_RAW: &str = "0:077F69B02858C4A6BC33E5F052E64E5BAD13AB95DB61442CFE0958B97959BDE4";

    #[test]
    fn crc16_xmodem_check_value() {
        // The standard check value of CRC-16/XMODEM over "123456789".
        assert_eq!(crc16_xmodem(b"123456789"), 0x31C3);
    }

    #[test]
    fn the_doc_pairs_round_trip() {
        let (a, f) = TonAddress::parse(TF_EQ).unwrap();
        assert_eq!(a.raw(), TF_RAW);
        assert_eq!(
            f,
            Some(Flags {
                bounceable: true,
                testnet_only: false
            })
        );
        let (b, f) = TonAddress::parse(TF_UQ).unwrap();
        assert_eq!(a, b);
        assert!(!f.unwrap().bounceable);
        assert_eq!(a.friendly(true, false), TF_EQ);
        assert_eq!(a.friendly(false, false), TF_UQ);
        assert_eq!(TonAddress::parse(TF_RAW).unwrap().0, a);
        let (o, _) = TonAddress::parse(OTHER_UQ).unwrap();
        assert_eq!(o.raw(), OTHER_RAW);
        assert_eq!(o.for_guest(Network::Mainnet), OTHER_UQ);
    }

    #[test]
    fn both_base64_alphabets_are_accepted() {
        // OTHER_UQ has a '-' (url alphabet); the standard alphabet spells it '+'.
        let std = OTHER_UQ.replace('-', "+");
        assert_eq!(
            TonAddress::parse(&std).unwrap().0,
            TonAddress::parse(OTHER_UQ).unwrap().0
        );
        let mixed = format!("{}/", &std[..47]);
        assert!(TonAddress::parse(&mixed).is_err());
    }

    #[test]
    fn a_one_character_change_fails_the_checksum() {
        let mut s: Vec<char> = TF_UQ.chars().collect();
        s[10] = if s[10] == 'A' { 'B' } else { 'A' };
        let typo: String = s.into_iter().collect();
        assert_eq!(TonAddress::parse(&typo), Err(AddressError::Checksum));
    }

    #[test]
    fn testnet_forms_and_network_rules() {
        let (a, _) = TonAddress::parse(TF_UQ).unwrap();
        let t_nb = a.friendly(false, true);
        let t_b = a.friendly(true, true);
        assert!(t_nb.starts_with("0Q"), "{t_nb}");
        assert!(t_b.starts_with("kQ"), "{t_b}");
        assert_eq!(a.for_guest(Network::Testnet), t_nb);
        let (_, f) = TonAddress::parse(&t_nb).unwrap();
        assert!(f.unwrap().testnet_only);
        assert_eq!(
            TonAddress::for_network(&t_nb, Network::Mainnet),
            Err(AddressError::TestnetOnMainnet)
        );
        assert_eq!(TonAddress::for_network(&t_nb, Network::Testnet).unwrap(), a);
        assert_eq!(TonAddress::for_network(TF_EQ, Network::Testnet).unwrap(), a);
        assert_eq!(
            TonAddress::for_network(TF_RAW, Network::Mainnet).unwrap(),
            a
        );
    }

    #[test]
    fn masterchain_and_malformed_addresses_are_refused() {
        let mc = format!("-1:{}", &TF_RAW[2..]);
        assert_eq!(
            TonAddress::for_network(&mc, Network::Mainnet),
            Err(AddressError::Workchain)
        );
        let (a, _) = TonAddress::parse(&mc).unwrap();
        let friendly_mc = a.friendly(false, false);
        assert_eq!(
            TonAddress::for_network(&friendly_mc, Network::Mainnet),
            Err(AddressError::Workchain)
        );
        for bad in [
            "",
            "   ",
            "0:1234",
            "0:ZZDFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8",
            "x:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8",
            "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEB",
            "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI=",
            "UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xq!BI",
        ] {
            assert!(TonAddress::parse(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn unknown_tags_are_refused() {
        let (a, _) = TonAddress::parse(TF_UQ).unwrap();
        let mut bytes = [0u8; 36];
        bytes[0] = 0x22;
        bytes[2..34].copy_from_slice(&a.account);
        let crc = crc16_xmodem(&bytes[..34]).to_be_bytes();
        bytes[34..].copy_from_slice(&crc);
        let s = base64::engine::general_purpose::URL_SAFE.encode(bytes);
        assert_eq!(TonAddress::parse(&s), Err(AddressError::Tag));
    }

    #[test]
    fn raw_addresses_from_the_indexer_are_normalised() {
        assert_eq!(
            normalize_raw(&TF_RAW.to_lowercase()).as_deref(),
            Some(TF_RAW)
        );
        assert_eq!(normalize_raw("nope"), None);
    }
}
