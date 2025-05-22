//! This module implements fetching of information about app versions from disk.

use anyhow::Context;
use std::path::PathBuf;
use tokio::fs;

use crate::{
    format::SignedResponse,
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
        let raw_json = fs::read(metadata_file)
            .await
            .context("Failed to read metadata.json")?;

        let response =
            SignedResponse::deserialize_and_verify(&raw_json, params.lowest_metadata_version)
                .context("Failed to deserialize or verify metadata.json")?;

        let version_info = VersionInfo::try_from_response(&params, response.signed)?;

        Ok(Self {
            directory,
            version_info,
        })
    }
}
