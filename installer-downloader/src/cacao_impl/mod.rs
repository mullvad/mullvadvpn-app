use std::sync::Mutex;

use cacao::appkit::App;
use installer_downloader::environment::{Environment, Error as EnvError};
use ui::{Action, AppImpl};

mod delegate;
mod ui;

pub fn main() {
    let app = App::new("net.mullvad.downloader", AppImpl::default());

    // Load "global" values and resources
    let environment = match Environment::load() {
        Ok(env) => env,
        Err(EnvError::Arch) => {
            unreachable!("The CPU architecture will always be retrievable on macOS")
        }
    };

    let cb: Mutex<Option<ui::MainThreadCallback>> = Mutex::new(Some(Box::new(|self_| {
        crate::controller::initialize_controller(self_, environment);
    })));
    cacao::appkit::App::<ui::AppImpl, _>::dispatch_main(Action::QueueMain(cb));

    app.run();
}
