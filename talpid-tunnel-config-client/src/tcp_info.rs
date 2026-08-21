//! TCP-level diagnostics for the tunnel config client socket.
//!
//! Provides a way to query kernel TCP state (RTT, retransmits, bytes transferred,
//! connection state) from a socket that is still alive. This is used to enrich
//! log messages when ephemeral peer negotiation times out.
//!
//! On Unix, kernel TCP state is queried via [`nix`]'s safe `getsockopt` wrapper
//! (which internally calls `libc::getsockopt`). The only `unsafe` in this module
//! is [`std::os::fd::BorrowedFd::borrow_raw`] to convert the stored raw fd back
//! to a borrowed fd, which is safe as long as the underlying socket is still
//! alive (guaranteed by the `select!` + `pin!` pattern in `ephemeral.rs`).
//!
//! On Windows, no safe wrapper exists for `WSAIoctl(SIO_TCP_INFO)`, so a small
//! `unsafe` block is used, consistent with the rest of the codebase.

use std::fmt;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Instant;

/// Shared diagnostics handle that survives the future being dropped by a timeout.
///
/// The raw socket handle is set by `TcpSocket::new` when the socket
/// is created. On timeout, the caller can query kernel TCP info via
/// [`query_tcp_info`]. Byte counts come from the kernel (`TCP_INFO`), not from
/// application-level tracking.
#[derive(Debug)]
pub struct TcpDiagnostics {
    /// Raw socket file descriptor (Unix) or socket handle (Windows).
    /// Set by `TcpSocket::new`. -1 means "not yet set".
    raw_socket: AtomicI64,
    /// When the connection attempt started.
    start_time: Instant,
}

impl Default for TcpDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpDiagnostics {
    pub fn new() -> Self {
        Self {
            raw_socket: AtomicI64::new(-1),
            start_time: Instant::now(),
        }
    }

    /// Store the raw socket handle. Called by `TcpSocket::new`.
    pub fn set_raw_socket(&self, raw: i64) {
        self.raw_socket.store(raw, Ordering::Relaxed);
    }

    /// Elapsed time since the connection attempt started.
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// The raw socket handle, if set.
    fn raw_socket(&self) -> Option<i64> {
        let raw = self.raw_socket.load(Ordering::Relaxed);
        if raw == -1 { None } else { Some(raw) }
    }
}

/// Normalized TCP state, platform-independent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    /// The socket has not started connecting yet.
    Closed,
    /// SYN sent, waiting for SYN-ACK (handshake in progress).
    SynSent,
    /// SYN received, waiting for final ACK.
    SynReceived,
    /// Connection established.
    Established,
    /// FIN sent, waiting for ACK.
    FinWait1,
    /// FIN sent and ACKed, waiting for FIN from peer.
    FinWait2,
    /// Peer sent FIN, waiting for us to close.
    CloseWait,
    /// Both sides sent FIN, waiting for final ACKs.
    Closing,
    /// Received FIN, sent ACK, waiting for final ACK.
    LastAck,
    /// Waiting for enough time to pass to be sure the peer received our ACK.
    TimeWait,
    /// Listening (not applicable for client sockets).
    Listen,
    /// Unknown or unrecognized state.
    Unknown(i32),
}

impl fmt::Display for TcpState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => write!(f, "CLOSED"),
            Self::SynSent => write!(f, "SYN_SENT"),
            Self::SynReceived => write!(f, "SYN_RECEIVED"),
            Self::Established => write!(f, "ESTABLISHED"),
            Self::FinWait1 => write!(f, "FIN_WAIT_1"),
            Self::FinWait2 => write!(f, "FIN_WAIT_2"),
            Self::CloseWait => write!(f, "CLOSE_WAIT"),
            Self::Closing => write!(f, "CLOSING"),
            Self::LastAck => write!(f, "LAST_ACK"),
            Self::TimeWait => write!(f, "TIME_WAIT"),
            Self::Listen => write!(f, "LISTEN"),
            Self::Unknown(v) => write!(f, "UNKNOWN({v})"),
        }
    }
}

/// Normalized TCP diagnostics snapshot, populated from platform-specific APIs.
#[derive(Debug)]
pub struct TcpInfoSnapshot {
    /// TCP connection state.
    pub state: TcpState,
    /// Smoothed round-trip time in microseconds.
    pub rtt_us: u32,
    /// Bytes sent (kernel-level).
    pub bytes_sent: u64,
    /// Bytes received (kernel-level).
    pub bytes_received: u64,
    /// Total retransmissions.
    pub retransmits: u32,
    /// Congestion window size.
    pub cwnd: u32,
    /// RTT variance in microseconds.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub rtt_var_us: u32,
    /// SYN retransmissions (handshake phase).
    #[cfg(target_os = "windows")]
    pub syn_retransmits: u32,
    /// RTO timeout episodes.
    #[cfg(target_os = "windows")]
    pub timeout_episodes: u32,
    /// Bytes currently in flight (sent but not yet ACKed).
    #[cfg(target_os = "windows")]
    pub bytes_in_flight: u32,
}

/// Query TCP info from the socket stored in the diagnostics handle.
///
/// This should be called while the socket is still alive (i.e., before the
/// future containing the socket is dropped). Returns `None` if the socket
/// handle hasn't been set yet or if the query fails.
pub fn query_tcp_info(handle: &TcpDiagnostics) -> Option<TcpInfoSnapshot> {
    let raw = handle.raw_socket()?;
    platform::query(raw)
}

// ---------------------------------------------------------------------------
// Platform implementations
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::os::fd::BorrowedFd;
    // Both macros are needed: `sockopt_impl!` internally calls `getsockopt_impl!`.
    use nix::{getsockopt_impl, sockopt_impl};

    nix::sockopt_impl!(
        TcpInfoOpt,
        GetOnly,
        libc::SOL_TCP,
        libc::TCP_INFO,
        libc::tcp_info
    );

    pub fn query(raw_fd: i64) -> Option<TcpInfoSnapshot> {
        // SAFETY: The raw fd is valid because the socket is still alive in the
        // pinned future. The `BorrowedFd` is dropped before this function
        // returns, so it does not outlive the socket.
        let fd = unsafe { BorrowedFd::borrow_raw(raw_fd as i32) };
        let info = nix::sys::socket::getsockopt(&fd, TcpInfoOpt)
            .map_err(|err| {
                log::debug!("Failed to query TCP_INFO: {err}");
            })
            .ok()?;

        Some(TcpInfoSnapshot {
            state: linux_tcp_state(info.tcpi_state),
            rtt_us: info.tcpi_rtt,
            bytes_sent: info.tcpi_bytes_acked,
            bytes_received: info.tcpi_bytes_received,
            retransmits: info.tcpi_total_retrans,
            cwnd: info.tcpi_snd_cwnd,
            rtt_var_us: info.tcpi_rttvar,
        })
    }

    fn linux_tcp_state(state: u8) -> TcpState {
        // Linux TCP states from include/net/tcp_states.h
        match state {
            1 => TcpState::Established,
            2 => TcpState::SynSent,
            3 => TcpState::SynReceived,
            4 => TcpState::FinWait1,
            5 => TcpState::FinWait2,
            6 => TcpState::TimeWait,
            7 => TcpState::Closed,
            8 => TcpState::CloseWait,
            9 => TcpState::LastAck,
            10 => TcpState::Listen,
            11 => TcpState::Closing,
            v => TcpState::Unknown(i32::from(v)),
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use nix::{getsockopt_impl, sockopt_impl};
    use std::os::fd::BorrowedFd;

    nix::sockopt_impl!(
        TcpConnectionInfoOpt,
        GetOnly,
        libc::IPPROTO_TCP,
        libc::TCP_CONNECTION_INFO,
        libc::tcp_connection_info
    );

    pub fn query(raw_fd: i64) -> Option<TcpInfoSnapshot> {
        // SAFETY: The raw fd is valid because the socket is still alive in the
        // pinned future in `ephemeral.rs`. The `select!` + `pin!` pattern
        // ensures the future (which owns the `TcpStream`) is not dropped until
        // after this function returns. The `BorrowedFd` is dropped at the end
        // of this scope, so it does not outlive the socket.
        let fd = unsafe { BorrowedFd::borrow_raw(raw_fd as i32) };
        let info = nix::sys::socket::getsockopt(&fd, TcpConnectionInfoOpt)
            .map_err(|err| {
                log::debug!("Failed to query TCP_CONNECTION_INFO: {err}");
            })
            .ok()?;

        Some(TcpInfoSnapshot {
            state: macos_tcp_state(info.tcpi_state),
            rtt_us: info.tcpi_srtt,
            bytes_sent: info.tcpi_txbytes,
            bytes_received: info.tcpi_rxbytes,
            retransmits: u32::try_from(info.tcpi_txretransmitbytes).unwrap_or(u32::MAX),
            cwnd: info.tcpi_snd_cwnd,
            rtt_var_us: info.tcpi_rttvar,
        })
    }

    fn macos_tcp_state(state: u8) -> TcpState {
        // macOS TCP states from <netinet/tcp_fsm.h>
        match state {
            0 => TcpState::Closed,
            1 => TcpState::Listen,
            2 => TcpState::SynSent,
            3 => TcpState::SynReceived,
            4 => TcpState::Established,
            5 => TcpState::CloseWait,
            6 => TcpState::FinWait1,
            7 => TcpState::Closing,
            8 => TcpState::LastAck,
            9 => TcpState::FinWait2,
            10 => TcpState::TimeWait,
            v => TcpState::Unknown(i32::from(v)),
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use windows_sys::Win32::Networking::WinSock::{
        SIO_TCP_INFO, SOCKET, TCP_INFO_v0, TCPSTATE, WSAIoctl,
    };

    pub fn query(raw_socket: i64) -> Option<TcpInfoSnapshot> {
        let socket = raw_socket as SOCKET;
        let mut info: TCP_INFO_v0 = TCP_INFO_v0::default();
        let mut bytes_returned: u32 = 0;

        // SIO_TCP_INFO requires an input buffer containing a ULONG version
        // number (0 for TCP_INFO_v0).
        let version: u32 = 0;

        let ret = unsafe {
            // SAFETY: The socket handle is valid because the socket is still
            // alive in the pinned future in `ephemeral.rs` — the `select!` +
            // `pin!` pattern ensures the future (which owns the `TcpStream`)
            // is not dropped until after this function returns. The input and
            // output buffers are stack-allocated and properly sized. We pass
            // `null` for `lpOverlapped` and `lpCompletionRoutine` because this
            // is a synchronous (blocking) call.
            WSAIoctl(
                socket,
                SIO_TCP_INFO,
                &raw const version as *const core::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
                &raw mut info as *mut core::ffi::c_void,
                std::mem::size_of::<TCP_INFO_v0>() as u32,
                &raw mut bytes_returned,
                std::ptr::null_mut(),
                None,
            )
        };

        if ret != 0 {
            let err = std::io::Error::last_os_error();
            // WSAEINVAL (10022) is returned when the socket is not connected.
            // Unlike Linux's TCP_INFO, Windows SIO_TCP_INFO does not return
            // info for unconnected sockets (e.g. SYN_SENT state).
            if err.raw_os_error() == Some(10022) {
                log::debug!("TCP_INFO unavailable: socket not connected");
            } else {
                log::debug!("Failed to query TCP_INFO via WSAIoctl: {err}");
            }
            return None;
        }

        Some(TcpInfoSnapshot {
            state: windows_tcp_state(info.State),
            rtt_us: info.RttUs,
            bytes_sent: info.BytesOut,
            bytes_received: info.BytesIn,
            retransmits: info.BytesRetrans,
            cwnd: info.Cwnd,
            syn_retransmits: u32::from(info.SynRetrans),
            timeout_episodes: info.TimeoutEpisodes,
            bytes_in_flight: info.BytesInFlight,
        })
    }

    fn windows_tcp_state(state: TCPSTATE) -> TcpState {
        use windows_sys::Win32::Networking::WinSock::*;
        match state {
            TCPSTATE_CLOSED => TcpState::Closed,
            TCPSTATE_LISTEN => TcpState::Listen,
            TCPSTATE_SYN_SENT => TcpState::SynSent,
            TCPSTATE_SYN_RCVD => TcpState::SynReceived,
            TCPSTATE_ESTABLISHED => TcpState::Established,
            TCPSTATE_FIN_WAIT_1 => TcpState::FinWait1,
            TCPSTATE_FIN_WAIT_2 => TcpState::FinWait2,
            TCPSTATE_CLOSE_WAIT => TcpState::CloseWait,
            TCPSTATE_CLOSING => TcpState::Closing,
            TCPSTATE_LAST_ACK => TcpState::LastAck,
            TCPSTATE_TIME_WAIT => TcpState::TimeWait,
            v => TcpState::Unknown(v),
        }
    }
}

// Fallback for platforms not covered above (Android, iOS, etc.)
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod platform {
    use super::*;

    pub fn query(_raw: i64) -> Option<TcpInfoSnapshot> {
        log::debug!("TCP info querying not supported on this platform");
        None
    }
}
