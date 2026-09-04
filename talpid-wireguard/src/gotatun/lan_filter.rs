//! Drop packets read from the TUN device that are destined for the local network, unless allowed by
//! [`is_ip_allowed_in_tunnel`].

use gotatun::{
    packet::{Ip, Packet, PacketBufPool},
    tun::{IpRecv, MtuWatcher},
};
use std::io;
use talpid_types::net::is_ip_allowed_in_tunnel;

/// Wraps an [`IpRecv`] and drops any packet destined for the local network. Packets that we can't
/// read a destination address from at all are dropped as well.
pub struct LanFilter<R> {
    inner: R,
}

impl<R: IpRecv> LanFilter<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R: IpRecv> IpRecv for LanFilter<R> {
    async fn recv<'a>(
        &'a mut self,
        pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        let packets = self.inner.recv(pool).await?;

        Ok(
            packets.filter(|packet: &Packet<Ip>| match packet.destination() {
                Some(destination) => is_ip_allowed_in_tunnel(destination),
                None => false,
            }),
        )
    }

    fn mtu(&self) -> MtuWatcher {
        self.inner.mtu()
    }
}
