use super::{
    rt_msghdr, saddr_to_ipv4, saddr_to_ipv6, AddressFlag, Destination, Error, MessageType, Result,
    RouteFlag, RouteSockAddrIterator, RouteSocketAddress, ROUTE_MESSAGE_HEADER_SIZE,
};
use ipnetwork::IpNetwork;
use nix::{ifaddrs::InterfaceAddress, sys::socket::SockaddrStorage};
use std::{
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

const RTV_MTU: u32 = libc::RTV_MTU as u32;

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

    pub(crate) fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
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
                    .map_err(Error::InvalidNetmask)?;
                return Ok(destination);
            }

            if let Some(v6) = saddr.as_sockaddr_in6() {
                let ip_addr = *SocketAddrV6::from(*v6).ip();
                let netmask = self.netmask().unwrap_or(Ipv6Addr::UNSPECIFIED.into());
                let destination = IpNetwork::with_netmask(ip_addr.into(), netmask)
                    .map_err(Error::InvalidNetmask)?;
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

        let mut header = rt_msghdr {
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
