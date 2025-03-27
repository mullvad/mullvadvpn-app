// TODO:
/*
If a new upgrade version becomes available during an app upgrade it should not affect the suggested upgrade version in the SuggestedUpgrade message if the upgrade is still in progress. That is, if the current state is one of
AppUpgradeDownloadStarting
AppUpgradeDownloadProgress
AppUpgradeVerifyingInstaller
*/

// TODO: How should interruptions be handled?
// Probably magically resume when going from Idle/DownloadFailed -> Downloading

// TODO: handle abort
// TODO: handle beginning/resuming download

use std::path::PathBuf;
use std::time::Duration;

/// App updater state
pub enum UpdateState {
    Idle,
    /// An installer is being downloaded
    Downloading {
        /// A fraction in `[0,1]` that describes how much of the installer has been downloaded
        complete_frac: f32,
        /// Estimated time left
        time_left: Duration,
    },
    /// Failed to download installer
    DownloadFailed,
    /// The downloaded installer is being verified
    Verifying,
    /// VerificationFailed
    VerificationFailed,
    /// There is a downloaded and verified installer available
    Verified {
        verified_installer_path: PathBuf,
    },
}
