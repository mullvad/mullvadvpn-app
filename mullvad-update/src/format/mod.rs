//! This module includes all that is needed for the (de)serialization of Mullvad version metadata.
//! This includes ensuring authenticity and integrity of version metadata, and rejecting expired
//! metadata. There are also tools for producing new versions.
//!
//! Fundamentally, a version object is a JSON object with a `signed` key and a `signature` key.
//! `signature` contains a public key and an ed25519 signature of `signed` in canonical JSON form.
//! `signed` also contains an `expires` field, which is a timestamp indicating when the object
//! expires.
//!
//! For the deserializer to succeed in deserializing a file, it must verify that the canonicalized
//! form of `signed` is in fact signed by key/signature in `signature`. It also reads the `expires`
//! and rejects the file if it has expired.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod deserializer;
pub mod key;
#[cfg(feature = "sign")]
pub mod serializer;

/// JSON response including signature and signed content
/// This type does not implement [serde::Deserialize] to prevent accidental deserialization without
/// signature verification.
#[derive(Debug, Serialize)]
pub struct SignedResponse {
    /// Signatures of the canonicalized JSON of `signed`
    pub signatures: Vec<ResponseSignature>,
    /// Content signed by `signature`
    pub signed: Response,
}

/// Helper type that leaves the signed data untouched
/// Note that deserializing doesn't verify anything
#[derive(Deserialize, Serialize)]
struct PartialSignedResponse {
    /// Signatures of the canonicalized JSON of `signed`
    pub signatures: Vec<ResponseSignature>,
    /// Content signed by `signature`
    pub signed: serde_json::Value,
}

/// Signed JSON response, not including the signature
#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(test, derive(Clone))]
pub struct Response {
    /// Version counter
    pub metadata_version: usize,
    /// When the signature expires
    pub metadata_expiry: chrono::DateTime<chrono::Utc>,
    /// Available app releases
    pub releases: Vec<Release>,
}

/// App release
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(Clone))]
pub struct Release {
    /// Mullvad app version
    pub version: mullvad_version::Version,
    /// Changelog entries
    pub changelog: String,
    /// Installer details for different architectures
    pub installers: Vec<Installer>,
    /// Fraction of users that should receive the new version
    #[serde(default = "complete_rollout")]
    #[serde(skip_serializing_if = "is_complete_rollout")]
    pub rollout: f32,
}

/// A full rollout includes all users
fn complete_rollout() -> f32 {
    1.
}

fn is_complete_rollout(b: impl std::borrow::Borrow<f32>) -> bool {
    (b.borrow() - complete_rollout()).abs() < f32::EPSILON
}

/// App installer
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Installer {
    /// Installer architecture
    pub architecture: Architecture,
    /// Mirrors that host the artifact
    pub urls: Vec<String>,
    /// Size of the installer, in bytes
    pub size: usize,
    /// Hash of the installer, hexadecimal string
    pub sha256: String,
}

/// Installer architecture
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    /// x86-64 architecture
    X86,
    /// ARM64 architecture
    Arm64,
}

impl Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::X86 => f.write_str("x86"),
            Architecture::Arm64 => f.write_str("arm64"),
        }
    }
}

/// JSON response signature
#[derive(Debug, Deserialize, Serialize)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_rollout_serialize() {
        // rollout should not be serialized if equal to default value
        let serialized = serde_json::to_value(Release {
            version: "2024.1".parse().unwrap(),
            changelog: "".to_owned(),
            installers: vec![],
            rollout: complete_rollout(),
        })
        .unwrap();

        assert_eq!(
            serialized,
            serde_json::json!({
                "version": "2024.1",
                "changelog": "",
                "installers": [],
            })
        );

        // rollout *should* be serialized if not equal to default value
        let rollout = 0.99;
        let serialized = serde_json::to_value(Release {
            version: "2024.1".parse().unwrap(),
            changelog: "".to_owned(),
            installers: vec![],
            rollout,
        })
        .unwrap();

        assert_eq!(
            serialized,
            serde_json::json!({
                "version": "2024.1",
                "changelog": "",
                "installers": [],
                "rollout": rollout,
            })
        );
    }
}
