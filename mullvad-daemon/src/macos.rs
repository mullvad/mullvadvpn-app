use std::io;

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

    const INCREASED_FILEHANDLE_LIMIT: u64 = 1024;
    // if file handle limit is already big enough, there's no reason to decrease it.
    if limits.rlim_cur >= INCREASED_FILEHANDLE_LIMIT {
        return;
    }

    limits.rlim_cur = INCREASED_FILEHANDLE_LIMIT;
    // SAFETY: `&limits` is a valid pointer parameter for the getrlimit syscall
    let status = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limits) };
    if status != 0 {
        log::error!(
            "Failed to set file handle limit to {}: {}-{}",
            INCREASED_FILEHANDLE_LIMIT,
            io::Error::from_raw_os_error(status),
            status
        );
    }
}
