//! [`MaybeObfuscatingTransportFactory`] is an enum that either passes through to a plain UDP socket
//! or applies obfuscation.

mod lwo;
mod quic;

use std::{io, net::SocketAddr, sync::Arc};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{
        UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams,
        socket::{UdpSocket, UdpSocketFactory},
    },
};
use talpid_net::bypass::{BypassSocket, SocketBypass};
use tunnel_obfuscation::Settings as ObfuscationSettings;

use crate::gotatun::obfuscation::quic::QuicTransportFactory;

use lwo::{LwoRecv, LwoSend, LwoUdpTransportFactory};
use quic::{QuicRecv, QuicSend};

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

/// A [`UdpSend`] wrapper that optionally obfuscates outgoing packets.
#[derive(Clone)]
pub enum MaybeObfuscatingSend {
    Plain(BypassedUdpSend),
    Lwo(LwoSend<BypassedUdpSend>),
    Quic(QuicSend),
}

impl UdpSend for MaybeObfuscatingSend {
    type SendManyBuf = <UdpSocket as UdpSend>::SendManyBuf;

    async fn send_to(&self, packet: Packet, destination: SocketAddr) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.send_to(packet, destination).await,
            Self::Lwo(inner) => inner.send_to(packet, destination).await,
            Self::Quic(inner) => inner.send_to(packet, destination).await,
        }
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        match self {
            Self::Plain(inner) => inner.max_number_of_packets_to_send(),
            Self::Lwo(inner) => inner.max_number_of_packets_to_send(),
            Self::Quic(inner) => inner.max_number_of_packets_to_send(),
        }
    }

    async fn send_many_to(
        &self,
        send_buf: &mut Self::SendManyBuf,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.send_many_to(send_buf, packets).await,
            Self::Lwo(inner) => inner.send_many_to(send_buf, packets).await,
            Self::Quic(inner) => inner.send_many_to(&mut (), packets).await,
        }
    }

    fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        match self {
            Self::Plain(inner) => inner.local_addr(),
            Self::Lwo(inner) => inner.local_addr(),
            Self::Quic(inner) => inner.local_addr(),
        }
    }

    #[cfg(target_os = "linux")]
    fn set_fwmark(&self, mark: u32) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.set_fwmark(mark),
            Self::Lwo(inner) => inner.set_fwmark(mark),
            Self::Quic(inner) => inner.set_fwmark(mark),
        }
    }
}

/// A [`UdpRecv`] enum that either passes through to a plain receiver or applies deobfuscation.
pub enum MaybeObfuscatingRecv {
    Plain(BypassedUdpRecv),
    Lwo(LwoRecv<BypassedUdpRecv>),
    Quic(QuicRecv),
}

impl UdpRecv for MaybeObfuscatingRecv {
    type RecvManyBuf = <UdpSocket as UdpRecv>::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        match self {
            Self::Plain(inner) => inner.recv_from(pool).await,
            Self::Lwo(inner) => inner.recv_from(pool).await,
            Self::Quic(inner) => inner.recv_from(pool).await,
        }
    }

    async fn recv_many_from(
        &mut self,
        recv_buf: &mut Self::RecvManyBuf,
        pool: &mut PacketBufPool,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.recv_many_from(recv_buf, pool, packets).await,
            Self::Lwo(inner) => inner.recv_many_from(recv_buf, pool, packets).await,
            Self::Quic(inner) => inner.recv_many_from(&mut (), pool, packets).await,
        }
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.enable_udp_gro(),
            Self::Lwo(inner) => inner.enable_udp_gro(),
            Self::Quic(inner) => inner.enable_udp_gro(),
        }
    }
}

pub struct BypassingSocketFactory {
    inner: UdpSocketFactory,
    bypass: Arc<dyn SocketBypass>,
}

/// A [`UdpTransportFactory`] that either passes through to a plain factory or wraps it with
/// obfuscation.
pub enum MaybeObfuscatingTransportFactory {
    Plain(BypassingSocketFactory),
    Lwo(LwoUdpTransportFactory<BypassingSocketFactory>),
    Quic(QuicTransportFactory),
}

impl MaybeObfuscatingTransportFactory {
    /// Create a transport factory from the tunnel config.
    pub fn from_settings(
        optimize_buffer_size: bool,
        settings: Option<&ObfuscationSettings>,
        bypass: Arc<dyn SocketBypass>,
    ) -> Self {
        let make_factory = |bypass| BypassingSocketFactory {
            bypass,
            inner: udp_socket_factory(optimize_buffer_size),
        };
        match settings {
            Some(ObfuscationSettings::Lwo(settings)) => Self::Lwo(LwoUdpTransportFactory {
                inner: make_factory(bypass),
                rx_key: *settings.client_public_key.as_bytes(),
                tx_key: *settings.server_public_key.as_bytes(),
                endpoint: settings.server_addr,
            }),
            Some(ObfuscationSettings::Quic(settings)) => Self::Quic(QuicTransportFactory {
                settings: settings.clone(),
                running_client: None,
                bypass,
            }),

            // Use `Self::Plain` for proxy socket obfuscation or no obfuscation
            _ => Self::Plain(make_factory(bypass)),
        }
    }
}

/// Provide a [`UdpSocketFactory`] for the entry-device.
///
/// - `optimize_buffer_size`: if UDP socket buffer sizes should be tweaked.
///   This could be beneficial for performance reasons.
#[inline(always)]
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

impl UdpTransportFactory for MaybeObfuscatingTransportFactory {
    type Send = MaybeObfuscatingSend;
    type Recv = MaybeObfuscatingRecv;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        use MaybeObfuscatingRecv as Recv;
        use MaybeObfuscatingSend as Send;
        match self {
            Self::Plain(factory) => {
                let (sv, rv) = factory.bind(params).await?;
                Ok((Send::Plain(sv), Recv::Plain(rv)))
            }
            Self::Lwo(factory) => {
                let (sv, rv) = factory.bind(params).await?;
                Ok((Send::Lwo(sv), Recv::Lwo(rv)))
            }
            Self::Quic(factory) => {
                let (sv, rv) = factory.bind(params).await?;
                Ok((Send::Quic(sv), Recv::Quic(rv)))
            }
        }
    }
}
