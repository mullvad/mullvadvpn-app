use classic_mceliece_rust::{keypair_boxed, SharedSecret};

/// The `keypair_boxed` function needs just under 1 MiB of stack in debug
/// builds. Even though it probably works to run it directly on the main
/// thread on all OSes, we take this precaution and always generate the huge
/// keys on a separate thread with a large enough stack.
const STACK_SIZE: usize = 2 * 1024 * 1024;

pub use classic_mceliece_rust::{Ciphertext, PublicKey, SecretKey, CRYPTO_CIPHERTEXTBYTES};

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

pub fn decapsulate(secret: &SecretKey, ciphertext: &Ciphertext) -> SharedSecret<'static> {
    classic_mceliece_rust::decapsulate_boxed(ciphertext, secret)
}
