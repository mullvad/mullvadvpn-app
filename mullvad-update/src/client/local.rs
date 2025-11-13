#![cfg(any(target_os = "macos", target_os = "windows"))]

//! This module implements fetching of information about app versions from disk.

use anyhow::{Context, bail};
use std::{path::PathBuf, vec};
use tokio::fs;

use crate::format::response::SignedResponse;
use crate::version::VersionParameters;

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

    async fn get_metadata(&self) -> anyhow::Result<SignedResponse> {
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

    /// Get an iterator of cached installers for the current architecture
    fn get_cached_installers(self, metadata: SignedResponse) -> Vec<Self::Installer> {
        let releases = metadata.get_releases();
        releases
            .into_iter()
            // Map releases to their version and its installer for the current architecture
            .flat_map(move |release| {
                release
                    .installers
                    .into_iter()
                    .find(|installer| installer.architecture == self.version_params.architecture)
                    .map(|installer| (release.version, installer))
            })
            // Map to an `InstallerFile`, and filter out installers not present in cache
            .filter_map(move |(version, installer)| {
                InstallerFile::<false>::try_from_installer(&self.directory, version, installer).ok()
            }).collect()
    }
}

/// App cacher that does not return anything
pub struct NoopAppCacheDir;

impl AppCache for NoopAppCacheDir {
    type Installer = InstallerFile<false>;

    fn new(_directory: PathBuf, _version_params: VersionParameters) -> Self {
        NoopAppCacheDir
    }

    fn get_cached_installers(self, _metadata: SignedResponse) -> Vec<Self::Installer> {
        vec![]
    }

    async fn get_metadata(&self) -> anyhow::Result<SignedResponse> {
        bail!("NoopAppCacheDir can not present any metadata")
    }
}
