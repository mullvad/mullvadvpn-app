use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

/// AppVersionInfo represents the current stable and the current latest release versions of the
/// Mullvad VPN app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppVersionInfo {
    /// False if Mullvad has stopped supporting the currently running version. This could mean
    /// a number of things. For example:
    /// * API endpoints it uses might not work any more.
    /// * Software bundled with this version, such as OpenVPN or OpenSSL, has known security
    ///   issues, so using it is no longer recommended.
    ///
    /// The user should really upgrade when this is false.
    pub current_version_supported: bool,
    /// A newer version that may be upgraded to
    pub suggested_upgrade: Option<SuggestedUpgrade>,
}

impl Display for AppVersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Suggested upgrade: {}, current version supported: {}",
            self.suggested_upgrade
                .as_ref()
                .map_or("None".to_string(), |upgrade| {
                    format!("{}", upgrade.version)
                }),
            self.current_version_supported,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SuggestedUpgrade {
    /// Version available for update
    pub version: mullvad_version::Version,
    /// Changelog
    pub changelog: String,
    /// Path to the available installer, iff it has been verified
    pub verified_installer_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppUpgradeDownloadProgress {
    pub server: String,
    pub progress: u32,
    pub time_left: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppUpgradeEvent {
    DownloadStarting,
    DownloadProgress(AppUpgradeDownloadProgress),
    Aborted,
    VerifyingInstaller,
    VerifiedInstaller,
    Error(AppUpgradeError),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppUpgradeError {
    GeneralError,
    DownloadFailed,
    VerificationFailed,
}
