#![cfg(target_os = "android")]

mod api;
mod classes;
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
    sync::{mpsc, Arc, Once, OnceLock},
};
use talpid_types::{android::AndroidContext, ErrorExt};

const LOG_FILENAME: &str = "daemon.log";

static LOAD_CLASSES: Once = Once::new();

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

    #[error("Failed to init Tokio runtime")]
    InitTokio(#[source] io::Error),

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

    _ = context.daemon_command_tx.shutdown();
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
        let api_endpoint = api::api_endpoint_from_java(&env, api_endpoint);
        log::debug!("Overriding API endpoint: {api_endpoint:?}");
        if mullvad_api::API.override_init(api_endpoint).is_err() {
            log::warn!("Ignoring API settings (already initialized)");
        }
    }
    #[cfg(not(feature = "api-override"))]
    if !api_endpoint.is_null() {
        log::warn!("api_endpoint will be ignored since 'api-override' is not enabled");
    }

    spawn_daemon(android_context, rpc_socket, files_dir, cache_dir)
}

fn spawn_daemon(
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

    let runtime = new_multi_thread().build().map_err(Error::InitTokio)?;

    runtime.block_on(spawn_daemon_inner(
        rpc_socket,
        files_dir,
        cache_dir,
        daemon_command_channel,
        android_context,
    ))?;

    // Spawn a thread to keep the tokio runtime alive
    std::thread::spawn(move || {
        drop(runtime);
        _ = shutdown_complete_tx.send(());
        Ok::<(), Error>(())
    });

    Ok(Box::new(context))
}

async fn spawn_daemon_inner(
    rpc_socket: PathBuf,
    files_dir: PathBuf,
    cache_dir: PathBuf,
    command_channel: DaemonCommandChannel,
    android_context: AndroidContext,
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

    tokio::spawn(async move {
        match daemon.run().await {
            Ok(()) => log::info!("Mullvad daemon has stopped"),
            Err(error) => log::error!(
                "{}",
                error.display_chain_with_msg("Mullvad daemon exited with an error")
            ),
        }
    });

    Ok(())
}

fn start_logging(log_dir: &Path) -> Result<(), String> {
    static LOGGER_RESULT: OnceLock<Result<(), String>> = OnceLock::new();
    LOGGER_RESULT
        .get_or_init(|| start_logging_inner(log_dir).map_err(|e| e.display_chain()))
        .to_owned()
}

fn start_logging_inner(log_dir: &Path) -> Result<(), logging::Error> {
    let log_file = log_dir.join(LOG_FILENAME);

    logging::init_logger(log::LevelFilter::Debug, Some(&log_file), true)?;
    exception_logging::enable();
    log_panics::init();

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
