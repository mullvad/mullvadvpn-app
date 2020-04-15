use log::info;
use mullvad_types::settings::SettingsData;
use std::{
    io,
    ops::{Deref, DerefMut},
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
    pub fn load() -> Self {
        let mut data = match Self::load_data() {
            Ok(settings) => settings,
            #[cfg(windows)]
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if Self::migrate_after_windows_update() {
                    match SettingsData::load() {
                        Ok(settings) => {
                            info!("Successfully loaded migrated settings");
                            settings
                        }
                        Err(_) => {
                            warn!("Failed to load migrated settings, using defaults");
                            SettingsData::default()
                        }
                    }
                } else {
                    info!("Failed to migrate settings, using defaults");
                    SettingsData::default()
                }
            }
            Err(_) => {
                info!("Failed to load settings, using defaults");
                SettingsData::default()
            }
        };

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
        SettingsData::load().map_err(|error| match error {
            Error::ReadError(_, io_error) => io_error,
            _ => io::Error::new(io::ErrorKind::Other, "Failed to load settings"),
        })
    }

    pub fn to_data(&self) -> SettingsData {
        self.data.clone()
    }
}

impl Deref for Settings {
    type Target = SettingsData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Settings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
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
