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
        if response.expires < chrono::Utc::now() {
            anyhow::bail!("Signing failed since the data has expired");
        }

        // Sign it
        let partial_signed = sign(&key, &response)?;

        // Attempt to deserialize signed part as response
        // Probably unnecessary; mostly in case canonical JSON lost something
        let response: Response = serde_json::from_value(partial_signed.signed)?;

        Ok(SignedResponse {
            signature: partial_signed.signature,
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
        signature: ResponseSignature {
            keyid: key.pubkey(),
            sig,
        },
        // Attach now-signed data
        signed,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::format::deserializer::deserialize_and_verify;
    use serde_json::json;

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

        assert_eq!(partial.signature.keyid, pubkey);

        let bytes = serde_json::to_vec(&partial)?;

        deserialize_and_verify(&pubkey, &bytes)?;

        Ok(())
    }
}
