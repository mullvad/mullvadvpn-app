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
    /// A newer version that may be upgraded to
    pub suggested_upgrade: Option<SuggestedUpgrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SuggestedUpgrade {
    /// Version available for update
    pub version: mullvad_version::Version,
    /// Changelog
    pub changelog: Option<String>,
    /// Path to the available installer, iff it has been verified
    pub verified_installer_path: Option<PathBuf>,
}
