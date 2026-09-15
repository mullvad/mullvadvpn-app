//! GotaTun UDP transports that obfuscate traffic without a local proxy socket.
//!
//! [`MaybeObfuscatingTransportFactory`] either passes through to another transport factory, or
//! applies obfuscation.

mod lwo;
mod transport;

use std::{io, net::SocketAddr, sync::Arc};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
use talpid_types::net::{obfuscation::LwoVersion, wireguard::PublicKey};

use crate::ObfuscatedTransport;

use lwo::{LwoKeys, LwoRecv, LwoSend, LwoUdpTransportFactory};
use transport::{ObfuscatingRecv, ObfuscatingSend};

pub use lwo::lwo_timer_params;

/// A running obfuscated transport.
#[derive(Clone)]
pub enum RunningObfuscation {
    /// Rewrite each datagram in place on its way out, over GotaTun's own socket.
    Lwo(crate::lwo::Settings),

    /// Carry the datagrams through this transport, which has a socket of its own.
    Transport(Arc<dyn ObfuscatedTransport>),
}

impl RunningObfuscation {
    /// Obfuscate for a WireGuard device that uses `client_public_key`, which LWO keys off.
    pub fn with_client_public_key(mut self, client_public_key: PublicKey) -> Self {
        if let RunningObfuscation::Lwo(settings) = &mut self {
            settings.client_public_key = client_public_key;
        }
        self
    }
}

/// A [`UdpSend`] wrapper that optionally obfuscates outgoing packets.
#[derive(Clone)]
pub enum MaybeObfuscatingSend<S: UdpSend> {
    Plain(S),
    Lwo(LwoSend<S>),
    Transport(ObfuscatingSend),
}

impl<S: UdpSend> UdpSend for MaybeObfuscatingSend<S> {
    type SendManyBuf = S::SendManyBuf;

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
pub enum MaybeObfuscatingRecv<R: UdpRecv> {
    Plain(R),
    Lwo(LwoRecv<R>),
    Transport(ObfuscatingRecv),
}

impl<R: UdpRecv> UdpRecv for MaybeObfuscatingRecv<R> {
    type RecvManyBuf = R::RecvManyBuf;

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

/// A [`UdpTransportFactory`] that either passes through to another factory or wraps it with
/// obfuscation.
pub enum MaybeObfuscatingTransportFactory<F: UdpTransportFactory> {
    Plain(F),
    Lwo(LwoUdpTransportFactory<F>),
    Transport {
        transport: Arc<dyn ObfuscatedTransport>,
        /// See [`ObfuscatingRecv`].
        peer_endpoint: SocketAddr,
    },
}

impl<F: UdpTransportFactory> MaybeObfuscatingTransportFactory<F> {
    /// Create a transport factory that sends through `inner`, obfuscated by `obfuscation` if it
    /// is set.
    ///
    /// A [`RunningObfuscation::Transport`] has a socket of its own, so `inner` is not used for it.
    pub fn new(
        inner: F,
        obfuscation: Option<RunningObfuscation>,
        peer_endpoint: SocketAddr,
    ) -> Self {
        match obfuscation {
            Some(RunningObfuscation::Lwo(settings)) => Self::Lwo(LwoUdpTransportFactory {
                inner,
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
            Some(RunningObfuscation::Transport(transport)) => Self::Transport {
                transport,
                peer_endpoint,
            },

            // Use `Self::Plain` when there is no obfuscation
            None => Self::Plain(inner),
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
            Self::Transport {
                transport,
                peer_endpoint,
            } => {
                let (sv, rv) = transport::split(Arc::clone(transport), *peer_endpoint);
                Ok((Send::Transport(sv), Recv::Transport(rv)))
            }
        }
    }
}
