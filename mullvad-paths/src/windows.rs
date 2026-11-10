use crate::{Error, Result, UserPermissions};
use once_cell::sync::OnceCell;
use std::{
    ffi::c_void,
    io, mem,
    os::windows::io::{
        AsHandle as _, AsRawHandle, BorrowedHandle, FromRawHandle as _, OwnedHandle,
    },
    path::{Path, PathBuf},
    ptr::{self, NonNull},
};
use talpid_types::{aligned_bytes::AlignedBytes, windows::LocalFreeGuard};
use widestring::{U16CString, WideCStr, u16cstr};
use windows_sys::{
    Win32::{
        Foundation::{
            ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, GENERIC_ALL, GENERIC_EXECUTE, GENERIC_READ,
            GENERIC_WRITE, HANDLE, LUID, S_OK,
        },
        Security::{
            self, AdjustTokenPrivileges,
            Authorization::{
                EXPLICIT_ACCESS_W, GetNamedSecurityInfoW, NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT,
                SET_ACCESS, SetEntriesInAclW, SetNamedSecurityInfoW, TRUSTEE_IS_GROUP,
                TRUSTEE_IS_SID, TRUSTEE_W,
            },
            CreateWellKnownSid, EqualSid, GetTokenInformation, ImpersonateSelf,
            InitializeSecurityDescriptor, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW,
            NO_INHERITANCE, RevertToSelf, SE_PRIVILEGE_ENABLED, SECURITY_ATTRIBUTES,
            SECURITY_DESCRIPTOR, SID, SUB_CONTAINERS_AND_OBJECTS_INHERIT, SecurityImpersonation,
            SetSecurityDescriptorDacl, TOKEN_ADJUST_PRIVILEGES, TOKEN_DUPLICATE, TOKEN_IMPERSONATE,
            TOKEN_PRIVILEGES, TOKEN_QUERY, TOKEN_USER, TokenUser, WELL_KNOWN_SID_TYPE,
            WinAuthenticatedUserSid, WinBuiltinAdministratorsSid, WinLocalSystemSid,
        },
        Storage::FileSystem::{CreateDirectoryW, MAX_SID_SIZE},
        System::{
            Com::CoTaskMemFree,
            ProcessStatus::EnumProcesses,
            SystemServices::SECURITY_DESCRIPTOR_REVISION,
            Threading::{
                GetCurrentThread, OpenProcess, OpenProcessToken, OpenThreadToken,
                PROCESS_QUERY_INFORMATION,
            },
        },
        UI::Shell::{
            FOLDERID_LocalAppData, FOLDERID_System, KF_FLAG_DEFAULT, SHGetKnownFolderPath,
        },
    },
    core::GUID,
};

pub const PRODUCT_NAME: &str = "Mullvad VPN";

/// Get the path to the daemon data directory, creating it if it doesn't exist.
///
/// The following permissions are enforced:
/// - The directory is NOT writable by anyone other than SYSTEM.
/// - The directory is readable & executable by any authenticated user.
///
/// # Errors
/// This functions returns `Err`:
/// - A syscall unexpectedly errors.
/// - If the directory exists, but has unexpected permissions.
pub fn data_dir() -> Result<PathBuf> {
    let path = get_data_dir()?;

    let dacl = build_acl(UserPermissions {
        read: true,
        write: false, // no write-access for normal users
        execute: true,
    })?;

    // We assume that allusersprofile_dir exists
    match create_dir_with_dacl(&path, &dacl) {
        Ok(()) => (),
        Err(Error::CreateDirFailed(_, src)) if src.kind() == io::ErrorKind::AlreadyExists => {
            let actual_dacl = get_dacl(&path)?;
            if let Some(actual_dacl) = actual_dacl
                // SAFETY: TODO:
                && unsafe { acl_eq(&dacl, &actual_dacl) }
            {
                // all good!
                // TODO: describe reasoning of why all good
            } else {
                todo!("bad permissions! bail out and burn everything to the ground.")
            }
        }
        Err(e) => return Err(e),
    }

    Ok(path)
}

/// Get the path to the daemon data directory in `C:\ProgramData`.
///
/// Use [`data_dir`] if you intend to access this directory for any reason.
pub fn get_data_dir() -> Result<PathBuf> {
    Ok(get_allusersprofile_dir()?.join(PRODUCT_NAME))
}

pub fn get_allusersprofile_dir() -> Result<PathBuf> {
    match std::env::var_os("ALLUSERSPROFILE") {
        Some(dir) => Ok(PathBuf::from(&dir)),
        None => Err(Error::NoProgramDataDir),
    }
}

/// Create a directory with a specified access control list.
///
/// Will return [`io::ErrorKind::AlreadyExists`] if the directory already exists,
/// in this case the existing directory's ACL will be **unchanged**.
fn create_dir_with_dacl(path: &Path, acl: &LocalFreeGuard<Security::ACL>) -> Result<()> {
    let wpath = U16CString::from_os_str(path)?;

    let mut security_descriptor = SECURITY_DESCRIPTOR::default();

    // SAFETY: This function is trivially safe.
    let success = unsafe {
        InitializeSecurityDescriptor(
            (&raw mut security_descriptor).cast::<c_void>(),
            SECURITY_DESCRIPTOR_REVISION,
        )
    };
    if success == 0 {
        return Err(Error::CreateDirFailed(
            format!("InitializeSecurityDescriptor(...)"),
            io::Error::last_os_error(),
        ));
    }

    // SAFETY: security_descriptor has been initialized. ACL is valid.
    // SAFETY: TODO: ACL members must be in "absolute format" (pointers, not offsets). How do we know whether this is the case?
    let success = unsafe {
        SetSecurityDescriptorDacl(
            (&raw mut security_descriptor).cast::<c_void>(),
            true.into(), // DACL is present
            acl.as_ptr(),
            false.into(), // DACL is not defaulted; it's user-specified
        )
    };
    if success == 0 {
        return Err(Error::CreateDirFailed(
            format!("SetSecurityDescriptorDacl(...)"),
            io::Error::last_os_error(),
        ));
    }

    // TODO: is SetSecurityDescriptorOwner interesting?
    // TODO: is SetSecurityDescriptorGroup interesting?
    // TODO: is SetSecurityDescriptorRMControl interesting?
    // TODO: is SetSecurityDescriptorSacl interesting?

    let attrs = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: (&raw mut security_descriptor).cast::<c_void>(),
        bInheritHandle: false.into(),
    };

    // SAFETY: TODO:
    let success = unsafe { CreateDirectoryW(wpath.as_ptr(), &raw const attrs) };
    if success == 0 {
        return Err(Error::CreateDirFailed(
            format!("CreateDirectoryW({path:?})"),
            io::Error::last_os_error(),
        ));
    }

    Ok(())
}

/// Create a directory at `path`.
///
/// Parent directories are created as-needed.
/// If `user_permissions` is None, directory permissions are inherited from their parent.
/// If `user_permissions` is Some, the directory is configured as:
/// - Full access by SYSTEM
/// - Limited access according to `user_permissions` by any authenticated user.
///
/// Only directories that do not already exist and the leaf directory
/// will have their permissions set.
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

/// Wrapper for a windows security identifier ([`SID`]).
struct Sid {
    /// Byte buffer for the dyn-sized [`SID`] struct.
    pub buf: AlignedBytes<SID>,
}

impl Sid {
    pub fn as_ptr(&self) -> *const c_void {
        self.buf.as_ptr().cast::<c_void>()
    }
}

/// Get a [`SID`] from a [`WELL_KNOWN_SID_TYPE`].
fn get_well_known_sid(sid: WELL_KNOWN_SID_TYPE) -> Result<Sid> {
    // allocate a buf for the dyn-sized SID type.
    let mut len = MAX_SID_SIZE;
    let mut buf = AlignedBytes::<SID>::new(len as usize);

    // SAFETY: The pointer to the PSID is owned and valid for writes of `len` bytes
    let result = unsafe {
        CreateWellKnownSid(
            sid,
            ptr::null_mut(),
            buf.as_mut_ptr().cast::<c_void>(),
            &raw mut len,
        )
    };
    if result == 0 {
        return Err(Error::SetDirPermissionFailed(
            format!("Could not get well-known SID: {sid:?}"),
            io::Error::last_os_error(),
        ));
    }

    Ok(Sid { buf })
}

/// Get the DACL of a file. [Read more](get_file_security_descriptor)
fn get_dacl(path: &Path) -> Result<Option<LocalFreeGuard<Security::ACL>>> {
    let security_descriptor =
        get_file_security_descriptor(path, Security::DACL_SECURITY_INFORMATION)?;

    // If file does not have a DACL, return None
    // SAFETY: security_descriptor was allocated by GetNamedSecurityInfoW and is owned by us.
    if unsafe { security_descriptor.as_ref() }.Dacl.is_null() {
        return Ok(None);
    };

    // SAFETY: security_descriptor was allocated by GetNamedSecurityInfoW and is owned by us.
    let dacl = security_descriptor.map(|mut sd| unsafe {
        NonNull::new(sd.as_mut().Dacl).expect("we've checkked that Dacl is Some")
    });

    Ok(Some(dacl))
}

/// Check if two ACLs are equal.
unsafe fn acl_eq(
    dacl: &LocalFreeGuard<Security::ACL>,
    actual_dacl: &LocalFreeGuard<Security::ACL>,
) -> bool {
    todo!()
}

/// Get the security descriptor of a file. `mask` defines which fields of the descriptor to get.
///
/// ## Example
/// ```ignore
/// let sd = get_file_security_descriptor(/* path */, Security::DACL_SECURITY_INFORMATION);
/// let dacl = unsafe { sd.as_ptr().as_ref_unchecked() }.Dacl;
/// ```
///
/// ## Further reading
/// - https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-security_descriptor
/// - https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-getnamedsecurityinfow
fn get_file_security_descriptor(
    path: &Path,
    mask: Security::OBJECT_SECURITY_INFORMATION,
) -> Result<LocalFreeGuard<SECURITY_DESCRIPTOR>> {
    let wide_path = U16CString::from_os_str(path)?;

    let mut security_descriptor: *mut SECURITY_DESCRIPTOR = ptr::null_mut();

    let result = unsafe {
        GetNamedSecurityInfoW(
            wide_path.as_ptr(),
            SE_FILE_OBJECT,
            mask,
            // These null_mut()s are helper args for getting pointers to fields inside the
            // security descriptor. Ignore them to stay sane.
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            (&raw mut security_descriptor).cast::<*mut c_void>(),
        )
    };

    if result != ERROR_SUCCESS {
        if result == ERROR_INSUFFICIENT_BUFFER {
            // https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-getnamedsecurityinfow
            todo!("This is a race condition in the syscall. Retry in this case.");
        }
        return Err(Error::GetDirPermissionFailed(
            format!("Could not security desciptor for file: {path:?}"),
            io::Error::last_os_error(),
        ));
    }

    let security_descriptor = NonNull::new(security_descriptor)
        .expect("GetNamedSecurityInfoW returns non-null on success");
    // SAFETY: GetNamedSecurityInfoW created this object and requires us to LocalFree it.
    let security_descriptor = unsafe { LocalFreeGuard::from_ptr(security_descriptor) };
    Ok(security_descriptor)
}

/// Build a windows ACL (access-control-list)
///
// TODO: explain what this function does
fn build_acl(user_permissions: UserPermissions) -> Result<LocalFreeGuard<Security::ACL>> {
    fn new_explicit_access_entry(sid: &Sid, access_permissions: u32) -> EXPLICIT_ACCESS_W {
        EXPLICIT_ACCESS_W {
            grfAccessPermissions: access_permissions,
            grfAccessMode: SET_ACCESS,
            grfInheritance: NO_INHERITANCE | SUB_CONTAINERS_AND_OBJECTS_INHERIT,
            Trustee: TRUSTEE_W {
                pMultipleTrustee: ptr::null_mut(),
                MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_GROUP,
                // TODO is casting from a SID to a string really legit???
                // Note that despite the type, this value is not actually mutated. FFI is just bad.
                ptstrName: sid.as_ptr().cast::<u16>().cast_mut(),
            },
        }
    }

    // Grant admin user full access
    let admin_sid = get_well_known_sid(WinBuiltinAdministratorsSid)?;
    let admin_ea = new_explicit_access_entry(&admin_sid, GENERIC_ALL);

    // Grant authenticated user the specified access
    let au_sid = get_well_known_sid(WinAuthenticatedUserSid)?;
    let authenticated_users_ea = new_explicit_access_entry(&au_sid, user_permissions.flags());

    let ea_entries = [admin_ea, authenticated_users_ea];
    let mut new_dacl = ptr::null_mut();

    // Build ACL from access entries.
    // SAFETY: `ea_entries` is valid for reads of `ea_entries.len()` elements
    // `new_dacl` is a valid pointer to an ACL pointer
    let result = unsafe {
        SetEntriesInAclW(
            u32::try_from(ea_entries.len()).expect("number of entries fits in u32"),
            ea_entries.as_ptr(),
            ptr::null(),
            &raw mut new_dacl,
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

    let new_dacl = NonNull::new(new_dacl).expect("SetEntriesInAclW creates an ACL on success");
    // SAFETY: new_dacl is a valid ACL pointer initialized by SetEntriesInAclW. We own it.
    let new_dacl = unsafe { LocalFreeGuard::from_ptr(new_dacl) };
    Ok(new_dacl)
}

/// Sets security permissions for path such that admin has full ownership and access while
/// authenticated users have the access specified by `user_permissions`.
fn set_security_permissions(path: &Path, user_permissions: UserPermissions) -> Result<()> {
    let wide_path = U16CString::from_os_str(path)?;
    let security_information = Security::DACL_SECURITY_INFORMATION
        | Security::PROTECTED_DACL_SECURITY_INFORMATION
        | Security::GROUP_SECURITY_INFORMATION
        | Security::OWNER_SECURITY_INFORMATION;
    let admin_sid = get_well_known_sid(WinBuiltinAdministratorsSid)?;
    let new_dacl = build_acl(user_permissions)?;

    // SAFETY: All pointers are valid.
    // Note that admin_sid is not actually mutated, the PSID
    // type is simply *mut because the windows FFI is bad.
    let result = unsafe {
        SetNamedSecurityInfoW(
            wide_path.as_ptr(),
            SE_FILE_OBJECT,
            security_information,
            admin_sid.as_ptr().cast_mut(),
            admin_sid.as_ptr().cast_mut(),
            new_dacl.as_ptr(),
            ptr::null(),
        )
    };

    if result != ERROR_SUCCESS {
        return Err(Error::SetDirPermissionFailed(
            String::from("SetNamedSecurityInfoW failed"),
            io::Error::from_raw_os_error(
                i32::try_from(result).expect("result does not fit in i32"),
            ),
        ));
    }

    Ok(())
}

/// Get local AppData path for the system service user.
pub fn get_system_service_appdata() -> io::Result<PathBuf> {
    static APPDATA_PATH: OnceCell<PathBuf> = OnceCell::new();

    APPDATA_PATH
        .get_or_try_init(|| {
            let join_handle = std::thread::spawn(|| {
                impersonate_self(|| {
                    let user_token = get_system_user_token()?;
                    get_known_folder_path(
                        &FOLDERID_LocalAppData,
                        KF_FLAG_DEFAULT,
                        user_token.as_ref().map(|t| t.as_handle()),
                    )
                })
                .or_else(|error| {
                    log::error!("Failed to get AppData path: {error}");
                    infer_appdata_from_system_directory()
                })
            });
            join_handle.join().expect("ImpersonateSelf should succeed")
        })
        .cloned()
}

/// Get user token for the system service user. Requires elevated privileges to work.
/// Useful for deducing the config path for the daemon on Windows when running as a user that
/// isn't the system service.
/// If the current user is system, this function succeeds and returns a NULL handle
fn get_system_user_token() -> io::Result<Option<OwnedHandle>> {
    let thread_token = get_current_thread_token()?;

    if is_local_system_user_token(&thread_token)? {
        return Ok(None);
    }

    let system_debug_priv = u16cstr!("SeDebugPrivilege");
    adjust_token_privilege(&thread_token, system_debug_priv, true)?;

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

    if let Err(err) = adjust_token_privilege(&thread_token, system_debug_priv, false) {
        log::error!("Failed to drop SeDebugPrivilege: {}", err);
    }

    find_result.map(Some)
}

fn open_process_token(process: &impl AsRawHandle, access: u32) -> io::Result<OwnedHandle> {
    let mut process_token = ptr::null_mut();
    // SAFETY: `process` is a valid handle
    if unsafe { OpenProcessToken(process.as_raw_handle(), access, &raw mut process_token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `process_token` is a valid handle since `OpenProcessToken` succeeded
    Ok(unsafe { OwnedHandle::from_raw_handle(process_token) })
}

/// If all else fails, infer the AppData path from the system directory.
fn infer_appdata_from_system_directory() -> io::Result<PathBuf> {
    let mut sysdir = get_known_folder_path(&FOLDERID_System, KF_FLAG_DEFAULT, None)?;
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
            &raw mut token_handle,
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
    if unsafe { LookupPrivilegeValueW(ptr::null(), privilege.as_ptr(), &raw mut privilege_luid) }
        == 0
    {
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
            &raw const privileges,
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

/// Get the path for a known Windows folder, with an optional user token.
///
/// `folder_id` must point to a valid KNOWNFOLDERID, or this will return an error.
pub fn get_known_folder_path(
    folder_id: &GUID,
    flags: i32,
    user_token: Option<BorrowedHandle<'_>>,
) -> io::Result<PathBuf> {
    let mut path_ptr: windows_sys::core::PWSTR = ptr::null_mut();
    // SAFETY: `folder_id` is a valid GUID;
    // null token uses the calling thread's identity; `&mut path_ptr` is a
    // stack-local for the API to fill in with a CoTaskMem-allocated PWSTR.
    let status = unsafe {
        SHGetKnownFolderPath(
            folder_id,
            flags as u32,
            user_token
                .map(|h| h.as_raw_handle())
                .unwrap_or(ptr::null_mut()),
            &raw mut path_ptr,
        )
    };

    let result = if status == S_OK {
        // SAFETY: on success `path_ptr` points to a null-terminated wide
        // string allocated by `SHGetKnownFolderPath`, valid until the
        // `CoTaskMemFree` below.
        let wide = unsafe { WideCStr::from_ptr_str(path_ptr) };
        Ok(PathBuf::from(wide.to_os_string()))
    } else {
        Err(io::Error::from_raw_os_error(status))
    };

    // `SHGetKnownFolderPath` requires the caller to free the buffer even on
    // failure (the API may allocate before returning an error code). A null
    // `path_ptr` is a documented no-op for `CoTaskMemFree`.
    // SAFETY: `path_ptr` was either allocated by `SHGetKnownFolderPath`
    // (CoTaskMem) or is null; either way `CoTaskMemFree` is sound.
    unsafe { CoTaskMemFree(path_ptr.cast()) };
    result
}

/// Enumerate over all processes until `handle_process` returns a result or until there are
/// no more processes left. In the latter case, an error is returned.
fn find_process<T>(handle_process: impl Fn(BorrowedHandle<'_>) -> Option<T>) -> io::Result<T> {
    let mut pid_buffer = vec![0u32; 2048];
    let mut num_procs: u32 =
        u32::try_from(pid_buffer.len()).expect("Number of processes IDs fit in u32");

    let bytes_available = num_procs * (mem::size_of::<u32>() as u32);
    let mut bytes_written = 0;

    // SAFETY: `pid_buffer` is valid for writes of `bytes_available` bytes
    if unsafe {
        EnumProcesses(
            pid_buffer.as_mut_ptr(),
            bytes_available,
            &raw mut bytes_written,
        )
    } == 0
    {
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
                token_info.as_mut_ptr().cast::<c_void>(),
                u32::try_from(token_info.len()).expect("len must fit in u32"),
                &raw mut returned_info_len,
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
    let mut local_system_size =
        u32::try_from(local_system_sid.len()).expect("local_system_sid length fits in u32");

    // SAFETY: `local_system_sid` is valid for writes of `local_system_size` bytes
    if unsafe {
        CreateWellKnownSid(
            WinLocalSystemSid,
            std::ptr::null_mut(),
            local_system_sid.as_mut_ptr().cast::<c_void>(),
            &raw mut local_system_size,
        )
    } == 0
    {
        let err = io::Error::last_os_error();
        log::error!("CreateWellKnownSid failed: {}", err);
        return Err(err);
    }

    // SAFETY: Both arguments point to valid security identifiers
    Ok(unsafe {
        EqualSid(
            token_user.User.Sid,
            local_system_sid.as_mut_ptr().cast::<c_void>(),
        )
    } != 0)
}
