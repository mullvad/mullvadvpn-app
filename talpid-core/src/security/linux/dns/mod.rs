extern crate resolv_conf;

mod network_manager;
mod resolvconf;
mod static_resolv_conf;
mod systemd_resolved;

use std::{env, fmt, net::IpAddr, path::Path};

use self::network_manager::NetworkManager;
use self::resolvconf::Resolvconf;
use self::static_resolv_conf::StaticResolvConf;
use self::systemd_resolved::SystemdResolved;

const RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

error_chain! {
    errors {
        NoDnsMonitor {
            description("No suitable DNS monitor implementation detected")
        }
    }

    links {
        Resolvconf(resolvconf::Error, resolvconf::ErrorKind);
        StaticResolvConf(static_resolv_conf::Error, static_resolv_conf::ErrorKind);
        SystemdResolved(systemd_resolved::Error, systemd_resolved::ErrorKind);
        NetworkManager(network_manager::Error, network_manager::ErrorKind);
    }
}

pub enum DnsMonitor {
    Resolvconf(Resolvconf),
    StaticResolvConf(StaticResolvConf),
    SystemdResolved(SystemdResolved),
    NetworkManager(NetworkManager),
}

impl fmt::Display for DnsMonitor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            DnsMonitor::Resolvconf(..) => "resolvconf",
            DnsMonitor::StaticResolvConf(..) => "/etc/resolv.conf",
            DnsMonitor::SystemdResolved(..) => "systemd-resolved",
            DnsMonitor::NetworkManager(..) => "network manager",
        };
        f.write_str(name)
    }
}

impl super::super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new(_cache_dir: impl AsRef<Path>) -> Result<Self> {
        DnsMonitor::new_internal()
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        self.reset()?;
        // Resetting the DNS manager in case the previously selected one isn't valid
        *self = DnsMonitor::new_internal()?;

        use self::DnsMonitor::*;
        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.set_dns(interface, servers)?,
            StaticResolvConf(ref mut static_resolv_conf) => {
                static_resolv_conf.set_dns(servers.to_vec())?
            }
            SystemdResolved(ref mut systemd_resolved) => {
                systemd_resolved.set_dns(interface, &servers)?
            }
            NetworkManager(ref mut network_manager) => network_manager.set_dns(&servers)?,
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        use self::DnsMonitor::*;

        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.reset()?,
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.reset()?,
            SystemdResolved(ref mut systemd_resolved) => systemd_resolved.reset()?,
            NetworkManager(ref mut network_manager) => network_manager.reset()?,
        }

        Ok(())
    }
}

impl DnsMonitor {
    fn new_internal() -> Result<Self> {
        let dns_module = env::var_os("TALPID_DNS_MODULE");

        let manager = match dns_module.as_ref().and_then(|value| value.to_str()) {
            Some("static-file") => DnsMonitor::StaticResolvConf(StaticResolvConf::new()?),
            Some("resolvconf") => DnsMonitor::Resolvconf(Resolvconf::new()?),
            Some("systemd") => DnsMonitor::SystemdResolved(SystemdResolved::new()?),
            Some("network-manager") => DnsMonitor::NetworkManager(NetworkManager::new()?),
            Some(_) | None => Self::with_detected_dns_manager()?,
        };
        log::debug!("Managing DNS via {}", manager);
        Ok(manager)
    }

    fn with_detected_dns_manager() -> Result<Self> {
        SystemdResolved::new()
            .map(DnsMonitor::SystemdResolved)
            .or_else(|_| NetworkManager::new().map(DnsMonitor::NetworkManager))
            .or_else(|_| Resolvconf::new().map(DnsMonitor::Resolvconf))
            .or_else(|_| StaticResolvConf::new().map(DnsMonitor::StaticResolvConf))
            .chain_err(|| ErrorKind::NoDnsMonitor)
    }
}
