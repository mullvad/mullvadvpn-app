use cfg_if::cfg_if;
use ipnetwork::IpNetwork;
#[cfg(target_os = "android")]
use jnix::IntoJava;
use std::net::IpAddr;
#[cfg(unix)]
use std::os::unix::io::AsRawFd;
#[cfg(target_os = "android")]
use std::os::unix::io::RawFd;
use talpid_types::BoxedError;

cfg_if! {
    if #[cfg(all(unix, not(target_os = "android")))] {
        #[path = "unix.rs"]
        mod imp;
        use self::imp::UnixTunProvider;

        /// Default implementation of `TunProvider` for Unix based operating systems.
        ///
        /// Android has a different mechanism to obtain tunnel interfaces, so it is not supported
        /// here.
        pub type PlatformTunProvider = UnixTunProvider;
    } else {
        mod stub;
        use self::stub::StubTunProvider;

        /// Default stub implementation of `TunProvider` for Android and Windows.
        pub type PlatformTunProvider = StubTunProvider;
    }
}

/// Generic tunnel device.
///
/// Must be associated with a platform specific file descriptor representing the device.
#[cfg(unix)]
pub trait Tun: AsRawFd + Send {
    /// Retrieve the tunnel interface name.
    fn interface_name(&self) -> &str;

    /// Allow a socket to bypass the tunnel.
    #[cfg(target_os = "android")]
    fn bypass(&mut self, socket: RawFd) -> Result<(), BoxedError>;
}

/// Stub tunnel device.
#[cfg(windows)]
pub trait Tun: Send {
    /// Retrieve the tunnel interface name.
    fn interface_name(&self) -> &str;
}

/// Factory of tunnel devices.
pub trait TunProvider: Send + 'static {
    /// Retrieve a tunnel device with the provided configuration.
    fn get_tun(&mut self, config: TunConfig) -> Result<Box<dyn Tun>, BoxedError>;

    /// Open a tunnel device using the previous or the default configuration.
    #[cfg(target_os = "android")]
    fn create_tun_if_closed(&mut self) -> Result<(), BoxedError>;

    /// Close currently active tunnel device.
    #[cfg(target_os = "android")]
    fn close_tun(&mut self);
}

/// Configuration for creating a tunnel device.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(
    target_os = "android",
    jnix(class_name = "net.mullvad.mullvadvpn.model.TunConfig")
)]
pub struct TunConfig {
    /// IP addresses for the tunnel interface.
    pub addresses: Vec<IpAddr>,

    /// IP addresses for the DNS servers to use.
    pub dns_servers: Vec<IpAddr>,

    /// Routes to configure for the tunnel.
    #[cfg_attr(
        target_os = "android",
        jnix(map = "|routes| convert_ip_networks(routes)")
    )]
    pub routes: Vec<IpNetwork>,

    /// Maximum Transmission Unit in the tunnel.
    #[cfg_attr(target_os = "android", jnix(map = "|mtu| mtu as i32"))]
    pub mtu: u16,
}

#[cfg(target_os = "android")]
mod convertible_ip_network {
    use ipnetwork::IpNetwork;
    use jnix::IntoJava;
    use std::net::IpAddr;

    pub fn convert_ip_networks(networks: Vec<IpNetwork>) -> Vec<ConvertibleIpNetwork> {
        networks
            .into_iter()
            .map(ConvertibleIpNetwork::from)
            .collect()
    }

    #[derive(IntoJava)]
    #[jnix(class_name = "net.mullvad.mullvadvpn.model.InetNetwork")]
    pub struct ConvertibleIpNetwork {
        ip_addr: IpAddr,
        prefix: i16,
    }

    impl From<IpNetwork> for ConvertibleIpNetwork {
        fn from(network: IpNetwork) -> Self {
            ConvertibleIpNetwork {
                ip_addr: network.ip(),
                prefix: network.prefix() as i16,
            }
        }
    }
}
#[cfg(target_os = "android")]
use self::convertible_ip_network::convert_ip_networks;
