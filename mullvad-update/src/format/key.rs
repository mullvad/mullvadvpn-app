//! Key and signature types for API version response format

use std::{fmt, str::FromStr};

use anyhow::{bail, Context};
use ed25519_dalek::ed25519::signature::Signer;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// ed25519 secret/signing key
#[derive(Clone, PartialEq, zeroize::ZeroizeOnDrop)]
#[cfg_attr(test, derive(Debug))]
pub struct SecretKey(ed25519_dalek::SigningKey);

impl SecretKey {
    /// Generate a new secret ed25519 key
    #[cfg(feature = "sign")]
    pub fn generate() -> Self {
        let key = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
        SecretKey(key)
    }

    pub fn pubkey(&self) -> VerifyingKey {
        VerifyingKey(self.0.verifying_key())
    }

    /// Sign data using this key
    pub fn sign(&self, msg: &[u8]) -> Signature {
        Signature(self.0.sign(msg))
    }

    /// Convert bytes to a signing key, and zero the original bytes
    fn from_bytes(mut key: [u8; ed25519_dalek::SECRET_KEY_LENGTH]) -> Self {
        let secret = ed25519_dalek::SigningKey::from_bytes(&key);
        key.zeroize();
        SecretKey(secret)
    }
}

impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.as_bytes()))
    }
}

impl<'de> Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut key_s = String::deserialize(deserializer)?;
        let key = bytes_from_hex::<{ ed25519_dalek::SECRET_KEY_LENGTH }>(&key_s)
            .map_err(|err| serde::de::Error::custom(err.to_string()))?;
        key_s.zeroize();
        Ok(SecretKey::from_bytes(key))
    }
}

impl FromStr for SecretKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = bytes_from_hex::<{ ed25519_dalek::SECRET_KEY_LENGTH }>(s)?;
        Ok(Self::from_bytes(bytes))
    }
}

impl Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0.as_bytes()))
    }
}

/// ed25519 verifying key
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VerifyingKey(pub ed25519_dalek::VerifyingKey);

impl fmt::Display for VerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.as_bytes()))
    }
}

impl VerifyingKey {
    pub fn from_hex(s: &str) -> anyhow::Result<Self> {
        let bytes = bytes_from_hex::<{ ed25519_dalek::PUBLIC_KEY_LENGTH }>(s)?;
        Ok(Self(
            ed25519_dalek::VerifyingKey::from_bytes(&bytes).context("Invalid ed25519 key")?,
        ))
    }
}

impl<'de> Deserialize<'de> for VerifyingKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = String::deserialize(deserializer)?;
        let bytes = bytes_from_hex::<{ ed25519_dalek::PUBLIC_KEY_LENGTH }>(&bytes)
            .map_err(|err| serde::de::Error::custom(err.to_string()))?;
        let key = ed25519_dalek::VerifyingKey::from_bytes(&bytes).map_err(|_err| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Other("invalid verifying key"),
                &"valid ed25519 key",
            )
        })?;
        Ok(VerifyingKey(key))
    }
}

impl Serialize for VerifyingKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0.as_bytes()))
    }
}

/// ed25519 signature
#[derive(Debug, PartialEq)]
pub struct Signature(pub ed25519_dalek::Signature);

impl Signature {
    pub fn from_hex(s: &str) -> anyhow::Result<Self> {
        let bytes = bytes_from_hex::<{ ed25519_dalek::SIGNATURE_LENGTH }>(s)?;
        Ok(Self(ed25519_dalek::Signature::from_bytes(&bytes)))
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = String::deserialize(deserializer)?;
        let bytes = bytes_from_hex::<{ ed25519_dalek::SIGNATURE_LENGTH }>(&bytes)
            .map_err(|err| serde::de::Error::custom(err.to_string()))?;
        Ok(Signature(ed25519_dalek::Signature::from_bytes(&bytes)))
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0.to_bytes()))
    }
}

/// Deserialize a hex-encoded string to a bytes array of an exact size
fn bytes_from_hex<const SIZE: usize>(key: &str) -> anyhow::Result<[u8; SIZE]> {
    let bytes = hex::decode(key).context("invalid hex")?;
    if bytes.len() != SIZE {
        bail!(
            "expected hex-encoded string of {SIZE} bytes, found {} bytes",
            bytes.len()
        );
    }
    let mut key = [0u8; SIZE];
    key.copy_from_slice(&bytes);
    Ok(key)
}

#[cfg(test)]
mod test {
    use rand::RngCore;

    use super::*;

    #[test]
    fn test_serialization_and_deserialization() {
        let mut secret = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret);

        let secret_hex = hex::encode(secret);
        let secret = SecretKey::from_str(&hex::encode(secret)).unwrap();

        let pubkey = secret.pubkey();
        let pubkey_hex = hex::encode(pubkey.0);

        // Test serialization
        let actual = serde_json::json!({
            "secret": secret,
            "key": pubkey,
        });
        let expected: serde_json::Value = serde_json::from_str(&format!(
            r#"{{
            "secret": "{secret_hex}",
            "key": "{pubkey_hex}"
        }}"#
        ))
        .unwrap();

        assert_eq!(actual, expected);

        // Test deserialization
        let secret_obj = actual.as_object().unwrap().get("secret").unwrap().clone();
        let deserialized_secret: SecretKey = serde_json::from_value(secret_obj).unwrap();

        let pubkey_obj = actual.as_object().unwrap().get("key").unwrap().clone();
        let deserialized_pubkey: VerifyingKey = serde_json::from_value(pubkey_obj).unwrap();

        assert_eq!(deserialized_secret, secret);
        assert_eq!(deserialized_pubkey, pubkey);
    }
}
