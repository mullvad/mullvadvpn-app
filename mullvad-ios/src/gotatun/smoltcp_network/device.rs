use bytes::BytesMut;
use gotatun::packet::{Ip, Packet};
use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use std::collections::VecDeque;
use tokio::sync::mpsc;

/// Maximum number of inbound packets buffered before tail-drop.
const MAX_QUEUE_DEPTH: usize = 512;

/// A virtual smoltcp device.
///
/// Inbound packets are staged in `rx_queue` and delivered to the stack on the next poll.
///
/// Outbound packets are handed sent on a blocking channel as soon as `smoltcp` creates them.
pub struct SmoltcpDevice {
    rx_queue: VecDeque<Packet<[u8]>>,
    tx: mpsc::Sender<Packet<Ip>>,
    mtu: usize,
}

impl SmoltcpDevice {
    pub fn new(mtu: u16, tx: mpsc::Sender<Packet<Ip>>) -> Self {
        Self {
            rx_queue: VecDeque::with_capacity(64),
            tx,
            mtu: mtu as usize,
        }
    }

    /// Enqueue an IP packet for smoltcp to process on the next poll.
    /// Drops the packet if the queue is full.
    pub fn enqueue_rx(&mut self, packet: Packet<[u8]>) {
        if self.rx_queue.len() >= MAX_QUEUE_DEPTH {
            log::warn!("smoltcp rx queue full, dropping packet");
            return;
        }
        self.rx_queue.push_back(packet);
    }
}

pub struct SmoltcpRxToken {
    packet: Packet<[u8]>,
}

impl phy::RxToken for SmoltcpRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.packet)
    }
}

pub struct SmoltcpTxToken<'a> {
    tx: &'a mpsc::Sender<Packet<Ip>>,
}

impl phy::TxToken for SmoltcpTxToken<'_> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = BytesMut::zeroed(len);
        let result = f(&mut buf);

        match Packet::from_bytes(buf).try_into_ip() {
            Ok(ip) => {
                if self.tx.try_send(ip).is_err() {
                    log::warn!("smoltcp: to_gotatun channel full or closed, dropping packet");
                }
            }
            Err(err) => log::error!("smoltcp emitted an unparseable IP packet, dropping: {err}"),
        }

        result
    }
}

impl Device for SmoltcpDevice {
    type RxToken<'a> = SmoltcpRxToken;
    type TxToken<'a> = SmoltcpTxToken<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let packet = self.rx_queue.pop_front()?;
        Some((SmoltcpRxToken { packet }, SmoltcpTxToken { tx: &self.tx }))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(SmoltcpTxToken { tx: &self.tx })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ip;
        caps.max_transmission_unit = self.mtu;
        caps
    }
}
