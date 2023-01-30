use crate::{
    ping_monitor::{new_pinger, Pinger},
    stats::StatsMap,
};
use std::{
    cmp,
    net::Ipv4Addr,
    sync::{mpsc, Mutex, Weak},
    time::{Duration, Instant},
};

use super::{Tunnel, TunnelError};

/// Sleep time used when initially establishing connectivity
const DELAY_ON_INITIAL_SETUP: Duration = Duration::from_millis(50);
/// Sleep time used when checking if an established connection is still working.
const REGULAR_LOOP_SLEEP: Duration = Duration::from_secs(1);

/// Timeout for waiting on receiving traffic after sending outgoing traffic.  Once this timeout is
/// hit, a ping will be sent every `SECONDS_PER_PING` until `PING_TIMEOUT` is reached, or traffic
/// is received.
const BYTES_RX_TIMEOUT: Duration = Duration::from_secs(5);
/// Timeout for waiting on receiving or sending any traffic.  Once this timeout is hit, a ping will
/// be sent every `SECONDS_PER_PING` until `PING_TIMEOUT` is reached or traffic is received.
const TRAFFIC_TIMEOUT: Duration = Duration::from_secs(120);
/// Timeout for waiting on receiving traffic after sending the first ICMP packet.  Once this
/// timeout is reached, it is assumed that the connection is lost.
const PING_TIMEOUT: Duration = Duration::from_secs(15);
/// Timeout for receiving traffic when establishing a connection.
const ESTABLISH_TIMEOUT: Duration = Duration::from_secs(4);
/// `ESTABLISH_TIMEOUT` is multiplied by this after each failed connection attempt.
const ESTABLISH_TIMEOUT_MULTIPLIER: u32 = 2;
/// Maximum timeout for establishing a connection.
const MAX_ESTABLISH_TIMEOUT: Duration = PING_TIMEOUT;
/// Number of seconds to wait between sending ICMP packets
const SECONDS_PER_PING: Duration = Duration::from_secs(3);

/// Connectivity monitor errors
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failed to read tunnel's configuration
    #[error(display = "Failed to read tunnel's configuration")]
    ConfigReadError(TunnelError),

    /// Failed to send ping
    #[error(display = "Ping monitor failed")]
    PingError(#[error(source)] crate::ping_monitor::Error),
}

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
/// - In case that we have observed a bump in the outgoing traffic but no coressponding incoming
/// traffic for longer than `BYTES_RX_TIMEOUT`, then the monitor will start pinging.
/// - In case that no increase in outgoing or incoming traffic has been observed for longer than
/// `TRAFFIC_TIMEOUT`, then the monitor will start pinging as well.
///
/// Once a connection established, a connection is only considered broken once the connectivity
/// monitor has started pinging and no traffic has been received for a duration of `PING_TIMEOUT`.
pub struct ConnectivityMonitor {
    tunnel_handle: Weak<Mutex<Option<Box<dyn Tunnel>>>>,
    conn_state: ConnState,
    initial_ping_timestamp: Option<Instant>,
    num_pings_sent: u32,
    pinger: Box<dyn Pinger>,
    close_receiver: mpsc::Receiver<()>,
}

impl ConnectivityMonitor {
    pub(super) fn new(
        addr: Ipv4Addr,
        #[cfg(any(target_os = "macos", target_os = "linux"))] interface: String,
        tunnel_handle: Weak<Mutex<Option<Box<dyn Tunnel>>>>,
        close_receiver: mpsc::Receiver<()>,
    ) -> Result<Self, Error> {
        let pinger = new_pinger(
            addr,
            #[cfg(any(target_os = "macos", target_os = "linux"))]
            interface,
        )
        .map_err(Error::PingError)?;

        let now = Instant::now();

        Ok(Self {
            tunnel_handle,
            conn_state: ConnState::new(now, Default::default()),
            initial_ping_timestamp: None,
            num_pings_sent: 0,
            pinger,
            close_receiver,
        })
    }

    // checks if the tunnel has ever worked. Intended to check if a connection to a tunnel is
    // successfull at the start of a connection.
    pub(super) fn establish_connectivity(&mut self, retry_attempt: u32) -> Result<bool, Error> {
        // Send initial ping to prod WireGuard into connecting.
        self.pinger.send_icmp().map_err(Error::PingError)?;
        self.establish_connectivity_inner(
            retry_attempt,
            ESTABLISH_TIMEOUT,
            ESTABLISH_TIMEOUT_MULTIPLIER,
            MAX_ESTABLISH_TIMEOUT,
        )
    }

    fn establish_connectivity_inner(
        &mut self,
        retry_attempt: u32,
        timeout_initial: Duration,
        timeout_multiplier: u32,
        max_timeout: Duration,
    ) -> Result<bool, Error> {
        if self.conn_state.connected() {
            return Ok(true);
        }

        let check_timeout = cmp::min(
            max_timeout,
            timeout_initial.saturating_mul(timeout_multiplier.saturating_pow(retry_attempt)),
        );

        let start = Instant::now();
        while start.elapsed() < check_timeout {
            if self.check_connectivity_interval(Instant::now(), check_timeout)? {
                return Ok(true);
            }
            if self.should_shut_down(DELAY_ON_INITIAL_SETUP) {
                return Ok(false);
            }
        }
        Ok(false)
    }

    pub(super) fn run(&mut self) -> Result<(), Error> {
        self.wait_loop(REGULAR_LOOP_SLEEP)
    }

    /// Returns true if monitor should be shut down
    fn should_shut_down(&mut self, timeout: Duration) -> bool {
        match self.close_receiver.recv_timeout(timeout) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => true,
            Err(mpsc::RecvTimeoutError::Timeout) => false,
        }
    }

    fn wait_loop(&mut self, iter_delay: Duration) -> Result<(), Error> {
        let mut last_iteration = Instant::now();
        while !self.should_shut_down(iter_delay) {
            let mut current_iteration = Instant::now();
            let time_slept = current_iteration - last_iteration;
            if time_slept < (iter_delay * 2) {
                if !self.check_connectivity(Instant::now())? {
                    return Ok(());
                }

                let end = Instant::now();
                if end - current_iteration > Duration::from_secs(1) {
                    current_iteration = end;
                }
            } else {
                // Loop was suspended for too long, so it's safer to assume that the host still has
                // connectivity.
                self.reset_pinger();
                self.conn_state.reset_after_suspension(current_iteration);
            }
            last_iteration = current_iteration;
        }
        Ok(())
    }

    /// Returns true if connection is established
    fn check_connectivity(&mut self, now: Instant) -> Result<bool, Error> {
        self.check_connectivity_interval(now, PING_TIMEOUT)
    }

    /// Returns true if connection is established
    fn check_connectivity_interval(
        &mut self,
        now: Instant,
        timeout: Duration,
    ) -> Result<bool, Error> {
        match self.get_stats() {
            None => Ok(false),
            Some(new_stats) => {
                let new_stats = new_stats?;

                if self.conn_state.update(now, new_stats) {
                    self.reset_pinger();
                    return Ok(true);
                }

                self.maybe_send_ping(now)?;
                Ok(!self.ping_timed_out(timeout) && self.conn_state.connected())
            }
        }
    }

    /// If None is returned, then the underlying tunnel has already been closed and all subsequent
    /// calls will also return None.
    fn get_stats(&self) -> Option<Result<StatsMap, Error>> {
        self.tunnel_handle
            .upgrade()?
            .lock()
            .ok()?
            .as_ref()
            .map(|tunnel| tunnel.get_tunnel_stats().map_err(Error::ConfigReadError))
    }

    fn maybe_send_ping(&mut self, now: Instant) -> Result<(), Error> {
        // Only send out a ping if we haven't received a byte in a while or no traffic has flowed
        // in the last 2 minutes, but if a ping already has been sent out, only send one out every
        // 3 seconds.
        if (self.conn_state.rx_timed_out() || self.conn_state.traffic_timed_out())
            && self
                .initial_ping_timestamp
                .map(|initial_ping_timestamp| {
                    initial_ping_timestamp.elapsed() / self.num_pings_sent < SECONDS_PER_PING
                })
                .unwrap_or(true)
        {
            self.pinger.send_icmp().map_err(Error::PingError)?;
            if self.initial_ping_timestamp.is_none() {
                self.initial_ping_timestamp = Some(now);
            }
            self.num_pings_sent += 1;
        }
        Ok(())
    }

    fn ping_timed_out(&self, timeout: Duration) -> bool {
        self.initial_ping_timestamp
            .map(|initial_ping_timestamp| initial_ping_timestamp.elapsed() > timeout)
            .unwrap_or(false)
    }

    /// Reset timeouts - assume that the last time bytes were received is now.
    fn reset_pinger(&mut self) {
        self.initial_ping_timestamp = None;
        self.num_pings_sent = 0;
        self.pinger.reset();
    }
}

enum ConnState {
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
        if let ConnState::Connected {
            ref mut rx_timestamp,
            ..
        } = self
        {
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
    use futures::Future;

    use super::*;
    use crate::{
        config::Config,
        stats::{self, Stats},
        TunnelError,
    };
    use std::{
        pin::Pin,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
        time::{Duration, Instant},
    };

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

    #[derive(Default)]
    struct MockPinger {
        on_send_ping: Option<Box<dyn FnMut() + Send>>,
    }

    impl Pinger for MockPinger {
        fn send_icmp(&mut self) -> Result<(), crate::ping_monitor::Error> {
            if let Some(callback) = self.on_send_ping.as_mut() {
                (callback)();
            }
            Ok(())
        }
    }

    struct MockTunnel {
        on_get_stats: Box<dyn Fn() -> Result<stats::StatsMap, TunnelError> + Send>,
    }

    impl MockTunnel {
        const PEER: [u8; 32] = [0u8; 32];

        fn new<F: Fn() -> Result<stats::StatsMap, TunnelError> + Send + 'static>(f: F) -> Self {
            Self {
                on_get_stats: Box::new(f),
            }
        }

        fn always_incrementing() -> Self {
            let mut map = stats::StatsMap::new();
            map.insert(
                Self::PEER,
                stats::Stats {
                    tx_bytes: 0,
                    rx_bytes: 0,
                },
            );
            let peers = Mutex::new(map);
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

        fn never_incrementing() -> Self {
            Self {
                on_get_stats: Box::new(|| {
                    let mut map = stats::StatsMap::new();
                    map.insert(
                        Self::PEER,
                        stats::Stats {
                            tx_bytes: 0,
                            rx_bytes: 0,
                        },
                    );
                    Ok(map)
                }),
            }
        }

        #[allow(clippy::type_complexity)]
        fn into_locked(
            self,
        ) -> (
            Arc<Mutex<Option<Box<dyn Tunnel>>>>,
            Weak<Mutex<Option<Box<dyn Tunnel>>>>,
        ) {
            let dyn_tunnel: Box<dyn Tunnel> = Box::new(self);
            let arc = Arc::new(Mutex::new(Some(dyn_tunnel)));
            let weak_ref = Arc::downgrade(&arc);
            (arc, weak_ref)
        }
    }

    impl Tunnel for MockTunnel {
        fn get_interface_name(&self) -> String {
            "mock-tunnel".to_string()
        }

        fn stop(self: Box<Self>) -> Result<(), TunnelError> {
            Ok(())
        }

        fn get_tunnel_stats(&self) -> Result<stats::StatsMap, TunnelError> {
            (self.on_get_stats)()
        }

        fn set_config(
            &self,
            _config: Config,
        ) -> Pin<Box<dyn Future<Output = std::result::Result<(), TunnelError>> + Send>> {
            Box::pin(async { Ok(()) })
        }
    }

    fn mock_monitor(
        now: Instant,
        pinger: Box<dyn Pinger>,
        tunnel_handle: Weak<Mutex<Option<Box<dyn Tunnel>>>>,
        close_receiver: mpsc::Receiver<()>,
    ) -> ConnectivityMonitor {
        ConnectivityMonitor {
            conn_state: ConnState::new(now, Default::default()),
            initial_ping_timestamp: None,
            num_pings_sent: 0,
            pinger,
            close_receiver,
            tunnel_handle,
        }
    }

    fn connected_state(timestamp: Instant) -> ConnState {
        const PEER: [u8; 32] = [0u8; 32];
        let mut stats = stats::StatsMap::new();
        stats.insert(
            PEER,
            stats::Stats {
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

    #[test]
    /// Verify that `check_connectivity()` returns `false` if the tunnel is connected and traffic is
    /// not flowing after `BYTES_RX_TIMEOUT` and `PING_TIMEOUT`.
    fn test_ping_times_out() {
        let (_tunnel_anchor, tunnel) = MockTunnel::never_incrementing().into_locked();
        let (_tx, rx) = mpsc::channel();
        let pinger = MockPinger::default();
        let now = Instant::now();
        let start = now
            .checked_sub(BYTES_RX_TIMEOUT + PING_TIMEOUT + Duration::from_secs(10))
            .unwrap();
        let mut monitor = mock_monitor(start, Box::new(pinger), tunnel, rx);

        // Mock the state - connectivity has been established
        monitor.conn_state = connected_state(start);
        // A ping was sent to verify connectivity
        monitor.maybe_send_ping(start).unwrap();
        assert!(!monitor.check_connectivity(now).unwrap())
    }

    #[test]
    /// Verify that `check_connectivity()` returns `true` if the tunnel is connected and traffic is
    /// flowing constantly.
    fn test_no_connection_on_start() {
        let (_tunnel_anchor, tunnel) = MockTunnel::never_incrementing().into_locked();
        let (_tx, rx) = mpsc::channel();
        let pinger = MockPinger::default();
        let now = Instant::now();
        let start = now.checked_sub(Duration::from_secs(1)).unwrap();
        let mut monitor = mock_monitor(start, Box::new(pinger), tunnel, rx);

        assert!(!monitor.check_connectivity(now).unwrap())
    }

    #[test]
    /// Verify that `check_connectivity()` returns `true` if the tunnel is connected and traffic is
    /// flowing constantly.
    fn test_connection_works() {
        let (_tunnel_anchor, tunnel) = MockTunnel::always_incrementing().into_locked();
        let (_tx, rx) = mpsc::channel();
        let pinger = MockPinger::default();
        let now = Instant::now();
        let start = now.checked_sub(Duration::from_secs(1)).unwrap();
        let mut monitor = mock_monitor(start, Box::new(pinger), tunnel, rx);

        // Mock the state - connectivity has been established
        monitor.conn_state = connected_state(start);

        assert!(monitor.check_connectivity(now).unwrap())
    }

    #[test]
    /// Verify that the connectivity monitor doesn't fail if the tunnel constantly sends traffic,
    /// and it shuts down properly.
    fn test_wait_loop() {
        let (result_tx, result_rx) = mpsc::channel();
        let (_tunnel_anchor, tunnel) = MockTunnel::always_incrementing().into_locked();
        let pinger = MockPinger::default();
        let (stop_tx, stop_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let now = Instant::now();
            let start = now.checked_sub(Duration::from_secs(1)).unwrap();
            let mut monitor = mock_monitor(start, Box::new(pinger), tunnel, stop_rx);

            let start_result = monitor.establish_connectivity(0);
            result_tx.send(start_result).unwrap();

            let result = monitor.run().map(|_| true);
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

        let mut map = stats::StatsMap::new();
        map.insert(
            [0u8; 32],
            stats::Stats {
                tx_bytes: 0,
                rx_bytes: 0,
            },
        );
        let tunnel_stats = Mutex::new(map);

        let pinger = MockPinger::default();
        let (_tunnel_anchor, tunnel) = MockTunnel::new(move || {
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
        .into_locked();

        let (result_tx, result_rx) = mpsc::channel();

        let (_stop_tx, stop_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let now = Instant::now();
            let start = now.checked_sub(Duration::from_secs(1)).unwrap();
            let mut monitor = mock_monitor(start, Box::new(pinger), tunnel, stop_rx);
            let start_result = monitor.establish_connectivity(0);
            result_tx.send(start_result).unwrap();
            let end_result = monitor.run().map(|_| true);
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

    #[test]
    /// Verify that the timeout for setting up a tunnel works as expected.
    fn test_establish_timeout() {
        let mut tunnel_stats = stats::StatsMap::new();
        tunnel_stats.insert(
            [0u8; 32],
            stats::Stats {
                tx_bytes: 0,
                rx_bytes: 0,
            },
        );

        let pinger = MockPinger::default();
        let (_tunnel_anchor, tunnel) =
            MockTunnel::new(move || Ok(tunnel_stats.clone())).into_locked();

        let (result_tx, result_rx) = mpsc::channel();

        let (_stop_tx, stop_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let now = Instant::now();
            let start = now.checked_sub(Duration::from_secs(1)).unwrap();
            let mut monitor = mock_monitor(start, Box::new(pinger), tunnel, stop_rx);

            const ESTABLISH_TIMEOUT_MULTIPLIER: u32 = 2;
            const ESTABLISH_TIMEOUT: Duration = Duration::from_millis(500);
            const MAX_ESTABLISH_TIMEOUT: Duration = Duration::from_secs(2);

            for attempt in 0..4 {
                result_tx
                    .send(monitor.establish_connectivity_inner(
                        attempt,
                        ESTABLISH_TIMEOUT,
                        ESTABLISH_TIMEOUT_MULTIPLIER,
                        MAX_ESTABLISH_TIMEOUT,
                    ))
                    .unwrap();
            }
        });
        let err = DELAY_ON_INITIAL_SETUP + Duration::from_millis(350);
        let assert_rx = |recv_timeout: Duration| {
            assert!(!result_rx.recv_timeout(recv_timeout + err).unwrap().unwrap());
        };
        assert_rx(Duration::from_millis(500));
        assert_rx(Duration::from_secs(1));
        assert_rx(Duration::from_secs(2));
        assert_rx(Duration::from_secs(2));
    }
}
