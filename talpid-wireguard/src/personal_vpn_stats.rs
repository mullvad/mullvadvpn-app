//! Report the stats of the personal VPN peer while the tunnel is up.

use crate::TunnelType;
use std::sync::Weak;
use tokio::sync::Mutex;

pub async fn get_stats(
    peer_pubkey: &[u8; 32],
    tunnel: Weak<Mutex<Option<TunnelType>>>,
) -> Option<talpid_types::Stats> {
    let Some(tunnel) = tunnel.upgrade() else {
        return None;
    };
    let peers = {
        let lock = tunnel.lock().await;
        let Some(tunnel) = lock.as_ref() else {
            return None;
        };
        tunnel.get_personal_vpn_stats().await
    };
    let stats = match peers {
        Ok(peers) => match peers.get(peer_pubkey) {
            Some(stats) => talpid_types::Stats {
                last_handshake_time: stats.last_handshake_time,
                rx_bytes: stats.rx_bytes,
                tx_bytes: stats.tx_bytes,
            },
            None => return None,
        },
        Err(err) => {
            log::error!("Failed to get personal VPN stats: {err:#?}");
            return None;
        }
    };
    Some(stats)
}
