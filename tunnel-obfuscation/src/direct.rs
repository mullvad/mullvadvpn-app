//! Unobfuscated UDP forwarding.

use std::{io, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use talpid_net::bypass::{BypassSocket, SocketBypass};
use tokio::net::UdpSocket;

use crate::{socket::create_remote_socket, transport::ObfuscatedTransport};

/// Sends WireGuard traffic to the remote endpoint verbatim.
///
/// The [multiplexer](crate::multiplexer) races this against real obfuscation methods, so that an
/// unobfuscated connection is used when it works.
pub struct Direct {
    socket: BypassSocket<UdpSocket>,
    peer: SocketAddr,
}

impl Direct {
    pub async fn new(bypass: &Arc<dyn SocketBypass>, peer: SocketAddr) -> crate::Result<Self> {
        let socket = create_remote_socket(bypass, peer.is_ipv4()).await?;
        socket
            .connect(peer)
            .await
            .map_err(crate::Error::ConnectRemoteUdp)?;
        Ok(Self { socket, peer })
    }
}

#[async_trait]
impl ObfuscatedTransport for Direct {
    async fn send(&self, packet: &mut [u8]) -> io::Result<()> {
        self.socket.send(packet).await.map(drop)
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buf).await
    }

    fn endpoint(&self) -> SocketAddr {
        self.peer
    }

    fn packet_overhead(&self) -> u16 {
        0
    }
}
