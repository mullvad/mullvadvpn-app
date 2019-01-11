/// Creates a new result type that returns the given result variant on error.
#[macro_export]
/// Defines a type to be used by FFI functions that return a boolean value to indicate a failure.
/// If the return value is true, the result unwraps to an `Ok(())`, otherwise `Err(ErrorType)`.
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
