//! A TCP stream with a low MSS set. This prevents incorrectly configured MTU from causing
//! fragmentation/packet loss. This is only supported on non-Windows targets.
//!
//! On windows this solution does not work. So on Windows we instead temporarily lower the MTU
//! while negotiating the ephemeral peer. This code lives in `talpid-wireguard/src/ephemeral.rs`
//! These two solutions to the same problem should probably be refactored to end up closer
//! to each other.

use std::io;
use std::net::SocketAddr;
use tokio::net::TcpSocket as StdTcpSocket;
use tokio::net::TcpStream;

#[cfg(unix)]
mod sys {
    use super::*;

    pub use libc::{setsockopt, socklen_t, IPPROTO_TCP, TCP_MAXSEG};
    use std::ffi::c_int;
    pub use std::os::fd::AsRawFd;

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
            Ok(Self { socket })
        }

        pub async fn connect(self, addr: SocketAddr) -> io::Result<TcpStream> {
            self.socket.connect(addr).await
        }
    }

    fn try_set_tcp_sock_mtu(sock: &impl AsRawFd) {
        let mss = c_int::from(desired_mss());

        log::debug!("Tunnel config TCP socket MSS: {mss}");

        // TODO: replace with nix when TcpMaxSeg is added for macos
        // SAFETY: `mss` is a valid pointer to a c_int.
        let result = unsafe {
            setsockopt(
                sock.as_raw_fd(),
                IPPROTO_TCP,
                TCP_MAXSEG,
                &mss as *const _ as _,
                socklen_t::try_from(std::mem::size_of_val(&mss)).unwrap(),
            )
        };
        if result != 0 {
            log::error!(
                "Failed to set MSS on tunnel config TCP socket: {}",
                std::io::Error::last_os_error()
            );
        }
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
            Ok(Self {
                socket: StdTcpSocket::new_v4()?,
            })
        }

        pub async fn connect(self, addr: SocketAddr) -> io::Result<TcpStream> {
            self.socket.connect(addr).await
        }
    }
}

pub use sys::*;
