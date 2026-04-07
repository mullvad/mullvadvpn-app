//! NSIS registry plugin: registry operations for the Mullvad VPN installer.
//!
//! Exports:
//! - `MoveKey` - move (rename) a registry key

#![cfg(all(target_arch = "x86", target_os = "windows"))]

use std::io;
use std::ptr;

use nsis_plugin_api::{nsis_fn, popstr, pushint, pushstr};
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
    HKEY_USERS, KEY_READ, KEY_WOW64_64KEY, KEY_WRITE, REG_OPTION_NON_VOLATILE, RegCloseKey,
    RegCopyTreeW, RegCreateKeyExW, RegDeleteTreeW, RegOpenKeyExW,
};

/// NSIS status codes returned to the installer scripts.
#[derive(Clone, Copy)]
#[repr(i32)]
enum NsisStatus {
    GeneralError = 0,
    Success = 1,
}

/// Open registry key handle.
struct RegKey(HKEY);

impl RegKey {
    /// Open an existing registry key.
    fn open(root: HKEY, subkey: &str, access: u32) -> io::Result<Self> {
        let subkey = to_wide_nul(subkey);
        let mut handle: HKEY = ptr::null_mut();
        // SAFETY: `root` is a valid HKEY, `subkey` is a null-terminated wide
        // string, and `&mut handle` is a stack-local.
        let result = unsafe { RegOpenKeyExW(root, subkey.as_ptr(), 0, access, &mut handle) };
        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        Ok(Self(handle))
    }

    /// Create a registry key (or open it if it already exists).
    fn create(root: HKEY, subkey: &str, options: u32, access: u32) -> io::Result<Self> {
        let subkey = to_wide_nul(subkey);
        let mut handle: HKEY = ptr::null_mut();
        let mut disposition: u32 = 0;
        // SAFETY: `root` is a valid HKEY, `subkey` is a null-terminated wide
        // string, the class/security pointers are null (allowed), and the
        // out-parameter pointers are stack-locals.
        let result = unsafe {
            RegCreateKeyExW(
                root,
                subkey.as_ptr(),
                0,
                ptr::null_mut(),
                options,
                access,
                ptr::null(),
                &mut handle,
                &mut disposition,
            )
        };
        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        Ok(Self(handle))
    }

    /// Copy this key's entire subtree onto `dest`.
    fn copy_tree_to(&self, dest: &RegKey) -> io::Result<()> {
        // SAFETY: both handles are live HKEYs owned by `self` and `dest`;
        // the optional source-subkey pointer is null (copy `self` itself).
        let result = unsafe { RegCopyTreeW(self.0, ptr::null(), dest.0) };
        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        Ok(())
    }

    /// Recursively delete a subkey and its descendants.
    ///
    /// This does not require an open handle to the subkey: the API resolves
    /// it relative to `root`.
    fn delete_tree(root: HKEY, subkey: &str) -> io::Result<()> {
        let subkey = to_wide_nul(subkey);
        // SAFETY: `root` is a valid HKEY and `subkey` is a null-terminated
        // wide string.
        let result = unsafe { RegDeleteTreeW(root, subkey.as_ptr()) };
        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        Ok(())
    }
}

/// Encode a UTF-8 string as a null-terminated UTF-16 buffer.
fn to_wide_nul(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

impl Drop for RegKey {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a valid HKEY produced by `RegOpenKeyExW` /
        // `RegCreateKeyExW` and not yet closed; `RegKey` owns it uniquely.
        unsafe { RegCloseKey(self.0) };
    }
}

/// Parse a registry path string like "HKLM\Software\A" into (root HKEY, subkey).
fn parse_registry_path(path: &str) -> Option<(HKEY, &str)> {
    let (root_str, subkey) = path.split_once('\\')?;
    let root = match root_str {
        "HKLM" | "HKEY_LOCAL_MACHINE" => HKEY_LOCAL_MACHINE,
        "HKCU" | "HKEY_CURRENT_USER" => HKEY_CURRENT_USER,
        "HKCR" | "HKEY_CLASSES_ROOT" => HKEY_CLASSES_ROOT,
        "HKU" | "HKEY_USERS" => HKEY_USERS,
        "HKCC" | "HKEY_CURRENT_CONFIG" => HKEY_CURRENT_CONFIG,
        _ => return None,
    };
    Some((root, subkey))
}

/// Move a registry key from source to destination, in the 64-bit registry view.
fn move_key(source: &str, destination: &str) -> io::Result<()> {
    let (src_root, src_subkey) = parse_registry_path(source)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid source path"))?;
    let (dst_root, dst_subkey) = parse_registry_path(destination)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid destination path"))?;

    let src = RegKey::open(src_root, src_subkey, KEY_READ | KEY_WOW64_64KEY)?;
    let dst = RegKey::create(
        dst_root,
        dst_subkey,
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE | KEY_WOW64_64KEY,
    )?;
    src.copy_tree_to(&dst)?;

    // Close the handles before deleting the source tree.
    drop(src);
    drop(dst);

    RegKey::delete_tree(src_root, src_subkey)
}

// MoveKey "source" "destination"
//
// Moves a registry key from source to destination.
// Pushes an error message string and a status code.
//
// Example: MoveKey "HKLM\Software\A" "HKLM\Software\B"
#[nsis_fn]
fn MoveKey() -> Result<(), nsis_plugin_api::Error> {
    // SAFETY: `exdll_init` was called.
    let (source, destination) = unsafe { (popstr()?, popstr()?) };

    let (message, status) = match move_key(&source, &destination) {
        Ok(()) => (String::new(), NsisStatus::Success),
        Err(e) => (e.to_string(), NsisStatus::GeneralError),
    };
    // SAFETY: `exdll_init` was called.
    unsafe {
        pushstr(&message)?;
        pushint(status as i32)
    }
}
