use std::ptr;

use libc::{c_char, c_void, wchar_t};
use widestring::WideCString;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failure to set metrics of network interfaces
    #[error(display = "Failed to set the metrics for a network interface")]
    MetricApplication,

    #[error(display = "Supplied interface alias is invalid")]
    InvalidInterfaceAlias(#[error(cause)] widestring::NulError<u16>),

    #[error(display = "Failed to read IPv6 status on the TAP network interface")]
    GetIpv6Status,
}

pub type ErrorSink = extern "system" fn(msg: *const c_char, ctx: *mut c_void);

pub extern "system" fn error_sink(msg: *const c_char, _ctx: *mut c_void) {
    use std::ffi::CStr;
    if msg.is_null() {
        log::error!("Log message from FFI boundary is NULL");
    } else {
        log::error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
    }
}

/// Returns true if metrics were changed, false otherwise
pub fn ensure_top_metric_for_interface(interface_alias: &str) -> Result<bool, Error> {
    let interface_alias_ws =
        WideCString::from_str(interface_alias).map_err(Error::InvalidInterfaceAlias)?;

    let metric_result = unsafe {
        WinRoute_EnsureTopMetric(
            interface_alias_ws.as_ptr(),
            Some(error_sink),
            ptr::null_mut(),
        )
    };

    match metric_result {
        // Metrics didn't change
        0 => Ok(false),
        // Metrics changed
        1 => Ok(true),
        // Failure
        2 => Err(Error::MetricApplication),
        // Unexpected value
        i => {
            log::error!(
                "Unexpected return code from WinRoute_EnsureTopMetric: {}",
                i
            );
            Err(Error::MetricApplication)
        }
    }
}

extern "system" {
    #[link_name = "WinRoute_EnsureTopMetric"]
    fn WinRoute_EnsureTopMetric(
        tunnel_interface_alias: *const wchar_t,
        sink: Option<ErrorSink>,
        sink_context: *mut c_void,
    ) -> u32;
}


/// Checks if IPv6 is enabled for the TAP interface
pub fn get_tap_interface_ipv6_status() -> Result<bool, Error> {
    let tap_ipv6_status = unsafe { GetTapInterfaceIpv6Status(Some(error_sink), ptr::null_mut()) };

    match tap_ipv6_status {
        // Enabled
        0 => Ok(true),
        // Disabled
        1 => Ok(false),
        // Failure
        2 => Err(Error::GetIpv6Status),
        // Unexpected value
        i => {
            log::error!(
                "Unexpected return code from GetTapInterfaceIpv6Status: {}",
                i
            );
            Err(Error::GetIpv6Status)
        }
    }
}

extern "system" {
    #[link_name = "GetTapInterfaceIpv6Status"]
    fn GetTapInterfaceIpv6Status(sink: Option<ErrorSink>, sink_context: *mut c_void) -> u32;
}
