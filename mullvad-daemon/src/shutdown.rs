#[cfg(unix)]
mod platform {
    use simple_signal::Signal;
    use std::io;

    pub fn set_shutdown_signal_handler(f: impl Fn() + 'static + Send) -> Result<(), io::Error> {
        simple_signal::set_handler(&[Signal::Term, Signal::Int], move |s| {
            log::debug!("Process received signal: {:?}", s);
            f();
        });
        Ok(())
    }
}

#[cfg(windows)]
mod platform {
    #[derive(err_derive::Error, Debug)]
    #[error(display = "Unable to attach ctrl-c handler")]
    pub struct Error(#[error(source)] ctrlc::Error);

    pub fn set_shutdown_signal_handler(f: impl Fn() + 'static + Send) -> Result<(), Error> {
        ctrlc::set_handler(move || {
            log::debug!("Process received Ctrl-c");
            f();
        })
        .map_err(Error)
    }
}

/// Returns true if systemd successfully reported that the machine is not shutting down or entering
/// maintenance. If obtaining this information fails, the return value will be `false` and it will
/// be assumed that the machine is shutting down.
#[cfg(target_os = "linux")]
pub fn is_shutdown_user_initiated() -> bool {
    match talpid_dbus::systemd::is_host_running() {
        Ok(is_host_running) => is_host_running,
        Err(err) => {
            log::error!(
                "{}",
                talpid_types::ErrorExt::display_chain_with_msg(
                    &err,
                    "Failed to determine if host is shutting down, assuming it is shutting down"
                )
            );
            false
        }
    }
}

/// Currently returns false all of the time to ensure that no leaks occur during shutdown.
// TODO: implement shutdown detection
#[cfg(target_os = "macos")]
pub fn is_shutdown_user_initiated() -> bool {
    false
}

pub use self::platform::*;
