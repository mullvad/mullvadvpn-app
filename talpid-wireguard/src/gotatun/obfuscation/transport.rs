//! Implements `UdpRecv` and `UdpSend` for any [`ObfuscatedTransport`].

use std::{io, net::SocketAddr, sync::Arc};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend},
};
use tunnel_obfuscation::transport::ObfuscatedTransport;

/// Split `transport` into the halves that [`UdpTransportFactory::bind`] returns.
///
/// [`UdpTransportFactory::bind`]: gotatun::udp::UdpTransportFactory::bind
pub fn split(
    transport: Arc<dyn ObfuscatedTransport>,
    peer_endpoint: SocketAddr,
) -> (ObfuscatingSend, ObfuscatingRecv) {
    (
        ObfuscatingSend(Arc::clone(&transport)),
        ObfuscatingRecv {
            transport,
            peer_endpoint,
        },
    )
}

/// The sending half, which obfuscates packets and sends them to the relay.
#[derive(Clone)]
pub struct ObfuscatingSend(Arc<dyn ObfuscatedTransport>);

impl UdpSend for ObfuscatingSend {
    /// Obfuscation methods take one datagram at a time, so there is nothing to batch.
    type SendManyBuf = ();

    async fn send_to(&self, mut packet: Packet, _destination: SocketAddr) -> io::Result<()> {
        // The transport knows the relay it is talking to, so the destination GotaTun asks for
        // carries no information that we do not already have.
        self.0.send(&mut packet).await
    }
}

/// The receiving half, which takes packets the transport received from the relay.
pub struct ObfuscatingRecv {
    transport: Arc<dyn ObfuscatedTransport>,

    /// Reported as the source of every datagram.
    ///
    /// This likely does not matter, but we set it to the WireGuard endpoint.
    peer_endpoint: SocketAddr,
}

impl UdpRecv for ObfuscatingRecv {
    /// See [`ObfuscatingSend::SendManyBuf`].
    type RecvManyBuf = ();

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        // Packets from the pool are far larger than any WireGuard datagram, so a datagram that
        // does not fit here was not meant for us.
        let mut packet = pool.get();
        let len = self.transport.recv(&mut packet).await?;
        packet.truncate(len);
        Ok((packet, self.peer_endpoint))
    }
}
