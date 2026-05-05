//! Registry operations.

use std::ffi::OsString;
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::ptr;

use widestring::U16CString;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    HKEY, REG_EXPAND_SZ, RegCloseKey, RegFlushKey, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW,
};

/// Open registry key handle.
pub struct RegKey(HKEY);

impl RegKey {
    /// Open an existing registry key.
    pub fn open(root: HKEY, subkey: &str, access: u32) -> io::Result<Self> {
        let subkey = U16CString::from_str_truncate(subkey);
        let mut handle: HKEY = ptr::null_mut();
        // SAFETY: `root` is a valid HKEY, `subkey` is a null-terminated wide
        // string, and `&mut handle` is a stack-local.
        let result = unsafe { RegOpenKeyExW(root, subkey.as_ptr(), 0, access, &raw mut handle) };
        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        Ok(Self(handle))
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
                &raw mut value_type,
                ptr::null_mut(),
                &raw mut buf_size,
            )
        };

        if buf_size == 0 {
            return Ok(OsString::new());
        }

        let elem_count = (buf_size as usize).div_ceil(2) + 1;
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
                &raw mut value_type,
                buf.as_mut_ptr().cast(),
                &raw mut actual_size,
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
        // SAFETY: `self.0` is a valid HKEY produced by `RegOpenKeyExW` and
        // not yet closed; `RegKey` owns it uniquely.
        unsafe { RegCloseKey(self.0) };
    }
}
