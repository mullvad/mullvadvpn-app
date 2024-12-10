use proto::PostQuantumRequestV1;
use std::fmt;
#[cfg(not(target_os = "ios"))]
use std::net::SocketAddr;
#[cfg(not(target_os = "ios"))]
use std::net::{IpAddr, Ipv4Addr};
use talpid_types::net::wireguard::{PresharedKey, PublicKey};
use tonic::transport::Channel;
#[cfg(not(target_os = "ios"))]
use tonic::transport::Endpoint;
#[cfg(not(target_os = "ios"))]
use tower::service_fn;
use zeroize::Zeroize;

mod classic_mceliece;
mod ml_kem;
#[cfg(not(target_os = "ios"))]
mod socket;

#[allow(clippy::derive_partial_eq_without_eq)]
mod proto {
    tonic::include_proto!("ephemeralpeer");
}

#[derive(Debug)]
pub enum Error {
    GrpcConnectError(tonic::transport::Error),
    GrpcError(tonic::Status),
    MissingCiphertexts,
    InvalidCiphertextLength {
        algorithm: &'static str,
        actual: usize,
        expected: usize,
    },
    InvalidCiphertextCount {
        actual: usize,
    },
    /// Failed to open TCP connection
    #[cfg(target_os = "ios")]
    TcpConnectionOpen,
    #[cfg(target_os = "ios")]
    UnableToCreateRuntime,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            GrpcConnectError(_) => "Failed to connect to config service".fmt(f),
            GrpcError(status) => write!(f, "RPC failed: {status}"),
            MissingCiphertexts => write!(f, "Found no ciphertexts in response"),
            InvalidCiphertextLength {
                algorithm,
                actual,
                expected,
            } => write!(
                f,
                "Expected a {expected} bytes ciphertext for {algorithm}, got {actual} bytes"
            ),
            InvalidCiphertextCount { actual } => {
                write!(f, "Expected 2 ciphertext in the response, got {actual}")
            }
            #[cfg(target_os = "ios")]
            TcpConnectionOpen => "Failed to open TCP connection".fmt(f),
            #[cfg(target_os = "ios")]
            UnableToCreateRuntime => "Unable to create iOS PQ PSK runtime".fmt(f),
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

pub type RelayConfigService = proto::ephemeral_peer_client::EphemeralPeerClient<Channel>;

/// Port used by the tunnel config service.
pub const CONFIG_SERVICE_PORT: u16 = 1337;

pub struct EphemeralPeer {
    pub psk: Option<PresharedKey>,
}

/// Negotiate a short-lived peer with a PQ-safe PSK or with DAITA enabled.
#[cfg(not(target_os = "ios"))]
pub async fn request_ephemeral_peer(
    service_address: Ipv4Addr,
    parent_pubkey: PublicKey,
    ephemeral_pubkey: PublicKey,
    enable_post_quantum: bool,
    enable_daita: bool,
) -> Result<EphemeralPeer, Error> {
    let client = connect_relay_config_client(service_address).await?;

    request_ephemeral_peer_with(
        client,
        parent_pubkey,
        ephemeral_pubkey,
        enable_post_quantum,
        enable_daita,
    )
    .await
}

pub async fn request_ephemeral_peer_with(
    mut client: RelayConfigService,
    parent_pubkey: PublicKey,
    ephemeral_pubkey: PublicKey,
    enable_quantum_resistant: bool,
    enable_daita: bool,
) -> Result<EphemeralPeer, Error> {
    let (pq_request, kem_secrets) = if enable_quantum_resistant {
        let (pq_request, kem_secrets) = post_quantum_secrets().await;
        (Some(pq_request), Some(kem_secrets))
    } else {
        (None, None)
    };

    let response = client
        .register_peer_v1(proto::EphemeralPeerRequestV1 {
            wg_parent_pubkey: parent_pubkey.as_bytes().to_vec(),
            wg_ephemeral_peer_pubkey: ephemeral_pubkey.as_bytes().to_vec(),
            post_quantum: pq_request,
            daita: Some(proto::DaitaRequestV1 {
                activate_daita: enable_daita,
            }),
        })
        .await
        .map_err(Error::GrpcError)?;

    let psk = if let Some((cme_kem_secret, ml_kem_secret)) = kem_secrets {
        let ciphertexts = response
            .into_inner()
            .post_quantum
            .ok_or(Error::MissingCiphertexts)?
            .ciphertexts;

        // Unpack the ciphertexts into one per KEM without needing to access them by index.
        let [cme_ciphertext, ml_kem_ciphertext] = <&[Vec<u8>; 2]>::try_from(ciphertexts.as_slice())
            .map_err(|_| Error::InvalidCiphertextCount {
                actual: ciphertexts.len(),
            })?;

        // Store the PSK data on the heap. So it can be passed around and then zeroized on drop
        // without being stored in a bunch of places on the stack.
        let mut psk_data = Box::new([0u8; 32]);

        // Decapsulate Classic McEliece and mix into PSK
        {
            let mut shared_secret = classic_mceliece::decapsulate(&cme_kem_secret, cme_ciphertext)?;
            xor_assign(&mut psk_data, shared_secret.as_array());

            // This should happen automatically due to `SharedSecret` implementing ZeroizeOnDrop.
            // But doing it explicitly provides a stronger guarantee that it's not
            // accidentally removed.
            shared_secret.zeroize();
        }
        // Decapsulate ML-KEM and mix into PSK
        {
            let mut shared_secret = ml_kem_secret.decapsulate(ml_kem_ciphertext)?;
            xor_assign(&mut psk_data, &shared_secret);

            // The shared secret is sadly stored in an array on the stack. So we can't get any
            // guarantees that it's not copied around on the stack. The best we can do here
            // is to zero out the version we have and hope the compiler optimizes out copies.
            // https://github.com/Argyle-Software/kyber/issues/59
            shared_secret.zeroize();
        }

        Some(PresharedKey::from(psk_data))
    } else {
        None
    };

    Ok(EphemeralPeer { psk })
}

async fn post_quantum_secrets() -> (
    PostQuantumRequestV1,
    (classic_mceliece_rust::SecretKey<'static>, ml_kem::Keypair),
) {
    let (cme_kem_pubkey, cme_kem_secret) = classic_mceliece::generate_keys().await;
    let ml_kem_keypair = ml_kem::keypair();

    (
        proto::PostQuantumRequestV1 {
            kem_pubkeys: vec![
                proto::KemPubkeyV1 {
                    algorithm_name: classic_mceliece::ALGORITHM_NAME.to_owned(),
                    key_data: cme_kem_pubkey.as_array().to_vec(),
                },
                proto::KemPubkeyV1 {
                    algorithm_name: ml_kem::ALGORITHM_NAME.to_owned(),
                    key_data: ml_kem_keypair.encapsulation_key(),
                },
            ],
        },
        (cme_kem_secret, ml_kem_keypair),
    )
}

/// Performs `dst = dst ^ src`.
fn xor_assign(dst: &mut [u8; 32], src: &[u8; 32]) {
    for (dst_byte, src_byte) in dst.iter_mut().zip(src.iter()) {
        *dst_byte ^= src_byte;
    }
}

/// Create a new `RelayConfigService` connected to the given IP.
/// On non-Windows platforms the connection is made with a socket where the MSS
/// value has been speficically lowered, to avoid MTU issues. See the `socket` module.
#[cfg(not(target_os = "ios"))]
async fn connect_relay_config_client(ip: Ipv4Addr) -> Result<RelayConfigService, Error> {
    use futures::TryFutureExt;

    let endpoint = Endpoint::from_static("tcp://0.0.0.0:0");
    let addr = SocketAddr::new(IpAddr::V4(ip), CONFIG_SERVICE_PORT);

    let connection = endpoint
        .connect_with_connector(service_fn(move |_| async move {
            let sock = socket::TcpSocket::new()?;
            sock.connect(addr)
                .map_ok(hyper_util::rt::tokio::TokioIo::new)
                .await
        }))
        .await
        .map_err(Error::GrpcConnectError)?;

    Ok(RelayConfigService::new(connection))
}
