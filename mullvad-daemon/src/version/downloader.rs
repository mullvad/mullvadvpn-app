#![cfg(update)]

use mullvad_types::version::{AppUpgradeDownloadProgress, AppUpgradeError, AppUpgradeEvent};
use mullvad_update::app::{bin_path, AppDownloader, AppDownloaderParameters, DownloadError};
use rand::seq::SliceRandom;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use talpid_types::ErrorExt;
use tokio::fs;
use tokio::sync::broadcast;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get download directory")]
    GetDownloadDir(#[from] mullvad_paths::Error),

    #[error("Failed to create download directory")]
    CreateDownloadDir(#[source] std::io::Error),

    #[error("Failed to clean up download directory")]
    RemoveDownloadDir(#[source] std::io::Error),

    #[error("Failed to download app")]
    Download(#[from] DownloadError),

    #[error("Download was cancelled or panicked")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Could not select URL for app update")]
    NoUrlFound,
}

pub type Result<T> = std::result::Result<T, Error>;

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

impl std::future::Future for DownloaderHandle {
    type Output = Result<PathBuf>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let task = std::pin::Pin::new(&mut self.task);
        let ready = futures::ready!(task.poll(cx))?;
        self.dropped_tx = None; // Prevent sending the aborted event after successful download
        std::task::Poll::Ready(ready)
    }
}

pub fn spawn_downloader<D>(
    version: mullvad_update::version::Version,
    event_tx: broadcast::Sender<AppUpgradeEvent>,
) -> DownloaderHandle
where
    D: AppDownloader + Send + 'static,
    D: From<AppDownloaderParameters<ProgressUpdater>>,
{
    DownloaderHandle {
        task: tokio::spawn(start::<D>(version, event_tx.clone())),
        dropped_tx: Some(event_tx),
    }
}

/// Begin or resume download of `version`
async fn start<D>(
    version: mullvad_update::version::Version,
    event_tx: broadcast::Sender<AppUpgradeEvent>,
) -> Result<PathBuf>
where
    D: AppDownloader + Send + 'static,
    D: From<AppDownloaderParameters<ProgressUpdater>>,
{
    let url = select_cdn_url(&version.urls)
        .ok_or(Error::NoUrlFound)?
        .to_owned();

    log::info!("Downloading app version '{}' from {url}", version.version);

    let download_dir = if cfg!(test) {
        PathBuf::new()
    } else {
        create_download_dir().await.inspect_err(|err| {
            log::error!("Failed to get download directory: {}", err.display_chain());
            let _ = event_tx.send(AppUpgradeEvent::Error(AppUpgradeError::GeneralError));
        })?
    };
    let bin_path = bin_path(&version.version, &download_dir);

    let params = AppDownloaderParameters {
        app_version: version.version,
        app_url: url.clone(),
        app_size: version.size,
        app_progress: ProgressUpdater::new(server_from_url(&url), event_tx.clone()),
        app_sha256: version.sha256,
        cache_dir: download_dir,
    };
    let mut downloader = D::from(params);

    let _ = event_tx.send(AppUpgradeEvent::DownloadStarting);
    if let Err(err) = downloader.download_executable().await {
        let _ = event_tx.send(AppUpgradeEvent::Error(AppUpgradeError::DownloadFailed));
        return Err(err.into());
    };
    let _ = event_tx.send(AppUpgradeEvent::VerifyingInstaller);
    if let Err(err) = downloader.verify().await {
        let _ = event_tx.send(AppUpgradeEvent::Error(AppUpgradeError::VerificationFailed));
        return Err(err.into());
    };
    let _ = event_tx.send(AppUpgradeEvent::VerifiedInstaller);

    // Note that we cannot call `downloader.install()` here, as it must be done by the user process.
    // Instead, the GUI is responsible for launching the installer.

    Ok(bin_path)
}

async fn create_download_dir() -> Result<PathBuf> {
    let download_dir = mullvad_paths::cache_dir()?.join("mullvad-update");
    log::trace!("Download directory: {download_dir:?}");
    fs::create_dir_all(&download_dir)
        .await
        .map_err(Error::CreateDownloadDir)?;
    Ok(download_dir)
}

/// Remove the download directory
pub async fn clear_download_dir() -> Result<PathBuf> {
    let download_dir = mullvad_paths::get_cache_dir()?.join("mullvad-update");
    log::info!("Cleaning up download directory: {}", download_dir.display());
    match fs::remove_dir_all(&download_dir).await {
        Ok(()) => Ok(download_dir),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(download_dir),
        Err(err) => Err(Error::CreateDownloadDir(err)),
    }
}

pub struct ProgressUpdater {
    server: String,
    event_tx: broadcast::Sender<AppUpgradeEvent>,
    complete_frac: f32,
    start_time: Instant,
    complete_frac_at_start: Option<f32>,
}

impl ProgressUpdater {
    fn new(server: String, event_tx: broadcast::Sender<AppUpgradeEvent>) -> Self {
        Self {
            server,
            event_tx,
            complete_frac: 0.,
            start_time: Instant::now(),
            complete_frac_at_start: None,
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
        let complete_frac_at_start = self.complete_frac_at_start.get_or_insert(fraction_complete);

        self.complete_frac = fraction_complete;

        let _ = self.event_tx.send(AppUpgradeEvent::DownloadProgress(
            AppUpgradeDownloadProgress {
                server: self.server.clone(),
                progress: (fraction_complete * 100.0) as u32,
                time_left: estimate_time_left(
                    self.start_time,
                    fraction_complete,
                    *complete_frac_at_start,
                ),
            },
        ));
    }

    fn clear_progress(&mut self) {
        self.complete_frac = 0.;

        let _ = self.event_tx.send(AppUpgradeEvent::DownloadProgress(
            AppUpgradeDownloadProgress {
                server: self.server.clone(),
                progress: 0,
                time_left: None,
            },
        ));
    }
}

fn estimate_time_left(
    start_time: Instant,
    fraction_complete: f32,
    complete_frac_at_start: f32,
) -> Option<Duration> {
    let completed_frac_since_start = fraction_complete - complete_frac_at_start;
    // Don't estimate time left if the progress is less than 1%, to avoid division numerical instability
    if completed_frac_since_start <= 0.01 {
        return None;
    }
    let remaining_frac = 1.0 - fraction_complete;

    let elapsed = start_time.elapsed();
    Some(elapsed.mul_f32(remaining_frac / completed_frac_since_start))
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
