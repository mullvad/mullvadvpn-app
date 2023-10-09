use std::{
    io, thread,
    time::{Duration, Instant},
};

static POLL_INTERVAL_MS: Duration = Duration::from_millis(50);

/// A best effort attempt at stopping a subprocess whilst also ensuring that the subprocess is
/// killed eventually.
#[async_trait::async_trait]
pub trait StoppableProcess
where
    Self: Sized,
{
    /// Gracefully stops a process.
    async fn stop(&self);

    /// Kills a process unconditionally. Implementations should strive to never fail.
    async fn kill(&self) -> io::Result<()>;

    /// Check if process is stopped. This method must not block.
    async fn has_stopped(&self) -> io::Result<bool>;

    /// Attempts to stop a process gracefully in the given time period, otherwise kills the
    /// process.
    async fn nice_kill(&self, timeout: Duration) -> io::Result<()> {
        log::debug!("Trying to stop child process gracefully");
        self.stop().await;
        if wait_timeout(self, timeout).await? {
            log::debug!("Child process terminated gracefully");
        } else {
            log::warn!(
                "Child process did not terminate gracefully within timeout, forcing termination"
            );
            self.kill().await?;
        }
        Ok(())
    }
}
/// Wait for a process to die for a maximum of `timeout`. Returns true if the process died within
/// the timeout.
async fn wait_timeout<T>(process: &T, timeout: Duration) -> io::Result<bool>
where
    T: StoppableProcess + Sized,
{
    let timer = Instant::now();
    while timer.elapsed() < timeout {
        if process.has_stopped().await? {
            return Ok(true);
        }
        thread::sleep(POLL_INTERVAL_MS);
    }
    Ok(false)
}
