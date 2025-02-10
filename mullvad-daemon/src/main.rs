use std::{path::PathBuf, thread, time::Duration};

#[cfg(not(windows))]
use mullvad_daemon::cleanup_old_rpc_socket;
use mullvad_daemon::{
    exception_logging, logging, rpc_uniqueness_check, runtime, version, Daemon,
    DaemonCommandChannel, DaemonConfig,
};
use talpid_types::ErrorExt;

mod cli;
#[cfg(target_os = "linux")]
mod early_boot_firewall;
#[cfg(target_os = "macos")]
mod macos_launch_daemon;
#[cfg(windows)]
mod system_service;

const DAEMON_LOG_FILENAME: &str = "daemon.log";
#[cfg(target_os = "linux")]
const EARLY_BOOT_LOG_FILENAME: &str = "early-boot-fw.log";

fn main() {
    let runtime = new_runtime();
    let exit_code = match runtime.block_on(run()) {
        Ok(_) => 0,
        Err(error) => {
            if logging::is_enabled() {
                log::error!("{error}");
            } else {
                eprintln!("{error}")
            }

            1
        }
    };

    log::debug!("Process exiting with code {}", exit_code);
    std::process::exit(exit_code);
}

fn new_runtime() -> tokio::runtime::Runtime {
    let mut builder = match cli::get_config().command {
        #[cfg(target_os = "windows")]
        cli::Command::RunAsService | cli::Command::RegisterService => runtime::new_current_thread(),
        _ => runtime::new_multi_thread(),
    };

    builder.build().unwrap_or_else(|e| {
        eprintln!("{}", e.display_chain());
        std::process::exit(1);
    })
}

async fn run() -> Result<(), String> {
    let config = cli::get_config();

    match config.command {
        cli::Command::Daemon => {
            // uniqueness check must happen before logging initializaton,
            // as initializing logs will rotate any existing log file.
            assert_unique().await?;
            let log_dir = init_daemon_logging(config)?;
            log::trace!("Using configuration: {:?}", config);

            run_standalone(log_dir).await
        }

        #[cfg(target_os = "linux")]
        cli::Command::InitializeEarlyBootFirewall => {
            init_early_boot_logging(config);

            crate::early_boot_firewall::initialize_firewall()
                .await
                .map_err(|err| format!("{err}"))
        }

        #[cfg(target_os = "windows")]
        cli::Command::RunAsService => {
            assert_unique().await?;
            let _ = init_daemon_logging(config)?;
            system_service::run()
        }

        #[cfg(target_os = "windows")]
        cli::Command::RegisterService => {
            init_logger(config, None)?;
            system_service::install_service()
                .inspect(|_| println!("Installed the service."))
                .map_err(|e| e.display_chain())
        }

        #[cfg(target_os = "macos")]
        cli::Command::LaunchDaemonStatus => {
            if version::is_dev_version() {
                eprintln!("Note: This command may not work on non-notarized builds.");
            }

            std::process::exit(macos_launch_daemon::get_status() as i32);
        }
    }
}

/// Check that there's not another daemon currently running.
async fn assert_unique() -> Result<(), &'static str> {
    if rpc_uniqueness_check::is_another_instance_running().await {
        return Err("Another instance of the daemon is already running");
    }
    Ok(())
}

/// Initialize logging to stderr and to file (if configured).
fn init_daemon_logging(config: &cli::Config) -> Result<Option<PathBuf>, String> {
    let log_dir = get_log_dir(config)?;
    let log_path = |filename| log_dir.as_ref().map(|dir| dir.join(filename));

    init_logger(config, log_path(DAEMON_LOG_FILENAME))?;

    if let Some(ref log_dir) = log_dir {
        log::info!("Logging to {}", log_dir.display());
    }
    Ok(log_dir)
}

/// Initialize logging to stder and to the [`EARLY_BOOT_LOG_FILENAME`]
#[cfg(target_os = "linux")]
fn init_early_boot_logging(config: &cli::Config) {
    // If it's possible to log to the filesystem - attempt to do so, but failing that mustn't stop
    // the daemon from starting here.
    if let Ok(Some(log_dir)) = get_log_dir(config) {
        if init_logger(config, Some(log_dir.join(EARLY_BOOT_LOG_FILENAME))).is_ok() {
            return;
        }
    }

    let _ = init_logger(config, None);
}

/// Initialize logging to stderr and to file (if provided).
///
/// Also install the [exception_logging] signal handler to log faults.
fn init_logger(config: &cli::Config, log_file: Option<PathBuf>) -> Result<(), String> {
    #[cfg(unix)]
    if let Some(log_file) = &log_file {
        use std::os::unix::ffi::OsStrExt;

        exception_logging::set_log_file(
            std::ffi::CString::new(log_file.as_os_str().as_bytes())
                .map_err(|_| "Log file path contains null-bytes".to_string())?,
        );
    }

    exception_logging::enable();

    logging::init_logger(
        config.log_level,
        log_file.as_ref(),
        config.log_stdout_timestamps,
    )
    .map_err(|e| e.display_chain_with_msg("Unable to initialize logger"))?;
    log_panics::init();
    version::log_version();
    Ok(())
}

fn get_log_dir(config: &cli::Config) -> Result<Option<PathBuf>, String> {
    if config.log_to_file {
        Ok(Some(mullvad_paths::log_dir().map_err(|e| {
            e.display_chain_with_msg("Unable to get log directory")
        })?))
    } else {
        Ok(None)
    }
}

async fn run_standalone(log_dir: Option<PathBuf>) -> Result<(), String> {
    #[cfg(not(windows))]
    cleanup_old_rpc_socket(mullvad_paths::get_rpc_socket_path()).await;

    if !running_as_admin() {
        log::warn!("Running daemon as a non-administrator user, clients might refuse to connect");
    }

    let daemon = create_daemon(log_dir).await?;

    let shutdown_handle = daemon.shutdown_handle();
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    mullvad_daemon::shutdown::set_shutdown_signal_handler(move || {
        shutdown_handle.shutdown(!mullvad_daemon::shutdown::is_shutdown_user_initiated())
    })
    .map_err(|e| e.display_chain())?;

    #[cfg(any(windows, target_os = "android"))]
    mullvad_daemon::shutdown::set_shutdown_signal_handler(move || shutdown_handle.shutdown(true))
        .map_err(|e| e.display_chain())?;

    daemon.run().await.map_err(|e| e.display_chain())?;

    #[cfg(not(windows))]
    cleanup_old_rpc_socket(mullvad_paths::get_rpc_socket_path()).await;

    log::info!("Mullvad daemon is quitting");
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

async fn create_daemon(log_dir: Option<PathBuf>) -> Result<Daemon, String> {
    let rpc_socket_path = mullvad_paths::get_rpc_socket_path();
    let resource_dir = mullvad_paths::get_resource_dir();
    let settings_dir = mullvad_paths::settings_dir()
        .map_err(|e| e.display_chain_with_msg("Unable to get settings dir"))?;
    let cache_dir = mullvad_paths::cache_dir()
        .map_err(|e| e.display_chain_with_msg("Unable to get cache dir"))?;

    Daemon::start(
        DaemonConfig {
            log_dir,
            resource_dir,
            settings_dir,
            cache_dir,
            rpc_socket_path,
            endpoint: mullvad_api::ApiEndpoint::from_env_vars(),
        },
        DaemonCommandChannel::new(),
    )
    .await
    .map_err(|e| e.display_chain_with_msg("Unable to initialize daemon"))
}

#[cfg(unix)]
fn running_as_admin() -> bool {
    nix::unistd::Uid::current().is_root()
}

#[cfg(windows)]
fn running_as_admin() -> bool {
    // TODO: Check if user is administrator correctly on Windows.
    true
}
