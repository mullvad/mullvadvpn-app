use libc::{c_int, c_void, siginfo_t};
use nix::sys::signal::*;

/// Installs a signal handler.
pub fn enable() {
    log::debug!("ENABLING EXCEPTION HANDLING");

    let mut signals = SigSet::empty();
    // Access to invalid memory address
    signals.add(Signal::SIGBUS);
    // Floating point exception
    signals.add(Signal::SIGFPE);
    // Illegal instructors
    signals.add(Signal::SIGILL);
    // Invalid memory reference
    signals.add(Signal::SIGSEGV);
    // Bad syscall
    signals.add(Signal::SIGSYS);


    // let handler = ;
    let signal_action = SigAction::new(
        SigHandler::SigAction(fault_handler),
        SaFlags::empty(),
        signals,
    );
}

/// Signal handler to catch signals that are used to indicate unrecoverable errors in the daemon
extern "C" fn fault_handler(
    signum: c_int,
    siginfo: *mut siginfo_t,
    thread_context: *mut c_void,
) {
    log::error!("GOT SIGNAL - {}", signum);
    panic!("FAILING WITH SIGNAL!");
}
