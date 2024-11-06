//! Installs signal handlers to catch critical program faults and logs them.

#[cfg(debug_assertions)]
use libc::siginfo_t;
#[cfg(debug_assertions)]
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};

#[cfg(debug_assertions)]
use std::{
    backtrace::Backtrace,
    ffi::{c_int, c_void},
    sync::{
        atomic::{AtomicBool, Ordering},
        Once,
    },
};

#[cfg(debug_assertions)]
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

/// Installs a signal handler.
pub fn enable() {
    #[cfg(debug_assertions)]
    static INIT_ONCE: Once = Once::new();

    #[cfg(debug_assertions)]
    INIT_ONCE.call_once(|| {
        // Setup alt stack for signal handlers to be executed in.
        // If the daemon ever needs to be compiled for architectures where memory can't be writeable
        // and executable, the following block of code has to be disabled. This will also mean that
        // stack overflows may be silent and undetectable in logs.
        let sig_handler_flags = {
            // The kernel will use the first properly aligned address, so alignment is not an issue.
            let alt_stack = vec![0u8; libc::SIGSTKSZ];
            let stack_t = libc::stack_t {
                ss_sp: alt_stack.as_ptr() as *mut c_void,
                ss_flags: 0,
                ss_size: alt_stack.len(),
            };
            let ret = unsafe { libc::sigaltstack(&stack_t, std::ptr::null_mut()) };
            if ret != 0 {
                log::error!(
                    "Failed to set alternative stack: {}",
                    std::io::Error::last_os_error()
                );
                SaFlags::empty()
            } else {
                std::mem::forget(alt_stack);
                SaFlags::SA_ONSTACK
            }
        };

        let signal_action = SigAction::new(
            SigHandler::SigAction(fault_handler),
            sig_handler_flags,
            SigSet::empty(),
        );

        for signal in &FAULT_SIGNALS {
            // SAFETY: fault_handler is NOT signal-safe (see logging_fault_handler). We still use
            // it, but only in development builds because it makes debugging easier.
            if let Err(err) = unsafe { sigaction(*signal, &signal_action) } {
                log::error!("Failed to install signal handler for {}: {}", signal, err);
            }
        }
    });
}

/// Signal handler to catch signals that are used to indicate unrecoverable errors in the daemon.
#[cfg(debug_assertions)]
extern "C" fn fault_handler(
    signum: c_int,
    _siginfo: *mut siginfo_t,
    _thread_context_ptr: *mut c_void,
) {
    // SAFETY: This function is known to be potentially unsound and should not be used in prod,
    // but we keep it in debug-builds because debugging SIGSEGV faults is a PITA.
    // See logging_fault_handler docs for more info.
    unsafe { logging_fault_handler(signum) };
}

/// Call from a signal handler to [log] the signal, and the current backtrace.
///
/// See also: [fault_handler].
///
/// # SAFETY
/// Calling this function from a signal handler is potentially unsound. This is because is performs
/// functions that are not "signal-safe", for example: `process::exit` and writing to to stdout.
/// See also: <https://man7.org/linux/man-pages/man7/signal-safety.7.html>.
///
/// This function is `cfg(debug_assertions)` because it should not be used in production builds.
// TODO: Consider rewriting this function to e.g. use a pipe to exfiltrate the backtrace to another
// process that can write the backtrace to tho log file. `write` is signal-safe.
#[cfg(debug_assertions)]
unsafe fn logging_fault_handler(signum: c_int) {
    // Guard against reentrancy, which can happen if this fault handler triggers another fault.
    static REENTRANCY_GUARD: AtomicBool = AtomicBool::new(false);
    if REENTRANCY_GUARD.swap(true, Ordering::SeqCst) {
        // `process::abort` is signal-safe, unlike `process::exit`.
        std::process::abort();
    }

    let signal: Signal = match Signal::try_from(signum) {
        Ok(signal) => signal,
        Err(err) => {
            log::error!(
                "Signal handler triggered by unknown signal {}, exiting: {}",
                signum,
                err
            );
            std::process::exit(2);
        }
    };

    log::error!("Caught signal {}", signal);
    log::error!("Backtrace:");
    for line in format!("{}", Backtrace::force_capture()).lines() {
        log::error!("{line}");
    }
    std::process::exit(2);
}
