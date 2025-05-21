//! This module implements fetching of information about app versions from disk.

use std::path::PathBuf;

use super::version_provider::VersionInfoProvider;

/// Obtain version data from disk
pub struct DirectoryVersionInfoProvider {
    /// Path to directory containing the metadata file.
    directory: PathBuf,
}
