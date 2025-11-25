use crate::version::rollout::Rollout;

/// Query type for [VersionInfo]
#[derive(Debug, Clone)]
pub struct VersionParameters {
    /// Architecture to retrieve data for
    pub architecture: Architecture,
    /// Rollout threshold. Any version in the response below this threshold will be ignored
    pub rollout: Rollout,
    /// Allow versions without any installers to be returned
    pub allow_empty: bool,
    /// Lowest allowed `metadata_version` in the version data
    /// Typically the current version plus 1
    pub lowest_metadata_version: usize,
}

/// Installer architecture
pub type Architecture = crate::format::Architecture;
