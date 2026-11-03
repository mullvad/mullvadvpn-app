use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
#[cfg(target_os = "windows")]
use tokio::signal::windows::{ctrl_break, ctrl_close, ctrl_logoff, ctrl_shutdown};

pub fn install_shutdown_signal_handler(daemon: &crate::Daemon) {
    let shutdown_handle = daemon.shutdown_handle();

    tokio::spawn(async move {
        cfg_select! {
            target_os = "windows" => {
                let mut ctrl_break = ctrl_break().unwrap();
                let mut ctrl_close = ctrl_close().unwrap();
                let mut ctrl_logoff = ctrl_logoff().unwrap();
                let mut ctrl_shutdown = ctrl_shutdown().unwrap();
                tokio::select! {
                    _ = ctrl_c() => {}
                    _ = ctrl_break.recv() => {}
                    _ = ctrl_close.recv() => {}
                    _ = ctrl_logoff.recv() => {}
                    _ = ctrl_shutdown.recv() => {}
                }
            }
            unix => {
                let mut sigint = signal(SignalKind::interrupt()).unwrap();
                let mut sigterm = signal(SignalKind::terminate()).unwrap();
                let mut sighup = signal(SignalKind::hangup()).unwrap();
                tokio::select! {
                    _ = ctrl_c() => {}
                    _ = sigint.recv() => {}
                    _ = sigterm.recv() => {}
                    _ = sighup.recv() => {}
                }
            }
        }
        let user_init_shutdown = is_shutdown_user_initiated().await;
        shutdown_handle.shutdown(user_init_shutdown);
    });
}

/// Returns true if systemd successfully reported that the machine is not shutting down or entering
/// maintenance. If obtaining this information fails, the return value will be `false` and it will
/// be assumed that the machine is shutting down.
#[cfg(target_os = "linux")]
pub async fn is_shutdown_user_initiated() -> bool {
    use talpid_error::ErrorExt;
    talpid_dbus::systemd::is_host_running()
        .await
        .map_err(|err| {
            err.display_chain_with_msg(
                "Failed to determine if host is shutting down, assuming it is shutting down",
            )
        })
        .inspect_err(|err| log::error!("{err}"))
        .unwrap_or(false)
}

/// Currently returns false all of the time to ensure that no leaks occur during shutdown.
// !!! Read before you think of implementing shutdown detection.
//
// I've tried using the IOKit API for this, but the launch daemon seemingly only receives a SIGTERM
// during shutdown/reboot.
//
// If you're thinking, "Ah, then I will attempt to use the undocumented BSD notification
// `com.apple.system.loginwindow.shutdownInitiated`": No. Same issue.
//
// It might be that I'm missing something, but neither of these two methods appears to work. You
// have been warned.
#[cfg(target_os = "macos")]
#[expect(clippy::unused_async)]
pub async fn is_shutdown_user_initiated() -> bool {
    false
}

/// Currently returns true all of the time.
#[cfg(any(target_os = "windows", target_os = "android"))]
#[expect(clippy::unused_async)]
pub async fn is_shutdown_user_initiated() -> bool {
    true
}

/// Install a signal handler for `SIGUSR1`.
///
/// The signal handler will request [`crate::Daemon`] to shut down without tearing down the
/// firewall. This is useful for when the daemon is expected to only be _temporarily_ shut down,
/// such as when rebooting or updating the system.
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
