#![cfg(any(target_os = "macos", target_os = "windows"))]

//! This module implements the flow of downloading and verifying the app.

use std::{ffi::OsString, path::PathBuf, time::Duration};

use tokio::{process::Command, time::timeout};

use crate::{
    fetch::{self, ProgressUpdater},
    verify::{AppVerifier, Sha256Verifier},
};

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Failed to download app")]
    FetchApp(#[source] anyhow::Error),
    #[error("Failed to verify app")]
    Verification(#[source] anyhow::Error),
    #[error("Failed to launch app")]
    Launch(#[source] std::io::Error),
    #[error("Installer exited with error: {0}")]
    InstallExited(std::process::ExitStatus),
    #[error("Installer failed on child.wait(): {0}")]
    InstallFailed(std::io::Error),
}

/// Parameters required to construct an [AppDownloader].
#[derive(Clone)]
pub struct AppDownloaderParameters<AppProgress> {
    pub app_version: mullvad_version::Version,
    pub app_url: String,
    pub app_size: usize,
    pub app_progress: AppProgress,
    pub app_sha256: [u8; 32],
    /// Directory to store the installer in.
    /// Ensure that this has proper permissions set.
    pub cache_dir: PathBuf,
}

/// See the [module-level documentation](self).
#[async_trait::async_trait]
pub trait AppDownloader: Send {
    /// Download the app binary.
    async fn download_executable(&mut self) -> Result<(), DownloadError>;

    /// Verify the app signature.
    async fn verify(&mut self) -> Result<(), DownloadError>;

    /// Execute installer.
    async fn install(&mut self) -> Result<(), DownloadError>;
}

/// How long to wait for the installer to exit before returning
const INSTALLER_STARTUP_TIMEOUT: Duration = Duration::from_millis(500);

/// Download the app and signature, and verify the app's signature
pub async fn install_and_upgrade(mut downloader: impl AppDownloader) -> Result<(), DownloadError> {
    downloader.download_executable().await?;
    downloader.verify().await?;
    downloader.install().await
}

#[derive(Clone)]
pub struct HttpAppDownloader<AppProgress> {
    params: AppDownloaderParameters<AppProgress>,
}

impl<AppProgress> HttpAppDownloader<AppProgress> {
    pub fn new(params: AppDownloaderParameters<AppProgress>) -> Self {
        Self { params }
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
        let bin_path = self.bin_path();
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
        let bin_path = self.bin_path();
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
        let launch_path = self.launch_path();

        // Launch process
        let mut cmd = Command::new(launch_path);
        cmd.args(self.launch_args());
        let mut child = cmd.spawn().map_err(DownloadError::Launch)?;

        // Wait to see if the installer fails
        match timeout(INSTALLER_STARTUP_TIMEOUT, child.wait()).await {
            // Timeout: Quit and let the installer take over
            Err(_timeout) => Ok(()),
            // No timeout: Incredibly quick but successful (or wrong exit code, probably)
            Ok(Ok(status)) if status.success() => Ok(()),
            // Installer exited with error code
            Ok(Ok(status)) => Err(DownloadError::InstallExited(status)),
            // `child.wait()` returned an error
            Ok(Err(err)) => Err(DownloadError::InstallFailed(err)),
        }
    }
}

impl<AppProgress> HttpAppDownloader<AppProgress> {
    fn bin_path(&self) -> PathBuf {
        #[cfg(windows)]
        let bin_filename = format!("mullvad-{}.exe", self.params.app_version);

        #[cfg(target_os = "macos")]
        let bin_filename = format!("mullvad-{}.pkg", self.params.app_version);

        self.params.cache_dir.join(bin_filename)
    }

    fn launch_path(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.bin_path()
        }

        #[cfg(target_os = "macos")]
        {
            use std::path::Path;

            Path::new("/usr/bin/open").to_owned()
        }
    }

    fn launch_args(&self) -> Vec<OsString> {
        #[cfg(target_os = "windows")]
        {
            vec![]
        }

        #[cfg(target_os = "macos")]
        {
            vec![self.bin_path().into()]
        }
    }

    fn hash_sha256(&self) -> &[u8; 32] {
        &self.params.app_sha256
    }
}
