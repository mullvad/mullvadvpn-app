//! Cleanup operations for the Mullvad VPN installer.
//!
//! Exports:
//! - `RemoveLogsAndCache` - remove all logs and cache for all users
//! - `RemoveSettings` - remove service user settings
//! - `RemoveRelayCache` - remove relay cache file
//! - `RemoveApiAddressCache` - remove API address cache file
//! - `CloseHoggingProcesses` - close processes blocking the install directory
//! - `IsEmptyDir` - check if a directory contains only subdirectories (no files)

use std::fs;
use std::io;
use std::path::Path;
use std::ptr;

use anyhow::Context;
use nsis_plugin_api::{nsis_fn, popint, popstr, pushint, pushstr};
use widestring::U16CString;
use windows_sys::Win32::Foundation::{ERROR_SUCCESS, GENERIC_ALL};
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
use windows_sys::Win32::UI::Shell::{
    FOLDERID_LocalAppData, FOLDERID_Profile, FOLDERID_RoamingAppData,
};

use crate::{NsisStatus, get_known_folder_path};

/// Disables WOW64 filesystem redirection for the lifetime of this guard.
/// Necessary for a 32-bit process to access real System32 paths on 64-bit Windows.
struct ScopedNativeFileSystem {
    old_value: *mut std::ffi::c_void,
    active: bool,
}

impl ScopedNativeFileSystem {
    fn new() -> Self {
        let mut old_value: *mut std::ffi::c_void = ptr::null_mut();
        // SAFETY: `&raw mut old_value` points to a stack-local for the API to
        // fill in with the previous redirection cookie.
        let result = unsafe { Wow64DisableWow64FsRedirection(&raw mut old_value) };
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

mod guard {
    use windows_sys::Win32::Foundation::LocalFree;

    /// Frees a local-heap allocation (via LocalFree) on drop.
    pub struct LocalFreeGuard<T>(*mut T);

    impl<T> LocalFreeGuard<T> {
        /// Create a guard for an object that's destroyed with `LocalFree`
        /// when this is dropped.
        ///
        /// # Safety
        ///
        /// It must be correct to free `p` with `LocalFree`. This function transfers ownership of
        /// `p` to this object.
        pub unsafe fn from_ptr(p: *mut T) -> Self {
            LocalFreeGuard(p)
        }

        /// Return the underlying pointer.
        pub fn as_ptr(&self) -> *const T {
            self.0
        }
    }

    impl<T> Drop for LocalFreeGuard<T> {
        fn drop(&mut self) {
            // SAFETY: `self.0` was allocated by a Win32 function that returns
            // a LocalAlloc-style handle (e.g. `SetEntriesInAclW`); we own it
            // uniquely and have not freed it.
            unsafe { LocalFree(self.0.cast()) };
        }
    }
}

use guard::LocalFreeGuard;

/// Self-relative security descriptor returned by `GetNamedSecurityInfoW`.
/// The borrowed DACL pointer lives as long as `self`.
struct SecurityDescriptor {
    dacl: *mut ACL,
    _sd: LocalFreeGuard<std::ffi::c_void>,
}

impl SecurityDescriptor {
    /// Fetch the DACL-only security descriptor for a filesystem path.
    fn from_path(path: &Path) -> io::Result<Self> {
        let path_wide = U16CString::from_os_str_truncate(path);
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
                &raw mut dacl,
                ptr::null_mut(),
                &raw mut sd,
            )
        };
        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        // SAFETY: The security descriptor must be freed with `LocalFree`.
        let _sd = unsafe { LocalFreeGuard::from_ptr(sd) };
        Ok(Self { _sd, dacl })
    }

    /// Borrow the DACL pointer (valid for `&self`'s lifetime).
    fn dacl(&self) -> *mut ACL {
        self.dacl
    }
}

/// Add Administrators group to the DACL of a filesystem path, granting full control.
fn add_admin_to_object_dacl(path: &Path) -> io::Result<()> {
    let sd = SecurityDescriptor::from_path(path)?;

    let path_wide = U16CString::from_os_str_truncate(path);

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
            &raw mut sid_size,
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
    let result = unsafe { SetEntriesInAclW(1, &raw const ea, sd.dacl(), &raw mut new_dacl) };

    if result != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(result as i32));
    }

    // SAFETY: This object must be destroyed with `new_dacl`.
    let new_dacl = unsafe { LocalFreeGuard::from_ptr(new_dacl.cast()) };

    // SAFETY: `path_wide` is a null-terminated wide string; only the DACL
    // pointer is non-null (the API only modifies the DACL portion).
    let result = unsafe {
        SetNamedSecurityInfoW(
            path_wide.as_ptr().cast_mut(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            new_dacl.as_ptr(),
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
    let local_appdata =
        get_known_folder_path(&FOLDERID_LocalAppData).context("FOLDERID_LocalAppData")?;
    remove_dir_all_if_exists(&local_appdata.join("Mullvad VPN"))?;

    let roaming_appdata =
        get_known_folder_path(&FOLDERID_RoamingAppData).context("FOLDERID_RoamingAppData")?;
    remove_dir_all_if_exists(&roaming_appdata.join("Mullvad VPN"))?;

    Ok(())
}

/// Remove Mullvad VPN data from all other users' app data directories.
fn remove_logs_cache_other_users() -> anyhow::Result<()> {
    let home_dir = get_known_folder_path(&FOLDERID_Profile).context("FOLDERID_Profile")?;
    let local_appdata =
        get_known_folder_path(&FOLDERID_LocalAppData).context("FOLDERID_LocalAppData")?;
    let roaming_appdata = get_known_folder_path(&FOLDERID_RoamingAppData).ok();

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
    let log_dir = mullvad_paths::get_default_log_dir().context("get allusersprofile log dir")?;

    // Remove only files; leave subdirectories untouched.
    let entries = match std::fs::read_dir(&log_dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(anyhow::Error::from(e))
                .with_context(|| format!("read_dir {}", log_dir.display()));
        }
    };

    for entry in entries
        .flatten()
        .filter(|entry| entry.file_type().as_ref().is_ok_and(fs::FileType::is_file))
    {
        std::fs::remove_file(entry.path())
            .with_context(|| format!("remove {}", entry.path().display()))?;
    }

    // Try to remove the now-empty directory; ignore failure if non-empty.
    let _ = std::fs::remove_dir(log_dir);
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
    let mullvad_appdata =
        mullvad_paths::get_default_settings_dir().context("get system service appdata")?;

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
    let result = [
        remove_logs_cache_current_user(),
        remove_logs_cache_other_users(),
        remove_cache_service_user(),
        remove_logs_service_user(),
    ]
    .into_iter()
    .collect::<anyhow::Result<()>>();

    let status = if result.is_ok() {
        NsisStatus::Success
    } else {
        NsisStatus::GeneralError
    };
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
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
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
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
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
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
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
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
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    let (install_path, allow_cancellation) = unsafe { (popstr()?, popint()? != 0) };

    let (message, status) =
        match crate::handle::terminate_processes(&install_path, allow_cancellation) {
            Ok(true) => (String::new(), NsisStatus::Success),
            Ok(false) => (String::from("Cancelled"), NsisStatus::Cancelled),
            Err(e) => (format!("{e:#}"), NsisStatus::GeneralError),
        };
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
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
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    let path = unsafe { popstr()? };

    let status = match crate::handle::is_empty_dir(&path) {
        Ok(true) => NsisStatus::Success,
        Ok(false) => NsisStatus::FileExists,
        Err(_) => NsisStatus::GeneralError,
    };
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    unsafe { pushint(status as i32) }
}
