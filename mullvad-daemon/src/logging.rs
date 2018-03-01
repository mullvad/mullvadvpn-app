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

pub const DATE_TIME_FORMAT_STR: &str = "%Y-%m-%d %H:%M:%S%.3f";

pub fn init_logger(log_level: log::LevelFilter, log_file: Option<&PathBuf>) -> Result<()> {
    let silenced_crates = [
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
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::Black);

    let mut top_dispatcher = fern::Dispatch::new().level(log_level);
    for silenced_crate in &silenced_crates {
        top_dispatcher = top_dispatcher.level_for(*silenced_crate, log::LevelFilter::Warn);
    }

    let stdout_dispatcher = fern::Dispatch::new()
        .format(move |out, message, record| format_log_msg(out, message, record, Some(&colors)))
        .chain(io::stdout());
    top_dispatcher = top_dispatcher.chain(stdout_dispatcher);

    if let Some(ref log_file) = log_file {
        let f = fern::log_file(log_file)
            .chain_err(|| ErrorKind::WriteFileError(log_file.to_path_buf()))?;
        let file_dispatcher = fern::Dispatch::new()
            .format(|out, message, record| format_log_msg(out, message, record, None))
            .chain(f);
        top_dispatcher = top_dispatcher.chain(file_dispatcher);
    }
    top_dispatcher.apply()?;
    Ok(())
}

fn format_log_msg(
    out: fern::FormatCallback,
    message: &fmt::Arguments,
    record: &log::Record,
    colors: Option<&ColoredLevelConfig>,
) {
    let timestamp = chrono::Local::now().format(DATE_TIME_FORMAT_STR);
    if let (Some(colors), false) = (colors, cfg!(windows)) {
        out.finish(format_args!(
            "[{}][{}][{}] {}",
            timestamp,
            record.target(),
            colors.color(record.level()),
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
