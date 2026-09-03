use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// When "allow local network" is enabled the app will allow traffic to and from these networks.
pub const ALLOWED_LAN_NETS: [IpNetwork; 6] = [
    v4(Ipv4Addr::new(10, 0, 0, 0), 8),
    v4(Ipv4Addr::new(172, 16, 0, 0), 12),
    v4(Ipv4Addr::new(192, 168, 0, 0), 16),
    v4(Ipv4Addr::new(169, 254, 0, 0), 16),
    v6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10),
    v6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7),
];

/// Private networks that are legitimately reachable *inside* a Mullvad tunnel.
///
/// [`ALLOWED_LAN_NETS`] covers the entire private address space, but a handful of those addresses
/// are Mullvad's own and only exist on the other side of the tunnel. Traffic to them must keep
/// working when LAN traffic is dropped on the tunnel interface.
pub const ALLOWED_IN_TUNNEL_LAN_NETS: [IpNetwork; 2] = [
    // The relay IPv4 gateway. Hosts the DNS resolver, the tunnel config service used for DAITA and
    // PQ key exchange, and connectivity check pings. Generous to avoid restricting potential future
    // ranges.
    v4(Ipv4Addr::new(10, 0, 0, 0), 8),
    // The relay IPv6 gateway.
    v6(
        Ipv6Addr::new(0xfc00, 0xbbbb, 0xbbbb, 0xbb01, 0, 0, 0, 0),
        64,
    ),
];

/// When "allow local network" is enabled the app will allow traffic to these networks.
pub const ALLOWED_LAN_MULTICAST_NETS: [IpNetwork; 8] = [
    // Local network broadcast. Not routable
    v4(Ipv4Addr::new(255, 255, 255, 255), 32),
    // Local subnetwork multicast. Not routable
    v4(Ipv4Addr::new(224, 0, 0, 0), 24),
    // Admin-local IPv4 multicast.
    v4(Ipv4Addr::new(239, 0, 0, 0), 8),
    // Interface-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff01, 0, 0, 0, 0, 0, 0, 0), 16),
    // Link-local IPv6 multicast. IPv6 equivalent of 224.0.0.0/24
    v6(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0), 16),
    // Realm-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff03, 0, 0, 0, 0, 0, 0, 0), 16),
    // Admin-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff04, 0, 0, 0, 0, 0, 0, 0), 16),
    // Site-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 0, 0), 16),
];

// Short-hand for `IpNetwork::V4(Ipv4Network::new_checked(address, prefix).unwrap())`.
const fn v4(address: Ipv4Addr, prefix: u8) -> IpNetwork {
    IpNetwork::V4(Ipv4Network::new_checked(address, prefix).unwrap())
}

// Short-hand for `IpNetwork::V6(Ipv6Network::new_checked(address, prefix).unwrap())`.
const fn v6(address: Ipv6Addr, prefix: u8) -> IpNetwork {
    IpNetwork::V6(Ipv6Network::new_checked(address, prefix).unwrap())
}

/// Whether `ip` should be an allowed remote address in the VPN tunnel.
#[inline]
pub fn is_ip_allowed_in_tunnel(ip: IpAddr) -> bool {
    ALLOWED_IN_TUNNEL_LAN_NETS
        .iter()
        .any(|net| net.contains(ip))
        || !ALLOWED_LAN_NETS
            .iter()
            .chain(ALLOWED_LAN_MULTICAST_NETS.iter())
            .any(|net| net.contains(ip))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_in_tunnel_nets() {
        // - IPv4 gateway/DNS: 10.64.0.1
        // - SOCKS proxies: 10.124.0.0/23
        // - Possible future range: 10.128.0.1
        let must_allow: Vec<IpAddr> = ["10.64.0.1", "10.128.0.1", "10.124.0.2", "100.64.0.1"]
            .iter()
            .map(|ip| ip.parse().unwrap())
            .collect();

        for ip in must_allow {
            assert!(
                is_ip_allowed_in_tunnel(ip),
                "{ip} must be allowed in tunnel"
            );
        }

        let must_disallow = "192.168.1.1".parse().unwrap();
        assert!(
            !is_ip_allowed_in_tunnel(must_disallow),
            "{must_disallow} must not be allowed in tunnel"
        );
    }
}
