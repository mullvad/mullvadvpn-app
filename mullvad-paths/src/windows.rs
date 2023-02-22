use std::{io, mem, path::PathBuf, ptr};
use widestring::{WideCStr, WideCString};
use windows_sys::{
    core::{GUID, PWSTR},
    Win32::{
        Foundation::{
            CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_TOKEN, ERROR_SUCCESS, HANDLE, LUID,
            S_OK,
        },
        Security::{
            AdjustTokenPrivileges, CreateWellKnownSid, EqualSid, GetTokenInformation,
            ImpersonateSelf, LookupPrivilegeValueW, RevertToSelf, SecurityImpersonation, TokenUser,
            WinLocalSystemSid, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES,
            TOKEN_DUPLICATE, TOKEN_IMPERSONATE, TOKEN_PRIVILEGES, TOKEN_QUERY, TOKEN_USER,
        },
        Storage::FileSystem::MAX_SID_SIZE,
        System::{
            Com::CoTaskMemFree,
            ProcessStatus::K32EnumProcesses,
            SystemServices::GENERIC_READ,
            Threading::{
                GetCurrentThread, OpenProcess, OpenProcessToken, OpenThreadToken,
                PROCESS_QUERY_INFORMATION,
            },
        },
        UI::Shell::{
            FOLDERID_LocalAppData, FOLDERID_System, SHGetKnownFolderPath, KF_FLAG_DEFAULT,
        },
    },
};

struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub fn get_system_service_appdata() -> io::Result<PathBuf> {
    let result = get_appdata_as_system_user()
        .or_else(|error| {
            log::error!("get_appdata_as_system_user failed: {error}");
            get_appdata_as_admin()
        })
        .or_else(|error| {
            log::error!("get_appdata_as_admin failed: {error}");
            infer_appdata_from_system()
        });
    if unsafe { RevertToSelf() } == 0 {
        log::error!("RevertToSelf failed: {}", io::Error::last_os_error());
    }
    result
}

/// Get local AppData path if the current user is the system service user.
fn get_appdata_as_system_user() -> io::Result<PathBuf> {
    let current_token = Handle(get_current_thread_token()?);
    let current_user_is_system = is_local_system_user_token(current_token.0);

    if current_user_is_system.map_err(|error| {
        log::error!("is_local_system_user_token failed: {error}");
        error
    })? {
        log::trace!("Is system user");
        return get_known_folder_path(&FOLDERID_LocalAppData, KF_FLAG_DEFAULT, 0);
    }

    log::trace!("Is not system user");
    Err(io::Error::new(
        io::ErrorKind::Other,
        "current user is not SYSTEM",
    ))
}

/// Get local AppData path for the system service user. Requires elevated privileges to work.
/// Useful for deducing the config path for the daemon on Windows when running as a user that
/// isn't the system service.
fn get_appdata_as_admin() -> std::io::Result<PathBuf> {
    let system_debug_priv = WideCString::from_str("SeDebugPrivilege").unwrap();
    let current_token = Handle(get_current_thread_token()?);
    adjust_token_privilege(current_token.0, &system_debug_priv, true)?;

    let known_path = find_process(|process_handle| {
        let mut process_token = 0;

        let status = unsafe {
            OpenProcessToken(
                process_handle,
                GENERIC_READ | TOKEN_IMPERSONATE | TOKEN_DUPLICATE,
                &mut process_token,
            )
        };
        if status == 0 {
            log::error!(
                "Failed to open process token: {}",
                io::Error::last_os_error()
            );
            return None;
        }

        let result = if let Ok(true) = is_local_system_user_token(process_token) {
            match get_known_folder_path(&FOLDERID_LocalAppData, KF_FLAG_DEFAULT, process_token) {
                Ok(path) => Some(Ok(path)),
                Err(error) => {
                    log::error!("Failed to obtain folder path: {}", error);
                    None
                }
            }
        } else {
            None
        };
        unsafe { CloseHandle(process_token) };

        result
    });

    if let Err(err) = adjust_token_privilege(current_token.0, &system_debug_priv, false) {
        log::error!("Failed to drop SeDebugPrivilege: {}", err);
    }

    known_path
}

/// If all else fails, infer the AppData path from the system directory.
fn infer_appdata_from_system() -> io::Result<PathBuf> {
    let mut sysdir = get_known_folder_path(&FOLDERID_System, KF_FLAG_DEFAULT, 0)?;
    sysdir.extend(["config", "systemprofile", "AppData", "Local"]);
    Ok(sysdir)
}

fn get_current_thread_token() -> std::io::Result<HANDLE> {
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

    Ok(token_handle)
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

/// Enumerate over all processes until `handle_process` returns a result or until there are
/// no more processes left. In the latter case, an error is returned.
fn find_process<T>(handle_process: impl Fn(HANDLE) -> Option<io::Result<T>>) -> io::Result<T> {
    let mut pid_buffer = vec![0u32; 2048];
    let mut num_procs: u32 = u32::try_from(pid_buffer.len()).unwrap();

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
            log::error!(
                "Failed to open process {}: {}",
                process,
                io::Error::last_os_error()
            );
            continue;
        }

        let result = handle_process(process_handle);

        unsafe { CloseHandle(process_handle) };

        if let Some(result) = result {
            return result;
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Could not find matching process",
    ))
}

fn is_local_system_user_token(token: HANDLE) -> io::Result<bool> {
    let mut token_info_len = 0;

    let info_result =
        unsafe { GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut token_info_len) };
    let err = io::Error::last_os_error();

    if info_result != 0 || err.raw_os_error() != Some(ERROR_INSUFFICIENT_BUFFER as i32) {
        log::error!("Failed to obtain token information (length): {}", err);
        return Err(err);
    }

    let mut token_info = vec![0u8; usize::try_from(token_info_len).expect("i32 must fit in usize")];
    let mut _ret_len = 0;

    if unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            token_info.as_mut_ptr() as _,
            token_info_len,
            &mut _ret_len,
        )
    } == 0
    {
        let err = io::Error::last_os_error();
        log::error!("Failed to obtain token information: {}", err);
        return Err(err);
    }

    // SAFETY: We specified `TokenUser` as the class, so that is what GetTokenInformation should
    // return.
    let token_user = unsafe { &*(token_info.as_mut_ptr() as *const TOKEN_USER) };

    let mut local_system_sid = [0u8; MAX_SID_SIZE as usize];
    let mut local_system_size = u32::try_from(local_system_sid.len()).unwrap();

    if unsafe {
        CreateWellKnownSid(
            WinLocalSystemSid,
            std::ptr::null_mut(),
            local_system_sid.as_mut_ptr() as _,
            &mut local_system_size,
        )
    } == 0
    {
        let err = io::Error::last_os_error();
        log::error!("CreateWellKnownSid failed: {}", err);
        return Err(err);
    }

    Ok(unsafe { EqualSid(token_user.User.Sid, local_system_sid.as_ptr() as _) } != 0)
}
