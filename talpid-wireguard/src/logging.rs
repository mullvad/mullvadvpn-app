use parking_lot::Mutex;
use std::{collections::HashMap, fmt, fs, io::Write, path::Path, sync::LazyLock};

static LOG_MUTEX: LazyLock<Mutex<LogState>> = LazyLock::new(|| Mutex::new(LogState::default()));

#[derive(Default)]
struct LogState {
    map: HashMap<u64, fs::File>,
    next_ordinal: u64,
}

/// Errors encountered when initializing logging
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to move or create a log file.
    #[error("Failed to setup a logging file")]
    PrepareLogFileError(#[from] std::io::Error),
}

pub fn initialize_logging(log_path: Option<&Path>) -> Result<u64, Error> {
    let log_file = create_log_file(log_path)?;

    let mut state = LOG_MUTEX.lock();
    let ordinal = state.next_ordinal;
    state.next_ordinal += 1;
    state.map.insert(ordinal, log_file);

    Ok(ordinal)
}

#[cfg(target_os = "windows")]
static NULL_DEVICE: &str = "NUL";

#[cfg(not(target_os = "windows"))]
static NULL_DEVICE: &str = "/dev/null";

fn create_log_file(log_path: Option<&Path>) -> Result<fs::File, Error> {
    fs::File::create(log_path.unwrap_or_else(|| NULL_DEVICE.as_ref()))
        .map_err(Error::PrepareLogFileError)
}

pub fn clean_up_logging(ordinal: u64) {
    let mut state = LOG_MUTEX.lock();
    state.map.remove(&ordinal);
}

#[allow(dead_code)]
pub enum LogLevel {
    Verbose,
    #[cfg_attr(not(feature = "boringtun"), allow(dead_code))]
    Info,
    #[cfg_attr(not(feature = "boringtun"), allow(dead_code))]
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

pub fn log(context: u64, level: LogLevel, tag: &str, msg: &str) {
    let mut state = LOG_MUTEX.lock();
    if let Some(logfile) = state.map.get_mut(&context) {
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
