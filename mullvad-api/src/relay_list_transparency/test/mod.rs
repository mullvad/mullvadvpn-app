use crate::relay_list_transparency::validate::validate_relay_list_envelope;
use crate::relay_list_transparency::{RelayListDigest, RelayListEnvelope, Sha256Bytes, validate};
use sha2::{Digest, Sha256};

#[test]
fn test_validate_relay_list_signature() {
    let sig = RelayListEnvelope::parse(RELAY_LIST_SIGNATURE).unwrap();
    let pubkeys = validate::parse_pubkeys(PUBKEYS, '\n').unwrap();
    let payload = validate_relay_list_envelope(&sig, &pubkeys).unwrap();
    let digest: Sha256Bytes = Sha256::digest(RELAY_LIST_CONTENT.as_bytes()).into();
    let digest_hex = RelayListDigest::new(digest);
    assert_eq!(payload.digest, digest_hex);
}

#[test]
fn test_invalid_signature_can_parse_unverified_timestamp() {
    let sig = RelayListEnvelope::parse(&format!("{RELAY_LIST_SIGNATURE}bad-signature")).unwrap();
    let pubkeys = validate::parse_pubkeys(PUBKEYS, '\n').unwrap();
    let err = validate_relay_list_envelope(&sig, &pubkeys).unwrap_err();
    let payload = err.timestamp_parser.parse_without_verification().unwrap();

    let digest: Sha256Bytes = Sha256::digest(RELAY_LIST_CONTENT.as_bytes()).into();
    let digest_hex = RelayListDigest::new(digest);
    assert_eq!(payload.digest, digest_hex);
}

#[test]
fn test_missing_signature_can_parse_unverified_timestamp() {
    // The API will return this if it couldn't generate a valid sigsum signature.
    let no_sig = "{\"digest\":\"06aa7debebf56285785ebb88bc0d2ffb43322f5b6722fedffc60636d64ecf51f\",\"timestamp\":\"2026-08-31T13:32:57+00:00\"}\n\nNO SIGNATURE";

    let sig = RelayListEnvelope::parse(no_sig).unwrap();
    let pubkeys = validate::parse_pubkeys(PUBKEYS, '\n').unwrap();
    let err = validate_relay_list_envelope(&sig, &pubkeys).unwrap_err();
    let payload = err.timestamp_parser.parse_without_verification().unwrap();

    let digest: Sha256Bytes = Sha256::digest(RELAY_LIST_CONTENT.as_bytes()).into();
    let digest_hex = RelayListDigest::new(digest);
    assert_eq!(payload.digest, digest_hex);
}

#[test]
fn test_invalid_pubkey_can_parse_unverified_timestamp() {
    let sig = RelayListEnvelope::parse(RELAY_LIST_SIGNATURE).unwrap();
    let pubkeys = validate::parse_pubkeys(PUBKEYS_INVALID, '\n').unwrap();
    let err = validate_relay_list_envelope(&sig, &pubkeys).unwrap_err();
    let payload = err.timestamp_parser.parse_without_verification().unwrap();

    let digest: Sha256Bytes = Sha256::digest(RELAY_LIST_CONTENT.as_bytes()).into();
    let digest_hex = RelayListDigest::new(digest);
    assert_eq!(payload.digest, digest_hex);
}

const PUBKEYS: &str = include_str!("valid_pubkeys");

const PUBKEYS_INVALID: &str = include_str!("invalid_pubkeys");

const RELAY_LIST_SIGNATURE: &str = include_str!("relay_list_signature");

const RELAY_LIST_CONTENT: &str = include_str!("relay_list_content");
