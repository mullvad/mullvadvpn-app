use std::ptr;

use libc;
use widestring::WideCString;

use ffi;

error_chain!{
    errors{
        /// Failure to set metrics of network interfaces
        MetricApplication{
            description("Failed to set the metrics for a network interface")
        }
        InvalidInterfaceAlias{
            description("Supplied interface alias is invalid")
        }
        GetIpv6Status {
            description("Failed to read IPv6 status on the TAP network interface")
        }
    }
}

/// Returns true if metrics were changed, false otherwise
pub fn ensure_top_metric_for_interface(interface_alias: &str) -> Result<bool> {
    let interface_alias_ws =
        WideCString::from_str(interface_alias).chain_err(|| ErrorKind::InvalidInterfaceAlias)?;

    let metric_result = unsafe {
        WinRoute_EnsureTopMetric(
            interface_alias_ws.as_wide_c_str().as_ptr(),
            Some(ffi::error_sink),
            ptr::null_mut(),
        )
    };

    match metric_result {
        // Metrics didn't change
        0 => Ok(false),
        // Metrics changed
        1 => Ok(true),
        // Failure
        2 => Err(Error::from(ErrorKind::MetricApplication)),
        // Unexpected value
        _ => {
            error!("Unexpected return code from WinRoute_EnsureTopMetric");
            Err(Error::from(ErrorKind::MetricApplication))
        }
    }
}

extern "system" {
    #[link_name(WinRoute_EnsureTopMetric)]
    fn WinRoute_EnsureTopMetric(
        tunnel_interface_alias: *const libc::wchar_t,
        sink: Option<ffi::ErrorSink>,
        sink_context: *mut libc::c_void,
    ) -> u32;
}


/// Checks if IPv6 is enabled for the TAP interface
pub fn get_tap_interface_ipv6_status() -> Result<bool> {
    let tap_ipv6_status =
        unsafe { GetTapInterfaceIpv6Status(Some(ffi::error_sink), ptr::null_mut()) };

    match tap_ipv6_status {
        // Enabled
        0 => Ok(true),
        // Disabled
        1 => Ok(true),
        // Failure
        2 => Err(Error::from(ErrorKind::GetIpv6Status)),
        // Unexpected value
        _ => {
            error!("Unexpected return code from GetTapInterfaceIpv6Status");
            Err(Error::from(ErrorKind::GetIpv6Status))
        }
    }
}

extern "system" {
    #[link_name(GetTapInterfaceIpv6Status)]
    fn GetTapInterfaceIpv6Status(
        sink: Option<ffi::ErrorSink>,
        sink_context: *mut libc::c_void,
    ) -> u32;
}
