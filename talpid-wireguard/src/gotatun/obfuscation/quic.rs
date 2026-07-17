use std::{io, net::SocketAddr};

use bytes::Bytes;
use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider::Tun;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct QuicSend {
    packet_tx: mpsc::Sender<Packet>,
}

pub struct QuicRecv {
    /// packets incoming from the network
    packet_rx: mpsc::Receiver<Bytes>,
    target_addr: SocketAddr,
}

impl UdpRecv for QuicRecv {
    type RecvManyBuf = ();

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
    /// Android tunnel used to bypass the quinn socket
    #[cfg(target_os = "android")]
    pub android_tun: std::sync::Arc<Tun>,
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
        log::debug!("Starting QUIC proxy using userspace transport");
        if self.running_client.is_some() {
            log::debug!("Reconnecting to QUIC proxy");
        }
        self.running_client = None;
        #[cfg(target_os = "linux")]
        if let Some(fwmark) = params.fwmark {
            self.settings.set_fwmark(fwmark);
        }
        let config = self
            .settings
            .build_client_config()
            .await
            .map_err(io::Error::other)?;

        #[cfg(target_os = "android")]
        {
            self.android_tun
                .bypass(&config.quinn_socket)
                .map_err(io::Error::other)?;
        }

        let client = tunnel_obfuscation::quic::Client::connect(config)
            .await
            .map_err(io::Error::other)?;

        let (outgoing_tx, outgoing_rx) =
            mpsc::channel(tunnel_obfuscation::quic::MAX_INFLIGHT_PACKETS);
        let (incoming_tx, incoming_rx) =
            mpsc::channel(tunnel_obfuscation::quic::MAX_INFLIGHT_PACKETS);
        let send = QuicSend {
            packet_tx: outgoing_tx,
        };
        let recv = QuicRecv {
            packet_rx: incoming_rx,
            target_addr: self.settings.wireguard_endpoint(),
        };
        let running_client = client.proxy_channels(outgoing_rx, incoming_tx);
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
