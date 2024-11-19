use std::{
    sync::Weak,
    time::{Duration, Instant},
};

use tokio::sync::Mutex;

use crate::TunnelType;

use super::check::{Cancellable, Check};
use super::error::Error;

/// Sleep time used when checking if an established connection is still working.
const REGULAR_LOOP_SLEEP: Duration = Duration::from_secs(1);

pub struct Monitor {
    connectivity_check: Check<Cancellable>,
}

impl Monitor {
    pub fn init(connectivity_check: Check<Cancellable>) -> Self {
        Self { connectivity_check }
    }

    pub fn run(self, tunnel_handle: Weak<Mutex<Option<TunnelType>>>) -> Result<(), Error> {
        self.wait_loop(REGULAR_LOOP_SLEEP, tunnel_handle)
    }

    fn wait_loop(
        mut self,
        iter_delay: Duration,
        tunnel_handle: Weak<Mutex<Option<TunnelType>>>,
    ) -> Result<(), Error> {
        let mut last_iteration = Instant::now();
        while !self.connectivity_check.should_shut_down(iter_delay) {
            let mut current_iteration = Instant::now();
            let time_slept = current_iteration - last_iteration;
            if time_slept < (iter_delay * 2) {
                let Some(tunnel) = tunnel_handle.upgrade() else {
                    return Ok(());
                };
                let lock = tunnel.blocking_lock();
                let Some(tunnel) = lock.as_ref() else {
                    return Ok(());
                };

                if !self
                    .connectivity_check
                    .check_connectivity(Instant::now(), tunnel)?
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
                self.connectivity_check.reset(current_iteration);
            }
            last_iteration = current_iteration;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO: Port to async + tokio to reduce cost of testing?
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::time::Duration;
    use std::time::Instant;

    use tokio::sync::Mutex;

    use crate::connectivity::constants::*;
    use crate::connectivity::mock::*;

    #[test]
    /// Verify that the connectivity monitor doesn't fail if the tunnel constantly sends traffic,
    /// and it shuts down properly.
    fn test_wait_loop() {
        use std::sync::mpsc;
        let (result_tx, result_rx) = mpsc::channel();
        let tunnel = MockTunnel::always_incrementing().boxed();
        let pinger = MockPinger::default();
        let (mut checker, stop_tx) = {
            let now = Instant::now();
            let start = now.checked_sub(Duration::from_secs(1)).unwrap();
            mock_checker(start, Box::new(pinger)).with_cancellation()
        };
        std::thread::spawn(move || {
            let start_result = checker.establish_connectivity(0, &tunnel);
            result_tx.send(start_result).unwrap();
            // Pointer dance
            let tunnel = Arc::new(Mutex::new(Some(tunnel)));
            let _tunnel = Arc::downgrade(&tunnel);
            let result = Monitor::init(checker).run(_tunnel).map(|_| true);
            result_tx.send(result).unwrap();
        });

        std::thread::sleep(Duration::from_secs(1));
        assert!(result_rx.try_recv().unwrap().unwrap());
        stop_tx.send(()).unwrap();
        std::thread::sleep(Duration::from_secs(1));
        assert!(result_rx.try_recv().unwrap().is_ok());
    }

    #[test]
    /// Verify that the connectivity monitor detects the tunnel timing out after no longer than
    /// `BYTES_RX_TIMEOUT` and `PING_TIMEOUT` combined.
    fn test_wait_loop_timeout() {
        let should_stop = Arc::new(AtomicBool::new(false));
        let should_stop_inner = should_stop.clone();

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
            if !should_stop_inner.load(Ordering::SeqCst) {
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

        let (result_tx, result_rx) = mpsc::channel();

        std::thread::spawn(move || {
            let (mut checker, _cancellation_token) = {
                let now = Instant::now();
                let start = now.checked_sub(Duration::from_secs(1)).unwrap();
                mock_checker(start, Box::new(pinger)).with_cancellation()
            };
            let start_result = checker.establish_connectivity(0, &tunnel);
            result_tx.send(start_result).unwrap();
            // Pointer dance
            let _tunnel = Arc::new(Mutex::new(Some(tunnel)));
            let tunnel = Arc::downgrade(&_tunnel);
            let end_result = Monitor::init(checker).run(tunnel).map(|_| true);
            result_tx.send(end_result).expect("Failed to send result");
        });
        assert!(result_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .unwrap());
        should_stop.store(true, Ordering::SeqCst);
        assert!(result_rx
            .recv_timeout(BYTES_RX_TIMEOUT + PING_TIMEOUT + Duration::from_secs(2))
            .unwrap()
            .is_ok());
    }
}
