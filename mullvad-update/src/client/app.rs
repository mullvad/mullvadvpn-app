#![cfg(any(target_os = "macos", target_os = "windows"))]

//! This module implements the flow of downloading and verifying the app.

use std::{
    ffi::OsString,
    future::Future,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{bail, Context};
use tokio::{process::Command, time::timeout};

use crate::{
    fetch::{self, ProgressUpdater},
    format::SignedResponse,
    verify::{AppVerifier, Sha256Verifier},
    version::VersionParameters,
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
pub trait AppDownloader: Send + 'static {
    /// Download the app binary.
    fn download_executable(
        self,
    ) -> impl Future<Output = Result<impl DownloadedInstaller, DownloadError>> + Send;
}

/// A cache where we can find past [DownloadedInstaller]s
pub trait AppCache: Send {
    type Installer: DownloadedInstaller + Clone + PartialOrd;

    fn new(directory: PathBuf, version_params: VersionParameters) -> Self;
    fn get_metadata(&self) -> impl Future<Output = anyhow::Result<SignedResponse>> + Send;
    fn get_cached_installers(self, metadata: SignedResponse) -> Vec<Self::Installer>;
}

pub trait DownloadedInstaller: Send + 'static {
    /// Verify the app signature.
    fn verify(self) -> impl Future<Output = Result<impl VerifiedInstaller, DownloadError>> + Send;

    fn version(&self) -> &mullvad_version::Version;
}

pub trait VerifiedInstaller: Send {
    /// Execute installer.
    fn install(self) -> impl Future<Output = Result<(), DownloadError>> + Send;
}

/// How long to wait for the installer to exit before returning
const INSTALLER_STARTUP_TIMEOUT: Duration = Duration::from_millis(500);

/// Download the app and signature, and verify the installer's signature
pub async fn download_install_and_upgrade(
    downloader: impl AppDownloader,
) -> Result<(), DownloadError> {
    downloader
        .download_executable()
        .await?
        .verify()
        .await?
        .install()
        .await
}

/// Verify and run the installer.
pub async fn install_and_upgrade(installer: impl DownloadedInstaller) -> Result<(), DownloadError> {
    installer.verify().await?.install().await
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

#[derive(Clone)]
pub struct InstallerFile<const VERIFIED: bool> {
    path: PathBuf,
    pub app_version: mullvad_version::Version,
    pub app_size: usize,
    pub app_sha256: [u8; 32],
}

// TODO: Explain
impl<const VERIFIED: bool> PartialOrd for InstallerFile<VERIFIED> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.app_version.partial_cmp(&other.app_version)
    }
}

// TODO: Explain
impl<const VERIFIED: bool> PartialEq for InstallerFile<VERIFIED> {
    fn eq(&self, other: &Self) -> bool {
        self.app_version == other.app_version
    }
}

impl<AppProgress: ProgressUpdater> AppDownloader for HttpAppDownloader<AppProgress> {
    async fn download_executable(mut self) -> Result<impl DownloadedInstaller, DownloadError> {
        let bin_path = bin_path(&self.params.cache_dir, &self.params.app_version);
        fetch::get_to_file(
            &bin_path,
            &self.params.app_url,
            &mut self.params.app_progress,
            fetch::SizeHint::Exact(self.params.app_size),
        )
        .await
        .map_err(DownloadError::FetchApp)?;

        Ok(InstallerFile::<false> {
            path: bin_path,
            app_version: self.params.app_version,
            app_size: self.params.app_size,
            app_sha256: self.params.app_sha256,
        })
    }
}

impl DownloadedInstaller for InstallerFile<false> {
    async fn verify(self) -> Result<impl VerifiedInstaller, DownloadError> {
        match Sha256Verifier::verify(&self.path, self.app_sha256)
            .await
            .map_err(DownloadError::Verification)
        {
            // Verification succeeded
            Ok(()) => Ok(InstallerFile::<true> {
                path: self.path,
                app_version: self.app_version,
                app_size: self.app_size,
                app_sha256: self.app_sha256,
            }),
            // Verification failed
            Err(err) => {
                // Attempt to clean up
                let _ = tokio::fs::remove_file(&self.path).await;
                Err(err)
            }
        }
    }

    fn version(&self) -> &mullvad_version::Version {
        &self.app_version
    }
}

impl VerifiedInstaller for InstallerFile<true> {
    async fn install(self) -> Result<(), DownloadError> {
        let launch_path = &self.launch_path();

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

fn bin_path(cache_dir: &Path, app_version: &mullvad_version::Version) -> PathBuf {
    #[cfg(windows)]
    let bin_filename = format!("mullvad-{}.exe", app_version);

    #[cfg(target_os = "macos")]
    let bin_filename = format!("mullvad-{}.pkg", app_version);

    cache_dir.join(bin_filename)
}

impl InstallerFile<false> {
    /// Create an unverified [InstallerFile] from a cache_dir and some metadata.
    pub fn try_from_version(
        cache_dir: &Path,
        version: crate::version::Version,
    ) -> anyhow::Result<Self> {
        let path = bin_path(cache_dir, &version.version);
        if !path.exists() {
            bail!("Installer file does not exist at path: {}", path.display());
        }
        Ok(Self {
            path,
            app_version: version.version,
            app_size: version.size,
            app_sha256: version.sha256,
        })
    }

    pub fn try_from_installer(
        cache_dir: &Path,
        app_version: mullvad_version::Version,
        installer: crate::format::Installer,
    ) -> anyhow::Result<Self> {
        let path = bin_path(cache_dir, &app_version);
        if !path.exists() {
            bail!("Installer file does not exist at path: {}", path.display());
        }
        let app_sha256 = hex::decode(installer.sha256)
            .context("Invalid checksum hex")?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid checksum length"))?;
        Ok(Self {
            path,
            app_version,
            app_size: installer.size,
            app_sha256,
        })
    }
}

impl<const VERIFIED: bool> InstallerFile<VERIFIED> {
    fn launch_path(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.path.clone()
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
            vec![self.path.clone().into_os_string()]
        }
    }
}
