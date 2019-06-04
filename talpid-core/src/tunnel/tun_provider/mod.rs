use cfg_if::cfg_if;
use ipnetwork::IpNetwork;
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

/// Windows tunnel
#[cfg(target_os = "windows")]
pub mod windows;

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
    ///
    /// Will open a new tunnel if there is already an active tunnel. The previous tunnel will be
    /// closed.
    #[cfg(target_os = "android")]
    fn create_tun(&mut self) -> Result<(), BoxedError>;

    /// Open a tunnel device using the previous or the default configuration if there is no
    /// currently active tunnel.
    #[cfg(target_os = "android")]
    fn create_tun_if_closed(&mut self) -> Result<(), BoxedError>;

    /// Close currently active tunnel device.
    #[cfg(target_os = "android")]
    fn close_tun(&mut self);
}

/// Configuration for creating a tunnel device.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TunConfig {
    /// IP addresses for the tunnel interface.
    pub addresses: Vec<IpAddr>,

    /// IP addresses for the DNS servers to use.
    pub dns_servers: Vec<IpAddr>,

    /// Routes to configure for the tunnel.
    pub routes: Vec<IpNetwork>,

    /// Maximum Transmission Unit in the tunnel.
    pub mtu: u16,
}
