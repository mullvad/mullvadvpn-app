//! Deserializer for version API response format

use anyhow::Context;
use serde::Deserialize;

/// JSON response including signature and signed content
/// This type does not implement [serde::Deserialize] to prevent accidental deserialization without
/// signature verification.
pub struct SignedResponse {
    /// Signature of the canonicalized JSON of `signed`
    pub signature: ResponseSignature,
    /// Content signed by `signature`
    pub signed: Response,
}

/// Helper class that leaves the signed data untouched
/// Note that deserializing doesn't verify anything
#[derive(serde::Deserialize)]
struct PartialSignedResponse {
    /// Signature of the canonicalized JSON of `signed`
    pub signature: ResponseSignature,
    /// Content signed by `signature`
    pub signed: serde_json::Value,
}

impl SignedResponse {
    /// Deserialize some bytes to JSON, and verify them, including signature and expiry.
    /// If successful, the deserialized data is returned.
    pub fn deserialize_and_verify(key: VerifyingKey, bytes: &[u8]) -> Result<Self, anyhow::Error> {
        Self::deserialize_and_verify_at_time(key, bytes, chrono::Utc::now())
    }

    /// This method is used for testing, and skips all verification.
    /// Own method to prevent accidental misuse.
    #[cfg(test)]
    pub fn deserialize_and_verify_insecure(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let partial_data: PartialSignedResponse =
            serde_json::from_slice(bytes).context("Invalid version JSON")?;
        let signed = serde_json::from_value(partial_data.signed)
            .context("Failed to deserialize response")?;
        Ok(Self {
            signature: partial_data.signature,
            signed,
        })
    }

    /// Deserialize some bytes to JSON, and verify them, including signature and expiry.
    /// If successful, the deserialized data is returned.
    fn deserialize_and_verify_at_time(
        key: VerifyingKey,
        bytes: &[u8],
        current_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Self, anyhow::Error> {
        let partial_data: PartialSignedResponse =
            serde_json::from_slice(bytes).context("Invalid version JSON")?;

        // Check if the key matches
        if partial_data.signature.keyid.0 != key.0 {
            anyhow::bail!("Unrecognized key");
        }

        // Serialize to canonical json format
        let canon_data = json_canon::to_vec(&partial_data.signed)
            .context("Failed to serialize to canonical JSON")?;

        // Check if the data is signed by our key
        partial_data
            .signature
            .keyid
            .0
            .verify_strict(&canon_data, &partial_data.signature.sig.0)
            .context("Signature verification failed")?;

        // Deserialize the canonical JSON to structured representation
        let signed_response: Response =
            serde_json::from_slice(&canon_data).context("Failed to deserialize response")?;

        // Reject time if the data has expired
        if current_time >= signed_response.expires {
            anyhow::bail!(
                "Version metadata has expired: valid until {}",
                signed_response.expires
            );
        }

        Ok(SignedResponse {
            signature: partial_data.signature,
            signed: signed_response,
        })
    }
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
    /// Hash of the installer, hexadecimal string
    pub sha256: String,
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test that a valid signed version response is successfully deserialized and verified
    #[test]
    fn test_response_deserialization_and_verification() {
        const TEST_PUBKEY: &str =
            "AEC24A08466F3D6A1EDCDB2AD3C234428AB9D991B6BEA7F53CB9F172E6CB40D8";
        let pubkey = hex::decode(TEST_PUBKEY).unwrap();
        let verifying_key =
            ed25519_dalek::VerifyingKey::from_bytes(&pubkey.try_into().unwrap()).unwrap();

        SignedResponse::deserialize_and_verify_at_time(
            VerifyingKey(verifying_key),
            include_bytes!("../test-version-response.json"),
            // It's 1970 again
            chrono::DateTime::UNIX_EPOCH,
        )
        .expect("expected valid signed version metadata");
    }
}
