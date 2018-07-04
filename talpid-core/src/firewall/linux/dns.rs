extern crate notify;
extern crate resolv_conf;

use std::net::IpAddr;
use std::ops::DerefMut;
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::{fs, io, thread};

use error_chain::ChainedError;

use self::notify::{RecommendedWatcher, RecursiveMode, Watcher};
use self::resolv_conf::{Config, ScopedIp};

const RESOLV_CONF_PATH: &str = "/etc/resolv.conf";
const RESOLV_CONF_BACKUP_PATH: &str = "/etc/resolv.conf.mullvadbackup";

error_chain! {
    errors {
        WatchResolvConf {
            description("Failed to watch /etc/resolv.conf for changes")
        }

        WriteResolvConf {
            description("Failed to write to /etc/resolv.conf")
        }

        BackupResolvConf {
            description("Failed to create backup of /etc/resolv.conf")
        }

        RestoreResolvConf {
            description("Failed to restore /etc/resolv.conf from backup")
        }
    }
}

pub struct DnsSettings {
    state: Arc<Mutex<Option<State>>>,
    _watcher: DnsWatcher,
}

impl DnsSettings {
    pub fn new() -> Result<Self> {
        restore_from_backup().chain_err(|| ErrorKind::RestoreResolvConf)?;

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
            None => {
                let backup = read_config().chain_err(|| ErrorKind::BackupResolvConf)?;
                write_backup(&backup).chain_err(|| ErrorKind::BackupResolvConf)?;

                State {
                    backup,
                    desired_dns: servers,
                }
            }
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
            write_config(&state.backup)?;
            let _ = fs::remove_file(RESOLV_CONF_BACKUP_PATH);
        }

        Ok(())
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

                write_backup(&state.backup).chain_err(|| "Failed to update /etc/resolv.conf backup")
            }
        } else {
            Ok(())
        }
    }
}

fn read_config() -> Result<Config> {
    let contents =
        fs::read_to_string(RESOLV_CONF_PATH).chain_err(|| "Failed to read /etc/resolv.conf")?;
    let config =
        Config::parse(&contents).chain_err(|| "Failed to parse contents of /etc/resolv.conf")?;

    Ok(config)
}

fn write_config(config: &Config) -> Result<()> {
    fs::write(RESOLV_CONF_PATH, config.to_string().as_bytes())
        .chain_err(|| ErrorKind::WriteResolvConf)
}

fn write_backup(backup: &Config) -> Result<()> {
    fs::write(RESOLV_CONF_BACKUP_PATH, backup.to_string().as_bytes())
        .chain_err(|| "Failed to write to /etc/resolv.conf backup file")
}

fn restore_from_backup() -> Result<()> {
    match fs::read_to_string(RESOLV_CONF_BACKUP_PATH) {
        Ok(backup) => {
            info!("Restoring DNS state from backup");
            let config = Config::parse(&backup)
                .chain_err(|| "Backup of /etc/resolv.conf could not be parsed")?;

            write_config(&config)?;

            fs::remove_file(RESOLV_CONF_BACKUP_PATH)
                .chain_err(|| "Failed to remove stale backup of /etc/resolv.conf")
        }
        Err(ref error) if error.kind() == io::ErrorKind::NotFound => {
            debug!("No DNS state backup to restore");
            Ok(())
        }
        Err(error) => Err(Error::with_chain(
            error,
            "Failed to read /etc/resolv.conf backup",
        )),
    }
}
