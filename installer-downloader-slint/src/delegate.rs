//! This module implements [AppDelegate] and [Queue], which allows the Slint UI to be hooked up to our
//! generic controller.

use crate::AppWindow;
use slint::Weak;
use tokio::sync::mpsc;

use installer_downloader::delegate::{AppDelegate, AppDelegateQueue};
use tracing::{debug, trace};

pub struct SlintApp {
    ui: Weak<AppWindow>,
    work: (Queue, Backlog),
}

impl SlintApp {
    pub fn new(ui: Weak<AppWindow>) -> Self {
        let work = Queue::new();
        Self { ui, work }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        while let Some(callback) = self.work.1.callbacks.recv().await {
            callback(&mut self);
        }
        Ok(())
    }
}

/// This simply mutates the UI on the main thread
#[derive(Clone)]
pub struct Queue {
    queue: mpsc::Sender<UICallback>,
}

pub struct Backlog {
    callbacks: mpsc::Receiver<UICallback>,
}

/// TODO: Document.
type UICallback = Box<dyn FnOnce(&mut SlintApp) + Send>;

impl Queue {
    fn new() -> (Queue, Backlog) {
        // TODO: Arbitrary
        let (queue, callbacks) = mpsc::channel(10);
        (Queue { queue }, Backlog { callbacks })
    }
}

impl AppDelegate for SlintApp {
    type Queue = Queue;

    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        let cb = move || {
            debug!("Calling `on_download!`");
            callback()
        };
        self.ui.unwrap().on_download(cb);
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
        // TODO: Signal main loop that we're exiting instead.
        std::process::exit(0);
    }

    fn queue(&self) -> Self::Queue {
        self.work.0.clone()
    }
}

impl AppDelegateQueue<SlintApp> for Queue {
    fn queue_main<F: FnOnce(&mut SlintApp) + 'static + Send>(&self, callback: F) {
        let rt = tokio::runtime::Handle::current();
        let queue = self.queue.clone();
        let fut = async move {
            if let Err(err) = queue.send(Box::new(callback)).await {
                trace!("{err}");
            }
        };
        rt.spawn(fut);
    }
}
