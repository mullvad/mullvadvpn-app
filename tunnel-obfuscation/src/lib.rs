use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::io;

pub mod lwo;
pub mod multiplexer;
pub mod quic;
pub mod shadowsocks;
pub mod socket;
pub mod udp2tcp;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create Udp2Tcp obfuscator")]
    CreateUdp2TcpObfuscator(#[source] udp2tcp::Error),

    #[error("Failed to run Udp2Tcp obfuscator")]
    RunUdp2TcpObfuscator(#[source] udp2tcp::Error),

    #[error("Failed to initialize Shadowsocks")]
    CreateShadowsocksObfuscator(#[source] shadowsocks::Error),

    #[error("Failed to run Shadowsocks")]
    RunShadowsocksObfuscator(#[source] shadowsocks::Error),

    #[error("Failed to initialize Quic")]
    CreateQuicObfuscator(#[source] quic::Error),

    #[error("Failed to run Quic")]
    RunQuicObfuscator(#[source] quic::Error),

    #[error("Failed to initialize LWO")]
    CreateLwoObfuscator(#[source] lwo::Error),

    #[error("Failed to run LWO")]
    RunLwoObfuscator(#[source] lwo::Error),

    #[error("Failed to bind socket")]
    BindRemoteUdp(#[source] io::Error),

    #[cfg(target_os = "linux")]
    #[error("Failed to set fwmark on remote socket")]
    SetFwmark(#[source] nix::Error),

    #[error("Failed to initialize multiplexer")]
    CreateMultiplexerObfuscator(#[source] io::Error),

    #[error("Failed to run multiplexer")]
    RunMultiplexerObfuscator(#[source] io::Error),
}

#[async_trait]
pub trait Obfuscator: Send {
    /// NOTE(Android): Make sure to call bypass on the obfuscator socket _before_ invoking run.
    async fn run(self: Box<Self>) -> Result<()>;

    /// Returns the address of the local socket.
    fn endpoint(&self) -> SocketAddr;

    /// Returns the file descriptor of the outbound socket.
    #[cfg(target_os = "android")]
    fn remote_socket_fd(&self) -> std::os::unix::io::RawFd;

    /// The overhead (in bytes) of this obfuscation protocol.
    ///
    /// This is used when deciding on MTUs.
    fn packet_overhead(&self) -> u16;
}

#[derive(Debug, Clone)]
pub enum Settings {
    Udp2Tcp(udp2tcp::Settings),
    Shadowsocks(shadowsocks::Settings),
    Quic(quic::Settings),
    Lwo(lwo::Settings),
    Multiplexer(multiplexer::Settings),
}

pub async fn create_obfuscator(settings: &Settings) -> Result<Box<dyn Obfuscator>> {
    match settings {
        Settings::Udp2Tcp(s) => udp2tcp::Udp2Tcp::new(s)
            .await
            .map(box_obfuscator)
            .map_err(Error::CreateUdp2TcpObfuscator),
        Settings::Shadowsocks(s) => shadowsocks::Shadowsocks::new(s).await.map(box_obfuscator),
        Settings::Quic(s) => quic::Quic::new(s).await.map(box_obfuscator),
        Settings::Lwo(s) => lwo::Lwo::new(s).await.map(box_obfuscator),
        Settings::Multiplexer(s) => multiplexer::Multiplexer::new(s).await.map(box_obfuscator),
    }
}

fn box_obfuscator(obfs: impl Obfuscator + 'static) -> Box<dyn Obfuscator> {
    Box::new(obfs) as Box<dyn Obfuscator>
}
