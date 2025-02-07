//! Deserializer and verifier of version metadata

use anyhow::Context;

use super::key::*;
use super::Response;
use super::{PartialSignedResponse, SignedResponse};

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
        // Deserialize and verify signature
        let partial_data = deserialize_and_verify(&key, bytes)?;

        // Deserialize the canonical JSON to structured representation
        let signed_response: Response = serde_json::from_value(partial_data.signed)
            .context("Failed to deserialize response")?;

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
        // Serialize back in case something was lost during deserialization
        signed: serde_json::from_slice(&canon_data)
            .context("Failed to serialize canonical JSON")?,
    })
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
            include_bytes!("../../test-version-response.json"),
            // It's 1970 again
            chrono::DateTime::UNIX_EPOCH,
        )
        .expect("expected valid signed version metadata");
    }
}
