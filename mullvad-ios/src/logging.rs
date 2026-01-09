//! FFI logging bridge that forwards Rust logs to Swift.
//!
//! This module provides a global logger that calls a Swift callback for each log message,
//! allowing Rust logs to be captured by Swift's logging infrastructure.

use std::ffi::CString;
use std::sync::OnceLock;

/// Callback function type for logging.
/// - `level`: The log level (1=Error, 2=Warn, 3=Info, 4=Debug, 5=Trace)
/// - `message`: Null-terminated UTF-8 string containing the log message
pub type LogCallback = extern "C" fn(level: u8, message: *const libc::c_char);

/// Global storage for the Swift logging callback
static LOG_CALLBACK: OnceLock<LogCallback> = OnceLock::new();

/// Default log level
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug;

/// Custom logger that forwards to Swift
struct SwiftLogger;

impl log::Log for SwiftLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        if LOG_CALLBACK.get().is_none() {
            return false;
        }

        let max_level =
            mullvad_logging::get_log_level_for_target(metadata.target(), DEFAULT_LOG_LEVEL);
        metadata.level() <= max_level
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Some(callback) = LOG_CALLBACK.get() {
            let level = match record.level() {
                log::Level::Error => 1u8,
                log::Level::Warn => 2u8,
                log::Level::Info => 3u8,
                log::Level::Debug => 4u8,
                log::Level::Trace => 5u8,
            };

            let message = format!("[{}] {}", record.target(), record.args());

            if let Ok(c_message) = CString::new(message) {
                callback(level, c_message.as_ptr());
            }
        }
    }

    fn flush(&self) {}
}

static SWIFT_LOGGER: SwiftLogger = SwiftLogger;

/// Initialize the Rust logger with a Swift callback.
///
/// This function should be called once early in the application lifecycle,
/// before any Rust code that uses logging is invoked.
///
/// # Safety
/// - `callback` must be a valid function pointer that remains valid for the lifetime of the program.
/// - This function is safe to call multiple times, but only the first call will have an effect.
#[unsafe(no_mangle)]
pub extern "C" fn init_rust_logging(callback: LogCallback) {
    if LOG_CALLBACK.set(callback).is_ok() {
        // Only set the logger if we successfully stored the callback
        // Set max level to Trace so our custom filtering in enabled() takes effect
        let _ =
            log::set_logger(&SWIFT_LOGGER).map(|()| log::set_max_level(log::LevelFilter::Trace));
    }
}
