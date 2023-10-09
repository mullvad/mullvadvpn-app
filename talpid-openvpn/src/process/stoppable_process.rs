use std::{io, process::ExitStatus, time::Duration};

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

    /// Wait until a process is stopped.
    async fn wait(&self) -> io::Result<ExitStatus>;

    /// Attempts to stop a process gracefully in the given time period, otherwise kills the
    /// process.
    async fn nice_kill(&self, timeout: Duration) -> io::Result<()> {
        log::debug!("Trying to stop child process gracefully");
        self.stop().await;

        // Wait for the process to die for a maximum of `timeout`.
        let wait_result = tokio::time::timeout(timeout, self.wait()).await;
        match wait_result {
            Ok(_) => log::debug!("Child process terminated gracefully"),
            Err(_) => {
                log::warn!(
                "Child process did not terminate gracefully within timeout, forcing termination"
            );
                self.kill().await?;
            }
        }
        Ok(())
    }
}
