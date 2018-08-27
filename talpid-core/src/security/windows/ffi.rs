use libc::{c_char, c_void};

pub type ErrorSink = extern "system" fn(msg: *const c_char, ctx: *mut c_void);

pub extern "system" fn error_sink(msg: *const c_char, _ctx: *mut c_void) {
    use std::ffi::CStr;
    if msg.is_null() {
        error!("Log message from FFI boundary is NULL");
    } else {
        error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
    }
}

#[macro_export]
macro_rules! ffi_error {
    ($result:ident, $error:expr) => {
        #[repr(C)]
        #[derive(Debug)]
        pub struct $result {
            success: bool,
        }

        impl $result {
            pub fn into_result(self) -> Result<()> {
                match self.success {
                    true => Ok(()),
                    false => Err($error),
                }
            }
        }

        impl Into<Result<()>> for $result {
            fn into(self) -> Result<()> {
                self.into_result()
            }
        }
    };
}
