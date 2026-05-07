use std::io;

use windows_registry::Key;
use windows_registry::LOCAL_MACHINE;
use windows_sys::Win32::Foundation::{ERROR_SUCCESS, SetLastError};
use windows_sys::Win32::System::Registry::RegFlushKey;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
};
use windows_sys::w;

/// Registry key for the system environment variables.
const PATH_KEY_NAME: &str = "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";

/// Registry value name for the system PATH.
const PATH_VAL_NAME: &str = "Path";

/// Add path to the system PATH variable, unless it exists.
pub fn add_path_to_sys_path(path_to_add: &str) -> io::Result<()> {
    let key = LOCAL_MACHINE.options().read().write().open(PATH_KEY_NAME)?;
    let current_path = key.get_string(PATH_VAL_NAME)?;

    if sys_path_contains(&current_path, path_to_add) {
        return Ok(());
    }

    let new_path = if current_path.is_empty() {
        path_to_add.to_owned()
    } else {
        format!("{current_path};{path_to_add}")
    };

    key.set_expand_string(PATH_VAL_NAME, &new_path)?;
    flush_key(&key);
    drop(key);
    broadcast_setting_change()?;
    Ok(())
}

/// Remove path from system PATH variable, if set.
pub fn remove_path_from_sys_path(path_to_remove: &str) -> io::Result<()> {
    let key = LOCAL_MACHINE.options().read().write().open(PATH_KEY_NAME)?;
    let current_path = key.get_string(PATH_VAL_NAME)?;

    let (new_path, changed) = remove_from_path(&current_path, path_to_remove);

    if changed {
        key.set_expand_string(PATH_VAL_NAME, &new_path)?;
        flush_key(&key);
        drop(key);
        broadcast_setting_change()?;
    }

    Ok(())
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

/// Broadcast WM_SETTINGCHANGE to notify all windows of the PATH change.
fn broadcast_setting_change() -> io::Result<()> {
    /// "Environment" as wide string (null-terminated). Identifies the setting
    /// changed in the WM_SETTINGCHANGE broadcast.
    const ENVIRONMENT_W: *const u16 = w!("Environment");

    const MESSAGE_TIMEOUT_MS: u32 = 5000;

    let mut result: usize = 0;

    // `SendMessageTimeoutW` does not always set the last OS error on failure.
    // SAFETY: Trivially safe.
    unsafe { SetLastError(ERROR_SUCCESS) };

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
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(ERROR_SUCCESS as i32) {
            return Err(io::Error::other(
                "SendMessageTimeoutW failed without setting an error",
            ));
        }
        return Err(err);
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

/// Flush pending writes for `key` to disk. The lazy flusher would do this
/// eventually, but the installer wants the system PATH change persisted before
/// broadcasting the change.
// TODO: This can probably be removed if broadcast is enough
fn flush_key(key: &Key) {
    // SAFETY: `key.as_raw()` returns the live HKEY owned by `key`.
    unsafe { RegFlushKey(key.as_raw()) };
}
