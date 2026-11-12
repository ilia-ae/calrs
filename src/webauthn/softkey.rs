//! ilia fork: a software authenticator and client for the tests: builds
//! clientDataJSON, authenticatorData, the COSE and SPKI forms of its key
//! and signatures, the way a browser and an authenticator would. ES256 and
//! Ed25519 keys come from fixed seeds; RS256 uses a 2048-bit test key kept
//! in `testdata/` (public test material, generating one per run is slow).

use super::verify::{ALG_EDDSA, ALG_ES256, ALG_RS256};
use base64::Engine;
use sha2::{Digest, Sha256};

pub(crate) const UP: u8 = 0x01;
pub(crate) const UV: u8 = 0x04;
pub(crate) const BE: u8 = 0x08;
pub(crate) const BS: u8 = 0x10;
pub(crate) const AT: u8 = 0x40;
pub(crate) const ED: u8 = 0x80;

pub(crate) const RP_ID: &str = "book.example.test";
pub(crate) const ORIGIN: &str = "https://book.example.test";

pub(crate) fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// clientDataJSON as browsers serialize it (§5.8.1.1), with `extra`
/// (`,"name":value` members) appended inside the object.
pub(crate) fn client_data(kind: &str, challenge: &[u8], origin: &str, extra: &str) -> Vec<u8> {
    format!(
        r#"{{"type":"{kind}","challenge":"{}","origin":"{origin}","crossOrigin":false{extra}}}"#,
        b64url(challenge)
    )
    .into_bytes()
}

/// `rpIdHash | flags | signCount | rest`.
pub(crate) fn auth_data(rp_id: &str, flags: u8, counter: u32, rest: &[u8]) -> Vec<u8> {
    let mut out = Sha256::digest(rp_id.as_bytes()).to_vec();
    out.push(flags);
    out.extend_from_slice(&counter.to_be_bytes());
    out.extend_from_slice(rest);
    out
}

/// Attested credential data: `aaguid | len | credentialId | coseKey`.
pub(crate) fn attested(aaguid: &[u8; 16], credential_id: &[u8], cose_key: &[u8]) -> Vec<u8> {
    let mut out = aaguid.to_vec();
    out.extend_from_slice(&(credential_id.len() as u16).to_be_bytes());
    out.extend_from_slice(credential_id);
    out.extend_from_slice(cose_key);
    out
}

/// The message an assertion signs: `authData || SHA-256(clientDataJSON)`.
pub(crate) fn signed_message(auth_data: &[u8], client_data: &[u8]) -> Vec<u8> {
    let mut message = auth_data.to_vec();
    message.extend_from_slice(&Sha256::digest(client_data));
    message
}

/// A `testdata/*.hex` file: `#` comment lines, then hex split over lines.
pub(crate) fn hex_fixture(text: &str) -> Vec<u8> {
    let hex: String = text
        .lines()
        .filter(|line| !line.starts_with('#'))
        .flat_map(|line| line.split_whitespace())
        .collect();
    hex::decode(hex).unwrap()
}

pub(crate) fn rsa2048() -> rsa::RsaPrivateKey {
    use rsa::pkcs8::DecodePrivateKey;
    let der = hex_fixture(include_str!("testdata/rsa2048.pkcs8.hex"));
    rsa::RsaPrivateKey::from_pkcs8_der(&der).unwrap()
}

pub(crate) enum Signer {
    Es256(p256::ecdsa::SigningKey),
    Ed25519(ed25519_dalek::SigningKey),
    Rs256(Box<rsa::RsaPrivateKey>),
}

/// A registration as the browser posts it.
pub(crate) struct Registered {
    pub client_data: Vec<u8>,
    pub auth_data: Vec<u8>,
    pub public_key: Vec<u8>,
    pub algorithm: i64,
    pub raw_id: Vec<u8>,
}

/// An assertion as the browser posts it.
pub(crate) struct Asserted {
    pub client_data: Vec<u8>,
    pub auth_data: Vec<u8>,
    pub signature: Vec<u8>,
}

pub(crate) struct SoftKey {
    pub signer: Signer,
    pub credential_id: Vec<u8>,
    pub aaguid: [u8; 16],
    /// UV, BE and BS bits added to UP (and AT at registration).
    pub flags: u8,
    pub counter: u32,
    /// Whether each assertion increments the counter (a synced passkey
    /// keeps 0).
    pub counting: bool,
}

impl SoftKey {
    fn with(signer: Signer, seed: u8) -> SoftKey {
        SoftKey {
            signer,
            credential_id: Sha256::digest([b'c', seed]).to_vec(),
            aaguid: [seed; 16],
            flags: UV,
            counter: 0,
            counting: true,
        }
    }

    pub(crate) fn es256(seed: u8) -> SoftKey {
        let scalar = Sha256::digest([b'p', seed]);
        let key = p256::ecdsa::SigningKey::from_slice(&scalar).unwrap();
        SoftKey::with(Signer::Es256(key), seed)
    }

    pub(crate) fn ed25519(seed: u8) -> SoftKey {
        let secret: [u8; 32] = Sha256::digest([b'e', seed]).into();
        SoftKey::with(
            Signer::Ed25519(ed25519_dalek::SigningKey::from_bytes(&secret)),
            seed,
        )
    }

    pub(crate) fn rs256(seed: u8) -> SoftKey {
        SoftKey::with(Signer::Rs256(Box::new(rsa2048())), seed)
    }

    pub(crate) fn algorithm(&self) -> i64 {
        match self.signer {
            Signer::Es256(_) => ALG_ES256,
            Signer::Ed25519(_) => ALG_EDDSA,
            Signer::Rs256(_) => ALG_RS256,
        }
    }

    /// What `getPublicKey()` returns.
    pub(crate) fn spki(&self) -> Vec<u8> {
        use p256::pkcs8::EncodePublicKey;
        let document = match &self.signer {
            Signer::Es256(key) => key.verifying_key().to_public_key_der(),
            Signer::Ed25519(key) => key.verifying_key().to_public_key_der(),
            Signer::Rs256(key) => key.to_public_key().to_public_key_der(),
        };
        document.unwrap().as_bytes().to_vec()
    }

    /// The credential public key in COSE form (RFC 9053), as it sits in
    /// the attested credential data.
    pub(crate) fn cose_key(&self) -> Vec<u8> {
        match &self.signer {
            Signer::Es256(key) => {
                let point = key.verifying_key().to_encoded_point(false);
                // {1: 2 (EC2), 3: -7, -1: 1 (P-256), -2: x, -3: y}
                let mut out = vec![0xa5, 0x01, 0x02, 0x03, 0x26, 0x20, 0x01, 0x21, 0x58, 0x20];
                out.extend_from_slice(point.x().unwrap());
                out.extend_from_slice(&[0x22, 0x58, 0x20]);
                out.extend_from_slice(point.y().unwrap());
                out
            }
            Signer::Ed25519(key) => {
                // {1: 1 (OKP), 3: -8, -1: 6 (Ed25519), -2: x}
                let mut out = vec![0xa4, 0x01, 0x01, 0x03, 0x27, 0x20, 0x06, 0x21, 0x58, 0x20];
                out.extend_from_slice(key.verifying_key().as_bytes());
                out
            }
            Signer::Rs256(key) => {
                use rsa::traits::PublicKeyParts;
                // {1: 3 (RSA), 3: -257, -1: n, -2: e}
                let n = key.n().to_bytes_be();
                let e = key.e().to_bytes_be();
                let mut out = vec![0xa4, 0x01, 0x03, 0x03, 0x39, 0x01, 0x00, 0x20, 0x59];
                out.extend_from_slice(&(n.len() as u16).to_be_bytes());
                out.extend_from_slice(&n);
                out.push(0x21);
                out.push(0x40 | e.len() as u8);
                out.extend_from_slice(&e);
                out
            }
        }
    }

    pub(crate) fn sign(&self, message: &[u8]) -> Vec<u8> {
        use p256::ecdsa::signature::{SignatureEncoding, Signer as _};
        match &self.signer {
            Signer::Es256(key) => {
                let signature: p256::ecdsa::Signature = key.sign(message);
                signature.to_der().as_bytes().to_vec()
            }
            Signer::Ed25519(key) => key.sign(message).to_bytes().to_vec(),
            Signer::Rs256(key) => {
                let signer = rsa::pkcs1v15::SigningKey::<Sha256>::new((**key).clone());
                signer.sign(message).to_vec()
            }
        }
    }

    /// `navigator.credentials.create()` on `rp_id` from `origin`.
    pub(crate) fn register_at(&self, rp_id: &str, origin: &str, challenge: &[u8]) -> Registered {
        let rest = attested(&self.aaguid, &self.credential_id, &self.cose_key());
        Registered {
            client_data: client_data("webauthn.create", challenge, origin, ""),
            auth_data: auth_data(rp_id, UP | AT | self.flags, self.counter, &rest),
            public_key: self.spki(),
            algorithm: self.algorithm(),
            raw_id: self.credential_id.clone(),
        }
    }

    pub(crate) fn register(&self, challenge: &[u8]) -> Registered {
        self.register_at(RP_ID, ORIGIN, challenge)
    }

    /// Signs over the given authenticator and client data.
    pub(crate) fn assert_raw(&self, auth_data: Vec<u8>, client_data: Vec<u8>) -> Asserted {
        let signature = self.sign(&signed_message(&auth_data, &client_data));
        Asserted {
            client_data,
            auth_data,
            signature,
        }
    }

    /// `navigator.credentials.get()` on `rp_id` from `origin`.
    pub(crate) fn assert_at(&mut self, rp_id: &str, origin: &str, challenge: &[u8]) -> Asserted {
        if self.counting {
            self.counter += 1;
        }
        let auth = auth_data(rp_id, UP | self.flags, self.counter, &[]);
        self.assert_raw(auth, client_data("webauthn.get", challenge, origin, ""))
    }

    pub(crate) fn assert(&mut self, challenge: &[u8]) -> Asserted {
        self.assert_at(RP_ID, ORIGIN, challenge)
    }
}
