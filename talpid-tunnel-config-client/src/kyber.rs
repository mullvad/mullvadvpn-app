use pqc_kyber::SecretKey;

pub fn decapsulate(secret_key: SecretKey, ciphertext: [u8; 1088]) -> [u8; 32] {
    pqc_kyber::decapsulate(ciphertext.as_slice(), secret_key.as_slice()).unwrap()
}
