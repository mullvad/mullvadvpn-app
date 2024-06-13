#![cfg(target_os = "android")]

mod classes;
mod is_null;
mod problem_report;
mod talpid_vpn_service;

use jnix::{
    jni::{objects::JObject, sys::jlong, JNIEnv},
    FromJava, JnixEnv,
};
use mullvad_daemon::{
    cleanup_old_rpc_socket, exception_logging, logging,
    management_interface::ManagementInterfaceServer, runtime::new_multi_thread, version, Daemon,
    DaemonCommandChannel, DaemonCommandSender,
};
use std::{
    io,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Once},
};
use talpid_types::{android::AndroidContext, ErrorExt};

#[cfg(feature = "api-override")]
use std::net::{IpAddr, SocketAddr};

const LOG_FILENAME: &str = "daemon.log";

static LOAD_CLASSES: Once = Once::new();
static LOG_START: Once = Once::new();
static mut LOG_INIT_RESULT: Option<Result<(), String>> = None;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create global reference to Java object")]
    CreateGlobalReference(#[source] jnix::jni::errors::Error),

    #[error("Failed to get Java VM instance")]
    GetJvmInstance(#[source] jnix::jni::errors::Error),

    #[error("Failed to initialize logging: {0}")]
    InitializeLogging(String),

    #[error("Failed to initialize the mullvad daemon")]
    InitializeDaemon(#[source] mullvad_daemon::Error),

    #[error("Mullvad daemon exited with an error")]
    DaemonFailed(#[source] mullvad_daemon::Error),

    #[error("Failed to spawn the tokio runtime")]
    InitializeTokioRuntime(#[source] io::Error),

    #[error("Failed to spawn the management interface")]
    SpawnManagementInterface(#[source] mullvad_daemon::management_interface::Error),
}

struct DaemonContext {
    daemon_command_tx: DaemonCommandSender,
    shutdown_complete_rx: mpsc::Receiver<()>,
}

/// Spawn Mullvad daemon. On success, a handle that can be passed to `MullvadDaemon.stop` is
/// returned. On error, an exception is thrown.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_start(
    env: JNIEnv<'_>,
    _this: JObject<'_>,
    vpnService: JObject<'_>,
    rpcSocketPath: JObject<'_>,
    filesDirectory: JObject<'_>,
    cacheDirectory: JObject<'_>,
    apiEndpoint: JObject<'_>,
) -> jlong {
    match start(
        env,
        vpnService,
        rpcSocketPath,
        filesDirectory,
        cacheDirectory,
        apiEndpoint,
    ) {
        Ok(daemon) => Box::into_raw(daemon) as jlong,
        Err(message) => {
            env.throw(message.to_string())
                .expect("Failed to throw exception");
            0
        }
    }
}

/// Shut down Mullvad daemon. The handle must be a valid handle returned by `MullvadDaemon.start`.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_stop(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    daemon_handle: jlong,
) {
    let pointer = daemon_handle as *mut DaemonContext;
    if pointer.is_null() {
        return;
    }

    // SAFETY: The caller promises that this is a valid pointer to a DaemonContext.
    let context: Box<DaemonContext> = unsafe { Box::from_raw(pointer) };

    context.daemon_command_tx.shutdown();
    _ = context.shutdown_complete_rx.recv();
}

fn start(
    env: JNIEnv<'_>,
    vpn_service: JObject<'_>,
    rpc_socket_path: JObject<'_>,
    files_directory: JObject<'_>,
    cache_directory: JObject<'_>,
    api_endpoint: JObject<'_>,
) -> Result<Box<DaemonContext>, Error> {
    let env = JnixEnv::from(env);

    LOAD_CLASSES.call_once(|| env.preload_classes(classes::CLASSES.iter().cloned()));

    let rpc_socket = PathBuf::from(String::from_java(&env, rpc_socket_path));
    let files_dir = PathBuf::from(String::from_java(&env, files_directory));
    let cache_dir = PathBuf::from(String::from_java(&env, cache_directory));

    start_logging(&files_dir).map_err(Error::InitializeLogging)?;
    version::log_version();

    let android_context = create_android_context(&env, vpn_service)?;

    #[cfg(feature = "api-override")]
    if !api_endpoint.is_null() {
        let api_endpoint = api_endpoint_from_java(&env, apiEndpoint);
        log::debug!("Overriding API endpoint: {api_endpoint:?}");
        if mullvad_api::API.override_init(api_endpoint).is_err() {
            log::warn!("Ignoring API settings (already initialized)");
        }
    }
    #[cfg(not(feature = "api-override"))]
    if !api_endpoint.is_null() {
        log::warn!("api_endpoint will be ignored since 'api-override' is not enabled");
    }

    run_daemon(android_context, rpc_socket, files_dir, cache_dir)
}

fn run_daemon(
    android_context: AndroidContext,
    rpc_socket: PathBuf,
    files_dir: PathBuf,
    cache_dir: PathBuf,
) -> Result<Box<DaemonContext>, Error> {
    let daemon_command_channel = DaemonCommandChannel::new();
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel();

    let context = DaemonContext {
        daemon_command_tx: daemon_command_channel.sender(),
        shutdown_complete_rx,
    };

    let (init_complete_tx, init_complete_rx) = mpsc::channel();

    let daemon_thread = std::thread::spawn(move || {
        let runtime = new_multi_thread()
            .build()
            .map_err(Error::InitializeTokioRuntime)?;

        runtime.block_on(run_daemon_inner(
            rpc_socket,
            files_dir,
            cache_dir,
            daemon_command_channel,
            android_context,
            init_complete_tx,
        ))?;

        _ = shutdown_complete_tx.send(());

        Ok::<(), Error>(())
    });

    // Do this silly maneuver instead of moving `daemon.run()` into `tokio::spawn()`,
    // since the latter requires that futures be `Send`, and `Daemon` isn't sendable.
    if init_complete_rx.recv().is_err() {
        return Err(daemon_thread.join().expect("thread panicked").unwrap_err());
    }

    Ok(Box::new(context))
}

async fn run_daemon_inner(
    rpc_socket: PathBuf,
    files_dir: PathBuf,
    cache_dir: PathBuf,
    command_channel: DaemonCommandChannel,
    android_context: AndroidContext,
    init_complete_tx: mpsc::Sender<()>,
) -> Result<(), Error> {
    cleanup_old_rpc_socket(&rpc_socket).await;

    let event_listener = ManagementInterfaceServer::start(command_channel.sender(), &rpc_socket)
        .map_err(Error::SpawnManagementInterface)?;

    log::info!("Management interface listening on {}", rpc_socket.display());

    let daemon = Daemon::start(
        Some(files_dir.clone()),
        files_dir.clone(),
        files_dir,
        cache_dir,
        event_listener,
        command_channel,
        android_context,
    )
    .await
    .map_err(Error::InitializeDaemon)?;

    _ = init_complete_tx.send(());

    daemon.run().await.map_err(Error::DaemonFailed)?;

    log::info!("Mullvad daemon has stopped");

    Ok(())
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
