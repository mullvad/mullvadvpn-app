//! Report the stats of the personal VPN peer while the tunnel is up.

use crate::{TunnelType, config::Config};
use std::{
    sync::{Arc, Weak},
    time::Duration,
};
use talpid_types::drop_guard::{OnDrop, on_drop};
use tokio::{
    sync::{Mutex, mpsc},
    time::MissedTickBehavior,
};

/// How often to read the stats of the personal VPN peer.
const INTERVAL: Duration = Duration::from_secs(1);

/// Start sending the stats of the personal VPN peer in `config` to `stats_tx`, if there is one.
/// Stops when the returned guard is dropped, or when the tunnel or the receiver is gone.
pub fn spawn(
    stats_tx: Option<mpsc::Sender<talpid_types::Stats>>,
    config: &Config,
    tunnel: &Arc<Mutex<Option<TunnelType>>>,
) -> Option<OnDrop> {
    let stats_tx = stats_tx?;
    let peer_pubkey = *config.personal_vpn.as_ref()?.peer.public_key.as_bytes();
    let task = tokio::spawn(run(peer_pubkey, stats_tx, Arc::downgrade(tunnel)));
    Some(on_drop(move || task.abort()).boxed())
}

async fn run(
    peer_pubkey: [u8; 32],
    stats_tx: mpsc::Sender<talpid_types::Stats>,
    tunnel: Weak<Mutex<Option<TunnelType>>>,
) {
    let mut interval = tokio::time::interval(INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        interval.tick().await;

        let Some(tunnel) = tunnel.upgrade() else {
            return;
        };
        let peers = {
            let lock = tunnel.lock().await;
            let Some(tunnel) = lock.as_ref() else {
                return;
            };
            tunnel.get_personal_vpn_stats().await
        };
        let stats = match peers {
            Ok(peers) => match peers.get(&peer_pubkey) {
                Some(stats) => talpid_types::Stats {
                    last_handshake_time: stats.last_handshake_time,
                    rx_bytes: stats.rx_bytes,
                    tx_bytes: stats.tx_bytes,
                },
                None => continue,
            },
            Err(err) => {
                log::error!("Failed to get personal VPN stats: {err:#?}");
                continue;
            }
        };
        // Drop the update if the receiver is behind. The next one follows shortly.
        match stats_tx.try_send(stats) {
            Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => (),
            Err(mpsc::error::TrySendError::Closed(_)) => return,
        }
    }
}
