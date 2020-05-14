use log::{debug, error, info};
use mullvad_types::{
    relay_constraints::{BridgeSettings, BridgeState, RelaySettingsUpdate},
    settings::Settings,
};
use std::{
    fs::{self, File},
    io,
    ops::Deref,
    path::{Path, PathBuf},
};
use talpid_types::ErrorExt;

#[cfg(windows)]
use talpid_core::logging::windows::log_sink;


static SETTINGS_FILE: &str = "settings.json";


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
    #[error(display = "Failed to restore Windows Update backup: {}", _0)]
    WinMigrationError(ffi::WinUtilMigrationStatus),
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
        Self::load_settings_from_file(path)
            .or_else(|error| match error {
                #[cfg(windows)]
                LoadSettingsError::FileNotFound => {
                    Self::try_load_settings_after_windows_update(path)
                }
                _ => Err(error),
            })
            .unwrap_or_else(|error| {
                info!(
                    "{}",
                    error.display_chain_with_msg("Failed to load settings. Using defaults.")
                );
                (Settings::default(), true)
            })
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

    #[cfg(windows)]
    fn try_load_settings_after_windows_update(
        path: &Path,
    ) -> Result<(Settings, bool), LoadSettingsError> {
        info!("No settings file found. Attempting migration from Windows Update backup location");

        Self::migrate_after_windows_update()?;
        Self::load_settings_from_file(path)
    }

    #[cfg(windows)]
    fn migrate_after_windows_update() -> Result<(), LoadSettingsError> {
        unsafe {
            ffi::WinUtil_MigrateAfterWindowsUpdate(Some(log_sink), b"Settings migrator\0".as_ptr())
                .into()
        }
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

    pub fn set_wireguard_mtu(&mut self, mtu: Option<u16>) -> Result<bool, Error> {
        let should_save = Self::update_field(&mut self.settings.tunnel_options.wireguard.mtu, mtu);
        self.update(should_save)
    }

    pub fn set_wireguard_rotation_interval(
        &mut self,
        automatic_rotation: Option<u32>,
    ) -> Result<bool, Error> {
        let should_save = Self::update_field(
            &mut self.settings.tunnel_options.wireguard.automatic_rotation,
            automatic_rotation,
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
mod ffi {
    use std::fmt;
    use talpid_core::logging::windows::LogSink;

    #[derive(Debug)]
    #[allow(dead_code)]
    #[repr(u32)]
    pub enum WinUtilMigrationStatus {
        Success = 0,
        Aborted = 1,
        NothingToMigrate = 2,
        Failed = 3,
    }

    impl fmt::Display for WinUtilMigrationStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use WinUtilMigrationStatus::*;
            write!(
                f,
                "{}",
                match self {
                    Success => "Migration completed successfully",
                    Aborted => "Migration was aborted to avoid overwriting current settings",
                    NothingToMigrate => "Could not migrate settings - no backup present",
                    Failed => "Migration failed",
                }
            )
        }
    }

    impl Into<Result<(), super::LoadSettingsError>> for WinUtilMigrationStatus {
        fn into(self) -> Result<(), super::LoadSettingsError> {
            match self {
                WinUtilMigrationStatus::Success => Ok(()),
                val => Err(super::LoadSettingsError::WinMigrationError(val)),
            }
        }
    }

    #[allow(non_snake_case)]
    extern "system" {
        #[link_name = "WinUtil_MigrateAfterWindowsUpdate"]
        pub fn WinUtil_MigrateAfterWindowsUpdate(
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> WinUtilMigrationStatus;
    }
}
