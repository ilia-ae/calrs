//! ilia fork: verifier tests. One negative test per check of §7.1/§7.2,
//! the W3C WebAuthn Level 3 §16 test vectors, and robustness loops (every
//! prefix, random junk, random mutations) that must never panic.

use super::softkey::{self, SoftKey, AT, BE, BS, ED, ORIGIN, RP_ID, UP, UV};
use super::verify::*;
use rand::{Rng, RngCore, SeedableRng};
use std::collections::HashMap;

const CHALLENGE: [u8; 32] = [7; 32];

fn expected() -> Expected<'static> {
    Expected {
        rp_id: RP_ID,
        origin: ORIGIN,
        challenge: &CHALLENGE,
    }
}

fn register(key: &SoftKey) -> Result<Registration, Error> {
    let r = key.register(&CHALLENGE);
    verify_registration(
        &expected(),
        &r.client_data,
        &r.auth_data,
        &r.public_key,
        r.algorithm,
        &r.raw_id,
    )
}

fn register_parts(r: &softkey::Registered) -> Result<Registration, Error> {
    verify_registration(
        &expected(),
        &r.client_data,
        &r.auth_data,
        &r.public_key,
        r.algorithm,
        &r.raw_id,
    )
}

fn stored(reg: &Registration) -> StoredKey<'_> {
    StoredKey {
        public_key: &reg.public_key,
        algorithm: reg.algorithm,
        sign_count: reg.sign_count,
        backup_eligible: reg.backup_eligible,
    }
}

fn check(key: &StoredKey<'_>, a: &softkey::Asserted) -> Result<Assertion, Error> {
    verify_assertion(&expected(), key, &a.client_data, &a.auth_data, &a.signature)
}

/// A registered key and its stored record.
fn enrolled(mut key: SoftKey) -> (SoftKey, Registration) {
    key.counter = 0;
    let reg = register(&key).unwrap();
    (key, reg)
}

/// An assertion over a hand-written clientDataJSON, signed correctly.
fn assert_with_client_data(key: &SoftKey, client_data: &str) -> softkey::Asserted {
    let auth = softkey::auth_data(RP_ID, UP, 1, &[]);
    key.assert_raw(auth, client_data.as_bytes().to_vec())
}

/// A registration whose clientDataJSON is hand-written.
fn register_with_client_data(key: &SoftKey, client_data: &str) -> Result<Registration, Error> {
    let mut r = key.register(&CHALLENGE);
    r.client_data = client_data.as_bytes().to_vec();
    register_parts(&r)
}

fn cd(kind: &str, extra: &str) -> String {
    String::from_utf8(softkey::client_data(kind, &CHALLENGE, ORIGIN, extra)).unwrap()
}

fn all_keys() -> Vec<SoftKey> {
    vec![SoftKey::es256(1), SoftKey::ed25519(2), SoftKey::rs256(3)]
}

// ---------------------------------------------------------------------------
// W3C Web Authentication Level 3, §16 test vectors
// ---------------------------------------------------------------------------

fn spec_vectors() -> HashMap<(String, String), Vec<u8>> {
    include_str!("testdata/webauthn3-vectors.txt")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let mut parts = l.split_whitespace();
            let section = parts.next().unwrap().to_string();
            let field = parts.next().unwrap().to_string();
            let value = hex::decode(parts.next().unwrap()).unwrap();
            ((section, field), value)
        })
        .collect()
}

fn field<'a>(v: &'a HashMap<(String, String), Vec<u8>>, section: &str, name: &str) -> &'a [u8] {
    v.get(&(section.to_string(), name.to_string()))
        .unwrap_or_else(|| panic!("{section} {name}"))
}

/// The `authData` byte string of an attestationObject (test-only: finds
/// the text key and reads the CBOR byte-string header after it).
fn auth_data_of(attestation_object: &[u8]) -> Vec<u8> {
    let key = b"\x68authData";
    let at = attestation_object
        .windows(key.len())
        .position(|w| w == key)
        .unwrap()
        + key.len();
    let (len, start) = match attestation_object[at] {
        0x58 => (usize::from(attestation_object[at + 1]), at + 2),
        0x59 => (
            usize::from(u16::from_be_bytes([
                attestation_object[at + 1],
                attestation_object[at + 2],
            ])),
            at + 3,
        ),
        other => panic!("authData header {other:#x}"),
    };
    attestation_object[start..start + len].to_vec()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// The SPKI `getPublicKey()` would return for a section's credential,
/// derived from its private key and checked against the COSE key the
/// authenticator data carries.
fn spec_public_key(v: &HashMap<(String, String), Vec<u8>>, section: &str) -> (i64, Vec<u8>) {
    use p256::pkcs8::EncodePublicKey;
    let auth = auth_data_of(field(v, section, "reg.attestationObject"));
    match section {
        "16.10" => {
            use rsa::traits::PublicKeyParts;
            let p = rsa::BigUint::from_bytes_be(field(v, section, "reg.private_key_p"));
            let q = rsa::BigUint::from_bytes_be(field(v, section, "reg.private_key_q"));
            let key = rsa::RsaPublicKey::new(p * q, rsa::BigUint::from(65537u32)).unwrap();
            let n = key.n().to_bytes_be();
            assert!(contains(&auth, &n), "COSE n");
            assert!(contains(&auth, &[0x21, 0x43, 0x01, 0x00, 0x01]), "COSE e");
            (
                ALG_RS256,
                key.to_public_key_der().unwrap().as_bytes().to_vec(),
            )
        }
        "16.11" => {
            let secret: [u8; 32] = field(v, section, "reg.private_key").try_into().unwrap();
            let key = ed25519_dalek::SigningKey::from_bytes(&secret).verifying_key();
            let mut x = vec![0x21, 0x58, 0x20];
            x.extend_from_slice(key.as_bytes());
            assert!(contains(&auth, &x), "COSE x");
            (
                ALG_EDDSA,
                key.to_public_key_der().unwrap().as_bytes().to_vec(),
            )
        }
        _ => {
            let secret = field(v, section, "reg.credential_private_key");
            let key = *p256::ecdsa::SigningKey::from_slice(secret)
                .unwrap()
                .verifying_key();
            let point = key.to_encoded_point(false);
            let mut x = vec![0x21, 0x58, 0x20];
            x.extend_from_slice(point.x().unwrap());
            let mut y = vec![0x22, 0x58, 0x20];
            y.extend_from_slice(point.y().unwrap());
            assert!(contains(&auth, &x) && contains(&auth, &y), "COSE x, y");
            (
                ALG_ES256,
                key.to_public_key_der().unwrap().as_bytes().to_vec(),
            )
        }
    }
}

fn spec_expected(challenge: &[u8]) -> Expected<'_> {
    Expected {
        rp_id: "example.org",
        origin: "https://example.org",
        challenge,
    }
}

/// (section, registration UV/BE/BS, assertion UV/BS), from the flag bytes
/// the spec derives (`auth_data_UV_BE_BS`, `auth_data_UV_BS`).
const SPEC_OK: [(&str, [bool; 3], [bool; 2]); 6] = [
    ("16.2", [false, true, true], [false, true]),
    ("16.3", [true, true, true], [false, false]),
    ("16.6", [false, true, false], [true, false]),
    ("16.7", [true, true, false], [true, false]),
    ("16.10", [true, true, true], [false, true]),
    ("16.11", [false, false, false], [false, false]),
];

fn spec_register(
    v: &HashMap<(String, String), Vec<u8>>,
    section: &str,
) -> Result<Registration, Error> {
    let (alg, spki) = spec_public_key(v, section);
    verify_registration(
        &spec_expected(field(v, section, "reg.challenge")),
        field(v, section, "reg.clientDataJSON"),
        &auth_data_of(field(v, section, "reg.attestationObject")),
        &spki,
        alg,
        field(v, section, "reg.credential_id"),
    )
}

fn spec_assert(
    v: &HashMap<(String, String), Vec<u8>>,
    section: &str,
    key: &StoredKey<'_>,
    expected: &Expected<'_>,
) -> Result<Assertion, Error> {
    verify_assertion(
        expected,
        key,
        field(v, section, "auth.clientDataJSON"),
        field(v, section, "auth.authenticatorData"),
        field(v, section, "auth.signature"),
    )
}

#[test]
fn spec_vectors_register_and_assert() {
    let v = spec_vectors();
    for (section, [uv, be, bs], [auth_uv, auth_bs]) in SPEC_OK {
        let reg = spec_register(&v, section).unwrap_or_else(|e| panic!("{section}: {e}"));
        assert_eq!(reg.credential_id, field(&v, section, "reg.credential_id"));
        assert_eq!(&reg.aaguid.unwrap()[..], field(&v, section, "reg.aaguid"));
        assert_eq!(reg.sign_count, 0, "{section}");
        assert_eq!(
            [reg.user_verified, reg.backup_eligible, reg.backup_state],
            [uv, be, bs],
            "{section}"
        );
        let (_, spki) = spec_public_key(&v, section);
        assert_eq!(reg.public_key, spki, "{section}: SPKI kept canonical");

        let challenge = field(&v, section, "auth.challenge");
        let out = spec_assert(&v, section, &stored(&reg), &spec_expected(challenge))
            .unwrap_or_else(|e| panic!("{section}: {e}"));
        assert_eq!(out.sign_count, 0, "{section}");
        assert_eq!([out.user_verified, out.backup_state], [auth_uv, auth_bs]);
    }
}

#[test]
fn spec_vector_long_credential_id_is_1023_bytes() {
    let v = spec_vectors();
    assert_eq!(
        field(&v, "16.6", "reg.credential_id").len(),
        MAX_CREDENTIAL_ID
    );
    assert_eq!(spec_register(&v, "16.6").unwrap().credential_id.len(), 1023);
}

#[test]
fn spec_vector_es256_signature_is_high_s_and_accepted() {
    let v = spec_vectors();
    let signature = p256::ecdsa::Signature::from_der(field(&v, "16.2", "auth.signature")).unwrap();
    assert!(signature.normalize_s().is_some(), "16.2 has a high S");
    let reg = spec_register(&v, "16.2").unwrap();
    let challenge = field(&v, "16.2", "auth.challenge");
    assert!(spec_assert(&v, "16.2", &stored(&reg), &spec_expected(challenge)).is_ok());
}

#[test]
fn spec_vectors_cross_origin_and_top_origin_rejected() {
    let v = spec_vectors();
    // 16.4 has "crossOrigin": true; 16.5 adds a topOrigin (and so is
    // refused at crossOrigin first).
    for section in ["16.4", "16.5"] {
        assert_eq!(
            spec_register(&v, section),
            Err(Error::CrossOrigin),
            "{section}"
        );
        let (alg, spki) = spec_public_key(&v, section);
        let key = StoredKey {
            public_key: &spki,
            algorithm: alg,
            sign_count: 0,
            backup_eligible: false,
        };
        let challenge = field(&v, section, "auth.challenge");
        assert_eq!(
            spec_assert(&v, section, &key, &spec_expected(challenge)),
            Err(Error::CrossOrigin),
            "{section}"
        );
    }
}

#[test]
fn spec_vector_wrong_rp_origin_challenge_or_signature_rejected() {
    let v = spec_vectors();
    let reg = spec_register(&v, "16.2").unwrap();
    let key = stored(&reg);
    let challenge = field(&v, "16.2", "auth.challenge");
    let mut e = spec_expected(challenge);
    e.rp_id = "example.com";
    assert_eq!(spec_assert(&v, "16.2", &key, &e), Err(Error::RpIdHash));
    let mut e = spec_expected(challenge);
    e.origin = "https://example.com";
    assert_eq!(spec_assert(&v, "16.2", &key, &e), Err(Error::Origin));
    let other = field(&v, "16.2", "reg.challenge");
    assert_eq!(
        spec_assert(&v, "16.2", &key, &spec_expected(other)),
        Err(Error::Challenge)
    );
    // Another section's key cannot verify 16.2's signature.
    let other = spec_register(&v, "16.7").unwrap();
    let key = StoredKey {
        backup_eligible: true,
        ..stored(&other)
    };
    assert_eq!(
        spec_assert(&v, "16.2", &key, &spec_expected(challenge)),
        Err(Error::BadSignature)
    );
}

// ---------------------------------------------------------------------------
// Round trips with the software authenticator
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_every_algorithm() {
    for key in all_keys() {
        let alg = key.algorithm();
        let (mut key, reg) = enrolled(key);
        assert_eq!(reg.algorithm, alg);
        assert_eq!(reg.public_key, key.spki());
        assert_eq!(reg.credential_id, key.credential_id);
        assert_eq!(reg.aaguid, Some(key.aaguid));
        assert!(reg.user_verified && !reg.backup_eligible && !reg.backup_state);
        let mut count = reg.sign_count;
        for _ in 0..2 {
            let a = key.assert(&CHALLENGE);
            let record = StoredKey {
                sign_count: count,
                ..stored(&reg)
            };
            let out = check(&record, &a).unwrap_or_else(|e| panic!("{alg}: {e}"));
            assert!(out.sign_count > count);
            count = out.sign_count;
        }
    }
}

#[test]
fn ed25519_and_rs256_reject_another_keys_signature() {
    for (a, b) in [
        (SoftKey::ed25519(1), SoftKey::ed25519(2)),
        (SoftKey::es256(1), SoftKey::es256(2)),
    ] {
        let (_, reg) = enrolled(a);
        let mut b = b;
        let forged = b.assert(&CHALLENGE);
        assert_eq!(check(&stored(&reg), &forged), Err(Error::BadSignature));
    }
    // RS256: a signature by an ES256 key is not even the modulus' length.
    let (_, reg) = enrolled(SoftKey::rs256(1));
    let mut other = SoftKey::es256(9);
    let forged = other.assert(&CHALLENGE);
    assert!(check(&stored(&reg), &forged).is_err());
}

// ---------------------------------------------------------------------------
// clientDataJSON
// ---------------------------------------------------------------------------

#[test]
fn wrong_type_rejected() {
    let key = SoftKey::es256(1);
    let (_, reg) = enrolled(SoftKey::es256(1));
    for kind in [
        "webauthn.create",
        "payment.get",
        "WEBAUTHN.GET",
        "",
        "webauthn.get ",
    ] {
        let a = assert_with_client_data(&key, &cd(kind, ""));
        assert_eq!(
            check(&stored(&reg), &a),
            Err(Error::ClientDataType),
            "{kind}"
        );
    }
    for kind in ["webauthn.get", "webauthn.create.", "payment.create"] {
        assert_eq!(
            register_with_client_data(&key, &cd(kind, "")),
            Err(Error::ClientDataType),
            "{kind}"
        );
    }
}

#[test]
fn challenge_mismatch_rejected() {
    let (mut key, reg) = enrolled(SoftKey::es256(1));
    let mut other = CHALLENGE;
    other[31] ^= 1;
    let a = key.assert(&other);
    assert_eq!(check(&stored(&reg), &a), Err(Error::Challenge));
    let r = key.register(&other);
    assert_eq!(register_parts(&r), Err(Error::Challenge));
    // A prefix of the challenge is not the challenge.
    let a = key.assert(&CHALLENGE[..16]);
    assert_eq!(check(&stored(&reg), &a), Err(Error::Challenge));
}

#[test]
fn challenge_encoding_rejected() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    let good = softkey::b64url(&CHALLENGE);
    let std_alphabet =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [0xfb; 32]);
    let cases = [
        format!("{good}="),                             // padding
        std_alphabet.trim_end_matches('=').to_string(), // '+' and '/'
        format!(" {good}"),
        good[..good.len() - 1].to_string(),      // truncated
        format!("{}B", &good[..good.len() - 1]), // non-canonical trailing bits
        softkey::b64url(&[7; 15]),               // too short
        softkey::b64url(&[7; 65]),               // too long
        String::new(),
    ];
    for challenge in cases {
        let json =
            format!(r#"{{"type":"webauthn.get","challenge":"{challenge}","origin":"{ORIGIN}"}}"#);
        let a = assert_with_client_data(&key, &json);
        assert_eq!(
            check(&stored(&reg), &a),
            Err(Error::Challenge),
            "{challenge}"
        );
        // A truncated value still decodes; it just is not ours.
        assert_ne!(client_challenge(json.as_bytes()), Ok(CHALLENGE.to_vec()));
    }
}

#[test]
fn origin_variants_rejected() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    for origin in [
        "http://book.example.test",
        "https://book.example.test:443",
        "https://book.example.test:8443",
        "https://book.example.test/",
        "https://evil.book.example.test",
        "https://book.example.test.evil",
        "https://BOOK.example.test",
        "HTTPS://book.example.test",
        "https://example.test",
        "null",
        "",
        "https://book.example.test\u{0}",
    ] {
        let json = format!(
            r#"{{"type":"webauthn.get","challenge":"{}","origin":{}}}"#,
            softkey::b64url(&CHALLENGE),
            serde_json::to_string(origin).unwrap()
        );
        let a = assert_with_client_data(&key, &json);
        assert_eq!(check(&stored(&reg), &a), Err(Error::Origin), "{origin}");
        assert_eq!(
            register_with_client_data(&key, &json.replace(".get", ".create")),
            Err(Error::Origin)
        );
    }
    // Same origin written with JSON escapes is the same string.
    let json = format!(
        r#"{{"type":"webauthn.get","challenge":"{}","origin":"https:\/\/book.example.test"}}"#,
        softkey::b64url(&CHALLENGE)
    );
    assert!(check(&stored(&reg), &assert_with_client_data(&key, &json)).is_ok());
}

#[test]
fn cross_origin_must_be_absent_or_false() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    let base = format!(
        r#""type":"webauthn.get","challenge":"{}","origin":"{ORIGIN}""#,
        softkey::b64url(&CHALLENGE)
    );
    for ok in ["", r#","crossOrigin":false"#] {
        let a = assert_with_client_data(&key, &format!("{{{base}{ok}}}"));
        assert!(check(&stored(&reg), &a).is_ok(), "{ok}");
    }
    for bad in ["true", "null", "\"false\"", "0", "{}", "[]"] {
        let json = format!(r#"{{{base},"crossOrigin":{bad}}}"#);
        let a = assert_with_client_data(&key, &json);
        assert_eq!(check(&stored(&reg), &a), Err(Error::CrossOrigin), "{bad}");
        assert_eq!(
            register_with_client_data(&key, &json.replace(".get", ".create")),
            Err(Error::CrossOrigin)
        );
    }
}

#[test]
fn top_origin_present_rejected() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    for value in [
        r#""https://example.com""#,
        format!(r#""{ORIGIN}""#).as_str(),
        "null",
        "\"\"",
    ] {
        let json = cd("webauthn.get", &format!(r#","topOrigin":{value}"#));
        let a = assert_with_client_data(&key, &json);
        assert_eq!(check(&stored(&reg), &a), Err(Error::TopOrigin), "{value}");
        assert_eq!(
            register_with_client_data(&key, &json.replace(".get", ".create")),
            Err(Error::TopOrigin)
        );
    }
}

#[test]
fn unknown_fields_allowed() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    let json = cd(
        "webauthn.get",
        r#","extraData":"clientDataJSON may be extended","other":{"a":[1,2]}"#,
    );
    assert!(check(&stored(&reg), &assert_with_client_data(&key, &json)).is_ok());
}

#[test]
fn duplicate_fields_rejected() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    for extra in [
        r#","origin":"https://evil.example""#,
        r#","type":"webauthn.get""#,
        r#","challenge":"AAAAAAAAAAAAAAAAAAAAAA""#,
        r#","crossOrigin":false"#,
    ] {
        let a = assert_with_client_data(&key, &cd("webauthn.get", extra));
        assert_eq!(
            check(&stored(&reg), &a),
            Err(Error::ClientDataJson),
            "{extra}"
        );
    }
}

#[test]
fn client_data_shape_rejected() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    let ch = softkey::b64url(&CHALLENGE);
    for json in [
        format!(r#"["webauthn.get","{ch}","{ORIGIN}"]"#),
        format!(r#" {{"type":"webauthn.get","challenge":"{ch}","origin":"{ORIGIN}"}}"#),
        format!(r#"{{"type":"webauthn.get","challenge":"{ch}","origin":"{ORIGIN}"}}x"#),
        format!(r#"{{"type":"webauthn.get","challenge":"{ch}","origin":"{ORIGIN}""#),
        format!(r#"{{"type":"webauthn.get","challenge":"{ch}"}}"#),
        format!(r#"{{"challenge":"{ch}","origin":"{ORIGIN}"}}"#),
        format!(r#"{{"type":"webauthn.get","origin":"{ORIGIN}"}}"#),
        format!(r#"{{"type":1,"challenge":"{ch}","origin":"{ORIGIN}"}}"#),
        format!(r#"{{"type":"webauthn.get","challenge":["{ch}"],"origin":"{ORIGIN}"}}"#),
        format!(r#"{{"type":"webauthn.get","challenge":"{ch}","origin":null}}"#),
        "\"webauthn.get\"".to_string(),
        "{}".to_string(),
        "null".to_string(),
    ] {
        let a = assert_with_client_data(&key, &json);
        assert_eq!(
            check(&stored(&reg), &a),
            Err(Error::ClientDataJson),
            "{json}"
        );
        assert_eq!(
            client_challenge(json.as_bytes()),
            Err(Error::ClientDataJson)
        );
    }
}

#[test]
fn bom_and_invalid_utf8_rejected() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    let good = softkey::client_data("webauthn.get", &CHALLENGE, ORIGIN, "");
    let mut bom = "\u{feff}".as_bytes().to_vec();
    bom.extend_from_slice(&good);
    let mut latin1 = good.clone();
    latin1.splice(1..1, [0xe9]);
    let mut in_string = good.clone();
    let at = in_string.len() - 3;
    in_string.insert(at, 0xff);
    for bytes in [bom, latin1, in_string] {
        let auth = softkey::auth_data(RP_ID, UP, 1, &[]);
        let a = key.assert_raw(auth, bytes.clone());
        assert_eq!(check(&stored(&reg), &a), Err(Error::ClientDataEncoding));
        assert_eq!(client_challenge(&bytes), Err(Error::ClientDataEncoding));
    }
}

#[test]
fn empty_expectations_fail_closed() {
    let (mut key, reg) = enrolled(SoftKey::es256(1));
    // An authenticator answering an empty RP ID, origin and challenge.
    let a = key.assert_at("", "", &[]);
    let record = stored(&reg);
    let e = Expected {
        rp_id: "",
        origin: "",
        challenge: &[],
    };
    assert!(verify_assertion(&e, &record, &a.client_data, &a.auth_data, &a.signature).is_err());
    let a = key.assert_at("", ORIGIN, &CHALLENGE);
    let e = Expected {
        rp_id: "",
        ..expected()
    };
    assert_eq!(
        verify_assertion(&e, &record, &a.client_data, &a.auth_data, &a.signature),
        Err(Error::RpIdHash)
    );
    let a = key.assert_at(RP_ID, "", &CHALLENGE);
    let e = Expected {
        origin: "",
        ..expected()
    };
    assert_eq!(
        verify_assertion(&e, &record, &a.client_data, &a.auth_data, &a.signature),
        Err(Error::Origin)
    );
    let short = [7u8; 8];
    let e = Expected {
        challenge: &short,
        ..expected()
    };
    let a = key.assert(&CHALLENGE);
    assert_eq!(
        verify_assertion(&e, &record, &a.client_data, &a.auth_data, &a.signature),
        Err(Error::Challenge)
    );
}

#[test]
fn client_challenge_reads_the_challenge() {
    let json = softkey::client_data("webauthn.get", &CHALLENGE, ORIGIN, "");
    assert_eq!(client_challenge(&json).unwrap(), CHALLENGE);
    assert_eq!(client_challenge(b""), Err(Error::ClientDataSize));
    assert_eq!(
        client_challenge(&vec![b' '; MAX_CLIENT_DATA + 1]),
        Err(Error::ClientDataSize)
    );
}

// ---------------------------------------------------------------------------
// authenticatorData
// ---------------------------------------------------------------------------

#[test]
fn rp_id_hash_mismatch_rejected() {
    let (mut key, reg) = enrolled(SoftKey::es256(1));
    for rp in [
        "example.test",
        "evil.book.example.test",
        "BOOK.example.test",
        "",
    ] {
        let a = key.assert_at(rp, ORIGIN, &CHALLENGE);
        assert_eq!(check(&stored(&reg), &a), Err(Error::RpIdHash), "{rp}");
        let r = key.register_at(rp, ORIGIN, &CHALLENGE);
        assert_eq!(register_parts(&r), Err(Error::RpIdHash), "{rp}");
    }
}

#[test]
fn user_present_required() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    let auth = softkey::auth_data(RP_ID, UV, 1, &[]);
    let a = key.assert_raw(
        auth,
        softkey::client_data("webauthn.get", &CHALLENGE, ORIGIN, ""),
    );
    assert_eq!(check(&stored(&reg), &a), Err(Error::UserNotPresent));
    let mut r = key.register(&CHALLENGE);
    r.auth_data[32] &= !UP;
    assert_eq!(register_parts(&r), Err(Error::UserNotPresent));
}

#[test]
fn uv_recorded_not_required() {
    let mut key = SoftKey::ed25519(1);
    key.flags = 0;
    let (mut key, reg) = enrolled(key);
    assert!(!reg.user_verified);
    let out = check(&stored(&reg), &key.assert(&CHALLENGE)).unwrap();
    assert!(!out.user_verified);
    key.flags = UV;
    let out = check(&stored(&reg), &key.assert(&CHALLENGE)).unwrap();
    assert!(out.user_verified);
}

#[test]
fn backup_flags_recorded() {
    // A hardware key: BE = 0. A synced passkey: BE = 1, BS = 1, counter 0.
    let mut synced = SoftKey::es256(4);
    synced.flags = UV | BE | BS;
    synced.counting = false;
    let (mut synced, reg) = enrolled(synced);
    assert!(reg.backup_eligible && reg.backup_state);
    let out = check(&stored(&reg), &synced.assert(&CHALLENGE)).unwrap();
    assert!(out.backup_state);
    assert_eq!(out.sign_count, 0);
    // BS may change while BE stays.
    synced.flags = UV | BE;
    let out = check(&stored(&reg), &synced.assert(&CHALLENGE)).unwrap();
    assert!(!out.backup_state);
}

#[test]
fn bs_without_be_rejected() {
    let mut key = SoftKey::es256(1);
    key.flags = BS;
    assert_eq!(register(&key), Err(Error::BackupStateWithoutEligibility));
    let (mut key, reg) = enrolled(SoftKey::es256(1));
    key.flags = BS;
    let a = key.assert(&CHALLENGE);
    assert_eq!(
        check(&stored(&reg), &a),
        Err(Error::BackupStateWithoutEligibility)
    );
}

#[test]
fn be_change_rejected_on_assertion() {
    let (mut key, reg) = enrolled(SoftKey::es256(1));
    key.flags = BE;
    let a = key.assert(&CHALLENGE);
    assert_eq!(
        check(&stored(&reg), &a),
        Err(Error::BackupEligibilityChanged)
    );

    let mut synced = SoftKey::es256(2);
    synced.flags = BE;
    let (mut synced, reg) = enrolled(synced);
    synced.flags = 0;
    let a = synced.assert(&CHALLENGE);
    assert_eq!(
        check(&stored(&reg), &a),
        Err(Error::BackupEligibilityChanged)
    );
}

#[test]
fn assertion_with_attested_data_rejected() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    let rest = softkey::attested(&key.aaguid, &key.credential_id, &key.cose_key());
    let auth = softkey::auth_data(RP_ID, UP | AT, 1, &rest);
    let a = key.assert_raw(
        auth,
        softkey::client_data("webauthn.get", &CHALLENGE, ORIGIN, ""),
    );
    assert_eq!(check(&stored(&reg), &a), Err(Error::AttestedDataUnexpected));
}

#[test]
fn assertion_extensions_follow_the_ed_flag() {
    let (key, reg) = enrolled(SoftKey::es256(1));
    let client = softkey::client_data("webauthn.get", &CHALLENGE, ORIGIN, "");
    // Trailing bytes without ED.
    let auth = softkey::auth_data(RP_ID, UP, 1, &[0xa1, 0x00, 0x00]);
    let a = key.assert_raw(auth, client.clone());
    assert_eq!(check(&stored(&reg), &a), Err(Error::AuthDataMalformed));
    // ED with nothing, or with something that is not a CBOR map.
    for rest in [&[][..], &[0x00], &[0x80], &[0xa0]] {
        let auth = softkey::auth_data(RP_ID, UP | ED, 1, rest);
        let a = key.assert_raw(auth, client.clone());
        assert_eq!(
            check(&stored(&reg), &a),
            Err(Error::AuthDataMalformed),
            "{rest:?}"
        );
    }
    // ED with a map (an unsolicited extension) is signed over and accepted.
    let auth = softkey::auth_data(RP_ID, UP | ED, 1, &[0xa1, 0x61, 0x78, 0xf5]);
    assert!(check(&stored(&reg), &key.assert_raw(auth, client)).is_ok());
}

// ---------------------------------------------------------------------------
// Signatures and keys
// ---------------------------------------------------------------------------

#[test]
fn signature_over_other_data_rejected() {
    for key in all_keys() {
        let (mut key, reg) = enrolled(key);
        let alg = key.algorithm();
        // Signed, then the counter raised.
        let mut a = key.assert(&CHALLENGE);
        a.auth_data[36] = a.auth_data[36].wrapping_add(1);
        assert_eq!(check(&stored(&reg), &a), Err(Error::BadSignature), "{alg}");
        // Signed, then UV added.
        let mut a = key.assert(&CHALLENGE);
        a.auth_data[32] ^= UV;
        assert_eq!(check(&stored(&reg), &a), Err(Error::BadSignature), "{alg}");
        // Signed, then the client data changed in a field nobody checks.
        let mut a = key.assert(&CHALLENGE);
        a.client_data = cd("webauthn.get", r#","extraData":"x""#).into_bytes();
        assert_eq!(check(&stored(&reg), &a), Err(Error::BadSignature), "{alg}");
        // Signed over authData alone (no client data hash).
        let mut a = key.assert(&CHALLENGE);
        a.signature = key.sign(&a.auth_data);
        assert_eq!(check(&stored(&reg), &a), Err(Error::BadSignature), "{alg}");
    }
}

#[test]
fn signature_encoding_rejected() {
    let (mut es, es_reg) = enrolled(SoftKey::es256(1));
    let (mut ed, ed_reg) = enrolled(SoftKey::ed25519(2));
    let a = es.assert(&CHALLENGE);
    for bad in [vec![], vec![0u8; MAX_SIGNATURE + 1]] {
        let b = softkey::Asserted {
            signature: bad,
            ..es.assert(&CHALLENGE)
        };
        assert_eq!(check(&stored(&es_reg), &b), Err(Error::SignatureSize));
    }
    // ES256 as raw r || s instead of DER.
    let der = p256::ecdsa::Signature::from_der(&a.signature).unwrap();
    let raw = softkey::Asserted {
        signature: der.to_bytes().to_vec(),
        ..es.assert(&CHALLENGE)
    };
    assert_eq!(check(&stored(&es_reg), &raw), Err(Error::SignatureEncoding));
    // DER with trailing bytes.
    let mut trailing = es.assert(&CHALLENGE);
    trailing.signature.push(0);
    assert_eq!(
        check(&stored(&es_reg), &trailing),
        Err(Error::SignatureEncoding)
    );
    // Ed25519 of the wrong length.
    for len in [63, 65] {
        let mut b = ed.assert(&CHALLENGE);
        b.signature.resize(len, 0);
        assert_eq!(check(&stored(&ed_reg), &b), Err(Error::SignatureEncoding));
    }
    // Ed25519 with a non-canonical S (S + L) is refused by verify_strict.
    let mut b = ed.assert(&CHALLENGE);
    add_ed25519_order(&mut b.signature[32..]);
    assert!(check(&stored(&ed_reg), &b).is_err());
}

/// Adds the Ed25519 group order L to a little-endian scalar.
fn add_ed25519_order(s: &mut [u8]) {
    const L: [u8; 32] = [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde,
        0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10,
    ];
    let mut carry = 0u16;
    for (byte, l) in s.iter_mut().zip(L) {
        let sum = u16::from(*byte) + u16::from(l) + carry;
        *byte = sum as u8;
        carry = sum >> 8;
    }
}

#[test]
fn es256_high_s_is_accepted() {
    let (mut key, reg) = enrolled(SoftKey::es256(1));
    let mut a = key.assert(&CHALLENGE);
    let low = p256::ecdsa::Signature::from_der(&a.signature).unwrap();
    let low = low.normalize_s().unwrap_or(low);
    let (r, s) = low.split_scalars();
    let high = p256::ecdsa::Signature::from_scalars(r, -s).unwrap();
    assert!(high.normalize_s().is_some(), "really high S");
    a.signature = high.to_der().as_bytes().to_vec();
    assert!(check(&stored(&reg), &a).is_ok());
}

#[test]
fn alg_spki_mismatch_rejected() {
    let keys = all_keys();
    for key in &keys {
        for alg in ALGORITHMS {
            if alg == key.algorithm() {
                continue;
            }
            let mut r = key.register(&CHALLENGE);
            r.algorithm = alg;
            assert_eq!(register_parts(&r), Err(Error::PublicKey), "{alg}");
        }
    }
    // A stored key whose algorithm does not match its SPKI.
    let (mut es, reg) = enrolled(SoftKey::es256(1));
    let a = es.assert(&CHALLENGE);
    for alg in [ALG_EDDSA, ALG_RS256] {
        let key = StoredKey {
            algorithm: alg,
            ..stored(&reg)
        };
        assert_eq!(check(&key, &a), Err(Error::PublicKey));
    }
}

#[test]
fn unsupported_alg_rejected() {
    let (mut es, reg) = enrolled(SoftKey::es256(1));
    let a = es.assert(&CHALLENGE);
    // ES384, ES512, PS256, RS1, none, a private-use value.
    for alg in [-35, -36, -37, -65535, 0, 1, i64::MIN, i64::MAX] {
        let mut r = es.register(&CHALLENGE);
        r.algorithm = alg;
        assert_eq!(
            register_parts(&r),
            Err(Error::UnsupportedAlgorithm),
            "{alg}"
        );
        let key = StoredKey {
            algorithm: alg,
            ..stored(&reg)
        };
        assert_eq!(check(&key, &a), Err(Error::UnsupportedAlgorithm), "{alg}");
    }
}

/// An RSA public key (e = 65537) whose modulus is the product of two
/// Mersenne primes 2^a - 1 and 2^b - 1, like the §16.10 key: any size,
/// with no key generation.
fn mersenne_rsa_spki(a: usize, b: usize) -> Vec<u8> {
    use p256::pkcs8::EncodePublicKey;
    let one = rsa::BigUint::from(1u32);
    let p = (rsa::BigUint::from(1u32) << a) - &one;
    let q = (rsa::BigUint::from(1u32) << b) - &one;
    rsa::RsaPublicKey::new_unchecked(p * q, rsa::BigUint::from(65537u32))
        .to_public_key_der()
        .unwrap()
        .as_bytes()
        .to_vec()
}

#[test]
fn rsa_key_sizes() {
    let es = SoftKey::es256(1);
    let with_spki = |spki: Vec<u8>| {
        let mut r = es.register(&CHALLENGE);
        r.public_key = spki;
        r.algorithm = ALG_RS256;
        register_parts(&r)
    };
    // 2048 (the soft key) and 4096 bits are accepted, so is §16.10's 3482.
    assert!(register(&SoftKey::rs256(1)).is_ok());
    let rsa4096 = softkey::hex_fixture(include_str!("testdata/rsa4096.spki.hex"));
    assert!(with_spki(rsa4096).is_ok());
    assert!(with_spki(mersenne_rsa_spki(1279, 2203)).is_ok());
    // 1128 and 1886 bits are too short.
    assert_eq!(
        with_spki(mersenne_rsa_spki(521, 607)),
        Err(Error::RsaKeySize)
    );
    assert_eq!(
        with_spki(mersenne_rsa_spki(607, 1279)),
        Err(Error::RsaKeySize)
    );
    // 4484 bits: refused by the rsa crate's own 4096-bit cap.
    assert!(with_spki(mersenne_rsa_spki(2203, 2281)).is_err());
}

#[test]
fn malformed_public_keys_rejected() {
    let es = SoftKey::es256(1);
    let with_spki = |spki: Vec<u8>, alg: i64| {
        let mut r = es.register(&CHALLENGE);
        r.public_key = spki;
        r.algorithm = alg;
        register_parts(&r)
    };
    assert_eq!(with_spki(vec![], ALG_ES256), Err(Error::PublicKeySize));
    assert_eq!(
        with_spki(vec![0x30; MAX_PUBLIC_KEY + 1], ALG_ES256),
        Err(Error::PublicKeySize)
    );
    // Trailing bytes after the DER.
    let mut spki = es.spki();
    spki.push(0);
    assert_eq!(with_spki(spki, ALG_ES256), Err(Error::PublicKey));
    // A P-256 point off the curve.
    let mut spki = es.spki();
    let last = spki.len() - 1;
    spki[last] ^= 1;
    assert_eq!(with_spki(spki, ALG_ES256), Err(Error::PublicKey));
    // The raw COSE key instead of SPKI.
    assert_eq!(with_spki(es.cose_key(), ALG_ES256), Err(Error::PublicKey));
    // Ed25519 small-order point (the identity).
    let mut weak = hex::decode("302a300506032b6570032100").unwrap();
    let mut identity = [0u8; 32];
    identity[0] = 1;
    weak.extend_from_slice(&identity);
    assert_eq!(with_spki(weak, ALG_EDDSA), Err(Error::WeakKey));
}

#[test]
fn compressed_p256_key_stored_uncompressed() {
    let es = SoftKey::es256(1);
    let softkey::Signer::Es256(signing) = &es.signer else {
        unreachable!()
    };
    // The SPKI is 30 59 | 30 13 <algorithm> | 03 42 00 04 x y: swap the
    // point for its compressed form and shorten both lengths by 32.
    let mut spki = es.spki();
    let point_at = spki.len() - 65;
    assert_eq!(spki[point_at - 3..=point_at], [0x03, 0x42, 0x00, 0x04]);
    spki.truncate(point_at);
    spki.extend_from_slice(signing.verifying_key().to_encoded_point(true).as_bytes());
    spki[1] -= 32;
    spki[point_at - 2] -= 32;
    let mut r = es.register(&CHALLENGE);
    r.public_key = spki;
    assert_eq!(register_parts(&r).unwrap().public_key, es.spki());
}

// ---------------------------------------------------------------------------
// Registration data
// ---------------------------------------------------------------------------

#[test]
fn registration_requires_at_flag() {
    let key = SoftKey::es256(1);
    let mut r = key.register(&CHALLENGE);
    r.auth_data[32] &= !AT;
    assert_eq!(register_parts(&r), Err(Error::AttestedDataMissing));
    // AT, but no attested data at all.
    r.auth_data = softkey::auth_data(RP_ID, UP | AT, 0, &[]);
    assert_eq!(register_parts(&r), Err(Error::AuthDataMalformed));
}

#[test]
fn registration_cred_id_must_equal_raw_id() {
    let key = SoftKey::es256(1);
    let mut r = key.register(&CHALLENGE);
    r.raw_id[0] ^= 1;
    assert_eq!(register_parts(&r), Err(Error::CredentialIdMismatch));
    let mut r = key.register(&CHALLENGE);
    r.raw_id.push(0);
    assert_eq!(register_parts(&r), Err(Error::CredentialIdMismatch));
}

#[test]
fn credential_id_sizes() {
    let mut key = SoftKey::es256(1);
    key.credential_id = vec![9; MAX_CREDENTIAL_ID];
    assert_eq!(register(&key).unwrap().credential_id.len(), 1023);

    let mut r = key.register(&CHALLENGE);
    r.raw_id = vec![];
    assert_eq!(register_parts(&r), Err(Error::CredentialIdSize));
    r.raw_id = vec![9; MAX_CREDENTIAL_ID + 1];
    assert_eq!(register_parts(&r), Err(Error::CredentialIdSize));

    // The length inside authData: 0 and 1024.
    for id in [vec![], vec![9; MAX_CREDENTIAL_ID + 1]] {
        let mut r = key.register(&CHALLENGE);
        let rest = softkey::attested(&key.aaguid, &id, &key.cose_key());
        r.auth_data = softkey::auth_data(RP_ID, UP | AT, 0, &rest);
        assert_eq!(
            register_parts(&r),
            Err(Error::CredentialIdSize),
            "{}",
            id.len()
        );
    }
    // A length that runs past the end.
    let mut r = key.register(&CHALLENGE);
    r.auth_data.truncate(37 + 18 + 100);
    assert_eq!(register_parts(&r), Err(Error::AuthDataMalformed));
}

#[test]
fn registration_needs_a_cose_map_after_the_id() {
    let key = SoftKey::es256(1);
    for cose in [&[][..], &[0xa0], &[0x80, 0x01], &[0x01]] {
        let mut r = key.register(&CHALLENGE);
        let rest = softkey::attested(&key.aaguid, &key.credential_id, cose);
        r.auth_data = softkey::auth_data(RP_ID, UP | AT, 0, &rest);
        assert_eq!(
            register_parts(&r),
            Err(Error::AuthDataMalformed),
            "{cose:?}"
        );
    }
}

#[test]
fn registration_extensions_follow_the_ed_flag() {
    let key = SoftKey::es256(1);
    let with = |flags: u8, tail: &[u8]| {
        let mut rest = softkey::attested(&key.aaguid, &key.credential_id, &key.cose_key());
        rest.extend_from_slice(tail);
        let mut r = key.register(&CHALLENGE);
        r.auth_data = softkey::auth_data(RP_ID, UP | AT | flags, 0, &rest);
        register_parts(&r)
    };
    // credProtect = 2, as a YubiKey reports it.
    let cred_protect = [
        0xa1, 0x6b, b'c', b'r', b'e', b'd', b'P', b'r', b'o', b't', b'e', b'c', b't', 0x02,
    ];
    assert!(with(0, &[]).is_ok());
    assert!(with(ED, &cred_protect).is_ok());
    assert_eq!(with(0, &cred_protect), Err(Error::AuthDataMalformed));
    assert_eq!(with(ED, &[]), Err(Error::AuthDataMalformed));
    assert_eq!(with(ED, &cred_protect[..5]), Err(Error::AuthDataMalformed));
    let mut twice = cred_protect.to_vec();
    twice.extend_from_slice(&cred_protect);
    assert_eq!(with(ED, &twice), Err(Error::AuthDataMalformed));
    assert_eq!(with(0, &[0x00]), Err(Error::AuthDataMalformed));
}

#[test]
fn cbor_skipper_accepts_only_well_formed_items() {
    let ok: [&[u8]; 10] = [
        &[0x00],
        &[0x37],
        &[0x18, 0x18],
        &[0x39, 0x01, 0x00],
        &[0x43, 1, 2, 3],
        &[0x63, b'a', b'b', b'c'],
        &[0x82, 0x01, 0x02],
        &[0xa1, 0x01, 0x02],
        &[0xc1, 0x00],
        &[0xf9, 0x3c, 0x00],
    ];
    for item in ok {
        assert_eq!(skip_cbor(item, 0), Some(&[][..]), "{item:x?}");
        let mut more = item.to_vec();
        more.push(0xff);
        assert_eq!(skip_cbor(&more, 0), Some(&[0xff][..]), "{item:x?}");
    }
    let bad: [&[u8]; 12] = [
        &[],
        &[0x18],
        &[0x19, 0x01],
        &[0x1c],
        &[0x5f, 0xff],
        &[0x9f, 0xff],
        &[0x43, 1, 2],
        &[0x62, 0xff, 0xfe],
        &[0x82, 0x01],
        &[0xa1, 0x01],
        &[0xf8, 0x10],
        &[0x5b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
    ];
    for item in bad {
        assert_eq!(skip_cbor(item, 0), None, "{item:x?}");
    }
    // A huge announced count over a short input ends with the input.
    assert_eq!(
        skip_cbor(
            &[0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00],
            0
        ),
        None
    );
    // Nesting is capped.
    let deep = [vec![0x81; 9], vec![0x00]].concat();
    assert_eq!(skip_cbor(&deep, 0), None);
    let shallow = [vec![0x81; 8], vec![0x00]].concat();
    assert_eq!(skip_cbor(&shallow, 0), Some(&[][..]));
}

#[test]
fn zero_aaguid_is_none() {
    let mut key = SoftKey::es256(1);
    key.aaguid = [0; 16];
    assert_eq!(register(&key).unwrap().aaguid, None);
}

#[test]
fn size_caps() {
    let key = SoftKey::es256(1);
    let mut r = key.register(&CHALLENGE);
    r.client_data = vec![];
    assert_eq!(register_parts(&r), Err(Error::ClientDataSize));
    r.client_data = vec![b'{'; MAX_CLIENT_DATA + 1];
    assert_eq!(register_parts(&r), Err(Error::ClientDataSize));
    let mut r = key.register(&CHALLENGE);
    r.auth_data.truncate(MIN_AUTH_DATA - 1);
    assert_eq!(register_parts(&r), Err(Error::AuthDataSize));
    r.auth_data = vec![0; MAX_AUTH_DATA + 1];
    assert_eq!(register_parts(&r), Err(Error::AuthDataSize));

    let (mut key, reg) = enrolled(SoftKey::es256(1));
    let mut a = key.assert(&CHALLENGE);
    a.auth_data.resize(MAX_AUTH_DATA + 1, 0);
    assert_eq!(check(&stored(&reg), &a), Err(Error::AuthDataSize));
    let a = key.assert(&CHALLENGE);
    let big = vec![0u8; MAX_PUBLIC_KEY + 1];
    let record = StoredKey {
        public_key: &big,
        ..stored(&reg)
    };
    assert_eq!(check(&record, &a), Err(Error::PublicKeySize));
}

// ---------------------------------------------------------------------------
// Counter (§6.1.1)
// ---------------------------------------------------------------------------

#[test]
fn counter_rules() {
    assert!(counter_ok(0, 0));
    assert!(counter_ok(5, 6));
    assert!(counter_ok(0, 7));
    assert!(counter_ok(u32::MAX - 1, u32::MAX));
    assert!(!counter_ok(5, 5));
    assert!(!counter_ok(5, 4));
    assert!(!counter_ok(7, 0));
    assert!(!counter_ok(u32::MAX, u32::MAX));
}

#[test]
fn counter_regression_refused_after_the_signature() {
    let (mut key, reg) = enrolled(SoftKey::es256(1));
    key.counter = 4;
    let a = key.assert(&CHALLENGE); // counter 5
    for (stored_count, ok) in [(4, true), (0, true), (5, false), (6, false)] {
        let record = StoredKey {
            sign_count: stored_count,
            ..stored(&reg)
        };
        assert_eq!(check(&record, &a).is_ok(), ok, "{stored_count}");
        if !ok {
            assert_eq!(check(&record, &a), Err(Error::CounterRegression));
        }
    }
    // A forged signature is reported as such, not as a counter problem.
    let mut forged = key.assert(&CHALLENGE);
    forged.signature = SoftKey::es256(2).sign(b"x");
    let record = StoredKey {
        sign_count: 100,
        ..stored(&reg)
    };
    assert_eq!(check(&record, &forged), Err(Error::BadSignature));
}

#[test]
fn synced_zero_counter_accepted_repeatedly() {
    let mut key = SoftKey::ed25519(1);
    key.counting = false;
    let (mut key, reg) = enrolled(key);
    for _ in 0..3 {
        assert_eq!(
            check(&stored(&reg), &key.assert(&CHALLENGE))
                .unwrap()
                .sign_count,
            0
        );
    }
    // A key that once counted cannot fall back to 0.
    let record = StoredKey {
        sign_count: 3,
        ..stored(&reg)
    };
    assert_eq!(
        check(&record, &key.assert(&CHALLENGE)),
        Err(Error::CounterRegression)
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[test]
fn decode_field_is_strict_base64url() {
    assert_eq!(decode_field("", 10).unwrap(), Vec::<u8>::new());
    assert_eq!(decode_field("AQID", 3).unwrap(), vec![1, 2, 3]);
    assert_eq!(decode_field("_-8", 2).unwrap(), vec![0xff, 0xef]);
    for bad in ["AQID=", "AQ==", "/+8", "AQ ID", "A", "AR"] {
        assert_eq!(decode_field(bad, 100), Err(Error::FieldEncoding), "{bad}");
    }
    assert_eq!(decode_field("AQID", 2), Err(Error::FieldEncoding));
    assert_eq!(
        decode_field(&"A".repeat(10_000), 64),
        Err(Error::FieldEncoding)
    );
    let max = softkey::b64url(&[1; MAX_CREDENTIAL_ID]);
    assert_eq!(decode_field(&max, MAX_CREDENTIAL_ID).unwrap().len(), 1023);
}

#[test]
fn user_handle_rules() {
    let handle = [3u8; 16];
    assert!(user_handle_matches(&[], &handle).is_ok());
    assert!(user_handle_matches(&handle, &handle).is_ok());
    assert_eq!(
        user_handle_matches(&[4; 16], &handle),
        Err(Error::UserHandleMismatch)
    );
    assert_eq!(
        user_handle_matches(&handle[..15], &handle),
        Err(Error::UserHandleMismatch)
    );
    assert_eq!(
        user_handle_matches(&[3; 65], &handle),
        Err(Error::UserHandleSize)
    );
}

#[test]
fn transports_sanitized() {
    assert_eq!(sanitize_transports(r#"["usb","nfc"]"#), vec!["usb", "nfc"]);
    assert_eq!(
        sanitize_transports(r#"["internal","hybrid","usb","usb","x","<script>"]"#),
        vec!["internal", "hybrid", "usb"]
    );
    assert_eq!(
        sanitize_transports(r#"["usb","nfc","ble","smart-card","hybrid","internal"]"#).len(),
        6
    );
    for bad in ["", "usb", "[1]", r#"{"a":"usb"}"#, "null", r#"["usb""#] {
        assert!(sanitize_transports(bad).is_empty(), "{bad}");
    }
    assert!(sanitize_transports(&format!(r#"["usb",{}"x"]"#, " ".repeat(600))).is_empty());
}

#[test]
fn error_codes_are_distinct() {
    let all = [
        Error::FieldEncoding,
        Error::ClientDataSize,
        Error::AuthDataSize,
        Error::SignatureSize,
        Error::PublicKeySize,
        Error::CredentialIdSize,
        Error::UserHandleSize,
        Error::ClientDataEncoding,
        Error::ClientDataJson,
        Error::ClientDataType,
        Error::Challenge,
        Error::Origin,
        Error::CrossOrigin,
        Error::TopOrigin,
        Error::RpIdHash,
        Error::UserNotPresent,
        Error::BackupStateWithoutEligibility,
        Error::BackupEligibilityChanged,
        Error::AttestedDataMissing,
        Error::AttestedDataUnexpected,
        Error::AuthDataMalformed,
        Error::CredentialIdMismatch,
        Error::UnsupportedAlgorithm,
        Error::PublicKey,
        Error::RsaKeySize,
        Error::WeakKey,
        Error::SignatureEncoding,
        Error::BadSignature,
        Error::CounterRegression,
        Error::UserHandleMismatch,
    ];
    let codes: std::collections::HashSet<_> = all.iter().map(|e| e.code()).collect();
    assert_eq!(codes.len(), all.len());
    assert_eq!(Error::Origin.to_string(), "webauthn: origin");
}

// ---------------------------------------------------------------------------
// Robustness: never panic
// ---------------------------------------------------------------------------

#[test]
fn truncated_inputs_never_panic() {
    for key in all_keys() {
        let alg = key.algorithm();
        let (mut key, reg) = enrolled(key);
        let r = key.register(&CHALLENGE);
        let a = key.assert(&CHALLENGE);
        let record = StoredKey {
            sign_count: 0,
            ..stored(&reg)
        };
        for n in 0..r.auth_data.len() {
            let mut t = key.register(&CHALLENGE);
            t.auth_data.truncate(n);
            assert!(register_parts(&t).is_err(), "{alg} reg authData[..{n}]");
        }
        for n in 0..r.client_data.len() {
            let mut t = key.register(&CHALLENGE);
            t.client_data.truncate(n);
            assert!(register_parts(&t).is_err(), "{alg} reg clientData[..{n}]");
        }
        for n in 0..r.public_key.len() {
            let mut t = key.register(&CHALLENGE);
            t.public_key.truncate(n);
            assert!(register_parts(&t).is_err(), "{alg} spki[..{n}]");
        }
        for n in 0..r.raw_id.len() {
            let mut t = key.register(&CHALLENGE);
            t.raw_id.truncate(n);
            assert!(register_parts(&t).is_err(), "{alg} rawId[..{n}]");
        }
        for n in 0..a.auth_data.len() {
            let t = verify_assertion(
                &expected(),
                &record,
                &a.client_data,
                &a.auth_data[..n],
                &a.signature,
            );
            assert!(t.is_err(), "{alg} authData[..{n}]");
        }
        for n in 0..a.client_data.len() {
            let t = verify_assertion(
                &expected(),
                &record,
                &a.client_data[..n],
                &a.auth_data,
                &a.signature,
            );
            assert!(t.is_err(), "{alg} clientData[..{n}]");
        }
        for n in 0..a.signature.len() {
            let t = verify_assertion(
                &expected(),
                &record,
                &a.client_data,
                &a.auth_data,
                &a.signature[..n],
            );
            assert!(t.is_err(), "{alg} signature[..{n}]");
        }
        for n in 0..reg.public_key.len() {
            let short = StoredKey {
                public_key: &reg.public_key[..n],
                ..record
            };
            assert!(check(&short, &a).is_err(), "{alg} stored key[..{n}]");
        }
        assert!(check(&record, &a).is_ok(), "{alg} full");
    }
}

fn junk(rng: &mut rand::rngs::StdRng, max: usize) -> Vec<u8> {
    let mut out = vec![0u8; rng.gen_range(0..=max)];
    rng.fill_bytes(&mut out);
    out
}

#[test]
fn junk_input_never_panics() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x5eed_a11e);
    let (_, reg) = enrolled(SoftKey::es256(1));
    let e = expected();
    for _ in 0..2000 {
        let client = junk(&mut rng, 300);
        let auth = junk(&mut rng, 300);
        let sig = junk(&mut rng, 120);
        let spki = junk(&mut rng, 120);
        let id = junk(&mut rng, 64);
        let alg = ALGORITHMS[rng.gen_range(0..3)];
        assert!(verify_registration(&e, &client, &auth, &spki, alg, &id).is_err());
        let key = StoredKey {
            public_key: if rng.gen() { &spki } else { &reg.public_key },
            algorithm: alg,
            sign_count: rng.gen(),
            backup_eligible: rng.gen(),
        };
        assert!(verify_assertion(&e, &key, &client, &auth, &sig).is_err());
        let _ = client_challenge(&client);
        let text = String::from_utf8_lossy(&client);
        let _ = decode_field(&text, 64);
        let _ = sanitize_transports(&text);
        let _ = user_handle_matches(&id, &[0; 16]);
    }
    // Structured junk: valid prefixes with random tails, random flags.
    for _ in 0..500 {
        let flags: u8 = rng.gen();
        let mut auth = softkey::auth_data(RP_ID, flags, rng.gen(), &[]);
        auth.extend_from_slice(&junk(&mut rng, 200));
        let client = softkey::client_data(
            if rng.gen() {
                "webauthn.get"
            } else {
                "webauthn.create"
            },
            &CHALLENGE,
            ORIGIN,
            "",
        );
        let id = junk(&mut rng, 40);
        let alg = ALGORITHMS[rng.gen_range(0..3)];
        let _ = verify_registration(&e, &client, &auth, &reg.public_key, alg, &id);
        let sig = junk(&mut rng, 80);
        assert!(verify_assertion(&e, &stored(&reg), &client, &auth, &sig).is_err());
    }
}

#[test]
fn mutated_assertions_never_pass() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0xbad_5eed);
    for (key, rounds) in [(SoftKey::ed25519(1), 300), (SoftKey::es256(2), 100)] {
        let (mut key, reg) = enrolled(key);
        let a = key.assert(&CHALLENGE);
        assert!(check(&stored(&reg), &a).is_ok());
        for _ in 0..rounds {
            let mut m = softkey::Asserted {
                client_data: a.client_data.clone(),
                auth_data: a.auth_data.clone(),
                signature: a.signature.clone(),
            };
            let target = match rng.gen_range(0..3) {
                0 => &mut m.client_data,
                1 => &mut m.auth_data,
                _ => &mut m.signature,
            };
            let at = rng.gen_range(0..target.len());
            let bit = 1u8 << rng.gen_range(0..8);
            target[at] ^= bit;
            assert!(check(&stored(&reg), &m).is_err(), "byte {at} bit {bit}");
        }
    }
}

#[test]
fn mutated_registrations_never_panic() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x0dd_ba11);
    for key in [SoftKey::es256(1), SoftKey::ed25519(2)] {
        let r = key.register(&CHALLENGE);
        for _ in 0..300 {
            let mut m = key.register(&CHALLENGE);
            let target = match rng.gen_range(0..4) {
                0 => &mut m.client_data,
                1 => &mut m.auth_data,
                2 => &mut m.public_key,
                _ => &mut m.raw_id,
            };
            let at = rng.gen_range(0..target.len());
            target[at] ^= 1u8 << rng.gen_range(0..8);
            // Unsigned fields (AAGUID, counter, the COSE key) may change and
            // still pass; nothing may panic, and a pass keeps what was sent.
            if let Ok(reg) = register_parts(&m) {
                assert_eq!(reg.credential_id, m.raw_id);
                assert_eq!(r.algorithm, reg.algorithm);
            }
        }
    }
}
