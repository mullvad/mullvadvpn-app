extern crate resolv_conf;

mod network_manager;
mod resolvconf;
mod static_resolv_conf;
mod systemd_resolved;

use std::{env, fmt, net::IpAddr};

use self::network_manager::NetworkManager;
use self::resolvconf::Resolvconf;
use self::static_resolv_conf::StaticResolvConf;
use self::systemd_resolved::SystemdResolved;

const RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

error_chain! {
    errors {
        NoDnsSettingsManager {
            description("No DNS settings manager detected")
        }
    }

    links {
        Resolvconf(resolvconf::Error, resolvconf::ErrorKind);
        StaticResolvConf(static_resolv_conf::Error, static_resolv_conf::ErrorKind);
        SystemdResolved(systemd_resolved::Error, systemd_resolved::ErrorKind);
        NetworkManager(network_manager::Error, network_manager::ErrorKind);
    }
}

pub enum DnsSettings {
    Resolvconf(Resolvconf),
    StaticResolvConf(StaticResolvConf),
    SystemdResolved(SystemdResolved),
    NetworkManager(NetworkManager),
}

impl fmt::Display for DnsSettings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            DnsSettings::Resolvconf(..) => "resolvconf",
            DnsSettings::StaticResolvConf(..) => "/etc/resolv.conf",
            DnsSettings::SystemdResolved(..) => "systemd-resolved",
            DnsSettings::NetworkManager(..) => "network manager",
        };
        f.write_str(name)
    }
}

impl DnsSettings {
    pub fn new() -> Result<Self> {
        let dns_module = env::var_os("TALPID_DNS_MODULE");

        let manager = match dns_module.as_ref().and_then(|value| value.to_str()) {
            Some("static-file") => DnsSettings::StaticResolvConf(StaticResolvConf::new()?),
            Some("resolvconf") => DnsSettings::Resolvconf(Resolvconf::new()?),
            Some("systemd") => DnsSettings::SystemdResolved(SystemdResolved::new()?),
            Some("network-manager") => DnsSettings::NetworkManager(NetworkManager::new()?),
            Some(_) | None => Self::with_detected_dns_manager()?,
        };
        debug!("Managing DNS via {}", manager);
        Ok(manager)
    }

    fn with_detected_dns_manager() -> Result<Self> {
        SystemdResolved::new()
            .map(DnsSettings::SystemdResolved)
            .or_else(|_| NetworkManager::new().map(DnsSettings::NetworkManager))
            .or_else(|_| Resolvconf::new().map(DnsSettings::Resolvconf))
            .or_else(|_| StaticResolvConf::new().map(DnsSettings::StaticResolvConf))
            .chain_err(|| ErrorKind::NoDnsSettingsManager)
    }

    pub fn set_dns(&mut self, interface: &str, servers: Vec<IpAddr>) -> Result<()> {
        use self::DnsSettings::*;

        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.set_dns(interface, servers)?,
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.set_dns(servers)?,
            SystemdResolved(ref mut systemd_resolved) => {
                systemd_resolved.set_dns(interface, &servers)?
            }
            NetworkManager(ref mut network_manager) => network_manager.set_dns(&servers)?,
        }

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        use self::DnsSettings::*;

        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.reset()?,
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.reset()?,
            SystemdResolved(ref mut systemd_resolved) => systemd_resolved.reset()?,
            NetworkManager(ref mut network_manager) => network_manager.reset()?,
        }

        Ok(())
    }
}
