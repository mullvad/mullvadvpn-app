use log::info;
use mullvad_types::{
    relay_constraints::{BridgeSettings, BridgeState, RelaySettingsUpdate},
    settings::SettingsData,
};
use std::{
    fs::File,
    io::{self, BufReader, Read},
    ops::Deref,
};

#[cfg(windows)]
use {
    log::{error, warn},
    talpid_core::logging::windows::log_sink,
};

pub use mullvad_types::settings::Error;

#[derive(Clone, Debug)]
pub struct Settings {
    data: SettingsData,
}

impl Settings {
    /// Loads user settings from file. If no file is present it returns the defaults.
    pub fn load() -> Self {
        let mut data = Self::load_data()
            .or_else(|error| match error.kind() {
                #[cfg(windows)]
                io::ErrorKind::NotFound => {
                    if Self::migrate_after_windows_update() {
                        let result = Self::load_data();

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
                SettingsData::default()
            });

        // Force IPv6 to be enabled on Android
        if cfg!(target_os = "android") {
            let _ = data.set_enable_ipv6(true);
        }

        Settings { data }
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

    fn load_data() -> Result<SettingsData, io::Error> {
        let path = SettingsData::get_settings_path().unwrap();
        let file = File::open(&path)?;

        info!("Loading settings from {}", path.display());
        let mut settings_bytes = vec![];
        BufReader::new(file).read_to_end(&mut settings_bytes)?;

        SettingsData::load_from_bytes(&settings_bytes)
            .ok()
            .or_else(|| SettingsData::migrate_from_bytes(&settings_bytes).ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to parse settings data"))
    }

    /// Resets default settings
    #[cfg(not(target_os = "android"))]
    pub fn reset(&mut self) -> Result<(), Error> {
        self.data.reset()
    }

    pub fn to_data(&self) -> SettingsData {
        self.data.clone()
    }

    /// Changes account number to the one given. Also saves the new settings to disk.
    /// The boolean in the Result indicates if the account token changed or not
    pub fn set_account_token(&mut self, account_token: Option<String>) -> Result<bool, Error> {
        self.data.set_account_token(account_token)
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Result<bool, Error> {
        self.data.update_relay_settings(update)
    }

    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Result<bool, Error> {
        self.data.set_allow_lan(allow_lan)
    }

    pub fn set_block_when_disconnected(
        &mut self,
        block_when_disconnected: bool,
    ) -> Result<bool, Error> {
        self.data
            .set_block_when_disconnected(block_when_disconnected)
    }

    pub fn set_auto_connect(&mut self, auto_connect: bool) -> Result<bool, Error> {
        self.data.set_auto_connect(auto_connect)
    }

    pub fn set_openvpn_mssfix(&mut self, openvpn_mssfix: Option<u16>) -> Result<bool, Error> {
        self.data.set_openvpn_mssfix(openvpn_mssfix)
    }

    pub fn set_enable_ipv6(&mut self, enable_ipv6: bool) -> Result<bool, Error> {
        self.data.set_enable_ipv6(enable_ipv6)
    }

    pub fn set_wireguard_mtu(&mut self, mtu: Option<u16>) -> Result<bool, Error> {
        self.data.set_wireguard_mtu(mtu)
    }

    pub fn set_wireguard_rotation_interval(
        &mut self,
        automatic_rotation: Option<u32>,
    ) -> Result<bool, Error> {
        self.data
            .set_wireguard_rotation_interval(automatic_rotation)
    }

    pub fn set_show_beta_releases(&mut self, enabled: bool) -> Result<bool, Error> {
        self.data.set_show_beta_releases(enabled)
    }

    pub fn set_bridge_settings(&mut self, bridge_settings: BridgeSettings) -> Result<bool, Error> {
        self.data.set_bridge_settings(bridge_settings)
    }

    pub fn set_bridge_state(&mut self, bridge_state: BridgeState) -> Result<bool, Error> {
        self.data.set_bridge_state(bridge_state)
    }
}

impl Deref for Settings {
    type Target = SettingsData;

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
