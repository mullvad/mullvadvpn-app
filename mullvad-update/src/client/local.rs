//! This module implements fetching of information about app versions from disk.

use anyhow::Context;
use mullvad_version::Version;
use std::path::PathBuf;
use tokio::fs;

use crate::{
    format::SignedResponse,
    version::{VersionInfo, VersionParameters},
};

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

    async fn get_app(self) -> anyhow::Result<(Version, InstallerFile<false>)> {
        let metadata_file = self.directory.join(METADATA_FILENAME);
        let raw_json = fs::read(metadata_file)
            .await
            .context("Failed to read metadata.json")?;

        let response = SignedResponse::deserialize_and_verify(
            &raw_json,
            self.version_params.lowest_metadata_version,
        )
        .context("Failed to deserialize or verify metadata.json")?;

        let version_info = VersionInfo::try_from_response(&self.version_params, response.signed)?;

        // TODO: beta?
        let version_info = version_info.stable;
        let version = version_info.version;

        let installer = InstallerFile::<false>::from_version(
            &self.directory,
            version.clone(),
            version_info.size,
            version_info.sha256,
        );

        Ok((version, installer))
    }
}

/// App cacher that does not return anything
pub struct NoopAppCacheDir;

impl AppCache for NoopAppCacheDir {
    type Installer = InstallerFile<false>;

    fn new(_directory: PathBuf, _version_params: VersionParameters) -> Self {
        NoopAppCacheDir
    }

    async fn get_app(self) -> anyhow::Result<(Version, InstallerFile<false>)> {
        Err(anyhow::anyhow!("No cache"))
    }
}
