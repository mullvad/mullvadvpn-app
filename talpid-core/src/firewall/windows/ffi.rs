/// Creates a new result type that returns the given result variant on error.
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
