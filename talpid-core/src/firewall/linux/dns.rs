extern crate resolv_conf;

use std::fs::File;
use std::io::{self, Read, Write};
use std::net::IpAddr;

use self::resolv_conf::{Config, ScopedIp};

static RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

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
    state: Option<State>,
}

impl DnsSettings {
    pub fn new() -> Result<Self> {
        Ok(DnsSettings { state: None })
    }

    pub fn set_dns(&mut self, servers: Vec<IpAddr>) -> Result<()> {
        let new_state = match self.state.take() {
            None => State {
                backup: read_config()?,
                desired_dns: servers,
            },
            Some(previous_state) => State {
                backup: previous_state.backup,
                desired_dns: servers,
            },
        };

        let new_config = new_state.desired_config();

        self.state = Some(new_state);

        write_config(&new_config)
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some(state) = self.state.take() {
            write_config(&state.backup)
        } else {
            Ok(())
        }
    }
}

struct State {
    backup: Config,
    desired_dns: Vec<IpAddr>,
}

impl State {
    fn desired_config(&self) -> Config {
        let mut config = self.backup.clone();

        config.nameservers = self.desired_dns
            .iter()
            .map(|&address| ScopedIp::from(address))
            .collect();

        config
    }
}

fn read_config() -> Result<Config> {
    let contents = read_resolv_conf().chain_err(|| ErrorKind::ReadResolvConf)?;
    let config = Config::parse(&contents).chain_err(|| ErrorKind::ParseResolvConf)?;

    Ok(config)
}

fn read_resolv_conf() -> io::Result<String> {
    let mut file = File::open(RESOLV_CONF_PATH)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn write_config(config: &Config) -> Result<()> {
    write_resolv_conf(&config.to_string()).chain_err(|| ErrorKind::WriteResolvConf)
}

fn write_resolv_conf(contents: &str) -> io::Result<()> {
    let mut file = File::create(RESOLV_CONF_PATH)?;

    file.write_all(contents.as_bytes())
}
