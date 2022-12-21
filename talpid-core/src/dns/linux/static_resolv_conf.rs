use futures::StreamExt;
use inotify::{Inotify, WatchMask};
use parking_lot::Mutex;
use resolv_conf::{Config, ScopedIp};
use std::{fs, io, net::IpAddr, sync::Arc};
use talpid_types::ErrorExt;
use triggered::{trigger, Listener, Trigger};

const RESOLV_CONF_BACKUP_PATH: &str = "/etc/resolv.conf.mullvadbackup";
const RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to watch /etc/resolv.conf for changes")]
    WatchResolvConf(#[error(source)] std::io::Error),

    #[error(display = "Failed to write to {}", _0)]
    WriteResolvConf(&'static str, #[error(source)] io::Error),

    #[error(display = "Failed to read from {}", _0)]
    ReadResolvConf(&'static str, #[error(source)] io::Error),

    #[error(display = "resolv.conf at {} could not be parsed", _0)]
    Parse(&'static str, #[error(source)] resolv_conf::ParseError),

    #[error(display = "Failed to remove stale resolv.conf backup at {}", _0)]
    RemoveBackup(&'static str, #[error(source)] io::Error),
}

pub struct StaticResolvConf {
    state: Arc<Mutex<Option<State>>>,
    _watcher: DnsWatcher,
}

impl StaticResolvConf {
    pub async fn new() -> Result<Self> {
        restore_from_backup()?;

        let state = Arc::new(Mutex::new(None));
        let watcher = DnsWatcher::start(state.clone()).await?;

        Ok(StaticResolvConf {
            state,
            _watcher: watcher,
        })
    }

    pub fn set_dns(&mut self, servers: Vec<IpAddr>) -> Result<()> {
        let mut state = self.state.lock();
        let new_state = match state.take() {
            None => {
                let backup = read_config()?;
                write_backup(&backup)?;

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
        if let Some(state) = self.state.lock().take() {
            write_config(&state.backup)?;
            let _ = fs::remove_file(RESOLV_CONF_BACKUP_PATH);
        }

        Ok(())
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
    cancel_trigger: Trigger,
}

impl Drop for DnsWatcher {
    fn drop(&mut self) {
        self.cancel_trigger.trigger();
    }
}

impl DnsWatcher {
    async fn start(state: Arc<Mutex<Option<State>>>) -> Result<Self> {
        let mut watcher = Inotify::init().map_err(Error::WatchResolvConf)?;
        let mut mask = WatchMask::empty();
        // Documentation for the meaning of these masks can be found in `man inotify`
        //
        // We do not watch for writes but instead for when a file opened for writing is closed.
        // This way we don't have collisions.
        mask.insert(WatchMask::CLOSE_WRITE);
        // DELETE_SELF is generated if the file watched is itself deleted
        mask.insert(WatchMask::DELETE_SELF);
        mask.insert(WatchMask::MOVE_SELF);

        watcher
            .add_watch(RESOLV_CONF_PATH, mask)
            .map_err(Error::WatchResolvConf)?;

        let (cancel_trigger, cancel_listener) = trigger();

        tokio::spawn(async move { Self::event_loop(watcher, cancel_listener, &state).await });

        Ok(DnsWatcher { cancel_trigger })
    }

    async fn event_loop(
        mut watcher: Inotify,
        mut cancel_listener: Listener,
        state: &Arc<Mutex<Option<State>>>,
    ) {
        const EVENT_BUFFER_SIZE: usize = 1024;
        let mut buffer = [0; EVENT_BUFFER_SIZE];
        let mut events = watcher
            .event_stream(&mut buffer)
            .expect("Could not read events for resolv.conf");

        loop {
            tokio::select! {
                _ = &mut cancel_listener => {
                    break;
                },
                Some(_) = events.next() => {
                    let mut locked_state = state.lock();
                    if let Err(error) = Self::update(locked_state.as_mut()) {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg(
                                "Failed to update DNS state after DNS settings changed"
                            )
                        );
                    }
                }
            }
        }
    }

    fn update(state: Option<&mut State>) -> Result<()> {
        if let Some(state) = state {
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

                write_backup(&state.backup)
            }
        } else {
            Ok(())
        }
    }
}

fn read_config() -> Result<Config> {
    if !std::path::Path::new(RESOLV_CONF_PATH).exists() {
        return Ok(Config::new());
    }

    let contents = fs::read_to_string(RESOLV_CONF_PATH)
        .map_err(|e| Error::ReadResolvConf(RESOLV_CONF_PATH, e))?;
    let config = Config::parse(contents).map_err(|e| Error::Parse(RESOLV_CONF_PATH, e))?;

    Ok(config)
}

fn write_config(config: &Config) -> Result<()> {
    fs::write(RESOLV_CONF_PATH, config.to_string().as_bytes())
        .map_err(|e| Error::WriteResolvConf(RESOLV_CONF_PATH, e))
}

fn write_backup(backup: &Config) -> Result<()> {
    fs::write(RESOLV_CONF_BACKUP_PATH, backup.to_string().as_bytes())
        .map_err(|e| Error::WriteResolvConf(RESOLV_CONF_BACKUP_PATH, e))
}

fn restore_from_backup() -> Result<()> {
    match fs::read_to_string(RESOLV_CONF_BACKUP_PATH) {
        Ok(backup) => {
            log::info!("Restoring DNS state from backup");
            let config =
                Config::parse(backup).map_err(|e| Error::Parse(RESOLV_CONF_BACKUP_PATH, e))?;

            write_config(&config)?;

            fs::remove_file(RESOLV_CONF_BACKUP_PATH)
                .map_err(|e| Error::RemoveBackup(RESOLV_CONF_BACKUP_PATH, e))
        }
        Err(ref error) if error.kind() == io::ErrorKind::NotFound => {
            log::debug!("No DNS state backup to restore");
            Ok(())
        }
        Err(error) => Err(Error::ReadResolvConf(RESOLV_CONF_BACKUP_PATH, error)),
    }
}
