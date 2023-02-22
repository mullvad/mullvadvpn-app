#![cfg(all(target_arch = "x86", target_os = "windows"))]

// TODO: add logging

use std::{os::windows::ffi::OsStrExt, panic::UnwindSafe, ptr};

macro_rules! check {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(status_error) => {
                return status_error;
            }
        }
    };
}

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

        let path = check!(mullvad_paths::windows::get_system_service_appdata()
            .map_err(|_error| { Status::OsError }));

        let path = path.as_os_str();

        let req_len = path.len() + 1;

        if *buffer_size < req_len {
            return Status::InsufficientBufferSize;
        }

        *buffer_size = req_len;

        let path_u16: Vec<u16> = path.encode_wide().chain(std::iter::once(0u16)).collect();

        ptr::copy_nonoverlapping(path_u16.as_ptr(), buffer, req_len);

        Status::Ok
    })
}

fn catch_and_log_unwind(func: impl FnOnce() -> Status + UnwindSafe) -> Status {
    match std::panic::catch_unwind(func) {
        Ok(status) => status,
        Err(_error) => Status::Panic,
    }
}
