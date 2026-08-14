//! Quic obfuscation

use async_trait::async_trait;
use bytes::BytesMut;
use mullvad_masque_proxy::client::ClientConfig;
use std::{io, net::SocketAddr, sync::Arc};
use talpid_net::bypass::{BypassGuard, BypassSocket, SocketBypass};
use tokio::{
    net::UdpSocket,
    sync::{Mutex, mpsc},
};
use tokio_util::task::AbortOnDropHandle;

pub use mullvad_masque_proxy::{
    HTTP_MASQUE_DATAGRAM_CONTEXT_ID, MAX_INFLIGHT_PACKETS,
    client::{Client, RunningClient},
};

use crate::{socket::create_remote_socket, transport::ObfuscatedTransport};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to bind UDP socket")]
    BindError(#[from] io::Error),
    #[error("Masque proxy error")]
    MasqueProxyError(#[from] mullvad_masque_proxy::client::Error),
}

/// Tunnels WireGuard packets through a MASQUE proxy over QUIC.
pub struct QuicTransport {
    /// Plaintext packets to be proxied to the remote.
    outgoing_tx: mpsc::Sender<BytesMut>,
    /// Plaintext packets received from the remote. Only ever drained by [ObfuscatedTransport::recv],
    /// so the lock is uncontended.
    incoming_rx: Mutex<mpsc::Receiver<BytesMut>>,
    /// Aborts the QUIC client when this transport is dropped.
    _client: AbortOnDropHandle<()>,
    _bypass: BypassGuard,
    wireguard_endpoint: SocketAddr,
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
        }
    }

    /// Set an explicit MTU for the Quic obfuscator.
    pub fn mtu(self, mtu: u16) -> Self {
        debug_assert!(mtu <= 1500, "MTU is too high: {mtu}");
        let mtu = Some(mtu);
        Self { mtu, ..self }
    }

    /// The masque-proxy server expects the Authentication header to be prefixed with "Bearer ", so
    /// prefix the auth token with that.
    fn auth_header(&self) -> String {
        format!("Bearer {token}", token = self.token.0)
    }

    /// Build the masque-proxy [`ClientConfig`], including binding the local QUIC endpoint socket.
    pub fn build_client_config(&self, quinn_socket: UdpSocket) -> ClientConfig {
        ClientConfig::builder()
            .quinn_socket(quinn_socket)
            .server_addr(self.quic_endpoint)
            .server_host(self.hostname.clone())
            .target_addr(self.wireguard_endpoint)
            .auth_header(Some(self.auth_header()))
            .mtu(self.mtu.unwrap_or(1500))
            .build()
    }

    pub fn wireguard_endpoint(&self) -> SocketAddr {
        self.wireguard_endpoint
    }

    pub fn quic_endpoint(&self) -> SocketAddr {
        self.quic_endpoint
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    /// The authentication token, without the "Bearer " prefix. See [`Settings::token`].
    pub fn auth_token(&self) -> &str {
        &self.token.0
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

impl QuicTransport {
    pub(crate) async fn new(
        bypass: Arc<dyn SocketBypass>,
        settings: &Settings,
    ) -> crate::Result<Self> {
        let BypassSocket {
            socket: quic_socket,
            guard: _bypass,
        } = create_remote_socket(&bypass, settings.quic_endpoint.is_ipv4()).await?;

        let config = settings.build_client_config(quic_socket);

        let (outgoing_tx, outgoing_rx) = mpsc::channel(MAX_INFLIGHT_PACKETS);
        let (incoming_tx, incoming_rx) = mpsc::channel(MAX_INFLIGHT_PACKETS);

        let client = tokio::spawn(run_client(config, outgoing_rx, incoming_tx));

        Ok(Self {
            outgoing_tx,
            incoming_rx: Mutex::new(incoming_rx),
            _client: AbortOnDropHandle::new(client),
            _bypass,
            wireguard_endpoint: settings.wireguard_endpoint,
        })
    }
}

async fn run_client(
    config: ClientConfig,
    outgoing_rx: mpsc::Receiver<BytesMut>,
    incoming_tx: mpsc::Sender<BytesMut>,
) {
    let client = match Client::connect(config).await {
        Ok(client) => client,
        Err(err) => {
            log::error!("Failed to connect QUIC client: {err}");
            return;
        }
    };
    log::trace!("QUIC client is running! QUIC Obfuscator is serving traffic 🎉");

    if let Err(err) = client
        .proxy_channels(outgoing_rx, incoming_tx)
        .until_closed()
        .await
    {
        log::debug!("QUIC client closed: {err}");
    }
}

#[async_trait]
impl ObfuscatedTransport for QuicTransport {
    async fn send(&self, packet: &mut [u8]) -> io::Result<()> {
        self.outgoing_tx
            .send(BytesMut::from(&packet[..]))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "QUIC client stopped"))
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let packet = self
            .incoming_rx
            .lock()
            .await
            .recv()
            .await
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "QUIC client stopped"))?;

        if packet.len() > buf.len() {
            return Err(io::Error::other(format!(
                "QUIC packet of {} bytes does not fit in a {} byte buffer",
                packet.len(),
                buf.len()
            )));
        }
        buf[..packet.len()].copy_from_slice(&packet);
        Ok(packet.len())
    }

    fn endpoint(&self) -> SocketAddr {
        self.wireguard_endpoint
    }

    fn packet_overhead(&self) -> u16 {
        // TODO: 95 = IPv6 (40) + UDP (8) + QUIC (<= 41) + stream ID (1) + fragment header (5)
        // The above would prevent mullvad-masque-proxy-level fragmentation
        0
    }
}
