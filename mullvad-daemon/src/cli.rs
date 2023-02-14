use clap::{crate_authors, crate_description, crate_name, App, Arg};

#[derive(Debug)]
pub struct Config {
    pub log_level: log::LevelFilter,
    pub log_to_file: bool,
    pub log_stdout_timestamps: bool,
    pub run_as_service: bool,
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

pub fn create_config() -> Config {
    let app = create_app();
    let matches = app.get_matches();

    let log_level = match matches.occurrences_of("v") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    let log_to_file = !matches.is_present("disable_log_to_file");
    let log_stdout_timestamps = !matches.is_present("disable_stdout_timestamps");

    #[cfg(target_os = "linux")]
    let initialize_firewall_and_exit =
        cfg!(target_os = "linux") && matches.is_present("initialize-early-boot-firewall");
    let run_as_service = cfg!(windows) && matches.is_present("run_as_service");
    let register_service = cfg!(windows) && matches.is_present("register_service");
    #[cfg(target_os = "macos")]
    let launch_daemon_status = matches.is_present("launch_daemon_status");

    Config {
        #[cfg(target_os = "linux")]
        initialize_firewall_and_exit,
        log_level,
        log_to_file,
        log_stdout_timestamps,
        run_as_service,
        register_service,
        #[cfg(target_os = "macos")]
        launch_daemon_status,
    }
}

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

fn create_app() -> App<'static> {
    let mut app = App::new(crate_name!())
        .version(mullvad_version::VERSION)
        .author(crate_authors!(", "))
        .about(crate_description!())
        .after_help(ENV_DESC.as_str())
        .arg(
            Arg::new("v")
                .short('v')
                .multiple_occurrences(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::new("disable_log_to_file")
                .long("disable-log-to-file")
                .help("Disable logging to file"),
        )
        .arg(
            Arg::new("disable_stdout_timestamps")
                .long("disable-stdout-timestamps")
                .help("Don't log timestamps when logging to stdout, useful when running as a systemd service")
        );

    if cfg!(windows) {
        app = app.arg(
            Arg::new("run_as_service")
                .long("run-as-service")
                .help("Run as a system service. On Windows this option must be used when running a system service"),
        )
        .arg(
            Arg::new("register_service")
                .long("register-service")
                .help("Register itself as a system service"),
        )
    }

    if cfg!(target_os = "linux") {
        app = app.arg(
            Arg::new("initialize-early-boot-firewall")
                .long("initialize-early-boot-firewall")
                .help("Initialize firewall to be used during early boot and exit"),
        )
    }

    if cfg!(target_os = "macos") {
        app = app.arg(Arg::new("launch_daemon_status").long("launch-daemon-status").help(
            "Checks the status of the launch daemon. The exit code represents the current status",
        ))
    }
    app
}
