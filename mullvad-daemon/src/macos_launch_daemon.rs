//! Provides functions to handle or query the status of the Mullvad launch
//! daemon/system service on macOS.
//!
//! If the service exists but needs to be approved by the user, this status
//! must be checked so that the user can be directed to approve the launch
//! daemon in the system settings.

/// Path to the plist that defines the Mullvad launch daemon.
const DAEMON_PLIST_PATH: &[u8] = b"/Library/LaunchDaemons/net.mullvad.daemon.plist\0";

// Framework that contains `SMAppService`.
#[link(name = "ServiceManagement", kind = "framework")]
extern "C" {}

/// Returned by `[NSProcessInfo operatingSystemVersion]`. Contains the current
#[repr(C)]
#[derive(Debug)]
struct NSOperatingSystemVersion {
    major_version: libc::c_ulong,
    minor_version: libc::c_ulong,
    patch_version: libc::c_ulong,
}

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
    use objc::*;
    type Id = *mut objc::runtime::Object;

    // Check what the current macOS version is.
    // `SMAppService` does not exist if the major version is less than 13.

    let proc_info: Id = unsafe { msg_send![class!(NSProcessInfo), processInfo] };
    let os_version: NSOperatingSystemVersion =
        unsafe { msg_send![proc_info, operatingSystemVersion] };

    if os_version.major_version < 13 {
        return LaunchDaemonStatus::Ok;
    }

    // Call [sm_app_service statusForLegacyURL: daemon_plist]

    let nsstr_inst: Id = unsafe { msg_send![class!(NSString), alloc] };
    let nsstr_inst: Id = unsafe { msg_send![nsstr_inst, initWithUTF8String: DAEMON_PLIST_PATH] };

    let nsurl_inst: Id = unsafe { msg_send![class!(NSURL), alloc] };
    let nsurl_inst: Id = unsafe { msg_send![nsurl_inst, initWithString: nsstr_inst] };

    let _: libc::c_void = unsafe { msg_send![nsstr_inst, release] };

    let status: libc::c_long =
        unsafe { msg_send![class!(SMAppService), statusForLegacyURL: nsurl_inst] };

    let _: libc::c_void = unsafe { msg_send![nsurl_inst, release] };

    match status {
        // SMAppServiceStatusNotRegistered | SMAppServiceStatusNotFound
        0 | 3 => LaunchDaemonStatus::NotFound,
        // SMAppServiceStatusEnabled
        1 => LaunchDaemonStatus::Ok,
        // SMAppServiceStatusRequiresApproval
        2 => LaunchDaemonStatus::NotAuthorized,
        // Unknown status
        _ => LaunchDaemonStatus::Unknown,
    }
}
