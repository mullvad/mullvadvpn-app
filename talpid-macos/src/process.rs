use libc::{
    c_void, pid_t, proc_bsdinfo, proc_fdinfo, proc_listallpids, proc_pidfdinfo, proc_pidinfo,
    proc_pidpath, PROC_PIDLISTFDS, PROC_PIDTBSDINFO,
};
use std::{
    ffi::{c_int, CStr, CString},
    io,
    path::{Path, PathBuf},
    ptr,
};

use crate::bindings::{vnode_fdinfowithpath, PROC_PIDFDVNODEPATHINFO, PROX_FDTYPE_VNODE};

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

/// Return file descriptors associated with `pid`
// reference: lsof source code: https://github.com/apple-oss-distributions/lsof/blob/c48c28f51e82a5d682a4459bdbdc42face73468f/lsof/dialects/darwin/libproc/dproc.c#L623
pub fn process_file_descriptors(pid: pid_t) -> io::Result<Vec<proc_fdinfo>> {
    // SAFETY: Passing nil arguments is safe and returns the required buffer size for the given pid
    let fds_buf_size = unsafe { proc_pidinfo(pid, PROC_PIDLISTFDS, 0, ptr::null_mut(), 0) };

    if fds_buf_size < 0 {
        return Err(io::Error::last_os_error());
    }

    let fds_num = fds_buf_size as usize / std::mem::size_of::<proc_fdinfo>();

    // SAFETY: This is a pure C struct which we're expected to zero-initialize
    let empty_fdinfo = unsafe { std::mem::zeroed::<proc_fdinfo>() };

    let mut file_desc_buf = vec![empty_fdinfo; fds_num as usize];

    // SAFETY: fds_buf is large enough to contain `fds_num`
    let fds_buf_size = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDLISTFDS,
            0,
            file_desc_buf.as_mut_ptr() as _,
            fds_buf_size as c_int,
        )
    };
    if fds_buf_size < 0 {
        return Err(io::Error::last_os_error());
    }

    // Truncate file descriptor vector based on new count
    let new_fds_num = fds_buf_size as usize / std::mem::size_of::<proc_fdinfo>();
    assert!(new_fds_num <= fds_num);
    file_desc_buf.truncate(new_fds_num);

    Ok(file_desc_buf)
}

/// Return the file path that belongs to a vnode file descriptor type for a given process.
pub fn get_file_desc_vnode_path(pid: pid_t, info: &proc_fdinfo) -> io::Result<CString> {
    assert!(info.proc_fdtype == PROX_FDTYPE_VNODE as _);

    // SAFETY: This is a pure C struct which we're expected to zero-initialize
    let mut vnode: vnode_fdinfowithpath = unsafe { std::mem::zeroed() };

    // SAFETY: Our buffer is initialized, aligned, and large enough to contain the result.
    let err = unsafe {
        proc_pidfdinfo(
            pid,
            info.proc_fd,
            PROC_PIDFDVNODEPATHINFO as _,
            &mut vnode as *mut _ as _,
            std::mem::size_of_val(&vnode) as _,
        )
    };
    if err <= 0 {
        return Err(io::Error::last_os_error());
    }

    // SAFETY: `proc_pidfdinfo` returned a null-terminated path here
    let cstr_path = unsafe { CStr::from_ptr(vnode.pvip.vip_path.as_ptr()) };
    Ok(cstr_path.to_owned())
}

/// Return the 'proc_bsdinfo' associated with a given process identifier
pub fn process_bsdinfo(pid: pid_t) -> io::Result<proc_bsdinfo> {
    // SAFETY: This is a pure C struct which we're expected to zero-initialize
    let mut info: proc_bsdinfo = unsafe { std::mem::zeroed() };

    // SAFETY: Our buffer (info) is initialized, aligned, and large enough to contain the result.
    let err = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDTBSDINFO as _,
            0,
            &mut info as *mut proc_bsdinfo as *mut c_void,
            std::mem::size_of_val(&info) as _,
        )
    };
    if err <= 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(info)
}
