//! Deserializer and verifier of version metadata

use anyhow::Context;
use vec1::Vec1;

use super::key::*;
use super::Response;
use super::{PartialSignedResponse, ResponseSignature, SignedResponse};

impl SignedResponse {
    /// Deserialize some bytes to JSON, and verify them, including signature and expiry.
    /// If successful, the deserialized data is returned.
    ///
    /// This uses the keys in `trusted-metadata-signing-pubkeys`
    pub fn deserialize_and_verify(
        bytes: &[u8],
        min_metadata_version: usize,
    ) -> Result<Self, anyhow::Error> {
        Self::deserialize_and_verify_at_time(
            &crate::defaults::TRUSTED_METADATA_SIGNING_PUBKEYS,
            bytes,
            chrono::Utc::now(),
            min_metadata_version,
        )
    }

    /// Deserialize some bytes to JSON, and verify them, including signature and expiry.
    /// If successful, the deserialized data is returned.
    pub(crate) fn deserialize_and_verify_with_keys(
        keys: &Vec1<VerifyingKey>,
        bytes: &[u8],
        min_metadata_version: usize,
    ) -> Result<Self, anyhow::Error> {
        Self::deserialize_and_verify_at_time(keys, bytes, chrono::Utc::now(), min_metadata_version)
    }

    /// This method is used mostly for testing, and skips all verification.
    /// Own method to prevent accidental misuse.
    pub fn deserialize_insecure(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let partial_data: PartialSignedResponse =
            serde_json::from_slice(bytes).context("Invalid version JSON")?;
        let signed = serde_json::from_value(partial_data.signed)
            .context("Failed to deserialize response")?;
        Ok(Self {
            signatures: partial_data.signatures,
            signed,
        })
    }

    /// Deserialize some bytes to JSON, and verify them, including signature and expiry.
    /// If successful, the deserialized data is returned.
    fn deserialize_and_verify_at_time(
        keys: &Vec1<VerifyingKey>,
        bytes: &[u8],
        current_time: chrono::DateTime<chrono::Utc>,
        min_metadata_version: usize,
    ) -> Result<Self, anyhow::Error> {
        // Deserialize and verify signature
        let partial_data = deserialize_and_verify(keys, bytes)?;

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
                "Version metadata is too old: {}, must be at least {}",
                signed_response.metadata_version,
                min_metadata_version,
            );
        }

        Ok(SignedResponse {
            signatures: partial_data.signatures,
            signed: signed_response,
        })
    }
}

/// Deserialize arbitrary JSON object with a signature attached.
/// WARNING: This only verifies the signature, not expiration.
///
/// On success, this returns verified data and signature
pub(super) fn deserialize_and_verify(
    keys: &Vec1<VerifyingKey>,
    bytes: &[u8],
) -> anyhow::Result<PartialSignedResponse> {
    let partial_data: PartialSignedResponse =
        serde_json::from_slice(bytes).context("Invalid version JSON")?;

    let valid_keys: Vec<_> = keys.into_iter().map(|k| k.0).collect();

    // Check if one of the keys matches
    let Some((key, sig)) = partial_data.signatures.iter().find_map(|sig| match sig {
        // Check if ed25519 key matches
        ResponseSignature::Ed25519 { keyid, sig } if valid_keys.contains(&keyid.0) => {
            Some((keyid, sig))
        }
        // Ignore all non-matching key
        _ => None,
    }) else {
        anyhow::bail!("Unrecognized key");
    };

    // Serialize to canonical json format
    let canon_data = json_canon::to_vec(&partial_data.signed)
        .context("Failed to serialize to canonical JSON")?;

    // Check if the data is signed by our key
    key.0
        .verify_strict(&canon_data, &sig.0)
        .context("Signature verification failed")?;

    Ok(PartialSignedResponse {
        signatures: partial_data.signatures,
        // Deserialize again from canonicalized JSON in case something was lost
        signed: serde_json::from_slice(&canon_data)
            .context("Failed to deserialize canonical JSON")?,
    })
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use vec1::vec1;

    use super::*;

    /// Test that a valid signed version response is successfully deserialized and verified
    #[test]
    fn test_response_deserialization_and_verification() {
        let pubkey = hex::decode(include_str!("../../test-pubkey")).unwrap();
        let verifying_key =
            ed25519_dalek::VerifyingKey::from_bytes(&pubkey.try_into().unwrap()).unwrap();

        SignedResponse::deserialize_and_verify_at_time(
            &vec1![VerifyingKey(verifying_key)],
            include_bytes!("../../test-version-response.json"),
            // It's 1970 again
            chrono::DateTime::UNIX_EPOCH,
            // Accept any version
            0,
        )
        .expect("expected valid signed version metadata");

        // Reject expired data
        SignedResponse::deserialize_and_verify_at_time(
            &vec1![VerifyingKey(verifying_key)],
            include_bytes!("../../test-version-response.json"),
            // In the year 3000
            chrono::DateTime::from_str("3000-01-01T00:00:00Z").unwrap(),
            // Accept any version
            0,
        )
        .expect_err("expected expired version metadata");

        // Reject expired version number
        SignedResponse::deserialize_and_verify_at_time(
            &vec1![VerifyingKey(verifying_key)],
            include_bytes!("../../test-version-response.json"),
            chrono::DateTime::UNIX_EPOCH,
            usize::MAX,
        )
        .expect_err("expected rejected version number");
    }

    /// Test that invalid key types deserialized to "other"
    #[test]
    fn test_response_unknown_keytypes() {
        //let secret = "F6631A59EBBF8AADEAC64CC30A08A83FC7283F39DE53B7F1BFBA6BE52663DC94";
        let pubkey = "8F735E412015D8976079E5FA0E090100A43A34937CCFC3A2341219E30291DD39";
        let fakesig = "08954286A9284718B83CAADA5DF8A9A9DF0CE569F8EFF669D8C2A2E5945C809C465C38168E2F6018461DD8801DBFC74126A2ED9102F99A49F6DD54722C9B3605";
        let value = serde_json::json!({
            "signatures": [
                {
                    "keytype": "ed25519",
                    "keyid": pubkey,
                    "sig": fakesig,
                },
                {
                    "keytype": "new shiny key",
                    "keyid": "test 1",
                    "sig": "test 2",
                }
            ],
            "signed": {
                "metadata_expiry": "3000-01-01T00:00:00Z",
                "metadata_version": 0,
                "releases": []
            }
        });

        let bytes = serde_json::to_vec(&value).expect("serialize should succeed");

        let response =
            SignedResponse::deserialize_insecure(&bytes).expect("deserialization failed");

        let expected_key = VerifyingKey::from_hex(pubkey).unwrap();
        let expected_sig = Signature::from_hex(fakesig).unwrap();

        // Ed25519 key
        assert!(
            matches!(&response.signatures[0], ResponseSignature::Ed25519 { keyid, sig } if keyid == &expected_key && sig == &expected_sig),
            "unexpected response sig: {:?}",
            response.signatures[0]
        );

        // Unrecognized key type
        assert!(
            matches!(&response.signatures[1], ResponseSignature::Other { keyid, sig } if keyid == "test 1" && sig == "test 2"),
            "expected unrecognized key: {:?}",
            response.signatures[1]
        );
    }
}
