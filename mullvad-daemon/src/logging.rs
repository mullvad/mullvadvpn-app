extern crate fern;

use chrono;
use log;

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

pub fn init_logger(log_level: log::LogLevelFilter, log_file: Option<&PathBuf>) -> Result<()> {
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
    let mut config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format(DATE_TIME_FORMAT_STR),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(io::stdout());
    for silenced_crate in &silenced_crates {
        config = config.level_for(*silenced_crate, log::LogLevelFilter::Warn);
    }
    if let Some(ref log_file) = log_file {
        let f = fern::log_file(log_file)
            .chain_err(|| ErrorKind::WriteFileError(log_file.to_path_buf()))?;
        config = config.chain(f);
    }
    config.apply()?;
    Ok(())
}
