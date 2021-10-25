use clap::{crate_authors, crate_description, crate_name, App, Arg};
use log;

use crate::version;

#[derive(Debug)]
pub struct Config {
    pub log_level: log::LevelFilter,
    pub log_to_file: bool,
    pub log_stdout_timestamps: bool,
    pub run_as_service: bool,
    pub register_service: bool,
    pub restart_service: bool,
}

pub fn get_config() -> &'static Config {
    lazy_static::lazy_static! {
        static ref CONFIG: Config = create_config();
    }
    &*CONFIG
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

    let run_as_service = cfg!(windows) && matches.is_present("run_as_service");
    let register_service = cfg!(windows) && matches.is_present("register_service");
    let restart_service = cfg!(windows) && matches.is_present("restart_service");

    Config {
        log_level,
        log_to_file,
        log_stdout_timestamps,
        run_as_service,
        register_service,
        restart_service,
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
        mullvad_paths::get_default_settings_dir().expect("Unable to get settings dir").display(),
        mullvad_paths::get_default_cache_dir().expect("Unable to get cache dir").display(),
        mullvad_paths::get_default_log_dir().expect("Unable to get log dir").display(),
        mullvad_paths::get_default_rpc_socket_path().display());
}

fn create_app() -> App<'static, 'static> {
    let mut app = App::new(crate_name!())
        .version(version::PRODUCT_VERSION)
        .author(crate_authors!(", "))
        .about(crate_description!())
        .after_help(ENV_DESC.as_str())
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("disable_log_to_file")
                .long("disable-log-to-file")
                .help("Disable logging to file"),
        )
        .arg(
            Arg::with_name("disable_stdout_timestamps")
                .long("disable-stdout-timestamps")
                .help("Don't log timestamps when logging to stdout, useful when running as a systemd service")
        );

    if cfg!(windows) {
        app = app.arg(
            Arg::with_name("run_as_service")
                .long("run-as-service")
                .help("Run as a system service. On Windows this option must be used when running a system service"),
        )
        .arg(
            Arg::with_name("register_service")
                .long("register-service")
                .help("Register itself as a system service"),
        )
        .arg(
            Arg::with_name("restart_service")
                .long("restart-service")
                .help("Restarts the existing system service"),
        )
    }
    app
}
