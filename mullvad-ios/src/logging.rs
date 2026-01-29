//! FFI logging bridge that forwards Rust logs to Swift.
//!
//! This module provides a tracing layer that calls a Swift callback for each log event,
//! allowing Rust logs to be captured by Swift's logging infrastructure with structured data.

use mullvad_logging::{silence_crates, EnvFilter, LevelFilter};
use std::ffi::CString;
use std::fmt::Write;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

/// Callback function type for logging.
/// - `level`: The log level (1=Error, 2=Warn, 3=Info, 4=Debug, 5=Trace)
/// - `target`: Null-terminated UTF-8 string containing the module/target name
/// - `message`: Null-terminated UTF-8 string containing the log message
pub type LogCallback =
    extern "C" fn(level: u8, target: *const libc::c_char, message: *const libc::c_char);

/// Default log level
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

/// Visitor that extracts the message and other fields from a tracing event.
struct MessageVisitor {
    message: String,
}

impl MessageVisitor {
    fn new() -> Self {
        Self {
            message: String::with_capacity(256),
        }
    }

    fn into_message(self) -> String {
        self.message
    }
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(&mut self.message, "{:?}", value);
        } else if self.message.is_empty() {
            let _ = write!(&mut self.message, "{}={:?}", field.name(), value);
        } else {
            let _ = write!(&mut self.message, ", {}={:?}", field.name(), value);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message.push_str(value);
        } else if self.message.is_empty() {
            let _ = write!(&mut self.message, "{}={}", field.name(), value);
        } else {
            let _ = write!(&mut self.message, ", {}={}", field.name(), value);
        }
    }
}

/// A tracing layer that forwards structured log events to Swift via FFI.
struct SwiftLayer {
    callback: LogCallback,
}

impl SwiftLayer {
    fn new(callback: LogCallback) -> Self {
        Self { callback }
    }

    fn level_to_u8(level: &tracing::Level) -> u8 {
        match *level {
            tracing::Level::ERROR => 1,
            tracing::Level::WARN => 2,
            tracing::Level::INFO => 3,
            tracing::Level::DEBUG => 4,
            tracing::Level::TRACE => 5,
        }
    }
}

impl<S> Layer<S> for SwiftLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = Self::level_to_u8(metadata.level());
        let target = metadata.target();

        // Extract the message using the visitor pattern
        let mut visitor = MessageVisitor::new();
        event.record(&mut visitor);
        let message = visitor.into_message();

        // Convert to C strings for FFI
        let target_cstring = match CString::new(target) {
            Ok(s) => s,
            Err(_) => return,
        };
        let message_cstring = match CString::new(message) {
            Ok(s) => s,
            Err(_) => return,
        };

        (self.callback)(level, target_cstring.as_ptr(), message_cstring.as_ptr());
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

    let layer = SwiftLayer::new(callback).with_filter(silence_crates(env_filter));

    let _ = tracing_subscriber::registry().with(layer).try_init();
}
