use crate::{Error, Result, UserPermissions};
use once_cell::sync::OnceCell;
use std::{
    ffi::OsStr,
    io, mem,
    os::windows::{
        io::{
            AsHandle, AsRawHandle, BorrowedHandle, FromRawHandle, HandleOrNull, IntoRawHandle,
            OwnedHandle,
        },
        prelude::OsStrExt,
    },
    path::{Path, PathBuf},
    ptr,
};
use widestring::{WideCStr, WideCString};
use windows_sys::{
    Win32::{
        Foundation::{
            ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, GENERIC_ALL, GENERIC_EXECUTE, GENERIC_READ,
            GENERIC_WRITE, HANDLE, LUID, LocalFree, S_OK,
        },
        Security::{
            self, AdjustTokenPrivileges,
            Authorization::{
                EXPLICIT_ACCESS_W, NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT, SET_ACCESS,
                SetEntriesInAclW, SetNamedSecurityInfoW, TRUSTEE_IS_GROUP, TRUSTEE_IS_SID,
                TRUSTEE_W,
            },
            CreateWellKnownSid, EqualSid, GetTokenInformation, ImpersonateSelf,
            LUID_AND_ATTRIBUTES, LookupPrivilegeValueW, NO_INHERITANCE, RevertToSelf,
            SE_PRIVILEGE_ENABLED, SUB_CONTAINERS_AND_OBJECTS_INHERIT, SecurityImpersonation,
            TOKEN_ADJUST_PRIVILEGES, TOKEN_DUPLICATE, TOKEN_IMPERSONATE, TOKEN_PRIVILEGES,
            TOKEN_QUERY, TOKEN_USER, TokenUser, WinAuthenticatedUserSid,
            WinBuiltinAdministratorsSid, WinLocalSystemSid,
        },
        Storage::FileSystem::MAX_SID_SIZE,
        System::{
            Com::CoTaskMemFree,
            ProcessStatus::EnumProcesses,
            Threading::{
                GetCurrentThread, OpenProcess, OpenProcessToken, OpenThreadToken,
                PROCESS_QUERY_INFORMATION,
            },
        },
        UI::Shell::{
            FOLDERID_LocalAppData, FOLDERID_System, KF_FLAG_DEFAULT, SHGetKnownFolderPath,
        },
    },
    core::{GUID, PWSTR},
};

pub const PRODUCT_NAME: &str = "Mullvad VPN";

pub fn get_allusersprofile_dir() -> Result<PathBuf> {
    match std::env::var_os("ALLUSERSPROFILE") {
        Some(dir) => Ok(PathBuf::from(&dir)),
        None => Err(Error::NoProgramDataDir),
    }
}

/// This recursively creates directories, if set_security_permissions is true it will set
/// file permissions corresponding to Authenticated Users - Read Only and Administrators - Full
/// Access. Only directories that do not already exist and the leaf directory will have their
/// permissions set.
#[cfg(windows)]
pub fn create_dir(path: PathBuf, user_permissions: Option<UserPermissions>) -> Result<PathBuf> {
    if let Some(user_permissions) = user_permissions {
        create_dir_recursive_with_permissions(&path, user_permissions)?;
    } else {
        std::fs::create_dir_all(&path).map_err(|e| {
            Error::CreateDirFailed(
                format!("Could not create directory at {}", path.display()),
                e,
            )
        })?;
    }
    Ok(path)
}

fn get_wide_str<S: AsRef<OsStr>>(string: S) -> Vec<u16> {
    let wide_string: Vec<u16> = string.as_ref()
        .encode_wide()
        // Add null terminator
        .chain(std::iter::once(0))
        .collect();
    wide_string
}

impl UserPermissions {
    fn flags(self) -> u32 {
        let mut flags = 0;
        if self.read {
            flags |= GENERIC_READ;
        }
        if self.write {
            flags |= GENERIC_WRITE;
        }
        if self.execute {
            flags |= GENERIC_EXECUTE;
        }
        flags
    }
}

/// If directory at path already exists, set permissions for it.
/// If directory at path don't exist but parent does, create directory and set permissions.
/// If parent directory at path does not exist then recurse and create parent directory and set
/// permissions for it, then create child directory and set permissions.
/// This does not set permissions for parent directories that already exists.
fn create_dir_recursive_with_permissions(
    path: &Path,
    user_permissions: UserPermissions,
) -> Result<()> {
    // No directory to create
    if path == Path::new("") {
        return Ok(());
    }

    match std::fs::create_dir(path) {
        Ok(()) => {
            return set_security_permissions(path, user_permissions);
        }
        // Could not find parent directory, try creating parent
        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
        // Directory already exists, set permissions
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists && path.is_dir() => {
            return set_security_permissions(path, user_permissions);
        }
        Err(e) => {
            return Err(Error::CreateDirFailed(
                format!("Could not create directory at {}", path.display()),
                e,
            ));
        }
    }

    match path.parent() {
        // Create parent directory
        Some(parent) => create_dir_recursive_with_permissions(parent, user_permissions)?,
        None => {
            // Reached the top of the tree but when creating directories only got NotFound for some
            // reason
            return Err(Error::CreateDirFailed(
                path.display().to_string(),
                io::Error::other("reached top of directory tree but could not create directory"),
            ));
        }
    }

    std::fs::create_dir(path).map_err(|e| Error::CreateDirFailed(path.display().to_string(), e))?;
    set_security_permissions(path, user_permissions)
}

/// Recursively creates directories for the given path with permissions that give full access to
/// admins and read only access to authenticated users. If any of the directories already exist this
/// will not return an error, instead it will apply the permissions and if successful return Ok(()).
pub fn create_privileged_directory(path: &Path) -> Result<()> {
    create_dir_recursive_with_permissions(path, UserPermissions::read_only())
}

/// Sets security permissions for path such that admin has full ownership and access while
/// authenticated users only have read access.
fn set_security_permissions(path: &Path, user_permissions: UserPermissions) -> Result<()> {
    let wide_path = get_wide_str(path);
    let security_information = Security::DACL_SECURITY_INFORMATION
        | Security::PROTECTED_DACL_SECURITY_INFORMATION
        | Security::GROUP_SECURITY_INFORMATION
        | Security::OWNER_SECURITY_INFORMATION;

    let mut admin_psid = [0u8; MAX_SID_SIZE as usize];
    let mut admin_psid_len = u32::try_from(admin_psid.len()).unwrap();

    // SAFETY: The pointer to the PSID is valid for writes of `admin_psid_len` bytes
    if unsafe {
        CreateWellKnownSid(
            WinBuiltinAdministratorsSid,
            ptr::null_mut(),
            admin_psid.as_mut_ptr() as _,
            &mut admin_psid_len,
        )
    } == 0
    {
        return Err(Error::SetDirPermissionFailed(
            String::from("Could not create admin SID"),
            io::Error::last_os_error(),
        ));
    }

    let trustee = TRUSTEE_W {
        pMultipleTrustee: ptr::null_mut(),
        MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
        TrusteeForm: TRUSTEE_IS_SID,
        TrusteeType: TRUSTEE_IS_GROUP,
        ptstrName: admin_psid.as_mut_ptr() as *mut _,
    };

    let admin_ea = EXPLICIT_ACCESS_W {
        grfAccessPermissions: GENERIC_ALL,
        grfAccessMode: SET_ACCESS,
        grfInheritance: NO_INHERITANCE | SUB_CONTAINERS_AND_OBJECTS_INHERIT,
        Trustee: trustee,
    };

    let mut au_psid = [0u8; MAX_SID_SIZE as usize];
    let mut au_psid_len = u32::try_from(au_psid.len()).unwrap();

    // SAFETY: The pointer to the PSID is valid for writes of `au_psid_len` bytes
    if unsafe {
        CreateWellKnownSid(
            WinAuthenticatedUserSid,
            ptr::null_mut(),
            au_psid.as_mut_ptr() as _,
            &mut au_psid_len,
        )
    } == 0
    {
        return Err(Error::SetDirPermissionFailed(
            String::from("Could not create authenticated users SID"),
            io::Error::last_os_error(),
        ));
    }

    let trustee = TRUSTEE_W {
        pMultipleTrustee: ptr::null_mut(),
        MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
        TrusteeForm: TRUSTEE_IS_SID,
        TrusteeType: TRUSTEE_IS_GROUP,
        ptstrName: au_psid.as_mut_ptr() as *mut _,
    };

    let authenticated_users_ea = EXPLICIT_ACCESS_W {
        grfAccessPermissions: user_permissions.flags(),
        grfAccessMode: SET_ACCESS,
        grfInheritance: NO_INHERITANCE | SUB_CONTAINERS_AND_OBJECTS_INHERIT,
        Trustee: trustee,
    };

    let ea_entries = [admin_ea, authenticated_users_ea];
    let mut new_dacl = ptr::null_mut();

    // SAFETY: `ea_entries` is valid for reads of `ea_entries.len()` elements
    // `new_dacl` is a valid pointer to an ACL pointer
    let result = unsafe {
        SetEntriesInAclW(
            u32::try_from(ea_entries.len()).unwrap(),
            ea_entries.as_ptr(),
            ptr::null(),
            &mut new_dacl,
        )
    };
    if result != ERROR_SUCCESS {
        return Err(Error::SetDirPermissionFailed(
            String::from("SetEntriesInAclW failed"),
            io::Error::from_raw_os_error(
                i32::try_from(result).expect("result does not fit in i32"),
            ),
        ));
    }
    // new_dacl is now allocated and must be freed with FreeLocal

    // SAFETY: All pointers are valid
    let result = unsafe {
        SetNamedSecurityInfoW(
            wide_path.as_ptr(),
            SE_FILE_OBJECT,
            security_information,
            admin_psid.as_mut_ptr() as *mut _,
            admin_psid.as_mut_ptr() as *mut _,
            new_dacl,
            ptr::null(),
        )
    };

    // SAFETY: `new_dacl` is a valid pointer since `SetEntriesInAclW` succeeded
    unsafe { LocalFree(new_dacl.cast()) };

    if result != ERROR_SUCCESS {
        Err(Error::SetDirPermissionFailed(
            String::from("SetNamedSecurityInfoW failed"),
            io::Error::from_raw_os_error(
                i32::try_from(result).expect("result does not fit in i32"),
            ),
        ))
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
                    let user_token = OwnedHandle::try_from(get_system_user_token()?).ok();
                    // SAFETY: `FOLDERID_LocalAppData` is a valid known folder ID
                    unsafe {
                        get_known_folder_path(
                            &FOLDERID_LocalAppData,
                            KF_FLAG_DEFAULT,
                            user_token.as_ref().map(|t| t.as_handle()),
                        )
                    }
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
/// If the current user is system, this function succeeds and returns a NULL handle
fn get_system_user_token() -> io::Result<HandleOrNull> {
    let thread_token = get_current_thread_token()?;

    if is_local_system_user_token(&thread_token)? {
        // SAFETY: It is safe to pass a null handle
        return Ok(unsafe { HandleOrNull::from_raw_handle(ptr::null_mut()) });
    }

    let system_debug_priv = WideCString::from_str("SeDebugPrivilege").unwrap();
    adjust_token_privilege(&thread_token, &system_debug_priv, true)?;

    let find_result = find_process(|process_handle| {
        let process_token = open_process_token(
            &process_handle,
            GENERIC_READ | TOKEN_IMPERSONATE | TOKEN_DUPLICATE,
        )
        .ok()?;

        match is_local_system_user_token(&process_token) {
            Ok(true) => Some(process_token),
            _ => None,
        }
    });

    if let Err(err) = adjust_token_privilege(&thread_token, &system_debug_priv, false) {
        log::error!("Failed to drop SeDebugPrivilege: {}", err);
    }

    // SAFETY: The handle is valid
    find_result.map(|h| unsafe { HandleOrNull::from_raw_handle(h.into_raw_handle()) })
}

fn open_process_token(process: &impl AsRawHandle, access: u32) -> io::Result<OwnedHandle> {
    let mut process_token = ptr::null_mut();
    // SAFETY: `process` is a valid handle
    if unsafe { OpenProcessToken(process.as_raw_handle(), access, &mut process_token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `process_token` is a valid handle since `OpenProcessToken` succeeded
    Ok(unsafe { OwnedHandle::from_raw_handle(process_token) })
}

/// If all else fails, infer the AppData path from the system directory.
fn infer_appdata_from_system_directory() -> io::Result<PathBuf> {
    // SAFETY: `FOLDERID_System` is a valid known folder ID
    let mut sysdir = unsafe { get_known_folder_path(&FOLDERID_System, KF_FLAG_DEFAULT, None) }?;
    sysdir.extend(["config", "systemprofile", "AppData", "Local"]);
    Ok(sysdir)
}

fn get_current_thread_token() -> std::io::Result<OwnedHandle> {
    let mut token_handle: HANDLE = ptr::null_mut();
    // SAFETY: `GetCurrentThread` always returns a valid handle
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
    // SAFETY: `token_handle` is a valid handle since `OpenThreadToken` succeeded
    Ok(unsafe { OwnedHandle::from_raw_handle(token_handle) })
}

/// Run provided closure in the security context of the calling process' impersonation token.
///
/// # Panics
///
/// If privileges can not be dropped after running `func`, the running process is shut down.
fn impersonate_self<T>(func: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    // SAFETY: SecurityImpersonation is a valid ImpersonationLevel.
    if unsafe { ImpersonateSelf(SecurityImpersonation) } == 0 {
        return Err(std::io::Error::last_os_error());
    }

    let result = func();

    // SAFETY: Must be called after a successful call to ImpersonateSelf.
    if unsafe { RevertToSelf() } == 0 {
        // The Windows documentation *strongly* suggest that the process should shut down if
        // RevertToSelf fails. A failure to do so means that the current process keep running in
        // an unintended context.
        log::error!("RevertToSelf failed: {}", io::Error::last_os_error());
        panic!("RevertToSelf failed. Aborting");
    }

    result
}

fn adjust_token_privilege(
    token_handle: &impl AsRawHandle,
    privilege: &WideCStr,
    enable: bool,
) -> std::io::Result<()> {
    let mut privilege_luid = LUID::default();

    // SAFETY: `privilege` is a valid null-terminated string, and `privilege_luid` points to a LUID
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
    // SAFETY: All pointers are valid
    let result = unsafe {
        AdjustTokenPrivileges(
            token_handle.as_raw_handle(),
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

/// Retrieve path to a known folder for a specific user token.
///
/// # Safety
///
/// `folder_id` must be a valid pointer to a known folder ID GUID.
unsafe fn get_known_folder_path(
    folder_id: *const GUID,
    flags: i32,
    user_token: Option<BorrowedHandle<'_>>,
) -> std::io::Result<PathBuf> {
    let mut folder_path: PWSTR = ptr::null_mut();
    // SAFETY: All arguments are valid
    let status = unsafe {
        SHGetKnownFolderPath(
            folder_id,
            flags as u32,
            user_token
                .map(|h| h.as_raw_handle())
                .unwrap_or(ptr::null_mut()),
            &mut folder_path,
        )
    };
    let result = if status == S_OK {
        // SAFETY: `folder_path` is valid and null-terminated since `SHGetKnownFolderPath` succeeded
        let path = unsafe { WideCStr::from_ptr_str(folder_path) };
        Ok(PathBuf::from(path.to_os_string()))
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Can't find known folder {:?}", &folder_id),
        ))
    };

    // SAFETY: `folder_path` was allocated by `SHGetKnownFolderPath` and must be freed with `CoTaskMemFree
    unsafe { CoTaskMemFree(folder_path as *mut _) };
    result
}

/// Enumerate over all processes until `handle_process` returns a result or until there are
/// no more processes left. In the latter case, an error is returned.
fn find_process<T>(handle_process: impl Fn(BorrowedHandle<'_>) -> Option<T>) -> io::Result<T> {
    let mut pid_buffer = vec![0u32; 2048];
    let mut num_procs: u32 = u32::try_from(pid_buffer.len()).unwrap();

    let bytes_available = num_procs * (mem::size_of::<u32>() as u32);
    let mut bytes_written = 0;

    // SAFETY: `pid_buffer` is valid for writes of `bytes_available` bytes
    if unsafe { EnumProcesses(pid_buffer.as_mut_ptr(), bytes_available, &mut bytes_written) } == 0 {
        return Err(io::Error::last_os_error());
    }

    num_procs = bytes_written / (mem::size_of::<u32>() as u32);
    pid_buffer.resize(num_procs as usize, 0);

    pid_buffer
        .into_iter()
        .find_map(|process| {
            // SAFETY: Trivially safe
            let process_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, process) };
            if process_handle.is_null() {
                return None;
            }
            // SAFETY: `process_handle` is a valid handle since `OpenProcess` succeeded
            let process_handle = unsafe { OwnedHandle::from_raw_handle(process_handle) };
            handle_process(process_handle.as_handle())
        })
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find matching process",
        ))
}

fn is_local_system_user_token(token: &impl AsRawHandle) -> io::Result<bool> {
    let mut token_info = vec![0u8; 1024];

    loop {
        let mut returned_info_len = 0;

        // SAFETY: `token` is a valid handle, and `token_info` is valid for writes of
        // `token_info.len()` bytes
        let info_result = unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
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

    // SAFETY: `local_system_sid` is valid for writes of `local_system_size` bytes
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

    // SAFETY: Both arguments point to valid security identifiers
    Ok(unsafe { EqualSid(token_user.User.Sid, local_system_sid.as_ptr() as _) } != 0)
}
