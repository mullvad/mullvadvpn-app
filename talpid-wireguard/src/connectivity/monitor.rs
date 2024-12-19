use std::{sync::Weak, time::Duration};

use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::TunnelType;

use super::check::Check;
use super::error::Error;

/// Sleep time used when checking if an established connection is still working.
const REGULAR_LOOP_SLEEP: Duration = Duration::from_secs(1);

pub struct Monitor {
    connectivity_check: Check,
}

impl Monitor {
    pub fn init(connectivity_check: Check) -> Self {
        Self { connectivity_check }
    }

    pub async fn run(self, tunnel_handle: Weak<Mutex<Option<TunnelType>>>) -> Result<(), Error> {
        self.wait_loop(REGULAR_LOOP_SLEEP, tunnel_handle).await
    }

    async fn wait_loop(
        mut self,
        iter_delay: Duration,
        tunnel_handle: Weak<Mutex<Option<TunnelType>>>,
    ) -> Result<(), Error> {
        let mut last_iteration = Instant::now();
        while !self.connectivity_check.should_shut_down() {
            let mut current_iteration = Instant::now();
            let time_slept = current_iteration - last_iteration;
            if time_slept < (iter_delay * 2) {
                let Some(tunnel) = tunnel_handle.upgrade() else {
                    return Ok(());
                };
                let lock = tunnel.lock().await;
                let Some(tunnel) = lock.as_ref() else {
                    return Ok(());
                };

                if !self
                    .connectivity_check
                    .check_connectivity(Instant::now(), tunnel)
                    .await?
                {
                    return Ok(());
                }
                drop(lock);

                let end = Instant::now();
                if end - current_iteration > Duration::from_secs(1) {
                    current_iteration = end;
                }
            } else {
                // Loop was suspended for too long, so it's safer to assume that the host still has
                // connectivity.
                self.connectivity_check.reset(current_iteration).await;
            }
            last_iteration = current_iteration;

            // Sleep for a while
            tokio::time::sleep(iter_delay).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::sync::mpsc;
    use tokio::sync::Mutex;

    use crate::connectivity::constants::*;
    use crate::connectivity::mock::*;

    #[tokio::test(start_paused = true)]
    /// Verify that the connectivity monitor doesn't fail if the tunnel constantly sends traffic,
    /// and it shuts down properly.
    async fn test_wait_loop() {
        let (result_tx, mut result_rx) = mpsc::channel(1);
        let tunnel = MockTunnel::always_incrementing().boxed();
        let pinger = MockPinger::default();
        let (mut checker, stop_tx) = {
            let now = Instant::now();
            let start = now.checked_sub(Duration::from_secs(1)).unwrap();
            mock_checker(start, Box::new(pinger))
        };

        tokio::spawn(async move {
            let start_result = checker.establish_connectivity(&tunnel).await;
            result_tx.send(start_result).await.unwrap();
            // Pointer dance
            let tunnel = Arc::new(Mutex::new(Some(tunnel)));
            let _tunnel = Arc::downgrade(&tunnel);
            let result = Monitor::init(checker).run(_tunnel).await.map(|_| true);
            result_tx.send(result).await.unwrap();
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(result_rx.try_recv().unwrap().unwrap());
        stop_tx.close();
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(result_rx.try_recv().unwrap().is_ok());
    }

    #[tokio::test(start_paused = true)]
    /// Verify that the connectivity monitor detects the tunnel timing out after no longer than
    /// `BYTES_RX_TIMEOUT` and `PING_TIMEOUT` combined.
    async fn test_wait_loop_timeout() {
        let receive_bytes = Arc::new(AtomicBool::new(false));
        let receive_bytes_inner = receive_bytes.clone();

        let mut map = StatsMap::new();
        map.insert(
            [0u8; 32],
            Stats {
                tx_bytes: 0,
                rx_bytes: 0,
            },
        );
        let tunnel_stats = std::sync::Mutex::new(map);

        let pinger = MockPinger::default();
        let tunnel = MockTunnel::new(move || {
            let mut tunnel_stats = tunnel_stats.lock().unwrap();
            if !receive_bytes_inner.load(Ordering::SeqCst) {
                for traffic in tunnel_stats.values_mut() {
                    traffic.rx_bytes += 1;
                }
            }
            for traffic in tunnel_stats.values_mut() {
                traffic.tx_bytes += 1;
            }
            Ok(tunnel_stats.clone())
        })
        .boxed();

        let (result_tx, mut result_rx) = mpsc::channel(1);

        tokio::spawn(async move {
            let (mut checker, _cancellation_token) = {
                let now = Instant::now();
                let start = now.checked_sub(Duration::from_secs(1)).unwrap();
                mock_checker(start, Box::new(pinger))
            };
            let start_result = checker.establish_connectivity(&tunnel).await;
            result_tx.send(start_result).await.unwrap();
            // Pointer dance
            let _tunnel = Arc::new(Mutex::new(Some(tunnel)));
            let tunnel = Arc::downgrade(&_tunnel);
            let end_result = Monitor::init(checker).run(tunnel).await.map(|_| true);
            result_tx
                .send(end_result)
                .await
                .expect("Failed to send result");
        });

        assert!(
            tokio::time::timeout(Duration::from_secs(1), result_rx.recv())
                .await
                .unwrap()
                .unwrap()
                .unwrap()
        );
        receive_bytes.store(true, Ordering::SeqCst);
        assert!(tokio::time::timeout(
            BYTES_RX_TIMEOUT + PING_TIMEOUT + Duration::from_secs(2),
            result_rx.recv()
        )
        .await
        .unwrap()
        .unwrap()
        .is_ok());
    }
}
