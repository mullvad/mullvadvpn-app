#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};

/// AppVersionInfo represents the current stable and the current latest release versions of the
/// Mullvad VPN app.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct AppVersionInfo {
    /// False if Mullvad has stopped supporting the currently running version. This could mean
    /// a number of things. For example:
    /// * API endpoints it uses might not work any more.
    /// * Software bundled with this version, such as OpenVPN or OpenSSL, has known security
    ///   issues, so using it is no longer recommended.
    /// The user should really upgrade when this is false.
    pub current_is_supported: bool,
    /// True if there is a newer version that contains any functional differences compared to the
    /// running version. User should upgrade if they want the latest features and bugfixes.
    /// DEPRECATED
    pub current_is_outdated: bool,
    pub latest_stable: AppVersion,
    /// Equal to `latest_stable` when the newest release is a stable release. But will contain
    /// beta versions when those are out for testing.
    pub latest: AppVersion,
}

pub type AppVersion = String;
