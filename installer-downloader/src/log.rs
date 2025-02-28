use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use std::{io, path::PathBuf};

pub fn init() -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(io::stdout())
        .chain(fern::log_file(log_path())?)
        .apply()?;

    Ok(())
}

fn log_path() -> PathBuf {
    std::env::temp_dir().join("mullvad-downloader.log")
}
