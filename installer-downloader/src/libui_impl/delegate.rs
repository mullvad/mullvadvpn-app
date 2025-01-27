use super::ui::AppUi;
use crate::controller::{AppDelegate, AppDelegateQueue};

use libui::controls::*;
use libui::{EventQueueWithData, UI};
use std::cell::RefCell;

#[derive(Clone)]
pub struct LibuiAppDelegate {
    ui: UI,
    app_ui: AppUi,
}

impl LibuiAppDelegate {
    pub fn new(ui: &UI, app_ui: &AppUi) -> Self {
        Self {
            ui: ui.clone(),
            app_ui: app_ui.clone(),
        }
    }
}

pub struct LibuiAppDelegateQueue {
    queue: EventQueueWithData<RefCell<LibuiAppDelegate>>,
}

impl AppDelegateQueue<LibuiAppDelegate> for LibuiAppDelegateQueue {
    fn queue_main<F: FnOnce(&mut LibuiAppDelegate) + 'static + Send>(&self, callback: F) {
        self.queue
            .queue_main(move |self_| callback(&mut *self_.borrow_mut()));
    }
}

impl AppDelegate for LibuiAppDelegate {
    type Queue = LibuiAppDelegateQueue;

    fn on_download<F>(&mut self, mut callback: F)
    where
        F: Fn(&mut Self) + Send + 'static,
    {
        // FIXME
        //self.app_ui.download_button.on_clicked(move |_| callback());
    }

    fn on_cancel<F>(&mut self, callback: F)
    where
        F: Fn(&mut Self) + Send + 'static,
    {
        // TODO
    }

    fn set_status_text(&mut self, text: &str) {
        self.app_ui.download_text.set_text(text);
    }

    fn show_download_progress(&mut self) {
        // TODO
    }

    fn hide_download_progress(&mut self) {
        // TODO
    }

    fn set_download_progress(&mut self, value: u32) {
        self.app_ui
            .progress_bar
            .set_value(ProgressBarValue::Determinate(value));
    }

    fn enable_download_button(&mut self) {
        self.app_ui.download_button.enable();
    }

    fn disable_download_button(&mut self) {
        self.app_ui.download_button.disable();
    }

    fn show_download_button(&mut self) {
        // TODO
    }

    fn hide_download_button(&mut self) {
        // TODO
    }

    fn show_cancel_button(&mut self) {
        // TODO
    }

    fn hide_cancel_button(&mut self) {
        // TODO
    }

    fn queue(&self) -> Self::Queue {
        LibuiAppDelegateQueue {
            queue: EventQueueWithData::new(&self.ui, RefCell::new(self.clone())),
        }
    }
}
