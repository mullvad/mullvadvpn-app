//! System PATH editor.

use crate::NsisStatus;
use nsis_plugin_api::{nsis_fn, popstr, pushint, pushstr};

mod sys;
use sys::*;

// AddSysEnvPath "path"
//
// Adds "path" to the system PATH environment variable.
// Does nothing if it already exists.
// Pushes error message and status code.
#[nsis_fn]
fn AddSysEnvPath() -> Result<(), nsis_plugin_api::Error> {
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    let path_to_add = unsafe { popstr()? };

    let (message, status) = match add_path_to_sys_path(&path_to_add) {
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

// RemoveSysEnvPath "path"
//
// Removes "path" from the system PATH environment variable.
// Does nothing if it doesn't exist.
// Pushes error message and status code.
#[nsis_fn]
fn RemoveSysEnvPath() -> Result<(), nsis_plugin_api::Error> {
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    let path_to_remove = unsafe { popstr()? };

    let (message, status) = match remove_path_from_sys_path(&path_to_remove) {
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
