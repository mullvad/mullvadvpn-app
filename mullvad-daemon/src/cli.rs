use clap::{App, Arg};
use log;

use std::path::PathBuf;

use version;

pub struct Config {
    pub log_level: log::LevelFilter,
    pub log_file: Option<PathBuf>,
    pub tunnel_log_file: Option<PathBuf>,
    pub resource_dir: Option<PathBuf>,
    pub require_auth: bool,
    pub log_stdout_timestamps : bool, }

pub fn get_config() -> Config {
    let app = create_app();
    let matches = app.get_matches();

    let log_level = match matches.occurrences_of("v") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    let log_file = matches.value_of_os("log_file").map(PathBuf::from);
    let tunnel_log_file = matches.value_of_os("tunnel_log_file").map(PathBuf::from);
    let resource_dir = matches.value_of_os("resource_dir").map(PathBuf::from);
    let require_auth = !matches.is_present("disable_rpc_auth");
    let log_stdout_timestamps = !matches.is_present("disable_stdout_timestamps");

    Config {
        log_level,
        log_file,
        tunnel_log_file,
        resource_dir,
        require_auth,
        log_stdout_timestamps,
    }
}

fn create_app() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(version::current())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity."),
        )
        .arg(
            Arg::with_name("log_file")
                .long("log")
                .takes_value(true)
                .value_name("PATH")
                .help("Activates file logging to the given path."),
        )
        .arg(
            Arg::with_name("tunnel_log_file")
                .long("tunnel-log")
                .takes_value(true)
                .value_name("PATH")
                .help("Save log from tunnel implementation process to this file path."),
        )
        .arg(
            Arg::with_name("resource_dir")
                .long("resource-dir")
                .takes_value(true)
                .value_name("DIR")
                .help("Uses the given directory to read needed resources, such as certificates."),
        )
        .arg(
            Arg::with_name("disable_rpc_auth")
                .long("disable-rpc-auth")
                .help("Don't require authentication on the RPC management interface."),
        )
        .arg(
            Arg::with_name("disable_stdout_timestamps")
            .long("no-stdout-timestamps")
            .help("Don't log timestamps when logging to stdout, useful when running as a systemd service")
            )
}
