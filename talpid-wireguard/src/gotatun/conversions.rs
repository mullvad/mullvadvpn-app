//! Conversions between [`gotatun`]-types and `talpid_wireguard`-types.

use std::time::SystemTime;

use crate::stats::{DaitaStats, Stats};

impl From<gotatun::device::configure::Stats> for Stats {
    fn from(peer_stats: gotatun::device::configure::Stats) -> Self {
        let daita = peer_stats.daita.as_ref().map(DaitaStats::from);

        let last_handshake_time = peer_stats
            .last_handshake
            .map(|duration_since| SystemTime::now() - duration_since);

        let stats = Stats {
            tx_bytes: peer_stats.tx_bytes as u64,
            rx_bytes: peer_stats.rx_bytes as u64,
            last_handshake_time,
            daita,
        };

        stats
    }
}

impl From<&gotatun::device::configure::DaitaStats> for DaitaStats {
    fn from(daita_stats: &gotatun::device::configure::DaitaStats) -> Self {
        Self {
            tx_padding_bytes: daita_stats.tx_padding_bytes as u64,
            tx_padding_packet_bytes: daita_stats.tx_padding_packet_bytes as u64,
            rx_padding_bytes: daita_stats.rx_padding_bytes as u64,
            rx_padding_packet_bytes: daita_stats.rx_padding_packet_bytes as u64,
        }
    }
}
