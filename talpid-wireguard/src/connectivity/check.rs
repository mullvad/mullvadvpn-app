use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::Instant;

use super::constants::*;
use super::error::Error;
use super::pinger;

use crate::stats::StatsMap;
#[cfg(target_os = "android")]
use crate::Tunnel;
use crate::{TunnelError, TunnelType};
use pinger::Pinger;

/// Verifies if a connection to a tunnel is working.
/// The connectivity monitor is biased to receiving traffic - it is expected that all outgoing
/// traffic will be answered with a response.
///
/// The connectivity monitor tries to opportunistically use information about how much data has
/// been sent through the tunnel to infer connectivity. This is done by reading the traffic data
/// from the tunnel and recording the time of the reading - the connectivity monitor only stores
/// the timestamp of when was the last time an increase in either incoming or outgoing traffic was
/// observed. The connectivity monitor tries to read the data at a set interval, and the connection
/// is considered to be working if the incoming traffic timestamp has been incremented in a given
/// timeout. A connection is considered to be established the first time an increase in incoming
/// traffic is observed.
///
/// The connectivity monitor will start sending pings and start the countdown to `PING_TIMEOUT` in
/// the following cases:
/// - In case that we have observed a bump in the outgoing traffic but no corresponding incoming
///   traffic for longer than `BYTES_RX_TIMEOUT`, then the monitor will start pinging.
/// - In case that no increase in outgoing or incoming traffic has been observed for longer than
///   `TRAFFIC_TIMEOUT`, then the monitor will start pinging as well.
///
/// Once a connection established, a connection is only considered broken once the connectivity
/// monitor has started pinging and no traffic has been received for a duration of `PING_TIMEOUT`.
pub struct Check {
    conn_state: ConnState,
    ping_state: PingState,
    cancel_receiver: CancelReceiver,
    retry_attempt: u32,
}

/// A handle that can be used to shut down the connectivity monitor.
/// The monitor will also be shut down if all tokens are dropped.
#[derive(Debug, Clone)]
pub struct CancelToken {
    closed: Arc<AtomicBool>,
    tx: broadcast::Sender<()>,
}

/// A handle that can be passed to a [Check]. The corresponding [CancelToken] causes the [Check] to
/// be stopped. Any [CancelToken] will cancel all receivers
#[derive(Debug)]
pub struct CancelReceiver {
    closed: Arc<AtomicBool>,
    rx: broadcast::Receiver<()>,
}

impl CancelReceiver {
    fn closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
}

impl Clone for CancelReceiver {
    fn clone(&self) -> Self {
        Self {
            closed: self.closed.clone(),
            rx: self.rx.resubscribe(),
        }
    }
}

impl CancelToken {
    pub fn new() -> (Self, CancelReceiver) {
        let (tx, rx) = broadcast::channel(1);
        let closed = Arc::new(AtomicBool::new(false));
        (
            CancelToken {
                closed: closed.clone(),
                tx,
            },
            CancelReceiver { closed, rx },
        )
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        let _ = self.tx.send(());
    }
}

impl Check {
    pub fn new(
        addr: Ipv4Addr,
        #[cfg(any(target_os = "macos", target_os = "linux"))] interface: String,
        retry_attempt: u32,
        cancel_receiver: CancelReceiver,
    ) -> Result<Check, Error> {
        Ok(Check {
            conn_state: ConnState::new(Instant::now(), Default::default()),
            ping_state: PingState::new(
                addr,
                #[cfg(any(target_os = "macos", target_os = "linux"))]
                interface,
            )?,
            retry_attempt,
            cancel_receiver,
        })
    }

    #[cfg(test)]
    /// Create a new [Check] with a custom initial state.
    pub(super) fn mock(conn_state: ConnState, ping_state: PingState) -> (Self, CancelToken) {
        let (cancel_token, cancel_receiver) = CancelToken::new();
        (
            Check {
                conn_state,
                ping_state,
                retry_attempt: 0,
                cancel_receiver,
            },
            cancel_token,
        )
    }

    // checks if the tunnel has ever worked. Intended to check if a connection to a tunnel is
    // successful at the start of a connection.
    pub async fn establish_connectivity(
        &mut self,
        tunnel_handle: &TunnelType,
    ) -> Result<bool, Error> {
        // Send initial ping to prod WireGuard into connecting.
        self.ping_state
            .pinger
            .send_icmp()
            .await
            .map_err(Error::PingError)?;
        self.establish_connectivity_inner(
            self.retry_attempt,
            ESTABLISH_TIMEOUT,
            ESTABLISH_TIMEOUT_MULTIPLIER,
            MAX_ESTABLISH_TIMEOUT,
            tunnel_handle,
        )
        .await
    }

    pub(crate) async fn reset(&mut self, current_iteration: Instant) {
        self.ping_state.reset().await;
        self.conn_state.reset_after_suspension(current_iteration);
    }

    async fn establish_connectivity_inner(
        &mut self,
        retry_attempt: u32,
        timeout_initial: Duration,
        timeout_multiplier: u32,
        max_timeout: Duration,
        tunnel_handle: &TunnelType,
    ) -> Result<bool, Error> {
        if self.conn_state.connected() {
            return Ok(true);
        }

        let check_timeout = max_timeout
            .min(timeout_initial.saturating_mul(timeout_multiplier.saturating_pow(retry_attempt)));

        // Begin polling tunnel traffic stats periodically
        let poll_check = async {
            loop {
                if Self::check_connectivity_interval(
                    &mut self.conn_state,
                    &mut self.ping_state,
                    Instant::now(),
                    check_timeout,
                    tunnel_handle,
                )
                .await?
                {
                    return Ok(true);
                }
                // Calling get_stats has an unwanted effect of possibly causing segmentation fault,
                // stacktrace hints towards Garbage Collector failing. The cause has yet not been
                // determined, it could be because some dangling pointer, bug inside WG-go or
                // something else. So for now we avoid spamming get_config too much since it lowers
                // the risk of crash happening.
                //
                // The value was previously set to 20 ms, depending on when we called
                // establish_connectivity, this caused the crash to reliably occur.
                //
                // Tracked by DROID-1825 (Investigate GO crash issue with runtime.GC())
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        };

        let timeout = tokio::time::sleep(check_timeout);

        tokio::select! {
            // Tunnel status polling returned a result
            result = poll_check => {
                result
            }

            // Cancel token signal
            _ = self.cancel_receiver.rx.recv() => {
                Ok(false)
            }

            // Give up if the timeout is hit
            _ = timeout => {
                Ok(false)
            }
        }
    }

    pub(crate) fn should_shut_down(&self) -> bool {
        self.cancel_receiver.closed()
    }

    /// Returns true if connection is established
    pub(crate) async fn check_connectivity(
        &mut self,
        now: Instant,
        tunnel_handle: &TunnelType,
    ) -> Result<bool, Error> {
        Self::check_connectivity_interval(
            &mut self.conn_state,
            &mut self.ping_state,
            now,
            PING_TIMEOUT,
            tunnel_handle,
        )
        .await
    }

    /// Returns true if connection is established
    async fn check_connectivity_interval(
        conn_state: &mut ConnState,
        ping_state: &mut PingState,
        now: Instant,
        timeout: Duration,
        tunnel_handle: &TunnelType,
    ) -> Result<bool, Error> {
        match Self::get_stats(tunnel_handle)
            .await
            .map_err(Error::ConfigReadError)?
        {
            None => Ok(false),
            Some(new_stats) => {
                if conn_state.update(now, new_stats) {
                    ping_state.reset().await;
                    return Ok(true);
                }

                Self::maybe_send_ping(conn_state, ping_state, now).await?;
                Ok(!ping_state.ping_timed_out(timeout) && conn_state.connected())
            }
        }
    }

    /// If None is returned, then the underlying tunnel has already been closed and all subsequent
    /// calls will also return None.
    async fn get_stats(tunnel_handle: &TunnelType) -> Result<Option<StatsMap>, TunnelError> {
        let stats = tunnel_handle.get_tunnel_stats().await?;
        if stats.is_empty() {
            log::error!("Tunnel unexpectedly shut down");
            Ok(None)
        } else {
            Ok(Some(stats))
        }
    }

    async fn maybe_send_ping(
        conn_state: &mut ConnState,
        ping_state: &mut PingState,
        now: Instant,
    ) -> Result<(), Error> {
        // Only send out a ping if we haven't received a byte in a while or no traffic has flowed
        // in the last 2 minutes, but if a ping already has been sent out, only send one out every
        // 3 seconds.
        if (conn_state.rx_timed_out() || conn_state.traffic_timed_out())
            && ping_state
                .initial_ping_timestamp
                .map(|initial_ping_timestamp| {
                    initial_ping_timestamp.elapsed() / ping_state.num_pings_sent < SECONDS_PER_PING
                })
                .unwrap_or(true)
        {
            ping_state
                .pinger
                .send_icmp()
                .await
                .map_err(Error::PingError)?;
            if ping_state.initial_ping_timestamp.is_none() {
                ping_state.initial_ping_timestamp = Some(now);
            }
            ping_state.num_pings_sent += 1;
        }
        Ok(())
    }
}

pub(super) struct PingState {
    initial_ping_timestamp: Option<Instant>,
    num_pings_sent: u32,
    pinger: Box<dyn Pinger>,
}

impl PingState {
    pub(super) fn new(
        addr: Ipv4Addr,
        #[cfg(any(target_os = "macos", target_os = "linux"))] interface: String,
    ) -> Result<Self, Error> {
        let pinger = pinger::new_pinger(
            addr,
            #[cfg(any(target_os = "macos", target_os = "linux"))]
            interface,
        )
        .map_err(Error::PingError)?;

        Ok(Self::new_with(pinger))
    }

    pub(super) fn new_with(pinger: Box<dyn Pinger>) -> Self {
        Self {
            initial_ping_timestamp: None,
            num_pings_sent: 0,
            pinger,
        }
    }

    fn ping_timed_out(&self, timeout: Duration) -> bool {
        self.initial_ping_timestamp
            .map(|initial_ping_timestamp| initial_ping_timestamp.elapsed() > timeout)
            .unwrap_or(false)
    }

    /// Reset timeouts - assume that the last time bytes were received is now.
    async fn reset(&mut self) {
        self.initial_ping_timestamp = None;
        self.num_pings_sent = 0;
        self.pinger.reset().await;
    }
}

pub(super) enum ConnState {
    Connecting {
        start: Instant,
        stats: StatsMap,
        tx_timestamp: Option<Instant>,
    },
    Connected {
        rx_timestamp: Instant,
        tx_timestamp: Instant,
        stats: StatsMap,
    },
}

impl ConnState {
    pub fn new(start: Instant, stats: StatsMap) -> Self {
        ConnState::Connecting {
            start,
            stats,
            tx_timestamp: None,
        }
    }

    /// Returns true if incoming traffic counters incremented
    pub fn update(&mut self, now: Instant, new_stats: StatsMap) -> bool {
        match self {
            ConnState::Connecting {
                start,
                stats,
                tx_timestamp,
            } => {
                if !new_stats.is_empty() && new_stats.values().all(|stats| stats.rx_bytes > 0) {
                    let tx_timestamp = tx_timestamp.unwrap_or(*start);
                    let connected_state = ConnState::Connected {
                        rx_timestamp: now,
                        tx_timestamp,
                        stats: new_stats,
                    };
                    *self = connected_state;
                    return true;
                }
                if stats.values().map(|stats| stats.tx_bytes).sum::<u64>()
                    < new_stats.values().map(|stats| stats.tx_bytes).sum()
                {
                    let start = *start;
                    let stats = new_stats;
                    *self = ConnState::Connecting {
                        start,
                        tx_timestamp: Some(now),
                        stats,
                    };
                    return false;
                }
                false
            }
            ConnState::Connected {
                rx_timestamp,
                tx_timestamp,
                stats,
            } => {
                let rx_incremented = stats.iter().all(|(key, peer_stats)| {
                    new_stats
                        .get(key)
                        .map(|new_stats| new_stats.rx_bytes > peer_stats.rx_bytes)
                        .unwrap_or(false)
                });
                let rx_timestamp = if rx_incremented { now } else { *rx_timestamp };
                let tx_timestamp = if stats.values().map(|stats| stats.tx_bytes).sum::<u64>()
                    < new_stats.values().map(|stats| stats.tx_bytes).sum()
                {
                    now
                } else {
                    *tx_timestamp
                };
                *self = ConnState::Connected {
                    rx_timestamp,
                    tx_timestamp,
                    stats: new_stats,
                };

                rx_incremented
            }
        }
    }

    pub fn reset_after_suspension(&mut self, now: Instant) {
        if let ConnState::Connected { rx_timestamp, .. } = self {
            *rx_timestamp = now;
        }
    }

    // check if last time data was received is too long ago
    pub fn rx_timed_out(&self) -> bool {
        match self {
            ConnState::Connecting { start, .. } => start.elapsed() >= BYTES_RX_TIMEOUT,
            ConnState::Connected {
                rx_timestamp,
                tx_timestamp,
                ..
            } => {
                // if last sent bytes were sent after or at the same time as last received bytes
                tx_timestamp >= rx_timestamp &&
                    // and the response hasn't been seen for BYTES_RX_TIMEOUT
                    rx_timestamp.elapsed() >= BYTES_RX_TIMEOUT
            }
        }
    }

    // check if no bytes have been sent or received in a while
    pub fn traffic_timed_out(&self) -> bool {
        match self {
            ConnState::Connecting { .. } => self.rx_timed_out(),
            ConnState::Connected {
                rx_timestamp,
                tx_timestamp,
                ..
            } => {
                rx_timestamp.elapsed() >= TRAFFIC_TIMEOUT
                    || tx_timestamp.elapsed() >= TRAFFIC_TIMEOUT
            }
        }
    }

    pub fn connected(&self) -> bool {
        matches!(self, ConnState::Connected { .. })
    }
}

#[cfg(test)]
mod test {
    use tokio::sync::mpsc;

    use super::*;
    use crate::connectivity::mock::*;

    /// Test if a newly created ConnState won't have timed out or consider itself connected
    #[test]
    fn test_conn_state_no_timeout_on_start() {
        let now = Instant::now();
        let conn_state = ConnState::new(now, Default::default());

        assert!(!conn_state.connected());
        assert!(!conn_state.rx_timed_out());
        assert!(!conn_state.traffic_timed_out());
    }

    /// Test if ConnState::Connecting will timeout after not receiving any traffic after
    /// BYTES_RX_TIMEOUT
    #[test]
    fn test_conn_state_timeout_after_rx_timeout() {
        let now = Instant::now().checked_sub(BYTES_RX_TIMEOUT).unwrap();
        let conn_state = ConnState::new(now, Default::default());

        assert!(!conn_state.connected());
        assert!(conn_state.rx_timed_out());
        assert!(conn_state.traffic_timed_out());
    }

    /// Test if ConnState::Connecting correctly transitions into ConnState::Connected if traffic is
    /// received
    #[test]
    fn test_conn_state_connects() {
        let start = Instant::now().checked_sub(Duration::from_secs(2)).unwrap();
        let mut conn_state = ConnState::new(start, Default::default());
        let mut stats = StatsMap::new();
        stats.insert(
            [0u8; 32],
            Stats {
                rx_bytes: 1,
                tx_bytes: 0,
            },
        );
        conn_state.update(Instant::now(), stats);

        assert!(conn_state.connected());
        assert!(!conn_state.rx_timed_out());
        assert!(!conn_state.traffic_timed_out());
    }

    /// Test if ConnState::Connected correctly times out after TRAFFIC_TIMEOUT when no traffic is
    /// observed
    #[test]
    fn test_conn_state_traffic_times_out_after_connecting() {
        let start = Instant::now()
            .checked_sub(TRAFFIC_TIMEOUT + Duration::from_secs(1))
            .unwrap();
        let mut conn_state = ConnState::new(start, Default::default());

        let connect_time = Instant::now().checked_sub(TRAFFIC_TIMEOUT).unwrap();
        let mut stats = StatsMap::new();
        stats.insert(
            [0u8; 32],
            Stats {
                rx_bytes: 1,
                tx_bytes: 0,
            },
        );
        conn_state.update(connect_time, stats);

        assert!(conn_state.connected());
        assert!(!conn_state.rx_timed_out());
        assert!(conn_state.traffic_timed_out());
    }

    /// Test if ConnState::Connected correctly times out after BYTES_RX_TIMEOUT when no incoming
    /// traffic is observed
    #[test]
    fn test_conn_state_rx_times_out_after_connecting() {
        let start = Instant::now()
            .checked_sub(BYTES_RX_TIMEOUT + Duration::from_secs(1))
            .unwrap();
        let mut conn_state = ConnState::new(start, Default::default());

        let mut stats = StatsMap::new();
        stats.insert(
            [0u8; 32],
            Stats {
                rx_bytes: 1,
                tx_bytes: 0,
            },
        );
        conn_state.update(start, stats);

        let update_time = Instant::now().checked_sub(BYTES_RX_TIMEOUT).unwrap();
        let mut stats = StatsMap::new();
        stats.insert(
            [0u8; 32],
            Stats {
                rx_bytes: 1,
                tx_bytes: 1,
            },
        );
        conn_state.update(update_time, stats);

        assert!(conn_state.connected());
        assert!(conn_state.rx_timed_out());
        assert!(!conn_state.traffic_timed_out());
    }

    #[tokio::test]
    /// Verify that `check_connectivity()` returns `false` if the tunnel is connected and traffic is
    /// not flowing after `BYTES_RX_TIMEOUT` and `PING_TIMEOUT`.
    async fn test_ping_times_out() {
        let tunnel = MockTunnel::never_incrementing().boxed();
        let pinger = MockPinger::default();
        let now = Instant::now();
        let start = now
            .checked_sub(BYTES_RX_TIMEOUT + PING_TIMEOUT + Duration::from_secs(10))
            .unwrap();
        let (mut checker, _cancel_token) = mock_checker(start, Box::new(pinger));

        // Mock the state - connectivity has been established
        checker.conn_state = connected_state(start);
        // A ping was sent to verify connectivity
        Check::maybe_send_ping(&mut checker.conn_state, &mut checker.ping_state, start)
            .await
            .unwrap();
        assert!(!checker.check_connectivity(now, &tunnel).await.unwrap())
    }

    #[tokio::test]
    /// Verify that `check_connectivity()` returns `true` if the tunnel is connected and traffic is
    /// flowing constantly.
    async fn test_no_connection_on_start() {
        let tunnel = MockTunnel::never_incrementing().boxed();
        let pinger = MockPinger::default();
        let now = Instant::now();
        let start = now.checked_sub(Duration::from_secs(1)).unwrap();
        let (mut checker, _cancel_token) = mock_checker(start, Box::new(pinger));

        assert!(!checker.check_connectivity(now, &tunnel).await.unwrap())
    }

    #[tokio::test]
    /// Verify that `check_connectivity()` returns `true` if the tunnel is connected and traffic is
    /// flowing constantly.
    async fn test_connection_works() {
        let tunnel = MockTunnel::always_incrementing().boxed();
        let pinger = MockPinger::default();
        let now = Instant::now();
        let start = now.checked_sub(Duration::from_secs(1)).unwrap();
        let (mut checker, _cancel_token) = mock_checker(start, Box::new(pinger));

        // Mock the state - connectivity has been established
        checker.conn_state = connected_state(start);

        assert!(checker.check_connectivity(now, &tunnel).await.unwrap())
    }

    #[tokio::test(start_paused = true)]
    /// Verify that the timeout for setting up a tunnel works as expected.
    async fn test_establish_timeout() {
        const ESTABLISH_TIMEOUT_MULTIPLIER: u32 = 2;
        const ESTABLISH_TIMEOUT: Duration = Duration::from_millis(500);
        const MAX_ESTABLISH_TIMEOUT: Duration = Duration::from_secs(2);

        let (result_tx, mut result_rx) = mpsc::channel(1);

        tokio::spawn(async move {
            let pinger = MockPinger::default();
            let now = Instant::now();
            let start = now.checked_sub(Duration::from_secs(1)).unwrap();
            let (mut monitor, _cancel_token) = mock_checker(start, Box::new(pinger));

            let tunnel = {
                let mut tunnel_stats = StatsMap::new();
                tunnel_stats.insert(
                    [0u8; 32],
                    Stats {
                        tx_bytes: 0,
                        rx_bytes: 0,
                    },
                );
                MockTunnel::new(move || Ok(tunnel_stats.clone())).boxed()
            };

            result_tx
                .send(
                    monitor
                        .establish_connectivity_inner(
                            0,
                            ESTABLISH_TIMEOUT,
                            ESTABLISH_TIMEOUT_MULTIPLIER,
                            MAX_ESTABLISH_TIMEOUT,
                            &tunnel,
                        )
                        .await,
                )
                .await
                .unwrap();
        });

        tokio::time::timeout(
            ESTABLISH_TIMEOUT - Duration::from_millis(100),
            result_rx.recv(),
        )
        .await
        .expect_err("expected timeout");

        // Should assume no connectivity after timeout
        let connected = tokio::time::timeout(
            ESTABLISH_TIMEOUT + Duration::from_millis(100),
            result_rx.recv(),
        )
        .await
        .expect("expected no timeout")
        .unwrap()
        .unwrap();
        assert!(!connected);
    }
}
