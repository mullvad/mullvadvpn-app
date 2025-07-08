//! Quic obfuscation

use async_trait::async_trait;
use mullvad_masque_proxy::client::{Client, ClientConfig};
use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};
use tokio::{net::UdpSocket, sync::oneshot};

use crate::Obfuscator;

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to bind UDP socket")]
    BindError(#[source] io::Error),
    #[error("Masque proxy error")]
    MasqueProxyError(#[source] mullvad_masque_proxy::client::Error),
}

#[derive(Debug)]
pub struct Quic {
    local_endpoint: SocketAddr,
    task: tokio::task::JoinHandle<Result<()>>,
    // task will be stopped when this is dropped
    _shutdown_tx: oneshot::Sender<()>, // TODO: Cancellation token?
}

#[derive(Debug)]
pub struct Settings {
    /// Remote Quic endpoint
    pub quic_endpoint: SocketAddr,
    /// Remote Wireguard endpoint
    pub wireguard_endpoint: SocketAddr,
    /// Hostname to use for QUIC
    pub hostname: String,
    /// Authentication token to set for the CONNECT request when establishing a QUIC connection.
    /// Must NOT be prefixed with "Bearer".
    pub token: String,
    /// fwmark to apply to use for the QUIC connection
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
}

impl Settings {
    /// The masque-proxy server expects the Authentication header to be prefixed with "Bearer ", so
    /// prefix the auth token with that.
    fn auth_header(&self) -> String {
        debug_assert!(!self.token.starts_with("Bearer"));
        format!("Bearer {token}", token = self.token)
    }
}

impl Quic {
    pub(crate) async fn new(settings: &Settings) -> Result<Self> {
        let (local_socket, local_udp_client_addr) =
            Quic::create_local_udp_socket(settings.quic_endpoint.is_ipv4()).await?;
        // The address family of the local QUIC client socket has to match the address family
        // of the endpoint we're connecting to. The address itself is not important to consumers wanting
        // to obfuscate traffic. It is solely used by the local proxy client to know where the QUIC
        // obfuscator is running.
        let quic_client_local_addr = if settings.quic_endpoint.is_ipv4() {
            SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))
        } else {
            SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0))
        };
        let config_builder = ClientConfig::builder()
            .client_socket(local_socket)
            .local_addr(quic_client_local_addr)
            .server_addr(settings.quic_endpoint)
            .server_host(settings.hostname.clone())
            .target_addr(settings.wireguard_endpoint)
            .auth_header(Some(settings.auth_header()));

        #[cfg(target_os = "linux")]
        let config_builder = config_builder.fwmark(settings.fwmark);

        let client = Client::connect(config_builder.build())
            .await
            .map_err(Error::MasqueProxyError)?;

        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let local_proxy = tokio::spawn(Quic::run_forwarding(client, shutdown_rx));

        let quic = Quic {
            local_endpoint: local_udp_client_addr,
            task: local_proxy,
            _shutdown_tx: shutdown_tx,
        };

        Ok(quic)
    }

    async fn run_forwarding(
        masque_proxy_client: Client,
        shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<()> {
        log::trace!("Spawning QUIC client ..");
        let mut client = tokio::spawn(masque_proxy_client.run());
        log::trace!("QUIC client is running! QUIC Obfuscator is serving traffic 🎉");
        tokio::select! {
            _ = shutdown_rx => log::trace!("Stopping QUIC obfuscation"),
            _result = &mut client => log::trace!("QUIC client closed"),
        };

        client.abort();
        Ok(())
    }

    /// Create a local proxy client.
    ///
    /// The resulting UdpSocket/the SocketAddr where programs that want to obfuscate their
    /// traffic with QUIC will write to.
    async fn create_local_udp_socket(ipv4: bool) -> Result<(UdpSocket, SocketAddr)> {
        let random_bind_addr = if ipv4 {
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0))
        } else {
            SocketAddr::from((Ipv6Addr::LOCALHOST, 0))
        };
        let local_udp_socket = UdpSocket::bind(random_bind_addr)
            .await
            .map_err(Error::BindError)?;
        let udp_client_addr = local_udp_socket.local_addr().unwrap();

        Ok((local_udp_socket, udp_client_addr))
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
