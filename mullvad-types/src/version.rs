use std::path::PathBuf;

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
    pub supported: bool,
    /// Latest stable version
    pub latest_stable: AppVersion,
    /// Equal to `latest_stable` when the newest release is a stable release. But will contain
    /// beta versions when those are out for testing.
    pub latest_beta: AppVersion,
    /// Whether should update to newer version
    pub suggested_upgrade: Option<AppVersion>,
}

pub type AppVersion = String;


/// AppVersionInfo represents the current stable and the current latest release versions of the
/// Mullvad VPN app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppVersionInfo2 {
    /// False if Mullvad has stopped supporting the currently running version. This could mean
    /// a number of things. For example:
    /// * API endpoints it uses might not work any more.
    /// * Software bundled with this version, such as OpenVPN or OpenSSL, has known security
    ///   issues, so using it is no longer recommended.
    ///
    /// The user should really upgrade when this is false.
    pub supported: bool,
    /// Whether should update to newer version
    pub suggested_upgrade: Option<AppVersion>,
}

pub struct SuggestedUpgrade {
    /// Version available for update
    pub version: AppVersion,
    /// Changelog
    pub changelog: Option<String>,
    /// Path to the available installer, iff it has been verified
    pub verified_installer_path: Option<PathBuf>,
}
