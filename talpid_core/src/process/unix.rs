extern crate libc;

use duct;
use duct::unix::HandleExt;

use std::io;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

/// Kills a process by first sending it the `SIGTERM` signal and then wait up to `timeout`. If the
/// process has not died after the timeout has expired it is killed.
pub fn nice_kill(handle: Arc<duct::Handle>, timeout: Duration) -> io::Result<()> {
    trace!("Sending SIGTERM to child process");
    handle.send_signal(libc::SIGTERM)?;

    if wait_timeout(handle.clone(), timeout) {
        debug!("Child process exited from SIGTERM");
        Ok(())
    } else {
        debug!("Child process did not exit from SIGTERM, sending SIGKILL");
        handle.kill()
    }
}

/// Wait for a process to die for a maximum of `timeout`. Returns true if the process died within
/// the timeout. Warning, if the process does not exit in the given time, this function will leave
/// a thread running until it does exit.
fn wait_timeout(handle: Arc<duct::Handle>, timeout: Duration) -> bool {
    let (stop, stopped) = mpsc::channel();
    thread::spawn(move || { let _ = stop.send(handle.wait().is_ok()); });
    stopped.recv_timeout(timeout).unwrap_or(false)
}
