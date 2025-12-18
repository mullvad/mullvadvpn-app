use std::{
    io,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use talpid_core::logging::rotate_log;
use tracing_subscriber;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Unable to open log file for writing
    #[error("Unable to open log file for writing: {path}")]
    WriteFile {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Unable to rotate daemon log file")]
    RotateLog(#[from] talpid_core::logging::RotateLogError),

    #[error("Unable to set logger")]
    SetLoggerError(#[from] log::SetLoggerError),
}

pub const WARNING_SILENCED_CRATES: &[&str] = &["netlink_proto", "quinn_udp"];
pub const SILENCED_CRATES: &[&str] = &[
    "h2",
    "tokio_core",
    "tokio_io",
    "tokio_proto",
    "tokio_reactor",
    "tokio_threadpool",
    "tokio_util",
    "tower",
    "want",
    "ws",
    "mio",
    "mnl",
    "hyper",
    "hyper_util",
    "rtnetlink",
    "rustls",
    "netlink_sys",
    "tracing",
    "hickory_proto",
    "hickory_server",
    "hickory_resolver",
    "shadowsocks::relay::udprelay",
    "quinn_proto",
    "quinn",
];
const SLIGHTLY_SILENCED_CRATES: &[&str] = &["nftnl", "udp_over_tcp"];

// const COLORS: ColoredLevelConfig = ColoredLevelConfig {
//     error: Color::Red,
//     warn: Color::Yellow,
//     info: Color::Green,
//     debug: Color::Blue,
//     trace: Color::Black,
// };

#[cfg(not(windows))]
const LINE_SEPARATOR: &str = "\n";

#[cfg(windows)]
const LINE_SEPARATOR: &str = "\r\n";

const DATE_TIME_FORMAT_STR: &str = "[%Y-%m-%d %H:%M:%S%.3f]";

/// Whether a [log] logger has been initialized.
// the log crate doesn't provide a nice way to tell if a logger has been initialized :(
static LOG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Check whether logging has been enabled, i.e. if [init_logger] has been called successfully.
pub fn is_enabled() -> bool {
    LOG_ENABLED.load(Ordering::SeqCst)
}

pub fn init_logger(
    log_level: log::LevelFilter,
    log_file: Option<&PathBuf>,
    output_timestamp: bool,
) -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    // for silenced_crate in WARNING_SILENCED_CRATES {
    //     top_dispatcher = top_dispatcher.level_for(*silenced_crate, log::LevelFilter::Error);
    // }
    // for silenced_crate in SILENCED_CRATES {
    //     top_dispatcher = top_dispatcher.level_for(*silenced_crate, log::LevelFilter::Warn);
    // }
    // for silenced_crate in SLIGHTLY_SILENCED_CRATES {
    //     top_dispatcher = top_dispatcher.level_for(*silenced_crate, one_level_quieter(log_level));
    // }

    if let Some(ref log_file) = log_file {
        rotate_log(log_file).map_err(Error::RotateLog)?;
    }
    #[cfg(all(target_os = "android", debug_assertions))]
    {
        use android_logger::{AndroidLogger, Config};
        let logger: Box<dyn log::Log> = Box::new(AndroidLogger::new(
            Config::default().with_tag("mullvad-daemon"),
        ));
        top_dispatcher = top_dispatcher.chain(logger);
    }

    LOG_ENABLED.store(true, Ordering::SeqCst);

    Ok(())
}

fn one_level_quieter(level: log::LevelFilter) -> log::LevelFilter {
    use log::LevelFilter::*;
    match level {
        Off => Off,
        Error => Off,
        Warn => Error,
        Info => Warn,
        Debug => Info,
        Trace => Debug,
    }
}

#[cfg(not(windows))]
fn escape_newlines(text: String) -> String {
    text
}

#[cfg(windows)]
fn escape_newlines(text: String) -> String {
    text.replace('\n', LINE_SEPARATOR)
}
