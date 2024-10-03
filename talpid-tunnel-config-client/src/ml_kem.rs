use std::time::Instant;

use ml_kem::array::typenum::marker_traits::Unsigned;
use ml_kem::kem::Decapsulate;
use ml_kem::{Ciphertext, EncodedSizeUser, KemCore, MlKem1024, MlKem1024Params};

/// Use the strongest variant of ML-KEM. It is fast and the keys are small, so there is no practical
/// benefit of going with anything lower. The servers also only supports the strongest variant.
pub const ALGORITHM_NAME: &str = "ML-KEM-1024";

/// The number of bytes in an ML-KEM 1024 ciphertext.
const CIPHERTEXT_LEN: usize = <MlKem1024 as KemCore>::CiphertextSize::USIZE;

pub struct Keypair {
    encapsulation_key: ml_kem::kem::EncapsulationKey<MlKem1024Params>,
    decapsulation_key: ml_kem::kem::DecapsulationKey<MlKem1024Params>,
}

impl Keypair {
    pub fn encapsulation_key(&self) -> Vec<u8> {
        self.encapsulation_key.as_bytes().as_slice().to_vec()
    }

    #[inline(always)]
    pub fn decapsulate(&self, ciphertext_slice: &[u8]) -> Result<[u8; 32], super::Error> {
        let start = Instant::now();
        // Convert the ciphertext byte slice into the appropriate Array<u8, ...> type.
        // This involves validating the length of the ciphertext.
        let ciphertext_array =
            <Ciphertext<MlKem1024>>::try_from(ciphertext_slice).map_err(|_| {
                super::Error::InvalidCiphertextLength {
                    algorithm: ALGORITHM_NAME,
                    actual: ciphertext_slice.len(),
                    expected: CIPHERTEXT_LEN,
                }
            })?;

        let shared_secret = self
            .decapsulation_key
            .decapsulate(&ciphertext_array)
            .unwrap();
        log::debug!(
            "ML-KEM decapsulation took {} ms",
            start.elapsed().as_millis()
        );
        Ok(shared_secret.0)
    }
}

pub fn keypair() -> Keypair {
    let (decapsulation_key, encapsulation_key) =
        ml_kem::MlKem1024::generate(&mut rand::thread_rng());
    Keypair {
        encapsulation_key,
        decapsulation_key,
    }
}
