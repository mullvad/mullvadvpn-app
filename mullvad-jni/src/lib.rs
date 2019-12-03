#![cfg(target_os = "android")]

mod classes;
mod daemon_interface;
mod from_java;
mod is_null;
mod jni_event_listener;

use crate::{
    daemon_interface::DaemonInterface, from_java::FromJava, jni_event_listener::JniEventListener,
};
use jnix::{
    jni::{
        objects::{JObject, JString, JValue},
        sys::{jboolean, JNI_FALSE, JNI_TRUE},
        JNIEnv,
    },
    IntoJava, JnixEnv,
};
use lazy_static::lazy_static;
use mullvad_daemon::{logging, version, Daemon, DaemonCommandSender};
use mullvad_types::account::AccountData;
use std::{
    path::{Path, PathBuf},
    sync::{mpsc, Once},
    thread,
};
use talpid_types::{android::AndroidContext, ErrorExt};

const LOG_FILENAME: &str = "daemon.log";

lazy_static! {
    static ref LOG_INIT_RESULT: Result<PathBuf, String> =
        start_logging().map_err(|error| error.display_chain());
    static ref DAEMON_INTERFACE: DaemonInterface = DaemonInterface::new();
}

static LOAD_CLASSES: Once = Once::new();

#[derive(Debug, err_derive::Error)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to create global reference to Java object")]
    CreateGlobalReference(#[error(cause)] jnix::jni::errors::Error),

    #[error(display = "Failed to get cache directory path")]
    GetCacheDir(#[error(source)] mullvad_paths::Error),

    #[error(display = "Failed to get Java VM instance")]
    GetJvmInstance(#[error(cause)] jnix::jni::errors::Error),

    #[error(display = "Failed to get log directory path")]
    GetLogDir(#[error(source)] mullvad_paths::Error),

    #[error(display = "Failed to initialize the mullvad daemon")]
    InitializeDaemon(#[error(source)] mullvad_daemon::Error),

    #[error(display = "Failed to spawn the JNI event listener")]
    SpawnJniEventListener(#[error(source)] jni_event_listener::Error),

    #[error(display = "Failed to start logger")]
    StartLogging(#[error(source)] logging::Error),
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum GetAccountDataResult {
    Ok(AccountData),
    InvalidAccount,
    RpcError,
    OtherError,
}

impl From<Result<AccountData, daemon_interface::Error>> for GetAccountDataResult {
    fn from(result: Result<AccountData, daemon_interface::Error>) -> Self {
        match result {
            Ok(account_data) => GetAccountDataResult::Ok(account_data),
            Err(error) => match error {
                daemon_interface::Error::RpcError(jsonrpc_client_core::Error(
                    jsonrpc_client_core::ErrorKind::JsonRpcError(jsonrpc_core::Error {
                        code: jsonrpc_core::ErrorCode::ServerError(-200),
                        ..
                    }),
                    _,
                )) => GetAccountDataResult::InvalidAccount,
                daemon_interface::Error::RpcError(_) => GetAccountDataResult::RpcError,
                _ => GetAccountDataResult::OtherError,
            },
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_initialize(
    env: JNIEnv,
    this: JObject,
    vpnService: JObject,
) {
    let env = JnixEnv::from(env);

    match *LOG_INIT_RESULT {
        Ok(ref log_dir) => {
            LOAD_CLASSES.call_once(|| env.preload_classes(classes::CLASSES.iter().cloned()));

            if let Err(error) = initialize(&env, &this, &vpnService, log_dir.clone()) {
                log::error!("{}", error.display_chain());
            }
        }
        Err(ref message) => env
            .throw(message.as_str())
            .expect("Failed to throw exception"),
    }
}

fn start_logging() -> Result<PathBuf, Error> {
    let log_dir = mullvad_paths::log_dir().map_err(Error::GetLogDir)?;
    let log_file = log_dir.join(LOG_FILENAME);

    logging::init_logger(log::LevelFilter::Debug, Some(&log_file), true)
        .map_err(Error::StartLogging)?;
    log_panics::init();
    version::log_version();

    Ok(log_dir)
}

fn initialize(
    env: &JnixEnv,
    this: &JObject,
    vpn_service: &JObject,
    log_dir: PathBuf,
) -> Result<(), Error> {
    let android_context = create_android_context(env, *vpn_service)?;
    let daemon_command_sender = spawn_daemon(env, this, log_dir, android_context)?;

    DAEMON_INTERFACE.set_command_sender(daemon_command_sender);

    Ok(())
}

fn create_android_context(env: &JnixEnv, vpn_service: JObject) -> Result<AndroidContext, Error> {
    Ok(AndroidContext {
        jvm: env.get_java_vm().map_err(Error::GetJvmInstance)?,
        vpn_service: env
            .new_global_ref(vpn_service)
            .map_err(Error::CreateGlobalReference)?,
    })
}

fn spawn_daemon(
    env: &JnixEnv,
    this: &JObject,
    log_dir: PathBuf,
    android_context: AndroidContext,
) -> Result<DaemonCommandSender, Error> {
    let listener = JniEventListener::spawn(env, this).map_err(Error::SpawnJniEventListener)?;
    let (tx, rx) = mpsc::channel();

    thread::spawn(
        move || match create_daemon(listener, log_dir, android_context) {
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
        },
    );

    rx.recv().unwrap()
}

fn create_daemon(
    listener: JniEventListener,
    log_dir: PathBuf,
    android_context: AndroidContext,
) -> Result<Daemon<JniEventListener>, Error> {
    let resource_dir = mullvad_paths::get_resource_dir();
    let cache_dir = mullvad_paths::cache_dir().map_err(Error::GetCacheDir)?;

    let daemon = Daemon::start_with_event_listener(
        listener,
        Some(log_dir),
        resource_dir,
        cache_dir,
        android_context,
    )
    .map_err(Error::InitializeDaemon)?;

    Ok(daemon)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_connect(_: JNIEnv, _: JObject) {
    if let Err(error) = DAEMON_INTERFACE.connect() {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to request daemon to connect")
        );
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_disconnect(_: JNIEnv, _: JObject) {
    if let Err(error) = DAEMON_INTERFACE.disconnect() {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to request daemon to disconnect")
        );
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_generateWireguardKey<'env>(
    env: JNIEnv<'env>,
    _: JObject,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    match DAEMON_INTERFACE.generate_wireguard_key() {
        Ok(keygen_event) => keygen_event.into_java(&env).forget(),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to request to generate wireguard key")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_verifyWireguardKey<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    match DAEMON_INTERFACE.verify_wireguard_key() {
        Ok(key_is_valid) => env
            .new_object(
                &env.get_class("java/lang/Boolean"),
                "(Z)V",
                &[JValue::Bool(key_is_valid as jboolean)],
            )
            .expect("Failed to create Boolean Java object"),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to verify wireguard key")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getAccountData<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
    accountToken: JString,
) -> JObject<'env> {
    let env = JnixEnv::from(env);
    let account = String::from_java(&env, accountToken);
    let result = DAEMON_INTERFACE.get_account_data(account);

    if let Err(ref error) = &result {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to get account data")
        );
    }

    GetAccountDataResult::from(result).into_java(&env).forget()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getWwwAuthToken<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    match DAEMON_INTERFACE.get_www_auth_token() {
        Ok(token) => token.into_java(&env).forget(),
        Err(err) => {
            log::error!(
                "{}",
                err.display_chain_with_msg("Failed to get WWW auth token")
            );
            String::new().into_java(&env).forget()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getCurrentLocation<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    match DAEMON_INTERFACE.get_current_location() {
        Ok(location) => location.into_java(&env).forget(),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get current location")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getCurrentVersion<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    match DAEMON_INTERFACE.get_current_version() {
        Ok(location) => location.into_java(&env).forget(),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get current version")
            );
            String::new().into_java(&env).forget()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getRelayLocations<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    match DAEMON_INTERFACE.get_relay_locations() {
        Ok(relay_list) => relay_list.into_java(&env).forget(),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get relay locations")
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
    let env = JnixEnv::from(env);

    match DAEMON_INTERFACE.get_settings() {
        Ok(settings) => settings.into_java(&env).forget(),
        Err(error) => {
            log::error!("{}", error.display_chain_with_msg("Failed to get settings"));
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getState<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    match DAEMON_INTERFACE.get_state() {
        Ok(state) => state.into_java(&env).forget(),
        Err(error) => {
            log::error!("{}", error.display_chain_with_msg("Failed to get state"));
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getVersionInfo<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    match DAEMON_INTERFACE.get_version_info() {
        Ok(version_info) => version_info.into_java(&env).forget(),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get version information")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getWireguardKey<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    match DAEMON_INTERFACE.get_wireguard_key() {
        Ok(key) => key.into_java(&env).forget(),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get wireguard key")
            );
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
    let env = JnixEnv::from(env);
    let account = <Option<String> as FromJava>::from_java(&env, accountToken);

    if let Err(error) = DAEMON_INTERFACE.set_account(account) {
        log::error!("{}", error.display_chain_with_msg("Failed to set account"));
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_shutdown(_: JNIEnv, _: JObject) {
    if let Err(error) = DAEMON_INTERFACE.shutdown() {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to shutdown daemon thread")
        );
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_updateRelaySettings(
    env: JNIEnv,
    _: JObject,
    relaySettingsUpdate: JObject,
) {
    let env = JnixEnv::from(env);
    let update = FromJava::from_java(&env, relaySettingsUpdate);

    if let Err(error) = DAEMON_INTERFACE.update_relay_settings(update) {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to update relay settings")
        );
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_dataproxy_MullvadProblemReport_collectReport(
    env: JNIEnv,
    _: JObject,
    outputPath: JString,
) -> jboolean {
    let env = JnixEnv::from(env);
    let output_path_string = String::from_java(&env, outputPath);
    let output_path = Path::new(&output_path_string);

    match mullvad_problem_report::collect_report(&[], output_path, Vec::new()) {
        Ok(()) => JNI_TRUE,
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to collect problem report")
            );
            JNI_FALSE
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_dataproxy_MullvadProblemReport_sendProblemReport(
    env: JNIEnv,
    _: JObject,
    userEmail: JString,
    userMessage: JString,
    outputPath: JString,
) -> jboolean {
    let env = JnixEnv::from(env);
    let user_email = String::from_java(&env, userEmail);
    let user_message = String::from_java(&env, userMessage);
    let output_path_string = String::from_java(&env, outputPath);
    let output_path = Path::new(&output_path_string);

    match mullvad_problem_report::send_problem_report(&user_email, &user_message, output_path) {
        Ok(()) => JNI_TRUE,
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to collect problem report")
            );
            JNI_FALSE
        }
    }
}
