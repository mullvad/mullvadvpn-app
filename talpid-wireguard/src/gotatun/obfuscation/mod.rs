//! [`MaybeObfuscatingTransportFactory`] is an enum that either passes through to a plain UDP socket
//! or applies obfuscation.

mod lwo;
mod transport;

use std::{io, net::SocketAddr, sync::Arc};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{
        UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams,
        socket::{UdpSocket, UdpSocketFactory},
    },
};
use talpid_net::bypass::{BypassSocket, SocketBypass};
use talpid_types::net::{obfuscation::LwoVersion, proxy::Socks5Proxy};
use tunnel_obfuscation::{Settings as TransportSettings, create_transport};

use crate::obfuscation::ObfuscationSettings;

use super::socks5::{MaybeSocks5Recv, MaybeSocks5Send, MaybeSocks5TransportFactory};
use lwo::{LwoKeys, LwoRecv, LwoSend, LwoUdpTransportFactory};
use transport::{ObfuscatingRecv, ObfuscatingSend};

pub use lwo::{lwo_timer_params, lwo_version};

type ProxiedFactory = MaybeSocks5TransportFactory<BypassingSocketFactory>;
type ProxiedSend = MaybeSocks5Send<BypassedUdpSend>;
type ProxiedRecv = MaybeSocks5Recv<BypassedUdpRecv>;

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
    Plain(ProxiedSend),
    Lwo(LwoSend<ProxiedSend>),
    Transport(ObfuscatingSend),
}

impl UdpSend for MaybeObfuscatingSend {
    type SendManyBuf = <UdpSocket as UdpSend>::SendManyBuf;

    async fn send_to(&self, packet: Packet, destination: SocketAddr) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.send_to(packet, destination).await,
            Self::Lwo(inner) => inner.send_to(packet, destination).await,
            Self::Transport(inner) => inner.send_to(packet, destination).await,
        }
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        match self {
            Self::Plain(inner) => inner.max_number_of_packets_to_send(),
            Self::Lwo(inner) => inner.max_number_of_packets_to_send(),
            Self::Transport(inner) => inner.max_number_of_packets_to_send(),
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
            Self::Transport(inner) => inner.send_many_to(&mut (), packets).await,
        }
    }

    fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        match self {
            Self::Plain(inner) => inner.local_addr(),
            Self::Lwo(inner) => inner.local_addr(),
            Self::Transport(inner) => inner.local_addr(),
        }
    }

    #[cfg(target_os = "linux")]
    fn set_fwmark(&self, mark: u32) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.set_fwmark(mark),
            Self::Lwo(inner) => inner.set_fwmark(mark),
            Self::Transport(inner) => inner.set_fwmark(mark),
        }
    }
}

/// A [`UdpRecv`] enum that either passes through to a plain receiver or applies deobfuscation.
pub enum MaybeObfuscatingRecv {
    Plain(ProxiedRecv),
    Lwo(LwoRecv<ProxiedRecv>),
    Transport(ObfuscatingRecv),
}

impl UdpRecv for MaybeObfuscatingRecv {
    type RecvManyBuf = <UdpSocket as UdpRecv>::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        match self {
            Self::Plain(inner) => inner.recv_from(pool).await,
            Self::Lwo(inner) => inner.recv_from(pool).await,
            Self::Transport(inner) => inner.recv_from(pool).await,
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
            Self::Transport(inner) => inner.recv_many_from(&mut (), pool, packets).await,
        }
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.enable_udp_gro(),
            Self::Lwo(inner) => inner.enable_udp_gro(),
            Self::Transport(inner) => inner.enable_udp_gro(),
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
    Plain(ProxiedFactory),
    Lwo(LwoUdpTransportFactory<ProxiedFactory>),
    Transport(TransportSettings, Arc<dyn SocketBypass>),
}

impl MaybeObfuscatingTransportFactory {
    /// Create a transport factory from the tunnel config.
    ///
    /// # Errors
    ///
    /// Fails if `proxy` is set but the obfuscation cannot be layered on top of it. Connecting
    /// directly when the caller asked to be relayed would leak their traffic, so this fails closed
    /// rather than ignoring the proxy.
    pub fn from_settings(
        optimize_buffer_size: bool,
        settings: Option<&ObfuscationSettings>,
        proxy: Option<Socks5Proxy>,
        bypass: Arc<dyn SocketBypass>,
    ) -> io::Result<Self> {
        if proxy.is_some()
            && settings.is_some_and(|settings| !settings.stacks_on_provided_socket())
        {
            return Err(io::Error::other(
                "the configured obfuscation cannot be relayed through a SOCKS5 proxy",
            ));
        }

        let make_factory = |bypass: Arc<dyn SocketBypass>| {
            let sockets = BypassingSocketFactory {
                bypass: Arc::clone(&bypass),
                inner: udp_socket_factory(optimize_buffer_size),
            };
            MaybeSocks5TransportFactory::new(sockets, proxy.clone(), bypass)
        };
        let factory = match settings.and_then(ObfuscationSettings::single) {
            Some(TransportSettings::Lwo(settings)) => Self::Lwo(LwoUdpTransportFactory {
                inner: make_factory(bypass),
                keys: match settings.version {
                    LwoVersion::V1 => LwoKeys::V1 {
                        tx_key: *settings.server_public_key.as_bytes(),
                        rx_key: *settings.client_public_key.as_bytes(),
                    },
                    LwoVersion::V2 => LwoKeys::V2 {
                        key: *settings.server_public_key.as_bytes(),
                    },
                },
                endpoint: settings.server_addr,
            }),
            Some(settings) => Self::Transport(settings.clone(), bypass),

            // No obfuscation, or a multiplexer that WireGuard reaches over a local socket.
            None => Self::Plain(make_factory(bypass)),
        };

        Ok(factory)
    }
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
            // The transport binds and excludes a socket of its own, so the addresses and the
            // fwmark in `params` do not apply to it.
            Self::Transport(settings, bypass) => {
                let transport = create_transport(Arc::clone(bypass), settings)
                    .await
                    .map_err(io::Error::other)?;
                let (sv, rv) = transport::split(transport);
                Ok((Send::Transport(sv), Recv::Transport(rv)))
            }
        }
    }
}
