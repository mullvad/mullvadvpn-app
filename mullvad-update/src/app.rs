//! This module implements the flow of downloading and verifying the app signature.

use std::path::PathBuf;

use crate::{
    fetch::{self, ProgressUpdater},
    verify::{AppVerifier, PgpVerifier},
};

#[derive(Debug)]
pub enum DownloadError {
    FetchSignature(anyhow::Error),
    FetchApp(anyhow::Error),
    Verification(anyhow::Error),
}

/// Parameters required to construct an [AppDownloader].
pub struct AppDownloaderParameters<SigProgress, AppProgress> {
    pub signature_url: String,
    pub app_url: String,
    pub app_size: usize,
    pub sig_progress: SigProgress,
    pub app_progress: AppProgress,
}

/// See the [module-level documentation](self).
#[async_trait::async_trait]
pub trait AppDownloader: Send {
    /// Download the app signature.
    async fn download_signature(&mut self) -> Result<(), DownloadError>;

    /// Download the app binary.
    async fn download_executable(&mut self) -> Result<(), DownloadError>;

    /// Verify the app signature.
    async fn verify(&mut self) -> Result<(), DownloadError>;
}

/// Download the app and signature, and verify the app's signature
pub async fn install_and_upgrade(mut downloader: impl AppDownloader) -> Result<(), DownloadError> {
    downloader.download_signature().await?;
    downloader.download_executable().await?;
    downloader.verify().await
}

#[derive(Clone)]
pub struct HttpAppDownloader<SigProgress, AppProgress> {
    signature_url: String,
    app_url: String,
    app_size: usize,
    signature_progress_updater: SigProgress,
    app_progress_updater: AppProgress,
    // TODO: set permissions
    tmp_dir: PathBuf,
}

impl<SigProgress, AppProgress> HttpAppDownloader<SigProgress, AppProgress> {
    const MAX_SIGNATURE_SIZE: usize = 1024;

    pub fn new(parameters: AppDownloaderParameters<SigProgress, AppProgress>) -> Self {
        let tmp_dir = std::env::temp_dir();
        Self {
            signature_url: parameters.signature_url,
            app_url: parameters.app_url,
            app_size: parameters.app_size,
            signature_progress_updater: parameters.sig_progress,
            app_progress_updater: parameters.app_progress,
            tmp_dir,
        }
    }
}

impl<SigProgress: ProgressUpdater, AppProgress: ProgressUpdater>
    From<AppDownloaderParameters<SigProgress, AppProgress>>
    for HttpAppDownloader<SigProgress, AppProgress>
{
    fn from(parameters: AppDownloaderParameters<SigProgress, AppProgress>) -> Self {
        HttpAppDownloader::new(parameters)
    }
}

#[async_trait::async_trait]
impl<SigProgress: ProgressUpdater, AppProgress: ProgressUpdater> AppDownloader
    for HttpAppDownloader<SigProgress, AppProgress>
{
    async fn download_signature(&mut self) -> Result<(), DownloadError> {
        fetch::get_to_file(
            self.sig_path(),
            &self.signature_url,
            &mut self.signature_progress_updater,
            fetch::SizeHint::Maximum(Self::MAX_SIGNATURE_SIZE),
        )
        .await
        .map_err(DownloadError::FetchSignature)
    }

    async fn download_executable(&mut self) -> Result<(), DownloadError> {
        fetch::get_to_file(
            self.bin_path(),
            &self.app_url,
            &mut self.app_progress_updater,
            // FIXME: use exact size hint
            fetch::SizeHint::Maximum(self.app_size),
        )
        .await
        .map_err(DownloadError::FetchApp)
    }

    async fn verify(&mut self) -> Result<(), DownloadError> {
        let bin_path = self.bin_path();
        let sig_path = self.sig_path();
        tokio::task::spawn_blocking(move || {
            PgpVerifier::verify(bin_path, sig_path).map_err(DownloadError::Verification)
        })
        .await
        .expect("verifier panicked")
    }
}

impl<SigProgress, AppProgress> HttpAppDownloader<SigProgress, AppProgress> {
    fn bin_path(&self) -> PathBuf {
        self.tmp_dir.join("temp.exe")
    }

    fn sig_path(&self) -> PathBuf {
        self.tmp_dir.join("temp.exe.sig")
    }
}
