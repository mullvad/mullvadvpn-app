//! Code for migrating between different versions of the settings.
//! Migration only supports migrating forward, to newer formats.
//!
//! A settings migration module is responsible for converting
//! from its own version to the next version. So `v3::migrate`
//! migrates from settings version `V3` to `V4` etc.
//!
//! Migration modules may NOT import and use structs that may
//! change. Because then a later change to the current code can break
//! old migrations. The only items a settings migration module may import
//! are anything from `std`, `jnix`, `serde` and the following:
//!
//! ```ignore
//! use super::{Error, Result};
//! use mullvad_types::relay_constraints::Constraint;
//! use mullvad_types::settings::SettingsVersion;
//! ```
//!
//! Any other type must be vendored into the migration module so the format
//! it has is locked over time.
//!
//! There should never be multiple migrations between two official releases. At most one.
//! Between releases, dev builds can break the settings without having a proper migration path.
//!
//! # Creating a migration
//!
//! 1. Copy `vX.rs.template` to `vX.rs` where `X` is the latest settings version right now.
//! 1. Add the new version (`Y = X+1`) to `SettingsVersion` and bump `CURRENT_SETTINGS_VERSION`
//!    to `Y`.
//! 1. Write a comment in the new module about how the format changed, what it needs to migrate.
//! 1. Implement the migration and add adequate tests.
//! 1. Add to the changelog: "Settings format updated to `vY`"

use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::{
    fs,
    io::{self, AsyncWriteExt},
};

mod account_history;
mod device;
mod v1;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;

const SETTINGS_FILE: &str = "settings.json";

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to read the settings")]
    Read(#[error(source)] io::Error),

    #[error(display = "Failed to deserialize settings")]
    Deserialize(#[error(source)] serde_json::Error),

    #[error(display = "Unexpected settings format")]
    InvalidSettingsContent,

    #[error(display = "Unable to serialize settings to JSON")]
    Serialize(#[error(source)] serde_json::Error),

    #[error(display = "Unable to open settings for writing")]
    Open(#[error(source)] io::Error),

    #[error(display = "Unable to write new settings")]
    Write(#[error(source)] io::Error),

    #[error(display = "Unable to sync settings to disk")]
    SyncSettings(#[error(source)] io::Error),

    #[error(display = "Failed to read the account history")]
    ReadHistory(#[error(source)] io::Error),

    #[error(display = "Failed to write new account history")]
    WriteHistory(#[error(source)] io::Error),

    #[error(display = "Failed to parse account history")]
    ParseHistoryError,

    #[cfg(windows)]
    #[error(display = "Failed to restore Windows update backup")]
    WinMigrationError(#[error(source)] windows::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Returns whether there is any background work remaining.
#[derive(Clone)]
pub struct MigrationComplete(Arc<AtomicBool>);

impl MigrationComplete {
    pub fn new(state: bool) -> Self {
        Self(Arc::new(AtomicBool::new(state)))
    }

    pub fn is_complete(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    fn set_complete(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// Contains discarded data that may be useful for later work.
pub type MigrationData = v5::MigrationData;

pub async fn migrate_all(cache_dir: &Path, settings_dir: &Path) -> Result<Option<MigrationData>> {
    #[cfg(windows)]
    windows::migrate_after_windows_update(settings_dir)
        .await
        .map_err(Error::WinMigrationError)?;

    let path = settings_dir.join(SETTINGS_FILE);

    if !path.is_file() {
        return Ok(None);
    }

    let settings_bytes = fs::read(&path).await.map_err(Error::Read)?;

    let mut settings: serde_json::Value =
        serde_json::from_reader(&settings_bytes[..]).map_err(Error::Deserialize)?;

    if !settings.is_object() {
        return Err(Error::InvalidSettingsContent);
    }

    let old_settings = settings.clone();

    v1::migrate(&mut settings)?;
    v2::migrate(&mut settings)?;
    v3::migrate(&mut settings)?;
    v4::migrate(&mut settings)?;

    account_history::migrate_location(cache_dir, settings_dir).await;
    account_history::migrate_formats(settings_dir, &mut settings).await?;

    let migration_data = v5::migrate(&mut settings)?;
    v6::migrate(&mut settings)?;

    if settings == old_settings {
        // Nothing changed
        return Ok(migration_data);
    }

    let buffer = serde_json::to_string_pretty(&settings).map_err(Error::Serialize)?;

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .await
        .map_err(Error::Open)?;
    file.write_all(&buffer.into_bytes())
        .await
        .map_err(Error::Write)?;
    file.sync_data().await.map_err(Error::SyncSettings)?;

    log::debug!("Migrated settings. Wrote settings to {}", path.display());

    Ok(migration_data)
}

pub(crate) fn migrate_device(
    migration_data: MigrationData,
    rest_handle: mullvad_api::rest::MullvadRestHandle,
    daemon_tx: crate::DaemonEventSender,
) -> MigrationComplete {
    let migration_complete = MigrationComplete::new(false);
    device::generate_device(
        migration_data,
        migration_complete.clone(),
        rest_handle,
        daemon_tx,
    );
    migration_complete
}

#[cfg(windows)]
mod windows {
    use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, path::Path, ptr};
    use talpid_types::ErrorExt;
    use tokio::fs;
    use windows_sys::Win32::{
        Foundation::{ERROR_SUCCESS, HANDLE, PSID},
        Security::{
            Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT, SE_OBJECT_TYPE},
            IsWellKnownSid, WinBuiltinAdministratorsSid, WinLocalSystemSid,
            OWNER_SECURITY_INFORMATION, SECURITY_DESCRIPTOR, SID, WELL_KNOWN_SID_TYPE,
        },
        System::Memory::LocalFree,
    };

    #[allow(non_camel_case_types)]
    type SECURITY_INFORMATION = u32;

    const MIGRATION_DIRNAME: &str = "windows.old";
    const MIGRATE_FILES: [(&str, bool); 3] = [
        ("settings.json", true),
        ("device.json", true),
        ("account-history.json", false),
    ];

    #[derive(err_derive::Error, Debug)]
    #[error(no_from)]
    pub enum Error {
        #[error(display = "Unable to find local appdata directory")]
        FindAppData,

        #[error(display = "Could not acquire security descriptor of backup directory")]
        SecurityInformation(#[error(source)] io::Error),

        #[error(display = "Backup directory is not owned by SYSTEM or Built-in Administrators")]
        WrongOwner,

        #[error(display = "Failed to copy files during migration")]
        IoError(#[error(source)] io::Error),
    }

    /// Attempts to restore the Mullvad settings from `C:\windows.old` after an update of Windows.
    /// Upon success, it returns `Ok(true)` if the migration succeeded, and `Ok(false)` if no
    /// migration was needed.
    pub async fn migrate_after_windows_update(
        destination_settings_dir: &Path,
    ) -> Result<bool, Error> {
        let system_appdata_dir = dirs_next::data_local_dir().ok_or(Error::FindAppData)?;
        if !destination_settings_dir.starts_with(system_appdata_dir) {
            return Ok(false);
        }

        let settings_path = destination_settings_dir.join(super::SETTINGS_FILE);
        if settings_path.exists() {
            return Ok(false);
        }

        let mut components = destination_settings_dir.components();
        let prefix = if let Some(prefix) = components.next() {
            prefix
        } else {
            return Ok(false);
        };
        let root = if let Some(root) = components.next() {
            root
        } else {
            return Ok(false);
        };

        let windows_old_dir = Path::new(&prefix).join(&root).join(MIGRATION_DIRNAME);
        let source_settings_dir = Path::new(&windows_old_dir).join(&components);
        if !source_settings_dir.exists() {
            return Ok(false);
        }

        let security_info =
            SecurityInformation::from_file(windows_old_dir.as_path(), OWNER_SECURITY_INFORMATION)
                .map_err(Error::SecurityInformation)?;

        let owner_sid = security_info.owner().ok_or(Error::WrongOwner)?;

        if !is_well_known_sid(owner_sid, WinLocalSystemSid)
            && !is_well_known_sid(owner_sid, WinBuiltinAdministratorsSid)
        {
            return Err(Error::WrongOwner);
        }

        if !destination_settings_dir.exists() {
            fs::create_dir_all(destination_settings_dir)
                .await
                .map_err(Error::IoError)?;
        }

        let mut result = Ok(true);

        for (file, required) in &MIGRATE_FILES {
            let from = source_settings_dir.join(file);
            let to = destination_settings_dir.join(file);

            log::debug!("Migrating {} to {}", from.display(), to.display());

            match fs::copy(&from, &to).await {
                Ok(_) => {
                    let _ = fs::remove_file(from).await;
                }
                Err(error) => {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(&format!(
                            "Failed to copy {} to {}",
                            from.display(),
                            to.display()
                        ))
                    );
                    if *required {
                        result = Err(Error::IoError(error));
                    }
                }
            }
        }

        if let Err(error) = fs::remove_dir(source_settings_dir).await {
            log::trace!(
                "{}",
                error.display_chain_with_msg("Failed to delete backup directory")
            );
        }

        result
    }

    struct SecurityInformation {
        security_descriptor: *mut SECURITY_DESCRIPTOR,
        owner: PSID,
    }

    impl SecurityInformation {
        pub fn from_file<T: AsRef<OsStr>>(
            path: T,
            security_information: SECURITY_INFORMATION,
        ) -> Result<Self, io::Error> {
            Self::from_object(path, SE_FILE_OBJECT, security_information)
        }

        pub fn from_object<T: AsRef<OsStr>>(
            object_name: T,
            object_type: SE_OBJECT_TYPE,
            security_information: SECURITY_INFORMATION,
        ) -> Result<Self, io::Error> {
            let mut u16_path: Vec<u16> = object_name.as_ref().encode_wide().collect();
            u16_path.push(0u16);

            let mut security_descriptor = ptr::null_mut();
            let mut owner = ptr::null_mut();

            let status = unsafe {
                GetNamedSecurityInfoW(
                    u16_path.as_ptr(),
                    object_type,
                    security_information,
                    &mut owner,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut security_descriptor,
                )
            };

            if status != ERROR_SUCCESS {
                return Err(std::io::Error::from_raw_os_error(status as i32));
            }

            Ok(SecurityInformation {
                security_descriptor: security_descriptor as *mut _,
                owner,
            })
        }

        pub fn owner(&self) -> Option<&SID> {
            unsafe { (self.owner as *const SID).as_ref() }
        }

        // TODO: Can be expanded with `group()`, `dacl()`, and `sacl()`.
    }

    impl Drop for SecurityInformation {
        fn drop(&mut self) {
            unsafe { LocalFree(self.security_descriptor as HANDLE) };
        }
    }

    fn is_well_known_sid(sid: &SID, well_known_sid_type: WELL_KNOWN_SID_TYPE) -> bool {
        unsafe { IsWellKnownSid(sid as *const SID as *mut _, well_known_sid_type) == 1 }
    }
}
