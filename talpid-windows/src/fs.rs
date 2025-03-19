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

    // SAFETY: `handle` is a valid handle
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

#[cfg(test)]
mod test {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;

    use super::is_admin_owned;

    #[test]
    pub fn test_is_admin_owned() {
        // The kernel image is owned by "TrustedInstaller", so we expect the function to return 'false'
        let path = std::fs::File::open(r"C:\Windows\System32\ntoskrnl.exe").unwrap();
        let result = is_admin_owned(path);
        assert!(
            matches!(result, Ok(false)),
            "expected ntoskrnl.exe to be owned by TrustedInstaller (false), got {result:?}"
        );

        // The Windows system temp directory is owned by SYSTEM, so we expect 'true'
        let path = std::fs::File::options()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(r"C:\Windows\Temp")
            .unwrap();
        let result = is_admin_owned(path);
        assert!(
            matches!(result, Ok(true)),
            "expected TEMP to be owned by SYSTEM (true), got {result:?}"
        );
    }
}
