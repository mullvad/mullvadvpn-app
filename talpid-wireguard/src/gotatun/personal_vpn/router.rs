//! Split one [`IpRecv`] into multiple receivers based on destination address, or combine two
//! [`IpSend`]s into one based on source address.
//!
//! See [`TunRxRouter`] and [`TunTxRouter`].

use super::packet::UdpPacket;
use gotatun::{
    packet::{Ip, Ipv4, Packet, PacketBufPool},
    tun::{IpRecv, IpSend, MtuWatcher, channel::Ipv4Fragments},
};
use ip_network_table::IpNetworkTable;
use ipnetwork::IpNetwork;
use std::{
    io, iter,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};
use tokio::sync::mpsc;
use zerocopy::{FromBytes, IntoBytes};

/// An [`IpRecv`] that receives some subset of the traffic of a [`TunRxRouter`].
pub struct SplitIpRecv {
    rx: mpsc::Receiver<Packet<Ip>>,
    mtu: MtuWatcher,
}

impl IpRecv for SplitIpRecv {
    async fn recv<'a>(
        &'a mut self,
        _pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        match self.rx.recv().await {
            Some(packet) => Ok(iter::once(packet)),
            None => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "channel closed",
            )),
        }
    }

    fn mtu(&self) -> MtuWatcher {
        self.mtu.clone()
    }
}

/// Splits a single [`IpRecv`] into multiple streams based on destination address.
pub struct TunRxRouter<R> {
    source: R,
    table: IpNetworkTable<mpsc::Sender<Packet<Ip>>>,
}

impl<R: IpRecv> TunRxRouter<R> {
    /// Create a router that reads from `source`. Packets that match no route are dropped.
    pub fn new(source: R) -> Self {
        Self {
            source,
            table: IpNetworkTable::new(),
        }
    }

    /// Route all IPv4 and IPv6 traffic that matches no other route to the returned receiver.
    /// This replaces any previous default route.
    pub fn add_default_route(&mut self, channel_capacity: usize) -> SplitIpRecv {
        let default_routes = [
            IpNetwork::new(Ipv4Addr::UNSPECIFIED.into(), 0).unwrap(),
            IpNetwork::new(Ipv6Addr::UNSPECIFIED.into(), 0).unwrap(),
        ];
        self.add_routes(&default_routes, channel_capacity)
    }

    /// Route traffic to any of `routes` to the returned receiver. This replaces any previous
    /// routes for the same networks.
    pub fn add_routes(&mut self, routes: &[IpNetwork], channel_capacity: usize) -> SplitIpRecv {
        let (tx, rx) = mpsc::channel(channel_capacity);
        for route in routes {
            self.table.insert(to_table_network(route), tx.clone());
        }
        SplitIpRecv {
            rx,
            mtu: self.source.mtu(),
        }
    }

    /// Forward packets from the source to the receivers, until the source or a receiver closes.
    pub async fn run(mut self, mut pool: PacketBufPool) {
        loop {
            let packets = match self.source.recv(&mut pool).await {
                Ok(packets) => packets,
                Err(error) => {
                    log::error!("TUN router recv error: {error}");
                    return;
                }
            };

            for packet in packets {
                let Some(destination) = packet.destination() else {
                    continue;
                };
                let Some((_net, tx)) = self.table.longest_match(destination) else {
                    continue;
                };
                if tx.send(packet).await.is_err() {
                    log::trace!("TUN router channel closed. Stopping");
                    return;
                }
            }
        }
    }
}

/// Combines two [`IpSend`]s: packets from the UDP socket `inner_tun_endpoint` are sent to
/// `inner_tx` (e.g. the UDP channel of an inner WireGuard device), and all other packets are sent
/// to `outer_tx` (e.g. the TUN device).
///
/// Packets whose source address is in `inner_allowed_ips` are dropped. This prevents the outer
/// tunnel's peer from forging packets that appear to come from inside the inner tunnel: such
/// packets must only ever reach the TUN device through the inner WireGuard device.
///
/// IPv4 fragments from the address of `inner_tun_endpoint` are reassembled before they are
/// routed, since only the first fragment has the UDP port that tells the two apart.
pub struct TunTxRouter<Inner, Outer> {
    inner_tx: Inner,
    outer_tx: Outer,
    inner_tun_endpoint: SocketAddr,
    inner_allowed_ips: IpNetworkTable<()>,
    endpoint_fragments: Ipv4Fragments,
}

impl<Inner: IpSend, Outer: IpSend> TunTxRouter<Inner, Outer> {
    /// Create a `TunTxRouter`.
    ///
    /// `inner_allowed_ips` are the source addresses that the inner tunnel is authoritative for.
    /// Pass an empty slice only if there is no inner tunnel, and thus nothing to spoof.
    pub fn new(
        inner_tx: Inner,
        outer_tx: Outer,
        inner_tun_endpoint: SocketAddr,
        inner_allowed_ips: &[IpNetwork],
    ) -> Self {
        let mut table = IpNetworkTable::new();
        for net in inner_allowed_ips {
            table.insert(to_table_network(net), ());
        }
        Self {
            inner_tx,
            outer_tx,
            inner_tun_endpoint,
            inner_allowed_ips: table,
            endpoint_fragments: Ipv4Fragments::default(),
        }
    }

    /// Reassemble `packet` if it is an IPv4 fragment from the inner tunnel endpoint's address.
    /// Returns `None` while fragments are missing, and any other packet unchanged.
    ///
    /// A reassembled datagram that turns out not to be for the inner device goes to `outer_tx`
    /// whole, and may exceed its MTU.
    fn reassemble_endpoint_fragment(&mut self, packet: Packet<Ip>) -> Option<Packet<Ip>> {
        let IpAddr::V4(endpoint_ip) = self.inner_tun_endpoint.ip() else {
            return Some(packet);
        };
        let is_endpoint_fragment =
            Ipv4::<[u8]>::ref_from_bytes(packet.as_bytes()).is_ok_and(|ipv4| {
                ipv4.header.source() == endpoint_ip
                    && (ipv4.header.more_fragments() || ipv4.header.fragment_offset() != 0)
            });
        if !is_endpoint_fragment {
            return Some(packet);
        }
        let fragment = Packet::<Ipv4>::try_from(packet).ok()?;
        self.endpoint_fragments
            .assemble_ipv4_fragment(fragment)
            .map(Packet::from)
    }
}

impl<Inner: IpSend, Outer: IpSend> IpSend for TunTxRouter<Inner, Outer> {
    async fn send(&mut self, packet: Packet<Ip>) -> io::Result<()> {
        if let Some(source) = packet.source()
            && self.inner_allowed_ips.longest_match(source).is_some()
        {
            log::trace!(
                "Dropping decapsulated outer tunnel packet from {source}, which is inside the \
                 inner tunnel's allowed IPs"
            );
            return Ok(());
        }
        let Some(packet) = self.reassemble_endpoint_fragment(packet) else {
            return Ok(());
        };
        if let Some(udp) = UdpPacket::parse(&packet)
            && SocketAddr::new(udp.source, udp.source_port()) == self.inner_tun_endpoint
        {
            return self.inner_tx.send(packet).await;
        }
        self.outer_tx.send(packet).await
    }
}

fn to_table_network(net: &IpNetwork) -> ip_network::IpNetwork {
    ip_network::IpNetwork::new_truncate(net.ip(), net.prefix()).expect("prefix is valid")
}

#[cfg(test)]
mod tests {
    use super::super::{
        packet::tests::udp_packet,
        test_util::{ChannelIpRecv, ChannelIpSend},
    };
    use super::*;
    use gotatun::packet::{IpNextProtocol, Ipv4FlagsFragmentOffset, Ipv4Header};

    fn packet(src: &str, dst: &str) -> Packet<Ip> {
        udp_packet(src.parse().unwrap(), dst.parse().unwrap(), b"hello")
    }

    #[tokio::test]
    async fn rx_router_routes_by_destination() {
        let (tx, source) = ChannelIpRecv::new();
        let mut router = TunRxRouter::new(source);
        let mut default_recv = router.add_default_route(10);
        let mut inner_recv = router.add_routes(&["10.100.0.0/24".parse().unwrap()], 10);
        tokio::spawn(router.run(PacketBufPool::new(10)));

        let mut pool = PacketBufPool::new(1);
        for (src, dst, to_inner) in [
            ("10.64.0.2:1234", "10.100.0.1:53", true),
            ("10.64.0.2:1234", "8.8.8.8:53", false),
            ("[fc00::2]:1234", "[2001:db8::1]:53", false),
        ] {
            tx.send(packet(src, dst)).await.unwrap();
            let recv = if to_inner {
                &mut inner_recv
            } else {
                &mut default_recv
            };
            let received = recv.recv(&mut pool).await.unwrap().next().unwrap();
            let dst: SocketAddr = dst.parse().unwrap();
            assert_eq!(received.destination(), Some(dst.ip()));
        }
    }

    #[tokio::test]
    async fn tx_router_routes_by_source() {
        let inner_endpoint: SocketAddr = "203.0.113.1:51820".parse().unwrap();
        let (inner_tx, mut inner_rx) = ChannelIpSend::new();
        let (outer_tx, mut outer_rx) = ChannelIpSend::new();
        let mut router = TunTxRouter::new(inner_tx, outer_tx, inner_endpoint, &[]);

        router
            .send(packet("203.0.113.1:51820", "10.64.0.2:1234"))
            .await
            .unwrap();
        assert!(inner_rx.try_recv().is_ok());
        assert!(outer_rx.try_recv().is_err());

        router
            .send(packet("203.0.113.1:443", "10.64.0.2:1234"))
            .await
            .unwrap();
        assert!(inner_rx.try_recv().is_err());
        assert!(outer_rx.try_recv().is_ok());
    }

    /// Split an IPv4 packet into two fragments, the first carrying `first_len` bytes of payload.
    fn fragment_v4(packet: &Packet<Ip>, first_len: usize) -> [Packet<Ip>; 2] {
        let ipv4 = Ipv4::<[u8]>::ref_from_bytes(packet.as_bytes()).unwrap();
        let (first, rest) = ipv4.payload.split_at(first_len);
        [(first, 0, true), (rest, first_len / 8, false)].map(|(payload, offset, more)| {
            let mut header = Ipv4Header::new_for_length(
                ipv4.header.source(),
                ipv4.header.destination(),
                IpNextProtocol::Udp,
                u16::try_from(payload.len()).unwrap(),
            );
            header.identification = 1.into();
            let mut flags = Ipv4FlagsFragmentOffset::new();
            flags.set_more_fragments(more);
            flags.set_fragment_offset(u16::try_from(offset).unwrap());
            header.flags_and_fragment_offset = flags;
            header.header_checksum = header.compute_checksum().into();
            let bytes = [header.as_bytes(), payload].concat();
            Packet::copy_from(&bytes[..]).try_into_ip().unwrap()
        })
    }

    /// Only the first fragment of a datagram has its UDP port, so fragments from the inner
    /// tunnel endpoint must be reassembled before they can be routed.
    #[tokio::test]
    async fn tx_router_reassembles_endpoint_fragments() {
        let inner_endpoint: SocketAddr = "203.0.113.1:51820".parse().unwrap();
        let (inner_tx, mut inner_rx) = ChannelIpSend::new();
        let (outer_tx, mut outer_rx) = ChannelIpSend::new();
        let mut router = TunTxRouter::new(inner_tx, outer_tx, inner_endpoint, &[]);

        let datagram = udp_packet(
            inner_endpoint,
            "10.64.0.2:1234".parse().unwrap(),
            &[0xab; 100],
        );
        for fragment in fragment_v4(&datagram, 64) {
            router.send(fragment).await.unwrap();
        }

        let received = inner_rx
            .try_recv()
            .expect("datagram must reach the inner device");
        assert_eq!(received.as_bytes().len(), datagram.as_bytes().len());
        assert_eq!(
            UdpPacket::parse(&received).unwrap().udp.payload,
            [0xab; 100]
        );
        assert!(inner_rx.try_recv().is_err());
        assert!(outer_rx.try_recv().is_err());

        // Fragments from another address are left for the OS to reassemble
        let other = udp_packet(
            "198.51.100.1:51820".parse().unwrap(),
            "10.64.0.2:1234".parse().unwrap(),
            &[0xab; 100],
        );
        for fragment in fragment_v4(&other, 64) {
            router.send(fragment).await.unwrap();
        }
        assert!(outer_rx.try_recv().is_ok());
        assert!(outer_rx.try_recv().is_ok());
        assert!(inner_rx.try_recv().is_err());
    }

    /// A malicious outer tunnel peer may send packets whose source address is inside the inner
    /// tunnel's allowed IPs. Those must not reach the TUN device, or the outer peer could
    /// impersonate any host behind the inner tunnel.
    #[tokio::test]
    async fn tx_router_blocks_spoofed_inner_source() {
        let inner_endpoint: SocketAddr = "203.0.113.1:51820".parse().unwrap();
        let (inner_tx, mut inner_rx) = ChannelIpSend::new();
        let (outer_tx, mut outer_rx) = ChannelIpSend::new();
        let mut router = TunTxRouter::new(
            inner_tx,
            outer_tx,
            inner_endpoint,
            &["10.100.0.0/24".parse().unwrap()],
        );

        router
            .send(packet("10.100.0.5:53", "10.64.0.2:1234"))
            .await
            .unwrap();
        assert!(
            outer_rx.try_recv().is_err(),
            "spoofed packet reached the TUN"
        );
        assert!(inner_rx.try_recv().is_err());

        router
            .send(packet("8.8.8.8:53", "10.64.0.2:1234"))
            .await
            .unwrap();
        let received = outer_rx.try_recv().expect("legitimate traffic must pass");
        assert_eq!(received.source(), Some(IpAddr::from([8, 8, 8, 8])));
    }
}
