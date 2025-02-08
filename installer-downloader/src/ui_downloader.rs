//! This module hooks up [AppDelegate]s to arbitrary implementations of [AppDownloader] and
//! [fetch::ProgressUpdater].

use crate::{
    delegate::{AppDelegate, AppDelegateQueue},
    resource,
};
use mullvad_update::{
    app::{self, AppDownloader, AppDownloaderParameters},
    fetch,
};

/// [AppDownloader] that delegates the actual work to some underlying `downloader` and uses it to
/// update a UI.
pub struct UiAppDownloader<Delegate: AppDelegate, Downloader> {
    downloader: Downloader,
    /// Queue used to control the app UI
    queue: Delegate::Queue,
}

/// Parameters for [UiAppDownloader]
pub type UiAppDownloaderParameters<Delegate> = AppDownloaderParameters<UiProgressUpdater<Delegate>>;

impl<Delegate: AppDelegate, Downloader: AppDownloader + Send + 'static>
    UiAppDownloader<Delegate, Downloader>
{
    /// Construct a [UiAppDownloader].
    pub fn new(delegate: &Delegate, downloader: Downloader) -> Self {
        Self {
            downloader,
            queue: delegate.queue(),
        }
    }
}

#[async_trait::async_trait]
impl<Delegate: AppDelegate, Downloader: AppDownloader + Send + 'static> AppDownloader
    for UiAppDownloader<Delegate, Downloader>
{
    async fn download_executable(&mut self) -> Result<(), app::DownloadError> {
        match self.downloader.download_executable().await {
            Ok(()) => {
                self.queue.queue_main(move |self_| {
                    self_.set_download_text(resource::DOWNLOAD_COMPLETE_DESC);
                    self_.disable_cancel_button();
                });

                Ok(())
            }
            Err(err) => {
                self.queue.queue_main(move |self_| {
                    self_.set_download_text("ERROR: Download failed. Please try again.");
                    self_.enable_download_button();
                    self_.hide_cancel_button();
                });

                Err(err)
            }
        }
    }

    async fn verify(&mut self) -> Result<(), app::DownloadError> {
        match self.downloader.verify().await {
            Ok(()) => {
                self.queue.queue_main(move |self_| {
                    self_.set_download_text(resource::VERIFICATION_SUCCEEDED_DESC);
                });

                Ok(())
            }
            Err(error) => {
                self.queue.queue_main(move |self_| {
                    self_.set_download_text("ERROR: Verification failed!");
                });

                Err(error)
            }
        }
    }
}

/// Implementation of [fetch::ProgressUpdater] that updates some [AppDelegate].
pub struct UiProgressUpdater<Delegate: AppDelegate> {
    domain: String,
    prev_progress: Option<u32>,
    queue: Delegate::Queue,
}

impl<Delegate: AppDelegate> UiProgressUpdater<Delegate> {
    pub fn new(queue: Delegate::Queue) -> Self {
        Self {
            domain: "unknown source".to_owned(),
            prev_progress: None,
            queue,
        }
    }
}

impl<Delegate: AppDelegate + 'static> fetch::ProgressUpdater for UiProgressUpdater<Delegate> {
    fn set_progress(&mut self, fraction_complete: f32) {
        let value = (100.0 * fraction_complete).min(100.0) as u32;

        if self.prev_progress == Some(value) {
            // Unconditionally updating causes flickering
            return;
        }

        let status = format!(
            "{} {}... ({value}%)",
            resource::DOWNLOADING_DESC_PREFIX,
            self.domain
        );

        self.queue.queue_main(move |self_| {
            self_.set_download_progress(value);
            self_.set_download_text(&status);
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
