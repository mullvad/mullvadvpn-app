use std::{
    ffi::OsString,
    io, mem,
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
    ptr,
};
use widestring::{WideCStr, WideCString};
use windows_sys::{
    core::{GUID, PWSTR},
    Win32::{
        Foundation::{CloseHandle, ERROR_NO_TOKEN, ERROR_SUCCESS, HANDLE, LUID, S_OK},
        Security::{
            AdjustTokenPrivileges, ImpersonateSelf, LookupPrivilegeValueW, RevertToSelf,
            SecurityImpersonation, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
            TOKEN_ADJUST_PRIVILEGES, TOKEN_DUPLICATE, TOKEN_IMPERSONATE, TOKEN_PRIVILEGES,
            TOKEN_QUERY,
        },
        System::{
            Com::CoTaskMemFree,
            ProcessStatus::K32EnumProcesses,
            SystemServices::GENERIC_READ,
            Threading::{
                GetCurrentThread, OpenProcess, OpenProcessToken, OpenThreadToken,
                QueryFullProcessImageNameW, PROCESS_QUERY_INFORMATION,
            },
        },
        UI::Shell::{
            FOLDERID_LocalAppData, FOLDERID_System, SHGetKnownFolderPath, KF_FLAG_DEFAULT,
        },
    },
};

pub(crate) fn get_system_service_appdata() -> io::Result<PathBuf> {
    get_system_service_known_folder(&FOLDERID_LocalAppData)
}

/// Get local AppData path for the system service user. Requires elevated privileges to work.
/// Useful for deducing the config path for the daemon on Windows when running as a user that
/// isn't the system service.
fn get_system_service_known_folder(known_folder_id: *const GUID) -> std::io::Result<PathBuf> {
    let system_debug_priv = WideCString::from_str("SeDebugPrivilege").unwrap();

    adjust_current_thread_token_privilege(&system_debug_priv, true)?;
    let known_folder: io::Result<PathBuf> = (|| {
        let mut lsass_path = get_known_folder_path(&FOLDERID_System, KF_FLAG_DEFAULT, 0)?;
        lsass_path.push("lsass.exe");

        let lsass_pid = get_running_process_id_from_name(&lsass_path)?;
        let lsass_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, lsass_pid) };
        if lsass_handle == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Failed to open process {:?} to query information",
                    lsass_handle
                ),
            ));
        }

        let mut lsass_token: HANDLE = 0;
        let status = unsafe {
            OpenProcessToken(
                lsass_handle,
                GENERIC_READ | TOKEN_IMPERSONATE | TOKEN_DUPLICATE,
                &mut lsass_token,
            )
        };
        unsafe { CloseHandle(lsass_handle) };
        if status == 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Failed to open process token, failure code {}", status),
            ));
        }

        let known_folder = get_known_folder_path(known_folder_id, KF_FLAG_DEFAULT, lsass_token);
        unsafe { CloseHandle(lsass_token) };

        known_folder
    })();

    if let Err(err) = adjust_current_thread_token_privilege(&system_debug_priv, false) {
        eprintln!("Failed to drop SeDebugPrivilege: {}", err);
    }
    if unsafe { RevertToSelf() } == 0 {
        return Err(io::Error::last_os_error());
    }

    known_folder
}

fn adjust_current_thread_token_privilege(
    privilege: &WideCStr,
    enable: bool,
) -> std::io::Result<()> {
    let mut token_handle: HANDLE = 0;
    if unsafe {
        OpenThreadToken(
            GetCurrentThread(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            0,
            &mut token_handle,
        )
    } == 0
    {
        let thread_token_error = std::io::Error::last_os_error();
        if thread_token_error.raw_os_error() == Some(ERROR_NO_TOKEN as i32) {
            if unsafe { ImpersonateSelf(SecurityImpersonation) } == 0 {
                return Err(std::io::Error::last_os_error());
            }

            if unsafe {
                OpenThreadToken(
                    GetCurrentThread(),
                    TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                    0,
                    &mut token_handle,
                )
            } == 0
            {
                return Err(std::io::Error::last_os_error());
            }
        } else {
            return Err(thread_token_error);
        }
    }

    let result = adjust_token_privilege(token_handle, privilege, enable);
    unsafe { CloseHandle(token_handle) };
    result
}

fn adjust_token_privilege(
    token_handle: HANDLE,
    privilege: &WideCStr,
    enable: bool,
) -> std::io::Result<()> {
    let mut privilege_luid: LUID = unsafe { mem::zeroed() };

    if unsafe { LookupPrivilegeValueW(ptr::null(), privilege.as_ptr(), &mut privilege_luid) } == 0 {
        return Err(std::io::Error::last_os_error());
    }

    let mut privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: privilege_luid,
            Attributes: if enable { SE_PRIVILEGE_ENABLED } else { 0 },
        }],
    };
    let result = unsafe {
        AdjustTokenPrivileges(
            token_handle,
            0,
            &mut privileges,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    // Terrible interface.
    //  Odd 2018
    let last_error = std::io::Error::last_os_error();
    if result == 0 || last_error.raw_os_error() != Some(ERROR_SUCCESS as i32) {
        return Err(last_error);
    }

    Ok(())
}

fn get_known_folder_path(
    folder_id: *const GUID,
    flags: i32,
    user_token: HANDLE,
) -> std::io::Result<PathBuf> {
    let mut folder_path: PWSTR = ptr::null_mut();
    let status = unsafe { SHGetKnownFolderPath(folder_id, flags, user_token, &mut folder_path) };
    let result = if status == S_OK {
        let path = unsafe { WideCStr::from_ptr_str(folder_path) };
        Ok(path.to_ustring().to_os_string().into())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Can't find known folder {:?}", &folder_id),
        ))
    };

    unsafe { CoTaskMemFree(folder_path as *mut _) };
    result
}

/// Find the PID of a process with the given image path. In case of multiple processes matching
/// the path, the first one found to match the path will be returned - the ordering of PIDs is
/// determined by `K32EnumProcesses`.
fn get_running_process_id_from_name(target_name: &Path) -> io::Result<u32> {
    let mut num_procs: u32 = 2048;
    let mut pid_buffer = vec![];
    let canonical_target = target_name
        .canonicalize()
        .unwrap_or(target_name.to_path_buf());

    pid_buffer.resize(num_procs as usize, 0);
    let bytes_available = num_procs * (mem::size_of::<u32>() as u32);
    let mut bytes_written = 0;
    if unsafe { K32EnumProcesses(pid_buffer.as_mut_ptr(), bytes_available, &mut bytes_written) }
        == 0
    {
        return Err(io::Error::last_os_error());
    }

    num_procs = bytes_written / (mem::size_of::<u32>() as u32);
    pid_buffer.resize(num_procs as usize, 0);

    for process in pid_buffer {
        let process_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, process) };
        if process_handle == 0 {
            eprintln!(
                "Failed to open process {}: {}",
                process,
                io::Error::last_os_error()
            );
            continue;
        }

        let mut process_name = vec![0u16; 512];
        let mut process_name_size = process_name.len() as u32;

        let status = unsafe {
            QueryFullProcessImageNameW(
                process_handle,
                0,
                process_name.as_mut_ptr(),
                &mut process_name_size,
            )
        };
        unsafe { CloseHandle(process_handle) };

        if 0 == status || process_name_size == 0 {
            continue;
        };

        process_name.resize(process_name_size as usize, 0u16);

        let process_path = PathBuf::from(OsString::from_wide(&process_name));
        let canonical_process_path = process_path.canonicalize().unwrap_or(process_path);

        if canonical_target == canonical_process_path {
            return Ok(process);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Process ID for {} not found", target_name.to_string_lossy()),
    ))
}
