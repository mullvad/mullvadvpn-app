#![allow(clippy::undocumented_unsafe_blocks)]

use mullvad_paths::log_dir;
use std::{
    borrow::Cow,
    ffi::c_void,
    fmt::Write,
    fs, io,
    os::windows::io::AsRawHandle,
    path::{Path, PathBuf},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};
use talpid_types::ErrorExt;
use talpid_windows::process::{ModuleEntry, ProcessSnapshot};
use winapi::vc::excpt::EXCEPTION_EXECUTE_HANDLER;
use windows_sys::Win32::{
    Foundation::HANDLE,
    System::{
        Diagnostics::{
            Debug::{
                MiniDumpNormal, MiniDumpWriteDump, SetUnhandledExceptionFilter, CONTEXT,
                EXCEPTION_POINTERS, EXCEPTION_RECORD, MINIDUMP_EXCEPTION_INFORMATION,
            },
            ToolHelp::TH32CS_SNAPMODULE,
        },
        Threading::{GetCurrentProcess, GetCurrentProcessId, GetCurrentThreadId},
    },
};

/// Minidump file name
const MINIDUMP_FILENAME: &str = "DAEMON.DMP";

/// Enable logging of unhandled SEH exceptions.
pub fn enable() {
    unsafe { SetUnhandledExceptionFilter(Some(logging_exception_filter)) };
}

#[derive(thiserror::Error, Debug)]
enum MinidumpError {
    #[error("Failed to create mini dump file")]
    CreateFileError(#[source] io::Error),
    #[error("Failed to produce mini dump and write it to disk")]
    GenerateError(#[source] io::Error),
}

fn generate_minidump(
    dump_file: &Path,
    exception_pointers: *const EXCEPTION_POINTERS,
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
        // We may treat *const as *mut if we wish?
        // https://internals.rust-lang.org/t/what-is-the-real-difference-between-const-t-and-mut-t-raw-pointers/6127/22
        ExceptionPointers: exception_pointers as *mut EXCEPTION_POINTERS,
        ClientPointers: 0,
    };

    // SAFETY: MiniDumpWriteDump is not thread-safe, so for this to be safe all threads need to be
    // synchronized. logging_exception_filter takes precaution by adding a thread-safe reentrancy
    // guard.
    if unsafe {
        MiniDumpWriteDump(
            process,
            process_id,
            handle as HANDLE,
            MiniDumpNormal,
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

unsafe extern "system" fn logging_exception_filter(info_ptr: *const EXCEPTION_POINTERS) -> i32 {
    // Guard against reentrancy, which can happen if this fault handler triggers another fault or
    // if multiple threads would fail "at the same time".
    // We have to take pre-cuation to synchronize all threads because the backing dbghelp Windows
    // API is *not* thread safe: https://learn.microsoft.com/en-us/windows/win32/dxtecharts/crash-dump-analysis#thread-safety.
    // We implicitly use it through for example MiniDumpWriteDump.
    static REENTRANCY_GUARD: AtomicBool = AtomicBool::new(false);
    if REENTRANCY_GUARD.swap(true, Ordering::SeqCst) {
        // We are already handling an error, so let someone else handle this error
        return EXCEPTION_EXECUTE_HANDLER;
    }

    if info_ptr.is_null() || !info_ptr.is_aligned() {
        // We can't properly handle this error, so let someone else handle this error
        return EXCEPTION_EXECUTE_HANDLER;
    }
    // SAFETY: We've explicitly checked for null pointer and
    // alignment. We assume that Windows gives us valid pointers, i.e. info points to valid data.
    let info: &EXCEPTION_POINTERS = unsafe { &*info_ptr };
    let record: &EXCEPTION_RECORD = unsafe { &*info.ExceptionRecord };
    let context: &CONTEXT = unsafe { &*info.ContextRecord };

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

    match generate_minidump(&dump_path, info_ptr) {
        Ok(()) => log::info!("Wrote Minidump to {}.", dump_path.to_string_lossy()),
        Err(e) => {
            log::error!(
                "{}",
                e.display_chain_with_msg("Failed to generate minidump")
            );
        }
    }

    // Log exception information
    let context_info = get_context_info(context);

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

#[cfg(target_arch = "aarch64")]
fn get_context_info(context: &CONTEXT) -> String {
    use windows_sys::Win32::System::Diagnostics::Debug::{
        CONTEXT_CONTROL_ARM64, CONTEXT_FLOATING_POINT_ARM64, CONTEXT_INTEGER_ARM64,
    };

    let mut context_str = "Context:\n".to_string();

    if context.ContextFlags & CONTEXT_CONTROL_ARM64 != 0 {
        writeln!(
            &mut context_str,
            "\n\tFp: {:#x?}\n \
             \tLr: {:#x?}\n \
             \tSp: {:#x?}\n \
             \tPc: {:#x?}\n \
             \tCpsr: {:#x?}",
            unsafe { context.Anonymous.Anonymous.Fp },
            unsafe { context.Anonymous.Anonymous.Lr },
            context.Sp,
            context.Pc,
            context.Cpsr
        )
        .unwrap();
    }

    if context.ContextFlags & CONTEXT_INTEGER_ARM64 != 0 {
        context_str.push('\n');
        for x in 0..=28 {
            writeln!(&mut context_str, "\tX{}: {:#x?}", x, unsafe {
                context.Anonymous.X[x]
            })
            .unwrap();
        }
    }
    if context.ContextFlags & CONTEXT_FLOATING_POINT_ARM64 != 0 {
        writeln!(
            &mut context_str,
            "\n\tFpcr: {:#x?}\n \
             \tFpsr: {:#x?}",
            context.Fpcr, context.Fpsr
        )
        .unwrap();
        for q in 0..=31 {
            writeln!(&mut context_str, "\tQ{}: {:#x?}", q, unsafe {
                context.V[q].B
            })
            .unwrap();
        }
    }

    context_str
}

#[cfg(target_arch = "x86_64")]
fn get_context_info(context: &CONTEXT) -> String {
    use windows_sys::Win32::System::Diagnostics::Debug::{
        CONTEXT_CONTROL_AMD64, CONTEXT_INTEGER_AMD64, CONTEXT_SEGMENTS_AMD64,
    };

    let mut context_str = "Context:\n".to_string();

    if context.ContextFlags & CONTEXT_CONTROL_AMD64 != 0 {
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

    if context.ContextFlags & CONTEXT_INTEGER_AMD64 != 0 {
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

    if context.ContextFlags & CONTEXT_SEGMENTS_AMD64 != 0 {
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
fn find_address_module(address: *mut c_void) -> io::Result<Option<ModuleEntry>> {
    let snap = ProcessSnapshot::new(TH32CS_SNAPMODULE, 0)?;

    for module in snap.modules() {
        let module = module?;
        let module_end_address = module.base_address.wrapping_add(module.size);
        if (address as *const u8) >= module.base_address
            && (address as *const u8) < module_end_address
        {
            return Ok(Some(module));
        }
    }

    Ok(None)
}
