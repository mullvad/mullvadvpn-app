use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub struct Stats {
    rx_packets: AtomicUsize,
    tx_packets: AtomicUsize,

    rx_bytes: AtomicUsize,
    tx_bytes: AtomicUsize,

    fragmented_tx_bytes: AtomicUsize,
    fragmented_rx_bytes: AtomicUsize,

    fragmented_tx_packets: AtomicUsize,
    fragmented_rx_packets: AtomicUsize,
}

const ORD: Ordering = Ordering::Relaxed;

impl Drop for Stats {
    fn drop(&mut self) {
        log::debug!("stats: {:?}", self);
    }
}

impl Stats {
    pub fn tx(&self, packet_len: usize, is_fragment: bool) {
        self.tx_packets.fetch_add(1, ORD);
        self.tx_bytes.fetch_add(packet_len, ORD);

        if is_fragment {
            self.fragmented_tx_packets.fetch_add(1, ORD);
            self.fragmented_tx_bytes.fetch_add(packet_len, ORD);
        }
    }

    pub fn rx(&self, packet_len: usize, is_fragment: bool) {
        self.rx_packets.fetch_add(1, ORD);
        self.rx_bytes.fetch_add(packet_len, ORD);

        if is_fragment {
            self.fragmented_rx_packets.fetch_add(1, ORD);
            self.fragmented_rx_bytes.fetch_add(packet_len, ORD);
        }
    }
}
