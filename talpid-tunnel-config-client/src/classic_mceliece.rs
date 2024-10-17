use classic_mceliece_rust::{keypair_boxed, Ciphertext, CRYPTO_CIPHERTEXTBYTES};
pub use classic_mceliece_rust::{PublicKey, SecretKey, SharedSecret};

/// The `keypair_boxed` function needs just under 1 MiB of stack in debug
/// builds.
const STACK_SIZE: usize = 2 * 1024 * 1024;

/// Use the smallest CME variant with NIST security level 3. This variant has significantly smaller
/// keys than the larger variants, and is considered safe.
pub const ALGORITHM_NAME: &str = "Classic-McEliece-460896f-round3";

pub async fn generate_keys() -> (PublicKey<'static>, SecretKey<'static>) {
    let (tx, rx) = tokio::sync::oneshot::channel();

    // We fork off the key computation to a separate thread for two reasons:
    // * The computation uses a lot of stack, and we don't want to rely on the default
    //   stack being large enough or having enough space left.
    // * The computation takes a long time and must not block the async runtime thread.
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
    secret: &SecretKey<'_>,
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
