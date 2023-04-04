use clap::Parser;

lazy_static::lazy_static! {
    static ref ENV_DESC: String = format!(
"ENV:

    MULLVAD_RESOURCE_DIR       Resource directory (i.e used to locate a root CA certificate)
                               [Default: {}]
    MULLVAD_SETTINGS_DIR       Directory path for storing settings. [Default: {}]
    MULLVAD_CACHE_DIR          Directory path for storing cache. [Default: {}]
    MULLVAD_LOG_DIR            Directory path for storing logs. [Default: {}]
    MULLVAD_RPC_SOCKET_PATH    Location of the management interface device.
                               It refers to Unix domain socket on Unix based platforms, and named pipe on Windows.
                               [Default: {}]

",
        mullvad_paths::get_default_resource_dir().display(),
        mullvad_paths::get_default_settings_dir().map(|dir| dir.display().to_string()).unwrap_or_else(|_| "N/A".to_string()),
        mullvad_paths::get_default_cache_dir().map(|dir| dir.display().to_string()).unwrap_or_else(|_| "N/A".to_string()),
        mullvad_paths::get_default_log_dir().map(|dir| dir.display().to_string()).unwrap_or_else(|_| "N/A".to_string()),
        mullvad_paths::get_default_rpc_socket_path().display());
}

#[derive(Debug, Parser)]
#[command(author, version = mullvad_version::VERSION, about, long_about = None, after_help = &*ENV_DESC)]
struct Cli {
    /// Set the level of verbosity
    #[arg(short='v', action = clap::ArgAction::Count)]
    verbosity: u8,
    /// Disable logging to file
    #[arg(long)]
    disable_log_to_file: bool,
    /// Don't log timestamps when logging to stdout, useful when running as a systemd service
    #[arg(long)]
    disable_stdout_timestamps: bool,

    /// Run as a system service
    #[cfg(target_os = "windows")]
    #[arg(long)]
    run_as_service: bool,
    /// Register Mullvad daemon as a system service
    #[cfg(target_os = "windows")]
    #[arg(long)]
    register_service: bool,

    /// Initialize firewall to be used during early boot and exit
    #[cfg(target_os = "linux")]
    #[arg(long)]
    initialize_early_boot_firewall: bool,

    /// Check the status of the launch daemon. The exit code represents the current status
    #[cfg(target_os = "macos")]
    #[arg(long)]
    launch_daemon_status: bool,
}

#[derive(Debug)]
pub struct Config {
    pub log_level: log::LevelFilter,
    pub log_to_file: bool,
    pub log_stdout_timestamps: bool,
    #[cfg(target_os = "windows")]
    pub run_as_service: bool,
    #[cfg(target_os = "windows")]
    pub register_service: bool,
    #[cfg(target_os = "macos")]
    pub launch_daemon_status: bool,
    #[cfg(target_os = "linux")]
    pub initialize_firewall_and_exit: bool,
}

pub fn get_config() -> &'static Config {
    lazy_static::lazy_static! {
        static ref CONFIG: Config = create_config();
    }
    &CONFIG
}

fn create_config() -> Config {
    let app = Cli::parse();

    let log_level = match app.verbosity {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    Config {
        log_level,
        log_to_file: !app.disable_log_to_file,
        log_stdout_timestamps: !app.disable_stdout_timestamps,
        #[cfg(target_os = "windows")]
        run_as_service: app.run_as_service,
        #[cfg(target_os = "windows")]
        register_service: app.register_service,
        #[cfg(target_os = "macos")]
        launch_daemon_status: app.launch_daemon_status,
        #[cfg(target_os = "linux")]
        initialize_firewall_and_exit: app.initialize_early_boot_firewall,
    }
}
