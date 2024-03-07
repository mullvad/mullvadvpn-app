use ipnetwork::IpNetwork;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

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
