//! Framework-agnostic module that hooks up a UI to actions

use tokio::sync::{mpsc, oneshot};

use crate::app::{self, AppDownloader, LatestAppDownloader};
use crate::fetch;

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

enum DownloadTaskMessage<T: AppDelegate> {
    BeginDownload(UiAppDownloader<T>),
    Cancel(oneshot::Sender<()>),
}

/// See [module-level](crate) documentation.
pub fn initialize_controller<T: AppDelegate + 'static>(delegate: &mut T) {
    delegate.hide_cancel_button();

    let (download_tx, download_rx) = mpsc::channel(1);
    tokio::spawn(handle_download_messages(download_rx));

    let tx = download_tx.clone();
    delegate.on_download(move |delegate| on_download(delegate, tx.clone()));
    delegate.on_cancel(move |delegate| on_cancel(delegate, download_tx.clone()));
}

async fn handle_download_messages<T: AppDelegate + 'static>(
    mut rx: mpsc::Receiver<DownloadTaskMessage<T>>,
) {
    let mut active_download = None;

    while let Some(msg) = rx.recv().await {
        match msg {
            DownloadTaskMessage::BeginDownload(downloader) => {
                if active_download.is_none() {
                    active_download = Some(tokio::spawn(async move {
                        let _ = app::install_and_upgrade(downloader).await;
                    }));
                }
            }
            DownloadTaskMessage::Cancel(done_tx) => {
                let Some(active_download) = active_download.take() else {
                    continue;
                };
                active_download.abort();
                let _ = active_download.await;
                let _ = done_tx.send(());
            }
        }
    }
}

fn on_download<T: AppDelegate + 'static>(
    delegate: &mut T,
    download_tx: mpsc::Sender<DownloadTaskMessage<T>>,
) {
    delegate.set_status_text("");
    delegate.disable_download_button();
    delegate.show_cancel_button();
    delegate.show_download_progress();

    let new_delegated_downloader =
        |sig_progress, app_progress| LatestAppDownloader::stable(sig_progress, app_progress);

    let downloader = UiAppDownloader::new(delegate, new_delegated_downloader);

    let _ = download_tx.try_send(DownloadTaskMessage::BeginDownload(downloader));
}

fn on_cancel<T: AppDelegate + 'static>(
    delegate: &mut T,
    download_tx: mpsc::Sender<DownloadTaskMessage<T>>,
) {
    let (done_tx, done_rx) = oneshot::channel();
    let _ = download_tx.try_send(DownloadTaskMessage::Cancel(done_tx));
    tokio::runtime::Handle::current().block_on(async move {
        let _ = done_rx.await;
    });

    delegate.set_status_text("");
    delegate.enable_download_button();
    delegate.hide_cancel_button();
    delegate.hide_download_progress();
    delegate.set_download_progress(0);
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
