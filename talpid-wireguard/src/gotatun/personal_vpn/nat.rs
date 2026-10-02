//! Source/destination NAT adapters for [`IpRecv`] and [`IpSend`].
//!
//! [`NatIpRecv`] rewrites the source address on TUN read.
//! [`NatIpSend`] rewrites the destination address on TUN write.

use super::packet::{Direction, rewrite_address};
use gotatun::{
    packet::{Ip, Packet, PacketBufPool},
    tun::{IpRecv, IpSend, MtuWatcher},
};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

/// Address rewrites, at most one per address family.
#[derive(Default, Clone, Copy)]
struct Rewrites {
    v4: Option<(Ipv4Addr, Ipv4Addr)>,
    v6: Option<(Ipv6Addr, Ipv6Addr)>,
}

impl Rewrites {
    fn apply(&self, packet: &mut Packet<Ip>, direction: Direction) {
        let (from, to) = match packet.header.version() {
            4 => match self.v4 {
                Some((from, to)) => (IpAddr::from(from), IpAddr::from(to)),
                None => return,
            },
            6 => match self.v6 {
                Some((from, to)) => (IpAddr::from(from), IpAddr::from(to)),
                None => return,
            },
            _ => return,
        };
        rewrite_address(packet, direction, from, to);
    }
}

/// Rewrites the source address of packets read from `inner` (outbound NAT).
pub struct NatIpRecv<R> {
    inner: R,
    rewrites: Rewrites,
}

/// Rewrites the destination address of packets sent to `inner` (inbound NAT).
pub struct NatIpSend<S> {
    inner: S,
    rewrites: Rewrites,
}

impl<R: IpRecv> NatIpRecv<R> {
    /// Wrap `inner` without rewriting anything.
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            rewrites: Rewrites::default(),
        }
    }

    /// Rewrite packets whose source IPv4 address is `from` to use `to` instead.
    pub fn with_v4(mut self, from: Ipv4Addr, to: Ipv4Addr) -> Self {
        self.rewrites.v4 = Some((from, to));
        self
    }

    /// Rewrite packets whose source IPv6 address is `from` to use `to` instead.
    pub fn with_v6(mut self, from: Ipv6Addr, to: Ipv6Addr) -> Self {
        self.rewrites.v6 = Some((from, to));
        self
    }
}

impl<S: IpSend> NatIpSend<S> {
    /// Wrap `inner` without rewriting anything.
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            rewrites: Rewrites::default(),
        }
    }

    /// Rewrite packets whose destination IPv4 address is `from` to use `to` instead.
    pub fn with_v4(mut self, from: Ipv4Addr, to: Ipv4Addr) -> Self {
        self.rewrites.v4 = Some((from, to));
        self
    }

    /// Rewrite packets whose destination IPv6 address is `from` to use `to` instead.
    pub fn with_v6(mut self, from: Ipv6Addr, to: Ipv6Addr) -> Self {
        self.rewrites.v6 = Some((from, to));
        self
    }
}

impl<R: IpRecv> IpRecv for NatIpRecv<R> {
    async fn recv<'a>(
        &'a mut self,
        pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        let rewrites = self.rewrites;
        let packets = self.inner.recv(pool).await?;
        Ok(packets.map(move |mut packet| {
            rewrites.apply(&mut packet, Direction::Source);
            packet
        }))
    }

    fn mtu(&self) -> MtuWatcher {
        self.inner.mtu()
    }
}

impl<S: IpSend> IpSend for NatIpSend<S> {
    async fn send(&mut self, mut packet: Packet<Ip>) -> io::Result<()> {
        self.rewrites.apply(&mut packet, Direction::Destination);
        self.inner.send(packet).await
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        packet::tests::{checksums_are_valid, udp_packet},
        test_util::{ChannelIpRecv, ChannelIpSend},
    };
    use super::*;
    use std::net::SocketAddr;

    const OUTER_V4: Ipv4Addr = Ipv4Addr::new(10, 64, 0, 2);
    const INNER_V4: Ipv4Addr = Ipv4Addr::new(172, 16, 0, 1);
    const OUTER_V6: Ipv6Addr = Ipv6Addr::new(0xfc00, 0xbbbb, 0xbbbb, 0xbb01, 0, 0, 0, 2);
    const INNER_V6: Ipv6Addr = Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, 0, 2);
    const REMOTE_V4: Ipv4Addr = Ipv4Addr::new(10, 100, 0, 5);
    const REMOTE_V6: Ipv6Addr = Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, 0, 5);

    fn packet(src: IpAddr, dst: IpAddr) -> Packet<Ip> {
        udp_packet(
            SocketAddr::new(src, 1234),
            SocketAddr::new(dst, 5678),
            b"hello",
        )
    }

    #[tokio::test]
    async fn recv_rewrites_source() {
        let (tx, recv) = ChannelIpRecv::new();
        let mut recv = NatIpRecv::new(recv)
            .with_v4(OUTER_V4, INNER_V4)
            .with_v6(OUTER_V6, INNER_V6);
        let v4 = |addr: Ipv4Addr| IpAddr::from(addr);
        let v6 = |addr: Ipv6Addr| IpAddr::from(addr);
        for (src, dst, expected_src) in [
            (v4(OUTER_V4), v4(REMOTE_V4), v4(INNER_V4)),
            (v6(OUTER_V6), v6(REMOTE_V6), v6(INNER_V6)),
            // Other sources are left alone
            (v4(REMOTE_V4), v4(REMOTE_V4), v4(REMOTE_V4)),
        ] {
            tx.send(packet(src, dst)).await.unwrap();
            let received = recv.recv(&mut PacketBufPool::new(1)).await.unwrap().next();
            let received = received.unwrap();
            assert_eq!(received.source(), Some(expected_src));
            assert_eq!(received.destination(), Some(dst));
            assert!(checksums_are_valid(&received));
        }
    }

    #[tokio::test]
    async fn send_rewrites_destination() {
        let (send, mut rx) = ChannelIpSend::new();
        let mut send = NatIpSend::new(send).with_v4(INNER_V4, OUTER_V4);
        send.send(packet(REMOTE_V4.into(), INNER_V4.into()))
            .await
            .unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.destination(), Some(OUTER_V4.into()));
        assert!(checksums_are_valid(&received));

        // IPv6 is not rewritten, since no IPv6 rewrite was configured
        send.send(packet(REMOTE_V6.into(), INNER_V6.into()))
            .await
            .unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.destination(), Some(INNER_V6.into()));
    }
}
