//! Functionality for identifying processes with open file handles.
//!
//! Updating the app fails pretty often because some other process has an open handle to the install
//! directory. This module helps users figure out what those are.
//!
//! [Restart Manager] is intended for this exact purpose, but since it does not work for
//! directories, only files, we instead rely on undocumented arguments to
//! `NtQuerySystemInformation`.
//!
//! [Restart Manager]: https://learn.microsoft.com/en-us/windows/win32/api/restartmanager/nf-restartmanager-rmstartsession

use std::ffi::{c_uchar, c_ulong, c_ushort};
use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::{Path, PathBuf};
use windows_sys::Wdk::Foundation::{OBJECT_INFORMATION_CLASS, ObjectTypeInformation};
use windows_sys::Wdk::System::SystemInformation::SYSTEM_INFORMATION_CLASS;
use windows_sys::Win32::Foundation::{
    DuplicateHandle, HANDLE, NTSTATUS, STATUS_INFO_LENGTH_MISMATCH, STATUS_SUCCESS,
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_NAME_NORMALIZED, FILE_TYPE_DISK, GetFileType, GetFinalPathNameByHandleW,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, PROCESS_DUP_HANDLE, PROCESS_QUERY_INFORMATION,
    QueryFullProcessImageNameW,
};
use windows_sys::Win32::System::WindowsProgramming::PUBLIC_OBJECT_TYPE_INFORMATION;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{IDRETRY, MB_ICONWARNING, MB_OK, MessageBoxW};

use crate::MAX_PATH_SIZE;

/// Used with NtQuerySystemInformation to retrieve process handles.
/// There is no official documentation for this class.
/// See https://www.geoffchappell.com/studies/windows/km/ntoskrnl/api/ex/sysinfo/handle.htm
#[allow(non_upper_case_globals)]
const SystemHandleInformation: SYSTEM_INFORMATION_CLASS = 16;

/// Returned by NtQuerySystemInformation when SystemHandleInformation is queried.
/// There is no official documentation for this structure.
/// See https://www.geoffchappell.com/studies/windows/km/ntoskrnl/api/ex/sysinfo/handle.htm
#[repr(C)]
#[allow(non_camel_case_types)]
struct SYSTEM_HANDLE_INFORMATION {
    number_of_handles: c_ulong,
    // NOTE: This is variable-length. Traditionally a 1-sized array in Windows C headers
    handles: [SYSTEM_HANDLE_TABLE_ENTRY_INFO; 1],
}

/// Returned by NtQuerySystemInformation when SystemHandleInformation is queried.
/// There is no official documentation for this structure.
/// See https://www.geoffchappell.com/studies/windows/km/ntoskrnl/api/ex/sysinfo/handle_table_entry.htm
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
struct SYSTEM_HANDLE_TABLE_ENTRY_INFO {
    process_id: c_ushort,
    creator_back_trace_index: c_ushort,
    object_type_index: c_uchar,
    handle_attributes: c_uchar,
    handle_value: c_ushort,
    object: *mut std::ffi::c_void,
    granted_access: c_ulong,
}

/// Show a message box asking the user to close processes that have handles in `dir_path`,
/// including the offending processes. If none are found, no message is shown.
pub fn ask_terminate_processes(dir_path: &impl AsRef<Path>) -> io::Result<()> {
    let conflicting_processes = file_handles_for_dir(dir_path)?;
    ask_terminate_processes_inner(&conflicting_processes);
    Ok(())
}

fn ask_terminate_processes_inner(processes: &[ConflictingProcess]) {
    if processes.is_empty() {
        return;
    }

    let mut s = String::new();
    for p in processes {
        s.push_str(&format!("{} (pid: {})\n", p.image_name, p.pid,));
    }

    let message = format!(
        "Found {} process(es) with handles to the install directory:\n\n
{}\n
Please close these processes before continuing.",
        processes.len(),
        s,
    );
    let title = "Failed to remove old installation";

    let mut message_wide: Vec<u16> = message.encode_utf16().collect();
    message_wide.push(0);

    let mut title_wide: Vec<u16> = title.encode_utf16().collect();
    title_wide.push(0);

    // SAFETY: Pointers point to valid and null-terminated strings
    unsafe {
        MessageBoxW(
            GetActiveWindow(),
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_OK | MB_ICONWARNING,
        )
    }
}

/// Return the type of handle that `handle` is using [`NtQueryObject`].
/// The type of handle is returned as a string.
///
/// If the type is unknown, or if any error occurs, this returns `None`.
///
/// [`NtQueryObject`]: https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryobject
fn get_handle_type(handle: HANDLE) -> Option<String> {
    // SAFETY: 'ntdll' is already loaded, so comments about initialization/cleanup routines do not
    // apply.
    let ntdll = unsafe { libloading::Library::new("ntdll.dll") }.ok()?;

    let nt_query_object: libloading::Symbol<
        '_,
        unsafe extern "system" fn(
            handle: HANDLE,
            object_information_class: OBJECT_INFORMATION_CLASS,
            object_information: *mut std::ffi::c_void,
            object_information_length: u32,
            return_length: *mut u32,
        ) -> NTSTATUS,
    > =
    // SAFETY: The signature is correct.
    // See https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryobject
    unsafe { ntdll.get(b"NtQueryObject\0") }.ok()?;

    let mut buffer = vec![0u8; 4096];
    let mut return_length: u32 = 0;

    // SAFETY: The buffer is valid and can hold up to `buffer.len()` bytes.
    // See https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryobject
    let status = unsafe {
        nt_query_object(
            handle,
            ObjectTypeInformation,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            &raw mut return_length,
        )
    };

    if status != STATUS_SUCCESS {
        return None;
    }

    let type_info_ptr = buffer.as_ptr() as *const PUBLIC_OBJECT_TYPE_INFORMATION;
    debug_assert!(type_info_ptr.is_aligned());
    // SAFETY: The buffer contains a PUBLIC_OBJECT_TYPE_INFORMATION struct.
    let type_info = unsafe { &*type_info_ptr };

    if type_info.TypeName.Length == 0 || type_info.TypeName.Buffer.is_null() {
        return None;
    }

    // SAFETY: `TypeName` is a valid UTF-16 string.
    let slice = unsafe {
        std::slice::from_raw_parts(
            type_info.TypeName.Buffer,
            (type_info.TypeName.Length / 2) as usize,
        )
    };

    Some(String::from_utf16_lossy(slice))
}

fn dup_handle(process_handle: HANDLE, handle_value: u16) -> Option<OwnedHandle> {
    let mut dup_handle: HANDLE = std::ptr::null_mut();

    // SAFETY: This function returns a valid handle if the process exists.
    let status = unsafe {
        DuplicateHandle(
            process_handle,
            handle_value as HANDLE,
            GetCurrentProcess(),
            &raw mut dup_handle,
            0,
            0,
            0,
        )
    };

    if status == 0 {
        return None;
    }

    // SAFETY: `DuplicateHandle` succeeded and returned a valid handle.
    Some(unsafe { OwnedHandle::from_raw_handle(dup_handle) })
}

/// Return the process image name for the given process ID.
/// If this fails for any reason, this returns "PID <pid>" instead.
fn query_process_image_name(pid: u32) -> String {
    let Some(process_handle) = open_process(pid) else {
        return format!("PID {}", pid);
    };

    let mut buffer = vec![0u16; MAX_PATH_SIZE];
    let mut size = buffer.len() as u32;

    // SAFETY: The handle is valid and `buffer` has a size of `size`.
    if unsafe {
        QueryFullProcessImageNameW(
            process_handle.as_raw_handle(),
            0,
            buffer.as_mut_ptr(),
            &raw mut size,
        )
    } != 0
    {
        // `size` receives the actual length of the path, so truncate to that
        buffer.truncate(size as usize);
        if let Ok(path) = String::from_utf16(&buffer).map(PathBuf::from) {
            // Extract just the filename if possible
            if let Some(filename) = path.file_name() {
                return filename.to_string_lossy().to_string();
            }
            return path.to_string_lossy().to_string();
        }
    }

    format!("PID {}", pid)
}

/// List of all file handles on the system
///
/// To do this, we use `NtQuerySystemInformation` with the undocumented `SYSTEM_HANDLE_INFORMATION`
/// class, which returns all file handles and their processes.
///
/// # Note
///
/// This does not appear to include handles owned by (at least some) services.
///
/// We use run-time dynamic linking as recommended by the [documentation]. (They also recommend not
/// using the function at all, but it seems like the best we can do.)
///
/// [documentation]: https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntquerysysteminformation
fn get_system_handles() -> io::Result<Vec<SYSTEM_HANDLE_TABLE_ENTRY_INFO>> {
    const INITIAL_INFO_BUF_SIZE: usize = 2 * 1024 * 1024;

    let mut info_buf = vec![0u8; INITIAL_INFO_BUF_SIZE];

    // SAFETY: 'ntdll' is already loaded, so comments about initialization/cleanup routines do not
    // apply.
    let ntdll = unsafe { libloading::Library::new("ntdll.dll") }
        .map_err(|_err| io::Error::other("Failed to load ntdll.dll"))?;

    let nt_query_system_information: libloading::Symbol<
        '_,
        unsafe extern "system" fn(
            system_information_class: SYSTEM_INFORMATION_CLASS,
            system_information: *mut std::ffi::c_void,
            system_information_length: c_ulong,
            return_length: *mut c_ulong,
        ) -> NTSTATUS,
    > =
    // SAFETY: The signature is correct.
    // See https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntquerysysteminformation
    unsafe { ntdll.get(b"NtQuerySystemInformation\0") }
        .map_err(|_err| io::Error::other("Failed to load ntdll.dll"))?;

    // Query system handle information with increasing buffer size
    loop {
        let mut return_length: u32 = 0;

        // SAFETY: `info_buf` is a valid pointer to a buffer of size `info_buf.len()`.
        let status = unsafe {
            nt_query_system_information(
                SystemHandleInformation,
                info_buf.as_mut_ptr() as *mut _,
                info_buf.len() as u32,
                &raw mut return_length,
            )
        };

        if status == STATUS_SUCCESS {
            break;
        }
        if status == STATUS_INFO_LENGTH_MISMATCH {
            // Resize buffer to the required length
            info_buf.resize(return_length as usize, 0);
        } else {
            return Err(io::Error::from_raw_os_error(status));
        }
    }

    let handle_info_ptr = info_buf.as_ptr() as *const SYSTEM_HANDLE_INFORMATION;
    debug_assert!(handle_info_ptr.is_aligned());
    // SAFETY: The buffer contains a SYSTEM_HANDLE_INFORMATION struct.
    let handle_info = unsafe { &*handle_info_ptr };
    let handle_count = handle_info.number_of_handles as usize;

    let handles_ptr = handle_info.handles.as_ptr();
    debug_assert!(handles_ptr.is_aligned());
    // SAFETY: The buffer contains an array of SYSTEM_HANDLE_TABLE_ENTRY_INFO structs.
    let handles = unsafe { std::slice::from_raw_parts(handles_ptr, handle_count) };

    Ok(handles.to_vec())
}

/// Return a handle to the given process ID
///
/// We request access to duplicate handles and querying
fn open_process(pid: u32) -> Option<OwnedHandle> {
    // SAFETY: This function returns a valid handle if the process exists.
    let process_handle =
        unsafe { OpenProcess(PROCESS_DUP_HANDLE | PROCESS_QUERY_INFORMATION, 0, pid) };
    if process_handle.is_null() {
        return None;
    }
    // SAFETY: `process_handle` is a valid handle
    Some(unsafe { OwnedHandle::from_raw_handle(process_handle) })
}

/// Return final path given a handle
fn get_final_path_name(handle: &impl AsRawHandle) -> Result<String, io::Error> {
    let mut path_buffer = vec![0u16; MAX_PATH_SIZE];

    loop {
        // SAFETY: `path_buffer` is a valid buffer for `GetFinalPathNameByHandleW`
        let result = unsafe {
            GetFinalPathNameByHandleW(
                handle.as_raw_handle(),
                path_buffer.as_mut_ptr(),
                path_buffer.len() as u32,
                FILE_NAME_NORMALIZED,
            )
        };

        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        if result >= path_buffer.len() as u32 {
            // Need a bigger buffer
            path_buffer.resize(result as usize, 0);
            continue;
        }

        path_buffer.truncate(result as usize);
        let path = String::from_utf16_lossy(&path_buffer);
        break Ok(path);
    }
}

struct ConflictingProcess {
    pid: u32,
    image_name: String,
}

/// Identify all processes containing file handles in the given `target_dir` directory
fn file_handles_for_dir(target_dir: impl AsRef<Path>) -> io::Result<Vec<ConflictingProcess>> {
    let handles = get_system_handles()?;

    let target_dir_normalized = {
        let normalized = target_dir.as_ref().to_string_lossy().replace('/', "\\");
        normalized.to_uppercase()
    };

    let mut in_use_processes = vec![];

    for handle in handles.iter() {
        // Open the process that owns the file handle
        let Some(process_handle) = open_process(handle.process_id as u32) else {
            continue;
        };

        // Duplicate its handle into our process
        let Some(dup_h) = dup_handle(process_handle.as_raw_handle(), handle.handle_value) else {
            continue;
        };

        // Get the type of the handle and skip non-file handles
        let Some(handle_type) = get_handle_type(dup_h.as_raw_handle()) else {
            continue;
        };
        if handle_type != "File" {
            continue;
        }

        // Skip irrelevant file handle types, such as named pipes
        // This is crucial because it turns out that `NtQueryObject` can cause deadlocks for certain
        // types of files.
        // SAFETY: `dup_h` is a valid file handle.
        if unsafe { GetFileType(dup_h.as_raw_handle()) } != FILE_TYPE_DISK {
            continue;
        }

        // Get file path and check if it matches the target directory
        let Ok(path) = get_final_path_name(&dup_h) else {
            continue;
        };

        let matches_filter = {
            let path = path.to_uppercase();
            let normalized_path = path.strip_prefix(r"\\?\").unwrap_or(&path);
            // Match if path equals target OR starts with target\ (subdirectory)
            normalized_path == target_dir_normalized.as_str()
                || normalized_path.starts_with(&format!("{}\\", target_dir_normalized))
        };

        if matches_filter {
            let pid = handle.process_id as u32;
            let image_name = query_process_image_name(pid);

            in_use_processes.push(ConflictingProcess { pid, image_name });
        }
    }

    Ok(in_use_processes)
}
