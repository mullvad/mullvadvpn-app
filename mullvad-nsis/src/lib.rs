#![cfg(all(target_arch = "x86", target_os = "windows"))]

use std::{
    ffi::OsString,
    iter,
    os::windows::ffi::{OsStrExt, OsStringExt},
    panic::UnwindSafe,
    path::Path,
    ptr,
};

mod handle;

#[repr(C)]
pub enum Status {
    Ok,
    InvalidArguments,
    InsufficientBufferSize,
    OsError,
    Panic,
}

/// Max path size allowed
const MAX_PATH_SIZE: usize = 32_767;

/// Creates a privileged directory at the specified Windows path.
///
/// # SAFETY
/// path needs to be a windows path encoded as a string of u16 that terminates in 0 (two
/// nul-bytes). The string is also not allowed to be greater than `MAX_PATH_SIZE`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn create_privileged_directory(path: *const u16) -> Status {
    catch_and_log_unwind(|| {
        // SAFETY: `path` is a null-terminated UTF-16 string
        let path = unsafe { osstr_from_wide(path) };
        let path = Path::new(&path);

        match mullvad_paths::windows::create_privileged_directory(path) {
            Ok(()) => Status::Ok,
            Err(_) => Status::OsError,
        }
    })
}

/// Writes the system's app data path into `buffer` when `Status::Ok` is returned.
/// If `buffer` is `null`, or if the buffer is too small, `InsufficientBufferSize`
/// is returned, and the required buffer size (in chars) is returned in `buffer_size`.
/// On success, `buffer_size` is set to the length of the string, including
/// the final null terminator.
///
/// # SAFETY
/// if `buffer` is not null, it must point to a valid memory location that can hold
/// at least `buffer_size` number of `u16` values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_system_local_appdata(
    buffer: *mut u16,
    buffer_size: *mut usize,
) -> Status {
    catch_and_log_unwind(|| {
        if buffer_size.is_null() {
            return Status::InvalidArguments;
        }

        let path = match mullvad_paths::windows::get_system_service_appdata() {
            Ok(path) => path,
            Err(_error) => {
                return Status::OsError;
            }
        };

        let path = path.as_os_str();
        let path_u16: Vec<u16> = path.encode_wide().chain(std::iter::once(0u16)).collect();

        // SAFETY: `buffer_size` is non-null because we checked it above.
        unsafe {
            let prev_len = *buffer_size;
            *buffer_size = path_u16.len();

            if prev_len < path_u16.len() || buffer.is_null() {
                return Status::InsufficientBufferSize;
            }
        }

        // SAFETY: We assume that `buffer` is a valid pointer to a u16 array
        // and because of the previous check, we know that `buffer` has enough space
        // to hold the contents of `path_u16`.
        unsafe { ptr::copy_nonoverlapping(path_u16.as_ptr(), buffer, path_u16.len()) };

        Status::Ok
    })
}

/// Writes the system's version data into `buffer` when `Status::Ok` is
/// returned. If `buffer` is `null`, or if the buffer is too small,
/// `InsufficientBufferSize` is returned, and the required buffer size (in
/// chars) is returned in `buffer_size`. On success, `buffer_size` is set to the
/// length of the string, including the final null terminator.
///
/// # Safety
/// If `buffer` is not null, it must point to a valid memory location that can hold
/// at least `*buffer_size` number of `u16` values. `buffer_size` must be a valid pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_system_version(buffer: *mut u16, buffer_size: *mut usize) -> Status {
    use talpid_platform_metadata::version;
    catch_and_log_unwind(|| {
        if buffer_size.is_null() {
            return Status::InvalidArguments;
        }

        let build_number_string = OsString::from(version());
        let build_number: Vec<u16> = build_number_string
            .encode_wide()
            .chain(iter::once(0u16))
            .collect();

        // SAFETY: `buffer_size` is non-null because we checked it above.
        unsafe {
            if *buffer_size < build_number.len() || buffer.is_null() {
                return Status::InsufficientBufferSize;
            }

            *buffer_size = build_number.len();
        }

        // SAFETY: We assume that `buffer` is a valid pointer to a u16 array
        // and because of the previous check, we know that `buffer` has enough space
        // to hold the contents of `build_number`.
        unsafe { ptr::copy_nonoverlapping(build_number.as_ptr(), buffer, build_number.len()) };
        Status::Ok
    })
}

/// Windows version details
#[repr(C)]
pub struct WindowsVer {
    major_version: u32,
    minor_version: u32,
    build_number: u32,
}

/// Write OS version into `version_out` when `Status::Ok` is returned.
///
/// # Safety
/// `version_out` must point to a valid `WindowsVer`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_system_version_struct(version_out: *mut WindowsVer) -> Status {
    use talpid_platform_metadata::WindowsVersion;
    catch_and_log_unwind(|| {
        // Try to retrieve the version based on the kernel image
        let winver = WindowsVersion::from_ntoskrnl().unwrap();
        let c_ver = WindowsVer {
            major_version: winver.major_version(),
            minor_version: winver.minor_version(),
            build_number: winver.build_number(),
        };
        // SAFETY: `version_out` is a valid `WindowsVer` if the caller upholds the contract.
        unsafe { ptr::write(version_out, c_ver) };
        Status::Ok
    })
}

/// Identify processes that may be using files in the install path, and ask the user to close them.
///
/// # Safety
///
/// * `install_path` must be a null-terminated wide string (UTF-16).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn find_in_use_processes(install_path: *const u16) -> Status {
    catch_and_log_unwind(|| {
        // SAFETY: `install_path` is a null-terminated wide string.
        let path = unsafe { osstr_from_wide(install_path) };
        if handle::ask_terminate_processes(&path).is_err() {
            return Status::OsError;
        }
        Status::Ok
    })
}

fn catch_and_log_unwind(func: impl FnOnce() -> Status + UnwindSafe) -> Status {
    match std::panic::catch_unwind(func) {
        Ok(status) => status,
        Err(_) => Status::Panic,
    }
}

/// Convert a null-terminated wide string (UTF-16) to an `OsString`.
///
/// # Safety
///
/// * `wide_str` must be a null-terminated.
unsafe fn osstr_from_wide(wide_str: *const u16) -> OsString {
    let mut len = 0;
    // SAFETY: `wide_str` is a valid pointer to a null-terminated wide string.
    while unsafe { *wide_str.offset(len) } != 0 {
        len += 1;
    }
    // SAFETY: Because we checked the length, we can safely create a slice
    // from the raw pointer.
    let s = unsafe { std::slice::from_raw_parts(wide_str, len as usize) };
    OsString::from_wide(s)
}
