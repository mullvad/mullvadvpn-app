use std::net::IpAddr;

use oqs::kem::{self, Algorithm, Kem, SecretKey};
use talpid_types::net::wireguard::{PresharedKey, PrivateKey, PublicKey};
use tonic::transport::{Channel, Endpoint, Uri};

mod types {
    tonic::include_proto!("feature");
}

type RelayConfigService = types::post_quantum_secure_client::PostQuantumSecureClient<Channel>;

const CONFIG_SERVICE_PORT: u16 = 1337;
const ALGORITHM: Algorithm = Algorithm::ClassicMcEliece8192128f;
const STACK_SIZE: usize = 8 * 1024 * 1024;

#[derive(Debug)]
pub enum Error {
    GrpcTransportError(tonic::transport::Error),
    GrpcError(tonic::Status),
    OqsError(oqs::Error),
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
                algorithm_name: algorithm_to_string(&ALGORITHM),
                key_data: pubkey.into_vec(),
            }),
        })
        .await
        .map_err(Error::GrpcError)?;

    let ciphertext = response.into_inner().ciphertext;
    let kem = Kem::new(ALGORITHM).map_err(Error::OqsError)?;
    let ciphertext = kem
        .ciphertext_from_bytes(&ciphertext)
        .ok_or(Error::InvalidCiphertext)?;
    let psk = kem
        .decapsulate(&secret, ciphertext)
        .map(|key| PresharedKey::from(<[u8; 32]>::try_from(key.as_ref()).unwrap()))
        .map_err(Error::OqsError)?;
    Ok((oqs_key, psk))
}

#[cfg(target_os = "windows")]
async fn generate_key() -> Result<(kem::PublicKey, SecretKey), Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let gen_key = move || {
        let kem = Kem::new(ALGORITHM).map_err(Error::OqsError)?;
        let (pubkey, secret) = kem.keypair().map_err(Error::OqsError)?;
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

#[cfg(not(target_os = "windows"))]
async fn generate_key() -> Result<(kem::PublicKey, SecretKey), Error> {
    let kem = Kem::new(ALGORITHM).map_err(Error::OqsError)?;
    kem.keypair().map_err(Error::OqsError)
}

fn algorithm_to_string(algorithm: &Algorithm) -> String {
    match algorithm {
        Algorithm::ClassicMcEliece8192128f => "Classic-McEliece-8192128f".to_string(),
        _ => unimplemented!(),
    }
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
