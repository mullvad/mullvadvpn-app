use log::info;
use mullvad_types::settings::Settings;
use std::{
    fs::File,
    io::{self, BufReader, Read},
    ops::{Deref, DerefMut},
};
use talpid_types::ErrorExt;

#[cfg(windows)]
use {
    log::{error, warn},
    talpid_core::logging::windows::log_sink,
};

pub use mullvad_types::settings::Error;


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
    pub fn load() -> Self {
        let settings = match Self::load_settings_from_file() {
            Ok(mut settings) => {
                // Force IPv6 to be enabled on Android
                if cfg!(target_os = "android") {
                    let _ = settings.set_enable_ipv6(true);
                }
                settings
            }
            #[cfg(windows)]
            Err(LoadSettingsError::FileNotFound) => {
                if Self::migrate_after_windows_update() {
                    match Settings::load() {
                        Ok(settings) => {
                            info!("Successfully loaded migrated settings");
                            settings
                        }
                        Err(_) => {
                            warn!("Failed to load migrated settings, using defaults");
                            Settings::default()
                        }
                    }
                } else {
                    info!("Failed to migrate settings, using defaults");
                    Settings::default()
                }
            }
            Err(_) => {
                info!("Failed to load settings, using defaults");
                Settings::default()
            }
        };

        SettingsPersister { settings }
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

    pub fn to_settings(&self) -> Settings {
        self.settings.clone()
    }
}

impl Deref for SettingsPersister {
    type Target = Settings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

impl DerefMut for SettingsPersister {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.settings
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
