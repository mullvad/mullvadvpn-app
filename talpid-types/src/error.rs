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
    use std::time::{Duration, Instant};

    const CALLS_INTERVAL: Duration = Duration::from_secs(5);
    const CALLS_THRESHOLD: usize = 1000;

    #[macro_export]
    macro_rules! detect_flood {
        () => {{
            static FLOOD: ::once_cell::sync::Lazy<
                ::std::sync::Mutex<talpid_types::flood::DetectFlood>,
            > = ::once_cell::sync::Lazy::new(|| {
                ::std::sync::Mutex::new(talpid_types::flood::DetectFlood::default())
            });
            if FLOOD.lock().unwrap().bump() {
                ::log::debug!("Flood: {}, line {}, col {}", file!(), line!(), column!());
            }
        }};
    }

    /// Used to detect code that is running too frequently
    pub struct DetectFlood {
        last_clear: Instant,
        counter: usize,
    }

    impl Default for DetectFlood {
        fn default() -> Self {
            DetectFlood {
                last_clear: Instant::now(),
                counter: 0,
            }
        }
    }

    impl DetectFlood {
        pub fn bump(&mut self) -> bool {
            let now = Instant::now();
            if now.saturating_duration_since(self.last_clear) >= CALLS_INTERVAL {
                self.last_clear = now;
                self.counter = 0;
            } else {
                let was_less = self.counter < CALLS_THRESHOLD;
                self.counter = self.counter.saturating_add(1);
                return was_less && self.counter >= CALLS_THRESHOLD;
            }
            false
        }
    }
}
