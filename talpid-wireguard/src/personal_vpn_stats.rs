//! Report the stats of the personal VPN peer while the tunnel is up.

use crate::TunnelType;
use std::sync::Weak;
use tokio::sync::Mutex;

pub async fn get_stats(tunnel: Weak<Mutex<Option<TunnelType>>>) -> Option<talpid_types::Stats> {
    let tunnel = tunnel.upgrade()?;
    let stats = {
        let lock = tunnel.lock().await;
        let tunnel = lock.as_ref()?;
        tunnel.get_personal_vpn_stats().await
    };
    match stats {
        Ok(stats) => Some(talpid_types::Stats {
            last_handshake_time: stats.last_handshake_time,
            rx_bytes: stats.rx_bytes,
            tx_bytes: stats.tx_bytes,
        }),
        Err(err) => {
            log::error!("Failed to get personal VPN stats: {err:#?}");
            None
        }
    }
}
