use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use std::collections::VecDeque;

/// A virtual smoltcp device backed by in-memory queues.
///
/// Packets enqueued into `rx_queue` are delivered to the smoltcp stack on the
/// next poll. Packets transmitted by smoltcp are collected in `tx_queue`.
pub struct SmoltcpDevice {
    rx_queue: VecDeque<Vec<u8>>,
    tx_queue: VecDeque<Vec<u8>>,
    mtu: usize,
}

impl SmoltcpDevice {
    pub fn new(mtu: u16) -> Self {
        Self {
            rx_queue: VecDeque::with_capacity(64),
            tx_queue: VecDeque::with_capacity(64),
            mtu: mtu as usize,
        }
    }

    /// Enqueue a raw IP packet for smoltcp to process on the next poll.
    pub fn enqueue_rx(&mut self, packet: Vec<u8>) {
        self.rx_queue.push_back(packet);
    }

    /// Drain all transmitted packets from smoltcp.
    pub fn drain_tx(&mut self) -> impl Iterator<Item = Vec<u8>> + '_ {
        self.tx_queue.drain(..)
    }
}

pub struct SmoltcpRxToken {
    buf: Vec<u8>,
}

impl phy::RxToken for SmoltcpRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.buf)
    }
}

pub struct SmoltcpTxToken<'a> {
    queue: &'a mut VecDeque<Vec<u8>>,
}

impl phy::TxToken for SmoltcpTxToken<'_> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = vec![0u8; len];
        let result = f(&mut buf);
        self.queue.push_back(buf);
        result
    }
}

impl Device for SmoltcpDevice {
    type RxToken<'a> = SmoltcpRxToken;
    type TxToken<'a> = SmoltcpTxToken<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let buf = self.rx_queue.pop_front()?;
        Some((
            SmoltcpRxToken { buf },
            SmoltcpTxToken {
                queue: &mut self.tx_queue,
            },
        ))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(SmoltcpTxToken {
            queue: &mut self.tx_queue,
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ip;
        caps.max_transmission_unit = self.mtu;
        caps
    }
}
