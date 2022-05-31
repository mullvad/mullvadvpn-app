use super::{kem, Error};
use std::net::IpAddr;
use talpid_types::net::wireguard::{PresharedKey, PrivateKey, PublicKey};
use tonic::transport::{Channel, Endpoint, Uri};

mod types {
    tonic::include_proto!("feature");
}

type RelayConfigService = types::post_quantum_secure_client::PostQuantumSecureClient<Channel>;

const CONFIG_SERVICE_PORT: u16 = 1337;
const ALGORITHM_NAME: &str = "Classic-McEliece-8192128f";

// TODO: consider binding to the tunnel interface here, on non-windows platforms
pub async fn push_pq_key(
    service_address: IpAddr,
    current_pubkey: PublicKey,
) -> Result<(PrivateKey, PresharedKey), Error> {
    let oqs_key = PrivateKey::new_from_random();
    let (pubkey, secret) = kem::generate_keys().await?;

    let mut client = new_client(service_address).await?;
    let response = client
        .psk_exchange(types::PskRequest {
            wg_pubkey: current_pubkey.as_bytes().to_vec(),
            wg_psk_pubkey: oqs_key.public_key().as_bytes().to_vec(),
            oqs_pubkey: Some(types::OqsPubkey {
                algorithm_name: ALGORITHM_NAME.to_string(),
                key_data: pubkey.into_vec(),
            }),
        })
        .await
        .map_err(Error::GrpcError)?;

    Ok((
        oqs_key,
        kem::decapsulate(&secret, &response.into_inner().ciphertext)?,
    ))
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
