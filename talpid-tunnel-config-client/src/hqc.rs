use pqcrypto_hqc::hqc256;
use pqcrypto_traits::kem::{Ciphertext as _, PublicKey as _, SharedSecret as _};
use sha2::{Digest as _, Sha256};

pub const ALGORITHM_NAME: &str = "HQC-256";

pub struct Keypair {
    public_key: hqc256::PublicKey,
    secret_key: hqc256::SecretKey,
}

impl Keypair {
    /// Returns the encapsulation key. This is sometimes called the public key.
    ///
    /// This is the key to send to the peer you want to negotiate a shared secret with.
    pub fn encapsulation_key(&self) -> Vec<u8> {
        self.public_key.as_bytes().to_vec()
    }

    /// Decapsulates a shared secret that was encapsulated to our encapsulation key.
    ///
    // Always inline in order to try to avoid potential copies of `shared_secret` to multiple places
    // on the stack. This is almost pointless as with optimization all bets are off regarding where
    // the shared secrets will end up in memory. In the future we can try to do better, by
    // cleaning the stack. But this is not trivial. Please see:
    // https://github.com/RustCrypto/KEMs/issues/70
    #[inline(always)]
    pub fn decapsulate(&self, ciphertext_slice: &[u8]) -> Result<[u8; 32], super::Error> {
        let ciphertext = hqc256::Ciphertext::from_bytes(ciphertext_slice).map_err(|_| {
            super::Error::InvalidCiphertextLength {
                algorithm: ALGORITHM_NAME,
                actual: ciphertext_slice.len(),
                expected: hqc256::ciphertext_bytes(),
            }
        })?;
        let shared_secret = hqc256::decapsulate(&ciphertext, &self.secret_key);

        // HQC outputs a 64 byte shared secret. But we need 32 bytes for the WireGuard PSK.
        // Our ephemeral peer API says shared secrets with a length other than 32 bytes should
        // be passed through sha256 in order to squash the entropy into 32 bytes.
        let output_shared_secret = Sha256::digest(shared_secret.as_bytes());
        Ok(output_shared_secret.into())
    }
}

pub fn keypair() -> Keypair {
    let (public_key, secret_key) = hqc256::keypair();
    Keypair {
        public_key,
        secret_key,
    }
}
