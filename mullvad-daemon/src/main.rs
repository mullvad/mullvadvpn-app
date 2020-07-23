#![deny(rust_2018_idioms)]

use log::{debug, error, info, warn};
use mullvad_daemon::{
    logging,
    management_interface::{ManagementInterfaceEventBroadcaster, ManagementInterfaceServer},
    rpc_uniqueness_check, version, Daemon, DaemonCommandChannel, DaemonCommandSender,
};
use std::{path::PathBuf, thread, time::Duration};
use talpid_types::ErrorExt;

mod cli;
mod exception_logging;
mod shutdown;
#[cfg(windows)]
mod system_service;

const DAEMON_LOG_FILENAME: &str = "daemon.log";

fn main() {
    let config = cli::get_config();
    let log_dir = init_logging(config).unwrap_or_else(|error| {
        eprintln!("{}", error);
        std::process::exit(1)
    });
    let exit_code = match run_platform(config, log_dir) {
        Ok(_) => 0,
        Err(error) => {
            error!("{}", error);
            1
        }
    };
    debug!("Process exiting with code {}", exit_code);
    std::process::exit(exit_code);
}

fn init_logging(config: &cli::Config) -> Result<Option<PathBuf>, String> {
    let log_dir = get_log_dir(config)?;
    let log_file = log_dir.as_ref().map(|dir| dir.join(DAEMON_LOG_FILENAME));

    logging::init_logger(
        config.log_level,
        log_file.as_ref(),
        config.log_stdout_timestamps,
    )
    .map_err(|e| e.display_chain_with_msg("Unable to initialize logger"))?;
    log_panics::init();
    exception_logging::enable();
    version::log_version();
    if let Some(ref log_dir) = log_dir {
        info!("Logging to {}", log_dir.display());
    }
    Ok(log_dir)
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

#[cfg(windows)]
fn run_platform(config: &cli::Config, log_dir: Option<PathBuf>) -> Result<(), String> {
    if config.run_as_service {
        system_service::run()
    } else {
        if config.register_service {
            let install_result = system_service::install_service().map_err(|e| e.display_chain());
            if install_result.is_ok() {
                println!("Installed the service.");
            }
            install_result
        } else {
            run_standalone(log_dir)
        }
    }
}

#[cfg(not(windows))]
fn run_platform(_config: &cli::Config, log_dir: Option<PathBuf>) -> Result<(), String> {
    run_standalone(log_dir)
}

fn run_standalone(log_dir: Option<PathBuf>) -> Result<(), String> {
    {
        let mut runtime = tokio02::runtime::Builder::new()
            .basic_scheduler()
            .build()
            .expect("Failed to create tokio runtime");
        if runtime.block_on(rpc_uniqueness_check::is_another_instance_running()) {
            return Err("Another instance of the daemon is already running".to_owned());
        }
    }

    if !running_as_admin() {
        warn!("Running daemon as a non-administrator user, clients might refuse to connect");
    }

    let daemon = create_daemon(log_dir)?;

    let shutdown_handle = daemon.shutdown_handle();
    shutdown::set_shutdown_signal_handler(move || shutdown_handle.shutdown())
        .map_err(|e| e.display_chain())?;

    daemon.run().map_err(|e| e.display_chain())?;

    info!("Mullvad daemon is quitting");
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

fn create_daemon(
    log_dir: Option<PathBuf>,
) -> Result<Daemon<ManagementInterfaceEventBroadcaster>, String> {
    let resource_dir = mullvad_paths::get_resource_dir();
    let settings_dir = mullvad_paths::settings_dir()
        .map_err(|e| e.display_chain_with_msg("Unable to get settings dir"))?;
    let cache_dir = mullvad_paths::cache_dir()
        .map_err(|e| e.display_chain_with_msg("Unable to get cache dir"))?;

    let command_channel = DaemonCommandChannel::new();
    let event_listener = spawn_management_interface(command_channel.sender())?;

    Daemon::start(
        log_dir,
        resource_dir,
        settings_dir,
        cache_dir,
        event_listener,
        command_channel,
    )
    .map_err(|e| e.display_chain_with_msg("Unable to initialize daemon"))
}

fn spawn_management_interface(
    command_sender: DaemonCommandSender,
) -> Result<ManagementInterfaceEventBroadcaster, String> {
    let server = ManagementInterfaceServer::start(command_sender).map_err(|error| {
        error.display_chain_with_msg("Unable to start management interface server")
    })?;
    let event_broadcaster = server.event_broadcaster();

    info!("Management interface listening on {}", server.socket_path());

    thread::spawn(|| {
        server.wait();
        info!("Management interface shut down");
    });

    Ok(event_broadcaster)
}

#[cfg(unix)]
fn running_as_admin() -> bool {
    let uid = unsafe { libc::getuid() };
    uid == 0
}

#[cfg(windows)]
fn running_as_admin() -> bool {
    // TODO: Check if user is administrator correctly on Windows.
    true
}
