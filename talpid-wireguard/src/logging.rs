use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{collections::HashMap, fmt, fs, io::Write, path::Path};

static LOG_MUTEX: Lazy<Mutex<HashMap<u32, fs::File>>> = Lazy::new(|| Mutex::new(HashMap::new()));

static mut LOG_CONTEXT_NEXT_ORDINAL: u32 = 0;

/// Errors encountered when initializing logging
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to move or create a log file.
    #[error("Failed to setup a logging file")]
    PrepareLogFileError(#[from] std::io::Error),
}

pub fn initialize_logging(log_path: Option<&Path>) -> Result<u32, Error> {
    let log_file = create_log_file(log_path)?;

    let log_context_ordinal = unsafe {
        let mut map = LOG_MUTEX.lock();
        let ordinal = LOG_CONTEXT_NEXT_ORDINAL;
        LOG_CONTEXT_NEXT_ORDINAL += 1;
        map.insert(ordinal, log_file);
        ordinal
    };

    Ok(log_context_ordinal)
}

#[cfg(target_os = "windows")]
static NULL_DEVICE: &str = "NUL";

#[cfg(not(target_os = "windows"))]
static NULL_DEVICE: &str = "/dev/null";

fn create_log_file(log_path: Option<&Path>) -> Result<fs::File, Error> {
    fs::File::create(log_path.unwrap_or_else(|| NULL_DEVICE.as_ref()))
        .map_err(Error::PrepareLogFileError)
}

pub fn clean_up_logging(ordinal: u32) {
    let mut map = LOG_MUTEX.lock();
    map.remove(&ordinal);
}

pub enum LogLevel {
    #[cfg_attr(windows, allow(dead_code))]
    Verbose,
    #[cfg_attr(wireguard_go, allow(dead_code))]
    Info,
    #[cfg_attr(wireguard_go, allow(dead_code))]
    Warning,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for LogLevel {
    fn as_ref(&self) -> &str {
        match self {
            LogLevel::Verbose => "VERBOSE",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
        }
    }
}

pub fn log(context: u32, level: LogLevel, tag: &str, msg: &str) {
    let mut map = LOG_MUTEX.lock();
    if let Some(logfile) = map.get_mut(&{ context }) {
        log_inner(logfile, level, tag, msg);
    }
}

fn log_inner(logfile: &mut fs::File, level: LogLevel, tag: &str, msg: &str) {
    let _ = write!(
        logfile,
        "{}[{}][{}] {}",
        chrono::Local::now().format("[%Y-%m-%d %H:%M:%S%.3f]"),
        tag,
        level,
        msg,
    );
}
