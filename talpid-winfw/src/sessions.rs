//! Diagnostics for a WFP transaction lock that could not be acquired: the
//! processes with open WFP sessions, one of which is holding it.

use std::{
    collections::BTreeSet,
    ffi::OsString,
    io,
    os::windows::{ffi::OsStringExt, io::AsRawHandle},
    path::PathBuf,
    ptr,
};

use wfp::FilterEngineBuilder;
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_SUCCESS, HANDLE, MAX_PATH},
    NetworkManagement::WindowsFilteringPlatform::{
        FWPM_SESSION0, FwpmFreeMemory0, FwpmSessionCreateEnumHandle0,
        FwpmSessionDestroyEnumHandle0, FwpmSessionEnum0,
    },
    System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    },
};

/// Number of sessions to request from the engine per enumeration call.
const ENUM_BATCH_SIZE: u32 = 100;

/// Log the processes with open WFP sessions, by file name, or by process ID
/// where the process cannot be queried. The path is not logged.
pub fn log_active_sessions() {
    match active_session_processes() {
        Ok(pids) if pids.is_empty() => log::debug!("No active WFP sessions found"),
        Ok(pids) => {
            let processes: Vec<String> = pids.into_iter().map(describe_process).collect();
            log::debug!(
                "Active WFP sessions from processes: {}",
                processes.join(", ")
            );
        }
        Err(error) => log::error!("Failed to log WFP sessions: {error}"),
    }
}

/// The IDs of the processes with open WFP sessions, except the kernel's.
fn active_session_processes() -> io::Result<BTreeSet<u32>> {
    let engine = FilterEngineBuilder::default().dynamic().open()?;
    let engine_handle = engine.as_raw_handle() as HANDLE;

    let mut enum_handle: HANDLE = ptr::null_mut();
    // SAFETY: The engine handle is open, and a null template enumerates every session.
    check(unsafe {
        FwpmSessionCreateEnumHandle0(engine_handle, ptr::null(), &raw mut enum_handle)
    })?;

    let result = enumerate_sessions(engine_handle, enum_handle);

    // SAFETY: The enumeration handle was created above, on the same engine handle.
    unsafe { FwpmSessionDestroyEnumHandle0(engine_handle, enum_handle) };

    result
}

fn enumerate_sessions(engine_handle: HANDLE, enum_handle: HANDLE) -> io::Result<BTreeSet<u32>> {
    let mut pids = BTreeSet::new();
    loop {
        let mut entries: *mut *mut FWPM_SESSION0 = ptr::null_mut();
        let mut num_entries = 0u32;
        // SAFETY: Both handles are valid. On success, `entries` points to `num_entries`
        // session pointers, owned by the engine until freed below.
        check(unsafe {
            FwpmSessionEnum0(
                engine_handle,
                enum_handle,
                ENUM_BATCH_SIZE,
                &raw mut entries,
                &raw mut num_entries,
            )
        })?;

        if !entries.is_null() {
            // SAFETY: See above.
            let sessions = unsafe { std::slice::from_raw_parts(entries, num_entries as usize) };
            for &session in sessions {
                // SAFETY: Every entry points to a valid session.
                let pid = unsafe { (*session).processId };
                if pid != 0 {
                    pids.insert(pid);
                }
            }
            // SAFETY: `entries` was allocated by `FwpmSessionEnum0`.
            unsafe { FwpmFreeMemory0((&raw mut entries).cast()) };
        }

        if num_entries < ENUM_BATCH_SIZE {
            return Ok(pids);
        }
    }
}

/// The file name of the process's image, or its ID if that cannot be found.
fn describe_process(pid: u32) -> String {
    process_image_path(pid)
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| format!("PID:{pid}"))
}

fn process_image_path(pid: u32) -> io::Result<PathBuf> {
    // SAFETY: Opening a process has no preconditions.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return Err(io::Error::last_os_error());
    }

    let mut buffer = [0u16; MAX_PATH as usize];
    let mut size = buffer.len() as u32;
    // SAFETY: `size` is the length of `buffer` in characters.
    let status = unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            buffer.as_mut_ptr(),
            &raw mut size,
        )
    };
    let result = if status != 0 {
        Ok(PathBuf::from(OsString::from_wide(&buffer[..size as usize])))
    } else {
        Err(io::Error::last_os_error())
    };

    // SAFETY: The handle was opened above.
    unsafe { CloseHandle(process) };

    result
}

fn check(status: u32) -> io::Result<()> {
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(status as i32))
    }
}
