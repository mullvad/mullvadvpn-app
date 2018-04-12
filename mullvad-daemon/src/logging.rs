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

pub const DATE_TIME_FORMAT_STR: &str = "[%Y-%m-%d %H:%M:%S%.3f]";

pub fn init_logger(log_level: log::LevelFilter, log_file: Option<&PathBuf>, output_timestamp: bool) -> Result<()> {
    let mut top_dispatcher = fern::Dispatch::new().level(log_level);
    for silenced_crate in SILENCED_CRATES {
        top_dispatcher = top_dispatcher.level_for(*silenced_crate, log::LevelFilter::Warn);
    }

    let stdout_formatter = Formatter {
        output_timestamp: output_timestamp,
        output_color: true,
    };
    let stdout_dispatcher = fern::Dispatch::new()
        .format(move |out, message, record| stdout_formatter.output_msg(out, message, record))
        .chain(io::stdout());
    top_dispatcher = top_dispatcher.chain(stdout_dispatcher);

    if let Some(ref log_file) = log_file {
        let file_formatter = Formatter {
            output_timestamp: true,
            output_color: false,
        };
        let f = fern::log_file(log_file)
            .chain_err(|| ErrorKind::WriteFileError(log_file.to_path_buf()))?;
        let file_dispatcher = fern::Dispatch::new()
            .format(move |out, message, record| file_formatter.output_msg(out, message, record))
            .chain(f);
        top_dispatcher = top_dispatcher.chain(file_dispatcher);
    }
    top_dispatcher.apply()?;
    Ok(())
}

#[derive(Default, Debug)]
struct Formatter {
    pub output_timestamp: bool,
    pub output_color: bool,
}

impl Formatter {
    fn get_timetsamp_fmt(&self) -> &str {
        if self.output_timestamp {
            DATE_TIME_FORMAT_STR
        } else {
            &""
        }
    }

    fn get_record_level(&self, level: log::Level) -> Box<fmt::Display> {
        if self.output_color && cfg!(not(windows)) {
            Box::new(COLORS.color(level))
        } else {
            Box::new(level)
        }
    }

    pub fn output_msg(
        &self,
        out: fern::FormatCallback,
        message: &fmt::Arguments,
        record: &log::Record,
    ) {
        out.finish(format_args!(
            "{}[{}][{}] {}",
            chrono::Local::now().format(self.get_timetsamp_fmt()),
            record.target(),
            self.get_record_level(record.level()),
            message,
        ))
    }
}
