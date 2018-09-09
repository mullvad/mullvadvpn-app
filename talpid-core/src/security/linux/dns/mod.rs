mod static_resolv_conf;

use std::net::IpAddr;

use self::static_resolv_conf::StaticResolvConf;

error_chain! {
    links {
        StaticResolvConf(static_resolv_conf::Error, static_resolv_conf::ErrorKind);
    }
}

pub enum DnsSettings {
    StaticResolvConf(StaticResolvConf),
}

impl DnsSettings {
    pub fn new() -> Result<Self> {
        Ok(DnsSettings::StaticResolvConf(StaticResolvConf::new()?))
    }

    pub fn set_dns(&mut self, servers: Vec<IpAddr>) -> Result<()> {
        use self::DnsSettings::*;

        match self {
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.set_dns(servers)?,
        }

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        use self::DnsSettings::*;

        match self {
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.reset()?,
        }

        Ok(())
    }
}
