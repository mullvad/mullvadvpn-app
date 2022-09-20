use std::{fmt, net::IpAddr};
use talpid_types::net::wireguard::{PresharedKey, PrivateKey, PublicKey};
use tonic::transport::Channel;

mod kem;

#[allow(clippy::derive_partial_eq_without_eq)]
mod proto {
    tonic::include_proto!("tunnel_config");
}

#[derive(Debug)]
pub enum Error {
    GrpcConnectError(tonic::transport::Error),
    GrpcError(tonic::Status),
    InvalidCiphertextLength(usize),
    InvalidCiphertextCount(usize),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            GrpcConnectError(_) => "Failed to connect to config service".fmt(f),
            GrpcError(status) => write!(f, "RPC failed: {}", status),
            InvalidCiphertextLength(len) => write!(
                f,
                "Expected a ciphertext of length {}, got {len} bytes",
                kem::CRYPTO_CIPHERTEXTBYTES
            ),
            InvalidCiphertextCount(len) => {
                write!(f, "Expected 1 ciphertext in the response, got {len}")
            }
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

/// Port used by the tunnel config service.
pub const CONFIG_SERVICE_PORT: u16 = 1337;

/// Use the smallest CME variant with NIST security level 3. This variant has significantly smaller
/// keys than the larger variants, and is considered safe.
const ALGORITHM_NAME: &str = "Classic-McEliece-460896f";

/// Generates a new WireGuard key pair and negotiates a PSK with the relay in a PQ-safe
/// manner. This creates a peer on the relay with the new WireGuard pubkey and PSK,
/// which can then be used to establish a PQ-safe tunnel to the relay.
// TODO: consider binding to the tunnel interface here, on non-windows platforms
pub async fn push_pq_key(
    service_address: IpAddr,
    wg_pubkey: PublicKey,
) -> Result<(PrivateKey, PresharedKey), Error> {
    let wg_psk_privkey = PrivateKey::new_from_random();
    let (kem_pubkey, kem_secret) = kem::generate_keys().await;

    let mut client = new_client(service_address).await?;
    let response = client
        .psk_exchange_experimental_v1(proto::PskRequestExperimentalV1 {
            wg_pubkey: wg_pubkey.as_bytes().to_vec(),
            wg_psk_pubkey: wg_psk_privkey.public_key().as_bytes().to_vec(),
            kem_pubkeys: vec![proto::KemPubkeyExperimentalV1 {
                algorithm_name: ALGORITHM_NAME.to_string(),
                key_data: kem_pubkey.as_array().to_vec(),
            }],
        })
        .await
        .map_err(Error::GrpcError)?;

    let ciphertexts = response.into_inner().ciphertexts;
    if ciphertexts.len() != 1 {
        return Err(Error::InvalidCiphertextCount(ciphertexts.len()));
    }
    let cme_ciphertext = ciphertexts[0].as_slice();
    let ciphertext_array = <[u8; kem::CRYPTO_CIPHERTEXTBYTES]>::try_from(cme_ciphertext)
        .map_err(|_| Error::InvalidCiphertextLength(cme_ciphertext.len()))?;
    let ciphertext = kem::Ciphertext::from(ciphertext_array);
    Ok((wg_psk_privkey, kem::decapsulate(&kem_secret, &ciphertext)))
}

async fn new_client(addr: IpAddr) -> Result<RelayConfigService, Error> {
    RelayConfigService::connect(format!("tcp://{addr}:{CONFIG_SERVICE_PORT}"))
        .await
        .map_err(Error::GrpcConnectError)
}
