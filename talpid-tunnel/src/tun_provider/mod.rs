use cfg_if::cfg_if;
use ipnetwork::IpNetwork;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

cfg_if! {
    if #[cfg(target_os = "android")] {
        #[path = "android/mod.rs"]
        mod imp;
        use self::imp::{AndroidTunProvider, VpnServiceTun};
        pub use self::imp::Error;

        pub type Tun = VpnServiceTun;
        pub type TunProvider = AndroidTunProvider;
    } else if #[cfg(all(unix, not(target_os = "android")))] {
        #[path = "unix.rs"]
        mod imp;
        use self::imp::{UnixTun, UnixTunProvider};
        pub use self::imp::Error;

        pub type Tun = UnixTun;
        pub type TunProvider = UnixTunProvider;
    } else {
        mod stub;
        use self::stub::StubTunProvider;
        pub use self::stub::Error;

        pub type TunProvider = StubTunProvider;
    }
}

/// Configuration for creating a tunnel device.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TunConfig {
    /// IP addresses for the tunnel interface.
    pub addresses: Vec<IpAddr>,

    /// MTU of the tunnel interface.
    pub mtu: u16,

    /// IPv4 address of the VPN server, and the default IPv4 DNS resolver.
    pub ipv4_gateway: Ipv4Addr,

    /// IPv6 address of the VPN server, and the default IPv6 DNS resolver.
    pub ipv6_gateway: Option<Ipv6Addr>,

    /// Routes to configure for the tunnel.
    pub routes: Vec<IpNetwork>,

    /// Exclude private IPs from the tunnel
    pub allow_lan: bool,

    /// DNS servers to use for the tunnel config.
    /// Unless specified, the gateways will be used for DNS
    pub dns_servers: Option<Vec<IpAddr>>,

    /// Applications to exclude from the tunnel.
    pub excluded_packages: Vec<String>,
}

impl TunConfig {
    /// Return a copy of all gateway addresses
    pub fn gateways(&self) -> Vec<IpAddr> {
        let mut servers = vec![self.ipv4_gateway.into()];
        if let Some(gateway) = self.ipv6_gateway {
            servers.push(gateway.into());
        }
        servers
    }
}

impl Default for TunConfig {
    fn default() -> Self {
        TunConfig {
            addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
            mtu: 1380,
            ipv4_gateway: Ipv4Addr::new(10, 64, 0, 1),
            ipv6_gateway: None,
            routes: vec![
                IpNetwork::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
                    .expect("Invalid IP network prefix for IPv4 address"),
                IpNetwork::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 0)
                    .expect("Invalid IP network prefix for IPv6 address"),
            ],
            allow_lan: false,
            dns_servers: None,
            excluded_packages: vec![],
        }
    }
}
