extern crate notify;
extern crate resolv_conf;

use std::fs::File;
use std::io::{self, Read, Write};
use std::net::IpAddr;
use std::sync::mpsc;
use std::thread;

use error_chain::ChainedError;

use self::notify::{RecommendedWatcher, RecursiveMode, Watcher};
use self::resolv_conf::{Config, ScopedIp};

static RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

error_chain!{
    errors {
        ConfiguratorStopped {
            description("DNS configurator thread has unexpectedly stopped")
        }

        ParseResolvConf {
            description("failed to parse contents of /etc/resolv.conf")
        }

        ReadResolvConf {
            description("failed to read /etc/resolv.conf")
        }

        WatchResolvConf {
            description("failed to watch /etc/resolv.conf for changes")
        }

        WriteResolvConf {
            description("failed to write to /etc/resolv.conf")
        }
    }
}

pub struct DnsSettings {
    configurator: mpsc::Sender<DnsEvent>,
    _watcher: DnsWatcher,
}

impl DnsSettings {
    pub fn new() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel();
        let watcher = DnsWatcher::start(event_tx.clone())?;

        Self::spawn_configurator_thread(event_rx)?;

        Ok(DnsSettings {
            configurator: event_tx,
            _watcher: watcher,
        })
    }

    pub fn set_dns(&mut self, servers: Vec<IpAddr>) -> Result<()> {
        let (result_tx, result_rx) = mpsc::channel();

        self.send_to_configurator(DnsEvent::Set(servers, result_tx))?;

        Self::receive_from_configurator(result_rx)
    }

    pub fn reset(&mut self) -> Result<()> {
        let (result_tx, result_rx) = mpsc::channel();

        self.send_to_configurator(DnsEvent::Reset(result_tx))?;

        Self::receive_from_configurator(result_rx)
    }

    fn spawn_configurator_thread(events: mpsc::Receiver<DnsEvent>) -> Result<()> {
        let (result_tx, result_rx) = mpsc::channel();

        thread::spawn(move || match DnsConfigurator::new() {
            Ok(configurator) => {
                if result_tx.send(Ok(())).is_ok() {
                    Self::run_configurator_thread(events, configurator);
                }
            }
            Err(error) => {
                let _ = result_tx.send(Err(error));
            }
        });

        Self::receive_from_configurator(result_rx)
    }

    fn run_configurator_thread(
        events: mpsc::Receiver<DnsEvent>,
        mut configurator: DnsConfigurator,
    ) {
        for event in events {
            match event {
                DnsEvent::Set(servers, reply) => {
                    let _ = reply.send(configurator.set_dns(servers));
                }
                DnsEvent::Reset(reply) => {
                    let _ = reply.send(configurator.reset());
                }
                DnsEvent::Update => {
                    if let Err(error) = configurator.update() {
                        error!(
                            "Failed to notify DNS configurator that DNS settings have changed: {}",
                            error.display_chain()
                        );
                    }
                }
            };
        }
    }

    fn send_to_configurator(&mut self, event: DnsEvent) -> Result<()> {
        self.configurator
            .send(event)
            .chain_err(|| ErrorKind::ConfiguratorStopped)
    }

    fn receive_from_configurator(result: mpsc::Receiver<Result<()>>) -> Result<()> {
        result
            .recv()
            .chain_err(|| ErrorKind::ConfiguratorStopped)
            .and_then(|result| result)
    }
}

enum DnsEvent {
    Set(Vec<IpAddr>, mpsc::Sender<Result<()>>),
    Reset(mpsc::Sender<Result<()>>),
    Update,
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

struct DnsConfigurator {
    state: Option<State>,
}

impl DnsConfigurator {
    fn new() -> Result<Self> {
        Ok(DnsConfigurator { state: None })
    }

    fn set_dns(&mut self, servers: Vec<IpAddr>) -> Result<()> {
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

    fn reset(&mut self) -> Result<()> {
        if let Some(state) = self.state.take() {
            write_config(&state.backup)
        } else {
            Ok(())
        }
    }

    fn update(&mut self) -> Result<()> {
        if let Some(ref mut state) = self.state {
            let mut new_config = read_config()?;
            let desired_nameservers = state
                .desired_dns
                .iter()
                .map(|&address| ScopedIp::from(address))
                .collect();

            if new_config.nameservers != desired_nameservers {
                state.backup = new_config.clone();
                new_config.nameservers = desired_nameservers;

                write_config(&new_config)
            } else {
                new_config.nameservers.clear();
                new_config.nameservers.append(&mut state.backup.nameservers);
                state.backup = new_config;

                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

struct DnsWatcher {
    _watcher: RecommendedWatcher,
}

impl DnsWatcher {
    fn start(notification_sink: mpsc::Sender<DnsEvent>) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel();
        let mut watcher = notify::raw_watcher(event_tx).chain_err(|| ErrorKind::WatchResolvConf)?;

        watcher
            .watch(RESOLV_CONF_PATH, RecursiveMode::NonRecursive)
            .chain_err(|| ErrorKind::WatchResolvConf)?;

        thread::spawn(move || {
            for _ in event_rx {
                if notification_sink.send(DnsEvent::Update).is_err() {
                    break;
                }
            }
        });

        Ok(DnsWatcher { _watcher: watcher })
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
