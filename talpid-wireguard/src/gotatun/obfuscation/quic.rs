use std::{io, net::SocketAddr};

use bytes::Bytes;
use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct QuicSend {
    packet_tx: mpsc::Sender<Packet>,
}

pub struct QuicRecv {
    /// packets incoming from the network
    packet_rx: mpsc::Receiver<Bytes>,
    // fragments: fragment::Fragments,
    target_addr: SocketAddr,
}

impl UdpRecv for QuicRecv {
    type RecvManyBuf = (); // TODO

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        let bytes = self
            .packet_rx
            .recv()
            .await
            .ok_or(io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))?;
        let mut packet = pool.get();
        // The packet comes with length 4096, we want to add bytes to the start
        // so we need to truncate it first
        packet.truncate(0);
        packet.buf_mut().extend_from_slice(&bytes);
        Ok((packet, self.target_addr))
    }
}

impl UdpSend for QuicSend {
    // Sending multiple packets at a time is pointless for an in process transport
    type SendManyBuf = ();

    async fn send_to(&self, packet: Packet, _destination: SocketAddr) -> io::Result<()> {
        self.packet_tx
            .send(packet)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
    }
}
pub struct QuicTransportFactory {
    pub(super) settings: tunnel_obfuscation::quic::Settings,
    pub running_client: Option<tunnel_obfuscation::quic::RunningClient>,
}

impl UdpTransportFactory for QuicTransportFactory {
    type SendV4 = QuicSend;
    type SendV6 = NoopSend;
    type RecvV4 = QuicRecv;
    type RecvV6 = NoopRecv;

    #[cfg_attr(not(target_os = "linux"), expect(unused))]
    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<((Self::SendV4, Self::RecvV4), (Self::SendV6, Self::RecvV6))> {
        if self.running_client.is_some() {
            // TODO: Is this an error?
            log::debug!("Reconnecting to QUIC proxy");
        }
        self.running_client = None;
        #[cfg(target_os = "linux")]
        if let Some(fwmark) = params.fwmark {
            self.settings.set_fwmark(fwmark);
        }
        let client = self
            .settings
            .connect_client()
            .await
            .map_err(io::Error::other)?; // TODO: Propagate inner IO error

        let (send_tx, send_rx) = mpsc::channel(1234); // TODO constant
        let (recv_tx, recv_rx) = mpsc::channel(1234); // TODO constant
        let send = QuicSend { packet_tx: send_tx };
        let recv = QuicRecv {
            packet_rx: recv_rx,
            target_addr: self.settings.wireguard_endpoint(),
        };
        let running_client = client.run_inline(send_rx, recv_tx);
        self.running_client = Some(running_client);
        Ok(((send, recv), (NoopSend, NoopRecv)))
    }
}

// The internal WireGuard endpoint for QUIC is always IPv4 (Ipv4Addr::LOCALHOST, 51820), but we must implement
// an internal transport for IPv6. These will never actually be called,
#[derive(Clone)]
pub struct NoopSend;
pub struct NoopRecv;
impl UdpRecv for NoopRecv {
    type RecvManyBuf = ();
    async fn recv_from(&mut self, _: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        std::future::pending().await
    }
}

impl UdpSend for NoopSend {
    type SendManyBuf = ();

    async fn send_to(&self, _: Packet, destination: SocketAddr) -> io::Result<()> {
        log::error!("Got unexpected packet to {destination:?}");
        Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "Proxying IPv6 WireGuard packets inside QUIC is not supported",
        ))
    }
}
