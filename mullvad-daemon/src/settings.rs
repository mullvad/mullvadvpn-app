use log::{debug, error, info};
use mullvad_types::{
    relay_constraints::{BridgeSettings, BridgeState, RelaySettingsUpdate},
    settings::{Settings, TunnelOptions},
};
use std::{
    fs::File,
    io::{self, BufReader, Read},
    ops::Deref,
    path::{Path, PathBuf},
};
use talpid_types::ErrorExt;

#[cfg(not(target_os = "android"))]
use std::fs;

#[cfg(windows)]
use {log::warn, talpid_core::logging::windows::log_sink};


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


#[derive(Clone, Debug)]
pub struct SettingsPersister {
    data: Settings,
    path: PathBuf,
}

impl SettingsPersister {
    /// Loads user settings from file. If no file is present it returns the defaults.
    pub fn load(settings_dir: &Path) -> Self {
        let path = settings_dir.join(SETTINGS_FILE);
        let (data, mut should_save) = Self::load_data(&path)
            .or_else(|error| match error.kind() {
                #[cfg(windows)]
                io::ErrorKind::NotFound => {
                    if Self::migrate_after_windows_update() {
                        let result = Self::load_data(&path);

                        match &result {
                            Ok(_) => info!("Successfully loaded migrated settings"),
                            Err(_) => warn!("Failed to load migrated settings, using defaults"),
                        }

                        result
                    } else {
                        info!("Failed to migrate settings, using defaults");
                        Err(error)
                    }
                }
                _ => Err(error),
            })
            .unwrap_or_else(|_| {
                info!("Failed to load settings, using defaults");
                (Settings::default(), true)
            });

        let mut settings = SettingsPersister { data, path };

        // Force IPv6 to be enabled on Android
        if cfg!(target_os = "android") && settings.tunnel_options.generic.enable_ipv6 == false {
            settings.data.tunnel_options.generic.enable_ipv6 = true;
            should_save = true;
        }

        if should_save {
            if let Err(error) = settings.save() {
                error!(
                    "{}",
                    error.display_chain_with_msg("Failed to save updated settings")
                );
            }
        }

        settings
    }

    #[cfg(windows)]
    fn migrate_after_windows_update() -> bool {
        info!("No settings file found. Attempting migration from Windows Update backup location");

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

    fn load_data(path: &Path) -> Result<(Settings, bool), io::Error> {
        let file = File::open(&path)?;

        info!("Loading settings from {}", path.display());
        let mut settings_bytes = vec![];
        BufReader::new(file).read_to_end(&mut settings_bytes)?;

        Settings::load_from_bytes(&settings_bytes)
            .ok()
            .map(|data| (data, false))
            .or_else(|| {
                Settings::migrate_from_bytes(&settings_bytes)
                    .ok()
                    .map(|data| (data, true))
            })
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to parse settings data"))
    }

    /// Serializes the settings and saves them to the file it was loaded from.
    fn save(&mut self) -> Result<(), Error> {
        debug!("Writing settings to {}", self.path.display());
        let mut file = File::create(&self.path)
            .map_err(|e| Error::WriteError(self.path.display().to_string(), e))?;

        serde_json::to_writer_pretty(&mut file, &self.data).map_err(Error::SerializeError)?;
        file.sync_all()
            .map_err(|e| Error::WriteError(self.path.display().to_string(), e))
    }

    /// Resets default settings
    #[cfg(not(target_os = "android"))]
    pub fn reset(&mut self) -> Result<(), Error> {
        self.data = Settings::default();
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

    pub fn to_data(&self) -> Settings {
        self.data.clone()
    }

    /// Changes account number to the one given. Also saves the new settings to disk.
    /// The boolean in the Result indicates if the account token changed or not
    pub fn set_account_token(&mut self, account_token: Option<String>) -> Result<bool, Error> {
        let should_save = self.data.set_account_token(account_token);
        self.update(should_save)
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Result<bool, Error> {
        let should_save = self.data.update_relay_settings(update);
        self.update(should_save)
    }

    pub fn get_allow_lan(&self) -> bool {
        self.data.allow_lan
    }

    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Result<bool, Error> {
        let should_save = Self::update_field(&mut self.data.allow_lan, allow_lan);
        self.update(should_save)
    }

    pub fn get_block_when_disconnected(&self) -> bool {
        self.data.block_when_disconnected
    }

    pub fn set_block_when_disconnected(
        &mut self,
        block_when_disconnected: bool,
    ) -> Result<bool, Error> {
        let should_save = Self::update_field(
            &mut self.data.block_when_disconnected,
            block_when_disconnected,
        );
        self.update(should_save)
    }

    pub fn get_auto_connect(&self) -> bool {
        self.data.auto_connect
    }

    pub fn set_auto_connect(&mut self, auto_connect: bool) -> Result<bool, Error> {
        let should_save = Self::update_field(&mut self.data.auto_connect, auto_connect);
        self.update(should_save)
    }

    pub fn get_tunnel_options(&self) -> &TunnelOptions {
        &self.data.tunnel_options
    }

    pub fn set_openvpn_mssfix(&mut self, openvpn_mssfix: Option<u16>) -> Result<bool, Error> {
        let should_save =
            Self::update_field(&mut self.data.tunnel_options.openvpn.mssfix, openvpn_mssfix);
        self.update(should_save)
    }

    pub fn set_enable_ipv6(&mut self, enable_ipv6: bool) -> Result<bool, Error> {
        let should_save = Self::update_field(
            &mut self.data.tunnel_options.generic.enable_ipv6,
            enable_ipv6,
        );
        self.update(should_save)
    }

    pub fn set_wireguard_mtu(&mut self, mtu: Option<u16>) -> Result<bool, Error> {
        let should_save = Self::update_field(&mut self.data.tunnel_options.wireguard.mtu, mtu);
        self.update(should_save)
    }

    pub fn set_wireguard_rotation_interval(
        &mut self,
        automatic_rotation: Option<u32>,
    ) -> Result<bool, Error> {
        let should_save = Self::update_field(
            &mut self.data.tunnel_options.wireguard.automatic_rotation,
            automatic_rotation,
        );
        self.update(should_save)
    }

    pub fn get_show_beta_releases(&self) -> Option<bool> {
        self.data.show_beta_releases
    }

    pub fn set_show_beta_releases(&mut self, show_beta_releases: bool) -> Result<bool, Error> {
        let should_save =
            Self::update_field(&mut self.data.show_beta_releases, Some(show_beta_releases));
        self.update(should_save)
    }

    pub fn get_bridge_settings(&self) -> &BridgeSettings {
        &self.bridge_settings
    }

    pub fn set_bridge_settings(&mut self, bridge_settings: BridgeSettings) -> Result<bool, Error> {
        let should_save = Self::update_field(&mut self.data.bridge_settings, bridge_settings);
        self.update(should_save)
    }

    pub fn set_bridge_state(&mut self, bridge_state: BridgeState) -> Result<bool, Error> {
        let should_save = self.data.set_bridge_state(bridge_state);
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
        &self.data
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
