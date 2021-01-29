use std::{net::IpAddr, path::Path};

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
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(DnsMonitor {
            inner: imp::DnsMonitor::new(cache_dir)?,
        })
    }

    /// Set DNS to the given servers. And start monitoring the system for changes.
    pub fn set(
        &mut self,
        interface: &str,
        gateways: &[IpAddr],
        servers: &[IpAddr],
    ) -> Result<(), Error> {
        log::info!(
            "Setting DNS servers to {}",
            servers
                .iter()
                .map(|ip| ip.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        self.inner.set(interface, gateways, servers)
    }

    /// Reset system DNS settings to what it was before being set by this instance.
    /// This succeeds if the interface does not exist.
    pub fn reset(&mut self) -> Result<(), Error> {
        log::info!("Resetting DNS");
        self.inner.reset()
    }
}

trait DnsMonitorT: Sized {
    type Error: std::error::Error;

    fn new(cache_dir: impl AsRef<Path>) -> Result<Self, Self::Error>;

    fn set(
        &mut self,
        interface: &str,
        gateways: &[IpAddr],
        servers: &[IpAddr],
    ) -> Result<(), Self::Error>;

    fn reset(&mut self) -> Result<(), Self::Error>;
}
