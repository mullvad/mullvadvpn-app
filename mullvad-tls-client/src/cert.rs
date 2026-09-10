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

#[cfg(test)]
mod tests {
    use super::*;

    /// The bundled root certificate parses and is accepted as a trust anchor.
    #[test]
    fn root_store_holds_exactly_one_anchor() {
        assert_eq!(LE_ROOT_STORE.len(), 1);
    }
}
