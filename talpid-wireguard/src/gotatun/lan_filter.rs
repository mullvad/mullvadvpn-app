//! Drop packets read from the TUN device that are destined for the local network, except for
//! [`ALLOWED_IN_TUNNEL_LAN_NETS`].

use gotatun::{
    packet::{Ip, Packet, PacketBufPool},
    tun::{IpRecv, MtuWatcher},
};
use std::{io, net::IpAddr};
use talpid_types::net::{ALLOWED_IN_TUNNEL_LAN_NETS, ALLOWED_LAN_NETS};

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
                Some(destination) => is_allowed_in_tunnel(destination),
                None => false,
            }),
        )
    }

    fn mtu(&self) -> MtuWatcher {
        self.inner.mtu()
    }
}

/// Whether `address` may be reached through the tunnel.
fn is_allowed_in_tunnel(address: IpAddr) -> bool {
    !ALLOWED_LAN_NETS.iter().any(|net| net.contains(address))
        || ALLOWED_IN_TUNNEL_LAN_NETS
            .iter()
            .any(|net| net.contains(address))
}
