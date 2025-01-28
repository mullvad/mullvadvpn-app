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

/// See the [module-level documentation](crate).
#[async_trait::async_trait]
pub trait AppDownloader {
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
    const MAX_SIGNATURE_SIZE: usize = 1 * 1024;

    pub fn new(
        signature_url: &str,
        app_url: &str,
        app_size: usize,
        signature_progress_updater: SigProgress,
        app_progress_updater: AppProgress,
    ) -> Self {
        let tmp_dir = std::env::temp_dir();
        Self {
            signature_url: signature_url.to_owned(),
            app_url: app_url.to_owned(),
            app_size,
            signature_progress_updater,
            app_progress_updater,
            tmp_dir,
        }
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
            fetch::SizeHint::Exact(self.app_size),
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
