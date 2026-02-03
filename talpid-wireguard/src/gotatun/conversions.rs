//! Conversions between [`gotatun`]-types and `talpid_wireguard`-types.

use std::time::SystemTime;

use gotatun::device::Peer;
use talpid_tunnel_config_client::DaitaSettings;
use talpid_types::net::wireguard::PeerConfig;

use crate::stats::{DaitaStats, Stats};

/// Convert a [`PeerConfig`] into a GotaTun [`Peer`].
pub fn to_gotatun_peer(peer: &PeerConfig, daita: Option<&DaitaSettings>) -> Peer {
    let PeerConfig {
        public_key,
        allowed_ips,
        endpoint,
        psk,
        constant_packet_size: _,
    } = peer.clone();

    let mut peer = Peer::new((*public_key.as_bytes()).into())
        .with_allowed_ips(allowed_ips)
        .with_endpoint(endpoint);

    if let Some(psk) = psk {
        // TODO: implement zeroize in gotatun
        peer = peer.with_preshared_key(*psk.as_bytes());
    }

    if let Some(daita) = daita {
        let daita = gotatun::device::daita::DaitaSettings {
            maybenot_machines: daita.client_machines.clone(),
            max_padding_frac: daita.max_padding_frac,
            max_blocking_frac: daita.max_blocking_frac,
            // TODO: tweak to sane values
            max_blocked_packets: 1024,
            min_blocking_capacity: 50,
        };
        peer = peer.with_daita(daita);
    }

    peer
}

impl From<gotatun::device::configure::Stats> for Stats {
    fn from(peer_stats: gotatun::device::configure::Stats) -> Self {
        let daita = peer_stats.daita.as_ref().map(DaitaStats::from);

        let last_handshake_time = peer_stats
            .last_handshake
            .map(|duration_since| SystemTime::now() - duration_since);

        Stats {
            tx_bytes: peer_stats.tx_bytes as u64,
            rx_bytes: peer_stats.rx_bytes as u64,
            last_handshake_time,
            daita,
        }
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
