//! This module implements the flow of downloading and verifying the app signature.

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
pub trait AppDownloader {
    /// Download the app signature.
    fn download_signature(&mut self) -> Result<(), DownloadError>;

    /// Download the app binary.
    fn download_executable(&mut self) -> Result<(), DownloadError>;

    /// Verify the app signature.
    fn verify(&mut self) -> Result<(), DownloadError>;
}

/// Download the app and signature, and verify the app's signature
pub fn install_and_upgrade(mut downloader: impl AppDownloader) -> Result<(), DownloadError> {
    downloader.download_signature()?;
    downloader.download_executable()?;
    downloader.verify()
}

#[derive(Clone)]
pub struct LatestAppDownloader<SigProgress, AppProgress> {
    signature_url: &'static str,
    app_url: &'static str,
    signature_progress_updater: SigProgress,
    app_progress_updater: AppProgress,
}

impl<SigProgress, AppProgress> LatestAppDownloader<SigProgress, AppProgress> {
    pub fn stable(
        signature_progress_updater: SigProgress,
        app_progress_updater: AppProgress,
    ) -> Self {
        Self {
            signature_url: "https://mullvad.net/en/download/app/exe/latest/signature",
            app_url: "https://mullvad.net/en/download/app/exe/latest",
            signature_progress_updater,
            app_progress_updater,
        }
    }

    pub fn beta(
        signature_progress_updater: SigProgress,
        app_progress_updater: AppProgress,
    ) -> Self {
        Self {
            signature_url: "https://mullvad.net/en/download/app/exe/latest-beta/signature",
            app_url: "https://mullvad.net/en/download/app/exe/latest-beta",
            signature_progress_updater,
            app_progress_updater,
        }
    }
}

impl<SigProgress: ProgressUpdater, AppProgress: ProgressUpdater> AppDownloader
    for LatestAppDownloader<SigProgress, AppProgress>
{
    fn download_signature(&mut self) -> Result<(), DownloadError> {
        fetch::get_to_file(
            "temp.exe.sig",
            &self.signature_url,
            &mut self.signature_progress_updater,
            1 * 1024,
        )
        .map_err(DownloadError::FetchSignature)
    }

    fn download_executable(&mut self) -> Result<(), DownloadError> {
        fetch::get_to_file(
            "temp.exe",
            &self.app_url,
            &mut self.app_progress_updater,
            100 * 1024 * 1024,
        )
        .map_err(DownloadError::FetchApp)
    }

    fn verify(&mut self) -> Result<(), DownloadError> {
        PgpVerifier::verify("temp.exe", "temp.exe.sig").map_err(DownloadError::Verification)
    }
}
