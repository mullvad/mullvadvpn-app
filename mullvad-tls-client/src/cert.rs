//! Trust anchors for the configurations in this crate.

use std::sync::{Arc, LazyLock};

use rustls::RootCertStore;
use rustls_pki_types::{CertificateDer, pem::PemObject};

/// The only trust anchor accepted for Mullvad endpoints: ISRG Root X1, the
/// Let's Encrypt root certificate.
const LE_ROOT_CERT: &[u8] = include_bytes!("../le_root_cert.pem");

/// A trust store containing only [`LE_ROOT_CERT`]. Parsed once, and shared by
/// every configuration that pins it.
pub static LE_ROOT_STORE: LazyLock<Arc<RootCertStore>> = LazyLock::new(|| {
    let cert = CertificateDer::from_pem_slice(LE_ROOT_CERT)
        .expect("bundled Let's Encrypt root certificate should be valid PEM");
    let mut store = RootCertStore::empty();
    store
        .add(cert)
        .expect("bundled Let's Encrypt root certificate should be a valid trust anchor");
    Arc::new(store)
});

/// The trust anchors browsers use, for hosts that are not ours.
pub static WEBPKI_ROOT_STORE: LazyLock<Arc<RootCertStore>> = LazyLock::new(|| {
    Arc::new(RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    })
});

#[cfg(test)]
mod tests {
    use super::*;

    /// The bundled root certificate parses and is accepted as a trust anchor.
    #[test]
    fn root_store_holds_exactly_one_anchor() {
        assert_eq!(LE_ROOT_STORE.len(), 1);
    }

    /// Third parties are trusted through the public roots, not our single pin.
    #[test]
    fn webpki_store_is_not_our_pin() {
        assert!(WEBPKI_ROOT_STORE.len() > 1);
        assert_ne!(WEBPKI_ROOT_STORE.len(), LE_ROOT_STORE.len());
    }
}
