use std::io;
use std::os::windows::io::AsRawHandle;
use std::ptr;

use windows_sys::Win32::{
    Foundation::ERROR_SUCCESS,
    Security::{
        Authorization::{GetSecurityInfo, SE_FILE_OBJECT},
        IsWellKnownSid, WinBuiltinAdministratorsSid, WinLocalSystemSid, OWNER_SECURITY_INFORMATION,
        SID,
    },
};

/// Return whether a file handle is owned by either SYSTEM or the built-in administrators account
pub fn is_admin_owned<T: AsRawHandle>(handle: T) -> io::Result<bool> {
    let mut owner: *mut SID = ptr::null_mut();

    // SAFETY: `handle` is a valid handle. We return a pointer to the owner associated with the handle(?)
    let result = unsafe {
        GetSecurityInfo(
            handle.as_raw_handle() as isize,
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            (&mut owner) as *mut *mut SID as _,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };

    if result != ERROR_SUCCESS {
        return Err(io::Error::last_os_error());
    }

    Ok(
        // SAFETY: `owner` is valid, and the well-known type is a valid argument
        unsafe { IsWellKnownSid(owner as _, WinBuiltinAdministratorsSid) != 0 } ||
        // SAFETY: `owner` is valid, and the well-known type is a valid argument
        unsafe { IsWellKnownSid(owner as _, WinLocalSystemSid) != 0 },
    )
}
