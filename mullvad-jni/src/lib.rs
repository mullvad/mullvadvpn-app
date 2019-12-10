#![cfg(target_os = "android")]
#![deny(rust_2018_idioms)]

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
        signature::{JavaType, Primitive},
        sys::{jboolean, jlong, JNI_FALSE, JNI_TRUE},
        JNIEnv,
    },
    IntoJava, JnixEnv,
};
use lazy_static::lazy_static;
use mullvad_daemon::{logging, version, Daemon, DaemonCommandSender};
use mullvad_types::account::AccountData;
use std::{
    path::{Path, PathBuf},
    ptr,
    sync::{mpsc, Arc, Once},
    thread,
};
use talpid_types::{android::AndroidContext, ErrorExt};

const LOG_FILENAME: &str = "daemon.log";

lazy_static! {
    static ref LOG_INIT_RESULT: Result<PathBuf, String> =
        start_logging().map_err(|error| error.display_chain());
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
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_initialize(
    env: JNIEnv<'_>,
    this: JObject<'_>,
    vpnService: JObject<'_>,
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
    env: &JnixEnv<'_>,
    this: &JObject<'_>,
    vpn_service: &JObject<'_>,
    log_dir: PathBuf,
) -> Result<(), Error> {
    let android_context = create_android_context(env, *vpn_service)?;
    let daemon_command_sender = spawn_daemon(env, this, log_dir, android_context)?;
    let daemon_interface = Box::new(DaemonInterface::new(daemon_command_sender));

    set_daemon_interface_address(env, this, Box::into_raw(daemon_interface) as jlong);

    Ok(())
}

fn create_android_context(
    env: &JnixEnv<'_>,
    vpn_service: JObject<'_>,
) -> Result<AndroidContext, Error> {
    Ok(AndroidContext {
        jvm: Arc::new(env.get_java_vm().map_err(Error::GetJvmInstance)?),
        vpn_service: env
            .new_global_ref(vpn_service)
            .map_err(Error::CreateGlobalReference)?,
    })
}

fn spawn_daemon(
    env: &JnixEnv<'_>,
    this: &JObject<'_>,
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

fn set_daemon_interface_address(env: &JnixEnv<'_>, this: &JObject<'_>, address: jlong) {
    let class = env.get_class("net/mullvad/mullvadvpn/service/MullvadDaemon");
    let method_id = env
        .get_method_id(&class, "setDaemonInterfaceAddress", "(J)V")
        .expect("Failed to get method ID for MullvadDaemon.setDaemonInterfaceAddress");
    let return_type = JavaType::Primitive(Primitive::Void);

    let result = env.call_method_unchecked(*this, method_id, return_type, &[JValue::Long(address)]);

    match result {
        Ok(JValue::Void) => {}
        Ok(value) => panic!(
            "Unexpected return value from MullvadDaemon.setDaemonInterfaceAddress: {:?}",
            value
        ),
        Err(error) => panic!(
            "{}",
            error.display_chain_with_msg("Failed to call MullvadDaemon.setDaemonInterfaceAddress")
        ),
    }
}

fn get_daemon_interface_address(env: &JnixEnv<'_>, this: &JObject<'_>) -> *mut DaemonInterface {
    let class = env.get_class("net/mullvad/mullvadvpn/service/MullvadDaemon");
    let method_id = env
        .get_method_id(&class, "getDaemonInterfaceAddress", "()J")
        .expect("Failed to get method ID for MullvadDaemon.getDaemonInterfaceAddress");
    let return_type = JavaType::Primitive(Primitive::Long);

    let result = env.call_method_unchecked(*this, method_id, return_type, &[]);

    match result {
        Ok(JValue::Long(address)) => address as *mut DaemonInterface,
        Ok(value) => panic!(
            "Invalid return value from MullvadDaemon.getDaemonInterfaceAddress: {:?}",
            value
        ),
        Err(error) => panic!(
            "{}",
            error.display_chain_with_msg("Failed to call MullvadDaemon.getDaemonInterfaceAddress")
        ),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_deinitialize(
    env: JNIEnv<'_>,
    this: JObject<'_>,
) {
    let env = JnixEnv::from(env);
    let daemon_interface_address = get_daemon_interface_address(&env, &this);

    set_daemon_interface_address(&env, &this, 0);

    if daemon_interface_address != ptr::null_mut() {
        let _ = unsafe { Box::from_raw(daemon_interface_address) };
    }
}

fn get_daemon_interface<'a>(address: jlong) -> Option<&'a mut DaemonInterface> {
    let pointer = address as *mut DaemonInterface;

    if pointer != ptr::null_mut() {
        Some(Box::leak(unsafe { Box::from_raw(pointer) }))
    } else {
        log::error!("Attempt to get daemon interface while it is null");
        None
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_connect(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) {
    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        if let Err(error) = daemon_interface.connect() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to request daemon to connect")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_disconnect(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) {
    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        if let Err(error) = daemon_interface.disconnect() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to request daemon to disconnect")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_generateWireguardKey<
    'env,
>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.generate_wireguard_key() {
            Ok(keygen_event) => keygen_event.into_java(&env).forget(),
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to request to generate wireguard key")
                );
                JObject::null()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_verifyWireguardKey<
    'env,
>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.verify_wireguard_key() {
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
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getAccountData<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    accountToken: JString<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let account = String::from_java(&env, accountToken);
        let result = daemon_interface.get_account_data(account);

        if let Err(ref error) = &result {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get account data")
            );
        }

        GetAccountDataResult::from(result).into_java(&env).forget()
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getWwwAuthToken<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.get_www_auth_token() {
            Ok(token) => token.into_java(&env).forget(),
            Err(err) => {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to get WWW auth token")
                );
                String::new().into_java(&env).forget()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getCurrentLocation<
    'env,
>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.get_current_location() {
            Ok(location) => location.into_java(&env).forget(),
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to get current location")
                );
                JObject::null()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getCurrentVersion<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.get_current_version() {
            Ok(location) => location.into_java(&env).forget(),
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to get current version")
                );
                String::new().into_java(&env).forget()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getRelayLocations<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.get_relay_locations() {
            Ok(relay_list) => relay_list.into_java(&env).forget(),
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to get relay locations")
                );
                JObject::null()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getSettings<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.get_settings() {
            Ok(settings) => settings.into_java(&env).forget(),
            Err(error) => {
                log::error!("{}", error.display_chain_with_msg("Failed to get settings"));
                JObject::null()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getState<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.get_state() {
            Ok(state) => state.into_java(&env).forget(),
            Err(error) => {
                log::error!("{}", error.display_chain_with_msg("Failed to get state"));
                JObject::null()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getVersionInfo<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.get_version_info() {
            Ok(version_info) => version_info.into_java(&env).forget(),
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to get version information")
                );
                JObject::null()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getWireguardKey<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.get_wireguard_key() {
            Ok(key) => key.into_java(&env).forget(),
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to get wireguard key")
                );
                JObject::null()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_setAccount(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    accountToken: JString<'_>,
) {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let account = <Option<String> as FromJava>::from_java(&env, accountToken);

        if let Err(error) = daemon_interface.set_account(account) {
            log::error!("{}", error.display_chain_with_msg("Failed to set account"));
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_shutdown(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) {
    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        if let Err(error) = daemon_interface.shutdown() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to shutdown daemon thread")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_updateRelaySettings(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    relaySettingsUpdate: JObject<'_>,
) {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let update = FromJava::from_java(&env, relaySettingsUpdate);

        if let Err(error) = daemon_interface.update_relay_settings(update) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to update relay settings")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_dataproxy_MullvadProblemReport_collectReport(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    outputPath: JString<'_>,
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
    env: JNIEnv<'_>,
    _: JObject<'_>,
    userEmail: JString<'_>,
    userMessage: JString<'_>,
    outputPath: JString<'_>,
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
