#![cfg(target_os = "android")]

use jni::{objects::JObject, JNIEnv};
use mullvad_daemon::{logging, version, Daemon, EventListener};
use mullvad_types::{relay_list::RelayList, settings::Settings};
use std::{path::PathBuf, sync::mpsc, thread};
use talpid_types::{tunnel::TunnelStateTransition, ErrorExt};

const LOG_FILENAME: &str = "daemon.log";

#[derive(Debug, err_derive::Error)]
pub enum Error {
    #[error(display = "Failed to get cache directory path")]
    GetCacheDir(#[error(cause)] mullvad_paths::Error),

    #[error(display = "Failed to initialize the mullvad daemon")]
    InitializeDaemon(#[error(cause)] mullvad_daemon::Error),
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_initialize(_: JNIEnv, _: JObject) {
    let log_dir = start_logging();

    if let Err(error) = initialize(log_dir) {
        log::error!("{}", error.display_chain());
    }
}

fn start_logging() -> PathBuf {
    let log_dir = mullvad_paths::log_dir().unwrap();
    let log_file = log_dir.join(LOG_FILENAME);

    logging::init_logger(log::LevelFilter::Debug, Some(&log_file), true).unwrap();
    log_panics::init();
    version::log_version();

    log_dir
}

fn initialize(log_dir: PathBuf) -> Result<(), Error> {
    spawn_daemon(log_dir)
}

fn spawn_daemon(log_dir: PathBuf) -> Result<(), Error> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || match create_daemon(log_dir) {
        Ok(daemon) => {
            let _ = tx.send(Ok(()));
            match daemon.run() {
                Ok(()) => log::info!("Mullvad daemon has stopped"),
                Err(error) => log::error!("{}", error.display_chain()),
            }
        }
        Err(error) => {
            let _ = tx.send(Err(error));
        }
    });

    rx.recv().unwrap()
}

fn create_daemon(log_dir: PathBuf) -> Result<Daemon<DummyListener>, Error> {
    let resource_dir = mullvad_paths::get_resource_dir();
    let cache_dir = mullvad_paths::cache_dir().map_err(Error::GetCacheDir)?;

    let daemon = Daemon::start_with_event_listener(
        DummyListener,
        Some(log_dir),
        resource_dir,
        cache_dir,
        version::PRODUCT_VERSION.to_owned(),
    )
    .map_err(Error::InitializeDaemon)?;

    Ok(daemon)
}

#[derive(Clone, Copy, Debug)]
struct DummyListener;

impl EventListener for DummyListener {
    fn notify_new_state(&self, _: TunnelStateTransition) {}
    fn notify_settings(&self, _: Settings) {}
    fn notify_relay_list(&self, _: RelayList) {}
}
