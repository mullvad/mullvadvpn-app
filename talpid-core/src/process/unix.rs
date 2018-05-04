extern crate libc;

use process::proc_handle::OpenVpnProcHandle;

use duct;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

static POLL_INTERVAL_MS: u64 = 50;

/// Extra methods for terminating `OpenVpnProcHandle` instances.
pub trait HandleKillExt {
    /// Kills a process by first sending it the `SIGTERM` signal and then wait up to `timeout`.
    /// If the process has not died after the timeout has expired it is killed.
    fn nice_kill(&self, timeout: Duration) -> io::Result<()>;
}

impl HandleKillExt for OpenVpnProcHandle {
    fn nice_kill(&self, timeout: Duration) -> io::Result<()> {
        trace!("Sending SIGTERM to child process");
        self.try_stop()?;

        if wait_timeout(&self.inner, timeout)? {
            debug!("Child process exited from SIGTERM");
            Ok(())
        } else {
            warn!("Child process did not exit from SIGTERM, sending SIGKILL");
            self.kill_process()
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
