use async_trait::async_trait;
use std::{net::SocketAddr, sync::Arc};
use talpid_net::bypass::{NoopBypass, SocketBypass};
use tokio::io;

pub mod direct;
pub mod local_socket;
pub mod lwo;
pub mod multiplexer;
pub mod quic;
pub mod shadowsocks;
pub mod socket;
pub mod socks5;
pub mod transport;
pub mod udp2tcp;
pub(crate) mod wireguard;

pub use transport::ObfuscatedTransport;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create Udp2Tcp obfuscator")]
    CreateUdp2TcpObfuscator(#[source] udp2tcp::Error),

    #[error("Failed to initialize Shadowsocks")]
    CreateShadowsocksObfuscator(#[source] shadowsocks::Error),

    #[error("Failed to initialize Quic")]
    CreateQuicObfuscator(#[source] quic::Error),

    #[error("Failed to bind remote socket")]
    BindRemoteUdp(#[source] io::Error),

    #[error("Failed to bypass socket")]
    Bypass(#[source] io::Error),

    #[error("Failed to connect remote socket")]
    ConnectRemoteUdp(#[source] io::Error),

    #[error("Failed to bind local socket")]
    BindLocalUdp(#[source] io::Error),

    #[error("Failed to run local socket obfuscator")]
    RunLocalSocketObfuscator(#[source] io::Error),

    #[error("Failed to run multiplexer")]
    RunMultiplexerObfuscator(#[source] io::Error),
}

/// An obfuscator that a local WireGuard instance reaches over a UDP socket on localhost.
#[async_trait]
pub trait LocalSocketObfuscator: Send {
    async fn run(self: Box<Self>) -> Result<()>;

    /// Returns the address of the local socket.
    fn endpoint(&self) -> SocketAddr;
}

/// Settings for a single obfuscator.
///
/// The multiplexer, which races several obfuscators against each other, is deliberately not
/// included here. It is constructed directly via [`multiplexer::Multiplexer::new`], so that it
/// cannot be nested within itself.
#[derive(Debug, Clone)]
pub enum Settings {
    Udp2Tcp(udp2tcp::Settings),
    Shadowsocks(shadowsocks::Settings),
    Quic(quic::Settings),
    Lwo(lwo::Settings),
}

impl Settings {
    /// The overhead (in bytes) that this obfuscation protocol adds to every packet.
    pub fn packet_overhead(&self) -> u16 {
        match self {
            Settings::Udp2Tcp(s) => s.packet_overhead(),
            Settings::Shadowsocks(s) => s.packet_overhead(),
            Settings::Quic(s) => s.packet_overhead(),
            Settings::Lwo(s) => s.packet_overhead(),
        }
    }

    /// Whether this obfuscation transforms datagrams on a socket provided by the caller, rather
    /// than opening its own connection to the obfuscation server.
    ///
    /// Only these can be layered on top of another transport, such as a SOCKS5 proxy.
    pub fn stacks_on_provided_socket(&self) -> bool {
        match self {
            // A pure in-place transform of each datagram.
            Settings::Lwo(_) => true,
            // Connects to the obfuscation server over TCP.
            Settings::Udp2Tcp(_) => false,
            // Bind their own UDP socket in `create_transport`.
            Settings::Shadowsocks(_) | Settings::Quic(_) => false,
        }
    }
}

/// Create an [ObfuscatedTransport] that obfuscates and deobfuscates packets in place where the
/// protocol allows it.
pub async fn create_transport(
    bypass: Arc<dyn SocketBypass>,
    settings: &Settings,
) -> Result<Arc<dyn ObfuscatedTransport>> {
    match settings {
        Settings::Shadowsocks(s) => shadowsocks::Shadowsocks::new(bypass, s)
            .await
            .map(arc_transport),
        Settings::Quic(s) => quic::QuicTransport::new(bypass, s).await.map(arc_transport),
        Settings::Lwo(s) => lwo::Lwo::new(bypass, s).await.map(arc_transport),

        Settings::Udp2Tcp(s) => udp2tcp::Udp2Tcp::new(bypass, s).map(arc_transport),
    }
}

pub async fn create_local_socket_obfuscator(
    settings: &Settings,
) -> Result<Box<dyn LocalSocketObfuscator>> {
    create_local_socket_obfuscator_with_bypass(Arc::new(NoopBypass), settings).await
}

pub async fn create_local_socket_obfuscator_with_bypass(
    bypass: Arc<dyn SocketBypass>,
    settings: &Settings,
) -> Result<Box<dyn LocalSocketObfuscator>> {
    let transport = create_transport(bypass, settings).await?;
    local_socket::LocalSocketRunner::new(transport)
        .await
        .map(box_obfuscator)
}

fn arc_transport(transport: impl ObfuscatedTransport + 'static) -> Arc<dyn ObfuscatedTransport> {
    Arc::new(transport) as Arc<dyn ObfuscatedTransport>
}

fn box_obfuscator(obfs: impl LocalSocketObfuscator + 'static) -> Box<dyn LocalSocketObfuscator> {
    Box::new(obfs) as Box<dyn LocalSocketObfuscator>
}
