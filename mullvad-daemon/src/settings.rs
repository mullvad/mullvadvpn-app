use log::info;
use mullvad_types::{
    relay_constraints::{BridgeSettings, BridgeState, RelaySettingsUpdate},
    settings::Settings,
};
use std::{
    fs::File,
    io::{self, BufReader, Read},
    ops::Deref,
};
use talpid_types::ErrorExt;

#[cfg(windows)]
use {
    log::{error, warn},
    talpid_core::logging::windows::log_sink,
};


#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Settings operation failed")]
    SettingsError(#[error(source)] mullvad_types::settings::Error),
}

#[derive(Debug)]
enum LoadSettingsError {
    FileNotFound,
    Other,
}


#[derive(Debug)]
pub struct SettingsPersister {
    settings: Settings,
}

impl SettingsPersister {
    /// Loads user settings from file. If no file is present it returns the defaults.
    pub fn load() -> Self {
        let mut settings = Self::load_settings();

        // Force IPv6 to be enabled on Android
        if cfg!(target_os = "android") {
            let _ = settings.set_enable_ipv6(true);
        }

        SettingsPersister { settings }
    }

    fn load_settings() -> Settings {
        Self::load_settings_from_file()
            .or_else(|error| match error {
                #[cfg(windows)]
                LoadSettingsError::FileNotFound => Self::try_load_settings_after_windows_update(),
                _ => Err(error),
            })
            .unwrap_or_else(|_| {
                info!("Failed to load settings, using defaults");
                Settings::default()
            })
    }

    fn load_settings_from_file() -> Result<Settings, LoadSettingsError> {
        let path = Settings::get_settings_path().unwrap();
        let file = File::open(&path).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                LoadSettingsError::FileNotFound
            } else {
                LoadSettingsError::Other
            }
        })?;

        info!("Loading settings from {}", path.display());
        let mut settings_bytes = vec![];
        BufReader::new(file)
            .read_to_end(&mut settings_bytes)
            .map_err(|_| LoadSettingsError::Other)?;

        Settings::load_from_bytes(&settings_bytes)
            .or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to parse settings file")
                );
                Settings::migrate_from_bytes(&settings_bytes)
            })
            .map_err(|_| LoadSettingsError::Other)
    }

    #[cfg(windows)]
    fn try_load_settings_after_windows_update() -> Result<Settings, LoadSettingsError> {
        info!("No settings file found. Attempting migration from Windows Update backup location");

        if Self::migrate_after_windows_update() {
            let result = Self::load_settings_from_file();

            match &result {
                Ok(_) => info!("Successfully loaded migrated settings"),
                Err(_) => warn!("Failed to load migrated settings, using defaults"),
            }

            result
        } else {
            Err(LoadSettingsError::Other)
        }
    }

    #[cfg(windows)]
    fn migrate_after_windows_update() -> bool {
        match unsafe {
            ffi::WinUtil_MigrateAfterWindowsUpdate(Some(log_sink), b"Settings migrator\0".as_ptr())
        } {
            ffi::WinUtilMigrationStatus::Success => {
                info!("Migration completed successfully");
                true
            }
            ffi::WinUtilMigrationStatus::Aborted => {
                error!("Migration was aborted to avoid overwriting current settings");
                false
            }
            ffi::WinUtilMigrationStatus::NothingToMigrate => {
                info!("Could not migrate settings - no backup present");
                false
            }
            ffi::WinUtilMigrationStatus::Failed | _ => {
                error!("Migration failed");
                false
            }
        }
    }

    /// Resets default settings
    #[cfg(not(target_os = "android"))]
    pub fn reset(&mut self) -> Result<(), Error> {
        self.settings.reset()
    }

    pub fn to_settings(&self) -> Settings {
        self.settings.clone()
    }

    /// Changes account number to the one given. Also saves the new settings to disk.
    /// The boolean in the Result indicates if the account token changed or not
    pub fn set_account_token(&mut self, account_token: Option<String>) -> Result<bool, Error> {
        Ok(self.settings.set_account_token(account_token)?)
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Result<bool, Error> {
        Ok(self.settings.update_relay_settings(update)?)
    }

    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Result<bool, Error> {
        Ok(self.settings.set_allow_lan(allow_lan)?)
    }

    pub fn set_block_when_disconnected(
        &mut self,
        block_when_disconnected: bool,
    ) -> Result<bool, Error> {
        Ok(self
            .settings
            .set_block_when_disconnected(block_when_disconnected)?)
    }

    pub fn set_auto_connect(&mut self, auto_connect: bool) -> Result<bool, Error> {
        Ok(self.settings.set_auto_connect(auto_connect)?)
    }

    pub fn set_openvpn_mssfix(&mut self, openvpn_mssfix: Option<u16>) -> Result<bool, Error> {
        Ok(self.settings.set_openvpn_mssfix(openvpn_mssfix)?)
    }

    pub fn set_enable_ipv6(&mut self, enable_ipv6: bool) -> Result<bool, Error> {
        Ok(self.settings.set_enable_ipv6(enable_ipv6)?)
    }

    pub fn set_wireguard_mtu(&mut self, mtu: Option<u16>) -> Result<bool, Error> {
        Ok(self.settings.set_wireguard_mtu(mtu)?)
    }

    pub fn set_wireguard_rotation_interval(
        &mut self,
        automatic_rotation: Option<u32>,
    ) -> Result<bool, Error> {
        Ok(self
            .settings
            .set_wireguard_rotation_interval(automatic_rotation)?)
    }

    pub fn set_show_beta_releases(&mut self, enabled: bool) -> Result<bool, Error> {
        Ok(self.settings.set_show_beta_releases(enabled)?)
    }

    pub fn set_bridge_settings(&mut self, bridge_settings: BridgeSettings) -> Result<bool, Error> {
        Ok(self.settings.set_bridge_settings(bridge_settings)?)
    }

    pub fn set_bridge_state(&mut self, bridge_state: BridgeState) -> Result<bool, Error> {
        Ok(self.settings.set_bridge_state(bridge_state)?)
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
    use talpid_core::logging::windows::LogSink;

    #[allow(dead_code)]
    #[repr(u32)]
    pub enum WinUtilMigrationStatus {
        Success = 0,
        Aborted = 1,
        NothingToMigrate = 2,
        Failed = 3,
        Dummy = 9001,
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
