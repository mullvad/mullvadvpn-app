use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    sync::LazyLock,
};

/// When "allow local network" is enabled the app will allow traffic to and from these networks.
pub static ALLOWED_LAN_NETS: LazyLock<[IpNetwork; 6]> = LazyLock::new(|| {
    [
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(172, 16, 0, 0), 12).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(169, 254, 0, 0), 16).unwrap()),
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10).unwrap()),
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7).unwrap()),
    ]
});
/// When "allow local network" is enabled the app will allow traffic to these networks.
pub static ALLOWED_LAN_MULTICAST_NETS: LazyLock<[IpNetwork; 8]> = LazyLock::new(|| {
    [
        // Local network broadcast. Not routable
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(255, 255, 255, 255), 32).unwrap()),
        // Local subnetwork multicast. Not routable
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(224, 0, 0, 0), 24).unwrap()),
        // Admin-local IPv4 multicast.
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(239, 0, 0, 0), 8).unwrap()),
        // Interface-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff01, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Link-local IPv6 multicast. IPv6 equivalent of 224.0.0.0/24
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Realm-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff03, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Admin-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff04, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Site-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
    ]
});
