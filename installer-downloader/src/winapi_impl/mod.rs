use installer_downloader::environment::{Environment, Error as EnvError};
use native_windows_gui as nwg;

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
        Err(error) => fatal_environment_error(error),
    };

    queue.queue_main(|window| {
        crate::controller::initialize_controller(window, environment);
    });

    nwg::dispatch_thread_events();
}

fn fatal_environment_error(error: EnvError) -> ! {
    let title = "Failed to initialize";
    let content = match error {
        EnvError::Arch => "Failed to detect CPU architecture",
    };
    nwg::fatal_message(title, content)
}
