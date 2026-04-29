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
