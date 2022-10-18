use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// To distinguish between globally addressable and unadressable IPv6 addresses with the current
/// stable Rust standard library, one has to reimplement the nightly code.
pub trait IpAddrExt {
    /// Returns true if the IPv6 address is globally addressable.
    fn is_global(&self) -> bool;
}

fn addr_has_a_chance_to_reach_internet(addr: &IpAddr) -> bool {
    match addr {
        IpAddr::V4(v4) => v4.is_private() || v4.is_global(),

        IpAddr::V6(v6) => IpAddrExt::is_global(v6),
    }
}

impl IpAddrExt for IpAddr {
    fn is_global(&self) -> bool {
        match self {
            IpAddr::V4(addr) => addr.is_global(),
            IpAddr::V6(addr) => addr.is_global(),
        }
    }
}

impl IpAddrExt for Ipv6Addr {
    fn is_global(&self) -> bool {
        !(self.is_unspecified()
            || self.is_loopback()
            // IPv4-mapped Address (`::ffff:0:0/96`)
            || matches!(self.segments(), [0, 0, 0, 0, 0, 0xffff, _, _])
            // IPv4-IPv6 Translat. (`64:ff9b:1::/48`)
            || matches!(self.segments(), [0x64, 0xff9b, 1, _, _, _, _, _])
            // Discard-Only Address Block (`100::/64`)
            || matches!(self.segments(), [0x100, 0, 0, 0, _, _, _, _])
            // IETF Protocol Assignments (`2001::/23`)
            || (matches!(self.segments(), [0x2001, b, _, _, _, _, _, _] if b < 0x200)
                && !(
                    // Port Control Protocol Anycast (`2001:1::1`)
                    u128::from_be_bytes(self.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0001
                    // Traversal Using Relays around NAT Anycast (`2001:1::2`)
                    || u128::from_be_bytes(self.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0002
                    // AMT (`2001:3::/32`)
                    || matches!(self.segments(), [0x2001, 3, _, _, _, _, _, _])
                    // AS112-v6 (`2001:4:112::/48`)
                    || matches!(self.segments(), [0x2001, 4, 0x112, _, _, _, _, _])
                    // ORCHIDv2 (`2001:20::/28`)
                    || matches!(self.segments(), [0x2001, b, _, _, _, _, _, _] if b >= 0x20 && b <= 0x2F)
                ))
            || ipv6_is_documentation(self)
            || ipv6_is_unique_local(self)
            || ipv6_is_unicast_link_local(self))
    }
}

impl IpAddrExt for Ipv4Addr {
    fn is_global(&self) -> bool {
        !(self.octets()[0] == 0 // "This network"
            || self.is_private()
            || ipv4_is_shared(self)
            || self.is_loopback()
            || self.is_link_local()
            // addresses reserved for future protocols (`192.0.0.0/24`)
            ||(self.octets()[0] == 192 && self.octets()[1] == 0 && self.octets()[2] == 0)
            || self.is_documentation()
            || ipv4_is_benchmarking(self)
            || ipv4_is_reserved(self)
            || self.is_broadcast())
    }
}

fn ipv6_is_documentation(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] == 0x2001) && (addr.segments()[1] == 0xdb8)
}

fn ipv6_is_unique_local(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xfe00) == 0xfc00
}

fn ipv6_is_unicast_link_local(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}

fn ipv4_is_shared(addr: &Ipv4Addr) -> bool {
    addr.octets()[0] == 100 && (addr.octets()[1] & 0b1100_0000 == 0b0100_0000)
}

fn ipv4_is_benchmarking(addr: &Ipv4Addr) -> bool {
    addr.octets()[0] == 198 && (addr.octets()[1] & 0xfe) == 18
}

fn ipv4_is_reserved(addr: &Ipv4Addr) -> bool {
    addr.octets()[0] & 240 == 240 && !addr.is_broadcast()
}
