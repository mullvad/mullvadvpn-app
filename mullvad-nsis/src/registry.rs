//! Registry operations.
//!
//! Exports:
//! - `MoveRegistryKey` - move (rename) a registry key

use std::ffi::OsString;
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::ptr;

use nsis_plugin_api::{nsis_fn, popstr, pushint, pushstr};
use widestring::U16CString;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
    HKEY_USERS, KEY_READ, KEY_WOW64_64KEY, KEY_WRITE, REG_EXPAND_SZ, REG_OPTION_NON_VOLATILE,
    RegCloseKey, RegCopyTreeW, RegCreateKeyExW, RegDeleteTreeW, RegFlushKey, RegOpenKeyExW,
    RegQueryValueExW, RegSetValueExW,
};

use crate::NsisStatus;

/// Open registry key handle.
pub struct RegKey(HKEY);

impl RegKey {
    /// Open an existing registry key.
    pub fn open(root: HKEY, subkey: &str, access: u32) -> io::Result<Self> {
        let subkey = U16CString::from_str_truncate(subkey);
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
    pub fn create(root: HKEY, subkey: &str, options: u32, access: u32) -> io::Result<Self> {
        let subkey = U16CString::from_str_truncate(subkey);
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
    pub fn copy_tree_to(&self, dest: &RegKey) -> io::Result<()> {
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
    pub fn delete_tree(root: HKEY, subkey: &str) -> io::Result<()> {
        let subkey = U16CString::from_str_truncate(subkey);
        // SAFETY: `root` is a valid HKEY and `subkey` is a null-terminated
        // wide string.
        let result = unsafe { RegDeleteTreeW(root, subkey.as_ptr()) };
        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        Ok(())
    }

    /// Read a `REG_SZ` or `REG_EXPAND_SZ` value as an `OsString`.
    pub fn read_string(&self, name: &str) -> io::Result<OsString> {
        let name = U16CString::from_str_truncate(name);
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
    pub fn write_expand_string(&self, name: &str, value: &OsString) -> io::Result<()> {
        let name = U16CString::from_str_truncate(name);
        let value_wide = U16CString::from_os_str_truncate(value);
        let byte_len = ((value_wide.len() + 1) * 2) as u32;

        // SAFETY: `self.0` is a live HKEY, `name` is a null-terminated wide
        // string, and `value_wide` holds `byte_len` valid bytes (its u16
        // length plus the trailing nul, times 2).
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
    pub fn flush(&self) {
        // SAFETY: `self.0` is a live HKEY owned by `RegKey`.
        unsafe { RegFlushKey(self.0) };
    }
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

// MoveRegistryKey "source" "destination"
//
// Moves a registry key from source to destination.
// Pushes an error message string and a status code.
//
// Example: MoveRegistryKey "HKLM\Software\A" "HKLM\Software\B"
#[nsis_fn]
fn MoveRegistryKey() -> Result<(), nsis_plugin_api::Error> {
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
