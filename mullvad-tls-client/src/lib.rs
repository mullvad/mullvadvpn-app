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
}
