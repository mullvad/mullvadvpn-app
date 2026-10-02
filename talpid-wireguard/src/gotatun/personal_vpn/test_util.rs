//! Channel-backed [`IpRecv`] and [`IpSend`] implementations for tests.

use gotatun::{
    packet::{Ip, Packet, PacketBufPool},
    tun::{IpRecv, IpSend, MtuWatcher},
};
use std::{io, iter};
use tokio::sync::{mpsc, watch};

const CHANNEL_CAPACITY: usize = 16;
const MTU: u16 = 1420;

/// An [`IpRecv`] that yields the packets sent to its channel.
pub struct ChannelIpRecv {
    rx: mpsc::Receiver<Packet<Ip>>,
    mtu: watch::Receiver<u16>,
}

impl ChannelIpRecv {
    pub fn new() -> (mpsc::Sender<Packet<Ip>>, Self) {
        let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
        let (_mtu_tx, mtu) = watch::channel(MTU);
        (tx, Self { rx, mtu })
    }
}

impl IpRecv for ChannelIpRecv {
    async fn recv<'a>(
        &'a mut self,
        _pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        let packet = self.rx.recv().await.ok_or(io::ErrorKind::UnexpectedEof)?;
        Ok(iter::once(packet))
    }

    fn mtu(&self) -> MtuWatcher {
        MtuWatcher::from(self.mtu.clone())
    }
}

/// An [`IpSend`] that sends all packets to its channel.
pub struct ChannelIpSend(mpsc::Sender<Packet<Ip>>);

impl ChannelIpSend {
    pub fn new() -> (Self, mpsc::Receiver<Packet<Ip>>) {
        let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
        (Self(tx), rx)
    }
}

impl IpSend for ChannelIpSend {
    async fn send(&mut self, packet: Packet<Ip>) -> io::Result<()> {
        self.0
            .send(packet)
            .await
            .map_err(|_| io::ErrorKind::BrokenPipe.into())
    }
}
