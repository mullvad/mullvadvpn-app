extern crate libc;
extern crate widestring;

use self::widestring::WideCString;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

error_chain!{
    errors{
        #[doc = "Failure to set metrics of network interfaces"]
        MetricApplication{
            description("Failed to set the metrics for a network interface")
        }
    }
}

pub fn ensure_metric_stuff(tunnel_alias: &str) -> Result<()> {
    let tunnel_alias_widestring =
        WideCString::new(tunnel_alias.encode_utf16().collect::<Vec<_>>()).unwrap();
    match unsafe {
        ffi::WinRoute_EnsureTopMetric(
            tunnel_alias_widestring.as_wide_c_str().as_ptr(),
            Some(error_sink),
            ptr::null_mut(),
        )
    } {
        0 => {
            debug!("No metrics were set for network interfaces, tunnel interface already has lowest metric");
            Ok(())
        }
        1 => {
            debug!("Set new metrics for interfaces");
            Ok(())
        }
        -1 => Err(Error::from(ErrorKind::MetricApplication)),
        _ => {
            error!("Unexpected return code from WinRoute_EnsureTopMetric");
            Err(Error::from(ErrorKind::MetricApplication))
        }
    }
}

pub type ErrorSink = extern "system" fn(msg: *const c_char, ctx: *mut libc::c_void);

extern "system" fn error_sink(msg: *const c_char, _ctx: *mut libc::c_void) {
    if msg == ptr::null() {
        error!("log message from WinFw is NULL");
    } else {
        error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
    }
}

mod ffi {
    // 	const wchar_t *deviceAlias,
    // 	WinRouteErrorSink errorSink,
    // 	void* errorSinkContext
    use super::libc;
    use super::ErrorSink;

    extern "system" {
        #[link_name(WinRoute_EnsureTopMetric)]
        pub fn WinRoute_EnsureTopMetric(
            tunnel_interface_alias: *const libc::wchar_t,
            sink: Option<ErrorSink>,
            sink_context: *mut libc::c_void,
        ) -> i32;
    }

}
