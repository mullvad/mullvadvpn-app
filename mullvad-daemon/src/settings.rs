use log::info;
use mullvad_types::settings::SettingsData;
use std::{
    fs::File,
    io::{self, BufReader, Read},
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
