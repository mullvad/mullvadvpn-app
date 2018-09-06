use std::ptr;

use libc::{c_char, c_void, wchar_t};
use widestring::WideCString;

error_chain!{
    errors{
        /// Failure to set metrics of network interfaces
        MetricApplication{
            description("Failed to set the metrics for a network interface")
        }
        InvalidInterfaceAlias{
            description("Supplied interface alias is invalid")
        }
    }
}

pub type ErrorSink = extern "system" fn(msg: *const c_char, ctx: *mut c_void);

pub extern "system" fn error_sink(msg: *const c_char, _ctx: *mut c_void) {
    use std::ffi::CStr;
    if msg.is_null() {
        error!("Log message from FFI boundary is NULL");
    } else {
        error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
    }
}

/// Returns true if metrics were changed, false otherwise
pub fn ensure_top_metric_for_interface(interface_alias: &str) -> Result<bool> {
    let interface_alias_ws =
        WideCString::from_str(interface_alias).chain_err(|| ErrorKind::InvalidInterfaceAlias)?;
    unsafe {
        WinRoute_EnsureTopMetric(
            interface_alias_ws.as_wide_c_str().as_ptr(),
            Some(error_sink),
            ptr::null_mut(),
        ).into()
    }
}

// Allowing dead code here as this type should only ever be constructed by an
// FFI function.
#[allow(dead_code)]
#[repr(u32)]
enum MetricResult {
    MetricsUnchanged = 0u32,
    MetricsChanged = 1u32,
    Failure = 2u32,
    UnexpectedValue,
}

impl Into<Result<bool>> for MetricResult {
    fn into(self) -> Result<bool> {
        match self {
            MetricResult::MetricsUnchanged => Ok(false),
            MetricResult::MetricsChanged => Ok(true),
            MetricResult::Failure => Err(Error::from(ErrorKind::MetricApplication)),
            MetricResult::UnexpectedValue => {
                error!("Unexpected return code from WinRoute_EnsureTopMetric");
                Err(Error::from(ErrorKind::MetricApplication))
            }
        }
    }
}

extern "system" {
    #[link_name(WinRoute_EnsureTopMetric)]
    fn WinRoute_EnsureTopMetric(
        tunnel_interface_alias: *const wchar_t,
        sink: Option<ErrorSink>,
        sink_context: *mut c_void,
    ) -> MetricResult;
}
