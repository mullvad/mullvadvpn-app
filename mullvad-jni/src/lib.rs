#![cfg(target_os = "android")]

mod daemon_interface;
mod from_java;
mod into_java;

use crate::{daemon_interface::DaemonInterface, from_java::FromJava, into_java::IntoJava};
use jni::{
    objects::{GlobalRef, JObject, JString},
    JNIEnv,
};
use lazy_static::lazy_static;
use mullvad_daemon::{logging, version, Daemon, DaemonCommandSender, EventListener};
use mullvad_types::{relay_list::RelayList, settings::Settings};
use parking_lot::{Mutex, RwLock};
use std::{collections::HashMap, path::PathBuf, sync::mpsc, thread};
use talpid_types::{tunnel::TunnelStateTransition, ErrorExt};

const LOG_FILENAME: &str = "daemon.log";

const CLASSES_TO_LOAD: &[&str] = &[
    "net/mullvad/mullvadvpn/model/AccountData",
    "net/mullvad/mullvadvpn/model/Settings",
];

lazy_static! {
    static ref DAEMON_INTERFACE: Mutex<DaemonInterface> = Mutex::new(DaemonInterface::new());
    static ref CLASSES: RwLock<HashMap<&'static str, GlobalRef>> =
        RwLock::new(HashMap::with_capacity(CLASSES_TO_LOAD.len()));
}

#[derive(Debug, err_derive::Error)]
pub enum Error {
    #[error(display = "Failed to get cache directory path")]
    GetCacheDir(#[error(cause)] mullvad_paths::Error),

    #[error(display = "Failed to initialize the mullvad daemon")]
    InitializeDaemon(#[error(cause)] mullvad_daemon::Error),
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_initialize(
    env: JNIEnv,
    _: JObject,
) {
    let log_dir = start_logging();

    load_classes(&env);

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

fn load_classes(env: &JNIEnv) {
    let mut classes = CLASSES.write();

    for class in CLASSES_TO_LOAD {
        classes.insert(class, load_class_reference(env, class));
    }
}

fn load_class_reference(env: &JNIEnv, name: &str) -> GlobalRef {
    let class = match env.find_class(name) {
        Ok(class) => class,
        Err(_) => panic!("Failed to find {} Java class", name),
    };

    env.new_global_ref(JObject::from(class))
        .expect("Failed to convert local reference to Java class into a global reference")
}

fn initialize(log_dir: PathBuf) -> Result<(), Error> {
    let daemon_command_sender = spawn_daemon(log_dir)?;

    DAEMON_INTERFACE
        .lock()
        .set_command_sender(daemon_command_sender);

    Ok(())
}

fn spawn_daemon(log_dir: PathBuf) -> Result<DaemonCommandSender, Error> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || match create_daemon(log_dir) {
        Ok(daemon) => {
            let _ = tx.send(Ok(daemon.command_sender()));
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

fn get_class(name: &str) -> GlobalRef {
    match CLASSES.read().get(name) {
        Some(class) => class.clone(),
        None => panic!("Class not loaded: {}", name),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getAccountData<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
    accountToken: JString,
) -> JObject<'env> {
    let daemon = DAEMON_INTERFACE.lock();

    let account = String::from_java(&env, accountToken);

    match daemon.get_account_data(account) {
        Ok(data) => data.into_java(&env),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get account data")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getSettings<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let daemon = DAEMON_INTERFACE.lock();

    match daemon.get_settings() {
        Ok(settings) => settings.into_java(&env),
        Err(error) => {
            log::error!("{}", error.display_chain_with_msg("Failed to get settings"));
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_setAccount(
    env: JNIEnv,
    _: JObject,
    accountToken: JString,
) {
    let daemon = DAEMON_INTERFACE.lock();

    let account = <Option<String> as FromJava>::from_java(&env, accountToken);

    if let Err(error) = daemon.set_account(account) {
        log::error!("{}", error.display_chain_with_msg("Failed to set account"));
    }
}
