use libc::{c_void, pid_t, proc_listallpids, proc_pidpath};
use std::{
    io,
    path::{Path, PathBuf},
};

/// Return the first process identifier matching a specified path, if one exists.
pub fn pid_of_path(find_path: impl AsRef<Path>) -> Option<pid_t> {
    match list_pids() {
        Ok(pids) => {
            for pid in pids {
                if let Ok(path) = process_path(pid) {
                    if path == find_path.as_ref() {
                        return Some(pid);
                    }
                }
            }
            None
        }
        Err(error) => {
            log::error!("Failed to list processes: {error}");
            None
        }
    }
}

/// Obtain a list of all process identifiers
pub fn list_pids() -> io::Result<Vec<pid_t>> {
    // SAFETY: Passing in null and 0 returns the number of processes
    let num_pids = unsafe { proc_listallpids(std::ptr::null_mut(), 0) };
    if num_pids <= 0 {
        return Err(io::Error::last_os_error());
    }
    let num_pids = usize::try_from(num_pids).unwrap();
    let mut pids = vec![0i32; num_pids];

    let buf_sz = (num_pids * std::mem::size_of::<pid_t>()) as i32;
    // SAFETY: 'pids' is large enough to contain 'num_pids' processes
    let num_pids = unsafe { proc_listallpids(pids.as_mut_ptr() as *mut c_void, buf_sz) };
    if num_pids == -1 {
        return Err(io::Error::last_os_error());
    }

    pids.resize(usize::try_from(num_pids).unwrap(), 0);

    Ok(pids)
}

/// Return the path of the process `pid`
pub fn process_path(pid: pid_t) -> io::Result<PathBuf> {
    let mut buffer = [0u8; libc::MAXPATHLEN as usize];
    // SAFETY: `proc_pidpath` returns at most `buffer.len()` bytes
    let buf_len = unsafe {
        proc_pidpath(
            pid,
            buffer.as_mut_ptr() as *mut c_void,
            u32::try_from(buffer.len()).unwrap(),
        )
    };
    if buf_len == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(PathBuf::from(
        std::str::from_utf8(&buffer[0..buf_len as usize])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid process path"))?,
    ))
}
