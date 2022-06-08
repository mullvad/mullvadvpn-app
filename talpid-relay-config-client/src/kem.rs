use std::fmt;

use super::Error;

use classic_mceliece_rust::{
    crypto_kem_dec, crypto_kem_keypair, CRYPTO_BYTES, CRYPTO_CIPHERTEXTBYTES,
    CRYPTO_PUBLICKEYBYTES, CRYPTO_SECRETKEYBYTES,
};
use talpid_types::net::wireguard::PresharedKey;

const STACK_SIZE: usize = 8 * 1024 * 1024;

#[derive(Debug)]
pub struct PublicKey(Box<[u8; CRYPTO_PUBLICKEYBYTES]>);

impl PublicKey {
    pub fn into_vec(self) -> Vec<u8> {
        (self.0 as Box<[u8]>).into_vec()
    }
}

pub struct SecretKey(Box<[u8; CRYPTO_SECRETKEYBYTES]>);

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretKey").finish()
    }
}

pub async fn generate_keys() -> Result<(PublicKey, SecretKey), Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let gen_key = move || {
        let mut rng = rand::thread_rng();
        let mut pubkey = Box::new([0u8; CRYPTO_PUBLICKEYBYTES]);
        let mut secret = Box::new([0u8; CRYPTO_SECRETKEYBYTES]);
        crypto_kem_keypair(&mut pubkey, &mut secret, &mut rng).map_err(|error| {
            log::error!("KEM keypair generation failed: {error}");
            Error::KeyGenerationFailed
        })?;

        Ok((PublicKey(pubkey), SecretKey(secret)))
    };

    std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            tx.send(gen_key()).unwrap();
        })
        .unwrap();

    rx.await.unwrap()
}

pub fn decapsulate(secret: &SecretKey, ciphertext: &[u8]) -> Result<PresharedKey, Error> {
    let ct: [u8; CRYPTO_CIPHERTEXTBYTES] = ciphertext
        .try_into()
        .map_err(|_| Error::InvalidCiphertext)?;
    let mut psk = [0u8; CRYPTO_BYTES];

    crypto_kem_dec(&mut psk, &ct, &secret.0).map_err(|error| {
        log::error!("KEM decapsulation failed: {error}");
        Error::DecapsulationError
    })?;

    Ok(PresharedKey::from(psk))
}
