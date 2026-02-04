mod test;

use mullvad_api::{RelayListSignature, Sha256Bytes};
use serde::Deserialize;
use sigsum::{Hash, ParseAsciiError, Policy, PublicKey, SigsumSignature, VerifyError};
use std::sync::LazyLock;

/// Pubkeys used to verify the sigsum signature of the relay list (stagemole)
pub static TRUSTED_SIGNING_PUBKEYS: LazyLock<Vec<PublicKey>> =
    LazyLock::new(|| parse_keys(include_str!("trusted-sigsum-signing-pubkeys")));

fn parse_keys(keys: &str) -> Vec<PublicKey> {
    keys.split('\n')
        .map(|key| {
            let key_hex = hex::decode(key).expect("invalid hex");
            let key_bytes: [u8; 32] = key_hex.as_slice().try_into().expect("invalid pubkey");
            PublicKey::from(key_bytes)
        })
        .collect()
}

const POLICY: &str = "sigsum-test-2025-3";

/// The digest and timestamp data that is parsed from the `unparsed_timestamp` field in `RelayListSignature`.
#[derive(Debug, Deserialize)]
pub struct Timestamp {
    /// The hash of the relay list.
    pub digest: String,

    /// When the signature was signed.
    pub timestamp: String,
}

pub(crate) fn parse_signature(
    sig: &RelayListSignature,
) -> Result<Timestamp, SignatureVerificationFailedError> {
    let policy = Policy::builtin(POLICY).unwrap();

    let sigsum_signature = SigsumSignature::from_ascii(&sig.sigsum_signature)
        .map_err(|e| SignatureVerificationFailedError::new(sig, SigsumError::from(e)))?;

    sigsum::verify(
        &Hash::new(sig.unparsed_timestamp.as_bytes()),
        sigsum_signature,
        TRUSTED_SIGNING_PUBKEYS.clone(),
        &policy,
    )
    .map_err(|e| SignatureVerificationFailedError::new(sig, SigsumError::from(e)))?;

    let timestamp = parse_timestamp(&sig.unparsed_timestamp)
        .map_err(|e| SignatureVerificationFailedError::new(sig, SigsumError::from(e)))?;

    Ok(timestamp)
}

pub(crate) fn validate_data(
    timestamp: &Timestamp,
    content_hash: &Sha256Bytes,
) -> Result<(), SigsumError> {
    let digest_bytes = hex::decode(&timestamp.digest)
        .map_err(|_| SigsumError::ContentDigestDoesNotMatchSigsumDigest)?;

    if content_hash != digest_bytes.as_slice() {
        Err(SigsumError::ContentDigestDoesNotMatchSigsumDigest)
    } else {
        Ok(())
    }
}

fn parse_timestamp(unparsed_timestamp: &str) -> Result<Timestamp, serde_json::Error> {
    serde_json::from_str(unparsed_timestamp)
}

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
    pub fn parse_timestamp_without_verification(&self) -> Result<Timestamp, serde_json::Error> {
        parse_timestamp(&self.unparsed_timestamp)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Signature verification failed")]
pub(crate) struct SignatureVerificationFailedError {
    pub source: SigsumError,
    pub parser: NoVerificationTimestampParser,
}

impl SignatureVerificationFailedError {
    fn new(relay_list_signature: &RelayListSignature, source: SigsumError) -> Self {
        Self {
            source,
            parser: NoVerificationTimestampParser::new(
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
