use clap::{App, Arg};
use log;

use std::path::PathBuf;

pub struct Config {
    pub log_level: log::LogLevelFilter,
    pub log_file: PathBuf,
}

pub fn get_config() -> Config {
    let app = create_app();
    let matches = app.get_matches();
    
    let log_level = match matches.occurrences_of("v") {
        0 => log::LogLevelFilter::Info,
        1 => log::LogLevelFilter::Debug,
        _ => log::LogLevelFilter::Trace,
    };
    let log_file = PathBuf::from(value_t_or_exit!(matches, "log_file", String));

    Config {
        log_level,
        log_file,
    }
}

fn create_app() -> App<'static, 'static> {
    App::new("mullvadd")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity."))
        .arg(Arg::with_name("log_file")
            .long("log")
            .default_value("./mullvadd.log")
            .help("Sets the path where to write the log"))
}
