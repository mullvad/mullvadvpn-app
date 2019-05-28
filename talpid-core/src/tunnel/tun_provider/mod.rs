use cfg_if::cfg_if;
use std::net::IpAddr;
#[cfg(unix)]
use std::os::unix::io::AsRawFd;
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
        pub use self::stub::StubTunProvider;

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
}

/// Stub tunnel device.
#[cfg(windows)]
pub trait Tun: Send {
    /// Retrieve the tunnel interface name.
    fn interface_name(&self) -> &str;
}

/// Factory of tunnel devices.
pub trait TunProvider: Send + 'static {
    /// Create a tunnel device using the provided configuration.
    fn create_tun(&self, config: TunConfig) -> Result<Box<dyn Tun>, BoxedError>;
}

/// Configuration for creating a tunnel device.
pub struct TunConfig {
    /// IP addresses for the tunnel interface.
    pub addresses: Vec<IpAddr>,
}
