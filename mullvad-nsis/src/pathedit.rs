//! System PATH editor.
//!
//! Exports:
//! - `AddSysEnvPath` - add a directory to the system PATH
//! - `RemoveSysEnvPath` - remove a directory from the system PATH

use std::ffi::OsString;
use std::io;

use nsis_plugin_api::{nsis_fn, popstr, pushint, pushstr};
use windows_sys::Win32::System::Registry::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
};
use windows_sys::w;

use crate::NsisStatus;
use crate::registry::RegKey;

/// Registry key for the system environment variables.
const PATH_KEY_NAME: &str = "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";

/// Registry value name for the system PATH.
const PATH_VAL_NAME: &str = "Path";

/// "Environment" as wide string (null-terminated). Identifies the setting
/// changed in the WM_SETTINGCHANGE broadcast.
const ENVIRONMENT_W: *const u16 = w!("Environment");

const MESSAGE_TIMEOUT_MS: u32 = 5000;

/// Broadcast WM_SETTINGCHANGE to notify all windows of the PATH change.
fn broadcast_setting_change() -> io::Result<()> {
    let mut result: usize = 0;

    // SAFETY: `ENVIRONMENT_W` is a static null-terminated wide string from
    // `w!()`. `&mut result` points to a stack-local for the API to fill in.
    let status = unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            ENVIRONMENT_W as isize,
            SMTO_ABORTIFHUNG,
            MESSAGE_TIMEOUT_MS,
            &raw mut result,
        )
    };
    if status == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

/// Case-insensitive check if `path_to_find` exists in the semicolon-separated `all_paths`.
fn sys_path_contains(all_paths: &str, path_to_find: &str) -> bool {
    let target = path_to_find.to_lowercase();
    all_paths
        .split(';')
        .any(|p| p.to_lowercase() == target.as_str())
}

/// Remove `path_to_remove` from the semicolon-separated path string.
/// Returns (new_path, did_remove).
fn remove_from_path(all_paths: &str, path_to_remove: &str) -> (String, bool) {
    let target = path_to_remove.to_lowercase();
    let tokens: Vec<&str> = all_paths.split(';').collect();
    let filtered: Vec<&str> = tokens
        .iter()
        .filter(|&&p| p.to_lowercase() != target)
        .copied()
        .collect();

    let changed = filtered.len() != tokens.len();
    (filtered.join(";"), changed)
}

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

    let result = (|| -> io::Result<()> {
        let key = RegKey::open(HKEY_LOCAL_MACHINE, PATH_KEY_NAME, KEY_READ | KEY_WRITE)?;
        let current_path = key.read_string(PATH_VAL_NAME)?;
        let current_path_str = current_path
            .to_str()
            .expect("system PATH should be valid UTF-8");

        if sys_path_contains(current_path_str, &path_to_add) {
            return Ok(());
        }

        let new_path = if current_path_str.is_empty() {
            OsString::from(&path_to_add)
        } else {
            let mut p = current_path_str.to_owned();
            p.push(';');
            p.push_str(&path_to_add);
            OsString::from(p)
        };

        key.write_expand_string(PATH_VAL_NAME, &new_path)?;
        drop(key);
        broadcast_setting_change()?;
        Ok(())
    })();

    let (message, status) = match result {
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

    let result = (|| -> io::Result<()> {
        let key = RegKey::open(HKEY_LOCAL_MACHINE, PATH_KEY_NAME, KEY_READ | KEY_WRITE)?;
        let current_path = key.read_string(PATH_VAL_NAME)?;
        let current_path_str = current_path
            .to_str()
            .expect("system PATH should be valid UTF-8");

        let (new_path, changed) = remove_from_path(current_path_str, &path_to_remove);

        if changed {
            key.write_expand_string(PATH_VAL_NAME, &OsString::from(new_path))?;
            drop(key);
            broadcast_setting_change()?;
        }

        Ok(())
    })();

    let (message, status) = match result {
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
