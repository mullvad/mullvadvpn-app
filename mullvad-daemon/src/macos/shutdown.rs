//! See [start_shutdown_detection].

use core::{ffi::c_void, ptr};
use std::sync::{
    LazyLock,
    atomic::{AtomicBool, Ordering},
};

use libc::MACH_PORT_NULL;
use objc2_core_foundation::{CFRunLoop, kCFRunLoopDefaultMode};
use objc2_io_kit::{
    IODeregisterForSystemPower, IONotificationPort, IORegisterForSystemPower, IOServiceClose,
    io_object_t, io_service_t, kIOMessageSystemWillNotPowerOff, kIOMessageSystemWillPowerOff,
    kIOMessageSystemWillRestart,
};

/// Return whether a shutdown was detected.
///
/// This will be set to `true` during a system shutdown or reboot, or if detection fails for
/// whatever reason.
pub static IS_SHUTTING_DOWN: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

#[allow(non_upper_case_globals)]
unsafe extern "C-unwind" fn service_interest_callback(
    _ctx: *mut c_void,
    _io_service: io_service_t,
    msg_type: u32,
    _msg_arg: *mut c_void,
) {
    match msg_type {
        kIOMessageSystemWillPowerOff | kIOMessageSystemWillRestart => {
            log::debug!("Shutdown or reboot detected ({msg_type:x})");
            IS_SHUTTING_DOWN.store(true, Ordering::SeqCst);
        }
        kIOMessageSystemWillNotPowerOff => {
            log::debug!("Shutdown interrupted ({msg_type:x})");
            IS_SHUTTING_DOWN.store(false, Ordering::SeqCst);
        }
        r#type => log::debug!("System power event received: {type:x}"),
    }
}

/// Detect shutdown/reboot using the [`IOKit`] framework, and set [`IS_SHUTTING_DOWN`] whenever the
/// system begins a shutdown or reboot.
///
/// [`IOKit`]: https://developer.apple.com/documentation/iokit/1557114-ioregisterforsystempower
pub fn start_shutdown_detection() {
    std::thread::spawn(|| run_shutdown_detection());
}

fn run_shutdown_detection() {
    log::debug!("Registering listener for shutdown/reboot events");

    let mut notifier: io_object_t = Default::default();
    let mut notify_port = ptr::null_mut();

    // SAFETY: Context arg may be null, the other arguments are valid pointers to the expected type.
    let root_port = unsafe {
        IORegisterForSystemPower(
            ptr::null_mut(),
            &mut notify_port,
            Some(service_interest_callback),
            &mut notifier,
        )
    };

    if root_port == u32::try_from(MACH_PORT_NULL).expect("MACH_PORT_NULL is 0") {
        // If we cannot detect that the system is shutting down, we should assume that it is.
        // We would like to preserve blocking rules on shutdown or reboot unless it is user-initiated,
        // OR if we cannot tell whether it is user-initiated.
        log::error!("IORegisterForSystemPower() failed - assuming shutdown is occurring");
        IS_SHUTTING_DOWN.store(false, Ordering::SeqCst);
        return;
    }

    // SAFETY: `notify_port` is a pointer to a `IONotificationPortRef` since
    // `IORegisterForSystemPower` succeeded.
    let run_loop_source = unsafe { IONotificationPort::run_loop_source(notify_port) };

    // Set up a runloop that listens for events from `notify_port`.
    let run_loop = CFRunLoop::current().expect("expected run loop for thread");
    run_loop.add_source(
        run_loop_source.as_deref(),
        // SAFETY: This is always valid and constant.
        unsafe { kCFRunLoopDefaultMode },
    );

    // This will block forever (or until the daemon exits).
    CFRunLoop::run();

    // SAFETY: `notifier` is a valid `IONotificationPort` returned by `IORegisterForSystemPower`.
    unsafe { IODeregisterForSystemPower(&mut notifier) };

    // As the docs state, we should close `root_port` after closing the notification port.
    IOServiceClose(root_port);
}
