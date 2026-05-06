//! NSIS plugin DLL (`mullvad_nsis.dll`) for the Mullvad VPN installer.
//!
//! The plugin functions are grouped by module by what they touch - registry,
//! environment PATH, on-disk cleanup, and installer logging. NSIS sees them as
//! flat exports of `mullvad_nsis.dll` (`mullvad_nsis::FunctionName` from NSIS
//! script).

#![cfg(all(target_arch = "x86", target_os = "windows"))]

use std::io;
use std::path::PathBuf;
use std::ptr;

use widestring::U16CStr;
use windows_sys::Win32::Foundation::S_OK;
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::UI::Shell::{KF_FLAG_DEFAULT, SHGetKnownFolderPath};

mod cleanup;
mod handle;
mod logger;
mod pathedit;
mod tray;

/// NSIS status codes returned to the installer scripts.
#[derive(Clone, Copy)]
#[repr(i32)]
pub(crate) enum NsisStatus {
    GeneralError = 0,
    Success = 1,
    FileExists = 2,
    Cancelled = 3,
}

/// Get the path for a known Windows folder.
///
/// `folder_id` must point to a valid KNOWNFOLDERID, or this will return an error.
fn get_known_folder_path(folder_id: &windows_sys::core::GUID) -> io::Result<PathBuf> {
    let mut path_ptr: windows_sys::core::PWSTR = ptr::null_mut();
    // SAFETY: `folder_id` is a valid GUID;
    // null token uses the calling thread's identity; `&mut path_ptr` is a
    // stack-local for the API to fill in with a CoTaskMem-allocated PWSTR.
    let status = unsafe {
        SHGetKnownFolderPath(
            folder_id,
            KF_FLAG_DEFAULT as u32,
            ptr::null_mut(),
            &raw mut path_ptr,
        )
    };

    let result = if status == S_OK {
        // SAFETY: on success `path_ptr` points to a null-terminated wide
        // string allocated by `SHGetKnownFolderPath`, valid until the
        // `CoTaskMemFree` below.
        let wide = unsafe { U16CStr::from_ptr_str(path_ptr) };
        Ok(PathBuf::from(wide.to_os_string()))
    } else {
        Err(io::Error::from_raw_os_error(status))
    };

    // `SHGetKnownFolderPath` requires the caller to free the buffer even on
    // failure (the API may allocate before returning an error code). A null
    // `path_ptr` is a documented no-op for `CoTaskMemFree`.
    // SAFETY: `path_ptr` was either allocated by `SHGetKnownFolderPath`
    // (CoTaskMem) or is null; either way `CoTaskMemFree` is sound.
    unsafe { CoTaskMemFree(path_ptr.cast()) };
    result
}
