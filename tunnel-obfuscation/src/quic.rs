//! Quic obfuscation

#[cfg(any(target_os = "android", target_os = "linux"))]
use std::os::fd::AsRawFd;
use async_trait::async_trait;
use std::{io, net::{Ipv4Addr, SocketAddr}, sync::Arc};
use tokio::{net::UdpSocket, sync::oneshot};
use mullvad_masque_proxy::client::{ClientConfig, Client};

use crate::Obfuscator;


type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Masque proxy error")]
    MasqueProxyError(#[source] mullvad_masque_proxy::client::Error)
}

pub struct Quic {
    pub local_endpoint: SocketAddr,
    client: Client
}

#[derive(Debug)]
pub struct Settings {
    /// Remote Quic endpoint
    pub quic_endpoint: SocketAddr,
    /// Remote Wireguard endpoint
    pub wireguard_endpoint: SocketAddr,

    pub hostname: String,

}

impl Quic {
    pub(crate) async fn new(settings: &Settings) -> Result<Self> {

        let local_socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .expect("Failed to bind address");

        let local_endpoint = local_socket.local_addr().unwrap();

        let config_builder = ClientConfig::builder()
            .client_socket(local_socket)
            .local_addr((Ipv4Addr::UNSPECIFIED, 0).into())
            .server_addr(settings.quic_endpoint)
            .server_host(settings.hostname.clone())
            .target_addr(settings.wireguard_endpoint);


        let client = Client::connect(config_builder.build()).await.map_err(Error::MasqueProxyError)?; 
        // TODO: defer calling connect to a separate method call

        Ok(Quic {
            local_endpoint,
            client
        })
    }


}


#[async_trait]
impl Obfuscator for Quic {
    fn endpoint(&self) -> SocketAddr {
        self.local_endpoint
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        self.client.run().await.unwrap();

        Ok(())
    }

    fn packet_overhead(&self) -> u16 {
        0 // FIXME
    }
}