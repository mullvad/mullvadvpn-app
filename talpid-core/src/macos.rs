use std::{ffi::CStr, io};

/// Returns the GID of the specified group name
pub fn get_group_id(group_name: &CStr) -> io::Result<u32> {
    // SAFETY: group_name is a valid CString
    let group = unsafe { libc::getgrnam(group_name.as_ptr() as *const _) };
    if group.is_null() {
        return Err(io::Error::from(io::ErrorKind::NotFound));
    }
    // SAFETY: group is not null
    let gid = unsafe { (*group).gr_gid };
    Ok(gid)
}

/// Sets group ID for the current process
pub fn set_gid(gid: u32) -> io::Result<()> {
    if unsafe { libc::setgid(gid) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

const INCREASED_FILEHANDLE_LIMIT: u64 = 1024;
/// Bump filehandle limit
pub fn bump_filehandle_limit() {
    let mut limits = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: `&mut limits` is a valid pointer parameter for the getrlimit syscall
    let status = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limits) };
    if status != 0 {
        log::error!(
            "Failed to get file handle limits: {}-{}",
            io::Error::from_raw_os_error(status),
            status
        );
        return;
    }

    // if file handle limit is already big enough, there's no reason to decrease it.
    if limits.rlim_cur >= INCREASED_FILEHANDLE_LIMIT {
        return;
    }

    limits.rlim_cur = INCREASED_FILEHANDLE_LIMIT;
    // SAFETY: `&limits` is a valid pointer parameter for the getrlimit syscall
    let status = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limits as *const _) };
    if status != 0 {
        log::error!(
            "Failed to set file handle limit to {}: {}-{}",
            INCREASED_FILEHANDLE_LIMIT,
            io::Error::from_raw_os_error(status),
            status
        );
    }
}

#[cfg(test)]
#[test]
fn test_unknown_group() {
    let unknown_group = CStr::from_bytes_with_nul(b"asdunknown\0").unwrap();
    let group_err = get_group_id(unknown_group).unwrap_err();
    assert!(group_err.kind() == io::ErrorKind::NotFound)

}
