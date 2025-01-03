use std::future::Future;
use std::pin::Pin;
use tokio::time::Instant;

use super::check::{CancelToken, ConnState, PingState};
use super::pinger;
use super::Check;

use crate::{Config, Tunnel, TunnelError};
use pinger::Pinger;

// Convenient re-exports
pub use crate::stats::{Stats, StatsMap};

#[derive(Default)]
pub(crate) struct MockPinger {
    on_send_ping: Option<Box<dyn FnMut() + Send + Sync>>,
}

pub(crate) struct MockTunnel {
    on_get_stats: Box<dyn Fn() -> Result<StatsMap, TunnelError> + Send + Sync>,
}

pub fn mock_checker(now: Instant, pinger: Box<dyn Pinger>) -> (Check, CancelToken) {
    let conn_state = ConnState::new(now, Default::default());
    let ping_state = PingState::new_with(pinger);
    Check::mock(conn_state, ping_state)
}

pub fn connected_state(timestamp: Instant) -> ConnState {
    const PEER: [u8; 32] = [0u8; 32];
    let mut stats = StatsMap::new();
    stats.insert(
        PEER,
        Stats {
            tx_bytes: 0,
            rx_bytes: 0,
        },
    );
    ConnState::Connected {
        rx_timestamp: timestamp,
        tx_timestamp: timestamp,
        stats,
    }
}

impl MockTunnel {
    const PEER: [u8; 32] = [0u8; 32];

    pub fn new<F: Fn() -> Result<StatsMap, TunnelError> + Send + Sync + 'static>(f: F) -> Self {
        Self {
            on_get_stats: Box::new(f),
        }
    }

    /// Convert self to the more general [TunnelType].
    pub fn boxed(self) -> Box<dyn Tunnel> {
        Box::new(self)
    }

    pub fn always_incrementing() -> Self {
        let mut map = StatsMap::new();
        map.insert(
            Self::PEER,
            Stats {
                tx_bytes: 0,
                rx_bytes: 0,
            },
        );
        let peers = std::sync::Mutex::new(map);
        Self {
            on_get_stats: Box::new(move || {
                let mut peers = peers.lock().unwrap();
                for traffic in peers.values_mut() {
                    traffic.tx_bytes += 1;
                    traffic.rx_bytes += 1;
                }
                Ok(peers.clone())
            }),
        }
    }

    pub fn never_incrementing() -> Self {
        Self {
            on_get_stats: Box::new(|| {
                let mut map = StatsMap::new();
                map.insert(
                    Self::PEER,
                    Stats {
                        tx_bytes: 0,
                        rx_bytes: 0,
                    },
                );
                Ok(map)
            }),
        }
    }
}

#[async_trait::async_trait]
impl Tunnel for MockTunnel {
    fn get_interface_name(&self) -> String {
        "mock-tunnel".to_string()
    }

    fn stop(self: Box<Self>) -> Result<(), TunnelError> {
        Ok(())
    }

    async fn get_tunnel_stats(&self) -> Result<StatsMap, TunnelError> {
        (self.on_get_stats)()
    }

    fn set_config(
        &mut self,
        _config: Config,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), TunnelError>> + Send>> {
        Box::pin(async { Ok(()) })
    }

    #[cfg(daita)]
    fn start_daita(
        &mut self,
        #[cfg(not(target_os = "windows"))] _: talpid_tunnel_config_client::DaitaSettings,
    ) -> std::result::Result<(), TunnelError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl Pinger for MockPinger {
    async fn send_icmp(&mut self) -> Result<(), pinger::Error> {
        if let Some(callback) = self.on_send_ping.as_mut() {
            (callback)();
        }
        Ok(())
    }
}
