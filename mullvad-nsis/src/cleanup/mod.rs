//! Cleanup operations for the Mullvad VPN installer.

use nsis_plugin_api::{nsis_fn, popint, popstr, pushint, pushstr};

use crate::NsisStatus;

mod sys;
use sys::*;

// RemoveLogsAndCache
//
// Removes all logs and cache for current user, other users, and the service user.
// Pushes a status code.
#[nsis_fn]
fn RemoveLogsAndCache() -> Result<(), nsis_plugin_api::Error> {
    let result = [
        remove_logs_cache_current_user(),
        remove_logs_cache_other_users(),
        remove_cache_service_user(),
        remove_logs_service_user(),
    ]
    .into_iter()
    .collect::<anyhow::Result<()>>();

    let status = if result.is_ok() {
        NsisStatus::Success
    } else {
        NsisStatus::GeneralError
    };
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    unsafe { pushint(status as i32) }
}

// RemoveSettings
//
// Removes the service user's settings directory.
// Pushes a status code.
#[nsis_fn]
fn RemoveSettings() -> Result<(), nsis_plugin_api::Error> {
    let status = match remove_settings_service_user() {
        Ok(()) => NsisStatus::Success,
        Err(_) => NsisStatus::GeneralError,
    };
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    unsafe { pushint(status as i32) }
}

// RemoveRelayCache
//
// Removes the relay cache file.
// Pushes error message and status code.
#[nsis_fn]
fn RemoveRelayCache() -> Result<(), nsis_plugin_api::Error> {
    let (message, status) = match remove_relay_cache_service_user() {
        Ok(()) => (String::new(), NsisStatus::Success),
        Err(e) => (format!("{e:#}"), NsisStatus::GeneralError),
    };
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    unsafe {
        pushstr(&message)?;
        pushint(status as i32)
    }
}

// RemoveApiAddressCache
//
// Removes the API address cache file.
// Pushes error message and status code.
#[nsis_fn]
fn RemoveApiAddressCache() -> Result<(), nsis_plugin_api::Error> {
    let (message, status) = match remove_api_address_cache_service_user() {
        Ok(()) => (String::new(), NsisStatus::Success),
        Err(e) => (format!("{e:#}"), NsisStatus::GeneralError),
    };
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    unsafe {
        pushstr(&message)?;
        pushint(status as i32)
    }
}

// CloseHoggingProcesses "installPath" allowCancellation
//
// Identifies and closes processes blocking files in the install path.
// allowCancellation: 1 = show Yes/No dialog, 0 = show OK-only dialog.
// Pushes error message and status code.
#[nsis_fn]
fn CloseHoggingProcesses() -> Result<(), nsis_plugin_api::Error> {
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    let (install_path, allow_cancellation) = unsafe { (popstr()?, popint()? != 0) };

    let (message, status) =
        match crate::handle::terminate_processes(&install_path, allow_cancellation) {
            Ok(true) => (String::new(), NsisStatus::Success),
            Ok(false) => (String::from("Cancelled"), NsisStatus::Cancelled),
            Err(e) => (format!("{e:#}"), NsisStatus::GeneralError),
        };
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    unsafe {
        pushstr(&message)?;
        pushint(status as i32)
    }
}

// IsEmptyDir "path"
//
// Checks if the directory contains no files (only directories/symlinks).
// Pushes SUCCESS if empty, FILE_EXISTS if files found, GENERAL_ERROR on error.
#[nsis_fn]
fn IsEmptyDir() -> Result<(), nsis_plugin_api::Error> {
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    let path = unsafe { popstr()? };

    let status = match crate::handle::is_empty_dir(&path) {
        Ok(true) => NsisStatus::Success,
        Ok(false) => NsisStatus::FileExists,
        Err(_) => NsisStatus::GeneralError,
    };
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    unsafe { pushint(status as i32) }
}
