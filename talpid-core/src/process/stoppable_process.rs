extern crate libc;

use process::openvpn::OpenVpnProcHandle;

use std::error::Error;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

static POLL_INTERVAL_MS: u64 = 50;

/// A best effort attempt at stopping a subprocess whilst also ensuring that the subprocess is
/// killed eventually.
pub trait StoppableProcess
where
    Self: Sized,
{
    /// Gracefully stops a process.
    fn stop(&self) -> io::Result<()>;
    /// Kills a process unconditionally. Implementations should strive to never fail.
    fn kill(&self) -> io::Result<()>;
    /// Check if process is stopped. This method must not block.
    fn has_stopped(&self) -> io::Result<bool>;

    /// Attempts to stop a process gracefully in the given time period, otherwise kills the
    /// process.
    fn nice_kill(&self, timeout: Duration) -> io::Result<()> {
        trace!("Trying to stop openvpn gracefully");
        if let Err(e) = self.stop() {
            error!(
                "Failed to stop the openvpn process gracefully: {}",
                e.description()
            );
            return self.kill();
        };

        if wait_timeout(self, timeout)? {
            debug!("Child process exited from SIGTERM");
            Ok(())
        } else {
            warn!("Child process did not exit from SIGTERM, sending SIGKILL");
            self.kill()
        }
    }
}

impl StoppableProcess for OpenVpnProcHandle {
    fn stop(&self) -> io::Result<()> {
        self.try_stop()
    }

    fn kill(&self) -> io::Result<()> {
        self.inner.kill()
    }

    fn has_stopped(&self) -> io::Result<bool> {
        match self.inner.try_wait() {
            Ok(None) => Ok(false),
            Ok(Some(_)) => Ok(true),
            Err(e) => Err(e),
        }
    }
}

/// Wait for a process to die for a maximum of `timeout`. Returns true if the process died within
/// the timeout.
fn wait_timeout<T>(process: &T, timeout: Duration) -> io::Result<bool>
where
    T: StoppableProcess + Sized,
{
    let timer = Instant::now();
    while timer.elapsed() < timeout {
        if process.has_stopped()? {
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }
    Ok(false)
}
