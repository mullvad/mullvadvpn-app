use std::sync::{Arc, Mutex};

use cacao::{
    control::Control,
    layout::{Layout, LayoutConstraint},
};

use super::ui::{Action, AppWindow, ErrorView};
use crate::delegate::{AppDelegate, AppDelegateQueue};

impl AppDelegate for AppWindow {
    type Queue = Queue;

    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.download_button.set_callback(callback);
    }

    fn on_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.cancel_button.set_callback(callback);
    }

    fn on_beta_link<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.beta_link.set_callback(callback);
    }

    fn set_status_text(&mut self, text: &str) {
        self.status_text.set_text(text);
    }

    fn set_download_text(&mut self, text: &str) {
        self.download_text.set_text(text);

        // If there is a download_text, move status_text up to make room

        let offset = if text.is_empty() { 59.0 } else { 39.0 };

        if let Some(previous_constraint) = self.status_text_position_y.take() {
            LayoutConstraint::deactivate(&[previous_constraint]);
        }

        let new_constraint = self
            .status_text
            .top
            .constraint_equal_to(&self.main_view.top)
            .offset(offset);
        self.status_text_position_y = Some(new_constraint.clone());
        LayoutConstraint::activate(&[new_constraint]);
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
        self.download_button.set_hidden(false);
    }

    fn hide_download_button(&mut self) {
        self.download_button.set_hidden(true);
    }

    fn enable_download_button(&mut self) {
        self.download_button.set_enabled(true);
    }

    fn disable_download_button(&mut self) {
        self.download_button.set_enabled(false);
    }

    fn show_cancel_button(&mut self) {
        self.cancel_button.set_hidden(false);
    }

    fn hide_cancel_button(&mut self) {
        self.cancel_button.set_hidden(true);
    }

    fn enable_cancel_button(&mut self) {
        self.cancel_button.set_enabled(true);
    }

    fn disable_cancel_button(&mut self) {
        self.cancel_button.set_enabled(false);
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

    fn quit(&mut self) {
        cacao::appkit::App::<super::ui::AppImpl, _>::dispatch_main(Action::Quit);
    }

    fn on_stable_link<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.stable_link.set_callback(callback);
    }

    fn show_stable_text(&mut self) {
        self.stable_link.set_hidden(false);
    }

    fn hide_stable_text(&mut self) {
        self.stable_link.set_hidden(true);
    }

    fn show_error_message(&mut self, message: installer_downloader::delegate::ErrorMessage) {
        let on_cancel = self.error_cancel_callback.clone().map(|callback| {
            move || {
                let callback = callback.clone();
                let callback = Action::ButtonClick { callback };
                cacao::appkit::App::<super::ui::AppImpl, _>::dispatch_main(callback);
            }
        });

        let on_retry = self.error_retry_callback.clone().map(|callback| {
            move || {
                let callback = callback.clone();
                let callback = Action::ButtonClick { callback };
                cacao::appkit::App::<super::ui::AppImpl, _>::dispatch_main(callback);
            }
        });

        self.error_view = Some(ErrorView::new(
            &self.main_view,
            message,
            on_retry,
            on_cancel,
        ));
    }

    fn hide_error_message(&mut self) {
        self.error_view.take();
    }

    fn on_error_message_retry<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.error_retry_callback = Some(Self::sync_callback(callback));
    }

    fn on_error_message_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.error_cancel_callback = Some(Self::sync_callback(callback));
    }
}

impl AppWindow {
    // NOTE: We need this horrible lock because Dispatcher demands Sync, but AppDelegate does not require Sync
    fn sync_callback(
        callback: impl Fn() + Send + 'static,
    ) -> Arc<Mutex<Box<dyn Fn() + Send + 'static>>> {
        Arc::new(Mutex::new(Box::new(callback)))
    }
}

/// This simply mutates the UI on the main thread using the GCD
pub struct Queue {}

impl AppDelegateQueue<AppWindow> for Queue {
    fn queue_main<F: FnOnce(&mut AppWindow) + 'static + Send>(&self, callback: F) {
        // NOTE: We need this horrible lock because Dispatcher demands Sync
        let cb: Mutex<Option<super::ui::MainThreadCallback>> = Mutex::new(Some(Box::new(callback)));
        cacao::appkit::App::<super::ui::AppImpl, _>::dispatch_main(Action::QueueMain(cb));
    }
}
