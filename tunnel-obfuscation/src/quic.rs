//! Quic obfuscation

use async_trait::async_trait;
use mullvad_masque_proxy::client::ClientConfig;
use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

pub use mullvad_masque_proxy::{
    HTTP_MASQUE_DATAGRAM_CONTEXT_ID, MAX_INFLIGHT_PACKETS,
    client::{Client, RunningClient},
};

use crate::{LocalSocketObfuscator, socket::create_remote_socket};

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to bind UDP socket")]
    BindError(#[from] io::Error),
    #[error("Masque proxy error")]
    MasqueProxyError(#[from] mullvad_masque_proxy::client::Error),
}

#[derive(Debug)]
pub struct QuicLocalSocket {
    local_endpoint: SocketAddr,
    /// Local UDP socket that WireGuard sends to and receives from.
    local_socket: UdpSocket,
    config: ClientConfig,
}

#[derive(Debug, Clone)]
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
    ///See [Settings] for details.
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
        debug_assert!(mtu <= 1500, "MTU is too high: {mtu}");
        let mtu = Some(mtu);
        Self { mtu, ..self }
    }

    /// Set `fwmark` for the Quic obfuscator.
    #[cfg(target_os = "linux")]
    pub fn set_fwmark(&mut self, fwmark: u32) {
        self.fwmark = Some(fwmark);
    }

    /// The masque-proxy server expects the Authentication header to be prefixed with "Bearer ", so
    /// prefix the auth token with that.
    fn auth_header(&self) -> String {
        format!("Bearer {token}", token = self.token.0)
    }

    /// Build the masque-proxy [`ClientConfig`], including binding the local QUIC endpoint socket.
    pub async fn build_client_config(&self) -> Result<ClientConfig> {
        let quic_socket = create_remote_socket(
            self.quic_endpoint.is_ipv4(),
            #[cfg(target_os = "linux")]
            self.fwmark,
        )
        .await
        .map_err(io::Error::from)?;

        Ok(ClientConfig::builder()
            .quinn_socket(quic_socket)
            .server_addr(self.quic_endpoint)
            .server_host(self.hostname.clone())
            .target_addr(self.wireguard_endpoint)
            .auth_header(Some(self.auth_header()))
            .mtu(self.mtu.unwrap_or(1500))
            .build())
    }

    pub fn wireguard_endpoint(&self) -> SocketAddr {
        self.wireguard_endpoint
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
        };
        Some(Self(token))
    }
}

impl std::str::FromStr for AuthToken {
    type Err = String;

    fn from_str(token: &str) -> std::result::Result<Self, Self::Err> {
        match Self::new(token.to_owned()) {
            Some(token) => Ok(token),
            None => Err(
            "Authentication token must not start with \"Bearer\". Please just the token, the Authentication header will be formatted before starting the QUIC client."
                .to_string())

        }
    }
}

impl QuicLocalSocket {
    pub(crate) async fn new(settings: &Settings) -> crate::Result<Self> {
        log::debug!("Starting QUIC proxy client over local socket");
        // The address family of the local QUIC client socket has to match the address family
        // of the endpoint we're connecting to. The address itself is not important to consumers wanting
        // to obfuscate traffic. It is solely used by the local proxy client to know where the QUIC
        // obfuscator is running.
        let (local_socket, local_udp_client_addr) =
            QuicLocalSocket::create_local_udp_socket(settings.quic_endpoint.is_ipv4())
                .await
                .map_err(crate::Error::CreateQuicObfuscator)?;
        let config = settings
            .build_client_config()
            .await
            .map_err(crate::Error::CreateQuicObfuscator)?;

        let quic = QuicLocalSocket {
            local_endpoint: local_udp_client_addr,
            local_socket,
            config,
        };

        Ok(quic)
    }

    async fn run_forwarding(
        client: Client,
        local_socket: UdpSocket,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        let client = client.proxy_socket(local_socket);
        log::trace!("QUIC client is running! QUIC Obfuscator is serving traffic 🎉");
        tokio::select! {
            _ = cancel_token.cancelled() => log::trace!("Stopping QUIC obfuscation"),
            _result = client.until_closed() => log::trace!("QUIC client closed"),
        };

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
impl LocalSocketObfuscator for QuicLocalSocket {
    fn endpoint(&self) -> SocketAddr {
        self.local_endpoint
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        let Self {
            config,
            local_socket,
            ..
        } = *self;

        let token = CancellationToken::new();
        let child_token = token.child_token();
        // This will always cancel `child_token` as soon as `run` is finished or aborted.
        let _drop_guard = token.drop_guard();

        let client = Client::connect(config)
            .await
            .map_err(Error::MasqueProxyError)
            .map_err(crate::Error::RunQuicObfuscator)?;

        tokio::spawn(QuicLocalSocket::run_forwarding(
            client,
            local_socket,
            child_token,
        ))
        .await
        .unwrap()
        .map_err(crate::Error::RunQuicObfuscator)
    }

    fn packet_overhead(&self) -> u16 {
        // TODO: 95 = IPv6 (40) + UDP (8) + QUIC (<= 41) + stream ID (1) + fragment header (5)
        // The above would prevent mullvad-masque-proxy-level fragmentation
        0
    }

    #[cfg(target_os = "android")]
    fn remote_socket_fd(&self) -> std::os::unix::io::RawFd {
        use std::os::fd::AsRawFd;
        self.config.quinn_socket.as_raw_fd()
    }
}
