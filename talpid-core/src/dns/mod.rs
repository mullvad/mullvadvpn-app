use std::net::IpAddr;
#[cfg(target_os = "linux")]
use talpid_routing::RouteManagerHandle;

#[cfg(target_os = "macos")]
use {
    crate::tunnel_state_machine::TunnelCommand, futures::channel::mpsc::UnboundedSender,
    std::sync::Weak,
};

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod imp;

#[cfg(target_os = "linux")]
pub use imp::will_use_nm;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

pub use self::imp::Error;

/// Sets and monitors system DNS settings. Makes sure the desired DNS servers are being used.
pub struct DnsMonitor {
    inner: imp::DnsMonitor,
}

impl DnsMonitor {
    /// Returns a new `DnsMonitor` that can set and monitor the system DNS.
    pub fn new(
        #[cfg(target_os = "linux")] handle: tokio::runtime::Handle,
        #[cfg(target_os = "linux")] route_manager: RouteManagerHandle,
        #[cfg(target_os = "macos")] tx: Weak<UnboundedSender<TunnelCommand>>,
    ) -> Result<Self, Error> {
        Ok(DnsMonitor {
            inner: imp::DnsMonitor::new(
                #[cfg(target_os = "linux")]
                handle,
                #[cfg(target_os = "linux")]
                route_manager,
                #[cfg(target_os = "macos")]
                tx,
            )?,
        })
    }

    /// Returns a map of interfaces and respective list of resolvers that don't contain our
    /// changes.
    #[cfg(target_os = "macos")]
    pub fn get_system_config(&self) -> Result<Option<(String, Vec<IpAddr>)>, Error> {
        self.inner.get_system_config()
    }

    /// Set DNS to the given servers. And start monitoring the system for changes.
    pub fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), Error> {
        log::info!(
            "Setting DNS servers to {}",
            servers
                .iter()
                .map(|ip| ip.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        self.inner.set(interface, servers)
    }

    /// Reset system DNS settings to what it was before being set by this instance.
    /// This succeeds if the interface does not exist.
    pub fn reset(&mut self) -> Result<(), Error> {
        log::info!("Resetting DNS");
        self.inner.reset()
    }

    /// Reset DNS settings to what they were before being set by this instance.
    /// If the settings only affect a specific interface, this can be a no-op,
    /// as the interface will be destroyed.
    pub fn reset_before_interface_removal(&mut self) -> Result<(), Error> {
        log::info!("Resetting DNS");
        self.inner.reset_before_interface_removal()
    }
}

trait DnsMonitorT: Sized {
    type Error: std::error::Error;

    fn new(
        #[cfg(target_os = "linux")] handle: tokio::runtime::Handle,
        #[cfg(target_os = "linux")] route_manager: RouteManagerHandle,
        #[cfg(target_os = "macos")] tx: Weak<UnboundedSender<TunnelCommand>>,
    ) -> Result<Self, Self::Error>;

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), Self::Error>;

    fn reset(&mut self) -> Result<(), Self::Error>;

    fn reset_before_interface_removal(&mut self) -> Result<(), Self::Error> {
        self.reset()
    }
}
