use std::{ffi::CStr, mem, os::raw::c_char};

use winapi::{
    ctypes::c_void,
    shared::{
        minwindef::{BYTE, DWORD, FALSE},
        winerror::ERROR_NO_MORE_FILES,
    },
    um::{
        errhandlingapi::SetUnhandledExceptionFilter,
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        tlhelp32::{
            CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE,
        },
        winnt::{EXCEPTION_POINTERS, EXCEPTION_RECORD, HANDLE, LONG},
    },
    vc::excpt::EXCEPTION_EXECUTE_HANDLER,
};

/// Enable logging of unhandled SEH exceptions.
pub fn enable() {
    unsafe { SetUnhandledExceptionFilter(Some(logging_exception_filter)) };
}

extern "system" fn logging_exception_filter(info: *mut EXCEPTION_POINTERS) -> LONG {
    // TODO: output the error constant's name instead of its numeric value
    // SAFETY: Windows gives us valid pointers
    let info: &EXCEPTION_POINTERS = unsafe { &*info };
    let record: &EXCEPTION_RECORD = unsafe { &*info.ExceptionRecord };

    match find_address_module(record.ExceptionAddress) {
        Some(mod_info) => log::error!(
            "Unhandled exception at {:#x?} in {}: {:#x?}",
            record.ExceptionAddress as usize - mod_info.base_address as usize,
            mod_info.name,
            record.ExceptionCode,
        ),
        None => log::error!(
            "Unhandled exception at {:#x?}: {:#x?}",
            record.ExceptionAddress,
            record.ExceptionCode
        ),
    }

    EXCEPTION_EXECUTE_HANDLER
}

/// Return module info for the current process and given memory address.
fn find_address_module(address: *mut c_void) -> Option<ModuleInfo> {
    let snap =
        ProcessSnapshot::new(TH32CS_SNAPMODULE, 0).expect("could not create process snapshot");

    for module in snap.modules() {
        let module_end_address = unsafe { module.base_address.offset(module.size as isize) };
        if (address as *const BYTE) >= module.base_address
            && (address as *const BYTE) < module_end_address
        {
            return Some(module);
        }
    }

    None
}

struct ModuleInfo {
    name: String,
    base_address: *const BYTE,
    size: usize,
}

struct ProcessSnapshot {
    handle: HANDLE,
}

impl ProcessSnapshot {
    fn new(flags: DWORD, process_id: DWORD) -> std::io::Result<ProcessSnapshot> {
        let snap = unsafe { CreateToolhelp32Snapshot(flags, process_id) };

        if snap == INVALID_HANDLE_VALUE {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(ProcessSnapshot { handle: snap })
        }
    }

    fn handle(&self) -> HANDLE {
        self.handle
    }

    fn modules(&self) -> ProcessSnapshotModules<'_> {
        let mut entry: MODULEENTRY32 = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;

        ProcessSnapshotModules {
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

struct ProcessSnapshotModules<'a> {
    snapshot: &'a ProcessSnapshot,
    iter_started: bool,
    temp_entry: MODULEENTRY32,
}

impl Iterator for ProcessSnapshotModules<'_> {
    type Item = ModuleInfo;

    fn next(&mut self) -> Option<ModuleInfo> {
        if self.iter_started {
            if unsafe { Module32Next(self.snapshot.handle(), &mut self.temp_entry) } == FALSE {
                let last_error = std::io::Error::last_os_error();

                return if last_error.raw_os_error().unwrap() as u32 == ERROR_NO_MORE_FILES {
                    None
                } else {
                    panic!(
                        "Windows error during ProcessSnapshot iteration: {}",
                        last_error
                    )
                };
            }
        } else {
            if unsafe { Module32First(self.snapshot.handle(), &mut self.temp_entry) } == FALSE {
                let last_error = std::io::Error::last_os_error();
                panic!(
                    "Windows error during ProcessSnapshot iteration: {}",
                    last_error
                );
            }
            self.iter_started = true;
        }

        let cstr = unsafe { CStr::from_ptr(&self.temp_entry.szModule[0] as *const c_char) };
        Some(ModuleInfo {
            name: cstr.to_string_lossy().into_owned(),
            base_address: self.temp_entry.modBaseAddr,
            size: self.temp_entry.modBaseSize as usize,
        })
    }
}
