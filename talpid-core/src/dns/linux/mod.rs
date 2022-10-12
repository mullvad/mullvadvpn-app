mod network_manager;
mod resolvconf;
mod static_resolv_conf;
pub(self) mod systemd_resolved;

use self::{
    network_manager::NetworkManager, resolvconf::Resolvconf, static_resolv_conf::StaticResolvConf,
    systemd_resolved::SystemdResolved,
};
use std::{env, fmt, net::IpAddr};
use talpid_routing::RouteManagerHandle;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Linux DNS monitor
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Error in systemd-resolved DNS monitor
    #[error(display = "Error in systemd-resolved DNS monitor")]
    SystemdResolved(#[error(source)] systemd_resolved::Error),

    /// Error in NetworkManager DNS monitor
    #[error(display = "Error in NetworkManager DNS monitor")]
    NetworkManager(#[error(source)] network_manager::Error),

    /// Error in resolvconf DNS monitor
    #[error(display = "Error in resolvconf DNS monitor")]
    Resolvconf(#[error(source)] resolvconf::Error),

    /// Error in static /etc/resolv.conf DNS monitor
    #[error(display = "Error in static /etc/resolv.conf DNS monitor")]
    StaticResolvConf(#[error(source)] static_resolv_conf::Error),

    /// No suitable DNS monitor implementation detected
    #[error(display = "No suitable DNS monitor implementation detected")]
    NoDnsMonitor,
}

pub struct DnsMonitor {
    route_manager: RouteManagerHandle,
    handle: tokio::runtime::Handle,
    inner: Option<DnsMonitorHolder>,
}

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new(handle: tokio::runtime::Handle, route_manager: RouteManagerHandle) -> Result<Self> {
        Ok(DnsMonitor {
            route_manager,
            handle,
            inner: None,
        })
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        self.reset()?;
        // Creating a new DNS monitor for each set, in case the system changed how it manages DNS.
        let mut inner = DnsMonitorHolder::new(&self.handle)?;
        if !servers.is_empty() {
            inner.set(&self.handle, &self.route_manager, interface, servers)?;
            self.inner = Some(inner);
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        if let Some(mut inner) = self.inner.take() {
            inner.reset(&self.handle)?;
        }
        Ok(())
    }
}

pub enum DnsMonitorHolder {
    SystemdResolved(SystemdResolved),
    NetworkManager(NetworkManager),
    Resolvconf(Resolvconf),
    StaticResolvConf(StaticResolvConf),
}

impl fmt::Display for DnsMonitorHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::DnsMonitorHolder::*;
        let name = match self {
            Resolvconf(..) => "resolvconf",
            StaticResolvConf(..) => "/etc/resolv.conf",
            SystemdResolved(..) => "systemd-resolved",
            NetworkManager(..) => "network manager",
        };
        f.write_str(name)
    }
}

impl DnsMonitorHolder {
    fn new(handle: &tokio::runtime::Handle) -> Result<Self> {
        let dns_module = env::var_os("TALPID_DNS_MODULE");

        let manager = match dns_module.as_ref().and_then(|value| value.to_str()) {
            Some("static-file") => {
                DnsMonitorHolder::StaticResolvConf(handle.block_on(StaticResolvConf::new())?)
            }
            Some("resolvconf") => DnsMonitorHolder::Resolvconf(Resolvconf::new()?),
            Some("systemd") => DnsMonitorHolder::SystemdResolved(SystemdResolved::new()?),
            Some("network-manager") => DnsMonitorHolder::NetworkManager(NetworkManager::new()?),
            Some(_) | None => Self::with_detected_dns_manager(handle)?,
        };
        log::debug!("Managing DNS via {}", manager);
        Ok(manager)
    }

    fn with_detected_dns_manager(handle: &tokio::runtime::Handle) -> Result<Self> {
        SystemdResolved::new()
            .map(DnsMonitorHolder::SystemdResolved)
            .or_else(|err| {
                match err {
                    systemd_resolved::Error::SystemdResolvedError(
                        systemd_resolved::SystemdDbusError::NoSystemdResolved(_),
                    ) => (),
                    other_error => {
                        log::debug!("NetworkManager is being used because {}", other_error)
                    }
                }
                NetworkManager::new().map(DnsMonitorHolder::NetworkManager)
            })
            .or_else(|_| Resolvconf::new().map(DnsMonitorHolder::Resolvconf))
            .or_else(|_| {
                handle
                    .block_on(StaticResolvConf::new())
                    .map(DnsMonitorHolder::StaticResolvConf)
            })
            .map_err(|_| Error::NoDnsMonitor)
    }

    fn set(
        &mut self,
        handle: &tokio::runtime::Handle,
        route_manager: &RouteManagerHandle,
        interface: &str,
        servers: &[IpAddr],
    ) -> Result<()> {
        use self::DnsMonitorHolder::*;
        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.set_dns(interface, servers)?,
            StaticResolvConf(ref mut static_resolv_conf) => {
                static_resolv_conf.set_dns(servers.to_vec())?
            }
            SystemdResolved(ref mut systemd_resolved) => handle
                .block_on(systemd_resolved.set_dns(route_manager.clone(), interface, servers))?,
            NetworkManager(ref mut network_manager) => {
                network_manager.set_dns(interface, servers)?
            }
        }
        Ok(())
    }

    fn reset(&mut self, handle: &tokio::runtime::Handle) -> Result<()> {
        use self::DnsMonitorHolder::*;
        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.reset()?,
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.reset()?,
            SystemdResolved(ref mut systemd_resolved) => {
                handle.block_on(systemd_resolved.reset())?
            }
            NetworkManager(ref mut network_manager) => network_manager.reset()?,
        }
        Ok(())
    }
}

/// Returns true if DnsMonitor will use NetworkManager to manage DNS.
pub fn will_use_nm() -> bool {
    crate::dns::imp::SystemdResolved::new().is_err()
        && crate::dns::imp::NetworkManager::new().is_ok()
}
