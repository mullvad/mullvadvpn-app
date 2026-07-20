use async_trait::async_trait;
use std::{net::SocketAddr, sync::Arc};
use talpid_net::bypass::{NoopBypass, SocketBypass};
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

    #[error("Failed to bypass socket")]
    Bypass(#[source] io::Error),

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
    create_obfuscator_with_bypass(Arc::new(NoopBypass), settings).await
}

pub async fn create_obfuscator_with_bypass(
    bypass: Arc<dyn SocketBypass>,
    settings: &Settings,
) -> Result<Box<dyn Obfuscator>> {
    match settings {
        Settings::Udp2Tcp(s) => udp2tcp::Udp2Tcp::new(bypass, s).await.map(box_obfuscator),
        Settings::Shadowsocks(s) => shadowsocks::Shadowsocks::new(bypass, s)
            .await
            .map(box_obfuscator),
        Settings::Quic(s) => quic::Quic::new(bypass, s).await.map(box_obfuscator),
        Settings::Lwo(s) => lwo::Lwo::new(bypass, s).await.map(box_obfuscator),
        Settings::Multiplexer(s) => multiplexer::Multiplexer::new(bypass, s)
            .await
            .map(box_obfuscator),
    }
}

fn box_obfuscator(obfs: impl Obfuscator + 'static) -> Box<dyn Obfuscator> {
    Box::new(obfs) as Box<dyn Obfuscator>
}
