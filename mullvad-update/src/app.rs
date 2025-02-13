//! This module implements the flow of downloading and verifying the app.

use std::{path::PathBuf, time::Duration};

use tokio::{process::Command, time::timeout};

use crate::{
    fetch::{self, ProgressUpdater},
    verify::{AppVerifier, Sha256Verifier},
};

const INSTALLER_STARTUP_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub enum DownloadError {
    CreateDir(anyhow::Error),
    FetchSignature(anyhow::Error),
    FetchApp(anyhow::Error),
    Verification(anyhow::Error),
    Launch(std::io::Error),
    InstallFailed(anyhow::Error),
}

/// Parameters required to construct an [AppDownloader].
#[derive(Clone)]
pub struct AppDownloaderParameters<AppProgress> {
    pub app_version: mullvad_version::Version,
    pub app_url: String,
    pub app_size: usize,
    pub app_progress: AppProgress,
    pub app_sha256: [u8; 32],
}

/// See the [module-level documentation](self).
#[async_trait::async_trait]
pub trait AppDownloader: Send {
    /// Create download directory.
    async fn create_cache_dir(&mut self) -> Result<(), DownloadError>;

    /// Download the app binary.
    async fn download_executable(&mut self) -> Result<(), DownloadError>;

    /// Verify the app signature.
    async fn verify(&mut self) -> Result<(), DownloadError>;

    /// Execute installer.
    async fn install(&mut self) -> Result<(), DownloadError>;
}

/// Download the app and signature, and verify the app's signature
pub async fn install_and_upgrade(mut downloader: impl AppDownloader) -> Result<(), DownloadError> {
    downloader.create_cache_dir().await?;
    downloader.download_executable().await?;
    downloader.verify().await?;
    downloader.install().await
}

#[derive(Clone)]
pub struct HttpAppDownloader<AppProgress> {
    params: AppDownloaderParameters<AppProgress>,
    cache_dir: Option<PathBuf>,
}

impl<AppProgress> HttpAppDownloader<AppProgress> {
    pub fn new(params: AppDownloaderParameters<AppProgress>) -> Self {
        Self { params, cache_dir: None }
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
    async fn create_cache_dir(&mut self) -> Result<(), DownloadError> {
        let dir = crate::dir::update_directory().await.map_err(DownloadError::CreateDir)?;
        self.cache_dir = Some(dir);
        Ok(())
    }

    async fn download_executable(&mut self) -> Result<(), DownloadError> {
        let bin_path = self.bin_path().expect("Performed after 'create_cache_dir'");
        fetch::get_to_file(
            bin_path,
            &self.params.app_url,
            &mut self.params.app_progress,
            fetch::SizeHint::Exact(self.params.app_size),
        )
        .await
        .map_err(DownloadError::FetchApp)
    }

    async fn verify(&mut self) -> Result<(), DownloadError> {
        let bin_path = self.bin_path().expect("Performed after 'create_cache_dir'");
        let hash = self.hash_sha256();

        match Sha256Verifier::verify(&bin_path, *hash)
            .await
            .map_err(DownloadError::Verification)
        {
            // Verification succeeded
            Ok(()) => Ok(()),
            // Verification failed
            Err(err) => {
                // Attempt to clean up
                let _ = tokio::fs::remove_file(bin_path).await;
                Err(err)
            }
        }
    }

    async fn install(&mut self) -> Result<(), DownloadError> {
        let bin_path = self.bin_path().expect("Performed after 'create_cache_dir'");

        // Launch process
        // TODO: move to launch.rs?
        let mut cmd = Command::new(bin_path);
        let mut child = cmd.spawn().map_err(DownloadError::Launch)?;

        // Wait to see if the installer fails
        match timeout(INSTALLER_STARTUP_TIMEOUT, child.wait()).await {
            // Timeout: Quit and let the installer take over
            Err(_timeout) => Ok(()),
            // No timeout: Incredibly quick but successful (or wrong exit code, probably)
            Ok(Ok(status)) if status.success() => Ok(()),
            // Installer failed
            Ok(Ok(status)) => Err(DownloadError::InstallFailed(anyhow::anyhow!("Install failed with status: {status}"))),
            // Installer failed
            Ok(Err(err)) => Err(DownloadError::InstallFailed(anyhow::anyhow!("Install failed : {err}"))),
        }
    }
}

impl<AppProgress> HttpAppDownloader<AppProgress> {
    fn bin_path(&self) -> Option<PathBuf> {
        #[cfg(windows)]
        let bin_filename = format!("{}.exe", self.params.app_version);

        #[cfg(unix)]
        let bin_filename = self.params.app_version.to_string();

        self.cache_dir.as_ref().map(|dir| dir.join(bin_filename))
    }

    fn hash_sha256(&self) -> &[u8; 32] {
        &self.params.app_sha256
    }
}
