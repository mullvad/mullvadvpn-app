use installer_downloader::environment::{Environment, Error as EnvError};
use native_windows_gui::{self as nwg, modal_fatal_message, ControlHandle};

use crate::delegate::{AppDelegate, AppDelegateQueue};

mod delegate;
mod ui;

pub fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let window = ui::AppWindow::default();
    let window = window.layout().unwrap();

    let queue = window.borrow().queue();

    // Load "global" values and resources
    let environment = match Environment::load() {
        Ok(env) => env,
        Err(error) => show_fatal_error_(error),
    };

    queue.queue_main(|window| {
        crate::controller::initialize_controller(window, environment);
    });

    nwg::dispatch_thread_events();
}

fn show_fatal_error_(error: EnvError) -> ! {
    let parent = ControlHandle::NoHandle;
    let title = "Failed to initialize";
    let content = match error {
        EnvError::Arch => "Failed to detect CPU architecture",
    };
    modal_fatal_message(parent, title, content)
}
