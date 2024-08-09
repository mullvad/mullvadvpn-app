use proto::PostQuantumRequestV1;
use std::fmt;
#[cfg(not(target_os = "ios"))]
use std::net::SocketAddr;
#[cfg(not(target_os = "ios"))]
use std::net::{IpAddr, Ipv4Addr};
use talpid_types::net::wireguard::{PresharedKey, PublicKey};
#[cfg(not(target_os = "ios"))]
use tokio::net::TcpSocket;
use tonic::transport::Channel;
#[cfg(not(target_os = "ios"))]
use tonic::transport::Endpoint;
#[cfg(not(target_os = "ios"))]
use tower::service_fn;
use zeroize::Zeroize;

mod classic_mceliece;
mod kyber;

#[allow(clippy::derive_partial_eq_without_eq)]
mod proto {
    tonic::include_proto!("ephemeralpeer");
}

#[cfg(not(target_os = "ios"))]
use libc::setsockopt;

#[cfg(not(any(target_os = "windows", target_os = "ios")))]
mod sys {
    pub use libc::{socklen_t, IPPROTO_TCP, TCP_MAXSEG};
    pub use std::os::fd::{AsRawFd, RawFd};
}

#[cfg(target_os = "windows")]
mod sys {
    pub use std::os::windows::io::{AsRawSocket, RawSocket};
    pub use windows_sys::Win32::Networking::WinSock::{IPPROTO_IP, IP_USER_MTU};
}
#[cfg(not(target_os = "ios"))]
use sys::*;

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
    FailedDecapsulateKyber(kyber::KyberError),
    #[cfg(target_os = "ios")]
    TcpConnectionExpired,
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
            FailedDecapsulateKyber(_) => "Failed to decapsulate Kyber1024 ciphertext".fmt(f),
            #[cfg(target_os = "ios")]
            TcpConnectionExpired => "TCP connection is already shut down".fmt(f),
            #[cfg(target_os = "ios")]
            UnableToCreateRuntime => "Unable to create iOS PQ PSK runtime".fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GrpcConnectError(error) => Some(error),
            Self::FailedDecapsulateKyber(error) => Some(error),
            _ => None,
        }
    }
}

pub type RelayConfigService = proto::ephemeral_peer_client::EphemeralPeerClient<Channel>;

/// Port used by the tunnel config service.
pub const CONFIG_SERVICE_PORT: u16 = 1337;

/// MTU to set on the tunnel config client socket. We want a low value to prevent fragmentation.
/// This is needed for two reasons:
/// 1. Especially on Android, we've found that the real MTU is often lower than the default MTU, and
///    we cannot lower it further. This causes the outer packets to be dropped. Also, MTU detection
///    will likely occur after the PQ handshake, so we cannot assume that the MTU is already
///    correctly configured.
/// 2. MH + PQ on macOS has connection issues during the handshake due to PF blocking packet
///    fragments for not having a port. In the longer term this might be fixed by allowing the
///    handshake to work even if there is fragmentation.
#[cfg(not(target_os = "ios"))]
const CONFIG_CLIENT_MTU: u16 = 576;

pub struct EphemeralPeer {
    pub psk: Option<PresharedKey>,
}

pub async fn request_ephemeral_peer_with(
    mut client: RelayConfigService,
    parent_pubkey: PublicKey,
    ephemeral_pubkey: PublicKey,
    enable_post_quantum: bool,
    enable_daita: bool,
) -> Result<EphemeralPeer, Error> {
    let (pq_request, kem_secrets) = post_quantum_secrets(enable_post_quantum).await;
    let daita = Some(proto::DaitaRequestV1 {
        activate_daita: enable_daita,
    });

    let response = client
        .register_peer_v1(proto::EphemeralPeerRequestV1 {
            wg_parent_pubkey: parent_pubkey.as_bytes().to_vec(),
            wg_ephemeral_peer_pubkey: ephemeral_pubkey.as_bytes().to_vec(),
            post_quantum: pq_request,
            daita,
        })
        .await
        .map_err(Error::GrpcError)?;

    let psk = if let Some((cme_kem_secret, kyber_secret)) = kem_secrets {
        let ciphertexts = response
            .into_inner()
            .post_quantum
            .ok_or(Error::MissingCiphertexts)?
            .ciphertexts;

        // Unpack the ciphertexts into one per KEM without needing to access them by index.
        let [cme_ciphertext, kyber_ciphertext] = <&[Vec<u8>; 2]>::try_from(ciphertexts.as_slice())
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
        // Decapsulate Kyber and mix into PSK
        {
            let mut shared_secret = kyber::decapsulate(kyber_secret, kyber_ciphertext)?;
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

/// Negotiate a short-lived peer with a PQ-safe PSK or with DAITA enabled.
#[cfg(not(target_os = "ios"))]
pub async fn request_ephemeral_peer(
    service_address: Ipv4Addr,
    parent_pubkey: PublicKey,
    ephemeral_pubkey: PublicKey,
    enable_post_quantum: bool,
    enable_daita: bool,
) -> Result<EphemeralPeer, Error> {
    let client = new_client(service_address).await?;

    request_ephemeral_peer_with(
        client,
        parent_pubkey,
        ephemeral_pubkey,
        enable_post_quantum,
        enable_daita,
    )
    .await
}

async fn post_quantum_secrets(
    enable_post_quantum: bool,
) -> (
    Option<PostQuantumRequestV1>,
    Option<(classic_mceliece_rust::SecretKey<'static>, [u8; 3168])>,
) {
    if enable_post_quantum {
        let (cme_kem_pubkey, cme_kem_secret) = classic_mceliece::generate_keys().await;
        let kyber_keypair = kyber::keypair(&mut rand::thread_rng());

        (
            Some(proto::PostQuantumRequestV1 {
                kem_pubkeys: vec![
                    proto::KemPubkeyV1 {
                        algorithm_name: classic_mceliece::ALGORITHM_NAME.to_owned(),
                        key_data: cme_kem_pubkey.as_array().to_vec(),
                    },
                    proto::KemPubkeyV1 {
                        algorithm_name: kyber::ALGORITHM_NAME.to_owned(),
                        key_data: kyber_keypair.public.to_vec(),
                    },
                ],
            }),
            Some((cme_kem_secret, kyber_keypair.secret)),
        )
    } else {
        (None, None)
    }
}

/// Performs `dst = dst ^ src`.
fn xor_assign(dst: &mut [u8; 32], src: &[u8; 32]) {
    for (dst_byte, src_byte) in dst.iter_mut().zip(src.iter()) {
        *dst_byte ^= src_byte;
    }
}

#[cfg(not(target_os = "ios"))]
async fn new_client(addr: Ipv4Addr) -> Result<RelayConfigService, Error> {
    let endpoint = Endpoint::from_static("tcp://0.0.0.0:0");
    let addr = IpAddr::V4(addr);

    let conn = endpoint
        .connect_with_connector(service_fn(move |_| async move {
            let sock = TcpSocket::new_v4()?;

            #[cfg(target_os = "windows")]
            try_set_tcp_sock_mtu(sock.as_raw_socket(), CONFIG_CLIENT_MTU);

            #[cfg(not(target_os = "windows"))]
            try_set_tcp_sock_mtu(&addr, sock.as_raw_fd(), CONFIG_CLIENT_MTU);

            sock.connect(SocketAddr::new(addr, CONFIG_SERVICE_PORT))
                .await
        }))
        .await
        .map_err(Error::GrpcConnectError)?;

    Ok(RelayConfigService::new(conn))
}

#[cfg(windows)]
fn try_set_tcp_sock_mtu(sock: RawSocket, mtu: u16) {
    let mtu = u32::from(mtu);
    log::debug!("Config client socket MTU: {mtu}");

    let raw_sock = usize::try_from(sock).unwrap();

    let result = unsafe {
        setsockopt(
            raw_sock,
            IPPROTO_IP,
            IP_USER_MTU,
            &mtu as *const _ as _,
            std::ffi::c_int::try_from(std::mem::size_of_val(&mtu)).unwrap(),
        )
    };
    if result != 0 {
        log::error!(
            "Failed to set user MTU on config client socket: {}",
            std::io::Error::last_os_error()
        );
    }
}

#[cfg(not(any(target_os = "windows", target_os = "ios")))]
fn try_set_tcp_sock_mtu(dest: &IpAddr, sock: RawFd, mut mtu: u16) {
    const IPV4_HEADER_SIZE: u16 = 20;
    const IPV6_HEADER_SIZE: u16 = 40;
    const MAX_TCP_HEADER_SIZE: u16 = 60;

    if dest.is_ipv4() {
        mtu = mtu.saturating_sub(IPV4_HEADER_SIZE);
    } else {
        mtu = mtu.saturating_sub(IPV6_HEADER_SIZE);
    }

    let mss = u32::from(mtu.saturating_sub(MAX_TCP_HEADER_SIZE));

    log::debug!("Config client socket MSS: {mss}");

    let result = unsafe {
        setsockopt(
            sock,
            IPPROTO_TCP,
            TCP_MAXSEG,
            &mss as *const _ as _,
            socklen_t::try_from(std::mem::size_of_val(&mss)).unwrap(),
        )
    };
    if result != 0 {
        log::error!(
            "Failed to set MSS on config client socket: {}",
            std::io::Error::last_os_error()
        );
    }
}
