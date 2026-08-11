//! Exposes an [ObfuscatedTransport] to a local WireGuard instance over a loopback UDP socket.

use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio_util::task::AbortOnDropHandle;

use crate::{
    LocalSocketObfuscator,
    transport::{MAX_DATAGRAM_SIZE, ObfuscatedTransport},
};

/// Runs an [ObfuscatedTransport] behind a loopback UDP socket, shuttling plaintext datagrams
/// between a local WireGuard instance and the transport.
pub struct LocalSocketRunner {
    /// Socket that the local WireGuard instance sends to and receives from.
    socket: UdpSocket,
    endpoint: SocketAddr,
    transport: Arc<dyn ObfuscatedTransport>,
}

impl LocalSocketRunner {
    /// Bind a loopback socket for WireGuard to talk to.
    pub async fn new(transport: Arc<dyn ObfuscatedTransport>) -> crate::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(crate::Error::BindLocalUdp)?;
        let endpoint = socket.local_addr().map_err(crate::Error::BindLocalUdp)?;

        Ok(Self {
            socket,
            endpoint,
            transport,
        })
    }
}

#[async_trait]
impl LocalSocketObfuscator for LocalSocketRunner {
    fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }

    fn packet_overhead(&self) -> u16 {
        self.transport.packet_overhead()
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        let Self {
            socket, transport, ..
        } = *self;

        connect_to_wireguard(&socket)
            .await
            .map_err(crate::Error::RunLocalSocketObfuscator)?;

        let socket = Arc::new(socket);

        let mut tx = AbortOnDropHandle::new(tokio::spawn(forward_to_transport(
            Arc::clone(&socket),
            Arc::clone(&transport),
        )));
        let mut rx = AbortOnDropHandle::new(tokio::spawn(forward_to_wireguard(socket, transport)));

        let result = tokio::select! {
            Ok(result) = &mut tx => result,
            Ok(result) = &mut rx => result,
            else => Ok(()),
        };
        result.map_err(crate::Error::RunLocalSocketObfuscator)
    }
}

/// Wait for WireGuard's first datagram and connect `socket` to its address, so that replies have
/// somewhere to go.
///
/// Blocks indefinitely, until WireGuard sends something.
async fn connect_to_wireguard(socket: &UdpSocket) -> io::Result<()> {
    let wg_addr = socket.peek_sender().await?;
    log::trace!("Local WireGuard instance connected from {wg_addr}");
    socket.connect(wg_addr).await
}

async fn forward_to_transport(
    socket: Arc<UdpSocket>,
    transport: Arc<dyn ObfuscatedTransport>,
) -> io::Result<()> {
    let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
    loop {
        let n = socket.recv(&mut buf).await?;
        transport.send(&mut buf[..n]).await?;
    }
}

async fn forward_to_wireguard(
    socket: Arc<UdpSocket>,
    transport: Arc<dyn ObfuscatedTransport>,
) -> io::Result<()> {
    let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
    loop {
        let n = transport.recv(&mut buf).await?;
        socket.send(&buf[..n]).await?;
    }
}
