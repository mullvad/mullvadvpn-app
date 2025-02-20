//! Deserializer and verifier of version metadata

use anyhow::Context;

use super::key::*;
use super::Response;
use super::{PartialSignedResponse, SignedResponse};

impl SignedResponse {
    /// Deserialize some bytes to JSON, and verify them, including signature and expiry.
    /// If successful, the deserialized data is returned.
    pub fn deserialize_and_verify(
        key: &VerifyingKey,
        bytes: &[u8],
        min_metadata_version: usize,
    ) -> Result<Self, anyhow::Error> {
        Self::deserialize_and_verify_at_time(key, bytes, chrono::Utc::now(), min_metadata_version)
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
        key: &VerifyingKey,
        bytes: &[u8],
        current_time: chrono::DateTime<chrono::Utc>,
        min_metadata_version: usize,
    ) -> Result<Self, anyhow::Error> {
        // Deserialize and verify signature
        let partial_data = deserialize_and_verify(&key, bytes)?;

        // Deserialize the canonical JSON to structured representation
        let signed_response: Response = serde_json::from_value(partial_data.signed)
            .context("Failed to deserialize response")?;

        // Reject time if the data has expired
        if current_time >= signed_response.metadata_expiry {
            anyhow::bail!(
                "Version metadata has expired: valid until {}",
                signed_response.metadata_expiry
            );
        }

        // Reject data if the version counter is below `min_metadata_version`
        if signed_response.metadata_version < min_metadata_version {
            anyhow::bail!(
                "Version metadata has is too old: {}, must be at least {}",
                signed_response.metadata_version,
                min_metadata_version,
            );
        }

        Ok(SignedResponse {
            signature: partial_data.signature,
            signed: signed_response,
        })
    }
}

/// Deserialize arbitrary JSON object with a signature attached.
/// WARNING: This only verifies the signature, not expiration.
///
/// On success, this returns verified data and signature
pub(super) fn deserialize_and_verify(
    key: &VerifyingKey,
    bytes: &[u8],
) -> anyhow::Result<PartialSignedResponse> {
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

    Ok(PartialSignedResponse {
        signature: partial_data.signature,
        signed: partial_data.signed,
    })
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    /// Test that a valid signed version response is successfully deserialized and verified
    #[test]
    fn test_response_deserialization_and_verification() {
        let pubkey = hex::decode(include_str!("../../test-pubkey")).unwrap();
        let verifying_key =
            ed25519_dalek::VerifyingKey::from_bytes(&pubkey.try_into().unwrap()).unwrap();

        SignedResponse::deserialize_and_verify_at_time(
            &VerifyingKey(verifying_key),
            include_bytes!("../../test-version-response.json"),
            // It's 1970 again
            chrono::DateTime::UNIX_EPOCH,
            // Accept any version
            0,
        )
        .expect("expected valid signed version metadata");

        // Reject expired data
        SignedResponse::deserialize_and_verify_at_time(
            &VerifyingKey(verifying_key),
            include_bytes!("../../test-version-response.json"),
            // In the year 3000
            chrono::DateTime::from_str("3000-01-01T00:00:00Z").unwrap(),
            // Accept any version
            0,
        )
        .expect_err("expected expired version metadata");

        // Reject expired version number
        SignedResponse::deserialize_and_verify_at_time(
            &VerifyingKey(verifying_key),
            include_bytes!("../../test-version-response.json"),
            chrono::DateTime::UNIX_EPOCH,
            usize::MAX,
        )
        .expect_err("expected rejected version number");
    }
}
