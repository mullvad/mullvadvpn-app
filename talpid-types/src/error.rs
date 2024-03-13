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

pub mod flood {
    #[doc(hidden)]
    pub use log;

    use std::time::{Duration, Instant};

    const CALLS_INTERVAL: Duration = Duration::from_secs(5);
    const CALLS_THRESHOLD: usize = 1000;

    /// Log when a line is hit unusually frequently, that is, over `CALLS_THRESHOLD` times within a
    /// period of `CALLS_INTERVAL`.
    #[macro_export]
    macro_rules! detect_flood {
        () => {{
            static FLOOD: ::std::sync::Mutex<$crate::flood::DetectFlood> =
                ::std::sync::Mutex::new($crate::flood::DetectFlood::new());
            if FLOOD.lock().unwrap().bump() {
                $crate::flood::log::warn!(
                    "Flood: {}, line {}, col {}",
                    file!(),
                    line!(),
                    column!()
                );
            }
        }};
    }

    /// Used to detect code that is running too frequently
    pub struct DetectFlood {
        last_clear: Option<Instant>,
        counter: usize,
    }

    impl DetectFlood {
        pub const fn new() -> Self {
            DetectFlood {
                last_clear: None,
                counter: 0,
            }
        }

        pub fn bump(&mut self) -> bool {
            let now = Instant::now();
            let last_clear = self.last_clear.get_or_insert(now);
            if now.saturating_duration_since(*last_clear) >= CALLS_INTERVAL {
                self.last_clear = Some(now);
                self.counter = 0;
                false
            } else {
                self.counter = self.counter.saturating_add(1);
                self.counter == CALLS_THRESHOLD
            }
        }
    }
}
