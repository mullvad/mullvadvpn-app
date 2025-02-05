use std::sync::{Arc, Mutex};

use cacao::{control::Control, layout::Layout};

use super::ui::{Action, AppWindow};
use crate::delegate::{AppDelegate, AppDelegateQueue};

impl AppDelegate for AppWindow {
    type Queue = Queue;

    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        let cb = Self::sync_callback(callback);
        self.download_button.button.set_action(move || {
            let cb = Action::DownloadClick(cb.clone());
            cacao::appkit::App::<super::ui::AppImpl, _>::dispatch_main(cb);
        });
    }

    fn on_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        let cb = Self::sync_callback(callback);
        self.cancel_button.button.set_action(move || {
            let cb = Action::CancelClick(cb.clone());
            cacao::appkit::App::<super::ui::AppImpl, _>::dispatch_main(cb);
        });
    }

    fn set_status_text(&mut self, text: &str) {
        self.status_text.set_text(text);
    }

    fn set_download_text(&mut self, text: &str) {
        self.download_text.set_text(text);
    }

    fn show_download_progress(&mut self) {
        self.progress.set_hidden(false);
    }

    fn hide_download_progress(&mut self) {
        self.progress.set_hidden(true);
    }

    fn set_download_progress(&mut self, complete: u32) {
        self.progress.set_value(complete as f64);
    }

    fn show_download_button(&mut self) {
        self.download_button.button.set_hidden(false);
    }

    fn hide_download_button(&mut self) {
        self.download_button.button.set_hidden(true);
    }

    fn enable_download_button(&mut self) {
        self.download_button.button.set_enabled(true);
    }

    fn disable_download_button(&mut self) {
        self.download_button.button.set_enabled(false);
    }

    fn show_cancel_button(&mut self) {
        self.cancel_button.button.set_hidden(false);
    }

    fn hide_cancel_button(&mut self) {
        self.cancel_button.button.set_hidden(true);
    }

    fn enable_cancel_button(&mut self) {
        self.cancel_button.button.set_enabled(true);
    }

    fn disable_cancel_button(&mut self) {
        self.cancel_button.button.set_enabled(false);
    }

    fn show_beta_text(&mut self) {
        self.beta_link.set_hidden(false);
        self.beta_link_preface.set_hidden(false);
    }

    fn hide_beta_text(&mut self) {
        self.beta_link.set_hidden(true);
        self.beta_link_preface.set_hidden(true);
    }

    fn queue(&self) -> Self::Queue {
        Queue {}
    }
}

impl AppWindow {
    // NOTE: We need this horrible lock because Dispatcher demands Sync, but AppDelegate does not require Sync
    fn sync_callback(
        callback: impl Fn() + Send + 'static,
    ) -> Arc<Mutex<Box<dyn Fn() + Send + 'static>>> {
        Arc::new(Mutex::new(Box::new(move || callback())))
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
