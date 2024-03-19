use clap::Parser;
use once_cell::sync::Lazy;

static ENV_DESC: Lazy<String> = Lazy::new(|| {
    format!(
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
        mullvad_paths::get_default_rpc_socket_path().display()
)
});

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
    #[arg(long, conflicts_with = "register_service")]
    run_as_service: bool,
    /// Register Mullvad daemon as a system service
    #[cfg(target_os = "windows")]
    #[arg(long, conflicts_with = "run_as_service")]
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

    pub command: Command,
}

#[derive(Debug)]
pub enum Command {
    /// Run the standalone daemon.
    Daemon,

    /// Initialize firewall to be used during early boot and exit
    #[cfg(target_os = "linux")]
    InitializeEarlyBootFirewall,

    /// Run the daemon as a system service.
    #[cfg(target_os = "windows")]
    RunAsService,

    /// Register Mullvad daemon as a system service.
    #[cfg(target_os = "windows")]
    RegisterService,

    /// Check the status of the launch daemon. The exit code represents the current status.
    #[cfg(target_os = "macos")]
    LaunchDaemonStatus,
}

pub fn get_config() -> &'static Config {
    static CONFIG: Lazy<Config> = Lazy::new(create_config);
    &CONFIG
}

fn create_config() -> Config {
    let app = Cli::parse();

    let log_level = match app.verbosity {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let command_flags = [
        #[cfg(target_os = "linux")]
        (
            app.initialize_early_boot_firewall,
            Command::InitializeEarlyBootFirewall,
        ),
        #[cfg(target_os = "windows")]
        (app.run_as_service, Command::RunAsService),
        #[cfg(target_os = "windows")]
        (app.register_service, Command::RegisterService),
        #[cfg(target_os = "macos")]
        (app.launch_daemon_status, Command::LaunchDaemonStatus),
    ];

    let command = command_flags
        .into_iter()
        .find_map(|(flag, command)| flag.then_some(command))
        .unwrap_or(Command::Daemon);

    Config {
        log_level,
        log_to_file: !app.disable_log_to_file,
        log_stdout_timestamps: !app.disable_stdout_timestamps,
        command,
    }
}
