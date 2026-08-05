//! A SOCKS5 transport for GotaTun's UDP sockets, using `UDP ASSOCIATE`.
//!
//! This is not obfuscation. It sits *underneath* the obfuscation layer, so that the bytes handed
//! to the proxy are whatever the layer above produced. See [`super::obfuscation`].
//!
//! Every outgoing datagram gets a SOCKS5 header prepended and is redirected to the relay address
//! the proxy handed out; incoming datagrams have their header stripped, and report the address the
//! header says they came from, so that WireGuard's endpoint roaming keeps pointing at the real
//! peer rather than at the proxy.

use std::{io, net::SocketAddr, sync::Arc};

use bytes::Buf;
use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
use talpid_types::net::proxy::Socks5Proxy;
use tokio::net::TcpStream;
use tunnel_obfuscation::socks5::{self, Socks5UdpAssociation};

use crate::config::Config;

#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider::Tun;

/// A [`UdpSend`] that optionally relays datagrams through a SOCKS5 proxy.
#[derive(Clone)]
pub enum MaybeSocks5Send<S: UdpSend> {
    Plain(S),
    Relayed {
        inner: S,
        association: Arc<Socks5UdpAssociation>,
    },
}

impl<S: UdpSend> UdpSend for MaybeSocks5Send<S> {
    type SendManyBuf = S::SendManyBuf;

    async fn send_to(&self, mut packet: Packet, destination: SocketAddr) -> io::Result<()> {
        match self {
            Self::Plain(inner) => inner.send_to(packet, destination).await,
            Self::Relayed { inner, association } => {
                prepend_header(&mut packet, destination);
                inner.send_to(packet, association.relay_endpoint()).await
            }
        }
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        self.inner().max_number_of_packets_to_send()
    }

    async fn send_many_to(
        &self,
        send_buf: &mut Self::SendManyBuf,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        if let Self::Relayed { association, .. } = self {
            for (packet, destination) in packets.iter_mut() {
                prepend_header(packet, *destination);
                *destination = association.relay_endpoint();
            }
        }
        self.inner().send_many_to(send_buf, packets).await
    }

    fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        self.inner().local_addr()
    }

    #[cfg(target_os = "linux")]
    fn set_fwmark(&self, mark: u32) -> io::Result<()> {
        self.inner().set_fwmark(mark)
    }
}

impl<S: UdpSend> MaybeSocks5Send<S> {
    fn inner(&self) -> &S {
        match self {
            Self::Plain(inner) | Self::Relayed { inner, .. } => inner,
        }
    }
}

/// A [`UdpRecv`] that optionally decapsulates datagrams relayed by a SOCKS5 proxy.
pub enum MaybeSocks5Recv<R: UdpRecv> {
    Plain(R),
    Relayed(R),
    /// The address family that the proxy is not reachable over. Nothing is ever sent from it, so
    /// nothing can arrive on it either.
    Unused,
}

impl<R: UdpRecv> UdpRecv for MaybeSocks5Recv<R> {
    type RecvManyBuf = R::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        match self {
            Self::Plain(inner) => inner.recv_from(pool).await,
            Self::Relayed(inner) => loop {
                let (mut packet, from) = inner.recv_from(pool).await?;
                if let Some(origin) = strip_header(&mut packet, from) {
                    return Ok((packet, origin));
                }
            },
            Self::Unused => Err(unused_address_family()),
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
            Self::Relayed(inner) => {
                // The trait contract appends to `packets`, so only touch what this call added.
                let start = packets.len();
                inner.recv_many_from(recv_buf, pool, packets).await?;

                let mut index = start;
                while index < packets.len() {
                    let (packet, from) = &mut packets[index];
                    match strip_header(packet, *from) {
                        Some(origin) => {
                            *from = origin;
                            index += 1;
                        }
                        None => drop(packets.remove(index)),
                    }
                }
                Ok(())
            }
            Self::Unused => Err(unused_address_family()),
        }
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        match self {
            Self::Plain(inner) | Self::Relayed(inner) => inner.enable_udp_gro(),
            Self::Unused => Ok(()),
        }
    }
}

/// Mirrors how GotaTun itself reports a UDP socket that cannot be used.
fn unused_address_family() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "The SOCKS5 proxy is not reachable over this address family",
    )
}

/// Prepend the SOCKS5 header for `destination` to `packet`, shifting the payload to make room.
///
/// Buffers from GotaTun's packet pool have no headroom, so this costs one copy of the payload per
/// datagram. An upstream API for reserving headroom in a [`Packet`] would remove it.
fn prepend_header(packet: &mut Packet, destination: SocketAddr) {
    let header_len = socks5::header_len(&destination);
    let buffer = packet.buf_mut();

    let payload_len = buffer.len();
    buffer.resize(payload_len + header_len, 0);
    buffer.copy_within(..payload_len, header_len);
    socks5::write_udp_header(destination, buffer);
}

/// Strip the SOCKS5 header from a relayed datagram, returning the address it originated from.
///
/// Returns `None` for a datagram that is not a well-formed relayed datagram, which the caller
/// should discard. `from` is only used for logging.
fn strip_header(packet: &mut Packet, from: SocketAddr) -> Option<SocketAddr> {
    let Some((origin, payload)) = socks5::decode_udp_datagram(packet) else {
        log::debug!("Discarding malformed datagram from proxy at {from}");
        return None;
    };

    // The payload is a suffix of the datagram, so this is where the header ends.
    let payload_offset = packet.len() - payload.len();
    packet.buf_mut().advance(payload_offset);

    Some(origin)
}

/// A [`UdpTransportFactory`] that optionally relays another factory's sockets through a SOCKS5
/// proxy.
pub struct MaybeSocks5TransportFactory<F: UdpTransportFactory> {
    inner: F,
    proxy: Option<Socks5Proxy>,
    #[cfg(target_os = "android")]
    tun: Arc<Tun>,
}

impl<F: UdpTransportFactory> MaybeSocks5TransportFactory<F> {
    /// Create a transport factory from the tunnel config.
    pub fn from_config(
        inner: F,
        config: &Config,
        #[cfg(target_os = "android")] tun: Arc<Tun>,
    ) -> Self {
        Self {
            inner,
            proxy: config.proxy.clone(),
            #[cfg(target_os = "android")]
            tun,
        }
    }

    /// Open the control connection and establish the UDP association.
    ///
    /// The control connection is excluded from the tunnel the same way the UDP sockets are, so
    /// that it does not try to route itself through the tunnel it is setting up.
    async fn associate(
        &self,
        proxy: &Socks5Proxy,
        #[cfg(target_os = "linux")] fwmark: Option<u32>,
    ) -> io::Result<Socks5UdpAssociation> {
        let proxy_addr = proxy.proxy_addr();
        log::debug!("Associating with SOCKS5 proxy at {proxy_addr}");

        let control = TcpStream::connect(proxy_addr).await?;

        #[cfg(target_os = "linux")]
        if let Some(fwmark) = fwmark {
            socket2::SockRef::from(&control).set_mark(fwmark)?;
        }
        #[cfg(target_os = "android")]
        {
            use std::os::fd::AsFd;
            self.tun
                .bypass(&control.as_fd())
                .map_err(io::Error::other)?;
        }

        Socks5UdpAssociation::associate(control, proxy.auth())
            .await
            .map_err(io::Error::other)
    }
}

impl<F: UdpTransportFactory> UdpTransportFactory for MaybeSocks5TransportFactory<F> {
    type SendV4 = MaybeSocks5Send<EitherSend<F::SendV4, F::SendV6>>;
    type SendV6 = MaybeSocks5Send<EitherSend<F::SendV4, F::SendV6>>;
    type RecvV4 = MaybeSocks5Recv<EitherRecv<F::RecvV4, F::RecvV6>>;
    type RecvV6 = MaybeSocks5Recv<EitherRecv<F::RecvV4, F::RecvV6>>;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<((Self::SendV4, Self::RecvV4), (Self::SendV6, Self::RecvV6))> {
        let ((send_v4, recv_v4), (send_v6, recv_v6)) = self.inner.bind(params).await?;

        let Some(proxy) = self.proxy.clone() else {
            return Ok((
                (
                    MaybeSocks5Send::Plain(EitherSend::V4(send_v4)),
                    MaybeSocks5Recv::Plain(EitherRecv::V4(recv_v4)),
                ),
                (
                    MaybeSocks5Send::Plain(EitherSend::V6(send_v6)),
                    MaybeSocks5Recv::Plain(EitherRecv::V6(recv_v6)),
                ),
            ));
        };

        // All traffic goes to the proxy, so only the socket of the proxy's own address family is
        // ever used. Hand the same sender to both slots, since GotaTun picks between them based
        // on the peer endpoint, which has no relation to where the proxy lives.
        let (send, recv) = match proxy.proxy_addr() {
            SocketAddr::V4(_) => (EitherSend::V4(send_v4), EitherRecv::V4(recv_v4)),
            SocketAddr::V6(_) => (EitherSend::V6(send_v6), EitherRecv::V6(recv_v6)),
        };

        let association = Arc::new(
            self.associate(
                &proxy,
                #[cfg(target_os = "linux")]
                params.fwmark,
            )
            .await?,
        );
        log::debug!(
            "Relaying WireGuard through SOCKS5 proxy, via {}",
            association.relay_endpoint()
        );

        let send = MaybeSocks5Send::Relayed {
            inner: send,
            association,
        };
        Ok((
            (send.clone(), MaybeSocks5Recv::Relayed(recv)),
            (send, MaybeSocks5Recv::Unused),
        ))
    }
}

/// A [`UdpSend`] that is one of two address families, chosen at runtime.
///
/// GotaTun's factory returns a separate sender per family, but a proxy is only reachable over one
/// of them, so both slots must be able to hold either.
#[derive(Clone)]
pub enum EitherSend<V4: UdpSend, V6: UdpSend> {
    V4(V4),
    V6(V6),
}

impl<V4: UdpSend, V6: UdpSend> UdpSend for EitherSend<V4, V6> {
    /// Both buffers are carried, since the variant is not known to the type system.
    type SendManyBuf = (V4::SendManyBuf, V6::SendManyBuf);

    async fn send_to(&self, packet: Packet, destination: SocketAddr) -> io::Result<()> {
        match self {
            Self::V4(inner) => inner.send_to(packet, destination).await,
            Self::V6(inner) => inner.send_to(packet, destination).await,
        }
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        match self {
            Self::V4(inner) => inner.max_number_of_packets_to_send(),
            Self::V6(inner) => inner.max_number_of_packets_to_send(),
        }
    }

    async fn send_many_to(
        &self,
        send_buf: &mut Self::SendManyBuf,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        match self {
            Self::V4(inner) => inner.send_many_to(&mut send_buf.0, packets).await,
            Self::V6(inner) => inner.send_many_to(&mut send_buf.1, packets).await,
        }
    }

    fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        match self {
            Self::V4(inner) => inner.local_addr(),
            Self::V6(inner) => inner.local_addr(),
        }
    }

    #[cfg(target_os = "linux")]
    fn set_fwmark(&self, mark: u32) -> io::Result<()> {
        match self {
            Self::V4(inner) => inner.set_fwmark(mark),
            Self::V6(inner) => inner.set_fwmark(mark),
        }
    }
}

/// The [`UdpRecv`] counterpart of [`EitherSend`].
pub enum EitherRecv<V4: UdpRecv, V6: UdpRecv> {
    V4(V4),
    V6(V6),
}

impl<V4: UdpRecv, V6: UdpRecv> UdpRecv for EitherRecv<V4, V6> {
    type RecvManyBuf = (V4::RecvManyBuf, V6::RecvManyBuf);

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        match self {
            Self::V4(inner) => inner.recv_from(pool).await,
            Self::V6(inner) => inner.recv_from(pool).await,
        }
    }

    async fn recv_many_from(
        &mut self,
        recv_buf: &mut Self::RecvManyBuf,
        pool: &mut PacketBufPool,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        match self {
            Self::V4(inner) => inner.recv_many_from(&mut recv_buf.0, pool, packets).await,
            Self::V6(inner) => inner.recv_many_from(&mut recv_buf.1, pool, packets).await,
        }
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        match self {
            Self::V4(inner) => inner.enable_udp_gro(),
            Self::V6(inner) => inner.enable_udp_gro(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Long enough to catch an off-by-one when the payload is shifted to make room.
    const PAYLOAD_LEN: usize = 1400;

    const IPV4_DESTINATION: &str = "192.0.2.1:51820";
    const IPV6_DESTINATION: &str = "[2001:db8::1]:51820";

    /// Build a packet from the pool, the way GotaTun hands them to the transport.
    fn pooled_packet(payload: &[u8]) -> Packet {
        let pool: PacketBufPool = PacketBufPool::new(1);
        let mut packet = pool.get();

        let buffer = packet.buf_mut();
        buffer.clear();
        buffer.extend_from_slice(payload);

        packet
    }

    fn payload() -> Vec<u8> {
        (0..PAYLOAD_LEN).map(|index| index as u8).collect()
    }

    /// Encapsulating must leave the payload byte-for-byte intact behind the header.
    #[test]
    fn prepended_header_precedes_an_intact_payload() {
        for destination in [IPV4_DESTINATION, IPV6_DESTINATION] {
            let destination: SocketAddr = destination.parse().unwrap();
            let payload = payload();

            let mut packet = pooled_packet(&payload);
            prepend_header(&mut packet, destination);

            let (parsed, encapsulated) = socks5::decode_udp_datagram(&packet).unwrap();
            assert_eq!(parsed, destination);
            assert_eq!(encapsulated, payload);
        }
    }

    #[test]
    fn stripping_undoes_prepending() {
        for destination in [IPV4_DESTINATION, IPV6_DESTINATION] {
            let destination: SocketAddr = destination.parse().unwrap();
            let payload = payload();

            let mut packet = pooled_packet(&payload);
            prepend_header(&mut packet, destination);
            let origin = strip_header(&mut packet, destination).unwrap();

            assert_eq!(origin, destination);
            assert_eq!(&packet[..], payload);
        }
    }

    /// A datagram that is not a well-formed relayed datagram must be discarded, not misparsed.
    #[test]
    fn malformed_datagrams_are_discarded() {
        let destination: SocketAddr = IPV4_DESTINATION.parse().unwrap();

        let mut too_short = pooled_packet(&[0, 0]);
        assert!(strip_header(&mut too_short, destination).is_none());

        let mut fragmented = pooled_packet(&payload());
        prepend_header(&mut fragmented, destination);
        fragmented.buf_mut()[2] = 1;
        assert!(strip_header(&mut fragmented, destination).is_none());
    }
}
