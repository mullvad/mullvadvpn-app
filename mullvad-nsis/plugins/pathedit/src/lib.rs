//! NSIS pathedit plugin: system PATH editor for the Mullvad VPN installer.
//!
//! Exports:
//! - `AddSysEnvPath` - add a directory to the system PATH
//! - `RemoveSysEnvPath` - remove a directory from the system PATH

#![cfg(all(target_arch = "x86", target_os = "windows"))]

use std::ffi::OsString;
use std::io;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr;

use nsis_plugin_api::{nsis_fn, popstr, pushint, pushstr};

/// NSIS status codes returned to the installer scripts.
#[derive(Clone, Copy)]
#[repr(i32)]
enum NsisStatus {
    GeneralError = 0,
    Success = 1,
}
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, REG_EXPAND_SZ, RegCloseKey, RegFlushKey,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
};
use windows_sys::w;

/// Registry key for the system environment variables.
const PATH_KEY_NAME: &str = "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";

/// Registry value name for the system PATH.
const PATH_VAL_NAME: &str = "Path";

/// "Environment" as wide string (null-terminated). Identifies the setting
/// changed in the WM_SETTINGCHANGE broadcast.
const ENVIRONMENT_W: *const u16 = w!("Environment");

const MESSAGE_TIMEOUT_MS: u32 = 5000;

struct RegKey(HKEY);

impl RegKey {
    /// Read a `REG_SZ` or `REG_EXPAND_SZ` value as an `OsString`.
    fn read_string(&self, name: &str) -> io::Result<OsString> {
        let name = to_wide_nul(name);
        let mut value_type: u32 = 0;
        let mut buf_size: u32 = 0;

        // First call: determine required buffer size.
        // SAFETY: `self.0` is a live HKEY (owned by `RegKey`), `name` is a
        // null-terminated wide string, and the buffer pointer is null so
        // only `buf_size` is written.
        unsafe {
            RegQueryValueExW(
                self.0,
                name.as_ptr(),
                ptr::null(),
                &mut value_type,
                ptr::null_mut(),
                &mut buf_size,
            )
        };

        if buf_size == 0 {
            return Ok(OsString::new());
        }

        let elem_count = (buf_size as usize + 1) / 2 + 1;
        let mut buf = vec![0u16; elem_count];
        let mut actual_size = buf_size;

        // SAFETY: `self.0` is a live HKEY, `name` is a null-terminated wide
        // string, `buf` is valid for `elem_count * 2` writable bytes, and
        // `actual_size` is initialized to the available capacity.
        let result = unsafe {
            RegQueryValueExW(
                self.0,
                name.as_ptr(),
                ptr::null(),
                &mut value_type,
                buf.as_mut_ptr().cast(),
                &mut actual_size,
            )
        };

        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }

        let char_count = actual_size as usize / 2;
        // REG_SZ/REG_EXPAND_SZ values typically include a trailing null
        // terminator in `actual_size`; strip it if present.
        let trimmed = buf[..char_count]
            .strip_suffix(&[0u16])
            .unwrap_or(&buf[..char_count]);
        Ok(OsString::from_wide(trimmed))
    }

    /// Write a `REG_EXPAND_SZ` value.
    fn write_expand_string(&self, name: &str, value: &OsString) -> io::Result<()> {
        let name = to_wide_nul(name);
        let value_wide: Vec<u16> = value.encode_wide().chain(std::iter::once(0)).collect();
        let byte_len = (value_wide.len() * 2) as u32;

        // SAFETY: `self.0` is a live HKEY, `name` is a null-terminated wide
        // string, and `value_wide` holds `byte_len` valid bytes (its u16
        // length times 2).
        let result = unsafe {
            RegSetValueExW(
                self.0,
                name.as_ptr(),
                0,
                REG_EXPAND_SZ,
                value_wide.as_ptr().cast(),
                byte_len,
            )
        };

        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }

        self.flush();
        Ok(())
    }

    /// Flush pending writes to disk.
    fn flush(&self) {
        // SAFETY: `self.0` is a live HKEY owned by `RegKey`.
        unsafe { RegFlushKey(self.0) };
    }
}

impl Drop for RegKey {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a valid HKEY produced by `RegOpenKeyExW` and
        // not yet closed; `RegKey` owns it uniquely.
        unsafe { RegCloseKey(self.0) };
    }
}

/// Encode a UTF-8 string as a null-terminated UTF-16 buffer.
fn to_wide_nul(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

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
            &mut result,
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
    // SAFETY: `exdll_init` was called.
    let path_to_add = unsafe { popstr()? };

    let result = (|| -> io::Result<()> {
        let path_key = to_wide_nul(PATH_KEY_NAME);
        let mut key: HKEY = ptr::null_mut();
        // SAFETY: `path_key` is a null-terminated wide string and `&mut key`
        // points to a stack-local for the API to fill in.
        let result = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                path_key.as_ptr(),
                0,
                KEY_READ | KEY_WRITE,
                &mut key,
            )
        };
        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        let key = RegKey(key);
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
    // SAFETY: `exdll_init` was called.
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
    // SAFETY: `exdll_init` was called.
    let path_to_remove = unsafe { popstr()? };

    let result = (|| -> io::Result<()> {
        let path_key = to_wide_nul(PATH_KEY_NAME);
        let mut key: HKEY = ptr::null_mut();
        // SAFETY: `path_key` is a null-terminated wide string and `&mut key`
        // points to a stack-local for the API to fill in.
        let result = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                path_key.as_ptr(),
                0,
                KEY_READ | KEY_WRITE,
                &mut key,
            )
        };
        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        let key = RegKey(key);
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
    // SAFETY: `exdll_init` was called.
    unsafe {
        pushstr(&message)?;
        pushint(status as i32)
    }
}
