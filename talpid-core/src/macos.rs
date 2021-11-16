use std::{ffi::CStr, io};

/// Returns the GID of `mullvad-exclusion` group if it exists.
pub fn get_group_id(group_name: &CStr) -> Option<u32> {
    let group = unsafe { libc::getgrnam(group_name.as_ptr() as *const _) };
    if group.is_null() {
        return None;
    }
    let gid = unsafe { (*group).gr_gid };
    Some(gid)
}


const INCREASED_FILEHANDLE_LIMIT: u64 = 1024;
/// Bump filehandle limit
pub fn bump_filehandle_limit() {
    let mut limits = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let status = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limits as *mut _) };
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
