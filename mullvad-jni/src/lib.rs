#![cfg(target_os = "android")]
#![deny(rust_2018_idioms)]

mod classes;
mod daemon_interface;
mod is_null;
mod jni_event_listener;
mod talpid_vpn_service;

use crate::{daemon_interface::DaemonInterface, jni_event_listener::JniEventListener};
use jnix::{
    jni::{
        objects::{GlobalRef, JObject, JString, JValue},
        signature::{JavaType, Primitive},
        sys::{jboolean, jlong, JNI_FALSE, JNI_TRUE},
        JNIEnv, JavaVM,
    },
    FromJava, IntoJava, JnixEnv,
};
use mullvad_daemon::{
    exception_logging, logging, runtime::new_runtime_builder, version, Daemon, DaemonCommandChannel,
};
use mullvad_rpc::{rest::Error as RestError, StatusCode};
use mullvad_types::account::{AccountData, VoucherSubmission};
use std::{
    io,
    path::{Path, PathBuf},
    ptr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Once,
    },
    thread,
};
use talpid_types::{android::AndroidContext, ErrorExt};

const LOG_FILENAME: &str = "daemon.log";

static DAEMON_INSTANCE_COUNT: AtomicUsize = AtomicUsize::new(0);
static LOAD_CLASSES: Once = Once::new();
static LOG_START: Once = Once::new();
static mut LOG_INIT_RESULT: Option<Result<(), String>> = None;

#[derive(Debug, err_derive::Error)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to create global reference to Java object")]
    CreateGlobalReference(#[error(cause)] jnix::jni::errors::Error),

    #[error(display = "Failed to get Java VM instance")]
    GetJvmInstance(#[error(cause)] jnix::jni::errors::Error),

    #[error(display = "Failed to initialize the mullvad daemon")]
    InitializeDaemon(#[error(source)] mullvad_daemon::Error),

    #[error(display = "Failed to spawn the tokio runtime")]
    InitializeTokioRuntime(#[error(source)] io::Error),

    #[error(display = "Failed to spawn the JNI event listener")]
    SpawnJniEventListener(#[error(source)] jni_event_listener::Error),
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
                daemon_interface::Error::RpcError(RestError::ApiError(status, _code))
                    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN =>
                {
                    GetAccountDataResult::InvalidAccount
                }
                daemon_interface::Error::RpcError(_) => GetAccountDataResult::RpcError,
                _ => GetAccountDataResult::OtherError,
            },
        }
    }
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum VoucherSubmissionResult {
    Ok(VoucherSubmission),
    InvalidVoucher,
    VoucherAlreadyUsed,
    RpcError,
    OtherError,
}

impl From<Result<VoucherSubmission, daemon_interface::Error>> for VoucherSubmissionResult {
    fn from(result: Result<VoucherSubmission, daemon_interface::Error>) -> Self {
        match result {
            Ok(submission) => VoucherSubmissionResult::Ok(submission),
            Err(daemon_interface::Error::RpcError(RestError::ApiError(_, code))) => {
                match code.as_str() {
                    mullvad_rpc::INVALID_VOUCHER => VoucherSubmissionResult::InvalidVoucher,
                    mullvad_rpc::VOUCHER_USED => VoucherSubmissionResult::VoucherAlreadyUsed,
                    _ => VoucherSubmissionResult::RpcError,
                }
            }
            Err(daemon_interface::Error::RpcError(_)) => VoucherSubmissionResult::RpcError,
            _ => VoucherSubmissionResult::OtherError,
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_initialize(
    env: JNIEnv<'_>,
    this: JObject<'_>,
    vpnService: JObject<'_>,
    cacheDirectory: JObject<'_>,
    resourceDirectory: JObject<'_>,
) {
    let env = JnixEnv::from(env);
    let cache_dir = PathBuf::from(String::from_java(&env, cacheDirectory));
    let resource_dir = PathBuf::from(String::from_java(&env, resourceDirectory));

    match start_logging(&resource_dir) {
        Ok(()) => {
            LOAD_CLASSES.call_once(|| env.preload_classes(classes::CLASSES.iter().cloned()));

            if let Err(error) = initialize(&env, &this, &vpnService, cache_dir, resource_dir) {
                log::error!("{}", error.display_chain());
            }
        }
        Err(message) => env
            .throw(message.as_str())
            .expect("Failed to throw exception"),
    }
}

fn start_logging(log_dir: &Path) -> Result<(), String> {
    unsafe {
        LOG_START.call_once(|| LOG_INIT_RESULT = Some(initialize_logging(log_dir)));
        LOG_INIT_RESULT
            .clone()
            .expect("Logging not properly initialized")
    }
}

fn initialize_logging(log_dir: &Path) -> Result<(), String> {
    let log_file = log_dir.join(LOG_FILENAME);

    logging::init_logger(log::LevelFilter::Debug, Some(&log_file), true)
        .map_err(|error| error.display_chain_with_msg("Failed to start logger"))?;
    exception_logging::enable();
    log_panics::init();
    version::log_version();

    Ok(())
}

fn initialize(
    env: &JnixEnv<'_>,
    this: &JObject<'_>,
    vpn_service: &JObject<'_>,
    cache_dir: PathBuf,
    resource_dir: PathBuf,
) -> Result<(), Error> {
    let android_context = create_android_context(env, *vpn_service)?;
    let daemon_command_channel = DaemonCommandChannel::new();
    let daemon_interface = Box::new(DaemonInterface::new(daemon_command_channel.sender()));

    spawn_daemon(
        env,
        this,
        cache_dir,
        resource_dir,
        daemon_command_channel,
        android_context,
    )?;

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
    cache_dir: PathBuf,
    resource_dir: PathBuf,
    command_channel: DaemonCommandChannel,
    android_context: AndroidContext,
) -> Result<(), Error> {
    let listener = JniEventListener::spawn(env, this).map_err(Error::SpawnJniEventListener)?;
    let daemon_object = env
        .new_global_ref(*this)
        .map_err(Error::CreateGlobalReference)?;
    let (tx, rx) = mpsc::channel();

    let mut runtime = new_runtime_builder()
        .build()
        .map_err(Error::InitializeTokioRuntime)?;

    thread::spawn(move || {
        let jvm = android_context.jvm.clone();
        let running_instances = DAEMON_INSTANCE_COUNT.fetch_add(1, Ordering::AcqRel);

        if running_instances != 0 {
            log::error!(
                "It seems that there are already {} instances of the Mullvad daemon running",
                running_instances
            );
        }

        let daemon = runtime.block_on(Daemon::start(
            Some(resource_dir.clone()),
            resource_dir.clone(),
            resource_dir,
            cache_dir,
            listener,
            command_channel,
            android_context,
        ));

        DAEMON_INSTANCE_COUNT.fetch_sub(1, Ordering::AcqRel);

        match daemon {
            Ok(daemon) => {
                let _ = tx.send(Ok(()));
                match runtime.block_on(daemon.run()) {
                    Ok(()) => log::info!("Mullvad daemon has stopped"),
                    Err(error) => log::error!("{}", error.display_chain()),
                }
            }
            Err(error) => {
                let _ = tx.send(Err(Error::InitializeDaemon(error)));
            }
        }

        notify_daemon_stopped(jvm, daemon_object);
    });

    rx.recv().unwrap()
}

fn notify_daemon_stopped(jvm: Arc<JavaVM>, daemon_object: GlobalRef) {
    match jvm.attach_current_thread_as_daemon() {
        Ok(env) => {
            let env = JnixEnv::from(env);
            let class = env.get_class("net/mullvad/mullvadvpn/service/MullvadDaemon");
            let object = daemon_object.as_obj();
            let method_id = env
                .get_method_id(&class, "notifyDaemonStopped", "()V")
                .expect("Failed to get method ID for MullvadDaemon.notifyDaemonStopped");
            let return_type = JavaType::Primitive(Primitive::Void);

            let result = env.call_method_unchecked(object, method_id, return_type, &[]);

            match result {
                Ok(JValue::Void) => {}
                Ok(value) => panic!(
                    "Unexpected return value from MullvadDaemon.notifyDaemonStopped: {:?}",
                    value
                ),
                Err(error) => panic!(
                    "{}",
                    error
                        .display_chain_with_msg("Failed to call MullvadDaemon.notifyDaemonStopped")
                ),
            }
        }
        Err(error) => log::error!(
            "{}",
            error.display_chain_with_msg("Failed to notify that the daemon stopped")
        ),
    }
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
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_deinitialize(
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
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_createNewAccount<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.create_new_account() {
            Ok(account) => account.into_java(&env).forget(),
            Err(error) => {
                log_request_error("create new account", &error);
                JObject::null()
            }
        }
    } else {
        JObject::null()
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
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getAccountHistory<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    match get_daemon_interface(daemon_interface_address) {
        Some(daemon_interface) => daemon_interface
            .get_account_history()
            .map(|history| history.into_java(&env).forget())
            .unwrap_or_else(|err| {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to get account history")
                );
                JObject::null()
            }),
        None => JObject::null(),
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
            log_request_error("get account data", error);
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
            Err(error) => {
                log_request_error("get WWW auth token", &error);
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
            Ok(version) => version.into_java(&env).forget(),
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
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_reconnect(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) {
    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        if let Err(error) = daemon_interface.reconnect() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to request daemon to reconnect")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_removeAccountFromHistory(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    accountToken: JString<'_>,
) {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let account = String::from_java(&env, accountToken);

        if let Err(error) = daemon_interface.remove_account_from_history(account) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to remove account from history")
            );
        }
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
        let account = Option::from_java(&env, accountToken);

        if let Err(error) = daemon_interface.set_account(account) {
            log::error!("{}", error.display_chain_with_msg("Failed to set account"));
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_setAllowLan(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    allow_lan: jboolean,
) {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let allow_lan = bool::from_java(&env, allow_lan);

        if let Err(error) = daemon_interface.set_allow_lan(allow_lan) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to set allow LAN")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_setAutoConnect(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    auto_connect: jboolean,
) {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let auto_connect = bool::from_java(&env, auto_connect);

        if let Err(error) = daemon_interface.set_auto_connect(auto_connect) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to set auto-connect")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_setWireguardMtu(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    wireguard_mtu: JObject<'_>,
) {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let wireguard_mtu = Option::<i32>::from_java(&env, wireguard_mtu).map(|value| value as u16);

        if let Err(error) = daemon_interface.set_wireguard_mtu(wireguard_mtu) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to set WireGuard MTU")
            );
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
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_submitVoucher<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    voucher: JString<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    let result = if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let voucher = String::from_java(&env, voucher);
        let raw_result = daemon_interface.submit_voucher(voucher);

        if let Err(ref error) = &raw_result {
            log_request_error("submit voucher code", error);
        }

        VoucherSubmissionResult::from(raw_result)
    } else {
        VoucherSubmissionResult::OtherError
    };

    result.into_java(&env).forget()
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
    logDirectory: JString<'_>,
    outputPath: JString<'_>,
) -> jboolean {
    let env = JnixEnv::from(env);
    let log_dir_string = String::from_java(&env, logDirectory);
    let log_dir = Path::new(&log_dir_string);
    let output_path_string = String::from_java(&env, outputPath);
    let output_path = Path::new(&output_path_string);

    match mullvad_problem_report::collect_report(&[], output_path, Vec::new(), log_dir) {
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

fn log_request_error(request: &str, error: &daemon_interface::Error) {
    match error {
        daemon_interface::Error::RpcError(RestError::Aborted(_)) => {
            log::debug!("Request to {} cancelled", request);
        }
        error => {
            log::error!(
                "{}",
                error.display_chain_with_msg(&format!("Failed to {}", request))
            );
        }
    }
}
