use ipnetwork::IpNetwork;
use nix::{
    ifaddrs::InterfaceAddress,
    sys::socket::{SockaddrLike, SockaddrStorage},
};
use std::{
    collections::BTreeMap,
    ffi::c_int,
    fmt::{self, Debug},
    mem,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

/// Errors associated with route socket and route messages
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Payload buffer didn't match the reported message size in header
    #[error("Buffer didn't match reported message size")]
    InvalidBuffer(Vec<u8>, AddressFlag),
    /// Buffer too small for specific message type
    #[error(
        "The buffer is too small for msg \"{message_type}\": expected size >= {expect_min_size}, actual {actual_size}"
    )]
    BufferTooSmall {
        /// Type of message
        message_type: &'static str,
        /// Expected minimum size of the buffer
        expect_min_size: usize,
        /// Actual size of the buffer
        actual_size: usize,
    },
    /// Unknown route flag
    #[error("Unknown route flag: {0}")]
    UnknownRouteFlag(c_int),
    /// Unrecognized address flag
    #[error("Unrecognized address flag: {0}")]
    UnknownAddressFlag(c_int),
    /// Mismatched socket address type
    #[error("Unrecognized socket address: expected IPv4 or IPv6")]
    MismatchedSocketAddress(AddressFlag, Box<SockaddrStorage>),
    /// Invalid netmask
    #[error("Invalid netmask")]
    InvalidNetmask,
    /// Route contains no netmask socket address
    #[error("Found no route destination")]
    NoDestination,
    /// Found no netmask
    #[error("Found no netmask")]
    NoNetmask,
    /// Address message does not contain an interface address
    #[error("Found no interface address")]
    NoInterfaceAddress,
}

type Result<T> = std::result::Result<T, Error>;

/// Message that describes a route - either an added, removed, changed or plainly retrieved route.
///
/// This corresponds to RTM_ADD, RTM_DELETE, RTM_CHANGE, or RTM_GET.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteMessage {
    // INVARIANT: The `AddressFlag` must match the variant of `RouteSocketAddress`.
    sockaddrs: BTreeMap<AddressFlag, RouteSocketAddress>,
    mtu: u32,
    route_flags: RouteFlag,
    interface_index: u16,
    errno: i32,
}

impl RouteMessage {
    /// Route message for `destination`.
    pub fn new_route(destination: Destination) -> Self {
        let mut route_flags = RouteFlag::RTF_STATIC | RouteFlag::RTF_DONE | RouteFlag::RTF_UP;
        let mut sockaddrs = BTreeMap::new();
        match destination {
            Destination::Network(net) => {
                let dest_addr = SockaddrStorage::from(SocketAddr::from((net.ip(), 0)));
                let destination = RouteSocketAddress::Destination(Some(dest_addr));
                let netmask =
                    RouteSocketAddress::Netmask(Some(SocketAddr::from((net.mask(), 0)).into()));
                sockaddrs.insert(destination.address_flag(), destination);
                sockaddrs.insert(netmask.address_flag(), netmask);
            }
            Destination::Host(addr) => {
                let destination =
                    RouteSocketAddress::Destination(Some(SocketAddr::from((addr, 0)).into()));
                route_flags |= RouteFlag::RTF_HOST;
                sockaddrs.insert(destination.address_flag(), destination);
            }
        };

        Self {
            sockaddrs,
            mtu: 0,
            route_flags,
            interface_index: 0,
            errno: 0,
        }
    }

    /// Return all addresses in the route message
    pub fn route_addrs(&self) -> impl Iterator<Item = &RouteSocketAddress> {
        self.sockaddrs.values()
    }

    fn socketaddress_to_ip(sockaddr: &SockaddrStorage) -> Option<IpAddr> {
        saddr_to_ipv4(sockaddr)
            .map(Into::into)
            .or_else(|| saddr_to_ipv6(sockaddr).map(Into::into))
    }

    /// Find netmask in the route message
    pub fn netmask(&self) -> Option<IpAddr> {
        self.route_addrs()
            .find_map(|saddr| match saddr {
                RouteSocketAddress::Netmask(netmask) => Some(netmask),
                _ => None,
            })?
            .as_ref()
            .and_then(Self::socketaddress_to_ip)
    }

    /// Return whether there is any default route in this message
    pub fn is_default(&self) -> Result<bool> {
        Ok(self.is_default_v4()? || self.is_default_v6()?)
    }

    /// Return whether there is a default IPv4 route in this message
    pub fn is_default_v4(&self) -> Result<bool> {
        let Some(v4_default) = self.destination_ip()? else {
            return Ok(false);
        };
        // TODO: Checking mask might be superfluous
        Ok(v4_default.mask().is_unspecified())
    }

    /// Return whether there is a default IPv6 route in this message
    pub fn is_default_v6(&self) -> Result<bool> {
        self.destination_ip()?
            .map(|addr| Ok(addr.ip() == Ipv6Addr::UNSPECIFIED))
            .unwrap_or(Ok(false))
    }

    fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
        let (header, payload) = split_rtmsg_hdr(buffer)?;

        let route_flags = RouteFlag::from_bits(header.rtm_flags)
            .ok_or(Error::UnknownRouteFlag(header.rtm_flags))?;
        let address_flags = AddressFlag::from_bits(header.rtm_addrs)
            .ok_or(Error::UnknownAddressFlag(header.rtm_addrs))?;
        if !address_flags.contains(AddressFlag::RTA_DST) {
            return Err(Error::NoDestination);
        }
        let sockaddrs = RouteSockAddrIterator::new(payload, address_flags)
            .map(|addr| addr.map(|a| (a.address_flag(), a)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let interface_index = header.rtm_index;

        let mtu = if header.rtm_inits & RTV_MTU != 0 {
            header.rtm_rmx.rmx_mtu
        } else {
            0
        };

        Ok(Self {
            route_flags,
            mtu,
            sockaddrs,
            interface_index,
            errno: header.rtm_errno,
        })
    }

    fn insert_sockaddr(&mut self, saddr: RouteSocketAddress) {
        self.sockaddrs.insert(saddr.address_flag(), saddr);
    }

    /// Set the destination address of the route message
    pub fn set_destination(mut self, destination: Destination) -> Self {
        match destination {
            Destination::Network(net) => {
                let sockaddr: SocketAddr = (net.ip(), 0).into();
                let netmask: SocketAddr = (net.mask(), 0).into();
                self.insert_sockaddr(RouteSocketAddress::Destination(Some(sockaddr.into())));
                self.insert_sockaddr(RouteSocketAddress::Netmask(Some(netmask.into())));
                self.route_flags.remove(RouteFlag::RTF_HOST);
            }
            Destination::Host(addr) => {
                self.route_flags.insert(RouteFlag::RTF_HOST);
                let sockaddr: SocketAddr = (addr, 0).into();
                self.insert_sockaddr(RouteSocketAddress::Destination(Some(sockaddr.into())));
            }
        };

        self
    }

    /// Set the MTU of the route message
    pub fn set_mtu(mut self, mtu: u32) -> Self {
        self.mtu = mtu;
        self
    }

    /// Append route flags to the route message
    pub fn append_route_flag(mut self, route_flag: RouteFlag) -> Self {
        self.route_flags |= route_flag;
        self
    }

    /// Set the interface address of the route message
    pub fn set_interface_addr(mut self, link: &InterfaceAddress) -> Self {
        self.insert_sockaddr(RouteSocketAddress::Gateway(link.address));
        self.route_flags |= RouteFlag::RTF_GATEWAY;
        self
    }

    /// Set the gateway address of the route message
    pub fn set_gateway_addr(mut self, gateway: impl Into<SockaddrStorage>) -> Self {
        self.insert_sockaddr(RouteSocketAddress::Gateway(Some(gateway.into())));
        self.route_flags |= RouteFlag::RTF_GATEWAY;

        self
    }

    /// Find gateway address of the route message
    pub fn gateway(&self) -> Option<&SockaddrStorage> {
        self.route_addrs()
            .find_map(|saddr| match saddr {
                RouteSocketAddress::Gateway(gateway) => Some(gateway),
                _ => None,
            })?
            .as_ref()
    }

    /// Gateway address of the route message, iff it is an IP address
    /// (rather than, for example, a link-layer address).
    pub fn gateway_ip(&self) -> Option<IpAddr> {
        self.gateway_v4()
            .map(IpAddr::V4)
            .or(self.gateway_v6().map(IpAddr::V6))
    }

    fn gateway_v4(&self) -> Option<Ipv4Addr> {
        saddr_to_ipv4(self.gateway()?)
    }

    fn gateway_v6(&self) -> Option<Ipv6Addr> {
        saddr_to_ipv6(self.gateway()?)
    }

    /// Destination address of the route message
    pub fn destination_ip(&self) -> Result<Option<IpNetwork>> {
        let Some(saddr) = self.destination()? else {
            return Ok(None);
        };

        if let Some(ip_addr) = saddr_to_ipv4(saddr) {
            let netmask = self.netmask().unwrap_or(Ipv4Addr::UNSPECIFIED.into());
            let destination = IpNetwork::with_netmask(ip_addr.into(), netmask)
                .map_err(|_| Error::InvalidNetmask)?;
            return Ok(Some(destination));
        }

        if let Some(ip_addr) = saddr_to_ipv6(saddr) {
            let netmask = self.netmask().unwrap_or(Ipv6Addr::UNSPECIFIED.into());
            let destination = IpNetwork::with_netmask(ip_addr.into(), netmask)
                .map_err(|_| Error::InvalidNetmask)?;
            return Ok(Some(destination));
        }

        Err(Error::MismatchedSocketAddress(
            AddressFlag::RTA_DST,
            Box::new(*saddr),
        ))
    }

    fn destination(&self) -> Result<Option<&SockaddrStorage>> {
        Ok(self
            .route_addrs()
            .find_map(|saddr| match saddr {
                RouteSocketAddress::Destination(destination) => Some(destination),
                _ => None,
            })
            .ok_or(Error::NoDestination)?
            .as_ref())
    }

    /// Serialize into structs/buffers compatible with `PF_ROUTE` sockets
    pub fn payload(
        &self,
        message_type: MessageType,
        sequence: i32,
        pid: i32,
    ) -> (libc::rt_msghdr, Vec<Vec<u8>>) {
        let address_flags = self.route_addrs().fold(AddressFlag::empty(), |flag, addr| {
            flag | addr.address_flag()
        });

        // The sockaddrs should be ordered by their address flag in the payload,
        // because the payload does not contain their flags. Flags are only specified
        // in the header.
        let mut sockaddrs = self.route_addrs().collect::<Vec<_>>();
        sockaddrs.sort_by_key(|saddr| saddr.address_flag());
        let payload_bytes = sockaddrs
            .into_iter()
            .map(RouteSocketAddress::to_bytes)
            .collect::<Vec<_>>();

        let payload_len: usize = payload_bytes.iter().map(Vec::len).sum();

        let rtm_msglen = (payload_len + mem::size_of::<libc::rt_msghdr>())
            .try_into()
            .expect("route message buffer size cannot fit in 32 bits");

        let mut header = libc::rt_msghdr {
            rtm_msglen,
            rtm_version: libc::RTM_VERSION.try_into().unwrap(),
            rtm_type: message_type.bits().try_into().unwrap(),
            rtm_index: self.interface_index,
            rtm_flags: self.route_flags.bits(),
            rtm_addrs: address_flags.bits(),
            rtm_pid: pid,
            rtm_seq: sequence,
            rtm_errno: 0,
            rtm_use: 0,
            rtm_inits: 0,
            rtm_rmx: libc::rt_metrics {
                rmx_locks: 0,
                rmx_mtu: 0,
                rmx_hopcount: 0,
                rmx_expire: 0,
                rmx_recvpipe: 0,
                rmx_sendpipe: 0,
                rmx_ssthresh: 0,
                rmx_rtt: 0,
                rmx_rttvar: 0,
                rmx_pksent: 0,
                rmx_state: 0,
                rmx_filler: [0; 3],
            },
        };

        if self.mtu != 0 {
            header.rtm_inits |= RTV_MTU;
            header.rtm_rmx.rmx_mtu = self.mtu;
        }

        (header, payload_bytes)
    }

    /// Interface index for the route
    pub fn interface_index(&self) -> u16 {
        self.interface_index
    }

    /// Set route interface index
    pub fn set_interface_index(mut self, index: u16) -> Self {
        self.interface_index = index;
        self
    }

    /// Error associated with this route message
    pub fn errno(&self) -> i32 {
        self.errno
    }

    /// Whether this route is an ifscope route.
    /// If set, the route is bound to `interface_index`.
    pub fn ifscope(&self) -> bool {
        self.route_flags.contains(RouteFlag::RTF_IFSCOPE)
    }

    /// Turn this route into a scoped (ifscope) route for the interface index.
    pub fn set_ifscope(mut self) -> Self {
        self.route_flags |= RouteFlag::RTF_IFSCOPE;
        self
    }
}

/// Address message - used for adding or removing interface addresses.
///
/// This corresponds to RTM_NEWADDR or RTM_DELADDR.
#[derive(Debug)]
pub struct AddressMessage {
    sockaddrs: BTreeMap<AddressFlag, RouteSocketAddress>,
    interface_index: u16,
}

impl AddressMessage {
    /// Interface index for the address message
    pub fn interface_index(&self) -> u16 {
        self.interface_index
    }

    /// IP address of the interface
    pub fn address(&self) -> Result<IpAddr> {
        self.get_address(&AddressFlag::RTA_IFP)
            .or_else(|| self.get_address(&AddressFlag::RTA_IFA))
            .ok_or(Error::NoInterfaceAddress)
    }

    fn get_address(&self, address_flag: &AddressFlag) -> Option<IpAddr> {
        let addr = self.sockaddrs.get(address_flag)?;
        saddr_to_ipv4(addr.address()?)
            .map(IpAddr::from)
            .or_else(|| saddr_to_ipv6(addr.address()?).map(IpAddr::from))
    }

    /// Netmask of the interface
    pub fn netmask(&self) -> Result<IpAddr> {
        self.get_address(&AddressFlag::RTA_NETMASK)
            .ok_or(Error::NoNetmask)
    }

    fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
        const HEADER_SIZE: usize = std::mem::size_of::<libc::ifa_msghdr>();
        if HEADER_SIZE > buffer.len() {
            return Err(Error::BufferTooSmall {
                message_type: "ifa_msghdr",
                expect_min_size: HEADER_SIZE,
                actual_size: buffer.len(),
            });
        }

        // SAFETY: buffer is pointing to enough memory to contain a valid value for ifa_msghdr
        let header: libc::ifa_msghdr =
            unsafe { std::ptr::read_unaligned(buffer.as_ptr() as *const _) };

        let msg_len = usize::from(header.ifam_msglen);
        if msg_len > buffer.len() {
            return Err(Error::BufferTooSmall {
                message_type: "address message (ifa_msghdr.msg_len)",
                expect_min_size: msg_len,
                actual_size: buffer.len(),
            });
        }

        let payload = &buffer[HEADER_SIZE..std::cmp::min(msg_len, buffer.len())];

        let address_flags = AddressFlag::from_bits(header.ifam_addrs)
            .ok_or(Error::UnknownAddressFlag(header.ifam_addrs))?;

        let sockaddrs = RouteSockAddrIterator::new(payload, address_flags)
            .map(|addr| addr.map(|addr| (addr.address_flag(), addr)))
            .collect::<Result<BTreeMap<_, _>>>()?;

        Ok(Self {
            sockaddrs,
            interface_index: header.ifam_index,
        })
    }
}

/// Message types for route socket messages (associated with a PF_ROUTE socket).
#[derive(Debug)]
pub enum RouteSocketMessage {
    /// A route add message.
    ///
    /// This corresponds to RTM_ADD.
    AddRoute(RouteMessage),
    /// A route delete message.
    ///
    /// This corresponds to RTM_DELETE.
    DeleteRoute(RouteMessage),
    /// A route change message.
    ///
    /// This corresponds to RTM_CHANGE.
    ChangeRoute(RouteMessage),
    /// A route get message.
    ///
    /// This corresponds to RTM_GET.
    GetRoute(RouteMessage),
    /// An interface message.
    ///
    /// This corresponds to RTM_IFINFO.
    Interface(Interface),
    /// An address message.
    ///
    /// This corresponds to RTM_NEWADDR.
    AddAddress(AddressMessage),
    /// An address message.
    ///
    /// This corresponds to RTM_DELADDR.
    DeleteAddress(AddressMessage),
    /// Unhandled message type.
    Other {
        /// Message header
        header: ffi::rt_msghdr_short,
        /// Raw payload of the message
        payload: Vec<u8>,
    },
}

/// Destination of a route
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Single host (i.e., netmask of 255.255.255.255)
    Host(IpAddr),
    /// Network
    Network(IpNetwork),
}

impl From<IpAddr> for Destination {
    fn from(addr: IpAddr) -> Self {
        Self::Host(addr)
    }
}

impl From<IpNetwork> for Destination {
    fn from(net: IpNetwork) -> Self {
        if net.prefix() == 32 && net.is_ipv4() {
            return Self::Host(net.ip());
        }

        Self::Network(net)
    }
}

impl RouteSocketMessage {
    /// Parse a raw buffer from a PF_ROUTE socket into a `RouteSocketMessage`
    pub fn parse_message(buffer: &[u8]) -> Result<Self> {
        let route_message = |route_constructor: fn(RouteMessage) -> RouteSocketMessage, buffer| {
            let route = RouteMessage::from_byte_buffer(buffer)?;
            Ok(route_constructor(route))
        };

        match ffi::rt_msghdr_short::from_bytes(buffer) {
            Some(header) if header.is_type(libc::RTM_ADD) => route_message(Self::AddRoute, buffer),

            Some(header) if header.is_type(libc::RTM_CHANGE) => {
                route_message(Self::ChangeRoute, buffer)
            }

            Some(header) if header.is_type(libc::RTM_DELETE) => {
                route_message(Self::DeleteRoute, buffer)
            }

            Some(header) if header.is_type(libc::RTM_GET) => route_message(Self::GetRoute, buffer),

            Some(header) if header.is_type(libc::RTM_IFINFO) => Ok(RouteSocketMessage::Interface(
                Interface::from_byte_buffer(buffer)?,
            )),

            Some(header) if header.is_type(libc::RTM_NEWADDR) => Ok(
                RouteSocketMessage::AddAddress(AddressMessage::from_byte_buffer(buffer)?),
            ),
            Some(header) if header.is_type(libc::RTM_DELADDR) => Ok(
                RouteSocketMessage::DeleteAddress(AddressMessage::from_byte_buffer(buffer)?),
            ),
            Some(header) => Ok(Self::Other {
                header,
                payload: buffer.to_vec(),
            }),
            None => Err(Error::BufferTooSmall {
                message_type: "rt_msghdr_short",
                expect_min_size: ffi::rt_msghdr_short::SIZE,
                actual_size: buffer.len(),
            }),
        }
    }
}

/// An interface message.
///
/// This corresponds to RTM_IFINFO.
#[derive(Debug)]
pub struct Interface {
    header: libc::if_msghdr,
}

impl Interface {
    /// Whether the interface is up
    ///
    /// Corresponds to the IFF_UP flag.
    pub fn is_up(&self) -> bool {
        self.header.ifm_flags & nix::libc::IFF_UP != 0
    }

    /// Interface index
    ///
    /// Corresponds to ifm_index.
    pub fn index(&self) -> u16 {
        self.header.ifm_index
    }

    /// Interface MTU
    ///
    /// Corresponds to ifi_mtu.
    pub fn mtu(&self) -> u32 {
        self.header.ifm_data.ifi_mtu
    }

    fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
        const INTERFACE_MESSAGE_HEADER_SIZE: usize = std::mem::size_of::<libc::if_msghdr>();
        if INTERFACE_MESSAGE_HEADER_SIZE > buffer.len() {
            return Err(Error::BufferTooSmall {
                message_type: "if_msghdr",
                expect_min_size: INTERFACE_MESSAGE_HEADER_SIZE,
                actual_size: buffer.len(),
            });
        }
        let header = buffer.as_ptr().cast::<libc::if_msghdr>();

        // SAFETY:
        // - `buffer` points to initialized memory of the correct size.
        // - if_msghdr is a C struct, and valid for any bit pattern
        let header: libc::if_msghdr = unsafe { header.read_unaligned() };
        // let payload = buffer[INTERFACE_MESSAGE_HEADER_SIZE..header.ifm_msglen.into()].to_vec();
        Ok(Self { header })
    }
}

bitflags::bitflags! {
    /// All enum values of address flags can be iterated via `flag <<= 1`, starting from 1.
    /// See https://www.manpagez.com/man/4/route/.
    #[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
    pub struct AddressFlag: i32 {
        /// Destination socket address
        const RTA_DST       = libc::RTA_DST;
        /// Gateway socket address
        const RTA_GATEWAY   = libc::RTA_GATEWAY;
        /// Netmask socket address
        const RTA_NETMASK   = libc::RTA_NETMASK;
        /// Cloning mask socket address
        const RTA_GENMASK   = libc::RTA_GENMASK;
        /// Interface name socket address
        const RTA_IFP       = libc::RTA_IFP;
        /// Interface address socket address
        const RTA_IFA       = libc::RTA_IFA;
        /// Socket address for author of redirect
        const RTA_AUTHOR    = libc::RTA_AUTHOR;
        /// Socket address for `NEWADDR`, broadcast or point-to-point destination address
        const RTA_BRD       = libc::RTA_BRD;
    }
}

bitflags::bitflags! {
    /// Types of routing messages
    /// See https://www.manpagez.com/man/4/route/.
    #[derive(Debug)]
    pub struct MessageType: i32 {
        /// Add Route
        const RTM_ADD         = libc::RTM_ADD;
        /// Delete Route
        const RTM_DELETE      = libc::RTM_DELETE;
        /// Change Metrics or flags
        const RTM_CHANGE      = libc::RTM_CHANGE;
        /// Report Metrics
        const RTM_GET         = libc::RTM_GET;
        /// RTM_LOSING is no longer generated by and is deprecated
        const RTM_LOSING      = libc::RTM_LOSING;
        /// Told to use different route
        const RTM_REDIRECT    = libc::RTM_REDIRECT;
        /// Lookup failed on this address
        const RTM_MISS        = libc::RTM_MISS;
        /// fix specified metrics
        const RTM_LOCK        = libc::RTM_LOCK;
        /// caused by SIOCADDRT
        const RTM_OLDADD      = libc::RTM_OLDADD;
        /// caused by SIOCDELRT
        const RTM_OLDDEL      = libc::RTM_OLDDEL;
        /// req to resolve dst to LL addr
        const RTM_RESOLVE     = libc::RTM_RESOLVE;
        /// address being added to iface
        const RTM_NEWADDR     = libc::RTM_NEWADDR;
        /// address being removed from iface
        const RTM_DELADDR     = libc::RTM_DELADDR;
        /// iface going up/down etc.
        const RTM_IFINFO      = libc::RTM_IFINFO;
        /// mcast group membership being added to if
        const RTM_NEWMADDR    = libc::RTM_NEWMADDR;
        /// mcast group membership being deleted
        const RTM_DELMADDR    = libc::RTM_DELMADDR;
    }

    /// Routing message flags
    /// See https://www.manpagez.com/man/4/route/.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct RouteFlag: i32 {
        /// route usable
        const RTF_UP = libc::RTF_UP;
        /// destination is a gateway
        const RTF_GATEWAY = libc::RTF_GATEWAY;
        /// host entry (net otherwise)
        const RTF_HOST = libc::RTF_HOST;
        /// host or net unreachable
        const RTF_REJECT = libc::RTF_REJECT;
        /// created dynamically (by redirect)
        const RTF_DYNAMIC = libc::RTF_DYNAMIC;
        /// modified dynamically (by redirect)
        const RTF_MODIFIED = libc::RTF_MODIFIED;
        /// message confirmed
        const RTF_DONE = libc::RTF_DONE;
        /// delete cloned route
        const RTF_DELCLONE = libc::RTF_DELCLONE;
        /// generate new routes on use
        const RTF_CLONING = libc::RTF_CLONING;
        /// external daemon resolves name
        const RTF_XRESOLVE = libc::RTF_XRESOLVE;
        /// used by apps to add/del L2 entries
        /// the newer constant is called RTF_LLDATA but absent in libc, has the same value.
        const RTF_LLINFO = libc::RTF_LLINFO;
        /// manually added
        const RTF_STATIC = libc::RTF_STATIC;
        /// just discard pkts (during updates)
        const RTF_BLACKHOLE = libc::RTF_BLACKHOLE;
        /// not eligible for RTF_IFREF
        const RTF_NOIFREF = libc::RTF_NOIFREF;
        /// protocol specific routing flag
        const RTF_PROTO2 = libc::RTF_PROTO2;
        /// protocol specific routing flag
        const RTF_PROTO1 = libc::RTF_PROTO1;
        /// protocol requires cloning
        const RTF_PRCLONING = libc::RTF_PRCLONING;
        /// route generated through cloning
        const RTF_WASCLONED = libc::RTF_WASCLONED;
        /// protocol specific routing flag
        const RTF_PROTO3 = libc::RTF_PROTO3;
        /// future use
        const RTF_PINNED = libc::RTF_PINNED;
        /// route represents a local address
        const RTF_LOCAL = libc::RTF_LOCAL;
        /// route represents a bcast address
        const RTF_BROADCAST = libc::RTF_BROADCAST;
        /// route represents a mcast address
        const RTF_MULTICAST = libc::RTF_MULTICAST;
        /// has valid interface scope
        const RTF_IFSCOPE = libc::RTF_IFSCOPE;
        /// defunct; no longer modifiable
        const RTF_CONDEMNED = libc::RTF_CONDEMNED;
        /// route holds a ref to interface
        const RTF_IFREF = libc::RTF_IFREF;
        /// proxying, no interface scope
        const RTF_PROXY = libc::RTF_PROXY;
        /// host is a router
        const RTF_ROUTER = libc::RTF_ROUTER;
        /// Route entry is being freed
        const RTF_DEAD = libc::RTF_DEAD;
        /// route to destination of the global internet
        const RTF_GLOBAL = libc::RTF_GLOBAL;
    }
}

/// A route socket address
#[derive(Clone, PartialEq)]
pub enum RouteSocketAddress {
    /// Destination address.
    ///
    /// Corresponds to RTA_DST
    Destination(Option<SockaddrStorage>),
    /// Gateway address.
    ///
    /// Corresponds to RTA_GATEWAY
    Gateway(Option<SockaddrStorage>),
    /// A netmask.
    ///
    /// Corresponds to RTA_NETMASK
    Netmask(Option<SockaddrStorage>),
    /// Corresponds to RTA_GENMASK
    CloningMask(Option<SockaddrStorage>),
    /// Interface name address.
    ///
    /// Corresponds to RTA_IFP
    IfName(Option<SockaddrStorage>),
    /// Interface address.
    ///
    /// Corresponds to RTA_IFA
    IfSockaddr(Option<SockaddrStorage>),
    /// Author of redirect.
    ///
    /// Corresponds to RTA_AUTHOR
    RedirectAuthor(Option<SockaddrStorage>),
    /// Broadcast address.
    ///
    /// Corresponds to RTA_BRD
    Broadcast(Option<SockaddrStorage>),
}

/// Custom Debug-impl that uses the Display-impl of [SockaddrStorage] since its Debug-impl is
/// basically unreadable.
impl Debug for RouteSocketAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (variant, sockaddr) = match self {
            Self::Destination(sockaddr) => ("Destination", sockaddr),
            Self::Gateway(sockaddr) => ("Gateway", sockaddr),
            Self::Netmask(sockaddr) => ("Netmask", sockaddr),
            Self::CloningMask(sockaddr) => ("CloningMask", sockaddr),
            Self::IfName(sockaddr) => ("IfName", sockaddr),
            Self::IfSockaddr(sockaddr) => ("IfSockaddr", sockaddr),
            Self::RedirectAuthor(sockaddr) => ("RedirectAuthor", sockaddr),
            Self::Broadcast(sockaddr) => ("Broadcast", sockaddr),
        };

        if let Some(sockaddr) = sockaddr {
            if let Some(link_addr) = sockaddr.as_link_addr() {
                // The default Display impl for LinkAddrs does not print ifindex
                write!(f, "{variant}(")?;
                f.debug_struct("LinkAddr")
                    .field("addr", &link_addr.addr())
                    .field("iface", &link_addr.ifindex())
                    .finish()?;
                write!(f, ")")
            } else {
                write!(f, "{variant}({sockaddr})")
            }
        } else {
            write!(f, "{variant}(None)")
        }
    }
}

impl RouteSocketAddress {
    /// Return a new route socket address and number of bytes read from the buffer
    pub fn new(flag: AddressFlag, buf: &[u8]) -> Result<(Self, u8)> {
        // If buffer is empty, then the socket address is empty too, the backing buffer shouldn't
        // be advanced.
        if buf.is_empty() {
            return Ok((Self::with_sockaddr(flag, None)?, 0));
        }

        // to get the length and type of
        if buf.len() < std::mem::size_of::<ffi::sockaddr_hdr>() {
            return Err(Error::BufferTooSmall {
                message_type: "sockaddr_hdr",
                expect_min_size: std::mem::size_of::<ffi::sockaddr_hdr>(),
                actual_size: buf.len(),
            });
        }

        let addr_header_ptr = buf.as_ptr() as *const ffi::sockaddr_hdr;
        // SAFETY: Since `buf` is at least as long as a `sockaddr_hdr`, it's perfectly valid to
        // read from.
        let addr_header = unsafe { std::ptr::read(addr_header_ptr) };
        let saddr_len = addr_header.sa_len;
        if saddr_len == 0 {
            return Ok((Self::with_sockaddr(flag, None)?, 4));
        }

        if Into::<usize>::into(saddr_len) > buf.len() {
            return Err(Error::InvalidBuffer(buf.to_vec(), flag));
        }

        // SAFETY: the buffer is big enough for the sockaddr struct inside it, so accessing as a
        // `sockaddr` is valid.
        let saddr = unsafe {
            SockaddrStorage::from_raw(
                addr_header_ptr as *const nix::libc::sockaddr,
                Some(saddr_len.into()),
            )
        };

        Ok((Self::with_sockaddr(flag, saddr)?, saddr_len))
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self.address() {
            None => vec![0u8; 4],
            Some(addr) => {
                let len = usize::try_from(addr.len()).unwrap();
                assert!(len >= 4);

                // The "serialized" socket addresses must be padded to be aligned to 4 bytes, with
                // the smallest size being 4 bytes.
                let buffer_size = len.next_multiple_of(4);
                let mut buffer = vec![0u8; buffer_size];
                // SAFETY: copying contents of addr into buffer is safe, as long as addr.len()
                // returns a correct size for the socket address pointer.
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        addr.as_ptr() as *const _,
                        buffer.as_mut_ptr(),
                        len,
                    )
                };
                buffer
            }
        }
    }

    fn address_flag(&self) -> AddressFlag {
        match &self {
            Self::Destination(_) => AddressFlag::RTA_DST,
            Self::Gateway(_) => AddressFlag::RTA_GATEWAY,
            Self::Netmask(_) => AddressFlag::RTA_NETMASK,
            Self::CloningMask(_) => AddressFlag::RTA_GENMASK,
            Self::IfName(_) => AddressFlag::RTA_IFP,
            Self::IfSockaddr(_) => AddressFlag::RTA_IFA,
            Self::RedirectAuthor(_) => AddressFlag::RTA_AUTHOR,
            Self::Broadcast(_) => AddressFlag::RTA_BRD,
        }
    }

    fn address(&self) -> Option<&SockaddrStorage> {
        match &self {
            Self::Gateway(addr)
            | Self::Destination(addr)
            | Self::Netmask(addr)
            | Self::CloningMask(addr)
            | Self::IfName(addr)
            | Self::IfSockaddr(addr)
            | Self::RedirectAuthor(addr)
            | Self::Broadcast(addr) => addr.as_ref(),
        }
    }

    fn with_sockaddr(flag: AddressFlag, sockaddr: Option<SockaddrStorage>) -> Result<Self> {
        let constructor = match flag {
            AddressFlag::RTA_GATEWAY => Self::Gateway,
            AddressFlag::RTA_DST => Self::Destination,
            AddressFlag::RTA_NETMASK => Self::Netmask,
            AddressFlag::RTA_GENMASK => Self::CloningMask,
            AddressFlag::RTA_IFP => Self::IfName,
            AddressFlag::RTA_IFA => Self::IfSockaddr,
            AddressFlag::RTA_AUTHOR => Self::RedirectAuthor,
            AddressFlag::RTA_BRD => Self::Broadcast,
            unknown => return Err(Error::UnknownAddressFlag(unknown.bits())),
        };

        Ok(constructor(sockaddr))
    }
}

/// Route socket addresses should be ordered by their corresponding address flag when a route
/// message is constructed
impl std::cmp::PartialOrd for RouteSocketAddress {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.address_flag().partial_cmp(&other.address_flag())
    }
}

/// An iterator to consume a byte buffer containing socket address structures originating from a
/// routing socket message.
pub struct RouteSockAddrIterator<'a> {
    buffer: &'a [u8],
    /// Iterator over all the set bits in the provided [AddressFlag].
    flags_iter: bitflags::iter::Iter<AddressFlag>,
}

impl<'a> RouteSockAddrIterator<'a> {
    fn new(buffer: &'a [u8], flags: AddressFlag) -> Self {
        Self {
            buffer,
            flags_iter: flags.iter(),
        }
    }

    /// Advances internal byte buffer by given amount. The byte amount will be padded to be
    /// aligned to 4 bytes if there's more data in the buffer.
    fn advance_buffer(&mut self, saddr_len: u8) {
        let saddr_len = usize::from(saddr_len);

        // if consumed as many bytes as are left in the buffer, the buffer can be cleared
        if saddr_len == self.buffer.len() {
            self.buffer = &[];
            return;
        }

        let padded_saddr_len = if saddr_len % 4 != 0 {
            saddr_len + (4 - saddr_len % 4)
        } else {
            saddr_len
        };

        // if offset is larger than current buffer, ensure slice gets truncated
        // since the socket address should've already be read from the buffer at this point, this
        // probably should be an invariant?
        self.buffer = &self.buffer[padded_saddr_len..];
    }
}

impl Iterator for RouteSockAddrIterator<'_> {
    type Item = Result<RouteSocketAddress>;

    fn next(&mut self) -> Option<Self::Item> {
        // Will return None if it runs out of set flags.
        let current_flag = self.flags_iter.next()?;

        // Any undefined flags are all returned as a clump in the final iteration.
        let no_undefined_flags = AddressFlag::all().contains(current_flag);
        debug_assert!(
            no_undefined_flags,
            "AddressFlag contained undefined bits! {current_flag:?}. \
            Consider adding them to the definition."
        );

        match RouteSocketAddress::new(current_flag, self.buffer) {
            Ok((next_addr, addr_len)) => {
                self.advance_buffer(addr_len);
                Some(Ok(next_addr))
            }
            Err(err) => {
                self.buffer = &[];
                Some(Err(err))
            }
        }
    }
}

fn saddr_to_ipv4(saddr: &SockaddrStorage) -> Option<Ipv4Addr> {
    let addr = saddr.as_sockaddr_in()?;
    Some(*SocketAddrV4::from(*addr).ip())
}

fn saddr_to_ipv6(saddr: &SockaddrStorage) -> Option<Ipv6Addr> {
    let addr = saddr.as_sockaddr_in6()?;
    Some(*SocketAddrV6::from(*addr).ip())
}

/// A route destination for [RouteMessage].
#[derive(PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct RouteDestination {
    /// The destination network
    pub network: IpNetwork,
    /// Interface index, if the route is scoped (RTF_IFSCOPE)
    pub ifscope_interface: Option<u16>,
    /// Gateway IP address
    pub gateway: Option<IpAddr>,
}

impl TryFrom<&RouteMessage> for RouteDestination {
    type Error = Error;

    fn try_from(msg: &RouteMessage) -> std::result::Result<Self, Self::Error> {
        let network = msg.destination_ip()?.ok_or(Error::NoDestination)?;
        let interface = msg.ifscope().then(|| msg.interface_index());
        let gateway = msg.gateway_ip();
        Ok(Self {
            network,
            ifscope_interface: interface,
            gateway,
        })
    }
}

/// Types from C headers that may not be available in libc crate
pub mod ffi {
    use std::ffi::{c_int, c_uchar, c_ushort};

    /// Socket address header
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct sockaddr_hdr {
        /// Socket address length
        pub sa_len: u8,
        /// Socket address family
        pub sa_family: libc::sa_family_t,
        /// Padding
        pub padding: u16,
    }

    /// Partial rt_msghdr that matches all route messages
    ///
    /// Docs: https://github.com/apple-open-source/macos/blob/4e997debe4327c8a40fe8b87b15640f3befdb53c/xnu/bsd/net/route.h#L159
    #[derive(Debug)]
    #[repr(C)]
    pub struct rt_msghdr_short {
        /// to skip over non-understood messages
        pub rtm_msglen: c_ushort,
        /// future binary compatibility
        pub rtm_version: c_uchar,
        /// message type
        pub rtm_type: c_uchar,
        /// index for associated ifp
        pub rtm_index: c_ushort,
        /// flags, incl. kern & message, e.g. DONE
        pub rtm_flags: c_int,
        /// bitmask identifying sockaddrs in msg
        pub rtm_addrs: c_int,
        /// identify sender
        pub rtm_pid: libc::pid_t,
        /// for sender to identify action
        pub rtm_seq: c_int,
        /// why it failed
        pub rtm_errno: c_int,
    }
}

impl ffi::rt_msghdr_short {
    const SIZE: usize = std::mem::size_of::<ffi::rt_msghdr_short>();

    /// Check if the message is of the expected type (i.e., check rtm_type)
    pub fn is_type(&self, expected_type: i32) -> bool {
        u8::try_from(expected_type)
            .map(|expected| self.rtm_type == expected)
            .unwrap_or(false)
    }

    /// Parse a raw buffer into a `rt_msghdr_short`
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() >= Self::SIZE {
            let ptr = buf.as_ptr();
            // SAFETY: `ptr` is backed by enough valid bytes to contain a rt_msghdr_short value and
            // is readable. `rt_msghdr_short` doesn't contain any pointers so any values are valid.
            Some(unsafe { std::ptr::read(ptr as *const ffi::rt_msghdr_short) })
        } else {
            None
        }
    }
}

/// Parse a raw buffer into a `rt_msghdr` and the remaining payload/body.
pub fn split_rtmsg_hdr(buf: &[u8]) -> Result<(libc::rt_msghdr, &[u8])> {
    const SIZE: usize = std::mem::size_of::<libc::rt_msghdr>();

    let header: libc::rt_msghdr = if buf.len() >= SIZE {
        let ptr = buf.as_ptr();
        // SAFETY: `ptr` is backed by enough valid bytes to contain a rt_msghdr value and it's
        // readable. rt_msghdr doesn't contain any pointers so any values are valid.
        unsafe { std::ptr::read(ptr as *const _) }
    } else {
        return Err(Error::BufferTooSmall {
            message_type: "rt_msghdr",
            expect_min_size: SIZE,
            actual_size: buf.len(),
        });
    };

    let msg_len = usize::from(header.rtm_msglen);
    if msg_len > buf.len() {
        return Err(Error::BufferTooSmall {
            message_type: "route message (rt_msghdr.msg_len)",
            expect_min_size: msg_len,
            actual_size: buf.len(),
        });
    }

    // NOTE: rtm_msglen includes the header size
    let payload = &buf[mem::size_of::<libc::rt_msghdr>()..msg_len];

    Ok((header, payload))
}

#[test]
fn test_failing_rtmsg() {
    let bytes = [
        135, 0, 5, 1, 11, 0, 0, 0, 1, 1, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 16, 2, 0, 0, 192, 168, 88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 18, 11, 0, 6, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 255, 255, 255, 255, 255, 255,
    ];
    let _ = RouteSocketMessage::parse_message(&bytes).unwrap();
}

// Set MTU flag. See route.h
const RTV_MTU: u32 = 0x1;
