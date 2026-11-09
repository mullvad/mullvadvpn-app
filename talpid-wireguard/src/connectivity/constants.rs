use std::time::Duration;

/// Timeout for waiting on receiving traffic after sending outgoing traffic.  Once this timeout is
/// hit, a ping will be sent every `SECONDS_PER_PING` until `PING_TIMEOUT` is reached, or traffic
/// is received.
pub(crate) const BYTES_RX_TIMEOUT: Duration = Duration::from_secs(5);
/// Timeout for waiting on receiving or sending any traffic.  Once this timeout is hit, a ping will
/// be sent every `SECONDS_PER_PING` until `PING_TIMEOUT` is reached or traffic is received.
pub(crate) const TRAFFIC_TIMEOUT: Duration = Duration::from_secs(120);
/// Timeout for waiting on receiving traffic after sending the first ICMP packet.  Once this
/// timeout is reached, it is assumed that the connection is lost.
pub(crate) const PING_TIMEOUT: Duration = Duration::from_secs(15);
/// Timeout for receiving traffic when establishing a connection.
pub(crate) const ESTABLISH_TIMEOUT: Duration = Duration::from_secs(4);
/// Timeout for receiving traffic when establishing a connection, after the first attempt failed.
/// This is long enough for WireGuard to retransmit a lost handshake initiation, and for slow links
/// that need a few round trips to set up obfuscation before the handshake.
pub(crate) const RETRY_ESTABLISH_TIMEOUT: Duration = Duration::from_secs(8);
/// Number of seconds to wait between sending ICMP packets
pub(crate) const SECONDS_PER_PING: Duration = Duration::from_secs(3);
