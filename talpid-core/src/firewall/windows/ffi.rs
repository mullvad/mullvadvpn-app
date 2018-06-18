extern crate libc;

use std::os::raw::c_char;
use std::ptr;

pub type ErrorSink = extern "system" fn(msg: *const c_char, ctx: *mut libc::c_void);

pub extern "system" fn error_sink(msg: *const c_char, _ctx: *mut libc::c_void) {
    use std::ffi::CStr;
    if msg == ptr::null() {
        error!("Log message from FFI boundary is NULL");
    } else {
        error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
    }
}

#[macro_export]
macro_rules! ffi_error {
    ($result:ident, $error:expr) => {
        pub mod $result {
            use super::*;

            #[repr(C)]
            #[derive(Debug)]
            pub struct FFIResult {
                success: bool,
            }

            impl FFIResult {
                pub fn into_result(self) -> Result<()> {
                    match self.success {
                        true => Ok(()),
                        false => Err($error),
                    }
                }
            }

            impl Into<Result<()>> for FFIResult {
                fn into(self) -> Result<()> {
                    self.into_result()
                }
            }
        }
    };
}
