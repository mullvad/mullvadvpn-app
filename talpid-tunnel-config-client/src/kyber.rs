use pqc_kyber::SecretKey;

pub use pqc_kyber::{keypair, KYBER_CIPHERTEXTBYTES, KyberError};

pub fn decapsulate(secret_key: SecretKey, ciphertext: [u8; KYBER_CIPHERTEXTBYTES]) -> Result<[u8; 32], KyberError> {
    pqc_kyber::decapsulate(ciphertext.as_slice(), secret_key.as_slice())
}
