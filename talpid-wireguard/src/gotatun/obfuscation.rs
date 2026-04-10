//! [`MaybeObfuscatingTransportFactory`] is an enum that either passes through to a plain UDP socket
//! or applies obfuscation.

use std::{io, net::SocketAddr};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
use talpid_types::net::obfuscation::{ObfuscatorConfig, Obfuscators};
use tunnel_obfuscation::lwo;

use crate::config::Config;

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

/// A [`UdpSend`] wrapper that LWO-obfuscates every outgoing packet before forwarding it to the
/// inner sender.
///
/// `tx_key` must be the **server** public key (the key used by the relay to deobfuscate).
#[derive(Clone)]
pub struct LwoSend<S: UdpSend> {
    inner: S,
    tx_key: [u8; 32],
}

impl<S: UdpSend> UdpSend for LwoSend<S> {
    type SendManyBuf = S::SendManyBuf;

    async fn send_to(&self, mut packet: Packet, destination: SocketAddr) -> io::Result<()> {
        lwo::obfuscate_thread_local(&mut packet, &self.tx_key);
        self.inner.send_to(packet, destination).await
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        self.inner.max_number_of_packets_to_send()
    }

    async fn send_many_to(
        &self,
        send_buf: &mut Self::SendManyBuf,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        obfuscate_all(packets, &self.tx_key);
        self.inner.send_many_to(send_buf, packets).await
    }

    fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        self.inner.local_addr()
    }

    #[cfg(target_os = "linux")]
    fn set_fwmark(&self, mark: u32) -> io::Result<()> {
        self.inner.set_fwmark(mark)
    }
}

/// A [`UdpRecv`] wrapper that LWO-deobfuscates every incoming packet after receiving it from the
/// inner receiver.
///
/// `rx_key` must be the **client** public key (the key used by the client to deobfuscate).
pub struct LwoRecv<R: UdpRecv> {
    inner: R,
    rx_key: [u8; 32],
}

impl<R: UdpRecv> UdpRecv for LwoRecv<R> {
    type RecvManyBuf = R::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        let (mut packet, addr) = self.inner.recv_from(pool).await?;
        lwo::deobfuscate(&mut packet, &self.rx_key);
        Ok((packet, addr))
    }

    async fn recv_many_from(
        &mut self,
        recv_buf: &mut Self::RecvManyBuf,
        pool: &mut PacketBufPool,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        // The trait contract appends to `packets`; only deobfuscate the entries the inner call
        // actually added so we don't touch packets the caller already owned.
        let start = packets.len();
        self.inner.recv_many_from(recv_buf, pool, packets).await?;
        deobfuscate_all(&mut packets[start..], &self.rx_key);
        Ok(())
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        self.inner.enable_udp_gro()
    }
}

/// Apply LWO obfuscation to every packet in `packets`.
fn obfuscate_all(packets: &mut [(Packet, SocketAddr)], tx_key: &[u8; 32]) {
    for (packet, _) in packets.iter_mut() {
        lwo::obfuscate_thread_local(packet, tx_key);
    }
}

/// Apply LWO deobfuscation to every packet in `packets`.
fn deobfuscate_all(packets: &mut [(Packet, SocketAddr)], rx_key: &[u8; 32]) {
    for (packet, _) in packets.iter_mut() {
        lwo::deobfuscate(packet, rx_key);
    }
}

/// Extract LWO obfuscation keys from the tunnel config.
///
/// Returns `(tx_key, rx_key)` where:
/// - `tx_key` is the server public key (used to obfuscate outgoing packets)
/// - `rx_key` is the client public key (used to deobfuscate incoming packets)
fn lwo_keys_from_config(config: &Config) -> Option<([u8; 32], [u8; 32])> {
    match &config.obfuscator_config {
        Some(Obfuscators::Single(ObfuscatorConfig::Lwo { .. })) => {
            let tx_key = *config.entry_peer.public_key.as_bytes();
            let rx_key = *config.tunnel.private_key.public_key().as_bytes();
            Some((tx_key, rx_key))
        }
        _ => None,
    }
}

/// A [`UdpTransportFactory`] that wraps another factory and applies LWO obfuscation inline.
///
/// * `tx_key` - server public key bytes, used to obfuscate outgoing packets.
/// * `rx_key` - client public key bytes, used to deobfuscate incoming packets.
pub struct LwoUdpTransportFactory<F: UdpTransportFactory> {
    inner: F,
    tx_key: [u8; 32],
    rx_key: [u8; 32],
}

impl<F: UdpTransportFactory> UdpTransportFactory for LwoUdpTransportFactory<F> {
    type SendV4 = LwoSend<F::SendV4>;
    type SendV6 = LwoSend<F::SendV6>;
    type RecvV4 = LwoRecv<F::RecvV4>;
    type RecvV6 = LwoRecv<F::RecvV6>;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<((Self::SendV4, Self::RecvV4), (Self::SendV6, Self::RecvV6))> {
        let ((send_v4, recv_v4), (send_v6, recv_v6)) = self.inner.bind(params).await?;
        Ok((
            (
                LwoSend {
                    inner: send_v4,
                    tx_key: self.tx_key,
                },
                LwoRecv {
                    inner: recv_v4,
                    rx_key: self.rx_key,
                },
            ),
            (
                LwoSend {
                    inner: send_v6,
                    tx_key: self.tx_key,
                },
                LwoRecv {
                    inner: recv_v6,
                    rx_key: self.rx_key,
                },
            ),
        ))
    }
}
