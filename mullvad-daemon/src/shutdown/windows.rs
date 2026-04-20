#[derive(thiserror::Error, Debug)]
#[error("Unable to attach ctrl-c handler")]
pub struct Error(#[from] ctrlc::Error);

pub fn set_shutdown_signal_handler(f: impl Fn() + 'static + Send) -> Result<(), Error> {
    ctrlc::set_handler(move || {
        log::debug!("Process received Ctrl-c");
        f();
    })
    .map_err(Error)
}
