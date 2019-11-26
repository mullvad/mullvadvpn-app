use libc::{c_char, c_void};
use std::ffi::CStr;

/// Logging callback type.
pub type LogSink = extern "system" fn(level: log::Level, msg: *const c_char, context: *mut c_void);


/// Logging callback implementation.
pub extern "system" fn log_sink(level: log::Level, msg: *const c_char, context: *mut c_void) {
    if msg.is_null() {
        log::error!("Log message from FFI boundary is NULL");
    } else {
        let rust_log_level = log::Level::from(level);
        let target = if context.is_null() {
            "UNKNOWN".into()
        } else {
            unsafe { CStr::from_ptr(context as *const _).to_string_lossy() }
        };

        let managed_msg = unsafe { CStr::from_ptr(msg).to_string_lossy() };

        log::log!(rust_log_level, "{}{}", target, managed_msg);
    }
}
