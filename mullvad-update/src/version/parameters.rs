use crate::format::Architecture;
use crate::version::rollout::Rollout;

/// Query type for [VersionInfo]
#[derive(Debug, Clone)]
pub struct VersionParameters {
    /// Architecture to retrieve data for
    pub architecture: VersionArchitecture,
    /// Rollout threshold. Any version in the response below this threshold will be ignored
    pub rollout: Rollout,
    /// Allow versions without any installers to be returned
    pub allow_empty: bool,
    /// Lowest allowed `metadata_version` in the version data
    /// Typically the current version plus 1
    pub lowest_metadata_version: usize,
}

/// Installer architecture
pub type VersionArchitecture = Architecture;
