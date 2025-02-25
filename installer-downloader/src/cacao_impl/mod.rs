use std::sync::Mutex;

use cacao::appkit::App;
use ui::{Action, AppImpl};

mod delegate;
mod ui;

pub fn main() {
    let app = App::new("net.mullvad.MullvadDownloader", AppImpl::default());

    let cb: Mutex<Option<ui::MainThreadCallback>> = Mutex::new(Some(Box::new(|self_| {
        crate::controller::initialize_controller(self_);
    })));
    cacao::appkit::App::<ui::AppImpl, _>::dispatch_main(Action::QueueMain(cb));

    app.run();
}
