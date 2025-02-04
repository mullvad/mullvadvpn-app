//! This module implements the actual logic performed by different UI components.

use crate::delegate::{AppDelegate, AppDelegateQueue};
use crate::resource;
use crate::ui_downloader::{UiAppDownloader, UiProgressUpdater};

use mullvad_update::api::Version;
use mullvad_update::{
    api::{self, VersionInfoProvider},
    app::{self, AppDownloaderFactory, AppDownloaderParameters},
};

use std::future::Future;
use tokio::sync::{mpsc, oneshot};

/// Actions handled by an async worker task in [handle_action_messages].
enum TaskMessage {
    SetVersionInfo(api::VersionInfo),
    BeginDownload,
    Cancel,
}

/// See the [module-level docs](self).
pub struct AppController {}

/// Public entry function for registering a [AppDelegate].
pub fn initialize_controller<T: AppDelegate + 'static>(delegate: &mut T) {
    use mullvad_update::api::LatestVersionInfoProvider;
    use mullvad_update::app::HttpAppDownloader;

    // App downloader (factory) to use
    type DownloaderFactory<T> = HttpAppDownloader<UiProgressUpdater<T>, UiProgressUpdater<T>>;
    // Version info provider to use
    type VersionInfoProvider = LatestVersionInfoProvider;

    AppController::initialize::<_, DownloaderFactory<T>, VersionInfoProvider>(delegate)
}

impl AppController {
    /// Initialize [AppController] using the provided delegate.
    ///
    /// Providing the downloader and version info fetcher as type arguments, they're decoupled from
    /// the logic of [AppController], allowing them to be mocked.
    pub fn initialize<Delegate, DownloaderFactory, VersionProvider>(delegate: &mut Delegate)
    where
        Delegate: AppDelegate + 'static,
        VersionProvider: VersionInfoProvider + 'static,
        DownloaderFactory: AppDownloaderFactory<
                SigProgress = UiProgressUpdater<Delegate>,
                AppProgress = UiProgressUpdater<Delegate>,
            > + 'static,
    {
        delegate.hide_download_progress();
        delegate.hide_download_button();
        delegate.hide_cancel_button();

        let (task_tx, task_rx) = mpsc::channel(1);
        tokio::spawn(handle_action_messages::<Delegate, DownloaderFactory>(
            delegate.queue(),
            task_rx,
        ));
        delegate.set_status_text(resource::FETCH_VERSION_DESC);
        tokio::spawn(fetch_app_version_info::<Delegate, VersionProvider>(
            delegate,
            task_tx.clone(),
        ));
        Self::register_user_action_callbacks(delegate, task_tx);
    }

    fn register_user_action_callbacks<T: AppDelegate + 'static>(
        delegate: &mut T,
        task_tx: mpsc::Sender<TaskMessage>,
    ) {
        let tx = task_tx.clone();
        delegate.on_download(move || {
            let _ = tx.try_send(TaskMessage::BeginDownload);
        });
        let tx = task_tx.clone();
        delegate.on_cancel(move || {
            let _ = tx.try_send(TaskMessage::Cancel);
        });
    }
}

/// Background task that fetches app version data.
fn fetch_app_version_info<Delegate, VersionProvider>(
    delegate: &mut Delegate,
    download_tx: mpsc::Sender<TaskMessage>,
) -> impl Future<Output = ()>
where
    Delegate: AppDelegate,
    VersionProvider: VersionInfoProvider,
{
    let queue = delegate.queue();

    async move {
        // TODO: handle errors, retry
        let Ok(version_info) = VersionProvider::get_version_info().await else {
            queue.queue_main(move |self_| {
                self_.set_status_text("Failed to fetch version info");
            });
            return;
        };
        let _ = download_tx.try_send(TaskMessage::SetVersionInfo(version_info));
    }
}

/// Async worker that handles actions such as initiating a download, cancelling it, and updating
/// labels.
async fn handle_action_messages<Delegate, DownloaderFactory>(
    queue: Delegate::Queue,
    mut rx: mpsc::Receiver<TaskMessage>,
) where
    Delegate: AppDelegate + 'static,
    DownloaderFactory: AppDownloaderFactory<
        SigProgress = UiProgressUpdater<Delegate>,
        AppProgress = UiProgressUpdater<Delegate>,
    >,
{
    let mut version_info = None;
    let mut active_download = None;

    while let Some(msg) = rx.recv().await {
        match msg {
            TaskMessage::SetVersionInfo(new_version_info) => {
                let version_label = format_latest_version(&new_version_info.stable);
                queue.queue_main(move |self_| {
                    self_.set_status_text(&version_label);
                    self_.show_download_button();
                });
                version_info = Some(new_version_info);
            }
            TaskMessage::BeginDownload => {
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
                    self_.hide_download_button();
                    self_.show_cancel_button();
                    self_.show_download_progress();

                    let downloader = DownloaderFactory::new_downloader(AppDownloaderParameters {
                        signature_url: signature_url.to_owned(),
                        app_url: app_url.to_owned(),
                        app_size,
                        sig_progress: UiProgressUpdater::new(self_.queue()),
                        app_progress: UiProgressUpdater::new(self_.queue()),
                    });

                    let ui_downloader = UiAppDownloader::new(self_, downloader);
                    let _ = tx.send(tokio::spawn(async move {
                        let _ = app::install_and_upgrade(ui_downloader).await;
                    }));
                });
                active_download = rx.await.ok();
            }
            TaskMessage::Cancel => {
                let Some(active_download) = active_download.take() else {
                    continue;
                };
                active_download.abort();
                let _ = active_download.await;

                let version_label = if let Some(version_info) = &version_info {
                    format_latest_version(&version_info.stable)
                } else {
                    "".to_owned()
                };

                queue.queue_main(move |self_| {
                    self_.set_status_text(&version_label);
                    self_.show_download_button();
                    self_.hide_cancel_button();
                    self_.hide_download_progress();
                    self_.set_download_progress(0);
                });
            }
        }
    }
}

fn format_latest_version(version: &Version) -> String {
    format!("{}: {}", resource::LATEST_VERSION_PREFIX, version.version)
}
