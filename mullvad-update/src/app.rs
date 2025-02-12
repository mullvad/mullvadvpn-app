//! This module implements the flow of downloading and verifying the app.

use std::path::PathBuf;

use crate::{
    fetch::{self, ProgressUpdater},
    verify::{AppVerifier, Sha256Verifier},
};

#[derive(Debug)]
pub enum DownloadError {
    FetchSignature(anyhow::Error),
    FetchApp(anyhow::Error),
    Verification(anyhow::Error),
}

/// Parameters required to construct an [AppDownloader].
#[derive(Clone)]
pub struct AppDownloaderParameters<AppProgress> {
    pub app_url: String,
    pub app_size: usize,
    pub app_progress: AppProgress,
    pub app_sha256: [u8; 32],
}

/// See the [module-level documentation](self).
#[async_trait::async_trait]
pub trait AppDownloader: Send {
    /// Download the app binary.
    async fn download_executable(&mut self) -> Result<(), DownloadError>;

    /// Verify the app signature.
    async fn verify(&mut self) -> Result<(), DownloadError>;
}

/// Download the app and signature, and verify the app's signature
pub async fn install_and_upgrade(mut downloader: impl AppDownloader) -> Result<(), DownloadError> {
    downloader.download_executable().await?;
    downloader.verify().await
}

#[derive(Clone)]
pub struct HttpAppDownloader<AppProgress> {
    params: AppDownloaderParameters<AppProgress>,
    // TODO: set permissions
    tmp_dir: PathBuf,
}

impl<AppProgress> HttpAppDownloader<AppProgress> {
    pub fn new(params: AppDownloaderParameters<AppProgress>) -> Self {
        let tmp_dir = std::env::temp_dir();
        Self { params, tmp_dir }
    }
}

impl<AppProgress: ProgressUpdater> From<AppDownloaderParameters<AppProgress>>
    for HttpAppDownloader<AppProgress>
{
    fn from(parameters: AppDownloaderParameters<AppProgress>) -> Self {
        HttpAppDownloader::new(parameters)
    }
}

#[async_trait::async_trait]
impl<AppProgress: ProgressUpdater> AppDownloader for HttpAppDownloader<AppProgress> {
    async fn download_executable(&mut self) -> Result<(), DownloadError> {
        fetch::get_to_file(
            self.bin_path(),
            &self.params.app_url,
            &mut self.params.app_progress,
            fetch::SizeHint::Exact(self.params.app_size),
        )
        .await
        .map_err(DownloadError::FetchApp)
    }

    async fn verify(&mut self) -> Result<(), DownloadError> {
        let bin_path = self.bin_path();
        let hash = self.hash_sha256();
        Sha256Verifier::verify(bin_path, *hash)
            .await
            .map_err(DownloadError::Verification)
    }
}

impl<AppProgress> HttpAppDownloader<AppProgress> {
    fn bin_path(&self) -> PathBuf {
        self.tmp_dir.join("temp.exe")
    }

    fn hash_sha256(&self) -> &[u8; 32] {
        &self.params.app_sha256
    }
}
