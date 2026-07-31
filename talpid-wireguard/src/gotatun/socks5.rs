//! A SOCKS5 transport for GotaTun's UDP socket, using `UDP ASSOCIATE`.
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
use talpid_net::bypass::{BypassGuard, BypassSocket, SocketBypass};
use talpid_types::net::proxy::Socks5Proxy;
use tokio::net::TcpSocket;
use tunnel_obfuscation::socks5::{self, Socks5UdpAssociation};

/// A [`UdpSend`] that optionally relays datagrams through a SOCKS5 proxy.
#[derive(Clone)]
pub enum MaybeSocks5Send<S: UdpSend> {
    Plain(S),
    Relayed {
        inner: S,
        association: Arc<Socks5Association>,
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
        }
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        match self {
            Self::Plain(inner) | Self::Relayed(inner) => inner.enable_udp_gro(),
        }
    }
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

/// An established UDP association, together with the bypass of its control connection.
///
/// The proxy tears the association down when the control connection closes, so the connection is
/// held for as long as datagrams are relayed over it.
pub struct Socks5Association {
    association: Socks5UdpAssociation,
    /// Keeps the control connection excluded from the tunnel for as long as it is open.
    _bypass: BypassGuard,
}

impl Socks5Association {
    fn relay_endpoint(&self) -> SocketAddr {
        self.association.relay_endpoint()
    }
}

/// A [`UdpTransportFactory`] that optionally relays another factory's socket through a SOCKS5
/// proxy.
pub struct MaybeSocks5TransportFactory<F> {
    inner: F,
    proxy: Option<Socks5Proxy>,
    bypass: Arc<dyn SocketBypass>,
}

impl<F> MaybeSocks5TransportFactory<F> {
    pub fn new(inner: F, proxy: Option<Socks5Proxy>, bypass: Arc<dyn SocketBypass>) -> Self {
        Self {
            inner,
            proxy,
            bypass,
        }
    }

    /// Open the control connection and establish the UDP association.
    ///
    /// The control connection is excluded from the tunnel the same way the UDP socket is, so that
    /// it does not try to route itself through the tunnel it is setting up. The exclusion is
    /// applied before connecting, since the route is picked at that point.
    async fn associate(&self, proxy: &Socks5Proxy) -> io::Result<Socks5Association> {
        let proxy_addr = proxy.proxy_addr();
        log::debug!("Associating with SOCKS5 proxy at {proxy_addr}");

        let socket = match proxy_addr {
            SocketAddr::V4(_) => TcpSocket::new_v4()?,
            SocketAddr::V6(_) => TcpSocket::new_v6()?,
        };
        let BypassSocket {
            socket,
            guard: _bypass,
        } = BypassSocket::new(Arc::clone(&self.bypass), socket)?;

        let control = socket.connect(proxy_addr).await?;

        let association = Socks5UdpAssociation::associate(control, proxy.auth())
            .await
            .map_err(io::Error::other)?;

        Ok(Socks5Association {
            association,
            _bypass,
        })
    }
}

impl<F: UdpTransportFactory> UdpTransportFactory for MaybeSocks5TransportFactory<F> {
    type Send = MaybeSocks5Send<F::Send>;
    type Recv = MaybeSocks5Recv<F::Recv>;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        let (send, recv) = self.inner.bind(params).await?;

        let Some(proxy) = self.proxy.clone() else {
            return Ok((MaybeSocks5Send::Plain(send), MaybeSocks5Recv::Plain(recv)));
        };

        let association = Arc::new(self.associate(&proxy).await?);
        log::debug!(
            "Relaying WireGuard through SOCKS5 proxy, via {}",
            association.relay_endpoint()
        );

        Ok((
            MaybeSocks5Send::Relayed {
                inner: send,
                association,
            },
            MaybeSocks5Recv::Relayed(recv),
        ))
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
