use self::api::*;
pub use self::api::{
    ErrorSink, WinNet_ActivateConnectivityMonitor, WinNet_DeactivateConnectivityMonitor,
};
use libc::{c_char, c_void, wchar_t};
use std::{ffi::OsString, ptr};
use widestring::WideCString;

/// Errors that this module may produce.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failed to set the metrics for a network interface.
    #[error(display = "Failed to set the metrics for a network interface")]
    MetricApplication,

    /// Supplied interface alias is invalid.
    #[error(display = "Supplied interface alias is invalid")]
    InvalidInterfaceAlias(#[error(cause)] widestring::NulError<u16>),

    /// Failed to read IPv6 status on the TAP network interface.
    #[error(display = "Failed to read IPv6 status on the TAP network interface")]
    GetIpv6Status,

    /// Failed to determine alias of TAP adapter.
    #[error(display = "Failed to determine alias of TAP adapter")]
    GetTapAlias,

    /// Can't establish whether host is connected to a non-virtual network
    #[error(display = "Network connectivity undecideable")]
    ConnectivityUnkown,
}

/// Error callback used with `winnet.dll`.
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
        WinNet_EnsureTopMetric(
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
            log::error!("Unexpected return code from WinNet_EnsureTopMetric: {}", i);
            Err(Error::MetricApplication)
        }
    }
}

/// Checks if IPv6 is enabled for the TAP interface
pub fn get_tap_interface_ipv6_status() -> Result<bool, Error> {
    let tap_ipv6_status =
        unsafe { WinNet_GetTapInterfaceIpv6Status(Some(error_sink), ptr::null_mut()) };

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
                "Unexpected return code from WinNet_GetTapInterfaceIpv6Status: {}",
                i
            );
            Err(Error::GetIpv6Status)
        }
    }
}

/// Dynamically determines the alias of the TAP adapter.
pub fn get_tap_interface_alias() -> Result<OsString, Error> {
    let mut alias_ptr: *mut wchar_t = ptr::null_mut();
    let status = unsafe {
        WinNet_GetTapInterfaceAlias(&mut alias_ptr as *mut _, Some(error_sink), ptr::null_mut())
    };

    if !status {
        return Err(Error::GetTapAlias);
    }

    let alias = unsafe { WideCString::from_ptr_str(alias_ptr) };
    unsafe { WinNet_ReleaseString(alias_ptr) };

    Ok(alias.to_os_string())
}

/// Returns true if current host is not connected to any network
pub fn is_offline() -> Result<bool, Error> {
    match unsafe { WinNet_CheckConnectivity(Some(error_sink), ptr::null_mut()) } {
        // Not connected
        0 => Ok(true),
        // Connected
        1 => Ok(false),
        // 2 means that connectivity can't be determined, but any other return value is unexpected
        // and as such, is considered to be an error.
        _ => Err(Error::ConnectivityUnkown),
    }
}

#[allow(non_snake_case)]
mod api {
    use libc::{c_char, c_void, wchar_t};

    /// Error callback type for use with `winnet.dll`.
    pub type ErrorSink = extern "system" fn(msg: *const c_char, ctx: *mut c_void);

    pub type ConnectivityCallback = unsafe extern "system" fn(is_connected: bool, ctx: *mut c_void);

    extern "system" {
        #[link_name = "WinNet_EnsureTopMetric"]
        pub fn WinNet_EnsureTopMetric(
            tunnel_interface_alias: *const wchar_t,
            sink: Option<ErrorSink>,
            sink_context: *mut c_void,
        ) -> u32;

        #[link_name = "WinNet_GetTapInterfaceIpv6Status"]
        pub fn WinNet_GetTapInterfaceIpv6Status(
            sink: Option<ErrorSink>,
            sink_context: *mut c_void,
        ) -> u32;

        #[link_name = "WinNet_GetTapInterfaceAlias"]
        pub fn WinNet_GetTapInterfaceAlias(
            tunnel_interface_alias: *mut *mut wchar_t,
            sink: Option<ErrorSink>,
            sink_context: *mut c_void,
        ) -> bool;

        #[link_name = "WinNet_ReleaseString"]
        pub fn WinNet_ReleaseString(string: *mut wchar_t) -> u32;

        #[link_name = "WinNet_ActivateConnectivityMonitor"]
        pub fn WinNet_ActivateConnectivityMonitor(
            callback: Option<ConnectivityCallback>,
            callbackContext: *mut libc::c_void,
            currentConnectivity: *mut bool,
            sink: Option<ErrorSink>,
            sink_context: *mut c_void,
        ) -> bool;

        #[link_name = "WinNet_DeactivateConnectivityMonitor"]
        pub fn WinNet_DeactivateConnectivityMonitor() -> bool;

        #[link_name = "WinNet_CheckConnectivity"]
        pub fn WinNet_CheckConnectivity(sink: Option<ErrorSink>, sink_context: *mut c_void) -> u32;
    }
}
