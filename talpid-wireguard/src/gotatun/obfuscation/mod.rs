//! [`MaybeObfuscatingTransportFactory`] is an enum that either passes through to a plain UDP socket
//! or applies obfuscation.

mod lwo;
mod quic;

use std::{io, net::SocketAddr};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider::Tun;
use tunnel_obfuscation::Settings as ObfuscationSettings;

use crate::gotatun::obfuscation::quic::{NoopRecv, NoopSend, QuicTransportFactory};

use lwo::{LwoRecv, LwoSend, LwoUdpTransportFactory};
use quic::{QuicRecv, QuicSend};

/// A [`UdpSend`] wrapper that optionally obfuscates outgoing packets.
#[derive(Clone)]
pub enum MaybeObfuscatingSend<S: UdpSend> {
    Plain(S),
    Lwo(LwoSend<S>),
    QuicV4(QuicSend),
    QuicV6(NoopSend),
}

impl<S: UdpSend> UdpSend for MaybeObfuscatingSend<S> {
    type SendManyBuf = S::SendManyBuf;

    async fn send_to(&self, packet: Packet, destination: SocketAddr) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.send_to(packet, destination).await,
            Self::Lwo(inner) => inner.send_to(packet, destination).await,
            Self::QuicV4(inner) => inner.send_to(packet, destination).await,
            Self::QuicV6(inner) => inner.send_to(packet, destination).await,
        }
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        match self {
            Self::Plain(inner) => inner.max_number_of_packets_to_send(),
            Self::Lwo(inner) => inner.max_number_of_packets_to_send(),
            Self::QuicV4(inner) => inner.max_number_of_packets_to_send(),
            Self::QuicV6(inner) => inner.max_number_of_packets_to_send(),
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
            Self::QuicV4(inner) => inner.send_many_to(&mut (), packets).await,
            Self::QuicV6(inner) => inner.send_many_to(&mut (), packets).await,
        }
    }

    fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        match self {
            Self::Plain(inner) => inner.local_addr(),
            Self::Lwo(inner) => inner.local_addr(),
            Self::QuicV4(inner) => inner.local_addr(),
            Self::QuicV6(inner) => inner.local_addr(),
        }
    }

    #[cfg(target_os = "linux")]
    fn set_fwmark(&self, mark: u32) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.set_fwmark(mark),
            Self::Lwo(inner) => inner.set_fwmark(mark),
            Self::QuicV4(inner) => inner.set_fwmark(mark),
            Self::QuicV6(inner) => inner.set_fwmark(mark),
        }
    }
}

/// A [`UdpRecv`] enum that either passes through to a plain receiver or applies deobfuscation.
pub enum MaybeObfuscatingRecv<R: UdpRecv> {
    Plain(R),
    Lwo(LwoRecv<R>),
    QuicV4(QuicRecv),
    QuicV6(NoopRecv),
}

impl<R: UdpRecv> UdpRecv for MaybeObfuscatingRecv<R> {
    type RecvManyBuf = R::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        match self {
            Self::Plain(inner) => inner.recv_from(pool).await,
            Self::Lwo(inner) => inner.recv_from(pool).await,
            Self::QuicV4(inner) => inner.recv_from(pool).await,
            Self::QuicV6(inner) => inner.recv_from(pool).await,
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
            Self::QuicV4(inner) => inner.recv_many_from(&mut (), pool, packets).await,
            Self::QuicV6(inner) => inner.recv_many_from(&mut (), pool, packets).await,
        }
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.enable_udp_gro(),
            Self::Lwo(inner) => inner.enable_udp_gro(),
            Self::QuicV4(inner) => inner.enable_udp_gro(),
            Self::QuicV6(inner) => inner.enable_udp_gro(),
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
        #[cfg(target_os = "android")] android_tun: std::sync::Arc<Tun>,
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
                #[cfg(target_os = "android")]
                android_tun,
            }),

            // Use `Self::Plain` for proxy socket obfuscation or no obfuscation
            _ => Self::Plain(inner),
        }
    }
}

impl<F: UdpTransportFactory> UdpTransportFactory for MaybeObfuscatingTransportFactory<F> {
    type SendV4 = MaybeObfuscatingSend<F::SendV4>;
    type SendV6 = MaybeObfuscatingSend<F::SendV6>;
    type RecvV4 = MaybeObfuscatingRecv<F::RecvV4>;
    type RecvV6 = MaybeObfuscatingRecv<F::RecvV6>;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<((Self::SendV4, Self::RecvV4), (Self::SendV6, Self::RecvV6))> {
        match self {
            Self::Plain(factory) => {
                let ((sv4, rv4), (sv6, rv6)) = factory.bind(params).await?;
                Ok((
                    (
                        MaybeObfuscatingSend::Plain(sv4),
                        MaybeObfuscatingRecv::Plain(rv4),
                    ),
                    (
                        MaybeObfuscatingSend::Plain(sv6),
                        MaybeObfuscatingRecv::Plain(rv6),
                    ),
                ))
            }
            Self::Lwo(factory) => {
                let ((sv4, rv4), (sv6, rv6)) = factory.bind(params).await?;
                Ok((
                    (
                        MaybeObfuscatingSend::Lwo(sv4),
                        MaybeObfuscatingRecv::Lwo(rv4),
                    ),
                    (
                        MaybeObfuscatingSend::Lwo(sv6),
                        MaybeObfuscatingRecv::Lwo(rv6),
                    ),
                ))
            }
            Self::Quic(factory) => {
                let ((sv4, rv4), (sv6, rv6)) = factory.bind(params).await?;
                Ok((
                    (
                        MaybeObfuscatingSend::QuicV4(sv4),
                        MaybeObfuscatingRecv::QuicV4(rv4),
                    ),
                    (
                        MaybeObfuscatingSend::QuicV6(sv6),
                        MaybeObfuscatingRecv::QuicV6(rv6),
                    ),
                ))
            }
        }
    }
}
