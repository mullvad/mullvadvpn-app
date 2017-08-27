extern crate libc;

use duct;
use duct::unix::HandleExt;

use std::io;
use std::thread;
use std::time::{Duration, Instant};

static POLL_INTERVAL_MS: u64 = 50;

/// Extra methods for terminating `duct::Handle` instances.
pub trait HandleKillExt {
    /// Kills a process by first sending it the `SIGTERM` signal and then wait up to `timeout`.
    /// If the process has not died after the timeout has expired it is killed.
    fn nice_kill(&self, timeout: Duration) -> io::Result<()>;
}

impl HandleKillExt for duct::Handle {
    fn nice_kill(&self, timeout: Duration) -> io::Result<()> {
        trace!("Sending SIGTERM to child process");
        self.send_signal(libc::SIGTERM)?;

        if wait_timeout(self, timeout)? {
            debug!("Child process exited from SIGTERM");
            Ok(())
        } else {
            debug!("Child process did not exit from SIGTERM, sending SIGKILL");
            self.kill()
        }
    }
}

/// Wait for a process to die for a maximum of `timeout`. Returns true if the process died within
/// the timeout.
fn wait_timeout(handle: &duct::Handle, timeout: Duration) -> io::Result<bool> {
    let timer = Instant::now();
    while timer.elapsed() < timeout {
        match handle.try_wait() {
            Ok(None) => thread::sleep(Duration::from_millis(POLL_INTERVAL_MS)),
            Ok(Some(_)) => return Ok(true),
            Err(e) => return Err(e),
        }
    }
    Ok(false)
}
