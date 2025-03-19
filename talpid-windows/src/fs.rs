use core::ffi::c_void;
use std::io;
use std::os::windows::io::AsRawHandle;
use std::ptr;

use windows_sys::Win32::{
    Foundation::{LocalFree, ERROR_SUCCESS},
    Security::{
        Authorization::{GetSecurityInfo, SE_FILE_OBJECT},
        IsWellKnownSid, WinBuiltinAdministratorsSid, WinLocalSystemSid, OWNER_SECURITY_INFORMATION,
        SECURITY_DESCRIPTOR, SID,
    },
};

/// Return whether a file handle is owned by either SYSTEM or the built-in administrators account
pub fn is_admin_owned<T: AsRawHandle>(handle: T) -> io::Result<bool> {
    let mut security_descriptor: *mut SECURITY_DESCRIPTOR = ptr::null_mut();
    let mut owner: *mut SID = ptr::null_mut();

    // SAFETY: `handle` is a valid handle. We return a pointer to the owner associated with the handle(?)
    let result = unsafe {
        GetSecurityInfo(
            handle.as_raw_handle() as isize,
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            (&mut owner) as *mut *mut SID as *mut *mut c_void,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            (&mut security_descriptor) as *mut *mut SECURITY_DESCRIPTOR as *mut *mut c_void,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(result as i32));
    }

    // SAFETY: `owner` is valid since `security_descriptor` still is, and the well-known type is a valid argument
    let is_system_owned = unsafe { IsWellKnownSid(owner as _, WinLocalSystemSid) != 0 };
    // SAFETY: `owner` is valid since `security_descriptor` still is, and the well-known type is a valid argument
    let is_admin_owned = unsafe { IsWellKnownSid(owner as _, WinBuiltinAdministratorsSid) != 0 };

    // SAFETY: Since we no longer need the descriptor (or owner), it may be freed
    unsafe { LocalFree(security_descriptor.cast()) };

    Ok(is_system_owned || is_admin_owned)
}
