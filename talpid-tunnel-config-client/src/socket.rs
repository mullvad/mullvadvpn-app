//! A TCP stream configured for reliability over throughput.
//!
//! On non-Windows targets the MSS is lowered via `setsockopt`. On Windows
//! this doesn't work, so the MTU is temporarily lowered instead — see
//! `talpid-wireguard/src/ephemeral.rs`.

use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpSocket as TokioTcpSocket;
use tokio::net::TcpStream;

/// Time before TCP starts sending keepalive probes.
const KEEPALIVE_TIME: Duration = Duration::from_secs(1);

/// Time between keepalive probes.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(1);

/// Number of unacknowledged keepalive probes before giving up.
///
/// Together with the one-second idle time and interval, this gives an
/// unresponsive peer roughly eleven seconds to recover. On Linux,
/// `TCP_USER_TIMEOUT` takes precedence.
const KEEPALIVE_RETRIES: u32 = 10;

/// TCP socket with parameters that prioritize reliability.
///
/// This has various platform specific parameters to make the ephemeral peer handshake
/// more reliable under poor network conditions.
///
/// This is achieved by increasing retransmissions and tuning timeout parameters, so that
/// the session is kept alive for longer if progress is made, or shorter for dead
/// connections.
pub struct TcpSocket {
    socket: TokioTcpSocket,
}

impl TcpSocket {
    pub fn new() -> io::Result<Self> {
        let socket = TokioTcpSocket::new_v4()?;
        // Send small gRPC frames immediately.
        if let Err(e) = socket.set_nodelay(true) {
            log_socket_option_error("TCP_NODELAY", e);
        }

        let sock_ref = socket2::SockRef::from(&socket);
        // Increase keepalive frequency. This only affects the connection
        // when no data is being sent, i.e. after sending the request and
        // while waiting for the response.
        let keepalive = socket2::TcpKeepalive::new()
            .with_time(KEEPALIVE_TIME)
            .with_interval(KEEPALIVE_INTERVAL)
            .with_retries(KEEPALIVE_RETRIES);
        if let Err(e) = sock_ref.set_tcp_keepalive(&keepalive) {
            log_socket_option_error("TCP keepalive", e);
        }

        #[cfg(unix)]
        try_set_tcp_sock_mtu(&socket);

        #[cfg(target_os = "linux")]
        set_linux_reliability_params(&socket);

        #[cfg(target_os = "macos")]
        set_macos_reliability_params(&socket);
        Ok(Self { socket })
    }

    /// Clone this socket into a new connectable `TcpSocket` sharing the
    /// same kernel socket object (and its configured options). Each clone
    /// can be connected independently.
    pub fn try_clone(&self) -> io::Result<Self> {
        let dup = socket2::SockRef::from(&self.socket).try_clone()?;
        // `from_std_stream` expects an unconnected, nonblocking socket.
        dup.set_nonblocking(true)?;
        let socket = TokioTcpSocket::from_std_stream(std::net::TcpStream::from(dup));
        Ok(Self { socket })
    }

    pub async fn connect(self, addr: SocketAddr) -> io::Result<TcpStream> {
        self.socket.connect(addr).await
    }

    pub fn query_tcp_info(&self) -> Option<tcp_info::TcpInfoSnapshot> {
        let sock_ref = socket2::SockRef::from(self);
        tcp_info::query_tcp_info(&sock_ref)
    }
}

#[cfg(unix)]
impl std::os::fd::AsFd for TcpSocket {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.socket.as_fd()
    }
}

#[cfg(windows)]
impl std::os::windows::io::AsSocket for TcpSocket {
    fn as_socket(&self) -> std::os::windows::io::BorrowedSocket<'_> {
        self.socket.as_socket()
    }
}

/// Sets the following TCP options to increase reliability:
///
/// - `TCP_RTO_MAX_MS` sets an upper bound on time between retries for unAck'ed data.
/// - `TCP_SYNCNT` sets the number of retries for the `SYN` packet, i.e. handshake inits.
/// - `TCP_USER_TIMEOUT` bounds how long transmitted data may remain unACK'ed.
///
/// Together with the keepalives, which only prove the connection when no data is exchanged,
/// they should cover the three phases: handshake, sending the `EphemeralPeerRequestV1` and
/// receiving the `EphemeralPeerResponseV1`.
#[cfg(target_os = "linux")]
fn set_linux_reliability_params(socket: &TokioTcpSocket) {
    use nix::sys::socket::sockopt::TcpUserTimeout;

    // SYN retransmits before aborting the handshake. The default is 6. Keep this above
    // the default so that the one-second RTO cap does not exhaust the retry count before
    // TCP_USER_TIMEOUT.
    const TCP_SYNCNT_VALUE: u32 = 10;

    // Cap on the retransmit timeout. Default is 120 seconds.
    // Only supported on Linux 6.15+.
    const TCP_RTO_MAX_MS_VALUE: u32 = 1_000;

    // On Linux 6.15+, TCP_SYNCNT and the RTO cap allow roughly ten rapid SYN
    // retransmissions. On older kernels, TCP_USER_TIMEOUT bounds the handshake.

    // Progress-based deadline (ms) for unACK'ed data: measured from the oldest
    // unACK'ed data, so it advances on ACK progress and only fires when there is
    // none. On Linux kernels older than 4.19, this does not apply to the TCP handshake.
    const USER_TIMEOUT_MS: u32 = 12_000;

    set_socket_option(socket, linux_sockopt::TcpSyncnt, &TCP_SYNCNT_VALUE);
    set_socket_option(socket, linux_sockopt::TcpRtoMaxMs, &TCP_RTO_MAX_MS_VALUE);
    set_socket_option(socket, TcpUserTimeout, &USER_TIMEOUT_MS);
}

/// nix 0.31 sockopt types for `TCP_SYNCNT` and `TCP_RTO_MAX_MS`.
#[cfg(target_os = "linux")]
mod linux_sockopt {
    use nix::{getsockopt_impl, setsockopt_impl, sockopt_impl};

    sockopt_impl!(TcpSyncnt, Both, libc::IPPROTO_TCP, libc::TCP_SYNCNT, u32);

    // Requires Linux 6.15+ and is not yet exposed by libc.
    const TCP_RTO_MAX_MS: libc::c_int = 44;

    sockopt_impl!(TcpRtoMaxMs, Both, libc::IPPROTO_TCP, TCP_RTO_MAX_MS, u32);
}

/// Sets the following macOS-specific TCP options to increase reliability:
///
/// - `TCP_CONNECTIONTIMEOUT`: timeout (seconds) for the handshake phase.
/// - `TCP_RXT_CONNDROPTIME`: time (seconds) after which a connection is dropped
///   during a retransmission episode. ACK progress resets the episode. Keepalives
///   cover the idle wait for the response.
#[cfg(target_os = "macos")]
fn set_macos_reliability_params(socket: &TokioTcpSocket) {
    // Handshake timeout (seconds). Default is system-wide (net.inet.tcp.keepinit,
    // ~75 s). Set lower to fail fast on a dead relay.
    const TCP_CONNECTIONTIMEOUT_VALUE: u32 = 10;

    // Deadline (seconds) for a retransmission episode in the established phase.
    // ACK progress resets the episode.
    const TCP_RXT_CONNDROPTIME_VALUE: u32 = 12;

    set_socket_option(
        socket,
        macos_sockopt::TcpConnectionTimeout,
        &TCP_CONNECTIONTIMEOUT_VALUE,
    );

    set_socket_option(
        socket,
        macos_sockopt::TcpRxtConnDropTime,
        &TCP_RXT_CONNDROPTIME_VALUE,
    );
}

/// nix 0.31 does not expose sockopt types for `TCP_CONNECTIONTIMEOUT` or
/// `TCP_RXT_CONNDROPTIME`. Neither constant is in libc, so they are declared
/// locally.
#[cfg(target_os = "macos")]
mod macos_sockopt {
    use nix::{getsockopt_impl, setsockopt_impl, sockopt_impl};

    // From <netinet/tcp.h> on macOS.
    const TCP_CONNECTIONTIMEOUT: libc::c_int = 0x20;
    const TCP_RXT_CONNDROPTIME: libc::c_int = 0x80;

    sockopt_impl!(
        TcpConnectionTimeout,
        Both,
        libc::IPPROTO_TCP,
        TCP_CONNECTIONTIMEOUT,
        u32
    );

    sockopt_impl!(
        TcpRxtConnDropTime,
        Both,
        libc::IPPROTO_TCP,
        TCP_RXT_CONNDROPTIME,
        u32
    );
}

/// Set a nix socket option, logging unsupported options quietly while
/// surfacing invalid values and other unexpected failures.
#[cfg(unix)]
fn set_socket_option<F, O>(socket: &F, option: O, value: &O::Val)
where
    F: std::os::fd::AsFd,
    O: nix::sys::socket::SetSockOpt + std::fmt::Debug,
{
    if let Err(error) = option.set(socket, value) {
        log_socket_option_error(&format!("{option:?}"), error);
    }
}

/// Log unavailable socket options quietly while surfacing invalid values and
/// other unexpected failures.
fn log_socket_option_error(option: &str, error: impl Into<io::Error>) {
    let error = error.into();
    if error.raw_os_error().is_some_and(unsupported_socket_option) {
        log::debug!("Socket option {option} is not supported: {error}");
    } else {
        log::warn!("Failed to set socket option {option}: {error}");
    }
}

#[cfg(unix)]
fn unsupported_socket_option(code: i32) -> bool {
    code == libc::ENOPROTOOPT || code == libc::EOPNOTSUPP
}

#[cfg(windows)]
fn unsupported_socket_option(code: i32) -> bool {
    use windows_sys::Win32::Networking::WinSock::{WSAENOPROTOOPT, WSAEOPNOTSUPP};
    code == WSAENOPROTOOPT || code == WSAEOPNOTSUPP
}

/// MTU to set on the tunnel config client socket. We want a low value to prevent fragmentation.
/// Especially on Android, we've found that the real MTU is often lower than the default MTU, and
/// we cannot lower it further. This causes the outer packets to be dropped. Also, MTU detection
/// will likely occur after the PQ handshake, so we cannot assume that the MTU is already
/// correctly configured.
/// This is set to the lowest possible IPv4 MTU.
#[cfg(unix)]
const CONFIG_CLIENT_MTU: u16 = 576;

#[cfg(unix)]
fn try_set_tcp_sock_mtu(sock: &impl std::os::fd::AsFd) {
    use nix::sys::socket::sockopt::TcpMaxSeg;

    let mss = u32::from(desired_mss());
    log::debug!("Tunnel config TCP socket MSS: {mss}");
    set_socket_option(sock, TcpMaxSeg, &mss);
}

#[cfg(unix)]
const fn desired_mss() -> u16 {
    const IPV4_HEADER_SIZE: u16 = 20;
    const MAX_TCP_HEADER_SIZE: u16 = 60;
    let mtu = CONFIG_CLIENT_MTU.saturating_sub(IPV4_HEADER_SIZE);
    mtu.saturating_sub(MAX_TCP_HEADER_SIZE)
}

/// TCP-level diagnostics for the tunnel config client socket.
///
/// Queries kernel TCP state (RTT, retransmits, bytes transferred, connection
/// state) from a socket. Used for diagnostics when ephemeral peer
/// negotiation times out.
pub mod tcp_info {
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

    /// Platform-specific TCP diagnostics snapshot, populated from the
    /// platform's kernel TCP info API.
    #[cfg(not(any(target_os = "ios", target_os = "tvos")))]
    pub type TcpInfoSnapshot = platform::TcpInfoSnapshot;

    /// Query TCP info from the given socket. Returns `None` if the query fails.
    #[cfg(not(any(target_os = "ios", target_os = "tvos")))]
    pub fn query_tcp_info(socket: &socket2::Socket) -> Option<TcpInfoSnapshot> {
        platform::query(socket)
    }

    /// Call `getsockopt` for a TCP info struct, reading into a zeroed `T`.
    ///
    /// `used_bytes` is the number of bytes needed to fill the fields actually
    /// read by the caller (computed via `offset_of!` on the last used field).
    /// A warning is only emitted if the kernel returns fewer bytes than this,
    /// meaning some fields the caller reads will be zero.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn getsockopt_tcp_struct<T>(
        socket: &socket2::Socket,
        level: libc::c_int,
        optname: libc::c_int,
        used_bytes: usize,
    ) -> Option<T> {
        use std::os::fd::AsRawFd;

        // SAFETY: T is a plain struct of integers, all valid when zeroed.
        let mut buf: T = unsafe { std::mem::zeroed() };
        let mut len = std::mem::size_of::<T>() as libc::socklen_t;

        // SAFETY: `buf` is a valid pointer to a zeroed buffer, `len` is its
        // size. The kernel writes at most `len` bytes and updates `len`.
        let ret = unsafe {
            libc::getsockopt(
                socket.as_raw_fd(),
                level,
                optname,
                std::ptr::addr_of_mut!(buf).cast(),
                std::ptr::addr_of_mut!(len),
            )
        };

        if ret != 0 {
            log::debug!(
                "Failed to query TCP info: {}",
                std::io::Error::last_os_error()
            );
            return None;
        }

        if (len as usize) < used_bytes {
            log::warn!(
                "Partial TCP info: kernel returned {len} bytes, \
                 need {used_bytes} for the fields in use; \
                 some fields will be zero"
            );
        }

        Some(buf)
    }

    // ---------------------------------------------------------------------------
    // Platform implementations
    // ---------------------------------------------------------------------------

    #[cfg(target_os = "linux")]
    mod platform {
        use super::*;

        /// TCP congestion algorithm state (Linux's `tcpi_ca_state`).
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum CaState {
            /// Open — no congestion, normal slow-start / congestion avoidance.
            Open,
            /// Disorder — duplicate ACKs or SACKs observed, cwnd may be reduced.
            Disorder,
            /// CWR — congestion window reduced (ECN or other signal).
            Cwr,
            /// Recovery — fast recovery after loss.
            Recovery,
            /// Loss — RTO fired, connection backed off.
            Loss,
            /// Unknown or unrecognized state.
            Unknown(u8),
        }

        /// Subset of  Linux's `TCP_INFO`.
        #[derive(Debug)]
        pub struct TcpInfoSnapshot {
            /// TCP connection state.
            pub state: TcpState,
            /// Congestion control state.
            pub ca_state: CaState,
            /// Smoothed round-trip time in microseconds.
            pub rtt_us: u32,
            /// Current retransmit timeout in microseconds.
            pub rto_us: u32,
            /// Packets declared lost.
            pub lost: u32,
            /// Packets currently in flight (sent but not yet ACKed).
            pub unacked: u32,
            /// Retransmits currently in flight.
            pub retrans_in_flight: u32,
            /// Bytes acknowledged by the peer.
            pub bytes_acked: u64,
            /// Bytes received.
            pub bytes_received: u64,
            /// Total retransmitted segments.
            pub retransmits: u32,
            /// Congestion window size in segments (MSS-sized packets).
            pub cwnd_segments: u32,
            /// Slow-start threshold in segments.
            pub ssthresh: u32,
            /// RTT variance in microseconds.
            pub rtt_var_us: u32,
            /// Milliseconds since the last data segment was received.
            pub last_data_recv_ms: u32,
        }

        pub fn query(socket: &socket2::Socket) -> Option<TcpInfoSnapshot> {
            // The last field we read is `tcpi_bytes_received` (u64). Only warn
            // if the kernel didn't return enough bytes to fill it.
            let used_bytes = std::mem::offset_of!(libc::tcp_info, tcpi_bytes_received)
                + std::mem::size_of::<u64>();
            let info = getsockopt_tcp_struct::<libc::tcp_info>(
                socket,
                libc::SOL_TCP,
                libc::TCP_INFO,
                used_bytes,
            )?;

            Some(TcpInfoSnapshot {
                state: linux_tcp_state(info.tcpi_state),
                ca_state: linux_ca_state(info.tcpi_ca_state),
                rtt_us: info.tcpi_rtt,
                rto_us: info.tcpi_rto,
                lost: info.tcpi_lost,
                unacked: info.tcpi_unacked,
                retrans_in_flight: info.tcpi_retrans,
                bytes_acked: info.tcpi_bytes_acked,
                bytes_received: info.tcpi_bytes_received,
                retransmits: info.tcpi_total_retrans,
                cwnd_segments: info.tcpi_snd_cwnd,
                ssthresh: info.tcpi_snd_ssthresh,
                rtt_var_us: info.tcpi_rttvar,
                last_data_recv_ms: info.tcpi_last_data_recv,
            })
        }

        fn linux_ca_state(state: u8) -> CaState {
            // Linux CA states from include/net/tcp.h
            match state {
                0 => CaState::Open,
                1 => CaState::Disorder,
                2 => CaState::Cwr,
                3 => CaState::Recovery,
                4 => CaState::Loss,
                v => CaState::Unknown(v),
            }
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

        /// Subset of macOS's `TCP_CONNECTION_INFO`.
        #[derive(Debug)]
        pub struct TcpInfoSnapshot {
            /// TCP connection state.
            pub state: TcpState,
            /// Smoothed round-trip time in microseconds.
            pub rtt_us: u32,
            /// Current (instantaneous) round-trip time in microseconds.
            pub rtt_cur_us: u32,
            /// Current retransmit timeout in microseconds.
            pub rto_us: u32,
            /// Bytes sent.
            pub bytes_sent: u64,
            /// Bytes received.
            pub bytes_received: u64,
            /// Total retransmitted bytes.
            pub retransmitted_bytes: u32,
            /// Congestion window size in bytes.
            pub cwnd_bytes: u32,
            /// Slow-start threshold in bytes.
            pub ssthresh: u32,
            /// Bytes currently in the send buffer.
            pub snd_buf_bytes: u32,
            /// RTT variance in microseconds.
            pub rtt_var_us: u32,
        }

        pub fn query(socket: &socket2::Socket) -> Option<TcpInfoSnapshot> {
            // The last field we read is `tcpi_rttvar` (u32). Only warn if the
            // kernel didn't return enough bytes to fill it.
            let used_bytes = std::mem::offset_of!(libc::tcp_connection_info, tcpi_rttvar)
                + std::mem::size_of::<u32>();
            let info = getsockopt_tcp_struct::<libc::tcp_connection_info>(
                socket,
                libc::IPPROTO_TCP,
                libc::TCP_CONNECTION_INFO,
                used_bytes,
            )?;

            Some(TcpInfoSnapshot {
                state: macos_tcp_state(info.tcpi_state),
                rtt_us: info.tcpi_srtt,
                rtt_cur_us: info.tcpi_rttcur,
                rto_us: info.tcpi_rto,
                bytes_sent: info.tcpi_txbytes,
                bytes_received: info.tcpi_rxbytes,
                retransmitted_bytes: u32::try_from(info.tcpi_txretransmitbytes).unwrap_or(u32::MAX),
                cwnd_bytes: info.tcpi_snd_cwnd,
                ssthresh: info.tcpi_snd_ssthresh,
                snd_buf_bytes: info.tcpi_snd_sbbytes,
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

        /// Subset of Windows's `SIO_TCP_INFO`.
        #[derive(Debug)]
        pub struct TcpInfoSnapshot {
            /// TCP connection state.
            pub state: TcpState,
            /// Smoothed round-trip time in microseconds.
            pub rtt_us: u32,
            /// Minimum round-trip time observed in microseconds.
            pub min_rtt_us: u32,
            /// Bytes sent.
            pub bytes_sent: u64,
            /// Bytes received.
            pub bytes_received: u64,
            /// Total retransmitted bytes.
            pub retransmitted_bytes: u32,
            /// Congestion window size in bytes.
            pub cwnd_bytes: u32,
            /// SYN retransmissions (handshake phase).
            pub syn_retransmits: u32,
            /// RTO timeout episodes.
            pub timeout_episodes: u32,
            /// Bytes currently in flight (sent but not yet ACKed).
            pub bytes_in_flight: u32,
            /// Fast retransmit count.
            pub fast_retrans: u32,
            /// Duplicate ACKs received.
            pub dup_acks_in: u32,
        }

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
                min_rtt_us: info.MinRttUs,
                bytes_sent: info.BytesOut,
                bytes_received: info.BytesIn,
                retransmitted_bytes: info.BytesRetrans,
                cwnd_bytes: info.Cwnd,
                syn_retransmits: u32::from(info.SynRetrans),
                timeout_episodes: info.TimeoutEpisodes,
                bytes_in_flight: info.BytesInFlight,
                fast_retrans: info.FastRetrans,
                dup_acks_in: info.DupAcksIn,
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
        /// TCP diagnostics are not available on this platform.
        #[derive(Debug)]
        pub struct TcpInfoSnapshot {}

        pub fn query(_socket: &socket2::Socket) -> Option<TcpInfoSnapshot> {
            log::debug!("TCP info querying not supported on this platform");
            None
        }
    }
}
