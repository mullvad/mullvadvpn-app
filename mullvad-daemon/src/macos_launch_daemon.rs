//! Provides functions to handle or query the status of the Mullvad launch
//! daemon/system service on macOS.
//!
//! If the service exists but needs to be approved by the user, this status
//! must be checked so that the user can be directed to approve the launch
//! daemon in the system settings.

use objc::{class, msg_send, sel, sel_impl};
use std::ffi::CStr;

type Id = *mut objc::runtime::Object;

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
    // `SMAppService` does not exist if the major version is less than 13.
    if get_os_version().major_version < 13 {
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
    // the object is lazily instantiated, so we don't release it
    let proc_info: Id = unsafe { msg_send![class!(NSProcessInfo), processInfo] };
    unsafe { msg_send![proc_info, operatingSystemVersion] }
}

/// Returns an `NSURL` instance for `DAEMON_PLIST_PATH`.
fn daemon_plist_url() -> Object {
    /// Path to the plist that defines the Mullvad launch daemon.
    /// It must be kept in sync with the path defined in
    /// `dist-assets/pkg-scripts/postinstall`.
    const DAEMON_PLIST_PATH: &CStr = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"/Library/LaunchDaemons/net.mullvad.daemon.plist\0")
    };

    let nsstr_inst: Id = unsafe { msg_send![class!(NSString), alloc] };
    let nsstr_inst: Id =
        unsafe { msg_send![nsstr_inst, initWithUTF8String: DAEMON_PLIST_PATH.as_ptr()] };

    let nsurl_inst: Id = unsafe { msg_send![class!(NSURL), alloc] };
    let nsurl_inst: Id = unsafe { msg_send![nsurl_inst, initWithString: nsstr_inst] };

    let _: libc::c_void = unsafe { msg_send![nsstr_inst, release] };

    assert!(!nsurl_inst.is_null());

    Object(nsurl_inst)
}

/// Calls `[self.0 release]` when the wrapped instance is dropped.
struct Object(Id);

impl Drop for Object {
    fn drop(&mut self) {
        let _: libc::c_void = unsafe { msg_send![self.0, release] };
    }
}
