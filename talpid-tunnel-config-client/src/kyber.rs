use pqc_kyber::{SecretKey, KYBER_CIPHERTEXTBYTES};

pub use pqc_kyber::{keypair, KyberError};

/// Use the strongest variant of Kyber. It is fast and the keys are small, so there is no practical
/// benefit of going with anything lower.
pub const ALGORITHM_NAME: &str = "Kyber1024";

// Always inline in order to try to avoid potential copies of `shared_secret` to multiple places on the stack.
#[inline(always)]
pub fn decapsulate(
    secret_key: SecretKey,
    ciphertext_slice: &[u8],
) -> Result<[u8; 32], super::Error> {
    // The `pqc_kyber` library takes a byte slice. But we convert it into an array
    // in order to catch the length mismatch error and report it better than `pqc_kyber` would.
    let ciphertext_array =
        <[u8; KYBER_CIPHERTEXTBYTES]>::try_from(ciphertext_slice).map_err(|_| {
            super::Error::InvalidCiphertextLength {
                algorithm: ALGORITHM_NAME,
                actual: ciphertext_slice.len(),
                expected: KYBER_CIPHERTEXTBYTES,
            }
        })?;
    let shared_secret = pqc_kyber::decapsulate(ciphertext_array.as_slice(), secret_key.as_slice())
        .map_err(super::Error::FailedDecapsulateKyber)?;
    Ok(shared_secret)
}
