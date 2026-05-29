//! The virtual smoltcp device backing the utun interface.
//!
//! Ingress is staged until the next poll; egress goes on a bounded channel drained by the tunnel
//! writer, which keeps the packets in the order smoltcp emitted them.

use bytes::{Bytes, BytesMut};
use smoltcp::{
    phy::{self, Device, DeviceCapabilities, Medium},
    time::Instant as SmoltcpInstant,
};
use std::collections::VecDeque;
use tokio::sync::mpsc;

use super::DEVICE_RX_QUEUE_DEPTH;

pub struct SmoltcpDevice {
    rx_queue: VecDeque<Bytes>,
    tx: mpsc::Sender<Bytes>,
    mtu: u16,
}

impl SmoltcpDevice {
    pub fn new(mtu: u16, tx: mpsc::Sender<Bytes>) -> Self {
        Self {
            rx_queue: VecDeque::with_capacity(64),
            tx,
            mtu,
        }
    }

    /// Queue an IP packet for smoltcp to process on the next poll, dropping it if the queue is
    /// full.
    pub fn enqueue_rx(&mut self, packet: Bytes) {
        if self.rx_queue.len() >= DEVICE_RX_QUEUE_DEPTH {
            log::warn!("smoltcp rx queue full, dropping packet");
            return;
        }
        self.rx_queue.push_back(packet);
    }

    /// Whether any ingress packet is still waiting to be handed to smoltcp.
    pub fn has_queued_rx(&self) -> bool {
        !self.rx_queue.is_empty()
    }
}

pub struct SmoltcpRxToken {
    packet: Bytes,
}

impl phy::RxToken for SmoltcpRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.packet)
    }
}

/// Holds a reserved slot on the egress channel, so a token is only handed to smoltcp when the
/// packet it produces is guaranteed to fit.
pub struct SmoltcpTxToken<'a> {
    permit: mpsc::Permit<'a, Bytes>,
}

impl phy::TxToken for SmoltcpTxToken<'_> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = BytesMut::zeroed(len);
        let result = f(&mut buf);
        self.permit.send(buf.freeze());
        result
    }
}

impl Device for SmoltcpDevice {
    type RxToken<'a> = SmoltcpRxToken;

    type TxToken<'a> = SmoltcpTxToken<'a>;

    fn receive(
        &mut self,
        _timestamp: SmoltcpInstant,
    ) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        // Reserve the egress slot first: processing an ingress packet almost always produces at
        // least an ACK, and without room for it smoltcp would have to drop the reply.
        let permit = self.tx.try_reserve().ok()?;
        let packet = self.rx_queue.pop_front()?;

        Some((SmoltcpRxToken { packet }, SmoltcpTxToken { permit }))
    }

    fn transmit(&mut self, _timestamp: SmoltcpInstant) -> Option<Self::TxToken<'_>> {
        let permit = self.tx.try_reserve().ok()?;

        Some(SmoltcpTxToken { permit })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ip;
        caps.max_transmission_unit = self.mtu.into();
        caps
    }
}
