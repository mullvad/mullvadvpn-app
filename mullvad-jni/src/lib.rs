#![cfg(target_os = "android")]

mod classes;
mod daemon_interface;
mod is_null;
mod problem_report;
mod talpid_vpn_service;

use crate::daemon_interface::DaemonInterface;
use jnix::{
    jni::{
        objects::{GlobalRef, JObject, JValue},
        signature::{JavaType, Primitive},
        sys::jlong,
        JNIEnv, JavaVM,
    },
    FromJava, JnixEnv,
};
use mullvad_daemon::{
    cleanup_old_rpc_socket, exception_logging, logging, runtime::new_multi_thread, version, Daemon,
    DaemonCommandChannel,
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

    #[error("Failed to spawn the management interface")]
    SpawnManagementInterface(#[source] mullvad_daemon::management_interface::Error),
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_initialize(
    env: JNIEnv<'_>,
    this: JObject<'_>,
    vpnService: JObject<'_>,
    rpcSocketPath: JObject<'_>,
    filesDirectory: JObject<'_>,
    cacheDirectory: JObject<'_>,
    apiEndpoint: JObject<'_>,
) {
    let env = JnixEnv::from(env);
    let rpc_socket = PathBuf::from(String::from_java(&env, rpcSocketPath));
    let files_dir = PathBuf::from(String::from_java(&env, filesDirectory));
    let cache_dir = PathBuf::from(String::from_java(&env, cacheDirectory));

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

    match start_logging(&files_dir) {
        Ok(()) => {
            version::log_version();

            LOAD_CLASSES.call_once(|| env.preload_classes(classes::CLASSES.iter().cloned()));

            if let Err(error) = initialize(
                &env,
                &this,
                &vpnService,
                rpc_socket,
                files_dir,
                cache_dir,
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
    rpc_socket: PathBuf,
    files_dir: PathBuf,
    cache_dir: PathBuf,
    api_endpoint: Option<mullvad_api::ApiEndpoint>,
) -> Result<(), Error> {
    let android_context = create_android_context(env, *vpn_service)?;
    let daemon_command_channel = DaemonCommandChannel::new();
    let daemon_interface = Box::new(DaemonInterface::new(daemon_command_channel.sender()));

    spawn_daemon(
        env,
        this,
        rpc_socket,
        files_dir,
        cache_dir,
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

#[allow(clippy::too_many_arguments)]
fn spawn_daemon(
    env: &JnixEnv<'_>,
    this: &JObject<'_>,
    rpc_socket: PathBuf,
    files_dir: PathBuf,
    cache_dir: PathBuf,
    #[cfg_attr(not(feature = "api-override"), allow(unused_variables))] api_endpoint: Option<
        mullvad_api::ApiEndpoint,
    >,
    command_channel: DaemonCommandChannel,
    android_context: AndroidContext,
) -> Result<(), Error> {
    let daemon_object = env
        .new_global_ref(*this)
        .map_err(Error::CreateGlobalReference)?;
    let (tx, rx) = mpsc::channel();

    let runtime = new_multi_thread()
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

        runtime.block_on(cleanup_old_rpc_socket(&rpc_socket));

        let event_listener = match runtime
            .block_on(async { spawn_management_interface(command_channel.sender(), &rpc_socket) })
        {
            Ok(event_listener) => event_listener,
            Err(error) => {
                let _ = tx.send(Err(error));
                return;
            }
        };

        let daemon = runtime.block_on(Daemon::start(
            Some(files_dir.clone()),
            files_dir.clone(),
            files_dir,
            cache_dir,
            event_listener,
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

use mullvad_daemon::{
    management_interface::{ManagementInterfaceEventBroadcaster, ManagementInterfaceServer},
    DaemonCommandSender,
};

fn spawn_management_interface(
    command_sender: DaemonCommandSender,
    rpc_socket_path: impl AsRef<Path>,
) -> Result<ManagementInterfaceEventBroadcaster, Error> {
    let event_broadcaster = ManagementInterfaceServer::start(command_sender, &rpc_socket_path)
        .map_err(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg("Unable to start management interface server")
            );
            Error::SpawnManagementInterface(error)
        })?;

    log::info!(
        "Management interface listening on {}",
        rpc_socket_path.as_ref().display()
    );

    Ok(event_broadcaster)
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
