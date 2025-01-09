use std::sync::OnceLock;

use classic_mceliece_rust::{keypair_boxed, Ciphertext, CRYPTO_CIPHERTEXTBYTES};
pub use classic_mceliece_rust::{PublicKey, SecretKey, SharedSecret};
use tokio::sync::{mpsc, Mutex};

/// The `keypair_boxed` function needs just under 1 MiB of stack in debug
/// builds.
const STACK_SIZE: usize = 2 * 1024 * 1024;

/// Number of McEliece key pairs to buffer. Note that, using the below algorithm, they take up around
/// 537 kB each. We therefore only buffer two, which is the largest useful amount, in case of multihop.
pub const BUFSIZE: usize = 2;

/// Use the smallest CME variant with NIST security level 3. This variant has significantly smaller
/// keys than the larger variants, and is considered safe.
pub const ALGORITHM_NAME: &str = "Classic-McEliece-460896f-round3";

type KeyPair = (PublicKey<'static>, SecretKey<'static>);

static KEYPAIR_RX: OnceLock<Mutex<mpsc::Receiver<KeyPair>>> = OnceLock::new();

/// Spawn a worker that pre computes `bufsize` McEliece key pairs in a separate thread, which can be
/// fetched asynchronously using the returned channel.
///
/// As it can take upwards of 200 ms to generate McEliece key pairs, it needs to be done before we
/// start connecting to the tunnel.
pub fn spawn_keypair_worker(bufsize: usize) -> mpsc::Receiver<KeyPair> {
    // As one of the key pairs will be buffered by the stack of the spawned thread, we reduce the
    // capacity of the channel by one
    let bufsize = bufsize.checked_sub(1).expect("bufsize must be at least 1");
    let (tx, rx) = mpsc::channel(bufsize);
    // We fork off the key computation to a separate thread for two reasons:
    // * The computation uses a lot of stack, and we don't want to rely on the default stack being
    //   large enough or having enough space left.
    // * The computation takes a long time and must not block the async runtime thread.
    std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || loop {
            let keypair = keypair_boxed(&mut rand::thread_rng());
            if tx.blocking_send(keypair).is_err() {
                return;
            }
        })
        .unwrap();

    rx
}

pub async fn generate_keys() -> KeyPair {
    get_or_init_keypair_receiver()
        .lock()
        .await
        .recv()
        .await
        .expect("Failed to receive key pair, generating working expectedly closed.")
}

/// Returns a receiver for McEliece key pairs used by PQ tunnels. These are generated in a separate
/// thread to reduce latency when connecting.
///
/// The first call will spawn the worker which immedietly starts to compute and buffer [`BUFSIZE`]
/// of key pairs.
pub fn get_or_init_keypair_receiver<'a>() -> &'a Mutex<mpsc::Receiver<KeyPair>> {
    KEYPAIR_RX.get_or_init(|| Mutex::new(spawn_keypair_worker(BUFSIZE)))
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
