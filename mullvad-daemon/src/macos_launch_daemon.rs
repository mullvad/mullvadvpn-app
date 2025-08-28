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
    let os_version = get_os_version();
    if os_version.majorVersion < 13 {
        return LaunchDaemonStatus::Ok;
    }
    // SAFETY: daemon_plist_path is not an empty path & it is a valid system path.
    // https://developer.apple.com/documentation/foundation/nsurl/fileurl(withpath:)#parameters
    let daemon_plist_url = unsafe { NSURL::fileURLWithPath(ns_string!(DAEMON_PLIST_PATH)) };
    get_status_for_url(&daemon_plist_url)
}

fn get_status_for_url(url: &NSURL) -> LaunchDaemonStatus {
    // SAFETY: Apple does not state *anything* regarding safety requirements of this function:
    // https://developer.apple.com/documentation/servicemanagement/smappservice/statusforlegacyplist(at:)
    // But using a bit of reasoning & the [guidelines of objc2](https://github.com/madsmtm/objc2/blob/master/crates/header-translator/README.md#what-is-required-for-a-method-to-be-safe):
    // """
    // What is required for a method to be safe?
    // 1. The method must not take a raw pointer; one could trivially pass ptr::invalid() and cause UB with that.
    // 2. Any extra requirements that the method states in its documentation must be upheld.
    // """
    // we can conclude that:
    // (1.) is upheld by the virtue of url being a reference, since references are always valid.
    // (2.) is trivially upheld since Apple does not state safety requirements.
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
