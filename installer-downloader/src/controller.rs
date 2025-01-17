//! This module hooks up the UI to actions

use std::cell::RefCell;

use libui::controls::*;
use libui::{EventQueueWithData, UI};

use crate::app::{self, AppDownloader, LatestAppDownloader};
use crate::fetch;
use crate::ui::AppUi;

/// See [module-level](crate) documentation.
#[derive(Clone)]
pub struct AppController {
    ui: UI,
    app_ui: AppUi,
}

impl AppController {
    pub fn new(ui: &UI, app_ui: &AppUi) -> Self {
        let mut self_ = Self {
            ui: ui.clone(),
            app_ui: app_ui.clone(),
        };

        self_.register_click_handlers();

        self_
    }

    fn register_click_handlers(&mut self) {
        let mut on_clicked_self = self.clone();
        self.app_ui.download_button.on_clicked(move |_btn| {
            on_clicked_self.begin_install();
        });
    }

    fn begin_install(&mut self) {
        self.app_ui.download_text.set_text("");

        self.disable_buttons();

        let new_delegated_downloader =
            |sig_progress, app_progress| LatestAppDownloader::stable(sig_progress, app_progress);

        let downloader = UiAppDownloader::new(
            self,
            new_delegated_downloader,
            &self.app_ui.progress_bar,
            &self.app_ui.download_text,
        );

        std::thread::spawn(move || app::install_and_upgrade(downloader));
    }

    fn enable_buttons(&mut self) {
        self.app_ui.download_button.enable();
    }

    fn disable_buttons(&mut self) {
        self.app_ui.download_button.disable();
    }
}

/// App downloader that delegates everything to a downloader and uses the results to update the UI.
struct UiAppDownloader {
    downloader: Box<dyn AppDownloader + Send>,
    queue: EventQueueWithData<RefCell<UiDownloaderContext>>,
}

#[derive(Clone)]
struct UiDownloaderContext {
    controller: AppController,
    status_text: Label,
}

impl UiAppDownloader {
    /// Construct a [UiAppDownloader]. `new_downloader` must construct a downloader that all actions
    /// are delegated to.
    pub fn new<Downloader: AppDownloader + Send + 'static>(
        controller: &AppController,
        new_downloader: impl FnOnce(UiProgressUpdater, UiProgressUpdater) -> Downloader,
        progress_bar: &ProgressBar,
        status_text: &Label,
    ) -> Self {
        let new_progress_notifier =
            || UiProgressUpdater::new(&controller.ui, progress_bar, status_text);

        let downloader = new_downloader(new_progress_notifier(), new_progress_notifier());
        let queue = EventQueueWithData::new(
            &controller.ui,
            RefCell::new(UiDownloaderContext {
                controller: controller.clone(),
                status_text: status_text.clone(),
            }),
        );

        Self {
            downloader: Box::new(downloader) as _,
            queue,
        }
    }
}

impl AppDownloader for UiAppDownloader {
    fn download_signature(&mut self) -> Result<(), crate::app::DownloadError> {
        if let Err(error) = self.downloader.download_signature() {
            self.queue.queue_main(move |self_| {
                let mut ctx = self_.borrow_mut();
                ctx.status_text
                    .set_text("ERROR: Failed to retrieve signature.");
                ctx.controller.enable_buttons();
            });
            Err(error)
        } else {
            Ok(())
        }
    }

    fn download_executable(&mut self) -> Result<(), crate::app::DownloadError> {
        match self.downloader.download_executable() {
            Ok(()) => {
                self.queue.queue_main(move |self_| {
                    let mut ctx = self_.borrow_mut();
                    ctx.status_text
                        .set_text("Download complete! Verifying signature...");
                });

                Ok(())
            }
            Err(err) => {
                self.queue.queue_main(move |self_| {
                    let mut ctx = self_.borrow_mut();
                    ctx.status_text
                        .set_text("ERROR: Download failed. Please try again.");
                    ctx.controller.enable_buttons();
                });

                Err(err)
            }
        }
    }

    fn verify(&mut self) -> Result<(), crate::app::DownloadError> {
        match self.downloader.verify() {
            Ok(()) => {
                self.queue.queue_main(move |self_| {
                    let mut ctx = self_.borrow_mut();
                    ctx.status_text.set_text("Verification complete!");
                });

                Ok(())
            }
            Err(error) => {
                self.queue.queue_main(move |self_| {
                    let mut ctx = self_.borrow_mut();
                    ctx.status_text.set_text("ERROR: Verification failed!");
                });

                Err(error)
            }
        }
    }
}

/// Progress updater that updates a progress bar UI element and status label
struct UiProgressUpdater {
    queue: EventQueueWithData<RefCell<(ProgressBar, Label)>>,
    domain: String,
    prev_progress: Option<u32>,
}

impl UiProgressUpdater {
    pub fn new(ui: &UI, progress_bar: &ProgressBar, status_text: &Label) -> Self {
        Self {
            queue: EventQueueWithData::new(
                &ui,
                RefCell::new((progress_bar.clone(), status_text.clone())),
            ),
            domain: "unknown source".to_owned(),
            prev_progress: None,
        }
    }
}

impl fetch::ProgressUpdater for UiProgressUpdater {
    fn set_progress(&mut self, fraction_complete: f32) {
        let value = (100.0 * fraction_complete).min(100.0) as u32;

        if self.prev_progress == Some(value) {
            // Unconditionally updating causes flickering
            return;
        }

        let status = format!("Downloading from {}... ({value}%)", self.domain);

        self.queue.queue_main(move |ctx| {
            let (progress_bar, status_text) = &mut *ctx.borrow_mut();

            progress_bar.set_value(ProgressBarValue::Determinate(value));
            status_text.set_text(&status);
        });

        self.prev_progress = Some(value);
    }

    fn set_url(&mut self, url: &str) {
        // Parse out domain name
        let url = url.strip_prefix("https://").unwrap_or(url);
        let (domain, _) = url.split_once('/').unwrap_or((url, ""));
        self.domain = domain.to_owned();
    }
}
