use std::{error::Error, fmt, fmt::Write};

/// Used to generate string representations of error chains.
pub trait ErrorExt {
    /// Creates a string representation of the entire error chain.
    fn display_chain(&self) -> String;

    /// Like [Self::display_chain] but with an extra message at the start of the chain
    fn display_chain_with_msg(&self, msg: &str) -> String;
}

impl<E: Error> ErrorExt for E {
    fn display_chain(&self) -> String {
        let mut s = format!("Error: {self}");
        let mut source = self.source();
        while let Some(error) = source {
            write!(&mut s, "\nCaused by: {error}").expect("formatting failed");
            source = error.source();
        }
        s
    }

    fn display_chain_with_msg(&self, msg: &str) -> String {
        let mut s = format!("Error: {msg}\nCaused by: {self}");
        let mut source = self.source();
        while let Some(error) = source {
            write!(&mut s, "\nCaused by: {error}").expect("formatting failed");
            source = error.source();
        }
        s
    }
}

#[derive(Debug)]
pub struct BoxedError(Box<dyn Error + 'static + Send>);

impl fmt::Display for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for BoxedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl BoxedError {
    pub fn new(error: impl Error + 'static + Send) -> Self {
        BoxedError(Box::new(error))
    }
}

/// Helper macro allowing simpler handling of Windows FFI returning `WIN32_ERROR`
/// status codes. Converts a `WIN32_ERROR` into an `io::Result<()>`.
///
/// The caller of this macro must have `windows_sys` as a dependency.
#[cfg(windows)]
#[macro_export]
macro_rules! win32_err {
    ($expr:expr) => {{
        let status = $expr;
        if status == ::windows_sys::Win32::Foundation::NO_ERROR {
            Ok(())
        } else {
            Err(::std::io::Error::from_raw_os_error(status as i32))
        }
    }};
}
