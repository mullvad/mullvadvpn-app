use crate::{ping_monitor::Pinger, tunnel::wireguard::stats::Stats};
use std::{
    net::Ipv4Addr,
    sync::{mpsc, Mutex, Weak},
    time::{Duration, Instant},
};

use super::{Error, Tunnel};

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
/// Number of seconds to wait between sending ICMP packets
const SECONDS_PER_PING: Duration = Duration::from_secs(3);


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
    last_stats: Stats,
    tx_timestamp: Instant,
    rx_timestamp: Instant,
    initial_ping_timestamp: Option<Instant>,
    num_pings_sent: u32,
    pinger: Pinger,
    close_receiver: mpsc::Receiver<()>,
}

impl ConnectivityMonitor {
    pub fn new(
        addr: Ipv4Addr,
        interface: String,
        tunnel_handle: Weak<Mutex<Option<Box<dyn Tunnel>>>>,
        close_receiver: mpsc::Receiver<()>,
    ) -> Result<Self, Error> {
        let pinger = Pinger::new(addr, interface).map_err(Error::PingError)?;

        let now = Instant::now();

        Ok(Self {
            tunnel_handle,
            last_stats: Default::default(),
            tx_timestamp: now,
            rx_timestamp: now,
            initial_ping_timestamp: None,
            num_pings_sent: 0,
            pinger,
            close_receiver,
        })
    }

    // checks if the tunnel has ever worked. Intended to check if a connection to a tunnel is
    // successfull at the start of a connection.
    pub fn establish_connectivity(&mut self) -> Result<bool, Error> {
        if self.last_stats.rx_bytes > 0 {
            return Ok(true);
        }

        let start = Instant::now();
        while start.elapsed() < PING_TIMEOUT {
            if self.check_connectivity()? {
                return Ok(true);
            }
            if self.should_shut_down(DELAY_ON_INITIAL_SETUP) {
                return Ok(false);
            }
        }
        Ok(false)
    }

    pub fn run(&mut self) -> Result<(), Error> {
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
        while self.check_connectivity()? && !self.should_shut_down(iter_delay) {}
        Ok(())
    }

    /// Returns true if connection is established
    fn check_connectivity(&mut self) -> Result<bool, Error> {
        let now = Instant::now();
        match self.get_stats() {
            None => Ok(false),
            Some(new_stats) => {
                let new_stats = new_stats?;
                let last_stats = self.last_stats;
                self.last_stats = new_stats;

                if new_stats.tx_bytes > last_stats.tx_bytes {
                    self.tx_timestamp = now;
                }

                if new_stats.rx_bytes > last_stats.rx_bytes {
                    self.rx_timestamp = now;
                    // resetting ping
                    self.initial_ping_timestamp = None;
                    self.num_pings_sent = 0;
                    return Ok(true);
                }

                self.maybe_send_ping()?;
                Ok(!self.ping_timed_out() && self.last_stats.rx_bytes > 0)
            }
        }
    }

    /// If None is returned, then the underlying tunnel has already been closed and all subsequent
    /// calls will also return None.
    fn get_stats(&self) -> Option<Result<Stats, Error>> {
        self.tunnel_handle
            .upgrade()?
            .lock()
            .ok()?
            .as_ref()
            .map(|tunnel| tunnel.get_config())
    }

    fn maybe_send_ping(&mut self) -> Result<(), Error> {
        // Only send out a ping if we haven't received a byte in a while or no traffic has flowed
        // in the last 2 minutes, but if a ping already has been sent out, only send one out every
        // 3 seconds.
        if (self.rx_timed_out() || self.traffic_timed_out())
            && self
                .initial_ping_timestamp
                .map(|initial_ping_timestamp| {
                    initial_ping_timestamp.elapsed() / self.num_pings_sent < SECONDS_PER_PING
                })
                .unwrap_or(true)
        {
            self.pinger.send_icmp().map_err(Error::PingError)?;
            if self.initial_ping_timestamp.is_none() {
                self.initial_ping_timestamp = Some(Instant::now());
            }
            self.num_pings_sent += 1;
        }
        Ok(())
    }

    // check if last time data was received is too long ago
    fn rx_timed_out(&self) -> bool {
        // if last sent bytes were sent after last received bytes
        self.tx_timestamp > self.rx_timestamp
            // and the response hasn't been seen for BYTES_RX_TIMEOUT
            && self.rx_timestamp.elapsed() >= BYTES_RX_TIMEOUT
    }

    // check if no bytes have been sent or received in a while
    fn traffic_timed_out(&self) -> bool {
        self.rx_timestamp.elapsed() >= TRAFFIC_TIMEOUT
            || self.tx_timestamp.elapsed() >= TRAFFIC_TIMEOUT
    }

    fn ping_timed_out(&self) -> bool {
        self.initial_ping_timestamp
            .map(|initial_ping_timestamp| initial_ping_timestamp.elapsed() > PING_TIMEOUT)
            .unwrap_or(false)
    }
}
