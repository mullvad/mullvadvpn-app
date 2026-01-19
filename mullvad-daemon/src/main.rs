#[cfg(not(target_os = "android"))]
pub(crate) mod cli;

#[cfg(target_os = "linux")]
pub(crate) mod early_boot_firewall;

#[cfg(target_os = "macos")]
pub(crate) mod macos_launch_daemon;

#[cfg(windows)]
pub(crate) mod system_service;

fn main() {
    #[cfg(not(target_os = "android"))]
    imp::main()
}

#[cfg(not(target_os = "android"))]
mod imp {
    use std::{path::PathBuf, thread, time::Duration};

    #[cfg(not(windows))]
    use mullvad_daemon::cleanup_old_rpc_socket;

    use mullvad_daemon::{
        Daemon, DaemonCommandChannel, DaemonConfig, exception_logging,
        logging::{self, LogLocation},
        rpc_uniqueness_check, runtime, version,
    };

    use talpid_types::ErrorExt;

    use crate::cli;

    #[cfg(target_os = "linux")]
    pub(crate) const EARLY_BOOT_LOG_FILENAME: &str = "early-boot-fw.log";

    pub(crate) fn main() {
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

    pub(crate) fn new_runtime() -> tokio::runtime::Runtime {
        let mut builder = match cli::get_config().command {
            #[cfg(target_os = "windows")]
            cli::Command::RunAsService | cli::Command::RegisterService => {
                runtime::new_current_thread()
            }
            _ => runtime::new_multi_thread(),
        };

        builder.build().unwrap_or_else(|e| {
            eprintln!("{}", e.display_chain());
            std::process::exit(1);
        })
    }

    pub(crate) async fn run() -> Result<(), String> {
        let config = cli::get_config();

        match config.command {
            cli::Command::Daemon => {
                // uniqueness check must happen before logging initializaton,
                // as initializing logs will rotate any existing log file.
                assert_unique().await?;
                let (log_location, reload_handle) = init_daemon_logging(config)?;
                log::trace!("Using configuration: {:?}", config);

                let log_dir = log_location.map(|l| l.directory);
                run_standalone(log_dir, reload_handle).await
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

                std::process::exit(crate::macos_launch_daemon::get_status() as i32);
            }
        }
    }

    /// Check that there's not another daemon currently running.
    pub(crate) async fn assert_unique() -> Result<(), &'static str> {
        if rpc_uniqueness_check::is_another_instance_running().await {
            return Err("Another instance of the daemon is already running");
        }
        Ok(())
    }

    /// Initialize logging to stderr and to file (if configured).
    pub(crate) fn init_daemon_logging(
        config: &cli::Config,
    ) -> Result<(Option<LogLocation>, logging::LogHandle), String> {
        let log_location = get_log_dir(config)?.map(|directory| LogLocation {
            directory,
            filename: PathBuf::from("daemon.log"),
        });

        let reload_handle = init_logger(config, log_location.clone())?;

        if let Some(log_location) = log_location.as_ref() {
            log::info!("Logging to {}", log_location.log_path().display());
        }
        Ok((log_location, reload_handle))
    }

    /// Initialize logging to stder and to the [`EARLY_BOOT_LOG_FILENAME`]
    #[cfg(target_os = "linux")]
    pub(crate) fn init_early_boot_logging(config: &cli::Config) {
        let logging = get_log_dir(config)
            .ok()
            .flatten()
            .map(|log_dir| LogLocation {
                directory: log_dir,
                filename: PathBuf::from(EARLY_BOOT_LOG_FILENAME),
            });

        // If it's possible to log to the filesystem - attempt to do so, but failing that mustn't stop
        // the daemon from starting here.
        if init_logger(config, logging).is_err() {
            let _ = init_logger(config, None);
        };
    }

    /// Initialize logging to stderr and to file (if provided).
    ///
    /// Also install the [exception_logging] signal handler to log faults.
    pub(crate) fn init_logger(
        config: &cli::Config,
        log_location: Option<LogLocation>,
    ) -> Result<logging::LogHandle, String> {
        #[cfg(unix)]
        if let Some(log_location) = log_location.as_ref() {
            use std::os::unix::ffi::OsStrExt;

            exception_logging::set_log_file(
                std::ffi::CString::new(log_location.log_path().as_os_str().as_bytes())
                    .map_err(|_| "Log file path contains null-bytes".to_string())?,
            );
        }

        exception_logging::enable();

        let reload_handle =
            logging::init_logger(config.log_level, log_location, config.log_stdout_timestamps)
                .map_err(|e| e.display_chain_with_msg("Unable to initialize logger"))?;
        log_panics::init();
        version::log_version();
        Ok(reload_handle)
    }

    pub(crate) fn get_log_dir(config: &cli::Config) -> Result<Option<PathBuf>, String> {
        if config.log_to_file {
            Ok(Some(mullvad_paths::log_dir().map_err(|e| {
                e.display_chain_with_msg("Unable to get log directory")
            })?))
        } else {
            Ok(None)
        }
    }

    pub(crate) async fn run_standalone(
        log_dir: Option<PathBuf>,
        log_handle: logging::LogHandle,
    ) -> Result<(), String> {
        #[cfg(not(windows))]
        cleanup_old_rpc_socket(mullvad_paths::get_rpc_socket_path()).await;

        if !running_as_admin() {
            log::warn!(
                "Running daemon as a non-administrator user, clients might refuse to connect"
            );
        }

        let daemon = create_daemon(log_dir, log_handle).await?;

        let shutdown_handle = daemon.shutdown_handle();
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        mullvad_daemon::shutdown::set_shutdown_signal_handler(move || {
            shutdown_handle.shutdown(!mullvad_daemon::shutdown::is_shutdown_user_initiated())
        })
        .map_err(|e| e.display_chain())?;

        #[cfg(any(windows, target_os = "android"))]
        mullvad_daemon::shutdown::set_shutdown_signal_handler(move || {
            shutdown_handle.shutdown(true)
        })
        .map_err(|e| e.display_chain())?;

        daemon.run().await.map_err(|e| e.display_chain())?;

        #[cfg(not(windows))]
        cleanup_old_rpc_socket(mullvad_paths::get_rpc_socket_path()).await;

        log::info!("Mullvad daemon is quitting");
        thread::sleep(Duration::from_millis(500));
        Ok(())
    }

    pub(crate) async fn create_daemon(
        log_dir: Option<PathBuf>,
        log_handle: logging::LogHandle,
    ) -> Result<Daemon, String> {
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
                log_handle,
            },
            DaemonCommandChannel::new(),
        )
        .await
        .map_err(|e| e.display_chain_with_msg("Unable to initialize daemon"))
    }

    #[cfg(unix)]
    pub(crate) fn running_as_admin() -> bool {
        nix::unistd::Uid::current().is_root()
    }

    #[cfg(windows)]
    pub(crate) fn running_as_admin() -> bool {
        // TODO: Check if user is administrator correctly on Windows.
        true
    }
}
