use std::{fmt, net::IpAddr};
use talpid_types::net::wireguard::{PresharedKey, PrivateKey, PublicKey};
use tonic::transport::Channel;

mod cme;
mod kyber;

#[allow(clippy::derive_partial_eq_without_eq)]
mod proto {
    tonic::include_proto!("tunnel_config");
}

#[derive(Debug)]
pub enum Error {
    GrpcConnectError(tonic::transport::Error),
    GrpcError(tonic::Status),
    InvalidCiphertextLength(usize, usize),
    InvalidCiphertextCount(usize),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            GrpcConnectError(_) => "Failed to connect to config service".fmt(f),
            GrpcError(status) => write!(f, "RPC failed: {}", status),
            InvalidCiphertextLength(actual, expected) => write!(
                f,
                "Expected a ciphertext of length {expected}, got {actual} bytes"
            ),
            InvalidCiphertextCount(actual) => {
                write!(f, "Expected 2 ciphertexts in the response, got {actual}")
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
    let (cme_kem_pubkey, cme_kem_secret) = cme::generate_keys().await;
    let kyber_keypair = pqc_kyber::keypair(&mut rand::thread_rng());

    let mut client = new_client(service_address).await?;
    let response = client
        .psk_exchange_experimental_v1(proto::PskRequestExperimentalV1 {
            wg_pubkey: wg_pubkey.as_bytes().to_vec(),
            wg_psk_pubkey: wg_psk_privkey.public_key().as_bytes().to_vec(),
            kem_pubkeys: vec![
                proto::KemPubkeyExperimentalV1 {
                    algorithm_name: ALGORITHM_NAME.to_owned(),
                    key_data: cme_kem_pubkey.as_array().to_vec(),
                },
                proto::KemPubkeyExperimentalV1 {
                    algorithm_name: "Kyber1024".to_owned(),
                    key_data: kyber_keypair.public.to_vec(),
                },
            ],
        })
        .await
        .map_err(Error::GrpcError)?;

    let ciphertexts = response.into_inner().ciphertexts;
    let [cme_ciphertext, kyber_ciphertext] = <&[Vec<u8>; 2]>::try_from(ciphertexts.as_slice())
        .map_err(|_| Error::InvalidCiphertextCount(ciphertexts.len()))?;

    let mut psk_data = [0u8; 32];
    // Decapsulate Classic McEliece and mix into PSK
    {
        let ciphertext_array = <[u8; cme::CRYPTO_CIPHERTEXTBYTES]>::try_from(
            cme_ciphertext.as_slice(),
        )
        .map_err(|_| {
            Error::InvalidCiphertextLength(cme_ciphertext.len(), cme::CRYPTO_CIPHERTEXTBYTES)
        })?;
        let ciphertext = cme::Ciphertext::from(ciphertext_array);
        let shared_secret = cme::decapsulate(&cme_kem_secret, &ciphertext);
        xor_assign(&mut psk_data, shared_secret.as_array());
    }
    // Decapsulate Kyber and mix into PSK
    {
        let ciphertext_array = <[u8; kyber::CRYPTO_CIPHERTEXTBYTES]>::try_from(
            kyber_ciphertext.as_slice(),
        )
        .map_err(|_| {
            Error::InvalidCiphertextLength(kyber_ciphertext.len(), kyber::CRYPTO_CIPHERTEXTBYTES)
        })?;
        let shared_secret = kyber::decapsulate(kyber_keypair.secret, ciphertext_array);
        xor_assign(&mut psk_data, &shared_secret);
    }

    Ok((wg_psk_privkey, PresharedKey::from(psk_data)))
}

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
