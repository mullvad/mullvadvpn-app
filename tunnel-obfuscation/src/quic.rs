//! Quic obfuscation

use async_trait::async_trait;
use mullvad_masque_proxy::client::{Client, ClientConfig};
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::net::UdpSocket;

use crate::Obfuscator;

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to bind UDP socket")]
    BindError(#[source] io::Error),
    #[error("Masque proxy error")]
    MasqueProxyError(#[source] mullvad_masque_proxy::client::Error),
}

pub struct Quic {
    local_endpoint: SocketAddr,
    task: tokio::task::JoinHandle<Result<()>>,
}

#[derive(Debug)]
pub struct Settings {
    /// Remote Quic endpoint
    pub quic_endpoint: SocketAddr,
    /// Remote Wireguard endpoint
    pub wireguard_endpoint: SocketAddr,
    /// Hostname to use for QUIC
    pub hostname: String,
    /// fwmark to apply to use for the QUIC connection
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
}

impl Quic {
    pub(crate) async fn new(settings: &Settings) -> Result<Self> {
        let local_socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .await
            .map_err(Error::BindError)?;

        let local_endpoint = local_socket.local_addr().unwrap();

        let config_builder = ClientConfig::builder()
            .client_socket(local_socket)
            .local_addr((Ipv4Addr::UNSPECIFIED, 0).into())
            .server_addr(settings.quic_endpoint)
            .server_host(settings.hostname.clone())
            .target_addr(settings.wireguard_endpoint);

        #[cfg(target_os = "linux")]
        let config_builder = config_builder.fwmark(settings.fwmark);

        let task = tokio::spawn(async move {
            let client = Client::connect(config_builder.build())
                .await
                .map_err(Error::MasqueProxyError)?;
            client.run().await.map_err(Error::MasqueProxyError)
        });

        Ok(Quic {
            local_endpoint,
            task,
        })
    }
}

#[async_trait]
impl Obfuscator for Quic {
    fn endpoint(&self) -> SocketAddr {
        self.local_endpoint
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        self.task
            .await
            .unwrap()
            .map_err(crate::Error::RunQuicObfuscator)
    }

    fn packet_overhead(&self) -> u16 {
        0 // FIXME
    }

    #[cfg(target_os = "android")]
    fn remote_socket_fd(&self) -> std::os::unix::io::RawFd {
        unimplemented!()
    }
}
