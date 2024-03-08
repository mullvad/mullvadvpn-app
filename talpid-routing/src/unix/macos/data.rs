use ipnetwork::IpNetwork;
use nix::{
    ifaddrs::InterfaceAddress,
    sys::socket::{SockaddrLike, SockaddrStorage},
};
use std::{
    collections::BTreeMap,
    ffi::{c_int, c_uchar, c_ushort},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

/// Message that describes a route - either an added, removed, changed or plainly retrieved route.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteMessage {
    sockaddrs: BTreeMap<AddressFlag, RouteSocketAddress>,
    mtu: u32,
    route_flags: RouteFlag,
    interface_index: u16,
    errno: i32,
}

impl RouteMessage {
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

    pub fn route_addrs(&self) -> impl Iterator<Item = &RouteSocketAddress> {
        self.sockaddrs.values()
    }

    fn socketaddress_to_ip(sockaddr: &SockaddrStorage) -> Option<IpAddr> {
        saddr_to_ipv4(sockaddr)
            .map(Into::into)
            .or_else(|| saddr_to_ipv6(sockaddr).map(Into::into))
    }

    pub fn netmask(&self) -> Option<IpAddr> {
        self.route_addrs()
            .find_map(|saddr| match saddr {
                RouteSocketAddress::Netmask(netmask) => Some(netmask),
                _ => None,
            })?
            .as_ref()
            .and_then(Self::socketaddress_to_ip)
    }

    pub fn is_default(&self) -> Result<bool> {
        Ok(self.is_default_v4()? || self.is_default_v6()?)
    }

    pub fn is_default_v4(&self) -> Result<bool> {
        let destination_is_default = self
            .destination_v4()?
            .map(|addr| addr == Ipv4Addr::UNSPECIFIED)
            .unwrap_or(false);
        let netmask = self.route_addrs().find_map(|saddr| match saddr {
            RouteSocketAddress::Netmask(addr) => Some(addr),
            _ => None,
        });

        // TODO: This might be superfluous
        let netmask_is_default = match netmask {
            // empty socket address implies that it is a 'default' netmask
            Some(None) => true,
            Some(Some(addr)) => {
                if let Some(netmask_addr) = saddr_to_ipv4(addr) {
                    netmask_addr.is_unspecified()
                } else if let Some(netmask_addr) = saddr_to_ipv6(addr) {
                    netmask_addr.is_unspecified()
                } else {
                    // if the route socket address describing the netmask isn't a sockaddr_in or a
                    // sockaddr_in6, it can't possibly be a default route for IP
                    false
                }
            }
            // absence of a netmask socket address implies that it is a host route
            None => false,
        };

        Ok(destination_is_default && netmask_is_default)
    }

    pub fn is_default_v6(&self) -> Result<bool> {
        Ok(self
            .destination_v6()?
            .map(|addr| addr == Ipv6Addr::UNSPECIFIED)
            .unwrap_or(false))
    }

    fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
        let header: rt_msghdr = rt_msghdr::from_bytes(buffer)?;

        let msg_len = usize::from(header.rtm_msglen);
        if msg_len > buffer.len() {
            return Err(Error::BufferTooSmall {
                message_type: "route message (rt_msghdr.msg_len)",
                expect_min_size: msg_len,
                actual_size: buffer.len(),
            });
        }

        let payload = &buffer[ROUTE_MESSAGE_HEADER_SIZE..std::cmp::min(msg_len, buffer.len())];

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

    pub fn set_mtu(mut self, mtu: u32) -> Self {
        self.mtu = mtu;
        self
    }

    pub fn set_interface_addr(mut self, link: &InterfaceAddress) -> Self {
        self.insert_sockaddr(RouteSocketAddress::Gateway(link.address));
        self.route_flags |= RouteFlag::RTF_GATEWAY;
        self
    }

    pub fn set_gateway_sockaddr(mut self, sockaddr: SockaddrStorage) -> Self {
        self.insert_sockaddr(RouteSocketAddress::Gateway(Some(sockaddr)));
        self.route_flags |= RouteFlag::RTF_GATEWAY;
        self
    }

    pub fn set_gateway_addr(mut self, gateway: impl Into<SockaddrStorage>) -> Self {
        self.insert_sockaddr(RouteSocketAddress::Gateway(Some(gateway.into())));
        self.route_flags |= RouteFlag::RTF_GATEWAY;

        self
    }

    pub fn set_gateway_route(mut self, is_gateway_route: bool) -> Self {
        if is_gateway_route {
            self.route_flags.insert(RouteFlag::RTF_GATEWAY);
        } else {
            self.route_flags.remove(RouteFlag::RTF_GATEWAY);
        }
        self
    }

    pub fn route_flag(mut self, route_flags: RouteFlag) -> Self {
        self.route_flags = route_flags;
        self
    }

    pub fn gateway(&self) -> Option<&SockaddrStorage> {
        self.route_addrs()
            .find_map(|saddr| match saddr {
                RouteSocketAddress::Gateway(gateway) => Some(gateway),
                _ => None,
            })?
            .as_ref()
    }

    pub fn gateway_ip(&self) -> Option<IpAddr> {
        self.gateway_v4()
            .map(IpAddr::V4)
            .or(self.gateway_v6().map(IpAddr::V6))
    }

    pub fn gateway_v4(&self) -> Option<Ipv4Addr> {
        saddr_to_ipv4(self.gateway()?)
    }

    pub fn gateway_v6(&self) -> Option<Ipv6Addr> {
        saddr_to_ipv6(self.gateway()?)
    }

    pub fn destination_ip(&self) -> Result<IpNetwork> {
        if let Some(saddr) = self.destination()? {
            if let Some(v4) = saddr.as_sockaddr_in() {
                let ip_addr = *SocketAddrV4::from(*v4).ip();
                let netmask = self.netmask().unwrap_or(Ipv4Addr::UNSPECIFIED.into());
                let destination = IpNetwork::with_netmask(ip_addr.into(), netmask)
                    .map_err(|_| Error::InvalidNetmask)?;
                return Ok(destination);
            }

            if let Some(v6) = saddr.as_sockaddr_in6() {
                let ip_addr = *SocketAddrV6::from(*v6).ip();
                let netmask = self.netmask().unwrap_or(Ipv6Addr::UNSPECIFIED.into());
                let destination = IpNetwork::with_netmask(ip_addr.into(), netmask)
                    .map_err(|_| Error::InvalidNetmask)?;
                return Ok(destination);
            }

            return Err(Error::MismatchedSocketAddress(
                AddressFlag::RTA_DST,
                Box::new(*saddr),
            ));
        }
        Err(Error::NoDestination)
    }

    pub fn destination(&self) -> Result<Option<&SockaddrStorage>> {
        Ok(self
            .route_addrs()
            .find_map(|saddr| match saddr {
                RouteSocketAddress::Destination(destination) => Some(destination),
                _ => None,
            })
            .ok_or(Error::NoDestination)?
            .as_ref())
    }

    pub fn destination_v4(&self) -> Result<Option<Ipv4Addr>> {
        Ok(self.destination()?.and_then(saddr_to_ipv4))
    }

    pub fn destination_v6(&self) -> Result<Option<Ipv6Addr>> {
        Ok(self.destination()?.and_then(saddr_to_ipv6))
    }

    pub fn flags(&self) -> &RouteFlag {
        &self.route_flags
    }

    pub fn payload(
        &self,
        message_type: MessageType,
        sequence: i32,
        pid: i32,
    ) -> (rt_msghdr, Vec<Vec<u8>>) {
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

        let rtm_msglen = (payload_len + ROUTE_MESSAGE_HEADER_SIZE)
            .try_into()
            .expect("route message buffer size cannot fit in 32 bits");

        let mut header = super::data::rt_msghdr {
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
            rtm_rmx: Default::default(),
        };

        if self.mtu != 0 {
            header.rtm_inits |= RTV_MTU;
            header.rtm_rmx.rmx_mtu = self.mtu;
        }

        (header, payload_bytes)
    }

    pub fn interface_index(&self) -> u16 {
        self.interface_index
    }

    pub fn set_interface_index(mut self, index: u16) -> Self {
        self.interface_index = index;
        self
    }

    pub fn interface_address(&self) -> Option<IpAddr> {
        self.get_address(&AddressFlag::RTA_IFA)
    }

    fn get_address(&self, address_flag: &AddressFlag) -> Option<IpAddr> {
        let addr = self.sockaddrs.get(address_flag)?;
        saddr_to_ipv4(addr.inner()?)
            .map(IpAddr::from)
            .or_else(|| saddr_to_ipv6(addr.inner()?).map(IpAddr::from))
    }

    pub fn interface_sockaddr_index(&self) -> Option<u16> {
        self.sockaddrs
            .values()
            .find_map(|addr| addr.interface_index())
    }

    pub fn errno(&self) -> i32 {
        self.errno
    }

    pub fn is_ipv4(&self) -> bool {
        self.destination_v4()
            .map(|addr| addr.is_some())
            .unwrap_or(false)
    }

    pub fn is_ipv6(&self) -> bool {
        self.destination_v6()
            .map(|addr| addr.is_some())
            .unwrap_or(false)
    }

    pub fn is_ifscope(&self) -> bool {
        self.route_flags.contains(RouteFlag::RTF_IFSCOPE)
    }

    pub fn ifscope(&self) -> Option<u16> {
        if self.is_ifscope() {
            Some(self.interface_index)
        } else {
            None
        }
    }

    pub fn unset_ifscope(mut self) -> Self {
        self.route_flags.remove(RouteFlag::RTF_IFSCOPE);
        self
    }

    pub fn set_ifscope(mut self, iface_index: u16) -> Self {
        if iface_index > 0 {
            self.interface_index = iface_index;
            self.route_flags.insert(RouteFlag::RTF_IFSCOPE);
        } else {
            self.route_flags.remove(RouteFlag::RTF_IFSCOPE);
        }

        self
    }
}

#[derive(Debug)]
#[repr(C)]
struct ifa_msghdr {
    ifam_msglen: c_ushort,
    ifam_version: c_uchar,
    ifam_type: c_uchar,
    ifam_addrs: c_int,
    ifam_flags: c_int,
    ifam_index: c_ushort,
    ifam_metric: c_int,
}

#[derive(Debug)]
pub struct AddressMessage {
    sockaddrs: BTreeMap<AddressFlag, RouteSocketAddress>,
    interface_index: u16,
}

impl AddressMessage {
    pub fn index(&self) -> u16 {
        self.interface_index
    }

    pub fn address(&self) -> Result<IpAddr> {
        self.get_address(&AddressFlag::RTA_IFP)
            .or_else(|| self.get_address(&AddressFlag::RTA_IFA))
            .ok_or(Error::NoInterfaceAddress)
    }

    fn get_address(&self, address_flag: &AddressFlag) -> Option<IpAddr> {
        let addr = self.sockaddrs.get(address_flag)?;
        saddr_to_ipv4(addr.inner()?)
            .map(IpAddr::from)
            .or_else(|| saddr_to_ipv6(addr.inner()?).map(IpAddr::from))
    }

    pub fn netmask(&self) -> Result<IpAddr> {
        self.get_address(&AddressFlag::RTA_NETMASK)
            .ok_or(Error::NoNetmask)
    }

    pub fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
        const HEADER_SIZE: usize = std::mem::size_of::<ifa_msghdr>();
        if HEADER_SIZE > buffer.len() {
            return Err(Error::BufferTooSmall {
                message_type: "ifa_msghdr",
                expect_min_size: HEADER_SIZE,
                actual_size: buffer.len(),
            });
        }

        // SAFETY: buffer is pointing to enough memory to contain a valid value for ifa_msghdr
        let header: ifa_msghdr = unsafe { std::ptr::read_unaligned(buffer.as_ptr() as *const _) };

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

#[derive(Debug)]
pub enum RouteSocketMessage {
    AddRoute(RouteMessage),
    DeleteRoute(RouteMessage),
    ChangeRoute(RouteMessage),
    GetRoute(RouteMessage),
    Interface(Interface),
    AddAddress(AddressMessage),
    DeleteAddress(AddressMessage),
    Other {
        header: rt_msghdr_short,
        payload: Vec<u8>,
    },
    Error {
        header: rt_msghdr_short,
        payload: Vec<u8>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Destination {
    Host(IpAddr),
    Network(IpNetwork),
}

impl Destination {
    pub fn is_network(&self) -> bool {
        matches!(self, Self::Network(_))
    }

    pub fn default_v4() -> Self {
        Destination::Network(IpNetwork::new(Ipv4Addr::UNSPECIFIED.into(), 0).unwrap())
    }

    pub fn default_v6() -> Self {
        Destination::Network(IpNetwork::new(Ipv6Addr::UNSPECIFIED.into(), 0).unwrap())
    }
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Payload buffer didn't match the reported message size in header
    #[error("Buffer didn't match reported message size")]
    InvalidBuffer(Vec<u8>, AddressFlag),
    /// Buffer too small for specific message type
    #[error("The buffer is too small for msg \"{message_type}\": expected size >= {expect_min_size}, actual {actual_size}")]
    BufferTooSmall {
        message_type: &'static str,
        expect_min_size: usize,
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

impl RouteSocketMessage {
    pub fn parse_message(buffer: &[u8]) -> Result<Self> {
        let route_message = |route_constructor: fn(RouteMessage) -> RouteSocketMessage, buffer| {
            let route = RouteMessage::from_byte_buffer(buffer)?;
            Ok(route_constructor(route))
        };

        match rt_msghdr_short::from_bytes(buffer) {
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
                expect_min_size: ROUTE_MESSAGE_HEADER_SHORT_SIZE,
                actual_size: buffer.len(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct Interface {
    header: libc::if_msghdr,
}

impl Interface {
    pub fn is_up(&self) -> bool {
        self.header.ifm_flags & nix::libc::IFF_UP != 0
    }

    pub fn index(&self) -> u16 {
        self.header.ifm_index
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
        let header: libc::if_msghdr = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
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

#[derive(Debug, Clone, PartialEq)]
pub enum RouteSocketAddress {
    /// Corresponds to RTA_DST
    Destination(Option<SockaddrStorage>),
    /// RTA_GATEWAY
    Gateway(Option<SockaddrStorage>),
    /// RTA_NETMASK
    Netmask(Option<SockaddrStorage>),
    /// RTA_GENMASK
    CloningMask(Option<SockaddrStorage>),
    /// RTA_IFP
    IfName(Option<SockaddrStorage>),
    /// RTA_IFA
    IfSockaddr(Option<SockaddrStorage>),
    /// RTA_AUTHOR
    RedirectAuthor(Option<SockaddrStorage>),
    /// RTA_BRD
    Broadcast(Option<SockaddrStorage>),
}

impl RouteSocketAddress {
    // Returns a new route socket address and number of bytes read from the buffer
    pub fn new(flag: AddressFlag, buf: &[u8]) -> Result<(Self, u8)> {
        // If buffer is empty, then the socket address is empty too, the backing buffer shouldn't
        // be advanced.
        if buf.is_empty() {
            return Ok((Self::with_sockaddr(flag, None)?, 0));
        }

        // to get the length and type of
        if buf.len() < std::mem::size_of::<sockaddr_hdr>() {
            return Err(Error::BufferTooSmall {
                message_type: "sockaddr_hdr",
                expect_min_size: std::mem::size_of::<sockaddr_hdr>(),
                actual_size: buf.len(),
            });
        }

        let addr_header_ptr = buf.as_ptr() as *const sockaddr_hdr;
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

    pub fn to_bytes(&self) -> Vec<u8> {
        match self.inner() {
            None => vec![0u8; 4],
            Some(addr) => {
                let len = usize::try_from(addr.len()).unwrap();
                assert!(len >= 4);

                // The "serialized" socket addresses must be padded to be aligned to 4 bytes, with
                // the smallest size being 4 bytes.
                let buffer_size = len + len % 4;
                let mut buffer = vec![0u8; buffer_size];
                unsafe {
                    // SAFETY: copying conents of addr into buffer is safe, as long as addr.len()
                    // returns a correct size for the socket address pointer.
                    std::ptr::copy_nonoverlapping(
                        addr.as_ptr() as *const _,
                        buffer.as_mut_ptr(),
                        len,
                    );
                }
                buffer
            }
        }
    }

    pub fn address_flag(&self) -> AddressFlag {
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

    pub fn inner(&self) -> Option<&SockaddrStorage> {
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

    pub fn interface_index(&self) -> Option<u16> {
        match self {
            Self::IfName(Some(iface)) => {
                let index = iface.as_link_addr()?.ifindex();
                Some(
                    u16::try_from(index)
                        .expect("interface indexes actually are u16s, nix is just *interesting*"),
                )
            }
            _ => None,
        }
    }
}

/// Route socket addreses should be ordered by their corresponding address flag when a route
/// message is constructed
impl std::cmp::PartialOrd for RouteSocketAddress {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.address_flag().partial_cmp(&other.address_flag())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct sockaddr_hdr {
    sa_len: u8,
    sa_family: libc::sa_family_t,
    padding: u16,
}

/// An iterator to consume a byte buffer containing socket address structures originating from a
/// routing socket message.
pub struct RouteSockAddrIterator<'a> {
    buffer: &'a [u8],
    flags: AddressFlag,
    // Cursor used to iterate through address flags
    flag_cursor: i32,
}

impl<'a> RouteSockAddrIterator<'a> {
    fn new(buffer: &'a [u8], flags: AddressFlag) -> Self {
        Self {
            buffer,
            flags,
            flag_cursor: AddressFlag::RTA_DST.bits(),
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

impl<'a> Iterator for RouteSockAddrIterator<'a> {
    type Item = Result<RouteSocketAddress>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If address flags don't contain the current one, try the next one.
            // Will return None if it runs out of valid flags.
            let current_flag = AddressFlag::from_bits(self.flag_cursor)?;
            self.flag_cursor <<= 1;

            if !self.flags.contains(current_flag) {
                continue;
            }
            return match RouteSocketAddress::new(current_flag, self.buffer) {
                Ok((next_addr, addr_len)) => {
                    self.advance_buffer(addr_len);
                    Some(Ok(next_addr))
                }
                Err(err) => {
                    self.buffer = &[];
                    Some(Err(err))
                }
            };
        }
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
const ROUTE_MESSAGE_HEADER_SIZE: usize = std::mem::size_of::<rt_msghdr>();

fn saddr_to_ipv4(saddr: &SockaddrStorage) -> Option<Ipv4Addr> {
    let addr = saddr.as_sockaddr_in()?;
    Some(*SocketAddrV4::from(*addr).ip())
}

fn saddr_to_ipv6(saddr: &SockaddrStorage) -> Option<Ipv6Addr> {
    let addr = saddr.as_sockaddr_in6()?;
    Some(*SocketAddrV6::from(*addr).ip())
}

impl rt_msghdr {
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() >= ROUTE_MESSAGE_HEADER_SIZE {
            let ptr = buf.as_ptr();
            // SAFETY: `ptr` is backed by enough valid bytes to contain a rt_msghdr value and it's
            // readable. rt_msghdr doesn't contain any pointers so any values are valid.
            Ok(unsafe { std::ptr::read(ptr as *const _) })
        } else {
            Err(Error::BufferTooSmall {
                message_type: "rt_msghdr",
                expect_min_size: ROUTE_MESSAGE_HEADER_SIZE,
                actual_size: buf.len(),
            })
        }
    }
}

/// Shorter rt_msghdr version that matches all routing messages
#[derive(Debug)]
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
    fn is_type(&self, expected_type: i32) -> bool {
        u8::try_from(expected_type)
            .map(|expected| self.rtm_type == expected)
            .unwrap_or(false)
    }

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
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct RouteDestination {
    pub network: IpNetwork,
    pub interface: Option<u16>,
    pub gateway: Option<IpAddr>,
}

impl TryFrom<&RouteMessage> for RouteDestination {
    type Error = Error;

    fn try_from(msg: &RouteMessage) -> std::result::Result<Self, Self::Error> {
        let network = msg.destination_ip()?;
        let interface = msg.ifscope();
        let gateway = msg.gateway_ip();
        Ok(Self {
            network,
            interface,
            gateway,
        })
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
