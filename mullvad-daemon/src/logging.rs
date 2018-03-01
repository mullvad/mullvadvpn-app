extern crate fern;

use self::fern::colors::{Color, ColoredLevelConfig};
use chrono;
use log;

use std::fmt;
use std::io;
use std::path::PathBuf;

error_chain! {
    errors {
        WriteFileError(path: PathBuf) {
            description("Unable to open log file for writing")
            display("Unable to open log file for writing: {}", path.to_string_lossy())
        }
    }
    foreign_links {
        SetLoggerError(log::SetLoggerError);
    }
}

const SILENCED_CRATES: &[&str] = &[
    "jsonrpc_core",
    // jsonrpc_core does some logging under the "rpc" target as well.
    "rpc",
    "tokio_core",
    "tokio_proto",
    "jsonrpc_ws_server",
    "ws",
    "mio",
    "hyper",
];

const COLORS: ColoredLevelConfig = ColoredLevelConfig {
    error: Color::Red,
    warn: Color::Yellow,
    info: Color::Green,
    debug: Color::Blue,
    trace: Color::Black,
};

pub const DATE_TIME_FORMAT_STR: &str = "%Y-%m-%d %H:%M:%S%.3f";

pub fn init_logger(log_level: log::LevelFilter, log_file: Option<&PathBuf>) -> Result<()> {
    let mut top_dispatcher = fern::Dispatch::new().level(log_level);
    for silenced_crate in SILENCED_CRATES {
        top_dispatcher = top_dispatcher.level_for(*silenced_crate, log::LevelFilter::Warn);
    }

    let stdout_dispatcher = fern::Dispatch::new()
        .format(move |out, message, record| format_log_message(out, message, record, true))
        .chain(io::stdout());
    top_dispatcher = top_dispatcher.chain(stdout_dispatcher);

    if let Some(ref log_file) = log_file {
        let f = fern::log_file(log_file)
            .chain_err(|| ErrorKind::WriteFileError(log_file.to_path_buf()))?;
        let file_dispatcher = fern::Dispatch::new()
            .format(|out, message, record| format_log_message(out, message, record, false))
            .chain(f);
        top_dispatcher = top_dispatcher.chain(file_dispatcher);
    }
    top_dispatcher.apply()?;
    Ok(())
}

fn format_log_message(
    out: fern::FormatCallback,
    message: &fmt::Arguments,
    record: &log::Record,
    color_output: bool,
) {
    let timestamp = chrono::Local::now().format(DATE_TIME_FORMAT_STR);
    if color_output && cfg!(not(windows)) {
        out.finish(format_args!(
            "[{}][{}][{}] {}",
            timestamp,
            record.target(),
            COLORS.color(record.level()),
            message,
        ))
    } else {
        out.finish(format_args!(
            "[{}][{}][{}] {}",
            timestamp,
            record.target(),
            record.level(),
            message
        ))
    }
}
