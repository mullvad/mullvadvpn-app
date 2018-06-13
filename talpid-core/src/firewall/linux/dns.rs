extern crate notify;
extern crate resolv_conf;

use std::fs;
use std::net::IpAddr;
use std::ops::DerefMut;
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread;

use error_chain::ChainedError;

use self::notify::{RecommendedWatcher, RecursiveMode, Watcher};
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

        WatchResolvConf {
            description("failed to watch /etc/resolv.conf for changes")
        }

        WriteResolvConf {
            description("failed to write to /etc/resolv.conf")
        }
    }
}

pub struct DnsSettings {
    state: Arc<Mutex<Option<State>>>,
    _watcher: DnsWatcher,
}

impl DnsSettings {
    pub fn new() -> Result<Self> {
        let state = Arc::new(Mutex::new(None));
        let watcher = DnsWatcher::start(state.clone())?;

        Ok(DnsSettings {
            state,
            _watcher: watcher,
        })
    }

    pub fn set_dns(&mut self, servers: Vec<IpAddr>) -> Result<()> {
        let mut state = self.lock_state();
        let new_state = match state.take() {
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

        *state = Some(new_state);

        write_config(&new_config)
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some(state) = self.lock_state().take() {
            write_config(&state.backup)
        } else {
            Ok(())
        }
    }

    fn lock_state(&self) -> MutexGuard<Option<State>> {
        self.state
            .lock()
            .expect("a thread panicked while using the DNS configuration state")
    }
}

struct State {
    backup: Config,
    desired_dns: Vec<IpAddr>,
}

impl State {
    fn desired_config(&self) -> Config {
        let mut config = self.backup.clone();

        config.nameservers = self
            .desired_dns
            .iter()
            .map(|&address| ScopedIp::from(address))
            .collect();

        config
    }
}

struct DnsWatcher {
    _watcher: RecommendedWatcher,
}

impl DnsWatcher {
    fn start(state: Arc<Mutex<Option<State>>>) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel();
        let mut watcher = notify::raw_watcher(event_tx).chain_err(|| ErrorKind::WatchResolvConf)?;

        watcher
            .watch(RESOLV_CONF_PATH, RecursiveMode::NonRecursive)
            .chain_err(|| ErrorKind::WatchResolvConf)?;

        thread::spawn(move || Self::event_loop(event_rx, state));

        Ok(DnsWatcher { _watcher: watcher })
    }

    fn event_loop(events: mpsc::Receiver<notify::RawEvent>, state: Arc<Mutex<Option<State>>>) {
        for _ in events {
            let locked_state = state
                .lock()
                .expect("a thread panicked while using the DNS configuration state");

            if let Err(error) = Self::update(locked_state) {
                let chained_error = error
                    .chain_err(|| "Failed to update DNS state after DNS settings have changed.");
                error!("{}", chained_error.display_chain());
            }
        }
    }

    fn update(mut locked_state: MutexGuard<Option<State>>) -> Result<()> {
        if let &mut Some(ref mut state) = locked_state.deref_mut() {
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

fn read_config() -> Result<Config> {
    let contents = fs::read_to_string(RESOLV_CONF_PATH).chain_err(|| ErrorKind::ReadResolvConf)?;
    let config = Config::parse(&contents).chain_err(|| ErrorKind::ParseResolvConf)?;

    Ok(config)
}

fn write_config(config: &Config) -> Result<()> {
    fs::write(RESOLV_CONF_PATH, config.to_string().as_bytes())
        .chain_err(|| ErrorKind::WriteResolvConf)
}
