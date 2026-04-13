//! [`MaybeObfuscatingTransportFactory`] is an enum that either passes through to a plain UDP socket
//! or applies obfuscation.

mod lwo;

use std::{io, net::SocketAddr};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};

use crate::config::Config;

use lwo::{LwoRecv, LwoSend, LwoUdpTransportFactory, lwo_keys_from_config};

/// A [`UdpSend`] wrapper that optionally obfuscates outgoing packets.
#[derive(Clone)]
pub enum MaybeObfuscatingSend<S: UdpSend> {
    Plain(S),
    Lwo(LwoSend<S>),
}

impl<S: UdpSend> UdpSend for MaybeObfuscatingSend<S> {
    type SendManyBuf = S::SendManyBuf;

    async fn send_to(&self, packet: Packet, destination: SocketAddr) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.send_to(packet, destination).await,
            Self::Lwo(inner) => inner.send_to(packet, destination).await,
        }
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        match self {
            Self::Plain(inner) => inner.max_number_of_packets_to_send(),
            Self::Lwo(inner) => inner.max_number_of_packets_to_send(),
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
        }
    }

    fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        match self {
            Self::Plain(inner) => inner.local_addr(),
            Self::Lwo(inner) => inner.local_addr(),
        }
    }

    #[cfg(target_os = "linux")]
    fn set_fwmark(&self, mark: u32) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.set_fwmark(mark),
            Self::Lwo(inner) => inner.set_fwmark(mark),
        }
    }
}

/// A [`UdpRecv`] enum that either passes through to a plain receiver or applies deobfuscation.
pub enum MaybeObfuscatingRecv<R: UdpRecv> {
    Plain(R),
    Lwo(LwoRecv<R>),
}

impl<R: UdpRecv> UdpRecv for MaybeObfuscatingRecv<R> {
    type RecvManyBuf = R::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        match self {
            Self::Plain(inner) => inner.recv_from(pool).await,
            Self::Lwo(inner) => inner.recv_from(pool).await,
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
        }
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.enable_udp_gro(),
            Self::Lwo(inner) => inner.enable_udp_gro(),
        }
    }
}

/// A [`UdpTransportFactory`] that either passes through to a plain factory or wraps it with
/// obfuscation.
pub enum MaybeObfuscatingTransportFactory<F: UdpTransportFactory> {
    Plain(F),
    Lwo(LwoUdpTransportFactory<F>),
}

impl<F: UdpTransportFactory> MaybeObfuscatingTransportFactory<F> {
    /// Create a transport factory from the tunnel config.
    pub fn from_config(inner: F, config: &Config) -> Self {
        match lwo_keys_from_config(config) {
            Some((tx_key, rx_key)) => Self::Lwo(LwoUdpTransportFactory {
                inner,
                tx_key,
                rx_key,
            }),
            // Use `Self::Plain` for proxy socket obfuscation or no obfuscation
            None => Self::Plain(inner),
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
        }
    }
}
