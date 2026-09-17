use std::{io, net::SocketAddr, sync::Arc};

use bytes::BytesMut;
use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
use talpid_net::bypass::{BypassGuard, BypassSocket, SocketBypass};
use tokio::{sync::mpsc, task::JoinHandle};
use tunnel_obfuscation::quic::{Client, ClientConfig, MAX_INFLIGHT_PACKETS};

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
    pub client_task: Option<JoinHandle<()>>,
    pub bypass: Arc<dyn SocketBypass>,
}

impl QuicTransportFactory {
    fn stop_client(&mut self) {
        if let Some(task) = self.client_task.take() {
            task.abort();
        }
    }
}

impl Drop for QuicTransportFactory {
    fn drop(&mut self) {
        self.stop_client();
    }
}

impl UdpTransportFactory for QuicTransportFactory {
    type Send = QuicSend;
    type Recv = QuicRecv;

    async fn bind(
        &mut self,
        _params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        log::debug!("Starting QUIC proxy using userspace transport");
        if self.client_task.is_some() {
            log::debug!("Reconnecting to QUIC proxy");
        }
        self.stop_client();

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

        let (outgoing_tx, outgoing_rx) = mpsc::channel(MAX_INFLIGHT_PACKETS);
        let (incoming_tx, incoming_rx) = mpsc::channel(MAX_INFLIGHT_PACKETS);
        let send = QuicSend {
            packet_tx: outgoing_tx,
            _bypass_guard: bypass_guard.clone(),
        };
        let recv = QuicRecv {
            packet_rx: incoming_rx,
            target_addr: self.settings.wireguard_endpoint(),
            _bypass_guard: bypass_guard,
        };
        self.client_task = Some(tokio::spawn(run_client(config, outgoing_rx, incoming_tx)));
        Ok((send, recv))
    }
}

async fn run_client(
    config: ClientConfig,
    outgoing_rx: mpsc::Receiver<Packet>,
    incoming_tx: mpsc::Sender<BytesMut>,
) {
    let client = match Client::connect(config).await {
        Ok(client) => client,
        Err(error) => {
            log::error!("Failed to connect to QUIC proxy: {error}");
            return;
        }
    };

    let running_client = client.proxy_channels(outgoing_rx, incoming_tx);
    if let Err(error) = running_client.until_closed().await {
        log::error!("QUIC proxy client stopped: {error}");
    }
}
