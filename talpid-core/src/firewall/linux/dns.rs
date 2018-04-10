extern crate resolv_conf;

use std::fs::File;
use std::io::{self, Read, Write};

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

pub struct DnsMonitor {
    backup: Option<String>,
    state: Option<Vec<String>>,
}

impl DnsMonitor {
    pub fn new() -> Result<Self> {
        Ok(DnsMonitor {
            backup: None,
            state: None,
        })
    }

    pub fn set_dns(&mut self, servers: Vec<String>) -> Result<()> {
        if self.backup.is_none() {
            self.backup = Some(Self::read_resolv_conf().chain_err(|| ErrorKind::ReadResolvConf)?);
        }

        self.state = Some(servers);
        self.write_state()?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.state = None;

        if let Some(backup) = self.backup.take() {
            Self::write_resolv_conf(&backup).chain_err(|| ErrorKind::WriteResolvConf)?;
        }

        Ok(())
    }

    fn write_state(&self) -> Result<()> {
        let mut config = match self.backup {
            Some(ref previous_config) => {
                Config::parse(previous_config).chain_err(|| ErrorKind::ParseResolvConf)?
            }
            None => Config::new(),
        };

        if let Some(ref nameservers) = self.state {
            config.nameservers = nameservers
                .iter()
                .filter_map(
                    |nameserver_string| match nameserver_string.parse::<ScopedIp>() {
                        Ok(address) => Some(address),
                        Err(_) => {
                            error!("Invalid IP address for DNS: {}", nameserver_string);
                            None
                        }
                    },
                )
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
