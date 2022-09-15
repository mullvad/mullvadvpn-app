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

#[cfg(target_os = "linux")]
pub fn is_host_shutting_down() -> bool {
    match talpid_dbus::systemd::is_host_shutting_down() {
        Ok(is_shutting_down) => is_shutting_down,
        Err(err) => {
            log::error!(
                "{}",
                talpid_types::ErrorExt::display_chain_with_msg(
                    &err,
                    "Failed to determine if host is shutting down, assuming it is shutting down"
                )
            );
            true
        }
    }
}

pub use self::platform::*;
