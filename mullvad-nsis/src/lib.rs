#![cfg(all(target_arch = "x86", target_os = "windows"))]

use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
    panic::UnwindSafe,
    path::Path,
    ptr,
};

#[repr(C)]
pub enum Status {
    Ok,
    InvalidArguments,
    InsufficientBufferSize,
    OsError,
    Panic,
}

/// Max path size allowed
const MAX_PATH_SIZE: isize = 32_767;

/// SAFETY: path needs to be a windows path encoded as a string of u16 that terminates in 0 (two nul-bytes).
/// The string is also not allowed to be greater than `MAX_PATH_SIZE`.
#[no_mangle]
pub unsafe extern "C" fn create_privileged_directory(path: *const u16) -> Status {
    catch_and_log_unwind(|| {
        let mut i = 0;
        // Calculate the length of the path by checking when the first u16 == 0
        let len = loop {
            if *(path.offset(i)) == 0 {
                break i;
            } else if i > MAX_PATH_SIZE {
                return Status::InvalidArguments;
            }
            i += 1;
        };
        let path = std::slice::from_raw_parts(path, len as usize);
        let path = OsString::from_wide(path);
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
#[no_mangle]
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

        let prev_len = *buffer_size;

        *buffer_size = path_u16.len();

        if prev_len < path_u16.len() || buffer.is_null() {
            return Status::InsufficientBufferSize;
        }

        ptr::copy_nonoverlapping(path_u16.as_ptr(), buffer, path_u16.len());

        Status::Ok
    })
}

fn catch_and_log_unwind(func: impl FnOnce() -> Status + UnwindSafe) -> Status {
    match std::panic::catch_unwind(func) {
        Ok(status) => status,
        Err(_) => Status::Panic,
    }
}
