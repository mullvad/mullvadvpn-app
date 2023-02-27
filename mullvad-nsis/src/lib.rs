#![cfg(all(target_arch = "x86", target_os = "windows"))]

use std::{os::windows::ffi::OsStrExt, panic::UnwindSafe, ptr};

#[repr(C)]
pub enum Status {
    Ok,
    InvalidArguments,
    InsufficientBufferSize,
    OsError,
    Panic,
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
        Err(_error) => Status::Panic,
    }
}
