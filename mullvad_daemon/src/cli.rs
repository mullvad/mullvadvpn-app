use clap::{App, Arg};
use log;

pub struct Config {
    pub log_level: log::LogLevelFilter,
}

pub fn get_config() -> Config {
    let app = create_app();
    let matches = app.get_matches();
    
    let log_level = match matches.occurrences_of("v") {
        0 => log::LogLevelFilter::Info,
        1 => log::LogLevelFilter::Debug,
        _ => log::LogLevelFilter::Trace,
    };

    Config {
        log_level,
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
}
