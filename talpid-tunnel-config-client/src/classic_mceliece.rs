use classic_mceliece_rust::{keypair_boxed, SharedSecret};

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
