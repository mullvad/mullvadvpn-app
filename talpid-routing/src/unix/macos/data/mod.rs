mod address_flag;
mod address_message;
mod destination;
mod interface;
mod message_type;
mod route_destination;
mod route_flag;
mod route_message;
mod route_sockaddr_iterator;
mod route_socket_address;
mod route_socket_message;

pub use address_flag::AddressFlag;
pub use address_message::AddressMessage;
pub use destination::Destination;
pub use interface::Interface;
pub use message_type::MessageType;
pub use route_destination::RouteDestination;
pub use route_flag::RouteFlag;
pub use route_message::RouteMessage;
pub use route_sockaddr_iterator::RouteSockAddrIterator;
pub use route_socket_address::RouteSocketAddress;
pub use route_socket_message::RouteSocketMessage;

use nix::sys::socket::SockaddrStorage;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    os::raw::{c_int, c_uchar, c_ushort},
};

/// Shorter rt_msghdr version that matches all routing messages
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct rt_msghdr_short {
    pub rtm_msglen: c_ushort,
    pub rtm_version: c_uchar,
    pub rtm_type: c_uchar,
    pub rtm_index: c_ushort,
    pub rtm_flags: c_int,
    pub rtm_addrs: c_int,
    pub rtm_pid: libc::pid_t,
    pub rtm_seq: c_int,
    pub rtm_errno: c_int,
}
const ROUTE_MESSAGE_HEADER_SHORT_SIZE: usize = std::mem::size_of::<rt_msghdr_short>();

impl rt_msghdr_short {
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() >= ROUTE_MESSAGE_HEADER_SHORT_SIZE {
            let ptr = buf.as_ptr();
            // SAFETY: `ptr` is backed by enough valid bytes to contain a rt_msghdr_short value and
            // is readable. `rt_msghdr_short` doesn't contain any pointers so any values are valid.
            Some(unsafe { std::ptr::read(ptr as *const rt_msghdr_short) })
        } else {
            None
        }
    }

    pub fn is_type(&self, expected_type: i32) -> bool {
        u8::try_from(expected_type)
            .map(|expected| self.rtm_type == expected)
            .unwrap_or(false)
    }
}

// struct rt_msghdr {
// 	u_short rtm_msglen;     /* to skip over non-understood messages */
// 	u_char  rtm_version;    /* future binary compatibility */
// 	u_char  rtm_type;       /* message type */
// 	u_short rtm_index;      /* index for associated ifp */
// 	int     rtm_flags;      /* flags, incl. kern & message, e.g. DONE */
// 	int     rtm_addrs;      /* bitmask identifying sockaddrs in msg */
// 	pid_t   rtm_pid;        /* identify sender */
// 	int     rtm_seq;        /* for sender to identify action */
// 	int     rtm_errno;      /* why failed */
// 	int     rtm_use;        /* from rtentry */
// 	u_int32_t rtm_inits;    /* which metrics we are initializing */
// 	struct rt_metrics rtm_rmx; /* metrics themselves */
// };
#[derive(Debug, Clone)]
#[repr(C)]
pub struct rt_msghdr {
    pub rtm_msglen: c_ushort,
    pub rtm_version: c_uchar,
    pub rtm_type: c_uchar,
    pub rtm_index: c_ushort,
    pub rtm_flags: c_int,
    pub rtm_addrs: c_int,
    pub rtm_pid: libc::pid_t,
    pub rtm_seq: c_int,
    pub rtm_errno: c_int,
    pub rtm_use: c_int,
    pub rtm_inits: u32,
    pub rtm_rmx: rt_metrics,
}
pub(crate) const ROUTE_MESSAGE_HEADER_SIZE: usize = std::mem::size_of::<rt_msghdr>();

fn saddr_to_ipv4(saddr: &SockaddrStorage) -> Option<Ipv4Addr> {
    saddr.as_sockaddr_in().map(|sin| sin.ip())
}

fn saddr_to_ipv6(saddr: &SockaddrStorage) -> Option<Ipv6Addr> {
    saddr.as_sockaddr_in6().map(|sin6| sin6.ip())
}

impl rt_msghdr {
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() >= ROUTE_MESSAGE_HEADER_SIZE {
            let ptr = buf.as_ptr();
            // SAFETY: `ptr` is backed by enough valid bytes to contain a rt_msghdr value and it's
            // readable. rt_msghdr doesn't contain any pointers so any values are valid.
            Ok(unsafe { std::ptr::read(ptr as *const _) })
        } else {
            Err(Error::BufferTooSmall(
                "if_msghdr",
                buf.len(),
                ROUTE_MESSAGE_HEADER_SIZE,
            ))
        }
    }
}

// Struct containing metrics of various metrics for a specific route
// struct rt_metrics {
// 	u_int32_t       rmx_locks;      /* Kernel leaves these values alone */
// 	u_int32_t       rmx_mtu;        /* MTU for this path */
// 	u_int32_t       rmx_hopcount;   /* max hops expected */
// 	int32_t         rmx_expire;     /* lifetime for route, e.g. redirect */
// 	u_int32_t       rmx_recvpipe;   /* inbound delay-bandwidth product */
// 	u_int32_t       rmx_sendpipe;   /* outbound delay-bandwidth product */
// 	u_int32_t       rmx_ssthresh;   /* outbound gateway buffer limit */
// 	u_int32_t       rmx_rtt;        /* estimated round trip time */
// 	u_int32_t       rmx_rttvar;     /* estimated rtt variance */
// 	u_int32_t       rmx_pksent;     /* packets sent using this route */
// 	u_int32_t       rmx_state;      /* route state */
// 	u_int32_t       rmx_filler[3];  /* will be used for TCP's peer-MSS cache */
// };
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct rt_metrics {
    pub rmx_locks: u32,
    pub rmx_mtu: u32,
    pub rmx_hopcount: u32,
    pub rmx_expire: i32,
    pub rmx_recvpipe: u32,
    pub rmx_sendpipe: u32,
    pub rmx_ssthresh: u32,
    pub rmx_rtt: u32,
    pub rmx_rttvar: u32,
    pub rmx_pksent: u32,
    pub rmx_state: u32,
    pub rmx_filler: [u32; 3],
}

#[derive(Debug)]
pub enum Error {
    /// Payload buffer didn't match the reported message size in header
    InvalidBuffer(Vec<u8>, AddressFlag),
    /// Buffer too small for specific message type
    BufferTooSmall(&'static str, usize, usize),
    /// Unknown route flag
    UnknownRouteFlag(i32),
    /// Socket address is empty for the given address flag
    EmptySockaddr(AddressFlag),
    /// Unrecognized message
    UnknownMessageType(u8),
    /// Unrecognized address flag
    UnknownAddressFlag(c_int),
    /// Mismatched socket address type
    MismatchedSocketAddress(AddressFlag, Box<SockaddrStorage>),
    /// Link socket address contains no identifier
    NoLinkIdentifier(nix::libc::sockaddr_dl),
    /// Failed to resolve an interface name to an index
    InterfaceIndex(nix::Error),
    /// Invalid netmask
    InvalidNetmask(ipnetwork::IpNetworkError),
    /// Route contains no netmask socket address
    NoDestination,
    /// Address message does not contain an interface address
    NoInterfaceAddress,
    /// Address message does not contain an interface address
    NoNetmaskAddress,
}

pub type Result<T> = std::result::Result<T, Error>;
