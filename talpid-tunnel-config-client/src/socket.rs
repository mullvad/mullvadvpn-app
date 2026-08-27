//! A TCP stream configured for reliability over throughput.
//!
//! The socket is configured with:
//! - Low MSS (prevents MTU fragmentation)
//! - TCP_NODELAY
//! - TCP keepalives (detect dead connections quickly)
//! - TCP_USER_TIMEOUT on Linux (kernel gives up if data isn't ACKed in time)
//!
//! On non-Windows targets the MSS is lowered via `setsockopt`. On Windows
//! this doesn't work, so the MTU is temporarily lowered instead — see
//! `talpid-wireguard/src/ephemeral.rs`.

use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpSocket as StdTcpSocket;
use tokio::net::TcpStream;

/// Time before TCP starts sending keepalive probes.
const KEEPALIVE_TIME: Duration = Duration::from_secs(5);

/// Time between keepalive probes.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(2);

/// Number of unacknowledged keepalive probes before giving up.
const KEEPALIVE_RETRIES: u32 = 3;

/// Set TCP parameters that prioritize reliability and low latency over throughput.
fn set_reliability_params(socket: &StdTcpSocket) {
    // Send small gRPC frames immediately.
    if let Err(e) = socket.set_nodelay(true) {
        log::error!("Failed to set TCP_NODELAY on tunnel config socket: {e}");
    }

    let sock_ref = socket2::SockRef::from(socket);
    let keepalive = socket2::TcpKeepalive::new()
        .with_time(KEEPALIVE_TIME)
        .with_interval(KEEPALIVE_INTERVAL)
        .with_retries(KEEPALIVE_RETRIES);
    if let Err(e) = sock_ref.set_tcp_keepalive(&keepalive) {
        log::error!("Failed to set TCP keepalive on tunnel config socket: {e}");
    }

    // Linux: set TCP_USER_TIMEOUT so the kernel gives up if our data
    // isn't ACKed within the specified time.
    #[cfg(target_os = "linux")]
    {
        const USER_TIMEOUT_MS: u32 = 30_000;
        if let Err(e) = nix::sys::socket::setsockopt(
            socket,
            nix::sys::socket::sockopt::TcpUserTimeout,
            &USER_TIMEOUT_MS,
        ) {
            log::error!("Failed to set TCP_USER_TIMEOUT on tunnel config socket: {e}");
        }
    }
}

#[cfg(unix)]
mod sys {
    use super::*;

    use nix::sys::socket::{setsockopt, sockopt::TcpMaxSeg};
    use std::os::fd::AsFd;

    /// MTU to set on the tunnel config client socket. We want a low value to prevent fragmentation.
    /// Especially on Android, we've found that the real MTU is often lower than the default MTU, and
    /// we cannot lower it further. This causes the outer packets to be dropped. Also, MTU detection
    /// will likely occur after the PQ handshake, so we cannot assume that the MTU is already
    /// correctly configured.
    /// This is set to the lowest possible IPv4 MTU.
    const CONFIG_CLIENT_MTU: u16 = 576;

    pub struct TcpSocket {
        socket: StdTcpSocket,
    }

    impl TcpSocket {
        pub fn new() -> io::Result<Self> {
            let socket = StdTcpSocket::new_v4()?;
            try_set_tcp_sock_mtu(&socket);
            set_reliability_params(&socket);
            Ok(Self { socket })
        }

        /// Duplicate the underlying kernel socket. The returned handle refers
        /// to the same kernel socket object and keeps it alive.
        pub fn dup(&self) -> io::Result<socket2::Socket> {
            socket2::SockRef::from(&self.socket).try_clone()
        }

        pub async fn connect(self, addr: SocketAddr) -> io::Result<TcpStream> {
            self.socket.connect(addr).await
        }
    }

    fn try_set_tcp_sock_mtu(sock: &impl AsFd) {
        let mss = u32::from(desired_mss());
        log::debug!("Tunnel config TCP socket MSS: {mss}");
        if let Err(e) = setsockopt(sock, TcpMaxSeg, &mss) {
            log::error!("Failed to set MSS on tunnel config TCP socket: {e}");
        };
    }

    const fn desired_mss() -> u16 {
        const IPV4_HEADER_SIZE: u16 = 20;
        const MAX_TCP_HEADER_SIZE: u16 = 60;
        let mtu = CONFIG_CLIENT_MTU.saturating_sub(IPV4_HEADER_SIZE);
        mtu.saturating_sub(MAX_TCP_HEADER_SIZE)
    }
}

#[cfg(windows)]
mod sys {
    use super::*;

    pub struct TcpSocket {
        socket: StdTcpSocket,
    }

    impl TcpSocket {
        pub fn new() -> io::Result<Self> {
            let socket = StdTcpSocket::new_v4()?;
            set_reliability_params(&socket);
            Ok(Self { socket })
        }

        /// Duplicate the underlying kernel socket. The returned handle refers
        /// to the same kernel socket object and keeps it alive.
        pub fn dup(&self) -> io::Result<socket2::Socket> {
            socket2::SockRef::from(&self.socket).try_clone()
        }

        pub async fn connect(self, addr: SocketAddr) -> io::Result<TcpStream> {
            self.socket.connect(addr).await
        }
    }
}

pub use sys::*;
