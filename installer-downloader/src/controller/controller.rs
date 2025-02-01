//! This module implements the actual logic performed by different UI components.

use super::{AppDelegate, AppDelegateQueue};

use crate::api::{self, VersionInfoProvider};
use crate::app::{self, HttpAppDownloader};

use std::future::Future;
use tokio::sync::{mpsc, oneshot};

use super::ui_downloader::UiAppDownloader;

/// Actions handled by an async worker task in [handle_action_messages].
enum TaskMessage {
    SetVersionInfo(api::VersionInfo),
    BeginDownload,
    Cancel,
}

/// See the [module-level docs](self).
pub struct AppController {}

impl AppController {
    /// Initialize the app controller.
    pub fn initialize<T: AppDelegate + 'static>(delegate: &mut T) {
        delegate.hide_download_button();
        delegate.hide_cancel_button();

        let (task_tx, task_rx) = mpsc::channel(1);
        tokio::spawn(handle_action_messages::<T>(delegate.queue(), task_rx));
        tokio::spawn(fetch_app_version_info(delegate, task_tx.clone()));
        Self::register_user_action_callbacks(delegate, task_tx);
    }

    fn register_user_action_callbacks<T: AppDelegate + 'static>(
        delegate: &mut T,
        task_tx: mpsc::Sender<TaskMessage>,
    ) {
        let tx = task_tx.clone();
        delegate.on_download(move |_delegate| {
            let _ = tx.try_send(TaskMessage::BeginDownload);
        });
        let tx = task_tx.clone();
        delegate.on_cancel(move |_delegate| {
            let _ = tx.try_send(TaskMessage::Cancel);
        });
    }
}

/// Background task that fetches app version data.
fn fetch_app_version_info<T: AppDelegate>(
    delegate: &mut T,
    download_tx: mpsc::Sender<TaskMessage>,
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
        let _ = download_tx.try_send(TaskMessage::SetVersionInfo(version_info));
    }
}

/// Async worker that handles actions such as initiating a download, cancelling it, and updating
/// labels.
async fn handle_action_messages<Delegate: AppDelegate + 'static>(
    queue: Delegate::Queue,
    mut rx: mpsc::Receiver<TaskMessage>,
) {
    let mut version_info = None;
    let mut active_download = None;

    while let Some(msg) = rx.recv().await {
        match msg {
            TaskMessage::SetVersionInfo(new_version_info) => {
                let version_label = format!("Latest version: {}", new_version_info.stable.version);
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
            TaskMessage::Cancel => {
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
