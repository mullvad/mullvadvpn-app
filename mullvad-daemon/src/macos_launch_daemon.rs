//! Provides functions to handle or query the status of the Mullvad launch
//! daemon/system service on macOS.
//!
//! If the service exists but needs to be approved by the user, this status
//! must be checked so that the user can be directed to approve the launch
//! daemon in the system settings.

#![allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare.

use std::ffi::CStr;

use objc2::{class, msg_send, runtime::AnyObject};
use objc2_foundation::{NSOperatingSystemVersion, NSProcessInfo};

type Id = *mut AnyObject;

// TODO: Replace with obcj2-service-management
// Framework that contains `SMAppService`.
#[link(name = "ServiceManagement", kind = "framework")]
unsafe extern "C" {}

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
    get_status_for_url(&daemon_plist_url())
}

fn get_status_for_url(url: &Object) -> LaunchDaemonStatus {
    let status: libc::c_long =
        unsafe { msg_send![class!(SMAppService), statusForLegacyURL: url.0] };

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

fn get_os_version() -> NSOperatingSystemVersion {
    let process_info = NSProcessInfo::processInfo();
    process_info.operatingSystemVersion()
}

/// Returns an `NSURL` instance for `DAEMON_PLIST_PATH`.
fn daemon_plist_url() -> Object {
    /// Path to the plist that defines the Mullvad launch daemon.
    /// It must be kept in sync with the path defined in
    /// `dist-assets/pkg-scripts/postinstall`.
    const DAEMON_PLIST_PATH: &CStr = c"/Library/LaunchDaemons/net.mullvad.daemon.plist";

    let nsstr_inst: Id = unsafe { msg_send![class!(NSString), alloc] };
    let nsstr_inst: Id =
        unsafe { msg_send![nsstr_inst, initWithUTF8String: DAEMON_PLIST_PATH.as_ptr()] };

    let nsurl_inst: Id = unsafe { msg_send![class!(NSURL), alloc] };
    let nsurl_inst: Id = unsafe { msg_send![nsurl_inst, initWithString: nsstr_inst] };

    let _: () = unsafe { msg_send![nsstr_inst, release] };

    assert!(!nsurl_inst.is_null());

    Object(nsurl_inst)
}

/// Calls `[self.0 release]` when the wrapped instance is dropped.
struct Object(Id);

impl Drop for Object {
    fn drop(&mut self) {
        let _: () = unsafe { msg_send![self.0, release] };
    }
}
