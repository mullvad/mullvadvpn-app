//! Built-in domain fronting access method configuration.

use std::{
    io,
    net::SocketAddr,
    ops::Deref as _,
    sync::{Arc, LazyLock},
    time::Duration,
};

use http::Uri;
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use tokio::net::TcpStream;
use tokio_rustls::rustls::KeyLogFile;
use tracing::{Level, instrument};

use crate::proxy::{ApiConnectionMode, ProxyConfig};

const FRONT_STR: &str = "https://www.phpmyadmin.net";
static FRONT: LazyLock<Uri> = LazyLock::new(|| FRONT_STR.parse().expect("Valid URI"));
const PROXY_HOST: &str = "1239602656.rsc.cdn77.org";

/// Whether we should use HTTP/2 (instead of HTTP/1.1) to talk to CDN77.
pub const USE_HTTP2: bool = true;

/// How long we may keep a domain fronting connection idle.
///
/// CDN77 has an idle timeout of 5 seconds, so we set our timeout just below that.
pub const IDLE_TIMEOUT: Duration = Duration::from_millis(4_500);

/// Resolve the built-in domain fronting configuration.
///
/// Performs DNS resolution of the front domain and returns the
/// corresponding [`ApiConnectionMode`].
#[instrument(level = Level::TRACE, ret)]
pub async fn resolve() -> Option<ApiConnectionMode> {
    // TODO: DNS lookup can be slow/flaky
    let df = domain_fronting::DomainFronting::new(FRONT.clone(), PROXY_HOST.to_string());
    let proxy_config = df
        .proxy_config()
        .await
        .inspect_err(|e| tracing::warn!("Failed to resolve domain fronting config: {e}"))
        .ok()?;

    let addr = DomainFrontingAddr {
        addr: proxy_config.addr,
        _front: FRONT.to_string(),
    };

    Some(ApiConnectionMode::Proxied(ProxyConfig::DomainFronting(
        addr,
    )))
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DomainFrontingAddr {
    /// Resolved address of the fronting domain.
    pub addr: SocketAddr,

    /// Domain that will be used to connect to a CDN.
    ///
    /// This is used as a cache-key, to guard against [`FRONT`] changing on crate updates.
    #[serde(rename = "front", deserialize_with = "deserialize_front_uri")]
    _front: String,
}

impl DomainFrontingAddr {
    pub fn config(&self) -> domain_fronting::DomainFronting {
        domain_fronting::DomainFronting::new(FRONT.clone(), PROXY_HOST.to_string())
    }
}

/// Deserialize a fronting URI and assert that it equals [`FRONT`].
pub fn deserialize_front_uri<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let front: &str = Deserialize::deserialize(deserializer)?;
    if front != FRONT_STR {
        return Err(D::Error::custom("Invalid fronting domain (Stale cache)"));
    }
    Ok(FRONT_STR.to_string())
}

/// Establish a TLS connection to the fronting CDN without certificate verification.
///
/// Certificate verification is intentionally skipped because the security of domain fronting
/// does not depend on the CDN TLS — the actual API connection is secured by the inner TLS
/// layer to `api.mullvad.net`.
#[instrument(level = Level::TRACE, skip(stream), ret)]
pub(crate) async fn cdn_tls_connect(
    stream: TcpStream,
    front_domain: &str,
    http2: bool,
) -> io::Result<tokio_rustls::client::TlsStream<TcpStream>> {
    use std::sync::LazyLock;
    use tokio_rustls::rustls::{self, client::danger, pki_types};

    #[derive(Debug)]
    struct NoCertVerification;

    impl danger::ServerCertVerifier for NoCertVerification {
        fn verify_server_cert(
            &self,
            _: &pki_types::CertificateDer<'_>,
            _: &[pki_types::CertificateDer<'_>],
            _: &pki_types::ServerName<'_>,
            _: &[u8],
            _: pki_types::UnixTime,
        ) -> Result<danger::ServerCertVerified, rustls::Error> {
            Ok(danger::ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _: &[u8],
            _: &pki_types::CertificateDer<'_>,
            _: &rustls::DigitallySignedStruct,
        ) -> Result<danger::HandshakeSignatureValid, rustls::Error> {
            Ok(danger::HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _: &[u8],
            _: &pki_types::CertificateDer<'_>,
            _: &rustls::DigitallySignedStruct,
        ) -> Result<danger::HandshakeSignatureValid, rustls::Error> {
            Ok(danger::HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            rustls::crypto::ring::default_provider()
                .signature_verification_algorithms
                .supported_schemes()
        }
    }

    static CDN_TLS_CONFIG: LazyLock<Arc<rustls::ClientConfig>> = LazyLock::new(|| {
        Arc::new(
            rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoCertVerification))
                .with_no_client_auth(),
        )
    });

    static CDN_TLS_CONFIG_HTTP2: LazyLock<Arc<rustls::ClientConfig>> = LazyLock::new(|| {
        let mut config = CDN_TLS_CONFIG.deref().clone();
        Arc::make_mut(&mut config)
            .alpn_protocols
            .push("h2".as_bytes().to_vec());
        Arc::make_mut(&mut config).key_log = Arc::new(KeyLogFile::new()) as Arc<_>;
        config
    });

    let config = if http2 {
        &CDN_TLS_CONFIG_HTTP2
    } else {
        &CDN_TLS_CONFIG
    };
    let connector = tokio_rustls::TlsConnector::from(Arc::clone(config));
    let server_name = pki_types::ServerName::try_from(front_domain.to_owned())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid front domain"))?;
    connector.connect(server_name, stream).await
}
