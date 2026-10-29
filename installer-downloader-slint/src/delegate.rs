//! This module implements [AppDelegate] and [Queue], which allows the Slint UI to be hooked up to our
//! generic controller.

use crate::AppWindow;
use slint::Weak;

use std::sync::{Arc, Mutex};

use installer_downloader::delegate::{AppDelegate, AppDelegateQueue};

pub struct SlintApp {
    ui: Weak<AppWindow>,
    queue: Queue,
}

impl SlintApp {
    pub fn new(ui: Weak<AppWindow>) -> Self {
        let queue = Queue::new();
        Self { ui, queue }
    }
}

/// This simply mutates the UI on the main thread
#[derive(Clone)]
pub struct Queue {
    // TODO: Actually run these callbacks eventually. On tick ?
    callbacks: Arc<Mutex<Vec<Box<dyn FnOnce(&mut SlintApp) + Send>>>>,
}

impl Queue {
    fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl AppDelegate for SlintApp {
    type Queue = Queue;

    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.ui.unwrap().on_download(callback);
    }

    fn on_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.ui.unwrap().on_cancel(callback);
    }

    fn on_beta_link<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.ui.unwrap().on_beta_link(callback);
    }

    fn on_stable_link<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.ui.unwrap().on_stable_link(callback);
    }

    fn set_status_text(&mut self, text: &str) {
        self.ui.unwrap().set_status_text(text.into());
    }

    fn clear_status_text(&mut self) {
        self.ui.unwrap().set_status_text("".into());
    }

    fn set_download_text(&mut self, text: &str) {
        self.ui.unwrap().set_download_text(text.into());
    }

    fn clear_download_text(&mut self) {
        self.ui.unwrap().set_download_text("".into());
    }

    fn show_download_progress(&mut self) {
        self.ui.unwrap().set_show_download_progress(true);
    }

    fn hide_download_progress(&mut self) {
        self.ui.unwrap().set_show_download_progress(false);
    }

    fn set_download_progress(&mut self, complete: u32) {
        self.ui.unwrap().set_download_progress(complete as i32);
    }

    fn clear_download_progress(&mut self) {
        self.ui.unwrap().set_download_progress(0);
    }

    fn enable_download_button(&mut self) {
        self.ui.unwrap().set_enable_download_button(true);
    }

    fn disable_download_button(&mut self) {
        self.ui.unwrap().set_enable_download_button(false);
    }

    fn show_download_button(&mut self) {
        self.ui.unwrap().set_show_download_button(true);
    }

    fn hide_download_button(&mut self) {
        self.ui.unwrap().set_show_download_button(false);
    }

    fn show_cancel_button(&mut self) {
        self.ui.unwrap().set_show_cancel_button(true);
    }

    fn hide_cancel_button(&mut self) {
        self.ui.unwrap().set_show_cancel_button(false);
    }

    fn enable_cancel_button(&mut self) {
        self.ui.unwrap().set_enable_cancel_button(true);
    }

    fn disable_cancel_button(&mut self) {
        self.ui.unwrap().set_enable_cancel_button(false);
    }

    fn show_beta_text(&mut self) {
        self.ui.unwrap().set_show_beta_text(true);
    }

    fn hide_beta_text(&mut self) {
        self.ui.unwrap().set_show_beta_text(false);
    }

    fn show_stable_text(&mut self) {
        self.ui.unwrap().set_show_stable_text(true);
    }

    fn hide_stable_text(&mut self) {
        self.ui.unwrap().set_show_stable_text(false);
    }

    fn show_error_message(&mut self, message: installer_downloader::delegate::ErrorMessage) {
        // TODO: Pass _all_ fields of `ErrorMessage` to the slint UI.
        let ui = self.ui.unwrap();
        ui.set_error_message(message.status_text.into());
        ui.set_show_error_message(true);
    }

    fn hide_error_message(&mut self) {
        self.ui.unwrap().set_show_error_message(false);
    }

    fn on_error_message_retry<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.ui.unwrap().on_error_message_retry(callback);
    }

    fn on_error_message_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.ui.unwrap().on_error_message_cancel(callback);
    }

    fn quit(&mut self) {
        todo!()
    }

    fn queue(&self) -> Self::Queue {
        self.queue.clone()
    }
}

impl AppDelegateQueue<SlintApp> for Queue {
    fn queue_main<F: FnOnce(&mut SlintApp) + 'static + Send>(&self, callback: F) {
        self.callbacks.lock().unwrap().push(Box::new(callback));
    }
}
