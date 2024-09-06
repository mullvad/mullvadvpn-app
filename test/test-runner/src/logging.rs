use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::LazyLock,
};
use test_rpc::logging::{Error, LogFile, LogOutput, Output};
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

pub static LOGGER: LazyLock<StdOutBuffer> = LazyLock::new(|| {
    let (sender, listener) = channel(MAX_OUTPUT_BUFFER);
    StdOutBuffer(Mutex::new(listener), sender)
});

pub struct StdOutBuffer(pub Mutex<Receiver<Output>>, pub Sender<Output>);

impl log::Log for StdOutBuffer {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record<'_>) {
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
        settings_json: read_settings_file().await.map(|log| log.content),
        log_files: read_log_files().await,
    }
}

async fn read_settings_file() -> Result<LogFile, Error> {
    let mut settings_path = mullvad_paths::get_default_settings_dir()
        .map_err(|error| Error::Logs(format!("{}", error)))?;
    settings_path.push("settings.json");
    read_truncated(&settings_path)
        .await
        .map_err(|error| Error::Logs(format!("{}: {}", settings_path.display(), error)))
}

async fn read_log_files() -> Result<Vec<Result<LogFile, Error>>, Error> {
    let log_dir =
        mullvad_paths::get_default_log_dir().map_err(|error| Error::Logs(format!("{error}")))?;
    let gui_log_dirs = get_gui_log_dirs()
        .await
        .map_err(|error| Error::Logs(format!("{error}")))?;

    // Get paths of individual log files
    let mut log_files = list_logs(log_dir).await?;
    for gui_log_dirs in gui_log_dirs {
        log_files.extend(list_logs(gui_log_dirs).await?);
    }

    // Read contents of logs
    let mut logs = vec![];
    for log_file in log_files {
        let log_file = read_truncated(&log_file)
            .await
            .map_err(|error| Error::Logs(format!("{}: {}", log_file.display(), error)));
        logs.push(log_file);
    }

    Ok(logs)
}

#[cfg(not(target_os = "macos"))]
async fn get_gui_log_dirs() -> Result<Vec<PathBuf>, Error> {
    // TODO: Linux
    // TODO: Windows
    Ok(vec![])
}

#[cfg(target_os = "macos")]
async fn get_gui_log_dirs() -> Result<Vec<PathBuf>, io::Error> {
    let mut log_dirs = vec![];
    let mut user_dirs = tokio::fs::read_dir("/Users/").await?;

    while let Ok(Some(dir)) = user_dirs.next_entry().await {
        if !dir.file_type().await?.is_dir() {
            continue;
        }

        let log_dir = dir.path().join("Library").join("Logs").join("Mullvad VPN");
        if log_dir.exists() {
            log_dirs.push(log_dir);
        }
    }

    Ok(log_dirs)
}

async fn list_logs<T: AsRef<Path>>(log_dir: T) -> Result<Vec<PathBuf>, Error> {
    let mut dir_entries = tokio::fs::read_dir(&log_dir)
        .await
        .map_err(|e| Error::Logs(format!("{}: {}", log_dir.as_ref().display(), e)))?;

    let mut paths = vec![];
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

async fn read_truncated<T: AsRef<Path>>(path: T) -> io::Result<LogFile> {
    let mut output = vec![];
    let reader = BufReader::new(File::open(&path).await?);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        output.push(line);
    }
    if output.len() > TRUNCATE_LOG_FILE_LINES {
        let drop_count = output.len() - TRUNCATE_LOG_FILE_LINES;
        // not the most efficient
        output.drain(0..drop_count);
    }
    Ok(LogFile {
        name: path.as_ref().to_path_buf(),
        content: output.join("\n"),
    })
}
