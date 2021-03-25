use log::{debug, error, info};
use mullvad_types::{
    relay_constraints::{BridgeSettings, BridgeState, RelaySettingsUpdate},
    settings::{DnsOptions, Settings},
    wireguard::RotationInterval,
};
use std::{
    fs::{self, File},
    io,
    ops::Deref,
    path::{Path, PathBuf},
};
use talpid_types::ErrorExt;


const SETTINGS_FILE: &str = "settings.json";


#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Unable to remove settings file {}", _0)]
    #[cfg(not(target_os = "android"))]
    DeleteError(String, #[error(source)] io::Error),

    #[error(display = "Unable to serialize settings to JSON")]
    SerializeError(#[error(source)] serde_json::Error),

    #[error(display = "Unable to write settings to {}", _0)]
    WriteError(String, #[error(source)] io::Error),
}

#[derive(err_derive::Error, Debug)]
enum LoadSettingsError {
    #[error(display = "Cannot find settings file")]
    FileNotFound,

    #[error(display = "Unable to read settings file")]
    Other(#[error(source)] io::Error),

    #[error(display = "Unable to parse settings file")]
    ParseError(#[error(source)] mullvad_types::settings::Error),

    #[cfg(windows)]
    #[error(display = "Failed to restore Windows update backup")]
    WinMigrationError(#[error(source)] windows::Error),
}


#[derive(Debug)]
pub struct SettingsPersister {
    settings: Settings,
    path: PathBuf,
}

impl SettingsPersister {
    /// Loads user settings from file. If no file is present it returns the defaults.
    pub fn load(settings_dir: &Path) -> Self {
        let path = settings_dir.join(SETTINGS_FILE);
        let (mut settings, mut should_save) = Self::load_settings(&path);

        // Force IPv6 to be enabled on Android
        if cfg!(target_os = "android") {
            should_save |=
                Self::update_field(&mut settings.tunnel_options.generic.enable_ipv6, true);
        }

        let mut persister = SettingsPersister { settings, path };

        if should_save {
            if let Err(error) = persister.save() {
                error!(
                    "{}",
                    error.display_chain_with_msg("Failed to save updated settings")
                );
            }
        }

        persister
    }

    fn load_settings(path: &Path) -> (Settings, bool) {
        let error = match Self::load_settings_from_file(path) {
            Ok(value) => return value,
            Err(error) => error,
        };

        #[cfg(windows)]
        let error = if let LoadSettingsError::FileNotFound = error {
            info!(
                "No settings file found. Attempting migration from Windows update backup location"
            );
            match windows::migrate_after_windows_update() {
                Ok(Some(())) => match Self::load_settings_from_file(path) {
                    Ok(value) => return value,
                    Err(error) => error,
                },
                Ok(None) => LoadSettingsError::FileNotFound,
                Err(error) => LoadSettingsError::WinMigrationError(error),
            }
        } else {
            error
        };

        if let LoadSettingsError::FileNotFound = error {
            info!("No settings were found. Using defaults.");
        } else {
            info!(
                "{}",
                error.display_chain_with_msg("Failed to load settings. Using defaults.")
            );
        }

        (Settings::default(), true)
    }

    fn load_settings_from_file(path: &Path) -> Result<(Settings, bool), LoadSettingsError> {
        info!("Loading settings from {}", path.display());

        let settings_bytes = fs::read(path).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                LoadSettingsError::FileNotFound
            } else {
                LoadSettingsError::Other(error)
            }
        })?;

        Settings::load_from_bytes(&settings_bytes)
            .map(|settings| (settings, false))
            .or_else(|_| {
                Settings::migrate_from_bytes(&settings_bytes).map(|settings| (settings, true))
            })
            .map_err(LoadSettingsError::ParseError)
    }

    /// Serializes the settings and saves them to the file it was loaded from.
    fn save(&mut self) -> Result<(), Error> {
        debug!("Writing settings to {}", self.path.display());
        let mut file = File::create(&self.path)
            .map_err(|e| Error::WriteError(self.path.display().to_string(), e))?;

        serde_json::to_writer_pretty(&mut file, &self.settings).map_err(Error::SerializeError)?;
        file.sync_all()
            .map_err(|e| Error::WriteError(self.path.display().to_string(), e))
    }

    /// Resets default settings
    #[cfg(not(target_os = "android"))]
    pub fn reset(&mut self) -> Result<(), Error> {
        self.settings = Settings::default();
        self.save().or_else(|e| {
            log::error!(
                "{}",
                e.display_chain_with_msg("Unable to save default settings")
            );
            log::info!("Will attempt to remove settings file");
            fs::remove_file(&self.path)
                .map_err(|e| Error::DeleteError(self.path.display().to_string(), e))
        })
    }

    pub fn to_settings(&self) -> Settings {
        self.settings.clone()
    }

    /// Changes account number to the one given. Also saves the new settings to disk.
    /// The boolean in the Result indicates if the account token changed or not
    pub fn set_account_token(&mut self, account_token: Option<String>) -> Result<bool, Error> {
        let should_save = self.settings.set_account_token(account_token);
        self.update(should_save)
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Result<bool, Error> {
        let should_save = self.settings.update_relay_settings(update);
        self.update(should_save)
    }

    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Result<bool, Error> {
        let should_save = Self::update_field(&mut self.settings.allow_lan, allow_lan);
        self.update(should_save)
    }

    pub fn set_block_when_disconnected(
        &mut self,
        block_when_disconnected: bool,
    ) -> Result<bool, Error> {
        let should_save = Self::update_field(
            &mut self.settings.block_when_disconnected,
            block_when_disconnected,
        );
        self.update(should_save)
    }

    pub fn set_auto_connect(&mut self, auto_connect: bool) -> Result<bool, Error> {
        let should_save = Self::update_field(&mut self.settings.auto_connect, auto_connect);
        self.update(should_save)
    }

    pub fn set_openvpn_mssfix(&mut self, openvpn_mssfix: Option<u16>) -> Result<bool, Error> {
        let should_save = Self::update_field(
            &mut self.settings.tunnel_options.openvpn.mssfix,
            openvpn_mssfix,
        );
        self.update(should_save)
    }

    pub fn set_enable_ipv6(&mut self, enable_ipv6: bool) -> Result<bool, Error> {
        let should_save = Self::update_field(
            &mut self.settings.tunnel_options.generic.enable_ipv6,
            enable_ipv6,
        );
        self.update(should_save)
    }

    pub fn set_dns_options(&mut self, options: DnsOptions) -> Result<bool, Error> {
        let should_save =
            Self::update_field(&mut self.settings.tunnel_options.dns_options, options);
        self.update(should_save)
    }

    pub fn set_wireguard_mtu(&mut self, mtu: Option<u16>) -> Result<bool, Error> {
        let should_save =
            Self::update_field(&mut self.settings.tunnel_options.wireguard.options.mtu, mtu);
        self.update(should_save)
    }

    pub fn set_wireguard_rotation_interval(
        &mut self,
        interval: Option<RotationInterval>,
    ) -> Result<bool, Error> {
        let should_save = Self::update_field(
            &mut self.settings.tunnel_options.wireguard.rotation_interval,
            interval,
        );
        self.update(should_save)
    }

    pub fn set_show_beta_releases(&mut self, show_beta_releases: bool) -> Result<bool, Error> {
        let should_save =
            Self::update_field(&mut self.settings.show_beta_releases, show_beta_releases);
        self.update(should_save)
    }

    pub fn set_bridge_settings(&mut self, bridge_settings: BridgeSettings) -> Result<bool, Error> {
        let should_save = Self::update_field(&mut self.settings.bridge_settings, bridge_settings);
        self.update(should_save)
    }

    pub fn set_bridge_state(&mut self, bridge_state: BridgeState) -> Result<bool, Error> {
        let should_save = self.settings.set_bridge_state(bridge_state);
        self.update(should_save)
    }

    fn update_field<T: Eq>(field: &mut T, new_value: T) -> bool {
        if *field != new_value {
            *field = new_value;
            true
        } else {
            false
        }
    }

    fn update(&mut self, should_save: bool) -> Result<bool, Error> {
        if should_save {
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }
}

impl Deref for SettingsPersister {
    type Target = Settings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}


#[cfg(windows)]
mod windows {
    use std::{ffi::OsStr, fs, io, os::windows::ffi::OsStrExt, path::Path, ptr};
    use talpid_types::ErrorExt;
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
        #[error(display = "Unable to find settings directory")]
        FindSettings(#[error(source)] mullvad_paths::Error),

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
    /// Upon success, it returns `Ok(Some(()))` if the migration succeeded, and `Ok(None)` if no
    /// migration was needed.
    pub fn migrate_after_windows_update() -> Result<Option<()>, Error> {
        let destination_settings_dir =
            mullvad_paths::settings_dir().map_err(Error::FindSettings)?;

        let system_appdata_dir = dirs_next::data_local_dir().ok_or(Error::FindAppData)?;
        if !destination_settings_dir.starts_with(system_appdata_dir) {
            return Ok(None);
        }

        let settings_path = destination_settings_dir.join(super::SETTINGS_FILE);
        if settings_path.exists() {
            return Ok(None);
        }

        let mut components = destination_settings_dir.components();
        let prefix = if let Some(prefix) = components.next() {
            prefix
        } else {
            return Ok(None);
        };
        let root = if let Some(root) = components.next() {
            root
        } else {
            return Ok(None);
        };

        let windows_old_dir = Path::new(&prefix).join(&root).join(MIGRATION_DIRNAME);
        let source_settings_dir = Path::new(&windows_old_dir).join(&components);
        if !source_settings_dir.exists() {
            return Ok(None);
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
            fs::create_dir_all(&destination_settings_dir).map_err(Error::IoError)?;
        }

        let mut result = Ok(Some(()));

        for (file, required) in &MIGRATE_FILES {
            let from = source_settings_dir.join(file);
            let to = destination_settings_dir.join(file);

            log::debug!("Migrating {} to {}", from.display(), to.display());

            match fs::copy(&from, &to) {
                Ok(_) => {
                    let _ = fs::remove_file(from);
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

        if let Err(error) = fs::remove_dir(source_settings_dir) {
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
