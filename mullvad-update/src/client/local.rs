//! This module implements fetching of information about app versions from disk.

use std::path::PathBuf;
use tokio::fs;

use crate::{format::Response, version::VersionInfo};

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
    pub async fn new(
        directory: PathBuf,
        params: crate::version::VersionParameters,
    ) -> anyhow::Result<Self> {
        let metadata_file = self.directory.join(Self::METADATA);
        let bytes = fs::read(metadata_file).await?;
        let response: Response = serde_json::from_slice(&bytes)?;

        let version_info = VersionInfo::try_from_response(&params, response)?;

        Ok(Self {
            directory,
            version_info,
        })
    }
}
