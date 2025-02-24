#[cfg(target_os = "android")]
use crate::tun_provider::imp::VpnServiceConfig;
use cfg_if::cfg_if;
use ipnetwork::IpNetwork;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::LazyLock,
};

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
    /// Interface name to use.
    #[cfg(target_os = "linux")]
    pub name: Option<String>,

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

    /// Routes to configure for the tunnel.
    #[cfg(target_os = "android")]
    pub fn real_routes(&self) -> Vec<IpNetwork> {
        VpnServiceConfig::new(self.clone())
            .routes
            .iter()
            .map(IpNetwork::from)
            .collect()
    }
}

/// Return a tunnel configuration that routes all traffic inside the tunnel.
///
/// Most values except the routes are nonsensical. This is mostly used as a reasonable default on
/// Android to route all traffic inside the tunnel.
pub fn blocking_config() -> TunConfig {
    TunConfig {
        #[cfg(target_os = "linux")]
        name: None,
        addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
        mtu: 1380,
        ipv4_gateway: Ipv4Addr::new(10, 64, 0, 1),
        ipv6_gateway: None,
        routes: DEFAULT_ROUTES.clone(),
        allow_lan: false,
        dns_servers: None,
        excluded_packages: vec![],
    }
}

static DEFAULT_ROUTES: LazyLock<Vec<IpNetwork>> =
    LazyLock::new(|| vec![*IPV4_DEFAULT_ROUTE, *IPV6_DEFAULT_ROUTE]);
static IPV4_DEFAULT_ROUTE: LazyLock<IpNetwork> = LazyLock::new(|| {
    IpNetwork::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
        .expect("Invalid IP network prefix for IPv4 address")
});
static IPV6_DEFAULT_ROUTE: LazyLock<IpNetwork> = LazyLock::new(|| {
    IpNetwork::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)
        .expect("Invalid IP network prefix for IPv6 address")
});
