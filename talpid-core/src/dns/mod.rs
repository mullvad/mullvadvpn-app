use std::fmt;
use std::net::IpAddr;

#[cfg(target_os = "linux")]
use talpid_routing::RouteManagerHandle;

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

/// DNS configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnsConfig {
    /// Use gateway addresses from the tunnel config
    Default,
    /// Use the specified addresses for DNS resolution
    Override {
        /// Addresses to configure on the tunnel interface
        tunnel_config: Vec<IpAddr>,
        /// Addresses to allow on non-tunnel interface.
        /// For the most part, the tunnel state machine will not handle any of this configuration
        /// on non-tunnel interface, only allow them in the firewall.
        non_tunnel_config: Vec<IpAddr>,
    },
}

impl DnsConfig {
    pub(crate) fn resolve(&self, gateways: &[IpAddr]) -> ResolvedDnsConfig {
        match self {
            DnsConfig::Default => ResolvedDnsConfig {
                tunnel_config: gateways.to_owned(),
                non_tunnel_config: vec![],
            },
            DnsConfig::Override {
                tunnel_config,
                non_tunnel_config,
            } => ResolvedDnsConfig {
                tunnel_config: tunnel_config.to_owned(),
                non_tunnel_config: non_tunnel_config.to_owned(),
            },
        }
    }
}

/// DNS configuration with `DnsConfig::Default` resolved
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDnsConfig {
    /// Addresses to configure on the tunnel interface
    pub tunnel_config: Vec<IpAddr>,
    /// Addresses to allow on non-tunnel interface.
    /// For the most part, the tunnel state machine will not handle any of this configuration
    /// on non-tunnel interface, only allow them in the firewall.
    pub non_tunnel_config: Vec<IpAddr>,
}

impl fmt::Display for ResolvedDnsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Tunnel DNS: ")?;
        Self::fmt_addr_set(f, &self.tunnel_config)?;

        f.write_str(" Non-tunnel DNS: ")?;
        Self::fmt_addr_set(f, &self.non_tunnel_config)
    }
}

impl ResolvedDnsConfig {
    fn fmt_addr_set(f: &mut fmt::Formatter<'_>, addrs: &[IpAddr]) -> fmt::Result {
        f.write_str("{")?;
        for (i, addr) in addrs.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", addr)?;
        }
        f.write_str("}")
    }

    /// Consume `self` and return a vector of all addresses
    pub fn addresses(self) -> Vec<IpAddr> {
        let mut v = self.tunnel_config;
        v.extend(self.non_tunnel_config);
        v
    }
}

/// Sets and monitors system DNS settings. Makes sure the desired DNS servers are being used.
pub struct DnsMonitor {
    inner: imp::DnsMonitor,
}

impl DnsMonitor {
    /// Returns a new `DnsMonitor` that can set and monitor the system DNS.
    pub fn new(
        #[cfg(target_os = "linux")] handle: tokio::runtime::Handle,
        #[cfg(target_os = "linux")] route_manager: RouteManagerHandle,
    ) -> Result<Self, Error> {
        Ok(DnsMonitor {
            inner: imp::DnsMonitor::new(
                #[cfg(target_os = "linux")]
                handle,
                #[cfg(target_os = "linux")]
                route_manager,
            )?,
        })
    }

    /// Set DNS to the given servers. And start monitoring the system for changes.
    pub fn set(&mut self, interface: &str, config: ResolvedDnsConfig) -> Result<(), Error> {
        log::info!("Setting DNS servers: {config}",);
        self.inner.set(interface, config)
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
    ) -> Result<Self, Self::Error>;

    fn set(&mut self, interface: &str, servers: ResolvedDnsConfig) -> Result<(), Self::Error>;

    fn reset(&mut self) -> Result<(), Self::Error>;

    fn reset_before_interface_removal(&mut self) -> Result<(), Self::Error> {
        self.reset()
    }
}
