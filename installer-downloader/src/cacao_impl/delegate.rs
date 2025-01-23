use std::sync::{Arc, Mutex};

use cacao::{control::Control, layout::Layout};

use super::ui::{Action, AppWindow};
use crate::controller::{AppDelegate, AppDelegateQueue};

impl AppDelegate for AppWindow {
    type Queue = Queue;

    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn(&mut Self) + Send + 'static,
    {
        // NOTE: We need this horrible lock because Dispatcher demands Sync
        let cb: Arc<Mutex<Box<dyn Fn(&mut Self) + Send + 'static>>> =
            Arc::new(Mutex::new(Box::new(move |self_| {
                self_.progress.set_hidden(false);

                callback(self_)
            })));
        self.button.button.set_action(move || {
            let cb = Action::DownloadClick(cb.clone());
            cacao::appkit::App::<super::ui::AppImpl, _>::dispatch_main(cb);
        });
    }

    fn set_status_text(&mut self, text: &str) {
        self.text.set_text(text);
    }

    fn set_download_progress(&mut self, complete: u32) {
        self.progress.set_value(complete as f64);
    }

    fn enable_download_button(&mut self) {
        self.button.button.set_enabled(true);
    }

    fn disable_download_button(&mut self) {
        self.button.button.set_enabled(false);
    }

    fn queue(&self) -> Self::Queue {
        Queue {}
    }
}

/// This simply mutates the UI on the main thread using the GCD
pub struct Queue {}

impl AppDelegateQueue<AppWindow> for Queue {
    fn queue_main<F: FnOnce(&mut AppWindow) + 'static + Send>(&self, callback: F) {
        // NOTE: We need this horrible lock because Dispatcher demands Sync
        let cb: Mutex<Option<Box<dyn FnOnce(&mut AppWindow) + Send>>> =
            Mutex::new(Some(Box::new(callback)));
        cacao::appkit::App::<super::ui::AppImpl, _>::dispatch_main(Action::QueueMain(cb));
    }
}
