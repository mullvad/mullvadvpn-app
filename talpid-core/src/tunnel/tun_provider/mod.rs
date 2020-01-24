use cfg_if::cfg_if;
use ipnetwork::IpNetwork;
#[cfg(target_os = "android")]
use jnix::IntoJava;
use std::net::IpAddr;

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
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(
    target_os = "android",
    jnix(package = "net.mullvad.talpid.tun_provider")
)]
pub struct TunConfig {
    /// IP addresses for the tunnel interface.
    pub addresses: Vec<IpAddr>,

    /// IP addresses for the DNS servers to use.
    pub dns_servers: Vec<IpAddr>,

    /// Routes to configure for the tunnel.
    #[cfg_attr(
        target_os = "android",
        jnix(map = "|networks| networks.into_iter().map(InetNetwork::from).collect::<Vec<_>>()")
    )]
    pub routes: Vec<IpNetwork>,

    /// Routes that are required to be configured for the tunnel.
    #[cfg(target_os = "android")]
    #[jnix(skip)]
    pub required_routes: Vec<IpNetwork>,

    /// Maximum Transmission Unit in the tunnel.
    #[cfg_attr(target_os = "android", jnix(map = "|mtu| mtu as i32"))]
    pub mtu: u16,
}

#[cfg(target_os = "android")]
#[derive(IntoJava)]
#[jnix(package = "net.mullvad.talpid.tun_provider")]
struct InetNetwork {
    address: IpAddr,
    prefix: i16,
}

#[cfg(target_os = "android")]
impl From<IpNetwork> for InetNetwork {
    fn from(ip_network: IpNetwork) -> Self {
        InetNetwork {
            address: ip_network.ip(),
            prefix: ip_network.prefix() as i16,
        }
    }
}
