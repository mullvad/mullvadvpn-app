//! Serializer for signed version response data
//!
//! Signing attaches a signature, and leaves the original JSON data in the "signed" key:
//!
//! ```ignore
//! {
//!     "signature": {
//!         "keyid": "...",
//!         "sig": "..."
//!     }
//!     "signed": {
//!         ...
//!     }
//! }
//! ```

use anyhow::Context;
use serde::Serialize;

use super::{key, PartialSignedResponse, Response, ResponseSignature, SignedResponse};

impl SignedResponse {
    pub fn sign(key: key::SecretKey, response: Response) -> anyhow::Result<SignedResponse> {
        // Refuse to sign expired data
        if response.metadata_expiry < chrono::Utc::now() {
            anyhow::bail!("Signing failed since the data has expired");
        }

        // Sign it
        let partial_signed = sign(&key, &response)?;

        // Attempt to deserialize signed part as response
        // Probably unnecessary; mostly in case canonical JSON lost something
        let response: Response = serde_json::from_value(partial_signed.signed)?;

        Ok(SignedResponse {
            signatures: partial_signed.signatures,
            signed: response,
        })
    }
}

/// Serialize JSON to bytes, with a signature attached, signed using `key`
fn sign<T: Serialize>(
    key: &key::SecretKey,
    unsigned_value: &T,
) -> anyhow::Result<PartialSignedResponse> {
    // Serialize unsigned data to canonical JSON
    let unsigned_canon =
        json_canon::to_vec(&unsigned_value).context("Failed to canonicalize JSON")?;

    // Generate signature for the canonical JSON
    let sig = key.sign(&unsigned_canon);

    // Deserialize in case something was lost during serialization
    let signed =
        serde_json::from_slice(&unsigned_canon).context("Failed to deserialize canonical JSON")?;

    // Attach signature
    Ok(PartialSignedResponse {
        signatures: vec![ResponseSignature::Ed25519 {
            keyid: key.pubkey(),
            sig,
        }],
        // Attach now-signed data
        signed,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::format::deserializer::deserialize_and_verify;
    use serde_json::json;
    use vec1::vec1;

    #[test]
    fn test_sign() -> anyhow::Result<()> {
        // Generate key and data
        let key = key::SecretKey::generate();
        let pubkey = key.pubkey();

        let data = json!({
            "stuff": "I can prove that I wrote this"
        });

        // Verify that we can deserialize and verify the data
        let partial = sign(&key, &data).context("Signing failed")?;

        assert!(
            matches!(&partial.signatures[0], ResponseSignature::Ed25519 {
            keyid,
            ..
        } if keyid == &pubkey)
        );

        let bytes = serde_json::to_vec(&partial)?;

        deserialize_and_verify(&vec1![pubkey.clone()], &bytes)?;

        // Verify that an irrelevant key is ignored
        let invalid_key = key::SecretKey::generate();
        let invalid_pubkey = invalid_key.pubkey();

        deserialize_and_verify(&vec1![pubkey.clone(), invalid_pubkey.clone()], &bytes)?;

        // Wrong public key only fails
        deserialize_and_verify(&vec1![invalid_pubkey], &bytes).unwrap_err();

        Ok(())
    }

    #[test]
    fn test_sign_multiple() -> anyhow::Result<()> {
        // Generate keys and data
        let key = key::SecretKey::generate();
        let pubkey = key.pubkey();

        let key2 = key::SecretKey::generate();
        let pubkey2 = key2.pubkey();

        let invalid_key = key::SecretKey::generate();
        let invalid_pubkey = invalid_key.pubkey();

        let data = json!({
            "stuff": "I can prove that I wrote this"
        });

        // Sign with two keys
        let mut partial = sign(&key, &data).context("Signing failed")?;
        let partial2 = sign(&key2, &data).context("Signing failed")?;
        partial.signatures.extend(partial2.signatures);

        let bytes = serde_json::to_vec(&partial)?;

        // Accept either (or both) keys
        deserialize_and_verify(&vec1![pubkey.clone(), pubkey2.clone()], &bytes)?;
        deserialize_and_verify(&vec1![pubkey2.clone()], &bytes)?;
        deserialize_and_verify(&vec1![pubkey.clone()], &bytes)?;

        // Ignore irrelevant key
        deserialize_and_verify(
            &vec1![pubkey.clone(), pubkey2.clone(), invalid_pubkey.clone()],
            &bytes,
        )?;
        deserialize_and_verify(&vec1![pubkey2, invalid_pubkey.clone()], &bytes)?;
        deserialize_and_verify(&vec1![invalid_pubkey.clone(), pubkey], &bytes)?;

        // Using wrong public key fails
        deserialize_and_verify(&vec1![invalid_pubkey], &bytes).unwrap_err();

        Ok(())
    }
}
