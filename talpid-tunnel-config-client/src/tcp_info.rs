//! TCP-level diagnostics for the tunnel config client socket.
//!
//! Queries kernel TCP state (RTT, retransmits, bytes transferred, connection
//! state) from a socket. Used for diagnostics when ephemeral peer
//! negotiation times out.

use std::fmt;

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

/// Query TCP info from the given socket. Returns `None` if the query fails.
#[cfg(not(target_os = "ios"))]
pub fn query_tcp_info(socket: &crate::socket::TcpSocket) -> Option<TcpInfoSnapshot> {
    let sock_ref = socket2::SockRef::from(socket);
    platform::query(&sock_ref)
}

// ---------------------------------------------------------------------------
// Platform implementations
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    // Both macros are needed: `sockopt_impl!` internally calls `getsockopt_impl!`.
    use nix::{getsockopt_impl, sockopt_impl};

    nix::sockopt_impl!(
        TcpInfoOpt,
        GetOnly,
        libc::SOL_TCP,
        libc::TCP_INFO,
        libc::tcp_info
    );

    pub fn query(socket: &socket2::Socket) -> Option<TcpInfoSnapshot> {
        let info = nix::sys::socket::getsockopt(socket, TcpInfoOpt)
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

    nix::sockopt_impl!(
        TcpConnectionInfoOpt,
        GetOnly,
        libc::IPPROTO_TCP,
        libc::TCP_CONNECTION_INFO,
        libc::tcp_connection_info
    );

    pub fn query(socket: &socket2::Socket) -> Option<TcpInfoSnapshot> {
        let info = nix::sys::socket::getsockopt(socket, TcpConnectionInfoOpt)
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
    use std::os::windows::io::AsRawSocket;
    use windows_sys::Win32::Networking::WinSock::{
        SIO_TCP_INFO, SOCKET, TCP_INFO_v0, TCPSTATE, WSAIoctl,
    };

    pub fn query(socket: &socket2::Socket) -> Option<TcpInfoSnapshot> {
        let raw = socket.as_raw_socket() as SOCKET;
        let mut info: TCP_INFO_v0 = TCP_INFO_v0::default();
        let mut bytes_returned: u32 = 0;

        // SIO_TCP_INFO requires an input buffer containing a ULONG version
        // number (0 for TCP_INFO_v0).
        let version: u32 = 0;

        let ret = unsafe {
            // SAFETY: `raw` is a valid handle owned by the `socket2::Socket`
            // argument. Buffers are stack-allocated and properly sized.
            // `lpOverlapped` and `lpCompletionRoutine` are null (synchronous call).
            WSAIoctl(
                raw,
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

// Fallback for platforms not covered above (Android, etc.). iOS does not use
// TCP info querying, so it's excluded to avoid dead code.
#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "ios"
)))]
mod platform {
    use super::*;

    pub fn query(_socket: &socket2::Socket) -> Option<TcpInfoSnapshot> {
        log::debug!("TCP info querying not supported on this platform");
        None
    }
}
