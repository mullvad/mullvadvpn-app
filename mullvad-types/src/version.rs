use serde::{Deserialize, Serialize};

/// AppVersionInfo represents the current stable and the current latest release versions of the
/// Mullvad VPN app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppVersionInfo {
    pub current_is_supported: bool,
    pub latest: LatestReleases,
}

/// LatestReleases represent the latest released versions of the Mullvad VPN app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestReleases {
    pub latest_stable: AppVersion,
    pub latest: AppVersion,
}

pub type AppVersion = String;
