use std::{fmt, net::IpAddr};
use talpid_types::net::wireguard::{PresharedKey, PrivateKey, PublicKey};
use tonic::transport::Channel;

mod kem;

mod proto {
    tonic::include_proto!("tunnel_config");
}

#[derive(Debug)]
pub enum Error {
    GrpcConnectError(tonic::transport::Error),
    GrpcError(tonic::Status),
    KeyGenerationFailed,
    DecapsulationError,
    InvalidCiphertext,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            GrpcConnectError(_) => "Failed to connect to config service".fmt(f),
            GrpcError(status) => write!(f, "RPC failed: {}", status),
            KeyGenerationFailed => "Failed to generate KEM key pair".fmt(f),
            DecapsulationError => "Failed to decapsulate secret".fmt(f),
            InvalidCiphertext => "The service returned an invalid ciphertext".fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GrpcConnectError(error) => Some(error),
            _ => None,
        }
    }
}

type RelayConfigService = proto::post_quantum_secure_client::PostQuantumSecureClient<Channel>;

const CONFIG_SERVICE_PORT: u16 = 1337;
const ALGORITHM_NAME: &str = "Classic-McEliece-8192128f";

/// Generates a new WireGuard key pair and negotiates a PSK with the relay in a PQ-safe
/// manner. This creates a peer on the relay with the new WireGuard pubkey and PSK,
/// which can then be used to establish a PQ-safe tunnel to the relay.
// TODO: consider binding to the tunnel interface here, on non-windows platforms
pub async fn push_pq_key(
    service_address: IpAddr,
    wg_pubkey: PublicKey,
) -> Result<(PrivateKey, PresharedKey), Error> {
    let wg_psk_privkey = PrivateKey::new_from_random();
    let (kem_pubkey, kem_secret) = kem::generate_keys().await?;

    let mut client = new_client(service_address).await?;
    let response = client
        .psk_exchange_experimental_v0(proto::PskRequestExperimentalV0 {
            wg_pubkey: wg_pubkey.as_bytes().to_vec(),
            wg_psk_pubkey: wg_psk_privkey.public_key().as_bytes().to_vec(),
            kem_pubkey: Some(proto::KemPubkeyExperimentalV0 {
                algorithm_name: ALGORITHM_NAME.to_string(),
                key_data: kem_pubkey.into_vec(),
            }),
        })
        .await
        .map_err(Error::GrpcError)?;

    let ciphertext: [u8; kem::CRYPTO_CIPHERTEXTBYTES] = response
        .into_inner()
        .ciphertext
        .try_into()
        .map_err(|_| Error::InvalidCiphertext)?;

    Ok((wg_psk_privkey, kem::decapsulate(&kem_secret, &ciphertext)?))
}

async fn new_client(addr: IpAddr) -> Result<RelayConfigService, Error> {
    RelayConfigService::connect(format!("tcp://{addr}:{CONFIG_SERVICE_PORT}"))
        .await
        .map_err(Error::GrpcConnectError)
}
