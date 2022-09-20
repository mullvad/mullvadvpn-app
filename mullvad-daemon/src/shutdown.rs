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
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(err_derive::Error, Debug)]
    #[error(display = "Unable to attach ctrl-c handler")]
    pub struct Error(#[error(source)] ctrlc::Error);

    static mut SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

    pub fn set_shutdown_signal_handler(f: impl Fn() + 'static + Send) -> Result<(), Error> {
        ctrlc::set_handler(move || {
            log::debug!("Process received Ctrl-c");
            f();
        })
        .map_err(Error)
    }

    #[allow(dead_code)]
    pub fn is_host_shutting_down() -> bool {
        unsafe { SHUTTING_DOWN.load(Ordering::Acquire) }
    }

    pub fn set_shutdown_status(is_shutting_down: bool) {
        unsafe { SHUTTING_DOWN.store(is_shutting_down, Ordering::Release) };
    }
}

pub use self::platform::*;
