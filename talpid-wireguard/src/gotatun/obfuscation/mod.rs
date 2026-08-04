//! [`MaybeObfuscatingTransportFactory`] is an enum that either passes through to a plain UDP socket
//! or applies obfuscation.

mod lwo;
mod quic;

use std::{io, net::SocketAddr, sync::Arc};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
use talpid_net::bypass::SocketBypass;
use tunnel_obfuscation::Settings as ObfuscationSettings;

use crate::gotatun::obfuscation::quic::QuicTransportFactory;

use lwo::{LwoRecv, LwoSend, LwoUdpTransportFactory};
use quic::{QuicRecv, QuicSend};

/// A [`UdpSend`] wrapper that optionally obfuscates outgoing packets.
#[derive(Clone)]
pub enum MaybeObfuscatingSend<S: UdpSend> {
    Plain(S),
    Lwo(LwoSend<S>),
    Quic(QuicSend),
}

impl<S: UdpSend> UdpSend for MaybeObfuscatingSend<S> {
    type SendManyBuf = S::SendManyBuf;

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
pub enum MaybeObfuscatingRecv<R: UdpRecv> {
    Plain(R),
    Lwo(LwoRecv<R>),
    Quic(QuicRecv),
}

impl<R: UdpRecv> UdpRecv for MaybeObfuscatingRecv<R> {
    type RecvManyBuf = R::RecvManyBuf;

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

/// A [`UdpTransportFactory`] that either passes through to a plain factory or wraps it with
/// obfuscation.
pub enum MaybeObfuscatingTransportFactory<F: UdpTransportFactory> {
    Plain(F),
    Lwo(LwoUdpTransportFactory<F>),
    Quic(QuicTransportFactory),
}

impl<F: UdpTransportFactory> MaybeObfuscatingTransportFactory<F> {
    /// Create a transport factory from the tunnel config.
    pub fn from_settings(
        inner: F,
        settings: Option<&ObfuscationSettings>,
        bypass: Arc<dyn SocketBypass>,
    ) -> Self {
        match settings {
            Some(ObfuscationSettings::Lwo(settings)) => Self::Lwo(LwoUdpTransportFactory {
                inner,
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
            _ => Self::Plain(inner),
        }
    }
}

impl<F: UdpTransportFactory> UdpTransportFactory for MaybeObfuscatingTransportFactory<F> {
    type Send = MaybeObfuscatingSend<F::Send>;
    type Recv = MaybeObfuscatingRecv<F::Recv>;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        match self {
            Self::Plain(factory) => {
                let (sv, rv) = factory.bind(params).await?;
                Ok((
                    MaybeObfuscatingSend::Plain(sv),
                    MaybeObfuscatingRecv::Plain(rv),
                ))
            }
            Self::Lwo(factory) => {
                let (sv, rv) = factory.bind(params).await?;
                Ok((MaybeObfuscatingSend::Lwo(sv), MaybeObfuscatingRecv::Lwo(rv)))
            }
            Self::Quic(factory) => {
                let (sv, rv) = factory.bind(params).await?;
                Ok((
                    MaybeObfuscatingSend::Quic(sv),
                    MaybeObfuscatingRecv::Quic(rv),
                ))
            }
        }
    }
}
