#![allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare

use std::{
    ffi::CStr,
    io, mem,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle},
};
use windows_sys::Win32::{
    Foundation::{ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE},
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, MODULEENTRY32, Module32First, Module32Next, PROCESSENTRY32W,
        Process32FirstW, Process32NextW,
    },
};

/// A snapshot of process modules, threads, and heaps
pub struct ProcessSnapshot {
    handle: OwnedHandle,
}

impl ProcessSnapshot {
    /// Create a new process snapshot using `CreateToolhelp32Snapshot`
    pub fn new(flags: u32, process_id: u32) -> io::Result<ProcessSnapshot> {
        // SAFETY: `CreateToolhelp32Snapshot` should handle invalid flags and process IDs
        let snap = unsafe { CreateToolhelp32Snapshot(flags, process_id) };

        if snap == INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(ProcessSnapshot {
                // SAFETY: `snap` is a valid handle since `CreateToolhelp32Snapshot` succeeded
                handle: unsafe { OwnedHandle::from_raw_handle(snap) },
            })
        }
    }

    /// Return an iterator over the modules in the snapshot
    pub fn modules(&self) -> ProcessSnapshotModules<'_> {
        let entry = MODULEENTRY32 {
            dwSize: mem::size_of::<MODULEENTRY32>() as u32,
            ..Default::default()
        };

        ProcessSnapshotModules {
            snapshot: self,
            iter_started: false,
            temp_entry: entry,
        }
    }

    /// Return an iterator over the processes in the snapshot
    pub fn processes(&self) -> ProcessSnapshotEntries<'_> {
        let entry = PROCESSENTRY32W {
            dwSize: mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        ProcessSnapshotEntries {
            snapshot: self,
            iter_started: false,
            temp_entry: entry,
        }
    }
}

impl AsRawHandle for ProcessSnapshot {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle.as_raw_handle()
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
            // SAFETY: `self.snapshot` is a valid pointer, and `temp_entry` is a valid `MODULEENTRY32`
            if unsafe { Module32Next(self.snapshot.as_raw_handle(), &raw mut self.temp_entry) } == 0
            {
                let last_error = io::Error::last_os_error();

                return if last_error.raw_os_error().unwrap() as u32 == ERROR_NO_MORE_FILES {
                    None
                } else {
                    Some(Err(last_error))
                };
            }
        } else {
            // SAFETY: `self.snapshot` is a valid pointer, and `temp_entry` is a valid `MODULEENTRY32`
            if unsafe { Module32First(self.snapshot.as_raw_handle(), &raw mut self.temp_entry) }
                == 0
            {
                return Some(Err(io::Error::last_os_error()));
            }
            self.iter_started = true;
        }

        let cstr_ref = &self.temp_entry.szModule[0];
        // SAFETY: `szModule` is a null-terminated C string
        let cstr = unsafe { CStr::from_ptr(cstr_ref) };
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
            // SAFETY: `self.snapshot` is a valid pointer, and `temp_entry` is a valid `PROCESSENTRY32W`
            if unsafe { Process32NextW(self.snapshot.as_raw_handle(), &raw mut self.temp_entry) }
                == 0
            {
                let last_error = io::Error::last_os_error();

                return if last_error.raw_os_error().unwrap() as u32 == ERROR_NO_MORE_FILES {
                    None
                } else {
                    Some(Err(last_error))
                };
            }
        } else {
            // SAFETY: `self.snapshot` is a valid pointer, and `temp_entry` is a valid `PROCESSENTRY32W`
            if unsafe { Process32FirstW(self.snapshot.as_raw_handle(), &raw mut self.temp_entry) }
                == 0
            {
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
