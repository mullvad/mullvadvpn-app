use parking_lot::Mutex;
use std::{collections::HashMap, fs, io::Write, path::Path};

lazy_static::lazy_static! {
    static ref LOG_MUTEX: Mutex<HashMap<u32, fs::File>> = Mutex::new(HashMap::new());
}

static mut LOG_CONTEXT_NEXT_ORDINAL: u32 = 0;

/// Errors encountered when initializing logging
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failed to move or create a log file.
    #[error(display = "Failed to setup a logging file")]
    PrepareLogFileError(#[error(source)] std::io::Error),
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
    fs::File::create(log_path.unwrap_or(NULL_DEVICE.as_ref())).map_err(Error::PrepareLogFileError)
}

pub fn clean_up_logging(ordinal: u32) {
    let mut map = LOG_MUTEX.lock();
    map.remove(&ordinal);
}

// Callback that receives messages from WireGuard
pub unsafe extern "system" fn logging_callback(
    level: WgLogLevel,
    msg: *const libc::c_char,
    context: *mut libc::c_void,
) {
    let map = LOG_MUTEX.lock();
    if let Some(mut logfile) = map.get(&(context as u32)) {
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

        let level_str = match level {
            WG_GO_LOG_VERBOSE => "VERBOSE",
            WG_GO_LOG_ERROR | _ => "ERROR",
        };

        let _ = write!(
            logfile,
            "{}[{}][{}] {}",
            chrono::Local::now().format("[%Y-%m-%d %H:%M:%S%.3f]"),
            "wireguard-go",
            level_str,
            managed_msg
        );
    }
}

// unsafe fn

pub type WgLogLevel = u32;
// wireguard-go supports log levels 0 through 3 with 3 being the most verbose
// const WG_GO_LOG_SILENT: WgLogLevel = 0;
const WG_GO_LOG_ERROR: WgLogLevel = 1;
const WG_GO_LOG_VERBOSE: WgLogLevel = 2;
