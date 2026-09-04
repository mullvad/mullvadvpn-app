//! TLS client configuration for the Mullvad app and its surrounding tooling.
//!
//! Every component that opens a TLS connection takes its configuration from
//! here, so that the properties we insist on are decided once rather than
//! restated at each call site. Helpers are named after the destination only
//! because that is what limits which properties are available; choosing them
//! is not the caller's job.

mod cert;

use std::sync::{Arc, LazyLock};

/// Re-exported so that consumers can name the type in their own
/// signatures without depending on `rustls` themselves.
pub use rustls::ClientConfig;

/// TLS configuration for `api.mullvad.net`.
///
/// * TLS 1.3 only.
/// * Certificate pinning to ISRG Root X1, the Let's Encrypt root.
/// * Post-quantum safe. Offers only `X25519MLKEM768` as key exchange.
/// * SNI disabled.
/// * No TLS session tickets.
pub fn api() -> &'static ClientConfig {
    static CONFIG: LazyLock<ClientConfig> = LazyLock::new(|| {
        let provider = rustls::crypto::CryptoProvider {
            kx_groups: vec![rustls::crypto::aws_lc_rs::kx_group::X25519MLKEM768],
            ..rustls::crypto::aws_lc_rs::default_provider()
        };
        let mut config = client_config(
            &[&rustls::version::TLS13],
            Some(provider),
            Arc::clone(&cert::LE_ROOT_STORE),
        );
        // The server presents a certificate for the domain without being asked.
        config.enable_sni = false;
        config
    });

    &CONFIG
}

/// TLS configuration for `releases.mullvad.net`.
///
/// * TLS 1.3 only.
/// * Certificate pinning to ISRG Root X1, the Let's Encrypt root.
/// * SNI disabled.
/// * No TLS session tickets.
#[cfg(feature = "releases-cdn")]
pub fn releases_cdn() -> &'static ClientConfig {
    static CONFIG: LazyLock<ClientConfig> = LazyLock::new(|| {
        let mut config = client_config(
            &[&rustls::version::TLS13],
            None,
            Arc::clone(&cert::LE_ROOT_STORE),
        );
        // The server presents a certificate for the domain without being asked.
        config.enable_sni = false;
        config
    });

    &CONFIG
}

/// TLS configuration for the public DoH resolvers that the encrypted DNS
/// proxy addresses are looked up through.
///
/// * TLS 1.2 and 1.3.
/// * The webpki root store, the trust anchors browsers use.
/// * Whichever key exchange groups the provider offers.
/// * SNI enabled.
/// * No TLS session tickets.
pub fn doh_resolvers() -> &'static ClientConfig {
    public_roots_config()
}

/// TLS configuration for GitHub, which serves the changelog for a release.
///
/// * TLS 1.2 and 1.3.
/// * The webpki root store, the trust anchors browsers use.
/// * Whichever key exchange groups the provider offers.
/// * SNI enabled.
/// * No TLS session tickets.
pub fn github() -> &'static ClientConfig {
    public_roots_config()
}

/// TLS configuration for the host serving app installers, which the signed
/// version metadata names. It is a content delivery network rather than ours,
/// so none of the pinned configurations reach it.
///
/// * TLS 1.2 and 1.3.
/// * The webpki root store, the trust anchors browsers use.
/// * Whichever key exchange groups the provider offers.
/// * SNI enabled.
/// * No TLS session tickets.
pub fn app_installers() -> &'static ClientConfig {
    public_roots_config()
}

/// The configuration every third party is reached with. These hosts are not
/// ours, so the floor is set by what they can be relied on to support rather
/// than by what we would prefer.
fn public_roots_config() -> &'static ClientConfig {
    static CONFIG: LazyLock<ClientConfig> = LazyLock::new(|| {
        client_config(
            &[&rustls::version::TLS12, &rustls::version::TLS13],
            None,
            Arc::clone(&cert::WEBPKI_ROOT_STORE),
        )
    });

    &CONFIG
}

/// Helper for creating TLS client configs.
///
/// It fixes what does not vary:
/// * No client certificate
/// * No session tickets
///
/// And makes it easy to set the TLS protocol versions, crypto provider and
/// trust anchors. `crypto_provider` falls back to the aws-lc-rs default, which
/// offers every key exchange group the provider supports.
fn client_config(
    protocol_versions: &[&'static rustls::SupportedProtocolVersion],
    crypto_provider: Option<rustls::crypto::CryptoProvider>,
    root_store: Arc<rustls::RootCertStore>,
) -> ClientConfig {
    let provider = crypto_provider.unwrap_or_else(rustls::crypto::aws_lc_rs::default_provider);
    let mut config = ClientConfig::builder_with_provider(Arc::new(provider))
        .with_protocol_versions(protocol_versions)
        .expect("aws-lc-rs crypto provider should support the requested TLS versions")
        .with_root_certificates(root_store)
        .with_no_client_auth();
    // Disable TLS tickets to reduce ability to track clients over time
    config.resumption = rustls::client::Resumption::disabled();
    config
}

/// TLS configuration for the domain fronting connection to `api.mullvad.net`.
///
/// The CDN edge presents a certificate for the front domain, and it is
/// accepted without being checked, so this outer layer says nothing about who
/// is on the other end. It does not need to: the API connection tunnelled
/// inside it is authenticated by [`api`].
///
/// * TLS 1.2 and 1.3, matching [`doh_resolvers`], since the edge is a third
///   party.
/// * No trust anchors and no certificate verification.
/// * Whichever key exchange groups the provider offers.
/// * SNI enabled.
/// * No TLS session tickets.
pub fn api_domain_fronting() -> &'static ClientConfig {
    static CONFIG: LazyLock<ClientConfig> = LazyLock::new(|| {
        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        let mut config = ClientConfig::builder_with_provider(Arc::clone(&provider))
            .with_protocol_versions(&[&rustls::version::TLS12, &rustls::version::TLS13])
            .expect("aws-lc-rs crypto provider should support TLS 1.2 and 1.3")
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AcceptAnyServerCertificate { provider }))
            .with_no_client_auth();
        // Disable TLS tickets to reduce ability to track clients over time
        config.resumption = rustls::client::Resumption::disabled();
        config
    });

    &CONFIG
}

/// Accepts every server certificate without validating it. See
/// [`api_domain_fronting`] for when that is appropriate.
#[derive(Debug)]
struct AcceptAnyServerCertificate {
    /// The provider the connection is configured with.
    provider: Arc<rustls::crypto::CryptoProvider>,
}

impl rustls::client::danger::ServerCertVerifier for AcceptAnyServerCertificate {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls_pki_types::CertificateDer<'_>,
        _intermediates: &[rustls_pki_types::CertificateDer<'_>],
        _server_name: &rustls_pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls_pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls_pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls_pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    /// This rejects nothing, since no certificate is verified. It only fills
    /// the ClientHello's `signature_algorithms` extension, and mirroring the
    /// provider keeps that identical to what a verifying client would send.
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The API configuration keeps the hostname out of the ClientHello.
    #[test]
    fn api_config_disables_sni() {
        assert!(!api().enable_sni);
    }

    /// No ALPN is set. `hyper_rustls::with_tls_config` panics if any is, and
    /// each consumer negotiates its own protocols.
    #[test]
    fn api_config_sets_no_alpn() {
        assert!(api().alpn_protocols.is_empty());
    }

    /// The API accepts nothing but the post-quantum hybrid.
    #[test]
    fn api_config_offers_only_post_quantum_kx() {
        let config = api();
        let kx_groups = &config.crypto_provider().kx_groups;
        assert_eq!(kx_groups.len(), 1);
        assert_eq!(
            kx_groups[0].name(),
            rustls::crypto::aws_lc_rs::kx_group::X25519MLKEM768.name()
        );
    }

    /// Both pinned configurations reach hosts that need no SNI.
    #[cfg(feature = "releases-cdn")]
    #[test]
    fn releases_config_disables_sni() {
        assert!(!releases_cdn().enable_sni);
    }

    /// No helper hands out a configuration that caches session tickets. A
    /// resumed session identifies the client to the server as one it has
    /// served before, linking connections made from different tunnels.
    #[test]
    fn configs_disable_session_resumption() {
        for (name, config) in [
            ("api", api()),
            ("doh_resolvers", doh_resolvers()),
            ("api_domain_fronting", api_domain_fronting()),
        ] {
            assert_caches_no_session(name, config);
        }
    }

    /// As does the one behind the `releases-cdn` feature.
    #[cfg(feature = "releases-cdn")]
    #[test]
    fn releases_config_disables_session_resumption() {
        assert_caches_no_session("releases_cdn", releases_cdn());
    }

    fn assert_caches_no_session(name: &str, config: &ClientConfig) {
        let resumption = format!("{:?}", config.resumption);
        assert!(
            resumption.contains("NoClientSessionStorage"),
            "{name} caches sessions: {resumption}"
        );
    }

    /// The unauthenticated verifier really does accept anything, including
    /// bytes that are not a certificate at all.
    #[test]
    fn api_domain_fronting_verifier_accepts_junk() {
        use rustls::client::danger::ServerCertVerifier as _;

        let verifier = AcceptAnyServerCertificate {
            provider: Arc::new(rustls::crypto::aws_lc_rs::default_provider()),
        };
        let result = verifier.verify_server_cert(
            &rustls_pki_types::CertificateDer::from(vec![0xde, 0xad, 0xbe, 0xef]),
            &[],
            &rustls_pki_types::ServerName::try_from("example.com").unwrap(),
            &[],
            rustls_pki_types::UnixTime::now(),
        );
        assert!(result.is_ok());
    }

    /// It advertises exactly what the provider can verify, so the ClientHello
    /// matches that of a client which does check certificates.
    #[test]
    fn api_domain_fronting_verifier_mirrors_provider_schemes() {
        use rustls::client::danger::ServerCertVerifier as _;

        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        let expected = provider
            .signature_verification_algorithms
            .supported_schemes();
        let verifier = AcceptAnyServerCertificate { provider };
        assert_eq!(verifier.supported_verify_schemes(), expected);
    }
}
