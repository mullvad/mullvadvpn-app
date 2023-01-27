use classic_mceliece_rust::{
    keypair_boxed, Ciphertext, PublicKey, SecretKey, SharedSecret, CRYPTO_CIPHERTEXTBYTES,
};

/// The `keypair_boxed` function needs just under 1 MiB of stack in debug
/// builds. Even though it probably works to run it directly on the main
/// thread on all OSes, we take this precaution and always generate the huge
/// keys on a separate thread with a large enough stack.
const STACK_SIZE: usize = 2 * 1024 * 1024;

/// Use the smallest CME variant with NIST security level 3. This variant has significantly smaller
/// keys than the larger variants, and is considered safe.
pub const ALGORITHM_NAME: &str = "Classic-McEliece-460896f-round3";

pub async fn generate_keys() -> (PublicKey<'static>, SecretKey<'static>) {
    let (tx, rx) = tokio::sync::oneshot::channel();

    std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let keypair = keypair_boxed(&mut rand::thread_rng());
            let _ = tx.send(keypair);
        })
        .unwrap();

    rx.await.unwrap()
}

pub fn decapsulate(
    secret: &SecretKey,
    ciphertext_slice: &[u8],
) -> Result<SharedSecret<'static>, super::Error> {
    let ciphertext_array =
        <[u8; CRYPTO_CIPHERTEXTBYTES]>::try_from(ciphertext_slice).map_err(|_| {
            super::Error::InvalidCiphertextLength {
                algorithm: ALGORITHM_NAME,
                actual: ciphertext_slice.len(),
                expected: CRYPTO_CIPHERTEXTBYTES,
            }
        })?;
    let ciphertext = Ciphertext::from(ciphertext_array);
    Ok(classic_mceliece_rust::decapsulate_boxed(
        &ciphertext,
        secret,
    ))
}
