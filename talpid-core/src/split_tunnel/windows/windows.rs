// TODO: The snapshot code could be combined with the mostly-identical code in
//       the windows_exception_logging module.

use std::{
    ffi::{OsStr, OsString},
    fs, io, iter, mem,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        io::{AsRawHandle, RawHandle},
    },
    path::{Component, Path},
    ptr,
};
use winapi::{
    shared::{
        minwindef::{DWORD, FALSE, FILETIME, TRUE},
        ntdef::ULARGE_INTEGER,
        winerror::{ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_FILES},
    },
    um::{
        fileapi::{GetFinalPathNameByHandleW, QueryDosDeviceW},
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        processthreadsapi::{GetProcessTimes, OpenProcess},
        psapi::K32GetProcessImageFileNameW,
        tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W},
        winnt::{HANDLE, PROCESS_QUERY_LIMITED_INFORMATION},
    },
};

/// Return path with the volume device path.
const VOLUME_NAME_NT: u32 = 0x02;

pub struct ProcessSnapshot {
    handle: HANDLE,
}

impl ProcessSnapshot {
    pub fn new(flags: DWORD, process_id: DWORD) -> io::Result<ProcessSnapshot> {
        let snap = unsafe { CreateToolhelp32Snapshot(flags, process_id) };

        if snap == INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(ProcessSnapshot { handle: snap })
        }
    }

    pub fn handle(&self) -> HANDLE {
        self.handle
    }

    pub fn entries(&self) -> ProcessSnapshotEntries<'_> {
        let mut entry: PROCESSENTRY32W = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

        ProcessSnapshotEntries {
            snapshot: self,
            iter_started: false,
            temp_entry: entry,
        }
    }
}

impl Drop for ProcessSnapshot {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

pub struct ProcessEntry {
    pub pid: u32,
    pub parent_pid: u32,
}

pub struct ProcessSnapshotEntries<'a> {
    snapshot: &'a ProcessSnapshot,
    iter_started: bool,
    temp_entry: PROCESSENTRY32W,
}

impl Iterator for ProcessSnapshotEntries<'_> {
    type Item = io::Result<ProcessEntry>;

    fn next(&mut self) -> Option<io::Result<ProcessEntry>> {
        if self.iter_started {
            if unsafe { Process32NextW(self.snapshot.handle(), &mut self.temp_entry) } == FALSE {
                let last_error = io::Error::last_os_error();

                return if last_error.raw_os_error().unwrap() as u32 == ERROR_NO_MORE_FILES {
                    None
                } else {
                    Some(Err(last_error))
                };
            }
        } else {
            if unsafe { Process32FirstW(self.snapshot.handle(), &mut self.temp_entry) } == FALSE {
                return Some(Err(io::Error::last_os_error()));
            }
            self.iter_started = true;
        }

        Some(Ok(ProcessEntry {
            pid: self.temp_entry.th32ProcessID,
            parent_pid: self.temp_entry.th32ParentProcessID,
        }))
    }
}

/// Obtains a device path without resolving links or mount points.
pub fn get_device_path<T: AsRef<Path>>(path: T) -> Result<OsString, io::Error> {
    // Preferentially, use GetFinalPathNameByHandleW. If the file does not exist
    // or cannot be opened, infer the path from the label only.
    if let Ok(file) = fs::OpenOptions::new().read(true).open(path.as_ref()) {
        return get_final_path_name_by_handle(file.as_raw_handle());
    }

    let mut components = path.as_ref().components();
    let drive_comp = components.next();
    let drive = match (drive_comp, components.next()) {
        (Some(Component::Prefix(prefix)), Some(Component::RootDir)) => prefix.as_os_str(),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path must be absolute",
            ))
        }
    };

    let mut new_path = query_dos_device(drive)?;
    let suffix = path
        .as_ref()
        .strip_prefix(drive_comp.unwrap())
        .map_err(|_error| {
            io::Error::new(io::ErrorKind::InvalidInput, "path missing own component")
        })?;
    new_path.push(suffix);

    Ok(new_path)
}

pub fn get_final_path_name_by_handle(raw_handle: RawHandle) -> Result<OsString, io::Error> {
    let buffer_size = unsafe {
        GetFinalPathNameByHandleW(raw_handle as *mut _, ptr::null_mut(), 0u32, VOLUME_NAME_NT)
    } as usize;
    let mut buffer = Vec::new();
    buffer.resize(buffer_size, 0);

    let status = unsafe {
        GetFinalPathNameByHandleW(
            raw_handle as *mut _,
            buffer.as_mut_ptr(),
            buffer_size as u32,
            VOLUME_NAME_NT,
        )
    } as usize;

    if status == 0 {
        return Err(io::Error::last_os_error());
    }

    buffer.resize(buffer_size - 1, 0);
    Ok(OsStringExt::from_wide(&buffer))
}

/// Obtains the real device path for a label (such as C:).
/// The underlying function may return multiple paths, but only the first is returned.
fn query_dos_device<T: AsRef<OsStr>>(device_name: T) -> io::Result<OsString> {
    let device_name_c: Vec<u16> = device_name
        .as_ref()
        .encode_wide()
        .chain(iter::once(0u16))
        .collect();
    let mut new_prefix = vec![0u16; 64];

    loop {
        let prefix_len = unsafe {
            QueryDosDeviceW(
                device_name_c.as_ptr(),
                new_prefix.as_mut_ptr(),
                new_prefix.len() as u32,
            ) as usize
        };

        if prefix_len == 0 {
            let last_error = io::Error::last_os_error();
            if last_error.raw_os_error() == Some(ERROR_INSUFFICIENT_BUFFER as i32) {
                // resize buffer and try again
                new_prefix.resize(2 * new_prefix.len(), 0);
                continue;
            }
            break Err(last_error);
        }

        // We must scan for the first null terminator
        // Because `new_prefix` may contain multiple strings.

        let real_len = new_prefix.iter().position(|&c| c == 0u16).unwrap();
        unsafe { new_prefix.set_len(real_len) };

        break Ok(OsString::from_wide(&new_prefix));
    }
}

/// Object that frees its handle when dropped.
pub struct WinHandle(RawHandle);

impl WinHandle {
    pub fn get_raw(&self) -> RawHandle {
        self.0
    }
}

impl Drop for WinHandle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

#[repr(u32)]
pub enum ProcessAccess {
    QueryLimitedInformation = PROCESS_QUERY_LIMITED_INFORMATION,
    // TODO: could be extended
}

/// Open an existing process object.
pub fn open_process(
    access: ProcessAccess,
    inherit_handle: bool,
    pid: u32,
) -> Result<WinHandle, io::Error> {
    let handle = unsafe {
        OpenProcess(
            access as u32,
            if inherit_handle { TRUE } else { FALSE },
            pid,
        )
    };

    if handle == ptr::null_mut() {
        return Err(io::Error::last_os_error());
    }
    Ok(WinHandle(handle))
}

/// Returns the age of a running process.
pub fn get_process_creation_time(handle: RawHandle) -> Result<u64, io::Error> {
    // TODO: FileTimeToSystemTime -> chrono::NaiveDateTime
    let mut creation_time: FILETIME = unsafe { mem::zeroed() };
    let mut dummy: FILETIME = unsafe { mem::zeroed() };
    if unsafe {
        GetProcessTimes(
            handle,
            &mut creation_time as *mut _,
            &mut dummy as *mut _,
            &mut dummy as *mut _,
            &mut dummy as *mut _,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }

    let mut uli_time: ULARGE_INTEGER = unsafe { mem::zeroed() };
    unsafe {
        uli_time.s_mut().LowPart = creation_time.dwLowDateTime;
        uli_time.s_mut().HighPart = creation_time.dwHighDateTime;
    }

    Ok(*unsafe { uli_time.QuadPart() })
}

/// Returns the device path for a running process.
pub fn get_process_device_path(handle: RawHandle) -> Result<OsString, io::Error> {
    let mut initial_capacity = 512;
    loop {
        let result = get_process_device_path_inner(handle, initial_capacity);
        match result {
            Ok(path) => return Ok(path),
            Err(error) => {
                if ERROR_INSUFFICIENT_BUFFER == error.raw_os_error().unwrap() as u32 {
                    // Try again with a larger buffer capacity.
                    initial_capacity *= 2;
                    continue;
                }
                return Err(error);
            }
        }
    }
}

fn get_process_device_path_inner(
    handle: RawHandle,
    buffer_capacity: usize,
) -> Result<OsString, io::Error> {
    let mut buffer = Vec::<u16>::new();
    buffer.reserve_exact(buffer_capacity);

    let written = unsafe {
        K32GetProcessImageFileNameW(
            handle,
            buffer.as_mut_ptr() as *mut _,
            buffer.capacity() as u32,
        )
    };
    if written == 0 {
        return Err(io::Error::last_os_error());
    }

    // `written` does not include a null terminator
    unsafe { buffer.set_len(written as usize) };

    Ok(OsStringExt::from_wide(&buffer))
}
