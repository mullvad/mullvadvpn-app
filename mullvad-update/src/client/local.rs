//! This module implements fetching of information about app versions from disk.

use std::path::PathBuf;

use super::version_provider::VersionInfoProvider;

/// Obtain version data from disk
pub struct DirectoryVersionInfoProvider {
    /// Path to directory containing the metadata file.
    pub directory: PathBuf,
}

impl VersionInfoProvider for DirectoryVersionInfoProvider {
    fn get_version_info(
        &self,
        params: crate::version::VersionParameters,
    ) -> impl std::future::Future<Output = anyhow::Result<crate::version::VersionInfo>> + Send {
        std::future::ready(todo!())
    }
}
