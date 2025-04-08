#![cfg(update)]

use mullvad_types::version::{AppUpgradeDownloadProgress, AppUpgradeError, AppUpgradeEvent};
use mullvad_update::app::{
    AppDownloader, AppDownloaderParameters, DownloadError, HttpAppDownloader,
};
use rand::seq::SliceRandom;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::sync::broadcast;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get download directory")]
    GetDownloadDir(#[from] mullvad_paths::Error),

    #[error("Failed to create download directory")]
    CreateDownloadDir(#[source] std::io::Error),

    #[error("Failed to download app")]
    Download(#[from] DownloadError),

    #[error("Download was cancelled or panicked")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Could not select URL for app update")]
    NoUrlFound,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct DownloaderHandle {
    /// Handle to the downloader task
    task: tokio::task::JoinHandle<std::result::Result<PathBuf, Error>>,
    /// Handle to send `AppUpgradeEvent::Aborted` when the downloader is dropped
    dropped_tx: Option<broadcast::Sender<AppUpgradeEvent>>,
}

impl Drop for DownloaderHandle {
    fn drop(&mut self) {
        self.task.abort();
        if let Some(dropped_tx) = self.dropped_tx.take() {
            // If the downloader is dropped, send an event to notify that it was aborted
            let _ = dropped_tx.send(AppUpgradeEvent::Aborted);
        }
    }
}

impl DownloaderHandle {
    /// Wait for the downloader to finish
    pub async fn wait(&mut self) -> Result<PathBuf> {
        let path = (&mut self.task).await?;
        self.dropped_tx = None; // Prevent sending the aborted event after successful download
        path
    }
}

pub fn spawn_downloader(
    version: mullvad_update::version::Version,
    event_tx: broadcast::Sender<AppUpgradeEvent>,
) -> DownloaderHandle {
    DownloaderHandle {
        task: tokio::spawn(start(version, event_tx.clone())),
        dropped_tx: Some(event_tx),
    }
}

/// Begin or resume download of `version`
async fn start(
    version: mullvad_update::version::Version,
    event_tx: broadcast::Sender<AppUpgradeEvent>,
) -> Result<PathBuf> {
    let url = select_cdn_url(&version.urls)
        .ok_or(Error::NoUrlFound)?
        .to_owned();

    log::info!("Downloading app version '{}' from {url}", version.version);

    let download_dir = mullvad_paths::cache_dir()?.join("mullvad-update");
    log::debug!("Download directory: {download_dir:?}");
    fs::create_dir_all(&download_dir)
        .await
        .map_err(Error::CreateDownloadDir)?;

    let params = AppDownloaderParameters {
        app_version: version.version,
        app_url: url.clone(),
        app_size: version.size,
        app_progress: ProgressUpdater::new(server_from_url(&url), event_tx.clone()),
        app_sha256: version.sha256,
        cache_dir: download_dir,
    };
    let mut downloader = HttpAppDownloader::from(params);

    if let Err(download_err) = downloader.download_executable().await {
        log::error!("Failed to download app: {download_err}");
        let _ = event_tx.send(AppUpgradeEvent::Error(AppUpgradeError::DownloadFailed));
        return Err(download_err.into());
    };

    let _ = event_tx.send(AppUpgradeEvent::VerifyingInstaller);

    if let Err(verify_err) = downloader.verify().await {
        log::error!("Failed to verify downloaded app: {verify_err}");
        let _ = event_tx.send(AppUpgradeEvent::Error(AppUpgradeError::VerificationFailed));
        return Err(verify_err.into());
    };

    let _ = event_tx.send(AppUpgradeEvent::VerifiedInstaller);
    Ok(downloader.bin_path())
}

struct ProgressUpdater {
    server: String,
    event_tx: broadcast::Sender<AppUpgradeEvent>,
    complete_frac: f32,
}

impl ProgressUpdater {
    fn new(server: String, event_tx: broadcast::Sender<AppUpgradeEvent>) -> Self {
        Self {
            server,
            event_tx,
            complete_frac: 0.,
        }
    }
}

impl mullvad_update::fetch::ProgressUpdater for ProgressUpdater {
    fn set_url(&mut self, _url: &str) {
        // ignored since we already know the URL
    }

    fn set_progress(&mut self, fraction_complete: f32) {
        if (self.complete_frac - fraction_complete).abs() < 0.01 {
            return;
        }

        self.complete_frac = fraction_complete;

        let _ = self.event_tx.send(AppUpgradeEvent::DownloadProgress(
            AppUpgradeDownloadProgress {
                server: self.server.clone(),
                progress: (fraction_complete * 100.0) as u32,
                // TODO: estimate time left based on how much was downloaded (maybe in last n seconds)
                time_left: Duration::ZERO,
            },
        ));
    }

    fn clear_progress(&mut self) {
        self.complete_frac = 0.;

        let _ = self.event_tx.send(AppUpgradeEvent::DownloadProgress(
            AppUpgradeDownloadProgress {
                server: self.server.clone(),
                progress: 0,
                // TODO: Check if this is reasonable
                time_left: Duration::ZERO,
            },
        ));
    }
}

/// Select a mirror to download from
/// Currently, the selection is random
fn select_cdn_url(urls: &[String]) -> Option<&str> {
    urls.choose(&mut rand::thread_rng()).map(String::as_str)
}

/// Extract domain name from a URL
fn server_from_url(url: &str) -> String {
    let url = url.strip_prefix("https://").unwrap_or(url);
    let (server, _) = url.split_once('/').unwrap_or((url, ""));
    server.to_owned()
}
