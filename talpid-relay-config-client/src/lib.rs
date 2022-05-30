use std::net::IpAddr;

use classic_mceliece_rust::{
    crypto_kem_dec, crypto_kem_keypair, AesState, RNGState, CRYPTO_BYTES, CRYPTO_CIPHERTEXTBYTES,
    CRYPTO_PUBLICKEYBYTES, CRYPTO_SECRETKEYBYTES,
};
use rand::RngCore;
use talpid_types::net::wireguard::{PresharedKey, PrivateKey, PublicKey};
use tonic::transport::{Channel, Endpoint, Uri};

mod types {
    tonic::include_proto!("feature");
}

type RelayConfigService = types::post_quantum_secure_client::PostQuantumSecureClient<Channel>;

const CONFIG_SERVICE_PORT: u16 = 1337;
const STACK_SIZE: usize = 8 * 1024 * 1024;
const ALGORITHM_NAME: &str = "Classic-McEliece-8192128f";

#[derive(Debug)]
pub enum Error {
    GrpcTransportError(tonic::transport::Error),
    GrpcError(tonic::Status),
    KeyGenerationFailed,
    DecapsulationError,
    InvalidCiphertext,
}

// TODO: consider binding to the tunnel interface here, on non-windows platforms
pub async fn push_pq_key(
    service_address: IpAddr,
    current_pubkey: PublicKey,
) -> Result<(PrivateKey, PresharedKey), Error> {
    let oqs_key = PrivateKey::new_from_random();

    let (pubkey, secret) = generate_key().await?;

    let mut client = new_client(service_address).await?;
    let response = client
        .psk_exchange(types::PskRequest {
            wg_pubkey: current_pubkey.as_bytes().to_vec(),
            wg_psk_pubkey: oqs_key.public_key().as_bytes().to_vec(),
            oqs_pubkey: Some(types::OqsPubkey {
                algorithm_name: ALGORITHM_NAME.to_string(),
                key_data: pubkey.to_vec(),
            }),
        })
        .await
        .map_err(Error::GrpcError)?;

    let ciphertext = response.into_inner().ciphertext;
    let ct: [u8; CRYPTO_CIPHERTEXTBYTES] = ciphertext
        .try_into()
        .map_err(|_| Error::InvalidCiphertext)?;
    let mut psk = [0u8; CRYPTO_BYTES];

    crypto_kem_dec(&mut psk, &ct, &secret).map_err(|error| {
        log::error!("KEM decapsulation failed: {error}");
        Error::DecapsulationError
    })?;
    Ok((oqs_key, PresharedKey::from(psk)))
}

async fn generate_key() -> Result<
    (
        Box<[u8; CRYPTO_PUBLICKEYBYTES]>,
        Box<[u8; CRYPTO_SECRETKEYBYTES]>,
    ),
    Error,
> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let gen_key = move || {
        let mut rng = AesState::new();

        let mut entropy = [0u8; 48];
        rand::thread_rng().fill_bytes(&mut entropy);
        rng.randombytes_init(entropy);

        let mut pubkey = Box::new([0u8; CRYPTO_PUBLICKEYBYTES]);
        let mut secret = Box::new([0u8; CRYPTO_SECRETKEYBYTES]);
        crypto_kem_keypair(&mut pubkey, &mut secret, &mut rng).map_err(|error| {
            log::error!("KEM keypair generation failed: {error}");
            Error::KeyGenerationFailed
        })?;

        Ok((pubkey, secret))
    };

    std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            tx.send(gen_key()).unwrap();
        })
        .unwrap();

    rx.await.unwrap()
}

async fn new_client(addr: IpAddr) -> Result<RelayConfigService, Error> {
    let channel = Endpoint::from_shared(format!("tcp://{addr}:{CONFIG_SERVICE_PORT}"))
        .expect("Failed to construct URI")
        .connect_with_connector(tower::service_fn(move |_: Uri| {
            tokio::net::TcpStream::connect((addr, CONFIG_SERVICE_PORT))
        }))
        .await
        .map_err(Error::GrpcTransportError)?;

    Ok(RelayConfigService::new(channel))
}
