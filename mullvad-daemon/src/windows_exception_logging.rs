use std::{borrow::Cow, ffi::CStr, fmt::Write, io, mem, os::raw::c_char};
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
        winnt::{
            CONTEXT, CONTEXT_CONTROL, CONTEXT_INTEGER, CONTEXT_SEGMENTS, EXCEPTION_POINTERS,
            EXCEPTION_RECORD, HANDLE, LONG,
        },
    },
    vc::excpt::EXCEPTION_EXECUTE_HANDLER,
};

/// Enable logging of unhandled SEH exceptions.
pub fn enable() {
    unsafe { SetUnhandledExceptionFilter(Some(logging_exception_filter)) };
}

fn exception_code_to_string(value: &EXCEPTION_RECORD) -> Option<Cow<'_, str>> {
    use winapi::um::minwinbase::*;
    let name = match value.ExceptionCode {
        EXCEPTION_ACCESS_VIOLATION => "EXCEPTION_ACCESS_VIOLATION",
        EXCEPTION_IN_PAGE_ERROR => "EXCEPTION_IN_PAGE_ERROR",
        EXCEPTION_ARRAY_BOUNDS_EXCEEDED => "EXCEPTION_ARRAY_BOUNDS_EXCEEDED",
        EXCEPTION_DATATYPE_MISALIGNMENT => "EXCEPTION_DATATYPE_MISALIGNMENT",
        EXCEPTION_FLT_DENORMAL_OPERAND => "EXCEPTION_FLT_DENORMAL_OPERAND",
        EXCEPTION_FLT_DIVIDE_BY_ZERO => "EXCEPTION_FLT_DIVIDE_BY_ZERO",
        EXCEPTION_FLT_INEXACT_RESULT => "EXCEPTION_FLT_INEXACT_RESULT",
        EXCEPTION_FLT_INVALID_OPERATION => "EXCEPTION_FLT_INVALID_OPERATION",
        EXCEPTION_FLT_STACK_CHECK => "EXCEPTION_FLT_STACK_CHECK",
        EXCEPTION_FLT_UNDERFLOW => "EXCEPTION_FLT_UNDERFLOW",
        EXCEPTION_ILLEGAL_INSTRUCTION => "EXCEPTION_ILLEGAL_INSTRUCTION",
        EXCEPTION_INT_DIVIDE_BY_ZERO => "EXCEPTION_INT_DIVIDE_BY_ZERO",
        EXCEPTION_INT_OVERFLOW => "EXCEPTION_INT_OVERFLOW",
        EXCEPTION_INVALID_DISPOSITION => "EXCEPTION_INVALID_DISPOSITION",
        EXCEPTION_NONCONTINUABLE_EXCEPTION => "EXCEPTION_NONCONTINUABLE_EXCEPTION",
        EXCEPTION_PRIV_INSTRUCTION => "EXCEPTION_PRIV_INSTRUCTION",
        EXCEPTION_SINGLE_STEP => "EXCEPTION_SINGLE_STEP",
        EXCEPTION_STACK_OVERFLOW => "EXCEPTION_STACK_OVERFLOW",
        _ => return None,
    };

    if value.ExceptionCode == EXCEPTION_ACCESS_VIOLATION
        || value.ExceptionCode == EXCEPTION_IN_PAGE_ERROR
    {
        let operation_type = match value.ExceptionInformation[0] {
            0 => "read from inaccessible address",
            1 => "wrote to inaccessible address",
            8 => "user-mode data execution prevention (DEP) violation",
            _ => "unknown error",
        };
        Some(Cow::Owned(format!(
            "{} ({}, VA {:#x?})",
            name, operation_type, value.ExceptionInformation[1]
        )))
    } else {
        Some(Cow::Borrowed(name))
    }
}

extern "system" fn logging_exception_filter(info: *mut EXCEPTION_POINTERS) -> LONG {
    // SAFETY: Windows gives us valid pointers
    let info: &EXCEPTION_POINTERS = unsafe { &*info };
    let record: &EXCEPTION_RECORD = unsafe { &*info.ExceptionRecord };

    let context_info = get_context_info(unsafe { &*info.ContextRecord });

    let error_str = match exception_code_to_string(record) {
        Some(errstr) => errstr,
        None => Cow::Owned(format!("{:#x?}", record.ExceptionCode)),
    };

    match find_address_module(record.ExceptionAddress) {
        Ok(Some(mod_info)) => log::error!(
            "Unhandled exception at RVA {:#x?} in {}: {}\n{}",
            record.ExceptionAddress as usize - mod_info.base_address as usize,
            mod_info.name,
            error_str,
            context_info
        ),
        Ok(None) => log::error!(
            "Unhandled exception at {:#x?}: {}\n{}",
            record.ExceptionAddress,
            error_str,
            context_info
        ),
        Err(code) => log::error!(
            "Unhandled exception at {:#x?}: {}\n{}\nError during module iteration: {}",
            record.ExceptionAddress,
            error_str,
            context_info,
            code
        ),
    }

    // TODO: check nested exception?

    EXCEPTION_EXECUTE_HANDLER
}

fn get_context_info(context: &CONTEXT) -> String {
    let mut context_str = "Context:\n".to_string();

    if context.ContextFlags & CONTEXT_CONTROL != 0 {
        writeln!(
            &mut context_str,
            "\n\tSegSs: {:#x?}\n \
             \tRsp: {:#x?}\n \
             \tSegCs: {:#x?}\n \
             \tRip: {:#x?}\n \
             \tEFlags: {:#x?}",
            context.SegSs, context.Rsp, context.SegCs, context.Rip, context.EFlags
        )
        .unwrap();
    }

    if context.ContextFlags & CONTEXT_INTEGER != 0 {
        writeln!(
            &mut context_str,
            "\n\tRax: {:#x?}\n \
             \tRcx: {:#x?}\n \
             \tRdx: {:#x?}\n \
             \tRbx: {:#x?}\n \
             \tRbp: {:#x?}\n \
             \tRsi: {:#x?}\n \
             \tRdi: {:#x?}\n \
             \tR8: {:#x?}\n \
             \tR9: {:#x?}\n \
             \tR10: {:#x?}\n \
             \tR11 {:#x?}\n \
             \tR12: {:#x?}\n \
             \tR13: {:#x?}\n \
             \tR14: {:#x?}\n \
             \tR15: {:#x?}",
            context.Rax,
            context.Rcx,
            context.Rdx,
            context.Rbx,
            context.Rbp,
            context.Rsi,
            context.Rdi,
            context.R8,
            context.R9,
            context.R10,
            context.R11,
            context.R12,
            context.R13,
            context.R14,
            context.R15
        )
        .unwrap();
    }

    if context.ContextFlags & CONTEXT_SEGMENTS != 0 {
        writeln!(
            &mut context_str,
            "\n\tSegDs: {:#x?}\n \
             \tSegEs: {:#x?}\n \
             \tSegFs: {:#x?}\n \
             \tSegGs: {:#x?}",
            context.SegDs, context.SegEs, context.SegFs, context.SegGs
        )
        .unwrap();
    }

    context_str
}

/// Return module info for the current process and given memory address.
fn find_address_module(address: *mut c_void) -> io::Result<Option<ModuleInfo>> {
    let snap = ProcessSnapshot::new(TH32CS_SNAPMODULE, 0)?;

    for module in snap.modules() {
        let module = module?;
        let module_end_address = unsafe { module.base_address.offset(module.size as isize) };
        if (address as *const BYTE) >= module.base_address
            && (address as *const BYTE) < module_end_address
        {
            return Ok(Some(module));
        }
    }

    Ok(None)
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
    fn new(flags: DWORD, process_id: DWORD) -> io::Result<ProcessSnapshot> {
        let snap = unsafe { CreateToolhelp32Snapshot(flags, process_id) };

        if snap == INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
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
    type Item = io::Result<ModuleInfo>;

    fn next(&mut self) -> Option<io::Result<ModuleInfo>> {
        if self.iter_started {
            if unsafe { Module32Next(self.snapshot.handle(), &mut self.temp_entry) } == FALSE {
                let last_error = io::Error::last_os_error();

                return if last_error.raw_os_error().unwrap() as u32 == ERROR_NO_MORE_FILES {
                    None
                } else {
                    Some(Err(last_error))
                };
            }
        } else {
            if unsafe { Module32First(self.snapshot.handle(), &mut self.temp_entry) } == FALSE {
                return Some(Err(io::Error::last_os_error()));
            }
            self.iter_started = true;
        }

        let cstr = unsafe { CStr::from_ptr(&self.temp_entry.szModule[0] as *const c_char) };
        Some(Ok(ModuleInfo {
            name: cstr.to_string_lossy().into_owned(),
            base_address: self.temp_entry.modBaseAddr,
            size: self.temp_entry.modBaseSize as usize,
        }))
    }
}
