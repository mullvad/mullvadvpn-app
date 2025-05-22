//! This module implements fetching of information about app versions from disk.

use anyhow::Context;
use std::path::PathBuf;
use tokio::fs;

use crate::{
    format::Response,
    version::{VersionInfo, VersionParameters},
};

/// Obtain version data from disk
pub struct DirectoryVersionInfoProvider {
    /// Path to directory containing the metadata file.
    pub directory: PathBuf,
    /// TODO
    pub version_info: VersionInfo,
}

impl DirectoryVersionInfoProvider {
    pub const METADATA: &str = "metadata.json";

    /// Read metadata.json from the local directory
    pub async fn new(directory: PathBuf, params: VersionParameters) -> anyhow::Result<Self> {
        let metadata_file = directory.join(Self::METADATA);
        let bytes = fs::read(metadata_file)
            .await
            .context("Failed to read metadata.json")?;
        let response: Response = serde_json::from_slice(&bytes)?;

        let version_info = VersionInfo::try_from_response(&params, response)?;

        Ok(Self {
            directory,
            version_info,
        })
    }
}
