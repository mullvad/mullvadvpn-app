use crate::SigsumPublicKey;
use crate::relay_list_transparency::{RelayListDigest, RelayListSignature, Sha256Bytes};
use chrono::{DateTime, Utc};
use hex::FromHexError;
use serde::Deserialize;
use sigsum::policy::BuiltInPolicy;
use sigsum::{Hash, ParseAsciiError, PublicKey, SigsumSignature, VerifyError};

/// Parses a vec of pubkeys from a string input where each key is in a 64 char long hex string and
/// separated by `delimiter`. Lines starting with `#` are ignored.
pub fn parse_pubkeys(
    keys: &str,
    delimiter: char,
) -> Result<Vec<PublicKey>, SigsumPublicKeyParseError> {
    keys.split(delimiter)
        .map(|key| key.trim())
        .filter(|key| !key.is_empty() && !key.starts_with('#')) // Filter out empty lines/comments
        .map(|key| {
            let key_hex = hex::decode(key)?;
            let key_bytes: Sha256Bytes = key_hex.as_slice().try_into()?;
            Ok(PublicKey::from(key_bytes))
        })
        .collect()
}

#[derive(thiserror::Error, Debug)]
pub enum SigsumPublicKeyParseError {
    #[error("Pubkey was not a valid hex string: {0}")]
    InvalidHex(#[from] FromHexError),

    #[error("Pubkey was not 32 bytes long: {0}")]
    InvalidLength(#[from] std::array::TryFromSliceError),
}

/// The digest and timestamp data that is parsed from the `unparsed_timestamp` field in `RelayListSignature`.
#[derive(Debug, Deserialize)]
pub struct Timestamp {
    /// The hash of the relay list.
    pub digest: RelayListDigest,

    /// When the signature was signed.
    pub timestamp: DateTime<Utc>,
}

/// Validates the that the sigsum signature format is correct and that the signed data
/// (the unparsed timestamp) is valid given our (hardcoded) policy.
/// If the signature is valid, the `[Timestamp]` is parsed and returned.
/// If the signature is invalid, an error struct is returned that exposes a method to parse
/// the unverified timestamp. This is a temporary solution and should be removed once we go over
/// to failing hard on signature validation errors.
pub(crate) fn validate_relay_list_signature(
    sig: &RelayListSignature,
    trusted_pubkeys: &[SigsumPublicKey],
) -> Result<Timestamp, SignatureVerificationFailedError> {
    static POLICY: &BuiltInPolicy = &sigsum::policy::SIGSUM_TEST_2025_3;

    let sigsum_signature = SigsumSignature::from_ascii(&sig.unparsed_sigsum_signature)
        .map_err(|e| SignatureVerificationFailedError::new(sig, SigsumError::from(e)))?;

    sigsum::verify(
        &Hash::new(sig.unparsed_timestamp.as_bytes()),
        &sigsum_signature,
        trusted_pubkeys,
        POLICY,
    )
    .map_err(|e| SignatureVerificationFailedError::new(sig, SigsumError::from(e)))?;

    let timestamp = parse_timestamp(&sig.unparsed_timestamp)
        .map_err(|e| SignatureVerificationFailedError::new(sig, SigsumError::from(e)))?;

    Ok(timestamp)
}

/// Validates that the digest we get from the [`Timestamp`] matches
/// the digest of the relay list content.
pub(crate) fn validate_relay_list_content(
    timestamp: &Timestamp,
    content_digest: &RelayListDigest,
) -> Result<(), SigsumError> {
    if &timestamp.digest != content_digest {
        Err(SigsumError::ContentDigestDoesNotMatchSigsumDigest)
    } else {
        Ok(())
    }
}

fn parse_timestamp(unparsed_timestamp: &str) -> Result<Timestamp, serde_json::Error> {
    serde_json::from_str(unparsed_timestamp)
}

/// Exposes a method to parse a [`Timestamp`] that failed signature validation.
/// Should be removed once we go over to failing hard on signature verification errors.
#[derive(Debug, Clone)]
pub struct NoVerificationTimestampParser {
    unparsed_timestamp: String,
}

/// Parses a timestamp even though the sigsum signature validation has failed.
impl NoVerificationTimestampParser {
    fn new(unparsed_timestamp: String) -> Self {
        Self { unparsed_timestamp }
    }

    /// This function will parse the timestamp even if the sigsum signature verification step has
    /// failed. It should only be used as long as we have the open fail policy in place.
    /// This function should be removed once we transition to rejecting relay list updates that
    /// fail sigsum verification.
    pub fn parse_without_verification(&self) -> Result<Timestamp, serde_json::Error> {
        parse_timestamp(&self.unparsed_timestamp)
    }
}

/// An error representing a signature verification error due to an invalid or policy-breaking
/// signature.
#[derive(Debug, thiserror::Error)]
#[error("Signature verification failed")]
pub(crate) struct SignatureVerificationFailedError {
    pub source: SigsumError,
    pub timestamp_parser: NoVerificationTimestampParser,
}

impl SignatureVerificationFailedError {
    fn new(relay_list_signature: &RelayListSignature, source: SigsumError) -> Self {
        Self {
            source,
            timestamp_parser: NoVerificationTimestampParser::new(
                relay_list_signature.unparsed_timestamp.clone(),
            ),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SigsumError {
    #[error("Signature parsing failed")]
    ParseAscii(#[from] ParseAsciiError),

    #[error("Signature verification failed")]
    Verify(#[from] VerifyError),

    #[error("Invalid timestamp")]
    InvalidTimestamp(#[from] serde_json::Error),

    #[error("Content digest does not match sigsum digest")]
    ContentDigestDoesNotMatchSigsumDigest,
}

#[cfg(test)]
mod test {
    use super::*;
    use hex::ToHex;

    #[test]
    fn test_parsing_pubkey_from_file() {
        let trusted =
            include_str!("../../mullvad-api-constants/src/trusted-sigsum-signing-pubkeys");
        let keys = parse_pubkeys(trusted, '\n').unwrap();
        assert!(!keys.is_empty());
    }

    #[test]
    fn test_parsing_pubkey_can_contain_empty_lines_and_comments() {
        let input = "";
        let keys = parse_pubkeys(input, '\n').unwrap();
        assert!(keys.is_empty());

        let input =
            "#this is a comment\n35809994d285fe3dd50d49c384db49519412008c545cb6588c138a86ae4c3284";
        let keys = parse_pubkeys(input, '\n').unwrap();
        assert_eq!(1, keys.len());
    }

    #[test]
    fn test_parsing_pubkey_with_comma_delimiter() {
        let input = "35809994d285fe3dd50d49c384db49519412008c545cb6588c138a86ae4c3284:9e05c843f17ed7225df58fdfd6ddcd65251aa6db4ad8ea63bd2bf0326e30577d";
        let keys = parse_pubkeys(input, ':').unwrap();
        let key1: String = keys[0].encode_hex();
        let key2: String = keys[1].encode_hex();

        assert_eq!(
            key1,
            "35809994d285fe3dd50d49c384db49519412008c545cb6588c138a86ae4c3284"
        );
        assert_eq!(
            key2,
            "9e05c843f17ed7225df58fdfd6ddcd65251aa6db4ad8ea63bd2bf0326e30577d"
        );
    }
}
