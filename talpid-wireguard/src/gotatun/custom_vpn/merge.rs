//! Merge two [`IpRecv`]s into one.

use either::Either;
use gotatun::{
    packet::{Ip, Packet, PacketBufPool},
    tun::{IpRecv, MtuWatcher},
};
use std::io;

/// Merges the packets of two [`IpRecv`]s. The MTU of the first one is used as the merged MTU.
pub struct MergingIpRecv<A, B> {
    a: A,
    b: B,
    /// Packet buffer pool used when receiving from `b`.
    pool_b: PacketBufPool,
}

impl<A: IpRecv, B: IpRecv> MergingIpRecv<A, B> {
    /// Merge the packets of `a` and `b`. `pool_b` is used when receiving from `b`.
    pub fn new(a: A, b: B, pool_b: PacketBufPool) -> Self {
        Self { a, b, pool_b }
    }
}

impl<A: IpRecv, B: IpRecv> IpRecv for MergingIpRecv<A, B> {
    async fn recv<'a>(
        &'a mut self,
        pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        let Self { a, b, pool_b } = self;
        tokio::select! {
            result = a.recv(pool) => result.map(Either::Left),
            result = b.recv(pool_b) => result.map(Either::Right),
        }
    }

    fn mtu(&self) -> MtuWatcher {
        self.a.mtu()
    }
}
