//! Install signal handlers to catch critical program faults and log them. See [`enable`].
#![warn(clippy::undocumented_unsafe_blocks)]

use libc::siginfo_t;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};

use core::fmt::{self};
use std::{
    backtrace::Backtrace,
    env,
    ffi::{c_int, c_void, CString},
    os::fd::{FromRawFd, RawFd},
    sync::{
        atomic::{AtomicBool, Ordering},
        Once, OnceLock,
    },
};

/// Write fault to this file.
static LOG_FILE_PATH: OnceLock<CString> = OnceLock::new();

/// If true, the signal-handler will log a backtrace when triggered.
///
/// Default value can be overridden using the env var [ENABLE_BACKTRACE_VAR].
///
/// # Safety
/// Printing a backtrace from the signal-handler is potentially unsound.
/// Therefore, this is only enabled by default in debug-builds.
static ENABLE_BACKTRACE: AtomicBool = AtomicBool::new(cfg!(debug_assertions));

/// Name of the environment variable that sets [ENABLE_BACKTRACE].
const ENABLE_BACKTRACE_VAR: &str = "MULLVAD_BACKTRACE_ON_FAULT";

/// The signals we install handlers for.
const FAULT_SIGNALS: [Signal; 5] = [
    // Access to invalid memory address
    Signal::SIGBUS,
    // Floating point exception
    Signal::SIGFPE,
    // Illegal instructors
    Signal::SIGILL,
    // Invalid memory reference
    Signal::SIGSEGV,
    // Bad syscall
    Signal::SIGSYS,
];

/// Set the file path used for fault handler logging.
///
/// Panics if called more than once.
pub fn set_log_file(file_path: impl Into<CString>) {
    if let Err(_file_path) = LOG_FILE_PATH.set(file_path.into()) {
        panic!("set_log_file may not be called more than once");
    }
}

/// Install signal handlers to catch critical program faults, log them, and exit the process.
pub fn enable() {
    static INIT_ONCE: Once = Once::new();

    INIT_ONCE.call_once(|| {
        if let Ok(override_backtrace) = env::var(ENABLE_BACKTRACE_VAR).map(|v| v != "0") {
            ENABLE_BACKTRACE.store(override_backtrace, Ordering::Release);
        }

        // XXX: SA_ONSTACK tells the signal handler to use an alternate stack, if one is available.
        //      The purpose of an alternate stack is to have somewhere to execute the signal handler
        //      in the case of a stack overflow. I.e. if an alternate stack hasn't been configured,
        //      stack overflows will silently cause the process to exit with code SIGSEGV.
        //
        // XXX: `libc::sigaltstack` can be used to set up an alternate stack on the current thread.
        //       The default behaviour of the Rust runtime is to set up alternate stacks for the
        //       main thread, and for every thread spawned using `std::thread`. However, Rust will
        //       not do this if any signal handlers have been configured before the Rust runtime is
        //       initialized (note: the initialization happens before main() is called). For
        //       example, if any Go code is linked into this binary, the Go runtime will probably
        //       be initialized first, and will set up it's own signal handlers.
        //
        // XXX: Go requires this flag to be set for all signal handlers.
        //      https://github.com/golang/go/blob/d6fb0ab2/src/os/signal/doc.go
        let sig_handler_flags = SaFlags::SA_ONSTACK;

        let signal_action = SigAction::new(
            SigHandler::SigAction(fault_handler),
            sig_handler_flags,
            SigSet::empty(),
        );

        for signal in &FAULT_SIGNALS {
            // SAFETY: `fault_handler` is signal-safe.
            if let Err(err) = unsafe { sigaction(*signal, &signal_action) } {
                log::error!("Failed to install signal handler for {}: {}", signal, err);
            }
        }
    });
}

/// Signal handler to catch signals that are used to indicate unrecoverable errors in the daemon.
extern "C" fn fault_handler(
    signum: c_int,
    _siginfo: *mut siginfo_t,
    _thread_context_ptr: *mut c_void,
) {
    // XXX: This function is a signal handler, meaning it must be signal safe.
    // For a detailed definition, see https://man7.org/linux/man-pages/man7/signal-safety.7.html
    // The short version is:
    // - This function must be re-entrant.
    // - This function must only call functions that are signal-safe.
    // - The man-page provides a list of posix-functions that are signal-safe. (These can be found
    //   in the `libc`-crate)

    // 128 + signum is the "standard" set by bash.
    let signum_code: c_int = signum.saturating_add(0x80);

    let code: c_int = match log_fault_to_file_and_stdout(signum) {
        // Signal numbers are positive integers
        Ok(()) => signum_code,

        // map error to error-codes
        Err(err) => match err {
            FaultHandlerErr::UnknownSignal => signum_code,
            FaultHandlerErr::Open => 2,
            FaultHandlerErr::Write => 3,
            FaultHandlerErr::FSync => 4,
            FaultHandlerErr::Reentrancy => 5,
        },
    };

    // SIGNAL-SAFETY: This function is listed in `man 7 signal-safety`.
    // SAFETY: This function is trivially safe to call.
    unsafe { libc::_exit(code) }
}

/// Call from a signal handler to try to print the signal (and optionally a backtrace).
///
/// The output is written to stdout, and to a file if [set_exception_logging_file] was called.
fn log_fault_to_file_and_stdout(signum: c_int) -> Result<(), FaultHandlerErr> {
    // XXX: This function must be signal-safe. See notes in `fn fault_handler`

    // Guard against reentrancy, which can happen if this fault handler triggers another fault.
    static REENTRANT: AtomicBool = AtomicBool::new(false);
    if REENTRANT.swap(true, Ordering::SeqCst) {
        return Err(FaultHandlerErr::Reentrancy);
    }

    // SAFETY: calling `write` on stdout is always safe, even if it's been closed.
    let stdout = unsafe { LibcWriter::from_raw_fd(libc::STDOUT_FILENO) };
    log_fault_to_writer(signum, stdout)?;

    // SIGNAL-SAFETY: OnceLock::get is atomic and non-blocking.
    if let Some(log_file) = LOG_FILE_PATH.get() {
        let open_flags = libc::O_WRONLY | libc::O_APPEND;

        // SIGNAL-SAFETY: This function is listed in `man 7 signal-safety`.
        // SAFETY: `path` is a null-terminated string.
        let log_file: RawFd = match unsafe { libc::open(log_file.as_ptr(), open_flags) } {
            ..0 => return Err(FaultHandlerErr::Open),
            fd => fd,
        };

        // SAFETY: `log_file` is an open file descriptor; it's not closed until the process exits.
        let mut log_file = unsafe { LibcWriter::from_raw_fd(log_file) };

        log_fault_to_writer(signum, &mut log_file)?;
        log_file.flush()?;
    }

    Ok(())
}

/// Call from a signal handler to try to write the signal (and optionally a backtrace).
///
/// The output is written to the writer passed in as an argument.
fn log_fault_to_writer(signum: c_int, mut w: impl fmt::Write) -> Result<(), FaultHandlerErr> {
    // SIGNAL-SAFETY: Signal::try_from(i32) is signal-safe
    let signal: Signal = match Signal::try_from(signum) {
        Ok(signal) => signal,
        Err(_) => {
            // SIGNAL-SAFETY: formatting an i32 is signal-safe.
            writeln!(w, "Signal handler triggered by unknown signal: {signum}")?;
            return Err(FaultHandlerErr::UnknownSignal);
        }
    };

    // SIGNAL-SAFETY:
    //   `writeln` resolves to calls to <LibcWriter as io::Write>::write, which is signal-safe.
    //   as_str is const and formatting a &str is signal-safe.
    writeln!(w, "Caught signal {}", signal.as_str())?;

    // Formatting a `Backtrace` is NOT signal-safe.
    if ENABLE_BACKTRACE.load(Ordering::Acquire) {
        writeln!(w, "Backtrace:")?;
        writeln!(w, "{}", Backtrace::force_capture())?;
    } else {
        writeln!(w, "Set {ENABLE_BACKTRACE_VAR}=1 to print backtrace.")?
    }

    Ok(())
}

enum FaultHandlerErr {
    /// Signal handler received an unknown signal number.
    UnknownSignal,

    /// A call to `libc::open` failed.
    Open,

    /// A call to `libc::write` failed.
    Write,

    /// A call to `libc::fsync` failed.
    FSync,

    /// Signal handler was called reentrantly.
    Reentrancy,
}

/// A wrapper type that implements `fmt::Write` for a file descriptor through `libc::write`.
struct LibcWriter {
    file_descriptor: RawFd,
}

impl LibcWriter {
    /// Call `libc::fsync` on the file descriptor.
    pub fn flush(&self) -> Result<(), FaultHandlerErr> {
        // SAFETY: This function is trivially safe to call
        match unsafe { libc::fsync(self.file_descriptor) } {
            ..0 => Err(FaultHandlerErr::FSync),
            _ => Ok(()),
        }
    }
}

impl FromRawFd for LibcWriter {
    /// Wrap a file descriptor in a [LibcWriter].
    ///
    /// # Safety
    /// The file descriptor must refer to an opened file, or to stdout/stderr.
    /// In the case of a file, it must remain open for the lifetime of the [LibcWriter].
    unsafe fn from_raw_fd(file_descriptor: RawFd) -> Self {
        Self { file_descriptor }
    }
}

impl fmt::Write for LibcWriter {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        let mut bytes = string.as_bytes();
        while !bytes.is_empty() {
            let ptr = bytes.as_ptr() as *const c_void;

            // SAFETY:
            // - `self.file_descriptor` is an open file descriptor.
            // - `ptr` points to the start of `bytes`
            let n = match unsafe { libc::write(self.file_descriptor, ptr, bytes.len()) } {
                n if n > 0 => n as usize,
                _ => return Err(fmt::Error),
            };

            bytes = &bytes[n..];
        }

        Ok(())
    }
}

impl From<fmt::Error> for FaultHandlerErr {
    /// Convert a [fmt::Error] into a [FaultHandlerErr::Write].
    /// [fmt::Error] is returned by [fmt::Write]-methods.
    fn from(_: fmt::Error) -> Self {
        FaultHandlerErr::Write
    }
}
