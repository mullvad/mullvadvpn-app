#![cfg(target_os = "android")]

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
    device,
    exception_logging,
    logging,
    runtime::new_runtime_builder,
    version,
    Daemon,
    DaemonCommandChannel,
    settings::patch::Error as PatchError,
};
use mullvad_types::{
    account::{AccountData, PlayPurchase, VoucherSubmission},
    custom_list::CustomList,
    settings::DnsOptions,
    relay_constraints::RelayOverride,
};
use std::{
    io,
    path::{Path, PathBuf},
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create global reference to Java object")]
    CreateGlobalReference(#[source] jnix::jni::errors::Error),

    #[error("Failed to get Java VM instance")]
    GetJvmInstance(#[source] jnix::jni::errors::Error),

    #[error("Failed to initialize the mullvad daemon")]
    InitializeDaemon(#[source] mullvad_daemon::Error),

    #[error("Failed to spawn the tokio runtime")]
    InitializeTokioRuntime(#[source] io::Error),

    #[error("Failed to spawn the JNI event listener")]
    SpawnJniEventListener(#[source] jni_event_listener::Error),
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
                daemon_interface::Error::Api(RestError::ApiError(status, _code))
                    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN =>
                {
                    GetAccountDataResult::InvalidAccount
                }
                daemon_interface::Error::Api(_) => GetAccountDataResult::RpcError,
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
                daemon_interface::Error::Other(mullvad_daemon::Error::LoginError(error)) => {
                    match error {
                        device::Error::InvalidAccount => LoginResult::InvalidAccount,
                        device::Error::MaxDevicesReached => LoginResult::MaxDevicesReached,
                        device::Error::OtherRestError(_) => LoginResult::RpcError,
                        _ => LoginResult::OtherError,
                    }
                }
                daemon_interface::Error::Api(_) => LoginResult::RpcError,
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
                daemon_interface::Error::Other(mullvad_daemon::Error::LoginError(error)) => {
                    match error {
                        device::Error::InvalidAccount => RemoveDeviceResult::RpcError,
                        device::Error::InvalidDevice => RemoveDeviceResult::NotFound,
                        device::Error::OtherRestError(_) => RemoveDeviceResult::RpcError,
                        _ => RemoveDeviceResult::OtherError,
                    }
                }
                daemon_interface::Error::Api(_) => RemoveDeviceResult::RpcError,
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
            daemon_interface::Error::Other(mullvad_daemon::Error::VoucherSubmission(
                device::Error::InvalidVoucher,
            )) => VoucherSubmissionError::InvalidVoucher,
            daemon_interface::Error::Other(mullvad_daemon::Error::VoucherSubmission(
                device::Error::UsedVoucher,
            )) => VoucherSubmissionError::VoucherAlreadyUsed,
            _ => VoucherSubmissionError::OtherError,
        }
    }
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum SettingsPatchError {
    InvalidOrMissingValue(String),
    UnknownOrProhibitedKey(String),
    ParsePatch,
    DeserializePatched,
    RecursionLimit,
    ApplyPatch,
}

impl From<daemon_interface::Error> for SettingsPatchError {
    fn from(error: daemon_interface::Error) -> Self {
        match error {
            daemon_interface::Error::Patch(PatchError::InvalidOrMissingValue(str))
                => SettingsPatchError::InvalidOrMissingValue(str.to_string()),
            daemon_interface::Error::Patch(PatchError::UnknownOrProhibitedKey(string))
                => SettingsPatchError::UnknownOrProhibitedKey(string),
            daemon_interface::Error::Patch(PatchError::ParsePatch(_))
                => SettingsPatchError::ParsePatch,
            daemon_interface::Error::Patch(PatchError::DeserializePatched(_))
                => SettingsPatchError::DeserializePatched,
            daemon_interface::Error::Patch(PatchError::SerializeSettings(_))
                => SettingsPatchError::ApplyPatch,
            daemon_interface::Error::Patch(PatchError::SerializeValue(_))
                => SettingsPatchError::ApplyPatch,
            daemon_interface::Error::Patch(PatchError::RecursionLimit)
                => SettingsPatchError::RecursionLimit,
            _ => SettingsPatchError::ApplyPatch,
        }
    }
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum PlayPurchaseInitResult {
    Ok(String),
    Error(PlayPurchaseInitError),
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum PlayPurchaseInitError {
    OtherError,
}

impl From<Result<String, daemon_interface::Error>> for PlayPurchaseInitResult {
    fn from(result: Result<String, daemon_interface::Error>) -> Self {
        match result {
            Ok(obfuscated_id) => PlayPurchaseInitResult::Ok(obfuscated_id),
            Err(error) => PlayPurchaseInitResult::Error(error.into()),
        }
    }
}

impl From<daemon_interface::Error> for PlayPurchaseInitError {
    fn from(_error: daemon_interface::Error) -> Self {
        PlayPurchaseInitError::OtherError
    }
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum PlayPurchaseVerifyResult {
    Ok,
    Error(PlayPurchaseVerifyError),
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum PlayPurchaseVerifyError {
    OtherError,
}

impl From<Result<(), daemon_interface::Error>> for PlayPurchaseVerifyResult {
    fn from(result: Result<(), daemon_interface::Error>) -> Self {
        match result {
            Ok(()) => PlayPurchaseVerifyResult::Ok,
            Err(error) => PlayPurchaseVerifyResult::Error(error.into()),
        }
    }
}

impl From<daemon_interface::Error> for PlayPurchaseVerifyError {
    fn from(_error: daemon_interface::Error) -> Self {
        PlayPurchaseVerifyError::OtherError
    }
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum CreateCustomListResult {
    Ok(mullvad_types::custom_list::Id),
    Error(CustomListsError),
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum UpdateCustomListResult {
    Ok,
    Error(CustomListsError),
}

#[derive(IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
pub enum CustomListsError {
    CustomListExists,
    OtherError,
}

impl From<Result<mullvad_types::custom_list::Id, daemon_interface::Error>>
    for CreateCustomListResult
{
    fn from(result: Result<mullvad_types::custom_list::Id, daemon_interface::Error>) -> Self {
        match result {
            Ok(id) => CreateCustomListResult::Ok(id),
            Err(error) => CreateCustomListResult::Error(error.into()),
        }
    }
}

impl From<Result<(), daemon_interface::Error>> for UpdateCustomListResult {
    fn from(result: Result<(), daemon_interface::Error>) -> Self {
        match result {
            Ok(()) => UpdateCustomListResult::Ok,
            Err(error) => UpdateCustomListResult::Error(error.into()),
        }
    }
}

impl From<daemon_interface::Error> for CustomListsError {
    fn from(_error: daemon_interface::Error) -> Self {
        CustomListsError::CustomListExists
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

    endpoint.address = Some(
        try_socketaddr_from_java(env, address).expect("received unresolved InetSocketAddress"),
    );
    endpoint.host = try_hostname_from_java(env, address);
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

/// Returns the hostname for an InetSocketAddress. This may be an IP address converted to
/// a string.
#[cfg(feature = "api-override")]
fn try_hostname_from_java(env: &JnixEnv<'_>, address: JObject<'_>) -> Option<String> {
    let class = env.get_class("java/net/InetSocketAddress");

    let method_id = env
        .get_method_id(&class, "getHostString", "()Ljava/lang/String;")
        .expect("Failed to get method ID for InetSocketAddress.getHostString()");
    let return_type = JavaType::Object("java/lang/String".to_owned());

    let hostname = env
        .call_method_unchecked(address, method_id, return_type, &[])
        .expect("Failed to call InetSocketAddress.getHostString()")
        .l()
        .expect("Call to InetSocketAddress.getHostString() did not return an object");

    if hostname.is_null() {
        return None;
    }

    Some(String::from_java(env, hostname))
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

    if !daemon_interface_address.is_null() {
        let _ = unsafe { Box::from_raw(daemon_interface_address) };
    }
}

/// # Safety
///
/// `address` must either be zero or a valid pointer to a `DaemonInterface` instance.
/// This function has no concept of lifetimes, so the caller must ensure that the
/// pointed to `DaemonInterface` is valid for at least as long as the return value
/// of this function is still alive.
unsafe fn get_daemon_interface(address: jlong) -> Option<&'static mut DaemonInterface> {
    let pointer = address as *mut DaemonInterface;

    if !pointer.is_null() {
        Some(&mut *pointer)
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
    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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
    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    match unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_getCurrentVersion<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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
    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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
    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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
    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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
    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    let result =
        // SAFETY: The address points to an instance valid for the duration of this function call
        if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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
    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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

    let result =
        // SAFETY: The address points to an instance valid for the duration of this function call
        if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
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
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_initPlayPurchase<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    let result =
        // SAFETY: The address points to an instance valid for the duration of this function call
        if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
            let raw_result = daemon_interface.init_play_purchase();

            if let Err(ref error) = &raw_result {
                log_request_error("init google play purchase", error);
            }

            PlayPurchaseInitResult::from(raw_result)
        } else {
            PlayPurchaseInitResult::Error(PlayPurchaseInitError::OtherError)
        };

    result.into_java(&env).forget()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_verifyPlayPurchase<
    'env,
>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    play_purchase: JObject<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    let result =
        // SAFETY: The address points to an instance valid for the duration of this function call
        if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
            let play_purchase = PlayPurchase::from_java(&env, play_purchase);
            let raw_result = daemon_interface.verify_play_purchase(play_purchase);

            if let Err(ref error) = &raw_result {
                log_request_error("verify google play purchase", error);
            }

            PlayPurchaseVerifyResult::from(raw_result)
        } else {
            PlayPurchaseVerifyResult::Error(PlayPurchaseVerifyError::OtherError)
        };

    result.into_java(&env).forget()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_setRelaySettings(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    relaySettings: JObject<'_>,
) {
    let env = JnixEnv::from(env);

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
        let update = FromJava::from_java(&env, relaySettings);

        if let Err(error) = daemon_interface.set_relay_settings(update) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to update relay settings")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_setObfuscationSettings(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    obfuscationSettings: JObject<'_>,
) {
    let env = JnixEnv::from(env);

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
        let settings = FromJava::from_java(&env, obfuscationSettings);

        if let Err(error) = daemon_interface.set_obfuscation_settings(settings) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to update obfuscation settings")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_setQuantumResistantTunnel(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    quantumResistantState: JObject<'_>,
) {
    let env = JnixEnv::from(env);

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
        let quantum_resistant = FromJava::from_java(&env, quantumResistantState);

        if let Err(error) = daemon_interface.set_quantum_resistant_tunnel(quantum_resistant) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to update quantum resistant tunnel state")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_createCustomList<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    name: JString<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
        let name = String::from_java(&env, name);
        let raw_result = daemon_interface.create_custom_list(name);

        if let Err(ref error) = &raw_result {
            log_request_error("Failed to create custom list", error);
        }

        CreateCustomListResult::from(raw_result)
            .into_java(&env)
            .forget()
    } else {
        CreateCustomListResult::Error(CustomListsError::OtherError)
            .into_java(&env)
            .forget()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_deleteCustomList(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    id: JString<'_>,
) {
    let env = JnixEnv::from(env);

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
        let id = mullvad_types::custom_list::Id::from_java(&env, id);
        if let Err(error) = daemon_interface.delete_custom_list(id) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to delete custom list")
            );
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_updateCustomList<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    customList: JObject<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
        let list = CustomList::from_java(&env, customList);
        let raw_result = daemon_interface.update_custom_list(list);

        if let Err(ref error) = &raw_result {
            log_request_error("Failed to update custom list", error);
        }

        UpdateCustomListResult::from(raw_result)
            .into_java(&env)
            .forget()
    } else {
        UpdateCustomListResult::Error(CustomListsError::OtherError)
            .into_java(&env)
            .forget()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_applyJsonSettings<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    json: JString<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);


    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
        let jsonSettings = String::from_java(&env, json);
        match daemon_interface.apply_json_settings(jsonSettings) {
            Ok(()) => JObject::null(),
            Err(error) => {
                log_request_error("apply json settings", &error);
                SettingsPatchError::from(error).into_java(&env).forget()
            }
        }
    } else {
        log::warn!("Daemon was unreachable");
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_exportJsonSettings<'env>(
    env: JNIEnv<'env>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    _: JObject<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
        match daemon_interface.export_json_settings() {
            Ok(exported_json) => exported_json.into_java(&env).forget(),
            Err(error) => {
                log_request_error("export json settings", &error);
                JObject::null()
            }
        }
    } else {
        log::warn!("Daemon was unreachable");
        JObject::null()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_setRelayOverride(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    relay_override: JObject<'_>,
) {
    let env = JnixEnv::from(env);


    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
        let r_override = RelayOverride::from_java(&env, relay_override);

        match daemon_interface.set_relay_override(r_override) {
            Ok(()) => (),
            Err(error) => {
                log_request_error("set relay override", &error);
            }
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_clearAllRelayOverrides(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_interface_address: jlong,
    _: JObject<'_>,
) {

    // SAFETY: The address points to an instance valid for the duration of this function call
    if let Some(daemon_interface) = unsafe { get_daemon_interface(daemon_interface_address) } {
        match daemon_interface.clear_all_relay_overrides() {
            Ok(()) => (),
            Err(error) => {
                log_request_error("clear all relay overrides", &error);
            }
        }
    }
}


fn log_request_error(request: &str, error: &daemon_interface::Error) {
    match error {
        daemon_interface::Error::Api(RestError::Aborted) => {
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
