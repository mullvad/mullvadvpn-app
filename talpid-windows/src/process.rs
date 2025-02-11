#![allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare

use std::{
    ffi::{c_char, CStr},
    io, mem,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE},
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Module32First, Module32Next, Process32FirstW, Process32NextW,
        MODULEENTRY32, PROCESSENTRY32W,
    },
};

/// A snapshot of process modules, threads, and heaps
pub struct ProcessSnapshot {
    handle: HANDLE,
}

impl ProcessSnapshot {
    /// Create a new process snapshot using `CreateToolhelp32Snapshot`
    pub fn new(flags: u32, process_id: u32) -> io::Result<ProcessSnapshot> {
        let snap = unsafe { CreateToolhelp32Snapshot(flags, process_id) };

        if snap == INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(ProcessSnapshot { handle: snap })
        }
    }

    /// Return the raw handle
    pub fn as_raw(&self) -> HANDLE {
        self.handle
    }

    /// Return an iterator over the modules in the snapshot
    pub fn modules(&self) -> ProcessSnapshotModules<'_> {
        let mut entry: MODULEENTRY32 = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;

        ProcessSnapshotModules {
            snapshot: self,
            iter_started: false,
            temp_entry: entry,
        }
    }

    /// Return an iterator over the processes in the snapshot
    pub fn processes(&self) -> ProcessSnapshotEntries<'_> {
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

/// Description of a snapshot module entry. See `MODULEENTRY32`
pub struct ModuleEntry {
    /// Module name
    pub name: String,
    /// Module base address (in the owning process)
    pub base_address: *const u8,
    /// Size of the module (in bytes)
    pub size: usize,
}

/// Module iterator for [ProcessSnapshot]
pub struct ProcessSnapshotModules<'a> {
    snapshot: &'a ProcessSnapshot,
    iter_started: bool,
    temp_entry: MODULEENTRY32,
}

impl Iterator for ProcessSnapshotModules<'_> {
    type Item = io::Result<ModuleEntry>;

    fn next(&mut self) -> Option<io::Result<ModuleEntry>> {
        if self.iter_started {
            if unsafe { Module32Next(self.snapshot.as_raw(), &mut self.temp_entry) } == 0 {
                let last_error = io::Error::last_os_error();

                return if last_error.raw_os_error().unwrap() as u32 == ERROR_NO_MORE_FILES {
                    None
                } else {
                    Some(Err(last_error))
                };
            }
        } else {
            if unsafe { Module32First(self.snapshot.as_raw(), &mut self.temp_entry) } == 0 {
                return Some(Err(io::Error::last_os_error()));
            }
            self.iter_started = true;
        }

        let cstr_ref = &self.temp_entry.szModule[0];
        let cstr = unsafe { CStr::from_ptr(cstr_ref as *const u8 as *const c_char) };
        Some(Ok(ModuleEntry {
            name: cstr.to_string_lossy().into_owned(),
            base_address: self.temp_entry.modBaseAddr,
            size: self.temp_entry.modBaseSize as usize,
        }))
    }
}

/// Description of a snapshot process entry. See `PROCESSENTRY32W`
pub struct ProcessEntry {
    /// Process identifier
    pub pid: u32,
    /// Process identifier of the parent process
    pub parent_pid: u32,
}

/// Process iterator for [ProcessSnapshot]
pub struct ProcessSnapshotEntries<'a> {
    snapshot: &'a ProcessSnapshot,
    iter_started: bool,
    temp_entry: PROCESSENTRY32W,
}

impl Iterator for ProcessSnapshotEntries<'_> {
    type Item = io::Result<ProcessEntry>;

    fn next(&mut self) -> Option<io::Result<ProcessEntry>> {
        if self.iter_started {
            if unsafe { Process32NextW(self.snapshot.as_raw(), &mut self.temp_entry) } == 0 {
                let last_error = io::Error::last_os_error();

                return if last_error.raw_os_error().unwrap() as u32 == ERROR_NO_MORE_FILES {
                    None
                } else {
                    Some(Err(last_error))
                };
            }
        } else {
            if unsafe { Process32FirstW(self.snapshot.as_raw(), &mut self.temp_entry) } == 0 {
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
