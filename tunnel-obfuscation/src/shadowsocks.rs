//! Shadowsocks obfuscation

use crate::{socket::create_remote_socket, transport::ObfuscatedTransport};

use async_trait::async_trait;
use shadowsocks::{
    ProxySocket,
    config::{ServerConfig, ServerConfigError, ServerType},
    context::Context,
    crypto::CipherKind,
    relay::{Address, udprelay::proxy_socket::UdpSocketType},
};
use std::{io, net::SocketAddr, sync::Arc};
use talpid_net::bypass::{BypassSocket, SocketBypass};
use tokio::net::UdpSocket;

const SHADOWSOCKS_CIPHER: CipherKind = CipherKind::AES_256_GCM;
const SHADOWSOCKS_PASSWORD: &str = "mullvad";

type Result<T> = std::result::Result<T, Error>;

type ShadowSocket = BypassSocket<ProxySocket<shadowsocks::net::UdpSocket>>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Server config error
    #[error("Server config error")]
    ServerConfig(#[from] ServerConfigError),
}

pub struct Shadowsocks {
    socket: ShadowSocket,
    shadowsocks_endpoint: SocketAddr,
    wireguard_endpoint: Address,
    wireguard_addr: SocketAddr,
    packet_overhead: u16,
}

#[derive(Debug, Clone)]
pub struct Settings {
    /// Remote Shadowsocks endpoint
    pub shadowsocks_endpoint: SocketAddr,
    /// Remote WireGuard endpoint
    pub wireguard_endpoint: SocketAddr,
}

impl Settings {
    /// The overhead (in bytes) that this obfuscation protocol adds to every packet.
    pub fn packet_overhead(&self) -> u16 {
        // This math relies on the packet structure of Shadowsocks AEAD UDP packets.
        // https://shadowsocks.org/doc/aead.html
        // Those packets look like this: [salt][address][payload][tag]
        debug_assert!(SHADOWSOCKS_CIPHER.is_aead());

        let overhead = SHADOWSOCKS_CIPHER.salt_len()
            + Address::SocketAddress(self.wireguard_endpoint).serialized_len()
            + SHADOWSOCKS_CIPHER.tag_len();

        u16::try_from(overhead).expect("packet overhead is less than u16::MAX")
    }
}

impl Shadowsocks {
    pub async fn new(bypass: Arc<dyn SocketBypass>, settings: &Settings) -> crate::Result<Self> {
        let remote_socket =
            create_remote_socket(&bypass, settings.shadowsocks_endpoint.is_ipv4()).await?;

        let socket = wrap_shadowsocks(remote_socket, settings.shadowsocks_endpoint)
            .map_err(crate::Error::CreateShadowsocksObfuscator)?;

        Ok(Shadowsocks {
            socket,
            shadowsocks_endpoint: settings.shadowsocks_endpoint,
            wireguard_endpoint: Address::SocketAddress(settings.wireguard_endpoint),
            wireguard_addr: settings.wireguard_endpoint,
            packet_overhead: settings.packet_overhead(),
        })
    }
}

/// Wrap the remote socket in a Shadowsocks [ProxySocket].
///
/// This performs no I/O; the socket has already been excluded from tunnel traffic by
/// [create_remote_socket].
fn wrap_shadowsocks(
    remote_socket: BypassSocket<UdpSocket>,
    shadowsocks_endpoint: SocketAddr,
) -> Result<ShadowSocket> {
    let ss_context = Context::new_shared(ServerType::Local);
    let ss_config = ServerConfig::new(
        shadowsocks_endpoint,
        SHADOWSOCKS_PASSWORD,
        SHADOWSOCKS_CIPHER,
    )?;
    let guard = remote_socket.guard;
    let socket = ProxySocket::from_socket(
        UdpSocketType::Client,
        ss_context,
        &ss_config,
        // wrap the tokio socket
        shadowsocks::net::UdpSocket::from(remote_socket.socket),
    );
    Ok(BypassSocket { socket, guard })
}

#[async_trait]
impl ObfuscatedTransport for Shadowsocks {
    async fn send(&self, packet: &mut [u8]) -> io::Result<()> {
        self.socket
            .send_to(self.shadowsocks_endpoint, &self.wireguard_endpoint, packet)
            .await
            .map_err(|error| {
                log::trace!("Failed to write to Shadowsocks client: {error}");
                io::Error::other(error)
            })
            .map(drop)
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let (n, rx_addr, addr, _ctrl) =
                self.socket.recv_from(buf).await.map_err(io::Error::other)?;

            if rx_addr != self.shadowsocks_endpoint {
                log::trace!("Ignoring packet from unexpected Shadowsocks server: {rx_addr}");
                continue;
            }
            if addr != self.wireguard_endpoint {
                log::trace!("Ignoring packet from unexpected source: {addr}");
                continue;
            }

            return Ok(n);
        }
    }

    fn endpoint(&self) -> SocketAddr {
        self.wireguard_addr
    }

    fn packet_overhead(&self) -> u16 {
        self.packet_overhead
    }
}
