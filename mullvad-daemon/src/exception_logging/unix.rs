//! Installs signal handlers to catch critical program faults and logs them.

use libc::siginfo_t;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};

use std::{
    ffi::{c_int, c_void},
    sync::Once,
};

static INIT_ONCE: Once = Once::new();

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
            if let Err(err) = unsafe { sigaction(*signal, &signal_action) } {
                log::error!("Failed to install signal handler for {}: {}", signal, err);
            }
        }
    });
}

/// Signal handler to catch signals that are used to indicate unrecoverable errors in the daemon
extern "C" fn fault_handler(
    signum: c_int,
    _siginfo: *mut siginfo_t,
    _thread_context_ptr: *mut c_void,
) {
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
    std::process::exit(2);
}
