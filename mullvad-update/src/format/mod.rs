use serde::{Deserialize, Serialize};

pub mod deserializer;
pub mod key;
#[cfg(feature = "sign")]
pub mod serializer;

/// JSON response including signature and signed content
/// This type does not implement [serde::Deserialize] to prevent accidental deserialization without
/// signature verification.
#[derive(Serialize)]
pub struct SignedResponse {
    /// Signature of the canonicalized JSON of `signed`
    pub signature: ResponseSignature,
    /// Content signed by `signature`
    pub signed: Response,
}

/// Helper class that leaves the signed data untouched
/// Note that deserializing doesn't verify anything
#[derive(Deserialize, Serialize)]
struct PartialSignedResponse {
    /// Signature of the canonicalized JSON of `signed`
    pub signature: ResponseSignature,
    /// Content signed by `signature`
    pub signed: serde_json::Value,
}

/// Signed JSON response, not including the signature
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    /// When the signature expires
    pub expires: chrono::DateTime<chrono::Utc>,
    /// Stable version response
    pub stable: VersionResponse,
    /// Beta version response
    pub beta: Option<VersionResponse>,
}

#[derive(Deserialize, Serialize)]
pub struct VersionResponse {
    /// The current version in this channel
    pub current: SpecificVersionResponse,
    /// The version being rolled out in this channel
    pub next: Option<NextSpecificVersionResponse>,
}

#[derive(Deserialize, Serialize)]
pub struct NextSpecificVersionResponse {
    /// The percentage of users that should receive the new version.
    pub rollout: f32,
    #[serde(flatten)]
    pub version: SpecificVersionResponse,
}

#[derive(Deserialize, Serialize)]
pub struct SpecificVersionResponse {
    /// Mullvad app version
    pub version: mullvad_version::Version,
    /// Changelog entries
    pub changelog: String,
    /// Installer details for different architectures
    pub installers: SpecificVersionArchitectureResponses,
}

/// Version details for supported architectures
#[derive(Deserialize, Serialize)]
pub struct SpecificVersionArchitectureResponses {
    /// Details for x86 installer
    pub x86: Option<SpecificVersionArchitectureResponse>,
    /// Details for ARM64 installer
    pub arm64: Option<SpecificVersionArchitectureResponse>,
}

#[derive(Deserialize, Serialize)]
pub struct SpecificVersionArchitectureResponse {
    /// Mirrors that host the artifact
    pub urls: Vec<String>,
    /// Size of the installer, in bytes
    pub size: usize,
    /// Hash of the installer, hexadecimal string
    pub sha256: String,
}

/// JSON response signature
#[derive(Deserialize, Serialize)]
pub struct ResponseSignature {
    pub keyid: key::VerifyingKey,
    pub sig: key::Signature,
}
