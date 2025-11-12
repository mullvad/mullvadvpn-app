//! Payload from the Mullvad metadata API.

use serde::{Deserialize, Serialize};

use super::key;
use super::release::Release;

/// JSON response including signature and signed content
/// This type does not implement [serde::Deserialize] to prevent accidental deserialization without
/// signature verification.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SignedResponse {
    /// Signatures of the canonicalized JSON of `signed`
    pub signatures: Vec<ResponseSignature>,
    /// Content signed by `signature`
    pub signed: Response,
}

impl SignedResponse {
    pub fn get_releases(self) -> Vec<Release> {
        self.signed.releases
    }
}

/// Signed JSON response, not including the signature
#[derive(Default, Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Response {
    /// Version counter
    pub metadata_version: usize,
    /// When the signature expires
    pub metadata_expiry: chrono::DateTime<chrono::Utc>,
    /// Available app releases
    pub releases: Vec<Release>,
}

/// JSON response signature
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "keytype")]
#[serde(rename_all = "lowercase")]
pub enum ResponseSignature {
    Ed25519 {
        keyid: key::VerifyingKey,
        sig: key::Signature,
    },
    #[serde(untagged)]
    Other { keyid: String, sig: String },
}

/// Helper type that leaves the signed data untouched
/// Note that deserializing doesn't verify anything
#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(super) struct PartialSignedResponse {
    /// Signatures of the canonicalized JSON of `signed`
    pub signatures: Vec<ResponseSignature>,
    /// Content signed by `signature`
    pub signed: serde_json::Value,
}
