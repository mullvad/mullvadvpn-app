use mullvad_types::settings::{Settings, CURRENT_SETTINGS_VERSION};
use std::path::Path;
use talpid_types::ErrorExt;
use tokio::{
    fs,
    io::{self, AsyncWriteExt},
};

mod v1;
mod v2;
mod v3;
mod v4;

const SETTINGS_FILE: &str = "settings.json";
const ACCOUNT_HISTORY_FILE: &str = "account-history.json";

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to read the settings")]
    ReadError(#[error(source)] io::Error),

    #[error(display = "Malformed settings")]
    ParseError(#[error(source)] serde_json::Error),

    #[error(display = "Unable to read any version of the settings")]
    NoMatchingVersion,

    #[error(display = "Unable to serialize settings to JSON")]
    SerializeError(#[error(source)] serde_json::Error),

    #[error(display = "Unable to write new settings")]
    WriteError(#[error(source)] io::Error),

    #[cfg(windows)]
    #[error(display = "Failed to restore Windows update backup")]
    WinMigrationError(#[error(source)] windows::Error),
}

pub type Result<T> = std::result::Result<T, Error>;


trait SettingsMigration {
    fn version_matches(&self, settings: &mut serde_json::Value) -> bool;
    fn migrate(&self, settings: &mut serde_json::Value) -> Result<()>;
}

pub async fn migrate_all(cache_dir: &Path, settings_dir: &Path) -> Result<()> {
    #[cfg(windows)]
    windows::migrate_after_windows_update(settings_dir)
        .await
        .map_err(Error::WinMigrationError)?;

    migrate_account_history_location(cache_dir, settings_dir).await;

    let path = settings_dir.join(SETTINGS_FILE);

    if !path.is_file() {
        return Ok(());
    }

    let settings_bytes = fs::read(&path).await.map_err(Error::ReadError)?;

    let mut settings: serde_json::Value =
        serde_json::from_reader(&settings_bytes[..]).map_err(Error::ParseError)?;

    if !settings.is_object() {
        return Err(Error::NoMatchingVersion);
    }

    {
        let settings: Settings =
            serde_json::from_slice(&settings_bytes[..]).map_err(Error::ParseError)?;
        if settings.get_settings_version() == CURRENT_SETTINGS_VERSION {
            return Ok(());
        }
    }

    let migrations: Vec<Box<dyn SettingsMigration>> = vec![
        Box::new(v1::Migration),
        Box::new(v2::Migration),
        Box::new(v3::Migration),
        Box::new(v4::Migration),
    ];

    for migration in &migrations {
        if !migration.version_matches(&mut settings) {
            continue;
        }
        migration.migrate(&mut settings)?;
    }

    let buffer = serde_json::to_string_pretty(&settings).map_err(Error::SerializeError)?;

    let mut options = fs::OpenOptions::new();
    #[cfg(unix)]
    {
        options.mode(0o600);
    }
    let mut file = options
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .await
        .map_err(Error::WriteError)?;
    file.write_all(&buffer.into_bytes())
        .await
        .map_err(Error::WriteError)?;

    Ok(())
}

async fn migrate_account_history_location(old_dir: &Path, new_dir: &Path) {
    let old_path = old_dir.join(ACCOUNT_HISTORY_FILE);
    let new_path = new_dir.join(ACCOUNT_HISTORY_FILE);
    if !old_path.exists() || new_path.exists() || new_path == old_path {
        return;
    }

    if let Err(error) = fs::copy(&old_path, &new_path).await {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to migrate account history file location")
        );
    } else {
        let _ = fs::remove_file(old_path).await;
    }
}

#[cfg(windows)]
mod windows {
    use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, path::Path, ptr};
    use talpid_types::ErrorExt;
    use tokio::fs;
    use winapi::{
        shared::{minwindef::TRUE, winerror::ERROR_SUCCESS},
        um::{
            accctrl::{SE_FILE_OBJECT, SE_OBJECT_TYPE},
            aclapi::GetNamedSecurityInfoW,
            securitybaseapi::IsWellKnownSid,
            winbase::LocalFree,
            winnt::{
                WinBuiltinAdministratorsSid, WinLocalSystemSid, OWNER_SECURITY_INFORMATION, PSID,
                SECURITY_DESCRIPTOR, SECURITY_INFORMATION, SID, WELL_KNOWN_SID_TYPE,
            },
        },
    };

    const MIGRATION_DIRNAME: &str = "windows.old";
    const MIGRATE_FILES: [(&str, bool); 2] =
        [("settings.json", true), ("account-history.json", false)];

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
            unsafe { LocalFree(self.security_descriptor as *mut _) };
        }
    }

    fn is_well_known_sid(sid: &SID, well_known_sid_type: WELL_KNOWN_SID_TYPE) -> bool {
        unsafe { IsWellKnownSid(sid as *const SID as *mut _, well_known_sid_type) == TRUE }
    }
}
