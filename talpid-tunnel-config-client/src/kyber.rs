use pqc_kyber::SecretKey;

pub const CRYPTO_CIPHERTEXTBYTES: usize = 1088;

pub fn decapsulate(secret_key: SecretKey, ciphertext: [u8; CRYPTO_CIPHERTEXTBYTES]) -> [u8; 32] {
    pqc_kyber::decapsulate(ciphertext.as_slice(), secret_key.as_slice()).unwrap()
}
