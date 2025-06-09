//! This module implements fetching of information about app versions from disk.

use anyhow::Context;
use std::path::PathBuf;
use tokio::fs;

use crate::{format::SignedResponse, version::VersionParameters};

use super::app::{AppCache, InstallerFile};

pub struct AppCacheDir {
    /// Path to directory containing the metadata file and the downloaded installer.
    pub directory: PathBuf,
    pub version_params: VersionParameters,
}

pub const METADATA_FILENAME: &str = "metadata.json";

impl AppCache for AppCacheDir {
    type Installer = InstallerFile<false>;

    fn new(directory: PathBuf, version_params: VersionParameters) -> Self {
        Self {
            directory,
            version_params,
        }
    }

    async fn get_downloaded_installers(
        self,
    ) -> anyhow::Result<impl Iterator<Item = Self::Installer>> {
        let response_metadata = self.get_metadata().await?;
        let releases = response_metadata.get_releases();
        let installers = crate::version::get_installers(releases); // installers are sorted by version here
        Ok(installers
            .into_iter()
            .filter(move |(_, installer)| {
                installer.architecture == self.version_params.architecture
            })
            .filter_map(move |(version, installer)| {
                InstallerFile::<false>::try_from_installer(&self.directory, version, installer).ok()
            }))
    }
}

impl AppCacheDir {
    async fn get_metadata(&self) -> Result<crate::format::SignedResponse, anyhow::Error> {
        let metadata_file = self.directory.join(METADATA_FILENAME);
        let raw_json = fs::read(metadata_file)
            .await
            .context("Failed to read metadata.json")?;
        let response = SignedResponse::deserialize_and_verify(
            &raw_json,
            self.version_params.lowest_metadata_version,
        )
        .context("Failed to deserialize or verify metadata.json")?;
        Ok(response)
    }
}

/// App cacher that does not return anything
pub struct NoopAppCacheDir;

impl AppCache for NoopAppCacheDir {
    type Installer = InstallerFile<false>;

    fn new(_directory: PathBuf, _version_params: VersionParameters) -> Self {
        NoopAppCacheDir
    }

    async fn get_downloaded_installers(
        self,
    ) -> anyhow::Result<impl Iterator<Item = Self::Installer>> {
        Err::<std::iter::Empty<_>, _>(anyhow::anyhow!("No cache"))
    }
}
