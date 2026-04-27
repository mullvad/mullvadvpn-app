//! Cleanup operations for the Mullvad VPN installer.
//!
//! Exports:
//! - `RemoveLogsAndCache` - remove all logs and cache for all users
//! - `RemoveSettings` - remove service user settings
//! - `RemoveRelayCache` - remove relay cache file
//! - `RemoveApiAddressCache` - remove API address cache file
//! - `CloseHoggingProcesses` - close processes blocking the install directory
//! - `IsEmptyDir` - check if a directory contains only subdirectories (no files)

use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr;

use anyhow::Context;
use nsis_plugin_api::{nsis_fn, popint, popstr, pushint, pushstr};
use widestring::U16CStr;
use windows_sys::Win32::Foundation::{ERROR_SUCCESS, GENERIC_ALL, LocalFree, S_OK};
use windows_sys::Win32::Security::Authorization::{
    EXPLICIT_ACCESS_W, GRANT_ACCESS, GetNamedSecurityInfoW, NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT,
    SetEntriesInAclW, SetNamedSecurityInfoW, TRUSTEE_IS_GROUP, TRUSTEE_IS_SID, TRUSTEE_W,
};
use windows_sys::Win32::Security::{
    ACL, CreateWellKnownSid, DACL_SECURITY_INFORMATION, NO_INHERITANCE,
    SUB_CONTAINERS_AND_OBJECTS_INHERIT, WinBuiltinAdministratorsSid,
};
use windows_sys::Win32::Storage::FileSystem::MAX_SID_SIZE;
use windows_sys::Win32::Storage::FileSystem::{
    Wow64DisableWow64FsRedirection, Wow64RevertWow64FsRedirection,
};
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::UI::Shell::{
    FOLDERID_LocalAppData, FOLDERID_Profile, FOLDERID_RoamingAppData, KF_FLAG_DEFAULT,
    SHGetKnownFolderPath,
};

use crate::NsisStatus;

/// Disables WOW64 filesystem redirection for the lifetime of this guard.
/// Necessary for a 32-bit process to access real System32 paths on 64-bit Windows.
struct ScopedNativeFileSystem {
    old_value: *mut std::ffi::c_void,
    active: bool,
}

impl ScopedNativeFileSystem {
    fn new() -> Self {
        let mut old_value: *mut std::ffi::c_void = ptr::null_mut();
        // SAFETY: `&mut old_value` points to a stack-local for the API to
        // fill in with the previous redirection cookie.
        let result = unsafe { Wow64DisableWow64FsRedirection(&mut old_value) };
        ScopedNativeFileSystem {
            old_value,
            active: result != 0,
        }
    }
}

impl Drop for ScopedNativeFileSystem {
    fn drop(&mut self) {
        if self.active {
            // SAFETY: `self.old_value` is the cookie returned by
            // `Wow64DisableWow64FsRedirection` above; pairing them is the
            // documented contract.
            unsafe { Wow64RevertWow64FsRedirection(self.old_value) };
        }
    }
}

/// Frees a local-heap allocation (via LocalFree) on drop.
struct LocalFreeGuard(*mut std::ffi::c_void);

impl LocalFreeGuard {
    fn from_ptr(p: *mut std::ffi::c_void) -> Self {
        LocalFreeGuard(p)
    }
}

impl Drop for LocalFreeGuard {
    fn drop(&mut self) {
        // SAFETY: `self.0` was allocated by a Win32 function that returns
        // a LocalAlloc-style handle (e.g. `SetEntriesInAclW`); we own it
        // uniquely and have not freed it.
        unsafe { LocalFree(self.0) };
    }
}

/// Self-relative security descriptor returned by `GetNamedSecurityInfoW`.
/// The borrowed DACL pointer lives as long as `self`.
struct SecurityDescriptor {
    sd: *mut std::ffi::c_void,
    dacl: *mut ACL,
}

impl SecurityDescriptor {
    /// Fetch the DACL-only security descriptor for a filesystem path.
    fn from_path(path: &Path) -> io::Result<Self> {
        let path_wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut dacl: *mut ACL = ptr::null_mut();
        let mut sd: *mut std::ffi::c_void = ptr::null_mut();
        // SAFETY: `path_wide` is a null-terminated wide string. Owner/group/
        // sacl out-pointers are null (we only ask for the DACL). `&mut dacl`
        // and `&mut sd` are stack-locals; on success the API LocalAlloc's
        // `sd` (freed in Drop) and points `dacl` into that allocation.
        let result = unsafe {
            GetNamedSecurityInfoW(
                path_wide.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut dacl,
                ptr::null_mut(),
                &mut sd,
            )
        };
        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        Ok(Self { sd, dacl })
    }

    /// Borrow the DACL pointer (valid for `&self`'s lifetime).
    fn dacl(&self) -> *mut ACL {
        self.dacl
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        // SAFETY: `self.sd` was allocated by `GetNamedSecurityInfoW` and
        // is owned uniquely by `self`.
        unsafe { LocalFree(self.sd) };
    }
}

/// Get the path for a known Windows folder.
///
/// # Safety
///
/// `folder_id` must point to a valid KNOWNFOLDERID GUID.
unsafe fn get_known_folder_path(folder_id: *const windows_sys::core::GUID) -> io::Result<PathBuf> {
    let mut path_ptr: windows_sys::core::PWSTR = ptr::null_mut();
    // SAFETY: `folder_id` is a valid KNOWNFOLDERID per this fn's contract;
    // null token uses the calling thread's identity; `&mut path_ptr` is a
    // stack-local for the API to fill in with a CoTaskMem-allocated PWSTR.
    let status = unsafe {
        SHGetKnownFolderPath(
            folder_id,
            KF_FLAG_DEFAULT as u32,
            ptr::null_mut(),
            &mut path_ptr,
        )
    };

    let result = if status == S_OK {
        // SAFETY: on success `path_ptr` points to a null-terminated wide
        // string allocated by `SHGetKnownFolderPath`, valid until the
        // `CoTaskMemFree` below.
        let wide = unsafe { U16CStr::from_ptr_str(path_ptr) };
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

/// Add Administrators group to the DACL of a filesystem path, granting full control.
fn add_admin_to_object_dacl(path: &Path) -> io::Result<()> {
    let sd = SecurityDescriptor::from_path(path)?;

    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Build the Administrators SID.
    let mut admin_sid = [0u8; MAX_SID_SIZE as usize];
    let mut sid_size = u32::try_from(admin_sid.len()).unwrap_or(u32::MAX);

    // SAFETY: `admin_sid` is `MAX_SID_SIZE` bytes of writable stack storage,
    // and `sid_size` is initialized to that capacity.
    if unsafe {
        CreateWellKnownSid(
            WinBuiltinAdministratorsSid,
            ptr::null_mut(),
            admin_sid.as_mut_ptr().cast(),
            &mut sid_size,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }

    let trustee = TRUSTEE_W {
        pMultipleTrustee: ptr::null_mut(),
        MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
        TrusteeForm: TRUSTEE_IS_SID,
        TrusteeType: TRUSTEE_IS_GROUP,
        ptstrName: admin_sid.as_mut_ptr().cast(),
    };

    let ea = EXPLICIT_ACCESS_W {
        grfAccessPermissions: GENERIC_ALL,
        grfAccessMode: GRANT_ACCESS,
        grfInheritance: NO_INHERITANCE | SUB_CONTAINERS_AND_OBJECTS_INHERIT,
        Trustee: trustee,
    };

    let mut new_dacl = ptr::null_mut();
    // SAFETY: `&ea` points to one fully-initialized `EXPLICIT_ACCESS_W`,
    // `sd.dacl()` is the DACL borrowed from `sd` (live for `&sd`'s scope),
    // and `&mut new_dacl` is a stack-local for the API to fill in.
    let result = unsafe { SetEntriesInAclW(1, &ea, sd.dacl(), &mut new_dacl) };

    if result != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(result as i32));
    }

    let _dacl_guard = LocalFreeGuard::from_ptr(new_dacl.cast());

    // SAFETY: `path_wide` is a null-terminated wide string; only the DACL
    // pointer is non-null (the API only modifies the DACL portion).
    let result = unsafe {
        SetNamedSecurityInfoW(
            path_wide.as_ptr().cast_mut(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            new_dacl,
            ptr::null_mut(),
        )
    };
    if result != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    Ok(())
}

/// `remove_dir_all` that treats a missing path as success.
fn remove_dir_all_if_exists(path: &Path) -> anyhow::Result<()> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(anyhow::Error::from(e)).with_context(|| format!("remove {}", path.display())),
    }
}

/// Remove Mullvad VPN data from the current user's LocalAppData and RoamingAppData.
fn remove_logs_cache_current_user() -> anyhow::Result<()> {
    // SAFETY: `FOLDERID_LocalAppData` is a valid KNOWNFOLDERID static.
    let local_appdata = unsafe { get_known_folder_path(&FOLDERID_LocalAppData) }
        .context("FOLDERID_LocalAppData")?;
    remove_dir_all_if_exists(&local_appdata.join("Mullvad VPN"))?;

    // SAFETY: `FOLDERID_RoamingAppData` is a valid KNOWNFOLDERID static.
    let roaming_appdata = unsafe { get_known_folder_path(&FOLDERID_RoamingAppData) }
        .context("FOLDERID_RoamingAppData")?;
    remove_dir_all_if_exists(&roaming_appdata.join("Mullvad VPN"))?;

    Ok(())
}

/// Remove Mullvad VPN data from all other users' app data directories.
fn remove_logs_cache_other_users() -> anyhow::Result<()> {
    // SAFETY: `FOLDERID_Profile` is a valid KNOWNFOLDERID static.
    let home_dir =
        unsafe { get_known_folder_path(&FOLDERID_Profile) }.context("FOLDERID_Profile")?;

    // SAFETY: `FOLDERID_LocalAppData` is a valid KNOWNFOLDERID static.
    let local_appdata = unsafe { get_known_folder_path(&FOLDERID_LocalAppData) }
        .context("FOLDERID_LocalAppData")?;

    // SAFETY: `FOLDERID_RoamingAppData` is a valid KNOWNFOLDERID static.
    let roaming_appdata = unsafe { get_known_folder_path(&FOLDERID_RoamingAppData) }.ok();

    // Find relative path from home to LocalAppData (e.g., "AppData\Local")
    let rel_local = local_appdata
        .strip_prefix(&home_dir)
        .context("LocalAppData not under home")?
        .to_owned();

    let rel_roaming = roaming_appdata
        .as_ref()
        .and_then(|r| r.strip_prefix(&home_dir).ok().map(|p| p.to_owned()));

    let current_user = home_dir
        .file_name()
        .context("home directory has no file name")?
        .to_owned();

    // The users directory is the parent of the user's home directory.
    let users_dir = home_dir
        .parent()
        .context("home directory has no parent")?
        .to_owned();

    let entries = std::fs::read_dir(&users_dir)
        .with_context(|| format!("read_dir {}", users_dir.display()))?;

    for entry in entries.flatten() {
        let file_name = entry.file_name();

        if file_name == current_user || file_name == "All Users" || file_name == "Public" {
            continue;
        }

        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if !file_type.is_dir() {
            continue;
        }

        let user_dir = entry.path();
        remove_dir_all_if_exists(&user_dir.join(&rel_local).join("Mullvad VPN"))?;

        if let Some(rel_roam) = &rel_roaming {
            remove_dir_all_if_exists(&user_dir.join(rel_roam).join("Mullvad VPN"))?;
        }
    }

    Ok(())
}

/// Remove log files from the service user's ProgramData\Mullvad VPN directory.
fn remove_logs_service_user() -> anyhow::Result<()> {
    let program_data =
        mullvad_paths::windows::get_allusersprofile_dir().context("get allusersprofile dir")?;
    let app_dir = program_data.join("Mullvad VPN");

    // Remove only files; leave subdirectories untouched.
    let entries = match std::fs::read_dir(&app_dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(anyhow::Error::from(e))
                .with_context(|| format!("read_dir {}", app_dir.display()));
        }
    };
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            std::fs::remove_file(entry.path())
                .with_context(|| format!("remove {}", entry.path().display()))?;
        }
    }

    // Try to remove the now-empty directory; ignore failure if non-empty.
    let _ = std::fs::remove_dir(&app_dir);
    Ok(())
}

/// Remove the service user's cache directory (ProgramData\Mullvad VPN\cache).
fn remove_cache_service_user() -> anyhow::Result<()> {
    let cache_dir = mullvad_paths::get_default_cache_dir()?;
    remove_dir_all_if_exists(&cache_dir)?;

    // Try to remove the parent Mullvad VPN directory if now empty; ignore
    // failure (it may contain other files).
    if let Some(parent) = cache_dir.parent() {
        let _ = std::fs::remove_dir(parent);
    }
    Ok(())
}

/// Remove the service user's settings directory.
fn remove_settings_service_user() -> anyhow::Result<()> {
    let local_appdata = mullvad_paths::windows::get_system_service_appdata()
        .context("get system service appdata")?;
    let mullvad_appdata = local_appdata.join("Mullvad VPN");

    let _native = ScopedNativeFileSystem::new();
    add_admin_to_object_dacl(&mullvad_appdata)
        .with_context(|| format!("add admin to DACL of {}", mullvad_appdata.display()))?;

    std::fs::remove_dir_all(&mullvad_appdata)
        .with_context(|| format!("remove_dir_all {}", mullvad_appdata.display()))
}

fn remove_relay_cache_service_user() -> anyhow::Result<()> {
    let cache_file = mullvad_paths::get_default_cache_dir()?.join("relays.json");
    std::fs::remove_file(&cache_file)
        .or_else(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        })
        .with_context(|| format!("remove {}", cache_file.display()))
}

fn remove_api_address_cache_service_user() -> anyhow::Result<()> {
    let cache_file = mullvad_paths::get_default_cache_dir()?.join("api-ip-address.txt");
    std::fs::remove_file(&cache_file)
        .or_else(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        })
        .with_context(|| format!("remove {}", cache_file.display()))
}

// ============================================================================
// NSIS-exported functions
// ============================================================================

// RemoveLogsAndCache
//
// Removes all logs and cache for current user, other users, and the service user.
// Pushes a status code.
#[nsis_fn]
fn RemoveLogsAndCache() -> Result<(), nsis_plugin_api::Error> {
    let mut success = true;
    for result in [
        remove_logs_cache_current_user(),
        remove_logs_cache_other_users(),
        remove_cache_service_user(),
        remove_logs_service_user(),
    ] {
        if result.is_err() {
            success = false;
        }
    }

    let status = if success {
        NsisStatus::Success
    } else {
        NsisStatus::GeneralError
    };
    // SAFETY: `exdll_init` was called.
    unsafe { pushint(status as i32) }
}

// RemoveSettings
//
// Removes the service user's settings directory.
// Pushes a status code.
#[nsis_fn]
fn RemoveSettings() -> Result<(), nsis_plugin_api::Error> {
    let status = match remove_settings_service_user() {
        Ok(()) => NsisStatus::Success,
        Err(_) => NsisStatus::GeneralError,
    };
    // SAFETY: `exdll_init` was called.
    unsafe { pushint(status as i32) }
}

// RemoveRelayCache
//
// Removes the relay cache file.
// Pushes error message and status code.
#[nsis_fn]
fn RemoveRelayCache() -> Result<(), nsis_plugin_api::Error> {
    let (message, status) = match remove_relay_cache_service_user() {
        Ok(()) => (String::new(), NsisStatus::Success),
        Err(e) => (format!("{e:#}"), NsisStatus::GeneralError),
    };
    // SAFETY: `exdll_init` was called.
    unsafe {
        pushstr(&message)?;
        pushint(status as i32)
    }
}

// RemoveApiAddressCache
//
// Removes the API address cache file.
// Pushes error message and status code.
#[nsis_fn]
fn RemoveApiAddressCache() -> Result<(), nsis_plugin_api::Error> {
    let (message, status) = match remove_api_address_cache_service_user() {
        Ok(()) => (String::new(), NsisStatus::Success),
        Err(e) => (format!("{e:#}"), NsisStatus::GeneralError),
    };
    // SAFETY: `exdll_init` was called.
    unsafe {
        pushstr(&message)?;
        pushint(status as i32)
    }
}

// CloseHoggingProcesses "installPath" allowCancellation
//
// Identifies and closes processes blocking files in the install path.
// allowCancellation: 1 = show Yes/No dialog, 0 = show OK-only dialog.
// Pushes error message and status code.
#[nsis_fn]
fn CloseHoggingProcesses() -> Result<(), nsis_plugin_api::Error> {
    // SAFETY: `exdll_init` was called.
    let (install_path, allow_cancellation) = unsafe { (popstr()?, popint()? != 0) };

    let (message, status) =
        match crate::handle::terminate_processes(&install_path, allow_cancellation) {
            Ok(true) => (String::new(), NsisStatus::Success),
            Ok(false) => (String::from("Cancelled"), NsisStatus::Cancelled),
            Err(e) => (format!("{e:#}"), NsisStatus::GeneralError),
        };
    // SAFETY: `exdll_init` was called.
    unsafe {
        pushstr(&message)?;
        pushint(status as i32)
    }
}

// IsEmptyDir "path"
//
// Checks if the directory contains no files (only directories/symlinks).
// Pushes SUCCESS if empty, FILE_EXISTS if files found, GENERAL_ERROR on error.
#[nsis_fn]
fn IsEmptyDir() -> Result<(), nsis_plugin_api::Error> {
    // SAFETY: `exdll_init` was called.
    let path = unsafe { popstr()? };

    let status = match crate::handle::is_empty_dir(&path) {
        Ok(true) => NsisStatus::Success,
        Ok(false) => NsisStatus::FileExists,
        Err(_) => NsisStatus::GeneralError,
    };
    // SAFETY: `exdll_init` was called.
    unsafe { pushint(status as i32) }
}
