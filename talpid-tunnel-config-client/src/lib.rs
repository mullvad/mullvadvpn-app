use std::{fmt, net::IpAddr};
use talpid_types::net::wireguard::{PresharedKey, PublicKey};
use tonic::transport::Channel;

mod classic_mceliece;

#[allow(clippy::derive_partial_eq_without_eq)]
mod proto {
    tonic::include_proto!("tunnel_config");
}

#[derive(Debug)]
pub enum Error {
    GrpcConnectError(tonic::transport::Error),
    GrpcError(tonic::Status),
    InvalidCiphertextLength { actual: usize, expected: usize },
    InvalidCiphertextCount { actual: usize },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            GrpcConnectError(_) => "Failed to connect to config service".fmt(f),
            GrpcError(status) => write!(f, "RPC failed: {}", status),
            InvalidCiphertextLength { actual, expected } => write!(
                f,
                "Expected a ciphertext of length {expected}, got {actual} bytes"
            ),
            InvalidCiphertextCount { actual } => {
                write!(f, "Expected 1 ciphertext in the response, got {actual}")
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
const CLASSIC_MCELIECE_VARIANT: &str = "Classic-McEliece-460896f";

/// Generates a new WireGuard key pair and negotiates a PSK with the relay in a PQ-safe
/// manner. This creates a peer on the relay with the new WireGuard pubkey and PSK,
/// which can then be used to establish a PQ-safe tunnel to the relay.
// TODO: consider binding to the tunnel interface here, on non-windows platforms
pub async fn push_pq_key(
    service_address: IpAddr,
    wg_pubkey: PublicKey,
    wg_psk_pubkey: PublicKey,
) -> Result<PresharedKey, Error> {
    let (cme_kem_pubkey, cme_kem_secret) = classic_mceliece::generate_keys().await;

    let client = new_client(service_address).await;
    let mut client = client?;
    let response = client
        .psk_exchange_experimental_v1(proto::PskRequestExperimentalV1 {
            wg_pubkey: wg_pubkey.as_bytes().to_vec(),
            wg_psk_pubkey: wg_psk_pubkey.as_bytes().to_vec(),
            kem_pubkeys: vec![proto::KemPubkeyExperimentalV1 {
                algorithm_name: CLASSIC_MCELIECE_VARIANT.to_owned(),
                key_data: cme_kem_pubkey.as_array().to_vec(),
            }],
        })
        .await
        .map_err(Error::GrpcError);
    let response = response?;

    let ciphertexts = response.into_inner().ciphertexts;
    // Unpack the ciphertexts into one per KEM without needing to access them by index.
    let [cme_ciphertext] = <&[Vec<u8>; 1]>::try_from(ciphertexts.as_slice()).map_err(|_| {
        Error::InvalidCiphertextCount {
            actual: ciphertexts.len(),
        }
    })?;

    // Store the PSK data on the heap. So it can be passed around and then zeroized on drop without
    // being stored in a bunch of places on the stack.
    let mut psk_data = Box::new([0u8; 32]);
    // Decapsulate Classic McEliece and mix into PSK
    {
        let ciphertext_array =
            <[u8; classic_mceliece::CRYPTO_CIPHERTEXTBYTES]>::try_from(cme_ciphertext.as_slice())
                .map_err(|_| Error::InvalidCiphertextLength {
                actual: cme_ciphertext.len(),
                expected: classic_mceliece::CRYPTO_CIPHERTEXTBYTES,
            })?;
        let ciphertext = classic_mceliece::Ciphertext::from(ciphertext_array);
        let shared_secret = classic_mceliece::decapsulate(&cme_kem_secret, &ciphertext);
        xor_assign(&mut psk_data, shared_secret.as_array());
    }

    Ok(PresharedKey::from(psk_data))
}

/// Performs `dst = dst ^ src`.
fn xor_assign(dst: &mut [u8; 32], src: &[u8; 32]) {
    for (dst_byte, src_byte) in dst.iter_mut().zip(src.iter()) {
        *dst_byte ^= src_byte;
    }
}

async fn new_client(addr: IpAddr) -> Result<RelayConfigService, Error> {
    RelayConfigService::connect(format!("tcp://{addr}:{CONFIG_SERVICE_PORT}"))
        .await
        .map_err(Error::GrpcConnectError)
}
