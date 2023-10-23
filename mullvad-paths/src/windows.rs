use crate::{Result, Error};
use once_cell::sync::OnceCell;
use std::{
    io, mem,
    os::windows::prelude::OsStrExt,
    path::{Path, PathBuf},
    ptr,
};
use widestring::{WideCStr, WideCString};
use windows_sys::{
    core::{GUID, PWSTR},
    Win32::{
        Foundation::{
            CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, GENERIC_READ, HANDLE,
            INVALID_HANDLE_VALUE, LUID, S_OK,
        },
        Security::{
            self, AdjustTokenPrivileges,
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
            },
            CreateWellKnownSid, EqualSid, GetTokenInformation, ImpersonateSelf,
            IsValidSecurityDescriptor, LookupPrivilegeValueW, RevertToSelf, SecurityImpersonation,
            SetFileSecurityW, TokenUser, WinLocalSystemSid, LUID_AND_ATTRIBUTES,
            SECURITY_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_DUPLICATE,
            TOKEN_IMPERSONATE, TOKEN_PRIVILEGES, TOKEN_QUERY, TOKEN_USER,
        },
        Storage::FileSystem::{CreateDirectoryW, MAX_SID_SIZE},
        System::{
            Com::CoTaskMemFree,
            ProcessStatus::EnumProcesses,
            Memory::LocalFree,
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

// SAFETY: Only allowed to hold a SECURITY_ATTRIBUTES which points to an allocated and valid `lpSecurityDescriptor`
pub struct SecurityAttributes(SECURITY_ATTRIBUTES);
struct Handle(HANDLE);

impl Drop for SecurityAttributes {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0.lpSecurityDescriptor as isize) };
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if self.0 != 0 && self.0 != INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

/// Creates security attributes that give full access to admins and read only access to authenticated users
pub fn create_security_attributes_with_admin_full_access_user_read_only(
) -> Result<SecurityAttributes> {
    let mut security_attributes = SECURITY_ATTRIBUTES {
        nLength: u32::try_from(std::mem::size_of::<SECURITY_ATTRIBUTES>()).unwrap(),
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 0,
    };

    // Security Descriptor gives ownership to admin, group to admin, full-access to admin and read only access to authenticated users. OICI describes inheritance.
    let sd: Vec<u16> = std::ffi::OsStr::new(
        "O:S-1-5-32-544G:S-1-5-32-544D:P(A;OICI;GA;;;S-1-5-32-544)(A;OICI;GR;;;AU)",
    )
    .encode_wide()
    .collect();

    if 0 == unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sd.as_ptr(),
                SDDL_REVISION_1,
                &mut security_attributes.lpSecurityDescriptor,
                ptr::null_mut(),
            )
        }
    {
        return Err(Error::GetSecurityAttributes(io::Error::last_os_error()));
    }

    // Guarantee that sd is dropped after pointer to sd is dropped.
    drop(sd);

    // SAFETY: `security_attributes.lpSecurityDescriptor` is now valid and allocated.
    let security_attributes = SecurityAttributes(security_attributes);

    if 0 == unsafe { IsValidSecurityDescriptor(security_attributes.0.lpSecurityDescriptor) }
    {
        return Err(Error::GetSecurityAttributes(io::Error::last_os_error()));
    }

    Ok(security_attributes)
}

/// Non-recursively create a directory at the given path with the given security attributes
pub fn create_directory(
    path: &Path,
    security_attributes: &mut Option<SecurityAttributes>,
) -> Result<()> {
    // FIXME
    //let security_attributes = &mut None;
    let wide_path: Vec<u16> = path.as_os_str().encode_wide().collect();

    let sa = security_attributes
        .as_ref()
        .map(|sa: &SecurityAttributes| &sa.0 as *const _)
        .unwrap_or(ptr::null());

    if ERROR_SUCCESS as i32 != unsafe { CreateDirectoryW(wide_path.as_ptr(), sa) } {
        let err = io::Error::last_os_error();
        if err.kind() != io::ErrorKind::AlreadyExists {
            return Err(Error::CreateDirFailed(path.display().to_string(), err));
        }
    }

    if let Some(security_attributes) = security_attributes {

        let security_information = Security::OWNER_SECURITY_INFORMATION
            | Security::GROUP_SECURITY_INFORMATION
            | Security::DACL_SECURITY_INFORMATION
            | Security::PROTECTED_DACL_SECURITY_INFORMATION;

        let sd = security_attributes.0.lpSecurityDescriptor;

        if ERROR_SUCCESS as i32
            != unsafe { SetFileSecurityW(wide_path.as_ptr(), security_information, sd) }
        {
            return Err(Error::SetDirPermissionFailed(path.display().to_string(), io::Error::last_os_error()));
        }
    }

    return Ok(());
}

fn write_to_file(msg: &str) {
    let p = "C:\\Users\\ioio\\Desktop\\DUMP.txt";
    let c = std::fs::read_to_string(p).unwrap();
    std::fs::write(p, format!("{}\n{}", c, msg)).unwrap();
}

/// Recursively creates directories for the given path with the given security attributes
/// only directories that do not already exist will have their permissions set.
pub fn create_dir_recursive(
    path: &Path,
    permissions: &mut Option<SecurityAttributes>,
) -> Result<()> {
    // No directory to create
    if path == Path::new("") {
        return Ok(());
    }

    match create_directory(path, permissions) {
        Ok(()) => return Ok(()),
        // Could not find parent directory, try creating parent
        Err(Error::CreateDirFailed(ref _path, ref e)) if e.kind() == io::ErrorKind::NotFound => (),
        // Directory already exists
        Err(_) if path.is_dir() => return Ok(()),
        Err(e) => return Err(e),
    }

    match path.parent() {
        // Create parent directory
        Some(parent) => create_dir_recursive(parent, permissions)?,
        None => {
            // Reached the top of the tree but when creating directories only got NotFound for some reason
            return Err(Error::CreateDirFailed(path.display().to_string(), io::Error::new(io::ErrorKind::Other, "reached top of directory tree but could not create directory")));
        }
    }

    // If still can't create directory then fail
    match create_directory(path, permissions) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Recursively creates directories for the given path with permissions that give full access to admins and read only access to authenticated users.
/// If any of the directories already exist this will not return an error, instead it will apply the permissions and if successful return Ok(()).
pub fn create_privileged_directory(path: &Path) -> Result<()> {
    write_to_file("Starting");
    if let Err(e) = create_dir_recursive(
        path,
        &mut Some(create_security_attributes_with_admin_full_access_user_read_only()?),
    ) {
        write_to_file(&format!("Found an ERROR!!: {:?}", e));
        return Err(e);
    } else {
        Ok(())
    }
}

/// Get local AppData path for the system service user.
pub fn get_system_service_appdata() -> io::Result<PathBuf> {
    static APPDATA_PATH: OnceCell<PathBuf> = OnceCell::new();

    APPDATA_PATH
        .get_or_try_init(|| {
            let join_handle = std::thread::spawn(|| {
                impersonate_self(|| {
                    let user_token = get_system_user_token()?;
                    get_known_folder_path(&FOLDERID_LocalAppData, KF_FLAG_DEFAULT, user_token.0)
                })
                .or_else(|error| {
                    log::error!("Failed to get AppData path: {error}");
                    infer_appdata_from_system_directory()
                })
            });
            join_handle.join().unwrap()
        })
        .cloned()
}

/// Get user token for the system service user. Requires elevated privileges to work.
/// Useful for deducing the config path for the daemon on Windows when running as a user that
/// isn't the system service.
/// If the current user is system, this function succeeds and returns a `NULL` handle;
fn get_system_user_token() -> io::Result<Handle> {
    let thread_token = get_current_thread_token()?;

    if is_local_system_user_token(thread_token.0)? {
        return Ok(Handle(0));
    }

    let system_debug_priv = WideCString::from_str("SeDebugPrivilege").unwrap();
    adjust_token_privilege(thread_token.0, &system_debug_priv, true)?;

    let find_result = find_process(|process_handle| {
        let process_token = open_process_token(
            process_handle,
            GENERIC_READ | TOKEN_IMPERSONATE | TOKEN_DUPLICATE,
        )
        .ok()?;

        match is_local_system_user_token(process_token.0) {
            Ok(true) => Some(process_token),
            _ => None,
        }
    });

    if let Err(err) = adjust_token_privilege(thread_token.0, &system_debug_priv, false) {
        log::error!("Failed to drop SeDebugPrivilege: {}", err);
    }

    find_result
}

fn open_process_token(process: HANDLE, access: u32) -> io::Result<Handle> {
    let mut process_token = 0;
    if unsafe { OpenProcessToken(process, access, &mut process_token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(Handle(process_token))
}

/// If all else fails, infer the AppData path from the system directory.
fn infer_appdata_from_system_directory() -> io::Result<PathBuf> {
    let mut sysdir = get_known_folder_path(&FOLDERID_System, KF_FLAG_DEFAULT, 0)?;
    sysdir.extend(["config", "systemprofile", "AppData", "Local"]);
    Ok(sysdir)
}

fn get_current_thread_token() -> std::io::Result<Handle> {
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
        return Err(std::io::Error::last_os_error());
    }
    Ok(Handle(token_handle))
}

fn impersonate_self<T>(func: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    if unsafe { ImpersonateSelf(SecurityImpersonation) } == 0 {
        return Err(std::io::Error::last_os_error());
    }

    let result = func();

    if unsafe { RevertToSelf() } == 0 {
        log::error!("RevertToSelf failed: {}", io::Error::last_os_error());
    }

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

    let privileges = TOKEN_PRIVILEGES {
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
            &privileges,
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
        Ok(PathBuf::from(path.to_os_string()))
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
fn find_process<T>(handle_process: impl Fn(HANDLE) -> Option<T>) -> io::Result<T> {
    let mut pid_buffer = vec![0u32; 2048];
    let mut num_procs: u32 = u32::try_from(pid_buffer.len()).unwrap();

    let bytes_available = num_procs * (mem::size_of::<u32>() as u32);
    let mut bytes_written = 0;
    if unsafe { EnumProcesses(pid_buffer.as_mut_ptr(), bytes_available, &mut bytes_written) } == 0 {
        return Err(io::Error::last_os_error());
    }

    num_procs = bytes_written / (mem::size_of::<u32>() as u32);
    pid_buffer.resize(num_procs as usize, 0);

    pid_buffer
        .into_iter()
        .find_map(|process| {
            let process_handle =
                Handle(unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, process) });
            if process_handle.0 == 0 {
                return None;
            }
            handle_process(process_handle.0)
        })
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find matching process",
        ))
}

fn is_local_system_user_token(token: HANDLE) -> io::Result<bool> {
    let mut token_info = vec![0u8; 1024];

    loop {
        let mut returned_info_len = 0;

        let info_result = unsafe {
            GetTokenInformation(
                token,
                TokenUser,
                token_info.as_mut_ptr() as _,
                u32::try_from(token_info.len()).expect("len must fit in u32"),
                &mut returned_info_len,
            )
        };

        let err = io::Error::last_os_error();
        if info_result == 0 && err.raw_os_error() != Some(ERROR_INSUFFICIENT_BUFFER as i32) {
            log::error!("Failed to obtain token information: {}", err);
            return Err(err);
        }

        token_info.resize(
            usize::try_from(returned_info_len).expect("u32 must fit in usize"),
            0,
        );
        if info_result != 0 {
            break;
        }
    }

    // SAFETY: We specified `TokenUser` as the class, so that is what GetTokenInformation should
    // return. This reference is valid for as long as `token_info` is valid.
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
