#![cfg(target_os = "android")]
#![deny(rust_2018_idioms)]

mod classes;
mod daemon_interface;
mod is_null;
mod jni_event_listener;
mod problem_report;
mod talpid_vpn_service;

use crate::{daemon_interface::DaemonInterface, jni_event_listener::JniEventListener};
use jnix::{
    jni::{
        objects::{GlobalRef, JObject, JString, JValue},
        signature::{JavaType, Primitive},
        sys::{jboolean, jlong},
        JNIEnv, JavaVM,
    },
    FromJava, IntoJava, JnixEnv,
};
use mullvad_api::{rest::Error as RestError, StatusCode};
use mullvad_daemon::{
    device, exception_logging, logging, runtime::new_runtime_builder, version, Daemon,
    DaemonCommandChannel,
};
use mullvad_types::{
    account::{AccountData, VoucherSubmission},
    settings::DnsOptions,
};
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

#[cfg(feature = "api-override")]
use std::net::{IpAddr, SocketAddr};

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
pub enum LoginResult {
    Ok,
    InvalidAccount,
    MaxDevicesReached,
    RpcError,
    OtherError,
}

impl From<Result<(), daemon_interface::Error>> for LoginResult {
    fn from(result: Result<(), daemon_interface::Error>) -> Self {
        match result {
            Ok(()) => LoginResult::Ok,
            Err(error) => match error {
                daemon_interface::Error::OtherError(mullvad_daemon::Error::LoginError(error)) => {
                    match error {
                        device::Error::InvalidAccount => LoginResult::InvalidAccount,
                        device::Error::MaxDevicesReached => LoginResult::MaxDevicesReached,
                        device::Error::OtherRestError(_) => LoginResult::RpcError,
                        _ => LoginResult::OtherError,
                    }
                }
                daemon_interface::Error::RpcError(_) => LoginResult::RpcError,
                _ => LoginResult::OtherError,
            },
        }
    }
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum RemoveDeviceResult {
    Ok,
    NotFound,
    RpcError,
    OtherError,
}

impl From<Result<(), daemon_interface::Error>> for RemoveDeviceResult {
    fn from(result: Result<(), daemon_interface::Error>) -> Self {
        match result {
            Ok(()) => RemoveDeviceResult::Ok,
            Err(error) => match error {
                daemon_interface::Error::OtherError(mullvad_daemon::Error::LoginError(error)) => {
                    match error {
                        device::Error::InvalidAccount => RemoveDeviceResult::RpcError,
                        device::Error::InvalidDevice => RemoveDeviceResult::NotFound,
                        device::Error::OtherRestError(_) => RemoveDeviceResult::RpcError,
                        _ => RemoveDeviceResult::OtherError,
                    }
                }
                daemon_interface::Error::RpcError(_) => RemoveDeviceResult::RpcError,
                _ => RemoveDeviceResult::OtherError,
            },
        }
    }
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum VoucherSubmissionResult {
    Ok(VoucherSubmission),
    Error(VoucherSubmissionError),
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum VoucherSubmissionError {
    InvalidVoucher,
    VoucherAlreadyUsed,
    RpcError,
    OtherError,
}

impl From<Result<VoucherSubmission, daemon_interface::Error>> for VoucherSubmissionResult {
    fn from(result: Result<VoucherSubmission, daemon_interface::Error>) -> Self {
        match result {
            Ok(submission) => VoucherSubmissionResult::Ok(submission),
            Err(error) => VoucherSubmissionResult::Error(error.into()),
        }
    }
}

impl From<daemon_interface::Error> for VoucherSubmissionError {
    fn from(error: daemon_interface::Error) -> Self {
        match error {
            daemon_interface::Error::RpcError(RestError::ApiError(_, code)) => {
                match code.as_str() {
                    mullvad_api::INVALID_VOUCHER => VoucherSubmissionError::InvalidVoucher,
                    mullvad_api::VOUCHER_USED => VoucherSubmissionError::VoucherAlreadyUsed,
                    _ => VoucherSubmissionError::RpcError,
                }
            }
            daemon_interface::Error::RpcError(_) => VoucherSubmissionError::RpcError,
            _ => VoucherSubmissionError::OtherError,
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
    apiEndpoint: JObject<'_>,
) {
    let env = JnixEnv::from(env);
    let cache_dir = PathBuf::from(String::from_java(&env, cacheDirectory));
    let resource_dir = PathBuf::from(String::from_java(&env, resourceDirectory));

    let api_endpoint = if !apiEndpoint.is_null() {
        #[cfg(feature = "api-override")]
        {
            Some(api_endpoint_from_java(&env, apiEndpoint))
        }
        #[cfg(not(feature = "api-override"))]
        {
            log::warn!("apiEndpoint will be ignored since 'api-override' is not enabled");
            None
        }
    } else {
        None
    };

    match start_logging(&resource_dir) {
        Ok(()) => {
            version::log_version();

            LOAD_CLASSES.call_once(|| env.preload_classes(classes::CLASSES.iter().cloned()));

            if let Err(error) = initialize(
                &env,
                &this,
                &vpnService,
                cache_dir,
                resource_dir,
                api_endpoint,
            ) {
                log::error!("{}", error.display_chain());
            }
        }
        Err(message) => env
            .throw(message.as_str())
            .expect("Failed to throw exception"),
    }
}

#[cfg(feature = "api-override")]
fn api_endpoint_from_java(env: &JnixEnv<'_>, object: JObject<'_>) -> mullvad_api::ApiEndpoint {
    let mut endpoint = mullvad_api::ApiEndpoint::from_env_vars();

    let address = env
        .call_method(object, "component1", "()Ljava/net/InetSocketAddress;", &[])
        .expect("missing ApiEndpoint.address")
        .l()
        .expect("ApiEndpoint.address is not an InetSocketAddress");

    endpoint.addr =
        try_socketaddr_from_java(env, address).expect("received unresolved InetSocketAddress");
    endpoint.disable_address_cache = env
        .call_method(object, "component2", "()Z", &[])
        .expect("missing ApiEndpoint.disableAddressCache")
        .z()
        .expect("ApiEndpoint.disableAddressCache is not a bool");
    endpoint.disable_tls = env
        .call_method(object, "component3", "()Z", &[])
        .expect("missing ApiEndpoint.disableTls")
        .z()
        .expect("ApiEndpoint.disableTls is not a bool");
    endpoint.force_direct_connection = env
        .call_method(object, "component4", "()Z", &[])
        .expect("missing ApiEndpoint.forceDirectConnection")
        .z()
        .expect("ApiEndpoint.forceDirectConnection is not a bool");

    endpoint
}

/// Converts InetSocketAddress to a SocketAddr. Return `None` if the
/// hostname is unresolved.
#[cfg(feature = "api-override")]
fn try_socketaddr_from_java(env: &JnixEnv<'_>, address: JObject<'_>) -> Option<SocketAddr> {
    let class = env.get_class("java/net/InetSocketAddress");

    let method_id = env
        .get_method_id(&class, "getAddress", "()Ljava/net/InetAddress;")
        .expect("Failed to get method ID for InetSocketAddress.getAddress()");
    let return_type = JavaType::Object("java/net/InetAddress".to_owned());

    let ip_addr = env
        .call_method_unchecked(address, method_id, return_type, &[])
        .expect("Failed to call InetSocketAddress.getAddress()")
        .l()
        .expect("Call to InetSocketAddress.getAddress() did not return an object");

    if ip_addr.is_null() {
        return None;
    }

    let method_id = env
        .get_method_id(&class, "getPort", "()I")
        .expect("Failed to get method ID for InetSocketAddress.getPort()");
    let return_type = JavaType::Primitive(Primitive::Int);

    let port = env
        .call_method_unchecked(address, method_id, return_type, &[])
        .expect("Failed to call InetSocketAddress.getPort()")
        .i()
        .expect("Call to InetSocketAddress.getPort() did not return an int");

    Some(SocketAddr::new(
        IpAddr::from_java(env, ip_addr),
        u16::try_from(port).expect("invalid port"),
    ))
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

    Ok(())
}

fn initialize(
    env: &JnixEnv<'_>,
    this: &JObject<'_>,
    vpn_service: &JObject<'_>,
    cache_dir: PathBuf,
    resource_dir: PathBuf,
    api_endpoint: Option<mullvad_api::ApiEndpoint>,
) -> Result<(), Error> {
    let android_context = create_android_context(env, *vpn_service)?;
    let daemon_command_channel = DaemonCommandChannel::new();
    let daemon_interface = Box::new(DaemonInterface::new(daemon_command_channel.sender()));

    spawn_daemon(
        env,
        this,
        cache_dir,
        resource_dir,
        api_endpoint,
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
    #[cfg_attr(not(feature = "api-override"), allow(unused_variables))] api_endpoint: Option<
        mullvad_api::ApiEndpoint,
    >,
    command_channel: DaemonCommandChannel,
    android_context: AndroidContext,
) -> Result<(), Error> {
    let listener = JniEventListener::spawn(env, this).map_err(Error::SpawnJniEventListener)?;
    let daemon_object = env
        .new_global_ref(*this)
        .map_err(Error::CreateGlobalReference)?;
    let (tx, rx) = mpsc::channel();

    let runtime = new_runtime_builder()
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

        #[cfg(feature = "api-override")]
        if let Some(api_endpoint) = api_endpoint {
            log::debug!("Overriding API endpoint: {api_endpoint:?}");
            if mullvad_api::API.override_init(api_endpoint).is_err() {
                log::warn!("Ignoring API settings (already initialized)");
            }
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
            .unwrap_or(JObject::null()),
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
        GetAccountDataResult::OtherError.into_java(&env).forget()
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
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_clearAccountHistory(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) {
    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        if let Err(error) = daemon_interface.clear_account_history() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to clear account history")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_loginAccount<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    accountToken: JString<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let account = String::from_java(&env, accountToken);
        let result = daemon_interface.login_account(account);

        if let Err(ref error) = &result {
            log_request_error("login account", error);
        }

        LoginResult::from(result).into_java(&env).forget()
    } else {
        LoginResult::OtherError.into_java(&env).forget()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_logoutAccount(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) {
    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        if let Err(error) = daemon_interface.logout_account() {
            log::error!("{}", error.display_chain_with_msg("Failed to log out"));
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getDevice<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        match daemon_interface.get_device() {
            Ok(key) => key.into_java(&env).forget(),
            Err(error) => {
                log::error!("{}", error.display_chain_with_msg("Failed to get device"));
                JObject::null()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_updateDevice(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) {
    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        if let Err(error) = daemon_interface.update_device() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to update device")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_listDevices<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    account_token: JString<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let token = String::from_java(&env, account_token);
        match daemon_interface.list_devices(token) {
            Ok(key) => key.into_java(&env).forget(),
            Err(error) => {
                log::error!("{}", error.display_chain_with_msg("Failed to list devices"));
                JObject::null()
            }
        }
    } else {
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_removeDevice<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    account_token: JString<'_>,
    device_id: JString<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    let result = if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let token = String::from_java(&env, account_token);
        let device_id = String::from_java(&env, device_id);
        let raw_result = daemon_interface.remove_device(token, device_id);

        if let Err(ref error) = &raw_result {
            log_request_error("remove device", error);
        }

        RemoveDeviceResult::from(raw_result)
    } else {
        RemoveDeviceResult::OtherError
    };

    result.into_java(&env).forget()
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
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_setDnsOptions(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    dnsOptions: JObject<'_>,
) {
    let env = JnixEnv::from(env);

    if let Some(daemon_interface) = get_daemon_interface(daemon_interface_address) {
        let dns_options = DnsOptions::from_java(&env, dnsOptions);

        if let Err(error) = daemon_interface.set_dns_options(dns_options) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to set custom DNS options")
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
        VoucherSubmissionResult::Error(VoucherSubmissionError::OtherError)
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

fn log_request_error(request: &str, error: &daemon_interface::Error) {
    match error {
        daemon_interface::Error::RpcError(RestError::Aborted) => {
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
