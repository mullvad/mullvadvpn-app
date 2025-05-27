use std::path::PathBuf;

use crate::version::{VersionInfo, VersionParameters};

/// See [module-level](self) docs.
pub trait VersionInfoProvider {
    /// Return info about the stable version
    fn get_version_info(
        &self,
        params: &VersionParameters,
    ) -> impl std::future::Future<Output = anyhow::Result<VersionInfo>> + Send;

    fn set_metadata_dump_path(&mut self, path: PathBuf);
}
