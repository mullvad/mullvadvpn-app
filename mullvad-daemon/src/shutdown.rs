use ctrlc;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Unable to attach ctrl-c handler")]
pub struct Error(#[from] ctrlc::Error);

pub fn set_shutdown_signal_handler(f: impl Fn() + 'static + Send) -> Result<(), Error> {
    ctrlc::set_handler(f)?;
    Ok(())
}

/// Returns true if systemd successfully reported that the machine is not shutting down or entering
/// maintenance. If obtaining this information fails, the return value will be `false` and it will
/// be assumed that the machine is shutting down.
#[cfg(target_os = "linux")]
pub fn is_shutdown_user_initiated() -> bool {
    use talpid_types::ErrorExt;
    talpid_dbus::systemd::is_host_running()
        .map_err(|err| {
            err.display_chain_with_msg(
                "Failed to determine if host is shutting down, assuming it is shutting down",
            )
        })
        .inspect_err(|err| log::error!("{err}"))
        .unwrap_or(false)
}

/// Currently returns false all of the time to ensure that no leaks occur during shutdown.
// FIXME: implement shutdown detection - the current implementation will always block network
// traffic when the daemon is shut down.
#[cfg(target_os = "macos")]
pub fn is_shutdown_user_initiated() -> bool {
    false
}

/// Install a signal handler for `SIGUSR1`.
///
/// The signal handler will request [`Daemon`] to shut down without tearing down the firewall.
/// This is useful for when the daemon is expected to only be _temporarily_ shut down, such a
/// when rebooting or updating the system.
#[cfg(target_os = "linux")]
pub fn install_sigusr1_shutdown_handler(daemon: &crate::Daemon) -> Result<(), String> {
    use tokio::signal::unix::{SignalKind, signal};

    let commands = daemon.commands();
    let mut sigusr1 = signal(SignalKind::user_defined1())
        .map_err(|e| format!("Failed to install SIGUSR1 signal handler: {e:?}"))?;

    tokio::spawn(async move {
        use talpid_core::mpsc::Sender;

        sigusr1.recv().await;
        log::warn!("SIGUSR1 caught, shutting down.");
        commands.send(crate::DaemonCommand::PrepareRestart { shutdown: true })
    });

    Ok(())
}
