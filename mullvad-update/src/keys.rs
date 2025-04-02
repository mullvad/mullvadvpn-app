//! Keys that may be used for verifying data

use crate::format::key::VerifyingKey;
use std::sync::LazyLock;
use vec1::Vec1;

/// Default TLS certificate to pin to
pub static PINNED_CERTIFICATE: LazyLock<reqwest::Certificate> = LazyLock::new(|| {
    const CERT_BYTES: &[u8] = include_bytes!("../../mullvad-api/le_root_cert.pem");
    reqwest::Certificate::from_pem(CERT_BYTES).expect("invalid cert")
});

/// Pubkeys used to verify metadata from the Mullvad API (production)
pub static TRUSTED_METADATA_SIGNING_PUBKEYS: LazyLock<Vec1<VerifyingKey>> =
    LazyLock::new(|| parse_keys(include_str!("../trusted-metadata-signing-pubkeys")));

fn parse_keys(keys: &str) -> Vec1<VerifyingKey> {
    let mut v = vec![];
    for key in keys.split('\n') {
        let key = key.trim();
        if key.starts_with('#') || key.is_empty() {
            continue;
        }
        v.push(VerifyingKey::from_hex(key).expect("invalid pubkey"));
    }
    v.try_into().expect("need at least one key")
}

#[cfg(test)]
#[test]
fn test_parse_keys() {
    let key1 = "AB4EF63FFDCC6BD5A19C30CD23B9DE03099407A04463418F17AE338B98AA09D4".to_lowercase();
    let key2 = "BB4EF63FFDCC6BD5A19C30CD23B9DE03099407A04463418F17AE338B98AA09D4".to_lowercase();
    let keys = parse_keys(&format!(
        r#"
# test
{key1}
# test 2
{key2}
"#
    ));
    assert_eq!(format!("{}", keys[0]), key1);
    assert_eq!(format!("{}", keys[1]), key2);

    // Test that actual keys are validly parsed
    let _prod = &*TRUSTED_METADATA_SIGNING_PUBKEYS;
}
