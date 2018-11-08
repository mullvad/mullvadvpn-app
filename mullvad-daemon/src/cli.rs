use clap::{crate_authors, crate_description, crate_name, App, Arg};
use log;

use crate::version;

pub struct Config {
    pub log_level: log::LevelFilter,
    pub log_to_file: bool,
    pub log_stdout_timestamps: bool,
    pub run_as_service: bool,
    pub register_service: bool,
}

pub fn get_config() -> Config {
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

    Config {
        log_level,
        log_to_file,
        log_stdout_timestamps,
        run_as_service,
        register_service,
    }
}

fn create_app() -> App<'static, 'static> {
    let app = App::new(crate_name!())
        .version(version::PRODUCT_VERSION)
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity."),
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
        app.arg(
            Arg::with_name("run_as_service")
                .long("run-as-service")
                .help("Run as a system service. On Windows this option must be used when running a system service"),
        ).arg(
            Arg::with_name("register_service")
                .long("register-service")
                .help("Register itself as a system service"),
        )
    } else {
        app
    }
}
