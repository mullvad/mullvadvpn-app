//! rustls client config shared by the reqwest-based HTTP clients in this crate.
//!
//! reqwest is compiled without the `__rustls-ring` feature, so it will not install
//! a `CryptoProvider` on its own. Each client has to bring its own
//! pre-built [`rustls::ClientConfig`] via
//! [`reqwest::ClientBuilder::use_preconfigured_tls`], and that config is what
//! we build here using the `aws-lc-rs` provider.

use std::sync::Arc;

use rustls::{ClientConfig, RootCertStore, crypto::aws_lc_rs};
use rustls_pki_types::CertificateDer;

/// Build a rustls [`ClientConfig`] backed by `aws-lc-rs`.
///
/// If `pinned_cert` is `Some`, only that certificate is trusted. Otherwise
/// the webpki built-in roots are used.
///
/// If `tls13_only` is true, only TLS 1.3 is accepted.
pub fn build_client_config(
    pinned_cert: Option<CertificateDer<'static>>,
    tls13_only: bool,
) -> ClientConfig {
    let mut roots = RootCertStore::empty();
    match pinned_cert {
        Some(cert) => {
            roots
                .add(cert)
                .expect("pinned certificate should be a valid trust anchor");
        }
        None => roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned()),
    }

    let builder = ClientConfig::builder_with_provider(Arc::new(aws_lc_rs::default_provider()));
    let builder = if tls13_only {
        builder
            .with_protocol_versions(&[&rustls::version::TLS13])
            .expect("aws-lc-rs crypto provider should support TLS 1.3")
    } else {
        builder
            .with_safe_default_protocol_versions()
            .expect("aws-lc-rs crypto provider should support default TLS versions")
    };

    builder.with_root_certificates(roots).with_no_client_auth()
}
