//! Tray icon management for the Mullvad VPN installer.

use crate::NsisStatus;

use nsis_plugin_api::{nsis_fn, pushint, pushstr};

mod sys;

// PromoteTrayIcon
//
// Ensure the Mullvad VPN tray icon is in the visible notification area.
// Pushes error message and status code.
#[nsis_fn]
fn PromoteTrayIcon() -> Result<(), nsis_plugin_api::Error> {
    let (message, status) = match sys::promote_tray_icon() {
        Ok(()) => (String::new(), NsisStatus::Success),
        Err(e) => (e.to_string(), NsisStatus::GeneralError),
    };
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    unsafe {
        pushstr(&message)?;
        pushint(status as i32)
    }
}
