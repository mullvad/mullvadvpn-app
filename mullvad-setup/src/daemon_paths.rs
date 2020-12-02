use std::{
    ffi::OsString,
    io, mem,
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
    ptr,
};
use widestring::{WideCStr, WideCString};
use winapi::{
    shared::{
        minwindef::{DWORD, FALSE},
        ntdef::LUID,
        winerror::{ERROR_NO_TOKEN, ERROR_SUCCESS, S_OK},
    },
    um::{
        combaseapi::CoTaskMemFree,
        handleapi::CloseHandle,
        knownfolders::{FOLDERID_LocalAppData, FOLDERID_System},
        processthreadsapi::{GetCurrentThread, OpenProcess, OpenProcessToken, OpenThreadToken},
        psapi::K32EnumProcesses,
        securitybaseapi::{AdjustTokenPrivileges, ImpersonateSelf, RevertToSelf},
        shlobj::{SHGetKnownFolderPath, KF_FLAG_DEFAULT},
        shtypes::KNOWNFOLDERID,
        winbase::{LookupPrivilegeValueW, QueryFullProcessImageNameW},
        winnt::{
            SecurityImpersonation, HANDLE, LUID_AND_ATTRIBUTES, PROCESS_QUERY_INFORMATION, PWSTR,
            SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_DUPLICATE, TOKEN_IMPERSONATE,
            TOKEN_PRIVILEGES, TOKEN_QUERY, TOKEN_READ,
        },
    },
};

pub fn get_mullvad_daemon_settings_path() -> io::Result<PathBuf> {
    get_system_service_known_folder(FOLDERID_LocalAppData)
        .map(|settings| settings.join(mullvad_paths::PRODUCT_NAME))
}


/// Get local AppData path for the system service user. Requires elevated privileges to work.
/// Useful for deducing the config path for the daemon on Windows when running as a user that
/// isn't the system service.
fn get_system_service_known_folder(known_folder_id: KNOWNFOLDERID) -> std::io::Result<PathBuf> {
    let system_debug_priv = WideCString::from_str("SeDebugPrivilege").unwrap();

    adjust_current_thread_token_privilege(&system_debug_priv, true)?;
    let known_folder: io::Result<PathBuf> = (|| {
        let mut lsass_path =
            get_known_folder_path(FOLDERID_System, KF_FLAG_DEFAULT, ptr::null_mut())?;
        lsass_path.push("lsass.exe");

        let lsass_pid = get_running_process_id_from_name(&lsass_path)?;
        let lsass_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, lsass_pid) };
        if lsass_handle.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Failed to open process {:?} to query information",
                    lsass_handle
                ),
            ));
        }

        let mut lsass_token = ptr::null_mut();
        let status = unsafe {
            OpenProcessToken(
                lsass_handle,
                TOKEN_READ | TOKEN_IMPERSONATE | TOKEN_DUPLICATE,
                &mut lsass_token,
            )
        };
        unsafe { CloseHandle(lsass_handle) };
        if status == FALSE {
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
        eprintln!("Failed to drop system privileges: {}", err);
    }
    if unsafe { RevertToSelf() } == FALSE {
        return Err(io::Error::last_os_error());
    }

    known_folder
}

fn adjust_current_thread_token_privilege(
    privilege: &WideCString,
    enable: bool,
) -> std::io::Result<()> {
    let mut token_handle: HANDLE = ptr::null_mut();
    if unsafe {
        OpenThreadToken(
            GetCurrentThread(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            FALSE,
            &mut token_handle,
        )
    } == FALSE
    {
        let thread_token_error = std::io::Error::last_os_error();
        if thread_token_error.raw_os_error() == Some(ERROR_NO_TOKEN as i32) {
            if unsafe { ImpersonateSelf(SecurityImpersonation) } == FALSE {
                return Err(std::io::Error::last_os_error());
            }

            if unsafe {
                OpenThreadToken(
                    GetCurrentThread(),
                    TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                    FALSE,
                    &mut token_handle,
                )
            } == FALSE
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
    privilege: &WideCString,
    enable: bool,
) -> std::io::Result<()> {
    let mut privilege_luid: LUID = Default::default();

    if unsafe { LookupPrivilegeValueW(ptr::null(), privilege.as_ptr(), &mut privilege_luid) }
        == FALSE
    {
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
            FALSE,
            &mut privileges,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    // Terrible interface.
    //  Odd 2018
    let last_error = std::io::Error::last_os_error();
    if result == FALSE || last_error.raw_os_error() != Some(ERROR_SUCCESS as i32) {
        return Err(last_error);
    }

    Ok(())
}

fn get_known_folder_path(
    folder_id: KNOWNFOLDERID,
    flags: DWORD,
    user_token: HANDLE,
) -> std::io::Result<PathBuf> {
    let mut folder_path: PWSTR = ptr::null_mut();
    let path = unsafe {
        let status = SHGetKnownFolderPath(&folder_id, flags, user_token, &mut folder_path);
        if status != S_OK || folder_path.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Can't find known folder {:?}", &folder_id),
            ));
        }
        WideCStr::from_ptr_str(folder_path)
    };

    let result = Ok(path.to_ustring().to_os_string().into());
    unsafe { CoTaskMemFree(path.as_ptr() as *const _ as *mut _) };
    result
}

/// Find the PID of a process with the given image path. In case of multiple processes matching
/// the path, the first one found to match the path will be returned - the ordering of PIDs is
/// determined by `K32EnumProcesses`.
fn get_running_process_id_from_name(target_name: &Path) -> io::Result<DWORD> {
    let mut num_procs: u32 = 2048;
    let mut pid_buffer = vec![];
    let canonical_target = target_name
        .canonicalize()
        .unwrap_or(target_name.to_path_buf());

    pid_buffer.resize(num_procs as usize, 0);
    let bytes_available = num_procs * (mem::size_of::<DWORD>() as u32);
    let mut bytes_written = 0;
    if unsafe { K32EnumProcesses(pid_buffer.as_mut_ptr(), bytes_available, &mut bytes_written) }
        == FALSE
    {
        return Err(io::Error::last_os_error());
    }

    num_procs = bytes_written / (mem::size_of::<DWORD>() as u32);
    pid_buffer.resize(num_procs as usize, 0);


    for process in pid_buffer {
        let process_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, process) };
        if process_handle.is_null() {
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
