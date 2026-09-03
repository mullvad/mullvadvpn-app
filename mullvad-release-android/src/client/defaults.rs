use rustls_pki_types::{CertificateDer, pem::PemObject};
use std::sync::LazyLock;

/// Default URL for the `releases`-API.
///
/// Note that this is just a proxy to _some_ of the files in [METADATA_URL].
pub const RELEASES_URL: &str = "https://api.mullvad.net/app/releases/";

/// Default URL for version metadata repository.
pub const METADATA_URL: &str = "https://releases.mullvad.net/android/metadata/";

/// Accepted root certificate for the hosts above. Only this certificate is
/// trusted when fetching version metadata.
///
/// This is the Let's Encrypt root-certificate.
pub static PINNED_CERTIFICATE: LazyLock<CertificateDer<'static>> = LazyLock::new(|| {
    const CERT_BYTES: &[u8] = include_bytes!("../../../mullvad-api/le_root_cert.pem");
    CertificateDer::from_pem_slice(CERT_BYTES).expect("invalid cert")
});
