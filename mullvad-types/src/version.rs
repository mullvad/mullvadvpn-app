/// `LatestAppVersions` represent the current stable and the current latest release versions of the
/// Mullvad VPN app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppVersionInfo {
    pub current_version: String,
    pub is_supported: bool,
    pub latest: LatestReleases,
}

/// `Latest` versions represent the latest released versions of the Mullvad VPN app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestReleases {
    pub latest_stable: String,
    pub latest: String,
}
