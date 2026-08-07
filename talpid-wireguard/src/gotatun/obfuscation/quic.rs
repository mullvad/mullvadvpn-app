use std::{io, net::SocketAddr, sync::Arc};

use bytes::BytesMut;
use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
use talpid_net::bypass::{BypassGuard, BypassSocket, SocketBypass};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct QuicSend {
    packet_tx: mpsc::Sender<Packet>,
    _bypass_guard: Arc<BypassGuard>,
}

pub struct QuicRecv {
    /// packets incoming from the network
    packet_rx: mpsc::Receiver<BytesMut>,
    target_addr: SocketAddr,
    _bypass_guard: Arc<BypassGuard>,
}

impl UdpRecv for QuicRecv {
    type RecvManyBuf = ();

    async fn recv_from(&mut self, _pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        let bytes = self
            .packet_rx
            .recv()
            .await
            .ok_or(io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))?;

        Ok((Packet::from_bytes(bytes), self.target_addr))
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
    pub bypass: Arc<dyn SocketBypass>,
}

impl UdpTransportFactory for QuicTransportFactory {
    type Send = QuicSend;
    type Recv = QuicRecv;

    async fn bind(
        &mut self,
        _params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        log::debug!("Starting QUIC proxy using userspace transport");
        if self.running_client.is_some() {
            log::debug!("Reconnecting to QUIC proxy");
        }
        self.running_client = None;

        let BypassSocket {
            socket: quinn_socket,
            guard: bypass_guard,
        } = tunnel_obfuscation::socket::create_remote_socket(
            &self.bypass,
            self.settings.quic_endpoint().is_ipv4(),
        )
        .await
        .map_err(io::Error::other)?;
        let bypass_guard = Arc::new(bypass_guard);

        let config = self.settings.build_client_config(quinn_socket);

        let client = tunnel_obfuscation::quic::Client::connect(config)
            .await
            .map_err(io::Error::other)?;

        let (outgoing_tx, outgoing_rx) =
            mpsc::channel(tunnel_obfuscation::quic::MAX_INFLIGHT_PACKETS);
        let (incoming_tx, incoming_rx) =
            mpsc::channel(tunnel_obfuscation::quic::MAX_INFLIGHT_PACKETS);
        let send = QuicSend {
            packet_tx: outgoing_tx,
            _bypass_guard: bypass_guard.clone(),
        };
        let recv = QuicRecv {
            packet_rx: incoming_rx,
            target_addr: self.settings.wireguard_endpoint(),
            _bypass_guard: bypass_guard,
        };
        let running_client = client.proxy_channels(outgoing_rx, incoming_tx);
        self.running_client = Some(running_client);
        Ok((send, recv))
    }
}
