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
    pub supported: bool,
    /// Latest version
    pub latest: AppVersion,
    /// Latest stable version
    pub latest_stable: AppVersion,
    /// Equal to `latest_stable` when the newest release is a stable release. But will contain
    /// beta versions when those are out for testing.
    pub latest_beta: AppVersion,
}

pub type AppVersion = String;
