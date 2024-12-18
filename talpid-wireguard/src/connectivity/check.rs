use std::cmp;
use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::time::{Duration, Instant};

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
pub struct Check<Strategy = Timeout> {
    conn_state: ConnState,
    ping_state: PingState,
    strategy: Strategy,
    retry_attempt: u32,
}

// Define the type state of [Check]
pub(crate) trait Strategy {
    fn should_shut_down(&mut self, timeout: Duration) -> bool;
}

/// An uncancellable [Check] that will run [Check::establish_connectivity] until
/// completion or until it times out.
pub struct Timeout;

impl Strategy for Timeout {
    /// The Timeout strategy cannot receive shut down signals so this function always returns false.
    fn should_shut_down(&mut self, _timeout: Duration) -> bool {
        false
    }
}

/// A cancellable [Check] may be cancelled before it will time out by sending
/// a signal on the channel returned by [Check::with_cancellation]. Otherwise,
/// it behaves as [Timeout].
pub struct Cancellable {
    close_receiver: mpsc::Receiver<()>,
}

impl Strategy for Cancellable {
    /// Returns true if monitor should be shut down
    fn should_shut_down(&mut self, timeout: Duration) -> bool {
        match self.close_receiver.recv_timeout(timeout) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => true,
            Err(mpsc::RecvTimeoutError::Timeout) => false,
        }
    }
}

impl Check<Timeout> {
    pub fn new(
        addr: Ipv4Addr,
        #[cfg(any(target_os = "macos", target_os = "linux"))] interface: String,
        retry_attempt: u32,
    ) -> Result<Check<Timeout>, Error> {
        Ok(Check {
            conn_state: ConnState::new(Instant::now(), Default::default()),
            ping_state: PingState::new(
                addr,
                #[cfg(any(target_os = "macos", target_os = "linux"))]
                interface,
            )?,
            strategy: Timeout,
            retry_attempt,
        })
    }

    /// Cancel a [Check] preemptively by sennding a message on the channel or by dropping
    /// the returned channel.
    pub fn with_cancellation(self) -> (Check<Cancellable>, mpsc::Sender<()>) {
        let (cancellation_tx, cancellation_rx) = mpsc::channel();
        let check = Check {
            conn_state: self.conn_state,
            ping_state: self.ping_state,
            strategy: Cancellable {
                close_receiver: cancellation_rx,
            },
            retry_attempt: self.retry_attempt,
        };
        (check, cancellation_tx)
    }

    #[cfg(test)]
    /// Create a new [Check] with a custom initial state. To use the [Cancellable] strategy,
    /// see [Check::with_cancellation].
    pub(super) fn mock(conn_state: ConnState, ping_state: PingState) -> Self {
        Check {
            conn_state,
            ping_state,
            strategy: Timeout,
            retry_attempt: 0,
        }
    }
}

impl<S: Strategy> Check<S> {
    // checks if the tunnel has ever worked. Intended to check if a connection to a tunnel is
    // successful at the start of a connection.
    pub fn establish_connectivity(&mut self, tunnel_handle: &TunnelType) -> Result<bool, Error> {
        // Send initial ping to prod WireGuard into connecting.
        self.ping_state
            .pinger
            .send_icmp()
            .map_err(Error::PingError)?;
        self.establish_connectivity_inner(
            self.retry_attempt,
            ESTABLISH_TIMEOUT,
            ESTABLISH_TIMEOUT_MULTIPLIER,
            MAX_ESTABLISH_TIMEOUT,
            tunnel_handle,
        )
    }

    pub(crate) fn reset(&mut self, current_iteration: Instant) {
        self.ping_state.reset();
        self.conn_state.reset_after_suspension(current_iteration);
    }

    pub(crate) fn should_shut_down(&mut self, timeout: Duration) -> bool {
        self.strategy.should_shut_down(timeout)
    }

    fn establish_connectivity_inner(
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

        let check_timeout = cmp::min(
            max_timeout,
            timeout_initial.saturating_mul(timeout_multiplier.saturating_pow(retry_attempt)),
        );

        let start = Instant::now();
        while start.elapsed() < check_timeout {
            if self.check_connectivity_interval(Instant::now(), check_timeout, tunnel_handle)? {
                return Ok(true);
            }
            if self.should_shut_down(DELAY_ON_INITIAL_SETUP) {
                return Ok(false);
            }
        }
        Ok(false)
    }

    /// Returns true if connection is established
    pub(crate) fn check_connectivity(
        &mut self,
        now: Instant,
        tunnel_handle: &TunnelType,
    ) -> Result<bool, Error> {
        self.check_connectivity_interval(now, PING_TIMEOUT, tunnel_handle)
    }

    /// Returns true if connection is established
    fn check_connectivity_interval(
        &mut self,
        now: Instant,
        timeout: Duration,
        tunnel_handle: &TunnelType,
    ) -> Result<bool, Error> {
        match Self::get_stats(tunnel_handle).map_err(Error::ConfigReadError)? {
            None => Ok(false),
            Some(new_stats) => {
                if self.conn_state.update(now, new_stats) {
                    self.ping_state.reset();
                    return Ok(true);
                }

                self.maybe_send_ping(now)?;
                Ok(!self.ping_state.ping_timed_out(timeout) && self.conn_state.connected())
            }
        }
    }

    /// If None is returned, then the underlying tunnel has already been closed and all subsequent
    /// calls will also return None.
    ///
    /// NOTE: will panic if called from within a tokio runtime.
    fn get_stats(tunnel_handle: &TunnelType) -> Result<Option<StatsMap>, TunnelError> {
        let stats = tunnel_handle.get_tunnel_stats()?;
        if stats.is_empty() {
            log::error!("Tunnel unexpectedly shut down");
            Ok(None)
        } else {
            Ok(Some(stats))
        }
    }

    fn maybe_send_ping(&mut self, now: Instant) -> Result<(), Error> {
        // Only send out a ping if we haven't received a byte in a while or no traffic has flowed
        // in the last 2 minutes, but if a ping already has been sent out, only send one out every
        // 3 seconds.
        if (self.conn_state.rx_timed_out() || self.conn_state.traffic_timed_out())
            && self
                .ping_state
                .initial_ping_timestamp
                .map(|initial_ping_timestamp| {
                    initial_ping_timestamp.elapsed() / self.ping_state.num_pings_sent
                        < SECONDS_PER_PING
                })
                .unwrap_or(true)
        {
            self.ping_state
                .pinger
                .send_icmp()
                .map_err(Error::PingError)?;
            if self.ping_state.initial_ping_timestamp.is_none() {
                self.ping_state.initial_ping_timestamp = Some(now);
            }
            self.ping_state.num_pings_sent += 1;
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
    fn reset(&mut self) {
        self.initial_ping_timestamp = None;
        self.num_pings_sent = 0;
        self.pinger.reset();
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

    #[test]
    /// Verify that `check_connectivity()` returns `false` if the tunnel is connected and traffic is
    /// not flowing after `BYTES_RX_TIMEOUT` and `PING_TIMEOUT`.
    fn test_ping_times_out() {
        let tunnel = MockTunnel::never_incrementing().boxed();
        let pinger = MockPinger::default();
        let now = Instant::now();
        let start = now
            .checked_sub(BYTES_RX_TIMEOUT + PING_TIMEOUT + Duration::from_secs(10))
            .unwrap();
        let mut checker = mock_checker(start, Box::new(pinger));

        // Mock the state - connectivity has been established
        checker.conn_state = connected_state(start);
        // A ping was sent to verify connectivity
        checker.maybe_send_ping(start).unwrap();
        assert!(!checker.check_connectivity(now, &tunnel).unwrap())
    }

    #[test]
    /// Verify that `check_connectivity()` returns `true` if the tunnel is connected and traffic is
    /// flowing constantly.
    fn test_no_connection_on_start() {
        let tunnel = MockTunnel::never_incrementing().boxed();
        let pinger = MockPinger::default();
        let now = Instant::now();
        let start = now.checked_sub(Duration::from_secs(1)).unwrap();
        let mut monitor = mock_checker(start, Box::new(pinger));

        assert!(!monitor.check_connectivity(now, &tunnel).unwrap())
    }

    #[test]
    /// Verify that `check_connectivity()` returns `true` if the tunnel is connected and traffic is
    /// flowing constantly.
    fn test_connection_works() {
        let tunnel = MockTunnel::always_incrementing().boxed();
        let pinger = MockPinger::default();
        let now = Instant::now();
        let start = now.checked_sub(Duration::from_secs(1)).unwrap();
        let mut monitor = mock_checker(start, Box::new(pinger));

        // Mock the state - connectivity has been established
        monitor.conn_state = connected_state(start);

        assert!(monitor.check_connectivity(now, &tunnel).unwrap())
    }

    #[test]
    /// Verify that the timeout for setting up a tunnel works as expected.
    fn test_establish_timeout() {
        let pinger = MockPinger::default();
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

        let (result_tx, result_rx) = mpsc::channel();

        std::thread::spawn(move || {
            let now = Instant::now();
            let start = now.checked_sub(Duration::from_secs(1)).unwrap();
            let mut monitor = mock_checker(start, Box::new(pinger));

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
                        &tunnel,
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
