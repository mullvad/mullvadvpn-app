use parking_lot::Mutex;
use std::{fmt, fs, io::Write, path::Path};
#[cfg(windows)]
use widestring::U16CStr;

lazy_static::lazy_static! {
    static ref LOG_MUTEX: Mutex<Option<fs::File>> = Mutex::new(None);
}

/// Errors encountered when initializing logging
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failed to move or create a log file.
    #[error(display = "Failed to setup a logging file")]
    PrepareLogFileError(#[error(source)] std::io::Error),

    /// A previous logger has not been cleaned up.
    #[error(display = "Logger already exists")]
    AlreadyInitialized,
}

pub fn initialize_logging(log_path: Option<&Path>) -> Result<(), Error> {
    let log_file = create_log_file(log_path)?;
    let mut map = LOG_MUTEX.lock();
    if map.is_some() {
        return Err(Error::AlreadyInitialized);
    }
    *map = Some(log_file);
    Ok(())
}

#[cfg(target_os = "windows")]
static NULL_DEVICE: &str = "NUL";

#[cfg(not(target_os = "windows"))]
static NULL_DEVICE: &str = "/dev/null";

fn create_log_file(log_path: Option<&Path>) -> Result<fs::File, Error> {
    fs::File::create(log_path.unwrap_or(NULL_DEVICE.as_ref())).map_err(Error::PrepareLogFileError)
}

pub fn clean_up_logging() {
    LOG_MUTEX.lock().take();
}

#[allow(dead_code)]
enum LogLevel {
    Verbose,
    Info,
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

fn log(level: LogLevel, tag: &str, msg: &str) {
    if let Some(logfile) = LOG_MUTEX.lock().as_mut() {
        let _ = write!(
            logfile,
            "{}[{}][{}] {}",
            chrono::Local::now().format("[%Y-%m-%d %H:%M:%S%.3f]"),
            tag,
            level,
            msg,
        );
    }
}

// Callback that receives messages from WireGuard
pub unsafe extern "system" fn go_logging_callback(
    level: WgLogLevel,
    msg: *const libc::c_char,
    _context: *mut libc::c_void,
) {
    let managed_msg = if !msg.is_null() {
        #[cfg(not(target_os = "windows"))]
        let m = std::ffi::CStr::from_ptr(msg).to_string_lossy().to_string();
        #[cfg(target_os = "windows")]
        let m = std::ffi::CStr::from_ptr(msg)
            .to_string_lossy()
            .to_string()
            .replace("\n", "\r\n");
        m
    } else {
        "Logging message from WireGuard is NULL".to_string()
    };
    let level = match level {
        WG_GO_LOG_VERBOSE => LogLevel::Verbose,
        WG_GO_LOG_ERROR | _ => LogLevel::Error,
    };
    log(level, "wireguard-go", &managed_msg);
}

pub type WgLogLevel = u32;
// wireguard-go supports log levels 0 through 3 with 3 being the most verbose
// const WG_GO_LOG_SILENT: WgLogLevel = 0;
const WG_GO_LOG_ERROR: WgLogLevel = 1;
const WG_GO_LOG_VERBOSE: WgLogLevel = 2;

#[cfg(windows)]
pub extern "stdcall" fn wg_nt_logging_callback(
    level: WgNtLogLevel,
    _timestamp: u64,
    message: *const u16,
) {
    if message.is_null() {
        return;
    }
    let mut message = unsafe { U16CStr::from_ptr_str(message) }.to_string_lossy();
    message.push_str("\r\n");
    log(LogLevel::from(level), "wireguard-nt", &message);
}

#[cfg(windows)]
#[repr(C)]
#[allow(dead_code)]
pub enum WgNtLogLevel {
    Info,
    Warn,
    Err,
}

#[cfg(windows)]
impl From<WgNtLogLevel> for LogLevel {
    fn from(level: WgNtLogLevel) -> Self {
        match level {
            WgNtLogLevel::Info => Self::Info,
            WgNtLogLevel::Warn => Self::Warning,
            WgNtLogLevel::Err => Self::Error,
        }
    }
}
