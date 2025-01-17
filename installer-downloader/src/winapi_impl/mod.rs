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

    queue.queue_main(|window| {
        crate::controller::initialize_controller(window);
    });

    nwg::dispatch_thread_events();
}
