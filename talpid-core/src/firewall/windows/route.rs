extern crate libc;
extern crate widestring;

use self::widestring::WideCString;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use super::ffi;

error_chain!{
    errors{
        #[doc = "Failure to set metrics of network interfaces"]
        MetricApplication{
            description("Failed to set the metrics for a network interface")
        }
    }
}

// Returns true if metrics were changed, false otherwise
pub fn ensure_top_metric_for_interface(interface_alias: &str) -> Result<bool> {
    let interface_alias_ws =
        WideCString::new(interface_alias.encode_utf16().collect::<Vec<_>>()).unwrap();
    match unsafe {
        winroute::WinRoute_EnsureTopMetric(
            interface_alias_ws.as_wide_c_str().as_ptr(),
            Some(ffi::error_sink),
            ptr::null_mut(),
        )
    } {
        0 => Ok(false),
        1 => Ok(true),
	-1 => Err(Error::from(ErrorKind::MetricApplication)),
        _ => {
            error!("Unexpected return code from WinRoute_EnsureTopMetric");
            Err(Error::from(ErrorKind::MetricApplication))
        }
    }
}

mod winroute {
    // 	const wchar_t *deviceAlias,
    // 	WinRouteErrorSink errorSink,
    // 	void* errorSinkContext
    use super::libc;
    use super::ffi;

    extern "system" {
        #[link_name(WinRoute_EnsureTopMetric)]
        pub fn WinRoute_EnsureTopMetric(
            tunnel_interface_alias: *const libc::wchar_t,
            sink: Option<ffi::ErrorSink>,
            sink_context: *mut libc::c_void,
        ) -> i32;
    }

}
