//! Provides functions to handle or query the status of the Mullvad launch
//! daemon/system service on macOS.
//!
//! If the service exists but needs to be approved by the user, this status
//! must be checked so that the user can be directed to approve the launch
//! daemon in the system settings.

use objc2_foundation::{NSOperatingSystemVersion, NSProcessInfo, NSURL, ns_string};
use objc2_service_management::{SMAppService, SMAppServiceStatus};

/// Path to the plist that defines the Mullvad launch daemon.
/// It must be kept in sync with the path defined in
/// `dist-assets/pkg-scripts/postinstall`.
const DAEMON_PLIST_PATH: &str = "/Library/LaunchDaemons/net.mullvad.daemon.plist";

/// Authorization status of the Mullvad daemon.
#[repr(i32)]
pub enum LaunchDaemonStatus {
    Ok = 0,
    NotFound = 1,
    NotAuthorized = 2,
    Unknown = 3,
}

/// Return whether the daemon is running, not found, or is not authorized.
/// NOTE: On macos < 13, this function always returns `LaunchDaemonStatus::Ok`.
pub fn get_status() -> LaunchDaemonStatus {
    // `SMAppService` does not exist if the major version is less than 13.
    // TODO: Could as well use processInfo.isOperatingSystemAtLeast
    let os_version = get_os_version();
    if os_version.majorVersion < 13 {
        return LaunchDaemonStatus::Ok;
    }

    // SAFETY: daemon_plist_path has is a well-formed url according to RFC 3986 & is not null.
    let daemon_plist_url = unsafe {
        NSURL::URLWithString_encodingInvalidCharacters(ns_string!(DAEMON_PLIST_PATH), false)
    };

    match daemon_plist_url {
        Some(url) => get_status_for_url(&url),
        // TODO: Technically, this is an error in allocating an URL, not checking the
        // launch daemon status ..
        None => LaunchDaemonStatus::Unknown,
    }
}

fn get_status_for_url(url: &NSURL) -> LaunchDaemonStatus {
    // SAFETY: url points to a valid instance of an NSURL.
    let status = unsafe { SMAppService::statusForLegacyURL(url) };
    match status {
        SMAppServiceStatus::NotRegistered | SMAppServiceStatus::NotFound => {
            LaunchDaemonStatus::NotFound
        }
        SMAppServiceStatus::Enabled => LaunchDaemonStatus::Ok,
        SMAppServiceStatus::RequiresApproval => LaunchDaemonStatus::NotAuthorized,
        // Unknown status
        _ => LaunchDaemonStatus::Unknown,
    }
}

fn get_os_version() -> NSOperatingSystemVersion {
    let process_info = NSProcessInfo::processInfo();
    process_info.operatingSystemVersion()
}
