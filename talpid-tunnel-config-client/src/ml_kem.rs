use ml_kem::array::typenum::marker_traits::Unsigned;
use ml_kem::kem::Decapsulate;
use ml_kem::{Ciphertext, EncodedSizeUser, KemCore, MlKem1024, MlKem1024Params};

/// Use the strongest variant of ML-KEM. It is fast and the keys are small, so there is no practical
/// benefit of going with anything lower. The servers also only supports the strongest variant.
const ALGORITHM_NAME: &str = "ML-KEM-1024";

/// The number of bytes in an ML-KEM 1024 ciphertext.
const CIPHERTEXT_LEN: usize = <MlKem1024 as KemCore>::CiphertextSize::USIZE;

pub struct Keypair {
    encapsulation_key: ml_kem::kem::EncapsulationKey<MlKem1024Params>,
    decapsulation_key: ml_kem::kem::DecapsulationKey<MlKem1024Params>,
}

impl Keypair {
    /// Returns the encapsulation key. This is sometimes called the public key.
    ///
    /// This is the key to send to the peer you want to negotiate a shared secret with.
    pub fn encapsulation_key(&self) -> Vec<u8> {
        self.encapsulation_key.as_bytes().as_slice().to_vec()
    }

    pub fn algorithm_name(&self) -> &'static str {
        ALGORITHM_NAME
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
        // Convert the ciphertext byte slice into the appropriate Array<u8, ...> type.
        // This involves validating the length of the ciphertext.
        let ciphertext_array =
            <Ciphertext<MlKem1024>>::try_from(ciphertext_slice).map_err(|_| {
                super::Error::InvalidCiphertextLength {
                    algorithm: self.algorithm_name(),
                    actual: ciphertext_slice.len(),
                    expected: CIPHERTEXT_LEN,
                }
            })?;

        // Decapsulate the shared secret. This is an infallible operation but
        // must due to the signature of the trait it is implemented via return a
        // Result that we must unwrap... For now. Please see:
        // https://github.com/RustCrypto/KEMs/pull/59
        let shared_secret = self
            .decapsulation_key
            .decapsulate(&ciphertext_array)
            .unwrap();
        Ok(shared_secret.0)
    }
}

/// Generates and returns an ML-KEM keypair.
pub fn keypair() -> Keypair {
    let (decapsulation_key, encapsulation_key) =
        ml_kem::MlKem1024::generate(&mut rand::thread_rng());
    Keypair {
        encapsulation_key,
        decapsulation_key,
    }
}
