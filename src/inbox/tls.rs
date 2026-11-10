//! IMAPS: implicit TLS on the configured port (993 by default). rustls with
//! the ring provider, the stack lettre and reqwest already use. No
//! plaintext, no STARTTLS. The server certificate is trusted one of two
//! ways:
//!
//! - **a chain**: the webpki roots plus an optional PEM trust anchor
//!   (`CALRS_INBOX_IMAP_CA_FILE`), the certificate of the CA that issued the
//!   server's (Let's Encrypt via Kerio, or a private CA). webpki checks the
//!   server's name against its subjectAltName only, and refuses a v1
//!   certificate or one marked CA:TRUE as the server's own, so Kerio's
//!   built-in self-signed certificate (`openssl req -x509` style: CA:TRUE,
//!   a CN and no SAN) is refused even when it is the CA file;
//! - **a pin** (`CALRS_INBOX_IMAP_CERT_SHA256`): the SHA-256 of the server
//!   certificate itself (`openssl x509 -noout -fingerprint -sha256`). Only
//!   that certificate is accepted, whatever its name, dates or form; the
//!   handshake signatures are still verified. It must be updated when the
//!   certificate is renewed.
//!
//! A refused certificate is logged with the reason rustls gives (unknown
//! issuer, CA used as the server's certificate, not valid for the name,
//! expired …), never the certificate.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::rustls::{
    self,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::WebPkiSupportedAlgorithms,
    pki_types::{CertificateDer, ServerName, UnixTime},
    ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme,
};

use super::Config;

/// How long a TCP connection and the TLS handshake may take.
pub(crate) const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// A connected stream the client can talk over.
pub(crate) trait Stream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Stream for T {}

/// Why a connection could not be opened: the transport, not the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConnectError {
    /// The CA file cannot be read or holds no certificate.
    CaFile,
    /// The host is not a valid server name.
    Name,
    /// TCP connect failed or timed out.
    Unreachable,
    /// The TLS handshake failed for another reason than the certificate.
    Handshake,
    /// The server's certificate was refused, for this reason.
    Certificate(String),
}

impl std::fmt::Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ConnectError::CaFile => "the CA file cannot be read",
            ConnectError::Name => "the host is not a valid server name",
            ConnectError::Unreachable => "the server cannot be reached",
            ConnectError::Handshake => "the TLS handshake failed",
            ConnectError::Certificate(why) => {
                return write!(f, "the server certificate was refused ({why})");
            }
        })
    }
}

pub(crate) type Connected = Box<dyn Stream>;
pub(crate) type ConnectFuture =
    Pin<Box<dyn Future<Output = Result<Connected, ConnectError>> + Send>>;
/// Opens a fresh connection to the server (the poller calls it once per
/// session).
pub(crate) type Connector = Arc<dyn Fn() -> ConnectFuture + Send + Sync>;

/// Trusts exactly one server certificate, by the SHA-256 of its DER.
#[derive(Debug)]
struct Pinned {
    sha256: [u8; 32],
    algorithms: WebPkiSupportedAlgorithms,
}

impl ServerCertVerifier for Pinned {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        use sha2::{Digest, Sha256};
        use subtle::ConstantTimeEq;
        let seen: [u8; 32] = Sha256::digest(end_entity.as_ref()).into();
        if bool::from(seen.ct_eq(&self.sha256)) {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::ApplicationVerificationFailure,
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.algorithms.supported_schemes()
    }
}

/// The TLS client configuration that trusts only the certificate whose
/// SHA-256 is `sha256`.
pub(crate) fn pinned_config(sha256: [u8; 32]) -> Result<Arc<ClientConfig>, ConnectError> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = Pinned {
        sha256,
        algorithms: provider.signature_verification_algorithms,
    };
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|_| ConnectError::Handshake)?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// Why rustls refused the server's certificate, in words for the log (no
/// part of the certificate).
fn certificate_reason(e: &rustls::CertificateError) -> String {
    use rustls::CertificateError as C;
    match e {
        C::UnknownIssuer => "unknown issuer: not signed by a trusted CA".into(),
        C::NotValidForName | C::NotValidForNameContext { .. } => {
            "not valid for the host name (webpki reads subjectAltName only)".into()
        }
        C::Expired | C::ExpiredContext { .. } => "expired".into(),
        C::NotValidYet | C::NotValidYetContext { .. } => "not valid yet".into(),
        C::BadEncoding => "badly encoded".into(),
        C::BadSignature => "bad signature".into(),
        C::InvalidPurpose | C::InvalidPurposeContext { .. } => "not for TLS servers".into(),
        C::ApplicationVerificationFailure => "does not match the pinned SHA-256".into(),
        // webpki's own errors (CaUsedAsEndEntity, UnsupportedCertVersion, …)
        // print as their names.
        C::Other(other) => format!("{other}").chars().take(80).collect(),
        other => format!("{other:?}").chars().take(80).collect(),
    }
}

/// The handshake error, with the certificate's reason when it was that.
fn handshake_error(e: &std::io::Error) -> ConnectError {
    match e
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<rustls::Error>())
    {
        Some(rustls::Error::InvalidCertificate(c)) => {
            ConnectError::Certificate(certificate_reason(c))
        }
        _ => ConnectError::Handshake,
    }
}

/// The TLS client configuration: webpki roots, plus `ca_pem`'s certificates.
pub(crate) fn client_config(ca_pem: Option<&[u8]>) -> Result<Arc<ClientConfig>, ConnectError> {
    use rustls::pki_types::pem::PemObject;
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    if let Some(pem) = ca_pem {
        let mut added = 0;
        for cert in rustls::pki_types::CertificateDer::pem_slice_iter(pem) {
            let cert = cert.map_err(|_| ConnectError::CaFile)?;
            roots.add(cert).map_err(|_| ConnectError::CaFile)?;
            added += 1;
        }
        if added == 0 {
            return Err(ConnectError::CaFile);
        }
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|_| ConnectError::Handshake)?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// Connect to `host:port` and complete the TLS handshake.
pub(crate) async fn connect_tls(
    host: &str,
    port: u16,
    tls: Arc<ClientConfig>,
) -> Result<Connected, ConnectError> {
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    let name = ServerName::try_from(bare.to_string()).map_err(|_| ConnectError::Name)?;
    let tcp = tokio::time::timeout(
        CONNECT_TIMEOUT,
        tokio::net::TcpStream::connect((bare, port)),
    )
    .await
    .map_err(|_| ConnectError::Unreachable)?
    .map_err(|_| ConnectError::Unreachable)?;
    let connector = tokio_rustls::TlsConnector::from(tls);
    let stream = tokio::time::timeout(CONNECT_TIMEOUT, connector.connect(name, tcp))
        .await
        .map_err(|_| ConnectError::Handshake)?
        .map_err(|e| handshake_error(&e))?;
    Ok(Box::new(stream))
}

/// The production connector: reads the CA file at each connection (it may
/// be renewed while calrs runs), or trusts the pinned certificate.
pub(crate) fn connector(config: Arc<Config>) -> Connector {
    Arc::new(move || {
        let config = config.clone();
        Box::pin(async move {
            if let Some(pin) = config.cert_sha256 {
                let tls = pinned_config(pin)?;
                return connect_tls(&config.host, config.port, tls).await;
            }
            let pem = match &config.ca_file {
                Some(path) => Some(
                    tokio::fs::read(path)
                        .await
                        .map_err(|_| ConnectError::CaFile)?,
                ),
                None => None,
            };
            let tls = client_config(pem.as_deref())?;
            connect_tls(&config.host, config.port, tls).await
        })
    })
}
