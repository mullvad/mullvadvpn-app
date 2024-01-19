use std::{
    fmt,
    net::{IpAddr, SocketAddr},
};
use talpid_types::net::wireguard::{PresharedKey, PublicKey};
use tokio::net::TcpSocket;
use tonic::transport::{Channel, Endpoint};
use tower::service_fn;
use zeroize::Zeroize;

mod classic_mceliece;
mod kyber;

#[allow(clippy::derive_partial_eq_without_eq)]
mod proto {
    tonic::include_proto!("tunnel_config");
}

use libc::setsockopt;

#[cfg(not(target_os = "windows"))]
mod sys {
    pub use libc::{socklen_t, IPPROTO_TCP, TCP_MAXSEG};
    pub use std::os::fd::{AsRawFd, RawFd};
}
#[cfg(target_os = "windows")]
mod sys {
    pub use std::os::windows::io::{AsRawSocket, RawSocket};
    pub use windows_sys::Win32::Networking::WinSock::{IPPROTO_IP, IP_USER_MTU};
}
use sys::*;

#[derive(Debug)]
pub enum Error {
    GrpcConnectError(tonic::transport::Error),
    GrpcError(tonic::Status),
    InvalidCiphertextLength {
        algorithm: &'static str,
        actual: usize,
        expected: usize,
    },
    InvalidCiphertextCount {
        actual: usize,
    },
    FailedDecapsulateKyber(kyber::KyberError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            GrpcConnectError(_) => "Failed to connect to config service".fmt(f),
            GrpcError(status) => write!(f, "RPC failed: {status}"),
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

type RelayConfigService = proto::post_quantum_secure_client::PostQuantumSecureClient<Channel>;

/// Port used by the tunnel config service.
pub const CONFIG_SERVICE_PORT: u16 = 1337;

/// MTU to set on the tunnel config client socket. We want a low value to prevent fragmentation.
/// This is needed for two reasons:
/// 1. Especially on Android, we've found that the default MTU is often lower than the real MTU, and
///    we cannot lower it further. This causes the outer packets to be dropped. Also, MTU detection
///    will likely occur after the PQ handshake, so we cannot assume that the MTU is already
///    correctly configured.
/// 2. MH + PQ on macOS has connection issues during the handshake due to PF blocking packet
///    fragments for not having a port. In the longer term this might be fixed by allowing the
///    handshake to work even if there is fragmentation.
const CONFIG_CLIENT_MTU: u16 = 576;

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
    let kyber_keypair = kyber::keypair(&mut rand::thread_rng());

    let mut client = new_client(service_address).await?;
    let response = client
        .psk_exchange_v1(proto::PskRequestV1 {
            wg_pubkey: wg_pubkey.as_bytes().to_vec(),
            wg_psk_pubkey: wg_psk_pubkey.as_bytes().to_vec(),
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
        })
        .await
        .map_err(Error::GrpcError)?;

    let ciphertexts = response.into_inner().ciphertexts;

    // Unpack the ciphertexts into one per KEM without needing to access them by index.
    let [cme_ciphertext, kyber_ciphertext] = <&[Vec<u8>; 2]>::try_from(ciphertexts.as_slice())
        .map_err(|_| Error::InvalidCiphertextCount {
            actual: ciphertexts.len(),
        })?;

    // Store the PSK data on the heap. So it can be passed around and then zeroized on drop without
    // being stored in a bunch of places on the stack.
    let mut psk_data = Box::new([0u8; 32]);

    // Decapsulate Classic McEliece and mix into PSK
    {
        let mut shared_secret = classic_mceliece::decapsulate(&cme_kem_secret, cme_ciphertext)?;
        xor_assign(&mut psk_data, shared_secret.as_array());

        // This should happen automatically due to `SharedSecret` implementing ZeroizeOnDrop. But
        // doing it explicitly provides a stronger guarantee that it's not accidentally
        // removed.
        shared_secret.zeroize();
    }
    // Decapsulate Kyber and mix into PSK
    {
        let mut shared_secret = kyber::decapsulate(kyber_keypair.secret, kyber_ciphertext)?;
        xor_assign(&mut psk_data, &shared_secret);

        // The shared secret is sadly stored in an array on the stack. So we can't get any
        // guarantees that it's not copied around on the stack. The best we can do here
        // is to zero out the version we have and hope the compiler optimizes out copies.
        // https://github.com/Argyle-Software/kyber/issues/59
        shared_secret.zeroize();
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
    let endpoint = Endpoint::from_static("tcp://0.0.0.0:0");

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

#[cfg(not(windows))]
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
