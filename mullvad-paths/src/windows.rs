use crate::{Error, Result};
use once_cell::sync::OnceCell;
use std::{
    io, mem,
    os::windows::prelude::OsStrExt,
    path::{Path, PathBuf},
    ptr,
    ffi::OsStr,
};
use widestring::{WideCStr, WideCString};
use windows_sys::{
    core::{GUID, PWSTR},
    Win32::{
        Foundation::{
            CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, GENERIC_READ, HANDLE,
            INVALID_HANDLE_VALUE, LUID, S_OK, GENERIC_ALL, PSID, GENERIC_ACCESS_RIGHTS, 
        },
        Security::{
            self, AdjustTokenPrivileges, NO_INHERITANCE, SUB_CONTAINERS_AND_OBJECTS_INHERIT, 
            Authorization::{
                SetNamedSecurityInfoW,
                SE_FILE_OBJECT, ConvertStringSidToSidW, TRUSTEE_W, NO_MULTIPLE_TRUSTEE,
                TRUSTEE_IS_SID, TRUSTEE_IS_GROUP, EXPLICIT_ACCESS_W, SET_ACCESS, SetEntriesInAclW,
            },
            CreateWellKnownSid, EqualSid, GetTokenInformation, ImpersonateSelf,
            LookupPrivilegeValueW, RevertToSelf, SecurityImpersonation,
            TokenUser, WinLocalSystemSid, LUID_AND_ATTRIBUTES,
            SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_DUPLICATE,
            TOKEN_IMPERSONATE, TOKEN_PRIVILEGES, TOKEN_QUERY, TOKEN_USER,
        },
        Storage::FileSystem::{MAX_SID_SIZE},
        System::{
            Com::CoTaskMemFree,
            Memory::LocalFree,
            ProcessStatus::EnumProcesses,
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
        if self.0 != 0 && self.0 != INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

fn get_wide_str<S: AsRef<OsStr>>(string: S) -> Vec<u16> {
    let wide_string: Vec<u16> = string.as_ref()
        .encode_wide()
        // Add null terminator
        .chain(std::iter::once(0))
        .collect();
    wide_string
}

fn write_to_file(msg: &str) {
    println!("{msg}");
    use std::io::Write;
    let p = "C:\\Users\\ioio\\Desktop\\DUMP.txt";
    let c = match std::fs::read_to_string(p) {
        Ok(c) => c,
        Err(_) => String::new(),
    };
    let mut file = std::fs::File::create(p).unwrap();
    file.write_all(format!("{}\n{}", c, msg).as_bytes())
        .unwrap();
    file.flush().unwrap();
}

/// Recursively creates directories for the given path with the given security attributes
/// only directories that do not already exist and the leaf directory will have their permissions set.
pub fn create_dir_recursive(
    path: &Path,
    set_security_permissions: bool,
) -> Result<()> {
    if set_security_permissions {
        std::fs::create_dir_all(path).map_err(|e| Error::CreateDirFailed(format!("Could not create directory at {}", path.display()), e))
    } else {
        create_dir_with_permissions_recursive(path)
    }
}

fn create_dir_with_permissions_recursive(
    path: &Path,
) -> Result<()> {
    write_to_file(&format!("create_dir_recursive_inner - path: {path:?}"));
    // No directory to create
    if path == Path::new("") {
        return Ok(());
    }

    match std::fs::create_dir(path) {
        Ok(()) => {
            return set_security_permissions(path);
        },
        // Could not find parent directory, try creating parent
        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
        // Directory already exists, set permissions
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists && path.is_dir() => {
            return set_security_permissions(path);
        },
        Err(e) => return Err(Error::CreateDirFailed(format!("Could not create directory at {}", path.display()), e)),
    }
    write_to_file(&format!("create_dir_recursive_inner: after attempting to create directory - path: {path:?}"));

    match path.parent() {
        // Create parent directory
        Some(parent) => create_dir_with_permissions_recursive(parent)?,
        None => {
            // Reached the top of the tree but when creating directories only got NotFound for some reason
            return Err(Error::CreateDirFailed(
                path.display().to_string(),
                io::Error::new(
                    io::ErrorKind::Other,
                    "reached top of directory tree but could not create directory",
                ),
            ));
        }
    }

    std::fs::create_dir(path)?;
    set_security_permissions(path)
}


/// Recursively creates directories for the given path with permissions that give full access to admins and read only access to authenticated users.
/// If any of the directories already exist this will not return an error, instead it will apply the permissions and if successful return Ok(()).
pub fn create_privileged_directory(path: &Path) -> Result<()> {
    write_to_file("Starting");
    if let Err(e) = create_dir_with_permissions_recursive(
        path,
    ) {
        write_to_file(&format!("Found an ERROR!!: {:?}", e));
        return Err(e);
    } else {
        Ok(())
    }
}

/// Sets security permissions for path such that admin has full ownership and access while authenticated users only have read access.
fn set_security_permissions(path: &Path) -> Result<()> {
    write_to_file(&format!("set_security_permissions - path: {path:?}"));
    let wide_path = get_wide_str(path);
    let security_information = Security::DACL_SECURITY_INFORMATION
        | Security::PROTECTED_DACL_SECURITY_INFORMATION
        | Security::GROUP_SECURITY_INFORMATION
        | Security::OWNER_SECURITY_INFORMATION;

    // Builtin admin SID https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers
    let (admin_ea, admin_psid) = create_explicit_access("S-1-5-32-544", GENERIC_ALL)?;
    // admin_psid is now allocated and must be freed with FreeLocal and *MUST* outlive admin_ea

    // Authenticated users SID https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers
    let (authenticated_users_ea, authenticated_users_psid) = match create_explicit_access("S-1-5-11", GENERIC_READ) {
        Ok((authenticated_users_ea, authenticated_users_psid)) => (authenticated_users_ea, authenticated_users_psid),
        Err(e) => {
            unsafe { LocalFree(admin_psid as isize) };
            return Err(e);
        }
    };
    // authenticated_users_psid is now allocated and must be freed with FreeLocal and *MUST* outlive authenticated_users_ea
    
    let ea_entries = [admin_ea, authenticated_users_ea];
    let mut new_dacl = ptr::null_mut();

    let result = unsafe { SetEntriesInAclW(u32::try_from(ea_entries.len()).unwrap(), ea_entries.as_ptr(), ptr::null(), &mut new_dacl) };
    if ERROR_SUCCESS != result {
        unsafe { LocalFree(admin_psid as isize) };
        unsafe { LocalFree(authenticated_users_psid as isize) };
        return Err(Error::SetDirPermissionFailed(format!("SetEntriesInAclW failed"), io::Error::from_raw_os_error(i32::try_from(result).expect("result does not fit in i32"))));
    }
    // new_dacl is now allocated and must be freed with FreeLocal

    let result = unsafe { SetNamedSecurityInfoW(wide_path.as_ptr(), SE_FILE_OBJECT, security_information, admin_psid, admin_psid, new_dacl, ptr::null()) };

    unsafe { LocalFree(new_dacl as isize) };
    unsafe { LocalFree(admin_psid as isize) };
    unsafe { LocalFree(authenticated_users_psid as isize) };

    if ERROR_SUCCESS != result {
        Err(Error::SetDirPermissionFailed(format!("SetNamedSecurityInfoW failed"), io::Error::from_raw_os_error(i32::try_from(result).expect("result does not fit in i32"))))
    } else {
        Ok(())
    }
}

/// Takes a wide string encoded SID string and access rights and produces a EXPLICIT_ACCESS_W object and a raw pointer to the
/// allocated SID object. The caller is responsible for calling LocalFree on the returned raw pointer when they are done with it.
/// The raw pointer *MUST* outlive the EXPLICIT_ACCESS_W object.
fn create_explicit_access(sid_wide_str: &str, access_rights: GENERIC_ACCESS_RIGHTS) -> Result<(EXPLICIT_ACCESS_W, PSID)> {
    let sid_wide_str = get_wide_str(sid_wide_str);
    let mut psid = ptr::null_mut();
    if 0 == unsafe { ConvertStringSidToSidW(sid_wide_str.as_ptr(), &mut psid) } {
        return Err(Error::SetDirPermissionFailed(format!("Could not create SID"), io::Error::last_os_error()));
    }

    let trustee = TRUSTEE_W {
        pMultipleTrustee: ptr::null_mut(),
        MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
        TrusteeForm: TRUSTEE_IS_SID,
        TrusteeType: TRUSTEE_IS_GROUP,
        ptstrName: psid as *mut _,
    };

    let explicit_access = EXPLICIT_ACCESS_W {
        grfAccessPermissions: access_rights,
        grfAccessMode: SET_ACCESS,
        grfInheritance: NO_INHERITANCE | SUB_CONTAINERS_AND_OBJECTS_INHERIT,
        Trustee: trustee,
    };

    Ok((explicit_access, psid))
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
