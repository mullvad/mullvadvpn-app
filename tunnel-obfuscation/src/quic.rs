//! Quic obfuscation

use async_trait::async_trait;
use mullvad_masque_proxy::client::{Client, ClientConfig};
use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};
use tokio::net::UdpSocket;
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::Obfuscator;

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to bind UDP socket")]
    BindError(#[source] io::Error),
    #[error("Masque proxy error")]
    MasqueProxyError(#[source] mullvad_masque_proxy::client::Error),
    #[error("Failed to get local socket address")]
    LocalAddrError(#[source] io::Error),
}

#[derive(Debug)]
pub struct Quic {
    local_endpoint: SocketAddr,
    task: tokio::task::JoinHandle<Result<()>>,
    _shutdown: DropGuard,
}

#[derive(Debug)]
pub struct Settings {
    /// Remote Quic endpoint
    quic_endpoint: SocketAddr,
    /// Remote Wireguard endpoint
    wireguard_endpoint: SocketAddr,
    /// Hostname to use for QUIC
    hostname: String,
    /// Authentication token to set for the CONNECT request when establishing a QUIC connection.
    /// Must NOT be prefixed with "Bearer".
    token: AuthToken,
    /// fwmark to apply to use for the QUIC connection
    #[cfg(target_os = "linux")]
    fwmark: Option<u32>,
    /// MTU for the QUIC client. This needs to account for the *additional* headers other than IP
    /// and UDP, but not for those specifically.
    mtu: Option<u16>,
}

impl Settings {
    /// See [Settings] for details.
    pub fn new(
        quic_server_endpoint: SocketAddr,
        hostname: String,
        token: AuthToken,
        target_endpoint: SocketAddr,
    ) -> Self {
        Self {
            quic_endpoint: quic_server_endpoint,
            wireguard_endpoint: target_endpoint,
            hostname,
            token,
            mtu: None,
            #[cfg(target_os = "linux")]
            fwmark: None,
        }
    }

    /// Set an explicit MTU for the Quic obfuscator.
    pub fn mtu(self, mtu: u16) -> Self {
        debug_assert!(mtu >= 1280 && mtu <= 1500, "MTU should be between 1280 and 1500: {mtu}");
        let mtu = Some(mtu);
        Self { mtu, ..self }
    }

    /// Set `fwmark` for the Quic obfuscator.
    #[cfg(target_os = "linux")]
    pub fn fwmark(self, fwmark: u32) -> Self {
        let fwmark = Some(fwmark);
        Self { fwmark, ..self }
    }

    /// The masque-proxy server expects the Authentication header to be prefixed with "Bearer ", so
    /// prefix the auth token with that.
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token.0)
    }
}

/// Authorization Token used when connecting to a masque-proxy.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthToken(String);

impl AuthToken {
    /// Create a new token for constructing a valid Authorization header when connecting to a
    /// masque-proxy.
    pub fn new(token: String) -> Option<Self> {
        // TODO: We could potentially do more validation, but the exact format of the auth token is
        // not known to be stable (yet).
        if token.starts_with("Bearer") {
            return None;
        }
        Some(Self(token))
    }
}

impl std::str::FromStr for AuthToken {
    type Err = String;

    fn from_str(token: &str) -> std::result::Result<Self, Self::Err> {
        match Self::new(token.to_owned()) {
            Some(token) => Ok(token),
            None => Err(
                "Authentication token must not start with \"Bearer\". Please just the token, the Authentication header will be formatted before starting the QUIC client.".to_string(),
            ),
        }
    }
}

impl Quic {
    pub(crate) async fn new(settings: &Settings) -> Result<Self> {
        let (local_socket, local_udp_client_addr) =
            Self::create_local_udp_socket(settings.quic_endpoint.is_ipv4()).await?;

        let quic_client_local_addr = if settings.quic_endpoint.is_ipv4() {
            SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))
        } else {
            SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0))
        };

        let mut config_builder = ClientConfig::builder()
            .client_socket(local_socket)
            .local_addr(quic_client_local_addr)
            .server_addr(settings.quic_endpoint)
            .server_host(settings.hostname.clone())
            .target_addr(settings.wireguard_endpoint)
            .auth_header(Some(settings.auth_header()))
            .mtu(settings.mtu.unwrap_or(1300)); // Default fallback MTU

        #[cfg(target_os = "linux")]
        {
            config_builder = config_builder.fwmark(settings.fwmark);
        }

        let client = Client::connect(config_builder.build())
            .await
            .map_err(Error::MasqueProxyError)?;

        let token = CancellationToken::new();
        let task = tokio::spawn(Self::run_forwarding(client, token.child_token()));

        Ok(Quic {
            local_endpoint: local_udp_client_addr,
            task,
            _shutdown: token.drop_guard(),
        })
    }

    async fn run_forwarding(
        masque_proxy_client: Client,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        log::trace!("Spawning QUIC client ..");
        let mut client = tokio::spawn(masque_proxy_client.run());
        log::trace!("QUIC client is running! QUIC Obfuscator is serving traffic 🎉");

        tokio::select! {
            _ = cancel_token.cancelled() => log::trace!("Stopping QUIC obfuscation"),
            _result = &mut client => log::trace!("QUIC client closed"),
        }

        client.abort();
        Ok(())
    }

    /// Create a local proxy client.
    ///
    /// The resulting UdpSocket/the SocketAddr where programs that want to obfuscate their
    /// traffic with QUIC will write to.
    async fn create_local_udp_socket(ipv4: bool) -> Result<(UdpSocket, SocketAddr)> {
        let bind_addr = if ipv4 {
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0))
        } else {
            SocketAddr::from((Ipv6Addr::LOCALHOST, 0))
        };

        let socket = UdpSocket::bind(bind_addr).await.map_err(Error::BindError)?;
        let addr = socket.local_addr().map_err(Error::LocalAddrError)?;
        Ok((socket, addr))
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
        // Approximate overhead of QUIC + MASQUE encapsulation
        // QUIC: ~40 bytes (variable), MASQUE: ~20 bytes
        60
    }

    #[cfg(target_os = "android")]
    fn remote_socket_fd(&self) -> std::os::unix::io::RawFd {
        todo!("Not yet implemented for Android")
    }
}
