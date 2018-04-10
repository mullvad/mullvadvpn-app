extern crate resolv_conf;

use std::fs::File;
use std::io::{self, Read, Write};
use std::net::IpAddr;

use self::resolv_conf::{Config, ScopedIp};

error_chain!{
    errors {
        ParseResolvConf {
            description("failed to parse contents of /etc/resolv.conf")
        }

        ReadResolvConf {
            description("failed to read /etc/resolv.conf")
        }

        WriteResolvConf {
            description("failed to write to /etc/resolv.conf")
        }
    }
}

pub struct DnsSettings {
    backup: Option<String>,
    desired_dns: Option<Vec<IpAddr>>,
}

impl DnsSettings {
    pub fn new() -> Result<Self> {
        Ok(DnsSettings {
            backup: None,
            desired_dns: None,
        })
    }

    pub fn set_dns(&mut self, servers: Vec<IpAddr>) -> Result<()> {
        if self.backup.is_none() {
            self.backup = Some(Self::read_resolv_conf().chain_err(|| ErrorKind::ReadResolvConf)?);
        }

        self.desired_dns = Some(servers);
        self.configure_dns()?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.desired_dns = None;

        if let Some(backup) = self.backup.take() {
            Self::write_resolv_conf(&backup).chain_err(|| ErrorKind::WriteResolvConf)?;
        }

        Ok(())
    }

    fn configure_dns(&self) -> Result<()> {
        let mut config = match self.backup {
            Some(ref previous_config) => {
                Config::parse(previous_config).chain_err(|| ErrorKind::ParseResolvConf)?
            }
            None => Config::new(),
        };

        if let Some(ref nameservers) = self.desired_dns {
            config.nameservers = nameservers
                .iter()
                .map(|&address| ScopedIp::from(address))
                .collect();
        } else {
            config.nameservers.clear();
        }

        Self::write_resolv_conf(&config.to_string()).chain_err(|| ErrorKind::WriteResolvConf)
    }

    fn read_resolv_conf() -> io::Result<String> {
        let mut file = File::open("/etc/resolv.conf")?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(contents)
    }

    fn write_resolv_conf(contents: &str) -> io::Result<()> {
        let mut file = File::create("/etc/resolv.conf")?;

        file.write_all(contents.as_bytes())
    }
}
