mod static_resolv_conf;
mod systemd_resolved;

use std::net::IpAddr;

use self::static_resolv_conf::StaticResolvConf;
use self::systemd_resolved::SystemdResolved;

error_chain! {
    errors {
        NoDnsSettingsManager {
            description("No DNS settings manager detected")
        }
    }

    links {
        StaticResolvConf(static_resolv_conf::Error, static_resolv_conf::ErrorKind);
        SystemdResolved(systemd_resolved::Error, systemd_resolved::ErrorKind);
    }
}

pub enum DnsSettings {
    StaticResolvConf(StaticResolvConf),
    SystemdResolved(SystemdResolved),
}

impl DnsSettings {
    pub fn new() -> Result<Self> {
        SystemdResolved::new()
            .map(DnsSettings::SystemdResolved)
            .or_else(|_| StaticResolvConf::new().map(DnsSettings::StaticResolvConf))
            .chain_err(|| ErrorKind::NoDnsSettingsManager)
    }

    pub fn set_dns(&mut self, interface: &str, servers: Vec<IpAddr>) -> Result<()> {
        use self::DnsSettings::*;

        match self {
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.set_dns(servers)?,
            SystemdResolved(ref mut systemd_resolved) => {
                systemd_resolved.set_dns(interface, servers)?
            }
        }

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        use self::DnsSettings::*;

        match self {
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.reset()?,
            SystemdResolved(ref mut systemd_resolved) => systemd_resolved.reset()?,
        }

        Ok(())
    }
}
