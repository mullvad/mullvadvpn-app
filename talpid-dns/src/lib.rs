//! Abstractions over operating system DNS settings.

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
#[derive(Debug, Clone, PartialEq)]
pub struct DnsConfig {
    config: InnerDnsConfig,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            config: InnerDnsConfig::Default,
        }
    }
}

impl DnsConfig {
    /// Use the specified addresses for DNS resolution
    pub fn from_addresses(tunnel_config: &[IpAddr], non_tunnel_config: &[IpAddr]) -> Self {
        DnsConfig {
            config: InnerDnsConfig::Override {
                tunnel_config: tunnel_config.to_owned(),
                non_tunnel_config: non_tunnel_config.to_owned(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum InnerDnsConfig {
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
    pub fn resolve(
        &self,
        default_tun_config: &[IpAddr],
        #[cfg(target_os = "macos")] port: u16,
    ) -> ResolvedDnsConfig {
        match &self.config {
            InnerDnsConfig::Default => ResolvedDnsConfig {
                tunnel_config: default_tun_config.to_owned(),
                non_tunnel_config: vec![],
                #[cfg(target_os = "macos")]
                port,
            },
            InnerDnsConfig::Override {
                tunnel_config,
                non_tunnel_config,
            } => ResolvedDnsConfig {
                tunnel_config: tunnel_config.to_owned(),
                non_tunnel_config: non_tunnel_config.to_owned(),
                #[cfg(target_os = "macos")]
                port,
            },
        }
    }
}

/// DNS configuration with `DnsConfig::Default` resolved
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDnsConfig {
    /// Addresses to configure on the tunnel interface
    tunnel_config: Vec<IpAddr>,
    /// Addresses to allow on non-tunnel interface.
    /// For the most part, the tunnel state machine will not handle any of this configuration
    /// on non-tunnel interface, only allow them in the firewall.
    non_tunnel_config: Vec<IpAddr>,
    /// Port to use
    #[cfg(target_os = "macos")]
    port: u16,
}

impl fmt::Display for ResolvedDnsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Tunnel DNS: ")?;
        Self::fmt_addr_set(f, &self.tunnel_config)?;

        f.write_str(" Non-tunnel DNS: ")?;
        Self::fmt_addr_set(f, &self.non_tunnel_config)?;

        #[cfg(target_os = "macos")]
        write!(f, " Port: {}", self.port)?;

        Ok(())
    }
}

impl ResolvedDnsConfig {
    fn fmt_addr_set(f: &mut fmt::Formatter<'_>, addrs: &[IpAddr]) -> fmt::Result {
        f.write_str("{")?;
        for (i, addr) in addrs.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{addr}")?;
        }
        f.write_str("}")
    }

    /// Addresses to configure on the tunnel interface
    pub fn tunnel_config(&self) -> &[IpAddr] {
        &self.tunnel_config
    }

    /// Addresses to allow on non-tunnel interface.
    /// For the most part, the tunnel state machine will not handle any of this configuration
    /// on non-tunnel interface, only allow them in the firewall.
    pub fn non_tunnel_config(&self) -> &[IpAddr] {
        &self.non_tunnel_config
    }

    /// Consume `self` and return a vector of all addresses
    pub fn addresses(self) -> impl Iterator<Item = IpAddr> {
        self.non_tunnel_config.into_iter().chain(self.tunnel_config)
    }

    /// Return whether the config contains only (and at least one) loopback addresses, and zero
    /// non-loopback addresses
    pub fn is_loopback(&self) -> bool {
        let (loopback_addrs, non_loopback_addrs) = self
            .tunnel_config
            .iter()
            .chain(self.non_tunnel_config.iter())
            .copied()
            .partition::<Vec<_>, _>(|ip| ip.is_loopback());

        !loopback_addrs.is_empty() && non_loopback_addrs.is_empty()
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
