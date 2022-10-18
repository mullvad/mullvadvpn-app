use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    ffi::{OsStr, OsString},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::unix::prelude::OsStringExt,
};

use ipnetwork::IpNetwork;
use nix::{
    ifaddrs::InterfaceAddress,
    net::if_::if_nametoindex,
    sys::socket::{SockAddr, SockaddrIn, SockaddrIn6, SockaddrLike, SockaddrStorage},
};

/// Message that describes a route - either an added, removed, changed or plainly retrieved route.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteMessage {
    sockaddrs: BTreeMap<AddressFlag, RouteSocketAddress>,
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
                let destination =
                    RouteSocketAddress::Destination(Some(SocketAddr::from((net.ip(), 0)).into()));
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

        let netmask_is_default = match netmask {
            // empty socket address implies that it is a 'default' netmask
            Some(None) => true,
            Some(Some(addr)) => {
                if let Some(netmask_addr) = addr.as_sockaddr_in() {
                    let std_addr = SocketAddrV4::from(netmask_addr.clone());
                    *std_addr.ip() == Ipv4Addr::UNSPECIFIED
                } else if let Some(netmask_addr) = addr.as_sockaddr_in6() {
                    let std_addr = SocketAddrV6::from(netmask_addr.clone());
                    *std_addr.ip() == Ipv6Addr::UNSPECIFIED
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

    pub fn print_route(&self) {
        println!(
            "route is default - {:?} - interface index: {} - is iscoped - {}",
            self.is_default(),
            self.interface_index,
            self.is_ifscope()
        );
        for sa in &self.sockaddrs {
            println!("\t{:?}", &sa);
        }
    }

    fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
        let header: rt_msghdr = rt_msghdr::from_bytes(buffer)?;

        let msg_len = usize::from(header.rtm_msglen);
        if msg_len > buffer.len() {
            return Err(Error::BufferTooSmall(
                "Message is shorter than it's msg_len indicates",
                msg_len,
                buffer.len(),
            ));
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

        Ok(Self {
            route_flags,
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

    pub fn set_interface_addr(mut self, link: &InterfaceAddress) -> Self {
        self.insert_sockaddr(RouteSocketAddress::Gateway(link.address.clone()));
        self.route_flags |= RouteFlag::RTF_GATEWAY;
        self
    }

    pub fn set_gateway_sockaddr(mut self, sockaddr: SockaddrStorage) -> Self {
        self.insert_sockaddr(RouteSocketAddress::Gateway(Some(sockaddr)));
        self.route_flags |= RouteFlag::RTF_GATEWAY;
        self
    }

    pub fn set_gateway_addr(mut self, addr: IpAddr) -> Self {
        let gateway: SocketAddr = (addr, 0).into();
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
                    .map_err(Error::InvalidNetmask)?;
                return Ok(destination.into());
            }

            if let Some(v6) = saddr.as_sockaddr_in6() {
                let ip_addr = *SocketAddrV6::from(*v6).ip();
                let netmask = self.netmask().unwrap_or(Ipv6Addr::UNSPECIFIED.into());
                let destination = IpNetwork::with_netmask(ip_addr.into(), netmask)
                    .map_err(Error::InvalidNetmask)?;
                return Ok(destination.into());
            }

            return Err(Error::MismatchedSocketAddress(
                AddressFlag::RTA_DST,
                saddr.clone(),
            ));
        };
        return Err(Error::NoDestination);
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

        let mut sockaddrs = self.route_addrs().collect::<Vec<_>>();
        sockaddrs.sort_by_key(|saddr| saddr.address_flag());
        let payload_bytes = sockaddrs
            .into_iter()
            .map(RouteSocketAddress::to_bytes)
            .collect::<Vec<_>>();

        let payload_len: usize = payload_bytes.iter().map(Vec::len).sum();

        let rtm_msglen = (payload_len + std::mem::size_of::<super::data::rt_msghdr>())
            .try_into()
            .expect("route message buffer size cannot fit in 32 bits");

        let header = super::data::rt_msghdr {
            rtm_msglen,
            rtm_version: libc::RTM_VERSION.try_into().unwrap(),
            rtm_type: message_type.bits(),
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

        (header, payload_bytes)
    }

    pub fn interface_index(&self) -> u16 {
        self.interface_index
    }

    pub fn interface_address(&self) -> Option<IpAddr> {
        self.get_address(&AddressFlag::RTA_IFA)
    }

    fn get_address(&self, address_flag: &AddressFlag) -> Option<IpAddr> {
        let addr = self.sockaddrs.get(&address_flag)?;
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
            self.interface_index = iface_index;
            self.route_flags.remove(RouteFlag::RTF_IFSCOPE);
        }

        self
    }
}

#[derive(Debug)]
#[repr(C)]
struct ifa_msghdr {
    ifam_msglen: libc::c_ushort,
    ifam_version: libc::c_uchar,
    ifam_type: libc::c_uchar,
    ifam_addrs: libc::c_int,
    ifam_flags: libc::c_int,
    ifam_index: libc::c_ushort,
    ifam_metric: libc::c_int,
}

#[derive(Debug)]
pub struct AddressMessage {
    sockaddrs: BTreeMap<AddressFlag, RouteSocketAddress>,
    interface_index: u16,
    ifam_type: libc::c_uchar,
    flags: RouteFlag,
}

impl AddressMessage {
    pub fn index(&self) -> u16 {
        self.interface_index
    }

    pub fn print_sockaddrs(&self) {
        println!("ifam_type - {}", self.ifam_type);
        match self.address() {
            Ok(addr) => println!("address - {addr}"),
            Err(err) => {
                println!("failed to get address {err:?}");
            }
        }
        for (flag, addr) in &self.sockaddrs {
            println!("{flag:?} - {addr:?}");
        }
    }

    pub fn address(&self) -> Result<IpAddr> {
        self.get_address(&AddressFlag::RTA_IFP)
            .or_else(|| self.get_address(&AddressFlag::RTA_IFA))
            .ok_or(Error::NoInterfaceAddress)
    }

    fn get_address(&self, address_flag: &AddressFlag) -> Option<IpAddr> {
        let addr = self.sockaddrs.get(&address_flag)?;
        saddr_to_ipv4(addr.inner()?)
            .map(IpAddr::from)
            .or_else(|| saddr_to_ipv6(addr.inner()?).map(IpAddr::from))
    }

    pub fn netmask(&self) -> Result<IpAddr> {
        self.get_address(&AddressFlag::RTA_NETMASK)
            .ok_or(Error::NoNetmaskAddress)
    }

    pub fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
        const HEADER_SIZE: usize = std::mem::size_of::<ifa_msghdr>();
        if HEADER_SIZE > buffer.len() {
            return Err(Error::BufferTooSmall(
                "ifa_msghdr",
                buffer.len(),
                HEADER_SIZE,
            ));
        }

        // SAFETY: buffer is pointing to enough memory to contain a valid value for ifa_msghdr
        let header: ifa_msghdr = unsafe { std::ptr::read_unaligned(buffer.as_ptr() as *const _) };

        let msg_len = usize::from(header.ifam_msglen);
        if msg_len > buffer.len() {
            return Err(Error::BufferTooSmall(
                "Mesage is shorter than it's msg_len indicates",
                msg_len,
                buffer.len(),
            ));
        }

        let payload = &buffer[HEADER_SIZE..std::cmp::min(msg_len, buffer.len())];

        let flags = RouteFlag::from_bits(header.ifam_flags)
            .ok_or(Error::UnknownRouteFlag(header.ifam_flags))?;

        let address_flags = AddressFlag::from_bits(header.ifam_addrs)
            .ok_or(Error::UnknownAddressFlag(header.ifam_addrs))?;

        let sockaddrs = RouteSockAddrIterator::new(payload, address_flags)
            .map(|addr| addr.map(|addr| (addr.address_flag(), addr)))
            .collect::<Result<BTreeMap<_, _>>>()?;

        Ok(Self {
            sockaddrs,
            flags,
            ifam_type: header.ifam_type,
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
    UnknownAddressFlag(libc::c_int),
    /// Mismatched socket address type
    MismatchedSocketAddress(AddressFlag, SockaddrStorage),
    /// Link socket address contains no identifier
    NoLinkIdentifier(nix::libc::sockaddr_dl),
    /// Failed to resolve an interface name to an index
    InterfaceIndex(nix::Error),
    /// An error message as received from the routing socket
    RouteError(rt_msghdr, Vec<u8>),
    /// Invalid netmask
    InvalidNetmask(ipnetwork::IpNetworkError),
    /// Route contains no netmask socket address
    NoDestination,
    /// Address message does not contain an interface address
    NoInterfaceAddress,
    /// Address message does not contain an interface address
    NoNetmaskAddress,
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
            None => Err(Error::BufferTooSmall(
                "rt_msghdr_short",
                buffer.len(),
                std::mem::size_of::<rt_msghdr_short>(),
            )),
        }
    }
}

/// hush, this will come in later
fn align_to_nearest_u32(idx: usize) -> usize {
    if idx > 0 {
        1 + (((idx) - 1) | (std::mem::size_of::<u32>() - 1))
    } else {
        std::mem::size_of::<u32>()
    }
}

pub struct Interface {
    header: libc::if_msghdr,
    payload: Vec<u8>,
}

impl Interface {
    pub fn is_up(&self) -> bool {
        self.header.ifm_flags & nix::libc::IFF_UP != 0
    }

    pub fn index(&self) -> u16 {
        self.header.ifm_index
    }
}

impl std::fmt::Debug for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let if_data = f
            .debug_struct("if_data")
            .field("ifi_type", &self.header.ifm_data.ifi_type)
            .field("ifi_typelen", &self.header.ifm_data.ifi_typelen)
            .field("ifi_physical", &self.header.ifm_data.ifi_physical)
            .field("ifi_addrlen", &self.header.ifm_data.ifi_addrlen)
            .field("ifi_hdrlen", &self.header.ifm_data.ifi_hdrlen)
            .field("ifi_recvquota", &self.header.ifm_data.ifi_recvquota)
            .field("ifi_xmitquota", &self.header.ifm_data.ifi_xmitquota)
            .field("ifi_unused1", &self.header.ifm_data.ifi_unused1)
            .field("ifi_mtu", &self.header.ifm_data.ifi_mtu)
            .field("ifi_metric", &self.header.ifm_data.ifi_metric)
            .field("ifi_baudrate", &self.header.ifm_data.ifi_baudrate)
            .field("ifi_ipackets", &self.header.ifm_data.ifi_ipackets)
            .field("ifi_ierrors", &self.header.ifm_data.ifi_ierrors)
            .field("ifi_opackets", &self.header.ifm_data.ifi_opackets)
            .field("ifi_oerrors", &self.header.ifm_data.ifi_oerrors)
            .field("ifi_collisions", &self.header.ifm_data.ifi_collisions)
            .field("ifi_ibytes", &self.header.ifm_data.ifi_ibytes)
            .field("ifi_obytes", &self.header.ifm_data.ifi_obytes)
            .field("ifi_imcasts", &self.header.ifm_data.ifi_imcasts)
            .field("ifi_omcasts", &self.header.ifm_data.ifi_omcasts)
            .field("ifi_iqdrops", &self.header.ifm_data.ifi_iqdrops)
            .field("ifi_noproto", &self.header.ifm_data.ifi_noproto)
            .field("ifi_recvtiming", &self.header.ifm_data.ifi_recvtiming)
            .field("ifi_xmittiming", &self.header.ifm_data.ifi_xmittiming)
            .field(
                "ifi_lastchange",
                &(
                    self.header.ifm_data.ifi_lastchange.tv_sec,
                    self.header.ifm_data.ifi_lastchange.tv_usec,
                ),
            )
            .field("ifi_unused2", &self.header.ifm_data.ifi_unused2)
            .field("ifi_hwassist", &self.header.ifm_data.ifi_hwassist)
            .field("ifi_reserved1", &self.header.ifm_data.ifi_reserved1)
            .field("ifi_reserved2", &self.header.ifm_data.ifi_reserved2)
            .finish();
        let header = f
            .debug_struct("if_msghdr")
            .field("ifm_msglen", &self.header.ifm_msglen)
            .field("ifm_version", &self.header.ifm_version)
            .field("ifm_type", &self.header.ifm_type)
            .field("ifm_addrs", &self.header.ifm_addrs)
            .field("ifm_flags", &self.header.ifm_flags)
            .field("ifm_index", &self.header.ifm_index)
            .field("ifm_data", &if_data)
            .finish()?;
        f.debug_struct("Interface")
            .field("header", &header)
            .field("payload", &self.payload)
            .finish()
    }
}

impl Interface {
    fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
        const INTERFACE_MESSAGE_HEADER_SIZE: usize = std::mem::size_of::<libc::if_msghdr>();
        if INTERFACE_MESSAGE_HEADER_SIZE > buffer.len() {
            return Err(Error::BufferTooSmall(
                "if_msghdr",
                buffer.len(),
                INTERFACE_MESSAGE_HEADER_SIZE,
            ));
        }
        let header: libc::if_msghdr = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
        let payload = buffer[INTERFACE_MESSAGE_HEADER_SIZE..header.ifm_msglen.into()].to_vec();
        Ok(Self { header, payload })
    }
}

// #define RTA_DST         0x1     /* destination sockaddr present */
// #define RTA_GATEWAY     0x2     /* gateway sockaddr present */
// #define RTA_NETMASK     0x4     /* netmask sockaddr present */
// #define RTA_GENMASK     0x8     /* cloning mask sockaddr present */
// #define RTA_IFP         0x10    /* interface name sockaddr present */
// #define RTA_IFA         0x20    /* interface addr sockaddr present */
// #define RTA_AUTHOR      0x40    /* sockaddr for author of redirect */
// #define RTA_BRD         0x80    /* for NEWADDR, broadcast or p-p dest addr */
bitflags::bitflags! {
    /// All enum values of address flags can be iterated via `flag <<= 1`, starting from 1.
    // #[derive(Clone, Copy, PartialOrd)]
    pub struct AddressFlag: i32 {
        /// Destination socket address
        const RTA_DST       = 0x1;
        /// Gateway socket address
        const RTA_GATEWAY   = 0x2;
        /// Netmask socket address
        const RTA_NETMASK   = 0x4;
        /// Cloning mask socket address
        const RTA_GENMASK   = 0x8;
        /// Interface name socket address
        const RTA_IFP       = 0x10;
        /// Interface address socket address
        const RTA_IFA       = 0x20;
        /// Socket address for author of redirect
        const RTA_AUTHOR    = 0x40;
        /// Socket address for `NEWADDR`, broadcast or point-to-point destination address
        const RTA_BRD       = 0x80;
    }
}

bitflags::bitflags! {
    /// Types of routing messages
    // #[derive(Clone, Copy, PartialOrd)]
    pub struct MessageType: u8 {
        /// Add Route
        const RTM_ADD         = 0x1;
        /// Delete Route
        const RTM_DELETE      = 0x2;
        /// Change Metrics or flags
        const RTM_CHANGE      = 0x3;
        /// Report Metrics
        const RTM_GET         = 0x4;
        /// RTM_LOSING is no longer generated by and is deprecated
        const RTM_LOSING      = 0x5;
        /// Told to use different route
        const RTM_REDIRECT    = 0x6;
        /// Lookup failed on this address
        const RTM_MISS        = 0x7;
        /// fix specified metrics
        const RTM_LOCK        = 0x8;
        /// caused by SIOCADDRT
        const RTM_OLDADD      = 0x9;
        /// caused by SIOCDELRT
        const RTM_OLDDEL      = 0xa;
        /// req to resolve dst to LL addr
        const RTM_RESOLVE     = 0xb;
        /// address being added to iface
        const RTM_NEWADDR     = 0xc;
        /// address being removed from iface
        const RTM_DELADDR     = 0xd;
        /// iface going up/down etc.
        const RTM_IFINFO      = 0xe;
        /// mcast group membership being added to if
        const RTM_NEWMADDR    = 0xf;
        /// mcast group membership being deleted
        const RTM_DELMADDR    = 0x10;
    }



    /// Types of routing messages
    // #[derive(Clone, Copy, PartialOrd)]
    pub struct RouteFlag: i32 {
        /// route usable
        const RTF_UP = 0x1;
        /// destination is a gateway
        const RTF_GATEWAY = 0x2;
        /// host entry (net otherwise)
        const RTF_HOST = 0x4;
        /// host or net unreachable
        const RTF_REJECT = 0x8;
        /// created dynamically (by redirect)
        const RTF_DYNAMIC = 0x10;
        /// modified dynamically (by redirect)
        const RTF_MODIFIED = 0x20;
        /// message confirmed
        const RTF_DONE = 0x40;
        /// delete cloned route
        const RTF_DELCLONE = 0x80;
        /// generate new routes on use
        const RTF_CLONING = 0x100;
        /// external daemon resolves name
        const RTF_XRESOLVE = 0x200;
        /// DEPRECATED - exists ONLY for backwards compatibility
        const RTF_LLINFO = 0x400;
        /// used by apps to add/del L2 entries
        const RTF_LLDATA = 0x400;
        /// manually added
        const RTF_STATIC = 0x800;
        /// just discard pkts (during updates)
        const RTF_BLACKHOLE = 0x1000;
        /// not eligible for RTF_IFREF
        const RTF_NOIFREF = 0x2000;
        /// protocol specific routing flag
        const RTF_PROTO2 = 0x4000;
        /// protocol specific routing flag
        const RTF_PROTO1 = 0x8000;
        /// protocol requires cloning
        const RTF_PRCLONING = 0x10000;
        /// route generated through cloning
        const RTF_WASCLONED = 0x20000;
        /// protocol specific routing flag
        const RTF_PROTO3 = 0x40000;
        /// future use
        const RTF_PINNED = 0x100000;
        /// route represents a local address
        const RTF_LOCAL = 0x200000;
        /// route represents a bcast address
        const RTF_BROADCAST = 0x400000;
        /// route represents a mcast address
        const RTF_MULTICAST = 0x800000;
        /// has valid interface scope
        const RTF_IFSCOPE = 0x1000000;
        /// defunct; no longer modifiable
        const RTF_CONDEMNED = 0x2000000;
        /// route holds a ref to interface
        const RTF_IFREF = 0x4000000;
        /// proxying, no interface scope
        const RTF_PROXY = 0x8000000;
        /// host is a router
        const RTF_ROUTER = 0x10000000;
        /// Route entry is being freed
        const RTF_DEAD = 0x20000000;
        /// route to destination of the global internet
        const RTF_GLOBAL = 0x40000000;
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
            return Err(Error::BufferTooSmall(
                "sockaddr buffer too small",
                buf.len(),
                std::mem::size_of::<sockaddr_hdr>(),
            ));
        }

        let addr_header_ptr = buf.as_ptr() as *const sockaddr_hdr;
        // safety - since `buf` is at least as long as a `sockaddr_hdr`, it's perfectly valid to
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

        return Ok((Self::with_sockaddr(flag, saddr)?, saddr_len));
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self.inner() {
            None => vec![0u8; 4],
            Some(addr) => {
                let len = addr.len();
                // The "serialized" socket addresses must be padded to be aligned to 4 bytes, with
                // the smallest size being 4 bytes.
                let buffer_size = len + len % 4;
                let mut buffer = vec![0u8; buffer_size as usize];
                let mut buffer_ptr = buffer.as_mut_ptr();
                unsafe {
                    // SAFETY: copying conents of addr into buffer is safe, as long as addr.len()
                    // returns a correct size for the socket address pointer.
                    std::ptr::copy_nonoverlapping(
                        addr.as_ptr() as *const _,
                        buffer.as_mut_ptr(),
                        addr.len() as usize,
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

    pub fn set_interface_index(mut self, index: u16) -> Self {
        unimplemented!()
        // self.insert_sockaddr(RouteSocketAddress::IfName(Some(sockaddr)));
        // self
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

pub enum InterfaceIdentifier {
    Index(u16),
    Name(OsString),
    Unspecified,
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
    fn advance_buffer(&mut self, mut saddr_len: u8) {
        let saddr_len = usize::from(saddr_len);

        // if consumed as many bytes as are left in the buffer, the buffer can be cleared
        if saddr_len == self.buffer.len() {
            self.buffer = &[];
            return;
        }

        let padded_saddr_len = if saddr_len % 4 != 0 {
            usize::from(saddr_len + (4 - saddr_len % 4))
        } else {
            usize::from(saddr_len)
        };

        // if offest is larger than current buffer, ensure slice gets truncated
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
    pub rtm_msglen: libc::c_ushort,
    pub rtm_version: libc::c_uchar,
    pub rtm_type: libc::c_uchar,
    pub rtm_index: libc::c_ushort,
    pub rtm_flags: libc::c_int,
    pub rtm_addrs: libc::c_int,
    pub rtm_pid: libc::pid_t,
    pub rtm_seq: libc::c_int,
    pub rtm_errno: libc::c_int,
    pub rtm_use: libc::c_int,
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
        if buf.len() >= std::mem::size_of::<rt_msghdr>() {
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

/// Shorter rt_msghdr version that matches all routing messages
#[derive(Debug)]
#[repr(C)]
pub struct rt_msghdr_short {
    pub rtm_msglen: libc::c_ushort,
    pub rtm_version: libc::c_uchar,
    pub rtm_type: libc::c_uchar,
    pub rtm_index: libc::c_ushort,
    pub rtm_flags: libc::c_int,
    pub rtm_addrs: libc::c_int,
    pub rtm_pid: libc::pid_t,
    pub rtm_seq: libc::c_int,
    pub rtm_errno: libc::c_int,
}

impl rt_msghdr_short {
    fn is_type(&self, expected_type: i32) -> bool {
        u8::try_from(expected_type)
            .map(|expected| self.rtm_type == expected)
            .unwrap_or(false)
    }

    pub fn from_bytes<'a>(buf: &'a [u8]) -> Option<Self> {
        if buf.len() >= std::mem::size_of::<rt_msghdr_short>() {
            let ptr = buf.as_ptr();
            // SAFETY: `ptr` is backed by enough valid bytes to contain a rt_msghdr_short value and
            // it's readable. rt_msghdr_short doesn't contain any pointers so any values are valid.
            Some(unsafe { std::ptr::read(ptr as *const rt_msghdr_short) })
        } else {
            None
        }
    }

    fn is_err(&self) -> bool {
        self.rtm_errno != 0
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct RouteDestination {
    pub network: IpNetwork,
    pub interface: Option<u16>,
    pub gateway: Option<IpAddr>,
}

impl RouteDestination {
    pub fn is_default(&self) -> bool {
        if self.network.prefix() != 0 {
            return false;
        }
        match self.network.ip() {
            IpAddr::V4(Ipv4Addr::UNSPECIFIED) => true,
            IpAddr::V6(Ipv6Addr::UNSPECIFIED) => true,
            _ => false,
        }
    }

    pub fn is_ipv4(&self) -> bool {
        self.network.is_ipv4()
    }
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
