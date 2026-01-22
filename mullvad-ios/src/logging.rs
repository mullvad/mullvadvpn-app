//! FFI logging bridge that forwards Rust logs to Swift.
//!
//! This module provides a tracing subscriber that calls a Swift callback for each log event,
//! allowing Rust logs to be captured by Swift's logging infrastructure.

use mullvad_logging::{EnvFilter, LevelFilter, silence_crates};
use std::io::Write;
use std::{
    ffi::CStr,
    sync::{Mutex, MutexGuard},
};
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Callback function type for logging.
/// - `level`: The log level (1=Error, 2=Warn, 3=Info, 4=Debug, 5=Trace)
/// - `message`: Null-terminated UTF-8 string containing the log message
pub type LogCallback = extern "C" fn(level: u8, message: *const libc::c_char);

/// Default log level
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

/// Factory for creating writers that forward to Swift.
struct SwiftMakeWriter {
    callback: LogCallback,
    buffer: Mutex<Vec<u8>>,
}

impl SwiftMakeWriter {
    fn new(callback: LogCallback) -> Self {
        Self {
            callback,
            buffer: Mutex::new(Vec::with_capacity(1024)),
        }
    }

    fn make_writer_for_level<'a>(&'a self, tracing_level: tracing::Level) -> SwiftWriter<'a> {
        let mut buffer = self.buffer.lock().expect("Failed to acquire logging lock");
        buffer.truncate(0);

        let level = match tracing_level {
            tracing::Level::ERROR => 1u8,
            tracing::Level::WARN => 2u8,
            tracing::Level::INFO => 3u8,
            tracing::Level::DEBUG => 4u8,
            tracing::Level::TRACE => 5u8,
        };

        SwiftWriter {
            callback: self.callback,
            level,
            buffer,
        }
    }
}

impl<'a> MakeWriter<'a> for SwiftMakeWriter {
    type Writer = SwiftWriter<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        self.make_writer_for_level(tracing::Level::DEBUG)
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        self.make_writer_for_level(*meta.level())
    }
}

/// Writer that buffers output and sends to Swift on flush/drop.
struct SwiftWriter<'a> {
    callback: LogCallback,
    level: u8,
    buffer: MutexGuard<'a, Vec<u8>>,
}

impl Write for SwiftWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        self.buffer.push(b'\0');
        if let Ok(message) = CStr::from_bytes_until_nul(&self.buffer) {
            (self.callback)(self.level, message.as_ptr());
        }
        Ok(())
    }
}

impl Drop for SwiftWriter<'_> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

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
    let env_filter = EnvFilter::builder()
        .with_default_directive(DEFAULT_LOG_LEVEL.into())
        .from_env_lossy();

    let layer = tracing_subscriber::fmt::layer()
        .with_writer(SwiftMakeWriter::new(callback))
        .with_ansi(false)
        .without_time()
        .with_level(false)
        .with_filter(silence_crates(env_filter));

    let _ = tracing_subscriber::registry().with(layer).try_init();
}
