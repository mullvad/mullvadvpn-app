//! The UDP transport of GotaTun devices that reach a relay: sockets that bypass the tunnel, with
//! optional obfuscation.

use std::{io, net::SocketAddr, sync::Arc};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{
        UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams,
        socket::{UdpSocket, UdpSocketFactory},
    },
};
use talpid_net::bypass::{BypassSocket, SocketBypass};
use talpid_types::net::obfuscation::{LwoVersion, ObfuscatorConfig, Obfuscators};
use tunnel_obfuscation::gotatun_transport::MaybeObfuscatingTransportFactory;

use crate::{config::Config, obfuscation::RunningObfuscation};

pub use tunnel_obfuscation::gotatun_transport::lwo_timer_params;

/// A [`UdpTransportFactory`] for sockets that bypass the tunnel, with optional obfuscation.
pub type TransportFactory = MaybeObfuscatingTransportFactory<BypassingSocketFactory>;

/// Create a transport factory with optional obfuscation.
///
/// - `optimize_buffer_size`: if UDP socket buffer sizes should be tweaked.
///   This could be beneficial for performance reasons.
pub fn transport_factory(
    optimize_buffer_size: bool,
    obfuscation: Option<RunningObfuscation>,
    peer_endpoint: SocketAddr,
    bypass: Arc<dyn SocketBypass>,
) -> TransportFactory {
    let sockets = BypassingSocketFactory {
        inner: udp_socket_factory(optimize_buffer_size),
        bypass,
    };
    MaybeObfuscatingTransportFactory::new(sockets, obfuscation, peer_endpoint)
}

/// The LWO version the tunnel config connects with, if it uses LWO at all.
///
/// [`LwoVersion::V2`] peers must have [`lwo_timer_params`] applied.
pub fn lwo_version(config: &Config) -> Option<LwoVersion> {
    match &config.obfuscator_config {
        Some(Obfuscators::Single(ObfuscatorConfig::Lwo { version, .. })) => Some(*version),
        _ => None,
    }
}

#[derive(Clone)]
pub struct BypassedUdpSend(Arc<BypassSocket<UdpSocket>>);
pub struct BypassedUdpRecv(BypassSocket<UdpSocket>);

impl UdpSend for BypassedUdpSend {
    type SendManyBuf = <UdpSocket as UdpSend>::SendManyBuf;

    async fn send_to(&self, packet: Packet, destination: SocketAddr) -> io::Result<()> {
        self.0.socket.send_to(packet, destination).await
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        self.0.socket.max_number_of_packets_to_send()
    }

    async fn send_many_to(
        &self,
        send_buf: &mut Self::SendManyBuf,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        self.0.socket.send_many_to(send_buf, packets).await
    }

    fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        self.0.socket.local_addr().map(Some)
    }

    #[cfg(target_os = "linux")]
    fn set_fwmark(&self, mark: u32) -> io::Result<()> {
        self.0.socket.set_fwmark(mark)
    }
}

impl UdpRecv for BypassedUdpRecv {
    type RecvManyBuf = <UdpSocket as UdpRecv>::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        self.0.socket.recv_from(pool).await
    }

    async fn recv_many_from(
        &mut self,
        recv_buf: &mut Self::RecvManyBuf,
        pool: &mut PacketBufPool,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        self.0.socket.recv_many_from(recv_buf, pool, packets).await
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        self.0.socket.enable_udp_gro()
    }
}

pub struct BypassingSocketFactory {
    inner: UdpSocketFactory,
    bypass: Arc<dyn SocketBypass>,
}

/// Provide a [`UdpSocketFactory`] for the entry-device.
///
/// - `optimize_buffer_size`: if UDP socket buffer sizes should be tweaked.
///   This could be beneficial for performance reasons.
fn udp_socket_factory(optimize_buffer_size: bool) -> UdpSocketFactory {
    /// See [`DeviceBuilder::udp_send_buffer_size`] for details.
    const UDP_SEND_BUFFER_SIZE: usize = 7 * 1024 * 1024; // 7 MB (mirror the default of `gotatun-cli`)
    /// See [`DeviceBuilder::udp_recv_buffer_size`] for details.
    const UDP_RECV_BUFFER_SIZE: usize = 7 * 1024 * 1024;

    if optimize_buffer_size {
        UdpSocketFactory {
            recv_buffer_size: Some(UDP_RECV_BUFFER_SIZE),
            send_buffer_size: Some(UDP_SEND_BUFFER_SIZE),
        }
    } else {
        UdpSocketFactory::default()
    }
}

impl UdpTransportFactory for BypassingSocketFactory {
    type Send = BypassedUdpSend;
    type Recv = BypassedUdpRecv;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        let (sv, rv) = self.inner.bind(params).await?;

        let send = BypassedUdpSend(Arc::new(BypassSocket::new(self.bypass.clone(), sv)?));
        let recv = BypassedUdpRecv(BypassSocket::new(self.bypass.clone(), rv)?);
        Ok((send, recv))
    }
}
