//! Keys that may be used for verifying data

use crate::format::key::VerifyingKey;
use std::sync::LazyLock;
use vec1::Vec1;

/// Pubkeys used to verify metadata from the Mullvad API (production)
pub static PRODUCTION_KEYS: LazyLock<Vec1<VerifyingKey>> =
    LazyLock::new(|| parse_keys(include_str!("../prod-pubkeys")));

/// Pubkeys used to verify metadata from stagemole
pub static STAGING_KEYS: LazyLock<Vec1<VerifyingKey>> =
    LazyLock::new(|| parse_keys(include_str!("../stagemole-pubkey")));

fn parse_keys(keys: &str) -> Vec1<VerifyingKey> {
    let mut v = vec![];
    for key in keys.split('\n') {
        v.push(VerifyingKey::from_hex(key.trim()).expect("invalid pubkey"));
    }
    v.try_into().expect("need at least one key")
}

#[cfg(test)]
#[test]
fn test_parse_keys() {
    // Test that actual keys are validly parsed
    let _prod = &*PRODUCTION_KEYS;
    let _staging = &*STAGING_KEYS;
}
