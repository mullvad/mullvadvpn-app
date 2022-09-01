use libc::c_void;
use mullvad_paths::log_dir;
use std::{
    borrow::Cow,
    ffi::CStr,
    fmt::Write,
    fs, io, mem,
    os::{raw::c_char, windows::io::AsRawHandle},
    path::{Path, PathBuf},
    ptr,
};
use talpid_types::ErrorExt;
use winapi::{
    um::winnt::{CONTEXT_CONTROL, CONTEXT_INTEGER, CONTEXT_SEGMENTS},
    vc::excpt::EXCEPTION_EXECUTE_HANDLER,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, BOOL, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE},
    System::{
        Diagnostics::{
            Debug::{SetUnhandledExceptionFilter, CONTEXT, EXCEPTION_POINTERS, EXCEPTION_RECORD},
            ToolHelp::{
                CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32,
                TH32CS_SNAPMODULE,
            },
        },
        Threading::{GetCurrentProcess, GetCurrentProcessId, GetCurrentThreadId},
    },
};

/// Minidump file name
const MINIDUMP_FILENAME: &'static str = "DAEMON.DMP";

#[repr(C)]
#[allow(dead_code)]
enum MINIDUMP_TYPE {
    MiniDumpNormal = 0,
    // Add missing values as needed
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
#[allow(non_snake_case)]
struct MINIDUMP_EXCEPTION_INFORMATION {
    ThreadId: u32,
    ExceptionPointers: *const EXCEPTION_POINTERS,
    ClientPointers: BOOL,
}

#[link(name = "dbghelp")]
extern "system" {
    /// Store exception information, stack trace, etc. in a file.
    fn MiniDumpWriteDump(
        hProcess: HANDLE,
        ProcessId: u32,
        hFile: HANDLE,
        DumpType: MINIDUMP_TYPE,
        ExceptionParam: *const MINIDUMP_EXCEPTION_INFORMATION,

        // Add types as needed:
        UserStreamParam: *const c_void,
        CallbackParam: *const c_void,
    ) -> BOOL;
}

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
enum MinidumpError {
    #[error(display = "Failed to create mini dump file")]
    CreateFileError(#[error(source)] io::Error),
    #[error(display = "Failed to produce mini dump and write it to disk")]
    GenerateError(#[error(source)] io::Error),
}

fn generate_minidump(
    dump_file: &Path,
    exception_pointers: &EXCEPTION_POINTERS,
) -> Result<(), MinidumpError> {
    // Open/create dump file
    let handle_rs = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dump_file)
        .map_err(MinidumpError::CreateFileError)?;
    let handle = handle_rs.as_raw_handle();

    // Generate minidump
    let process = unsafe { GetCurrentProcess() };
    let process_id = unsafe { GetCurrentProcessId() };
    let thread_id = unsafe { GetCurrentThreadId() };

    let exception_parameters = MINIDUMP_EXCEPTION_INFORMATION {
        ThreadId: thread_id,
        ExceptionPointers: exception_pointers,
        ClientPointers: 0,
    };

    if unsafe {
        MiniDumpWriteDump(
            process,
            process_id,
            handle as HANDLE,
            MINIDUMP_TYPE::MiniDumpNormal,
            &exception_parameters,
            ptr::null(),
            ptr::null(),
        )
    } == 0
    {
        return Err(MinidumpError::GenerateError(io::Error::last_os_error()));
    }

    Ok(())
}

/// Enable logging of unhandled SEH exceptions.
pub fn enable() {
    unsafe { SetUnhandledExceptionFilter(Some(logging_exception_filter)) };
}

fn exception_code_to_string(value: &EXCEPTION_RECORD) -> Option<Cow<'_, str>> {
    use windows_sys::Win32::Foundation::*;
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

unsafe extern "system" fn logging_exception_filter(info: *const EXCEPTION_POINTERS) -> i32 {
    // SAFETY: Windows gives us valid pointers
    let info: &EXCEPTION_POINTERS = &*info;
    let record: &EXCEPTION_RECORD = &*info.ExceptionRecord;

    // Generate minidump
    let dump_path = match log_dir() {
        Ok(dir) => dir.join(MINIDUMP_FILENAME),
        _ => {
            log::warn!("Failed to obtain log path. Using working directory.");
            let mut buf = PathBuf::new();
            buf.push(MINIDUMP_FILENAME);
            buf
        }
    };

    match generate_minidump(&dump_path, &info) {
        Ok(()) => log::info!("Wrote Minidump to {}.", dump_path.to_string_lossy()),
        Err(e) => {
            log::error!(
                "{}",
                e.display_chain_with_msg("Failed to generate minidump")
            );
        }
    }

    // Log exception information
    let context_info = get_context_info(&*info.ContextRecord);

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
        if (address as *const u8) >= module.base_address
            && (address as *const u8) < module_end_address
        {
            return Ok(Some(module));
        }
    }

    Ok(None)
}

struct ModuleInfo {
    name: String,
    base_address: *const u8,
    size: usize,
}

struct ProcessSnapshot {
    handle: HANDLE,
}

impl ProcessSnapshot {
    fn new(flags: u32, process_id: u32) -> io::Result<ProcessSnapshot> {
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
            if unsafe { Module32Next(self.snapshot.handle(), &mut self.temp_entry) } == 0 {
                let last_error = io::Error::last_os_error();

                return if last_error.raw_os_error().unwrap() as u32 == ERROR_NO_MORE_FILES {
                    None
                } else {
                    Some(Err(last_error))
                };
            }
        } else {
            if unsafe { Module32First(self.snapshot.handle(), &mut self.temp_entry) } == 0 {
                return Some(Err(io::Error::last_os_error()));
            }
            self.iter_started = true;
        }

        let cstr_ref = &self.temp_entry.szModule[0];
        let cstr = unsafe { CStr::from_ptr(cstr_ref as *const u8 as *const c_char) };
        Some(Ok(ModuleInfo {
            name: cstr.to_string_lossy().into_owned(),
            base_address: self.temp_entry.modBaseAddr,
            size: self.temp_entry.modBaseSize as usize,
        }))
    }
}
