//! Deserializer for version API response format

use serde::Deserialize;

/// JSON response including signature and signed content
/// Note that signature verification isn't accomplished by deserializing
#[derive(Deserialize)]
pub struct SignedResponse {
    /// Signature of the canonicalized JSON of `signed`
    pub signature: ResponseSignature,
    /// Content signed by `signature`
    pub signed: Response,
}

/// JSON response signature
#[derive(Deserialize)]
pub struct ResponseSignature {
    pub keyid: VerifyingKey,
    pub sig: Signature,
}

/// ed25519 verifying key
pub struct VerifyingKey(pub ed25519_dalek::VerifyingKey);

impl<'de> Deserialize<'de> for VerifyingKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = String::deserialize(deserializer).and_then(|string| {
            bytes_from_hex::<D, { ed25519_dalek::PUBLIC_KEY_LENGTH }>(&string)
        })?;
        let key = ed25519_dalek::VerifyingKey::from_bytes(&bytes).map_err(|_err| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Other("invalid verifying key"),
                &"valid ed25519 key",
            )
        })?;
        Ok(VerifyingKey(key))
    }
}

/// ed25519 signature
pub struct Signature(pub ed25519_dalek::Signature);

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = String::deserialize(deserializer)
            .and_then(|string| bytes_from_hex::<D, { ed25519_dalek::SIGNATURE_LENGTH }>(&string))?;
        Ok(Signature(ed25519_dalek::Signature::from_bytes(&bytes)))
    }
}

/// Deserialize a hex-encoded string to a bytes array of an exact size
fn bytes_from_hex<'de, D, const SIZE: usize>(key: &str) -> Result<[u8; SIZE], D::Error>
where
    D: serde::Deserializer<'de>,
{
    let bytes = hex::decode(key).map_err(|_err| {
        serde::de::Error::invalid_value(
            serde::de::Unexpected::Other("hex-encoded string"),
            &"valid hex",
        )
    })?;
    if bytes.len() != SIZE {
        let expected = format!("hex-encoded string of {SIZE} bytes");
        return Err(serde::de::Error::invalid_length(
            bytes.len(),
            &expected.as_str(),
        ));
    }
    let mut key = [0u8; SIZE];
    key.copy_from_slice(&bytes);
    Ok(key)
}

/// Signed JSON response, not including the signature
#[derive(Deserialize)]
pub struct Response {
    /// When the signature expires
    pub expires: chrono::DateTime<chrono::Utc>,
    /// Stable version response
    pub stable: VersionResponse,
    /// Beta version response
    pub beta: Option<VersionResponse>,
}

#[derive(Deserialize)]
pub struct VersionResponse {
    /// The current version in this channel
    pub current: SpecificVersionResponse,
    /// The version being rolled out in this channel
    pub next: Option<NextSpecificVersionResponse>,
}

#[derive(Deserialize)]
pub struct NextSpecificVersionResponse {
    /// The percentage of users that should receive the new version.
    pub rollout: f32,
    #[serde(flatten)]
    pub version: SpecificVersionResponse,
}

#[derive(Deserialize)]
pub struct SpecificVersionResponse {
    /// Mullvad app version
    pub version: mullvad_version::Version,
    /// Changelog entries
    pub changelog: String,
    /// Installer details for different architectures
    pub installers: SpecificVersionArchitectureResponses,
}

/// Version details for supported architectures
#[derive(Deserialize)]
pub struct SpecificVersionArchitectureResponses {
    /// Details for x86 installer
    pub x86: Option<SpecificVersionArchitectureResponse>,
    /// Details for ARM64 installer
    pub arm64: Option<SpecificVersionArchitectureResponse>,
}

#[derive(Deserialize)]
pub struct SpecificVersionArchitectureResponse {
    /// Mirrors that host the artifact
    pub urls: Vec<String>,
    /// Size of the installer, in bytes
    pub size: usize,
    /// TODO: hash of the installer, in bytes
    pub sha256: String,
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test that a valid version response is successfully deserialized
    #[test]
    fn test_response_deserialization() {
        let _: SignedResponse =
            serde_json::from_str(include_str!("../test-version-response.json")).unwrap();
    }
}
