#![cfg(target_os = "android")]

mod api;
mod classes;
mod problem_report;

use jnix::{
    jni::{
        objects::{JClass, JObject},
        JNIEnv,
    },
    FromJava, JnixEnv,
};
use mullvad_api::ApiEndpoint;
use mullvad_daemon::{
    cleanup_old_rpc_socket, exception_logging, logging, runtime::new_multi_thread, version, Daemon,
    DaemonCommandChannel, DaemonCommandSender, DaemonConfig,
};
use std::{
    ffi::CString,
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Once, OnceLock},
};
use talpid_types::{android::AndroidContext, ErrorExt};

const LOG_FILENAME: &str = "daemon.log";

/// Mullvad daemon instance. It must be initialized and destroyed by `MullvadDaemon.initialize` and
/// `MullvadDaemon.shutdown`, respectively.
static DAEMON_CONTEXT: Mutex<Option<DaemonContext>> = Mutex::new(None);

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
}

/// Throw a Java exception and return if `result` is an error
macro_rules! ok_or_throw {
    ($env:expr, $result:expr) => {{
        match $result {
            Ok(val) => val,
            Err(err) => {
                let env = $env;
                env.throw(err.to_string())
                    .expect("Failed to throw exception");
                return;
            }
        }
    }};
}

#[derive(Debug)]
struct DaemonContext {
    runtime: tokio::runtime::Runtime,
    daemon_command_tx: DaemonCommandSender,
    running_daemon: tokio::task::JoinHandle<()>,
}

fn use_after_free() -> i32 {
    let boxed_int = Box::new(10);
    let ptr = boxed_int.as_ref() as *const i32;
    drop(boxed_int);
    unsafe { *ptr }
}

/// Spawn Mullvad daemon. There can only be a single instance, which must be shut down using
/// `MullvadDaemon.shutdown`. On success, nothing is returned. On error, an exception is thrown.
#[unsafe(no_mangle)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_initialize(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
    vpn_service: JObject<'_>,
    rpc_socket_path: JObject<'_>,
    files_directory: JObject<'_>,
    cache_directory: JObject<'_>,
    api_endpoint: JObject<'_>,
) {
    let mut ctx = DAEMON_CONTEXT.lock().unwrap();
    assert!(ctx.is_none(), "multiple calls to MullvadDaemon.initialize");

    let env = JnixEnv::from(env);
    let files_dir = pathbuf_from_java(&env, files_directory);
    start_logging(&files_dir)
        .map_err(Error::InitializeLogging)
        .unwrap();
    version::log_version();

    log::info!("Pre-loading classes!");
    LOAD_CLASSES.call_once(|| env.preload_classes(classes::CLASSES.iter().cloned()));
    log::info!("Done loading classes");

    let rpc_socket = pathbuf_from_java(&env, rpc_socket_path);
    let cache_dir = pathbuf_from_java(&env, cache_directory);

    let android_context = ok_or_throw!(&env, create_android_context(&env, vpn_service));
    log::info!("Created Android Context");

    let api_endpoint = api::api_endpoint_from_java(&env, api_endpoint);

    use_after_free();

    log::info!("Starting daemon");
    let daemon = ok_or_throw!(
        &env,
        start(
            android_context,
            rpc_socket,
            files_dir,
            cache_dir,
            api_endpoint,
        )
    );

    *ctx = Some(daemon);
}

/// Shut down Mullvad daemon that was initialized using `MullvadDaemon.initialize`.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_service_MullvadDaemon_shutdown(
    _: JNIEnv<'_>,
    _class: JClass<'_>,
) {
    if let Some(context) = DAEMON_CONTEXT.lock().unwrap().take() {
        _ = context.daemon_command_tx.shutdown();
        _ = context.runtime.block_on(context.running_daemon);

        // Dropping the tokio runtime will block if there are any tasks in flight.
        // That is, until all async tasks yield *and* all blocking threads have stopped.
    }
}

fn start(
    android_context: AndroidContext,
    rpc_socket: PathBuf,
    files_dir: PathBuf,
    cache_dir: PathBuf,
    api_endpoint: Option<ApiEndpoint>,
) -> Result<DaemonContext, Error> {
    #[cfg(not(feature = "api-override"))]
    if api_endpoint.is_some() {
        log::warn!("api_endpoint will be ignored since 'api-override' is not enabled");
    }

    spawn_daemon(
        android_context,
        rpc_socket,
        files_dir,
        cache_dir,
        api_endpoint.unwrap_or(ApiEndpoint::from_env_vars()),
    )
}

fn spawn_daemon(
    android_context: AndroidContext,
    rpc_socket: PathBuf,
    files_dir: PathBuf,
    cache_dir: PathBuf,
    endpoint: ApiEndpoint,
) -> Result<DaemonContext, Error> {
    let daemon_command_channel = DaemonCommandChannel::new();
    let daemon_command_tx = daemon_command_channel.sender();

    let runtime = new_multi_thread().build().map_err(Error::InitTokio)?;

    let daemon_config = DaemonConfig {
        rpc_socket_path: rpc_socket,
        log_dir: Some(files_dir.clone()),
        resource_dir: files_dir.clone(),
        settings_dir: files_dir,
        cache_dir,
        android_context,
        endpoint,
    };

    let running_daemon =
        runtime.block_on(spawn_daemon_inner(daemon_config, daemon_command_channel))?;

    Ok(DaemonContext {
        runtime,
        daemon_command_tx,
        running_daemon,
    })
}

async fn spawn_daemon_inner(
    daemon_config: DaemonConfig,
    daemon_command_channel: DaemonCommandChannel,
) -> Result<tokio::task::JoinHandle<()>, Error> {
    cleanup_old_rpc_socket(&daemon_config.rpc_socket_path).await;

    let daemon = Daemon::start(daemon_config, daemon_command_channel)
        .await
        .map_err(Error::InitializeDaemon)?;

    let running_daemon = tokio::spawn(async move {
        match daemon.run().await {
            Ok(()) => log::info!("Mullvad daemon has stopped"),
            Err(error) => log::error!(
                "{}",
                error.display_chain_with_msg("Mullvad daemon exited with an error")
            ),
        }
    });

    Ok(running_daemon)
}

fn start_logging(log_dir: &Path) -> Result<(), String> {
    static LOGGER_RESULT: OnceLock<Result<(), String>> = OnceLock::new();
    LOGGER_RESULT
        .get_or_init(|| start_logging_inner(log_dir))
        .to_owned()
}

fn start_logging_inner(log_dir: &Path) -> Result<(), String> {
    let log_file = log_dir.join(LOG_FILENAME);

    logging::init_logger(log::LevelFilter::Debug, Some(&log_file), true)
        .map_err(|e| e.display_chain())?;
    log_panics::init();
    exception_logging::set_log_file(
        CString::new(log_file.as_os_str().as_bytes())
            .map_err(|_| "Log file path contained interior null bytes: {log_file:?}")?,
    );
    exception_logging::enable();

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

fn pathbuf_from_java(env: &JnixEnv<'_>, path: JObject<'_>) -> PathBuf {
    PathBuf::from(String::from_java(env, path))
}
