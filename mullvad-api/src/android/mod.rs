use serde::{Deserialize, Serialize};

/// Android releases
#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct AndroidReleases {
    /// Available app releases
    pub releases: Vec<Release>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd)]
pub struct Release {
    /// Mullvad app version
    pub version: mullvad_version::Version,
}

pub fn is_version_supported_android(
    current_version: &mullvad_version::Version,
    response: &AndroidReleases,
) -> bool {
    response
        .releases
        .iter()
        .any(|release| release.version == *current_version)
}
