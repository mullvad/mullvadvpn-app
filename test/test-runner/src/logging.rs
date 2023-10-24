use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use once_cell::sync::Lazy;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use test_rpc::logging::Error;
use test_rpc::logging::{LogFile, LogOutput, Output};
use tokio::{
    fs::File,
    io::{self, AsyncBufReadExt, BufReader},
    sync::{
        broadcast::{channel, Receiver, Sender},
        Mutex,
    },
};

const MAX_OUTPUT_BUFFER: usize = 10_000;
/// Only consider files that end with ".log"
const INCLUDE_LOG_FILE_EXT: &str = "log";
/// Ignore log files that contain ".old"
const EXCLUDE_LOG_FILE_CONTAIN: &str = ".old";
/// Maximum number of lines that each log file may contain
const TRUNCATE_LOG_FILE_LINES: usize = 100;

pub static LOGGER: Lazy<StdOutBuffer> = Lazy::new(|| {
    let (sender, listener) = channel(MAX_OUTPUT_BUFFER);
    StdOutBuffer(Mutex::new(listener), sender)
});

pub struct StdOutBuffer(pub Mutex<Receiver<Output>>, pub Sender<Output>);

impl log::Log for StdOutBuffer {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.metadata().level() {
                Level::Error => {
                    self.1
                        .send(Output::Error(format!("{}", record.args())))
                        .unwrap();
                }
                Level::Warn => {
                    self.1
                        .send(Output::Warning(format!("{}", record.args())))
                        .unwrap();
                }
                Level::Info => {
                    if !record.metadata().target().contains("tarpc") {
                        self.1
                            .send(Output::Info(format!("{}", record.args())))
                            .unwrap();
                    }
                }
                _ => (),
            }
            println!("{}", record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&*LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

pub async fn get_mullvad_app_logs() -> LogOutput {
    LogOutput {
        settings_json: read_settings_file().await,
        log_files: read_log_files().await,
    }
}

async fn read_settings_file() -> Result<String, Error> {
    let mut settings_path = mullvad_paths::get_default_settings_dir()
        .map_err(|error| Error::Logs(format!("{}", error)))?;
    settings_path.push("settings.json");
    read_truncated(&settings_path)
        .await
        .map_err(|error| Error::Logs(format!("{}: {}", settings_path.display(), error)))
}

async fn read_log_files() -> Result<Vec<Result<LogFile, Error>>, Error> {
    let log_dir =
        mullvad_paths::get_default_log_dir().map_err(|error| Error::Logs(format!("{}", error)))?;
    let paths = list_logs(log_dir)
        .await
        .map_err(|error| Error::Logs(format!("{}", error)))?;
    let mut log_files = Vec::new();
    for path in paths {
        let log_file = read_truncated(&path)
            .await
            .map_err(|error| Error::Logs(format!("{}: {}", path.display(), error)))
            .map(|content| LogFile {
                content,
                name: path,
            });
        log_files.push(log_file);
    }
    Ok(log_files)
}

async fn list_logs<T: AsRef<Path>>(log_dir: T) -> Result<Vec<PathBuf>, Error> {
    let mut dir_entries = tokio::fs::read_dir(&log_dir)
        .await
        .map_err(|e| Error::Logs(format!("{}: {}", log_dir.as_ref().display(), e)))?;

    let mut paths = Vec::new();
    while let Ok(Some(entry)) = dir_entries.next_entry().await {
        let path = entry.path();
        if let Some(u8_path) = path.to_str() {
            if u8_path.contains(EXCLUDE_LOG_FILE_CONTAIN) {
                continue;
            }
        }
        if path.extension() == Some(OsStr::new(INCLUDE_LOG_FILE_EXT)) {
            paths.push(path);
        }
    }
    Ok(paths)
}

async fn read_truncated<T: AsRef<Path>>(path: T) -> io::Result<String> {
    let mut output = vec![];
    let reader = BufReader::new(File::open(path).await?);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        output.push(line);
    }
    if output.len() > TRUNCATE_LOG_FILE_LINES {
        let drop_count = output.len() - TRUNCATE_LOG_FILE_LINES;
        // not the most efficient
        output.drain(0..drop_count);
    }
    Ok(output.join("\n"))
}
