//! Drop packets read from the TUN device that don't come from the given addresses.
//!
//! Packets can show up on the TUN device with a source address that is unexpected, notably the
//! address of a previous tunnel session. Whenever the tunnel address is rotated (on login to
//! another account, or on key rotation), sockets that were connected over the old tunnel keep
//! sending packets with the old source address:
//!
//! * When an IPv6 socket is connected, the Linux kernel picks a source address once, based
//!   on the route to the destination, i.e. an address on the tunnel interface. If that interface
//!   (or just the route) goes away and a new tunnel comes up, packets for that destination are
//!   routed out the new interface but still carry the old source address.
//! * IPv4 sockets don't have this problem: the kernel refuses to send at all if the source address
//!   doesn't exist on any interface.

use gotatun::{
    packet::{Ip, Packet, PacketBufPool},
    tun::{IpRecv, MtuWatcher},
};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

/// Wraps an [`IpRecv`] and drops any packet whose source address isn't one of our tunnel
/// addresses. Packets that we can't read a source address from at all are dropped as well.
pub struct SourceFilter<R> {
    inner: R,

    /// The only IPv4 source address accepted from `inner`. All IPv4 packets are dropped if `None`.
    v4: Option<Ipv4Addr>,

    /// The only IPv6 source address accepted from `inner`. All IPv6 packets are dropped if `None`.
    v6: Option<Ipv6Addr>,
}

impl<R: IpRecv> SourceFilter<R> {
    /// Create a filter that only accepts packets from the tunnel addresses `v4` and `v6`.
    ///
    /// If either address is `None`, all packets of that family are dropped.
    pub fn new(inner: R, v4: Option<Ipv4Addr>, v6: Option<Ipv6Addr>) -> Self {
        Self { inner, v4, v6 }
    }
}

impl<R: IpRecv> IpRecv for SourceFilter<R> {
    async fn recv<'a>(
        &'a mut self,
        pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        let (v4, v6) = (self.v4, self.v6);
        let packets = self.inner.recv(pool).await?;

        Ok(
            packets.filter(move |packet: &Packet<Ip>| match packet.source() {
                Some(IpAddr::V4(source)) => v4 == Some(source),
                Some(IpAddr::V6(source)) => v6 == Some(source),
                None => false,
            }),
        )
    }

    fn mtu(&self) -> MtuWatcher {
        self.inner.mtu()
    }
}
