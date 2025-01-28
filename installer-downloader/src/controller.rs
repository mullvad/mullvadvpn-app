//! Framework-agnostic module that hooks up a UI to actions

use std::future::Future;

use tokio::sync::{mpsc, oneshot};

use crate::api::VersionInfoProvider;
use crate::app::{self, AppDownloader, HttpAppDownloader};
use crate::{api, fetch};

/// Trait implementing high-level UI actions
pub trait AppDelegate {
    /// Queue lets us perform actions from other threads
    type Queue: AppDelegateQueue<Self>;

    /// Register click handler for the download button
    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn(&mut Self) + Send + 'static;

    /// Register click handler for the cancel button
    fn on_cancel<F>(&mut self, callback: F)
    where
        F: Fn(&mut Self) + Send + 'static;

    /// Set download status text
    fn set_status_text(&mut self, text: &str);

    /// Show download progress bar
    fn show_download_progress(&mut self);

    /// Hide download progress bar
    fn hide_download_progress(&mut self);

    /// Update download progress bar
    fn set_download_progress(&mut self, complete: u32);

    /// Enable download button
    fn enable_download_button(&mut self);

    /// Disable download button
    fn disable_download_button(&mut self);

    /// Show download button
    fn show_download_button(&mut self);

    /// Hide download button
    fn hide_download_button(&mut self);

    /// Show cancel button
    fn show_cancel_button(&mut self);

    /// Hide cancel button
    fn hide_cancel_button(&mut self);

    /// Create queue for scheduling actions on UI thread
    fn queue(&self) -> Self::Queue;
}

/// Schedules actions on the UI thread from other threads
pub trait AppDelegateQueue<T: ?Sized>: Send {
    fn queue_main<F: FnOnce(&mut T) + 'static + Send>(&self, callback: F);
}

enum DownloadTaskMessage {
    SetVersionInfo(api::VersionInfo),
    BeginDownload,
    Cancel,
}

/// See [module-level](self) documentation.
pub fn initialize_controller<T: AppDelegate + 'static>(delegate: &mut T) {
    delegate.hide_download_button();
    delegate.hide_cancel_button();

    let (download_tx, download_rx) = mpsc::channel(1);
    tokio::spawn(handle_download_messages::<T>(delegate.queue(), download_rx));

    let tx = download_tx.clone();
    delegate.on_download(move |_delegate| {
        let _ = tx.try_send(DownloadTaskMessage::BeginDownload);
    });
    let tx = download_tx.clone();
    delegate.on_cancel(move |_delegate| {
        let _ = tx.try_send(DownloadTaskMessage::Cancel);
    });

    tokio::spawn(fetch_app_version_info(delegate, download_tx));
}

fn fetch_app_version_info<T: AppDelegate>(
    delegate: &mut T,
    download_tx: mpsc::Sender<DownloadTaskMessage>,
) -> impl Future<Output = ()> {
    delegate.set_status_text("Fetching app version...");
    let queue = delegate.queue();

    async move {
        // TODO: handle errors, retry
        let Ok(version_info) = api::LatestVersionInfoProvider::get_version_info().await else {
            queue.queue_main(move |self_| {
                self_.set_status_text("Failed to fetch version info");
            });
            return;
        };
        let _ = download_tx.try_send(DownloadTaskMessage::SetVersionInfo(version_info));
    }
}

async fn handle_download_messages<Delegate: AppDelegate + 'static>(
    queue: Delegate::Queue,
    mut rx: mpsc::Receiver<DownloadTaskMessage>,
) {
    let mut version_info = None;
    let mut active_download = None;

    while let Some(msg) = rx.recv().await {
        match msg {
            DownloadTaskMessage::SetVersionInfo(new_version_info) => {
                let version_label = format!("Latest version: {}", new_version_info.stable.version);
                queue.queue_main(move |self_| {
                    self_.set_status_text(&version_label);
                    self_.show_download_button();
                });
                version_info = Some(new_version_info);
            }
            DownloadTaskMessage::BeginDownload => {
                if active_download.is_some() {
                    continue;
                }
                let Some(version_info) = version_info.clone() else {
                    continue;
                };

                let (tx, rx) = oneshot::channel();
                queue.queue_main(move |self_| {
                    // TODO: Select appropriate URLs
                    let Some(app_url) = version_info.stable.urls.first() else {
                        return;
                    };
                    let Some(signature_url) = version_info.stable.signature_urls.first() else {
                        return;
                    };
                    let app_size = version_info.stable.size;

                    self_.set_status_text("");
                    self_.disable_download_button();
                    self_.show_cancel_button();
                    self_.show_download_progress();

                    let new_delegated_downloader = |sig_progress, app_progress| {
                        HttpAppDownloader::new(
                            signature_url,
                            app_url,
                            app_size,
                            sig_progress,
                            app_progress,
                        )
                    };

                    let downloader = UiAppDownloader::new(self_, new_delegated_downloader);
                    let _ = tx.send(tokio::spawn(async move {
                        let _ = app::install_and_upgrade(downloader).await;
                    }));
                });
                active_download = rx.await.ok();
            }
            DownloadTaskMessage::Cancel => {
                let Some(active_download) = active_download.take() else {
                    continue;
                };
                active_download.abort();
                let _ = active_download.await;

                let version_label = if let Some(version_info) = &version_info {
                    format!("Latest version: {}", version_info.stable.version)
                } else {
                    "".to_owned()
                };

                queue.queue_main(move |self_| {
                    self_.set_status_text(&version_label);
                    self_.enable_download_button();
                    self_.hide_cancel_button();
                    self_.hide_download_progress();
                    self_.set_download_progress(0);
                });
            }
        }
    }
}

/// App downloader that delegates everything to a downloader and uses the results to update the UI.
struct UiAppDownloader<Delegate: AppDelegate> {
    downloader: Box<dyn AppDownloader + Send>,
    /// Queue used to control the app UI
    queue: Delegate::Queue,
}

impl<Delegate: AppDelegate> UiAppDownloader<Delegate> {
    /// Construct a [UiAppDownloader]. `new_downloader` must construct a downloader that all actions
    /// are delegated to.
    pub fn new<Downloader: AppDownloader + Send + 'static>(
        delegate: &Delegate,
        new_downloader: impl FnOnce(
            UiProgressUpdater<Delegate>,
            UiProgressUpdater<Delegate>,
        ) -> Downloader,
    ) -> Self {
        let new_progress_notifier = || UiProgressUpdater::new(delegate.queue());
        let downloader = new_downloader(new_progress_notifier(), new_progress_notifier());
        Self {
            downloader: Box::new(downloader) as _,
            queue: delegate.queue(),
        }
    }
}

#[async_trait::async_trait]
impl<Delegate: AppDelegate> AppDownloader for UiAppDownloader<Delegate> {
    async fn download_signature(&mut self) -> Result<(), crate::app::DownloadError> {
        if let Err(error) = self.downloader.download_signature().await {
            self.queue.queue_main(move |self_| {
                self_.set_status_text("ERROR: Failed to retrieve signature.");
                self_.enable_download_button();
                self_.hide_cancel_button();
            });
            Err(error)
        } else {
            Ok(())
        }
    }

    async fn download_executable(&mut self) -> Result<(), crate::app::DownloadError> {
        match self.downloader.download_executable().await {
            Ok(()) => {
                self.queue.queue_main(move |self_| {
                    self_.set_status_text("Download complete! Verifying signature...");
                    self_.hide_cancel_button();
                });

                Ok(())
            }
            Err(err) => {
                self.queue.queue_main(move |self_| {
                    self_.set_status_text("ERROR: Download failed. Please try again.");
                    self_.enable_download_button();
                    self_.hide_cancel_button();
                });

                Err(err)
            }
        }
    }

    async fn verify(&mut self) -> Result<(), crate::app::DownloadError> {
        match self.downloader.verify().await {
            Ok(()) => {
                self.queue.queue_main(move |self_| {
                    self_.set_status_text("Verification complete!");
                });

                Ok(())
            }
            Err(error) => {
                self.queue.queue_main(move |self_| {
                    self_.set_status_text("ERROR: Verification failed!");
                });

                Err(error)
            }
        }
    }
}

/// Progress updater that updates a progress bar UI element and status label
struct UiProgressUpdater<Delegate: AppDelegate> {
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

        let status = format!("Downloading from {}... ({value}%)", self.domain);

        self.queue.queue_main(move |self_| {
            self_.set_download_progress(value);
            self_.set_status_text(&status);
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
