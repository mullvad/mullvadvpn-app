use log::info;
use std::ops::{Deref, DerefMut};

#[cfg(windows)]
use {
    log::{error, warn},
    mullvad_types::settings::Error as SettingsError,
    std::io::ErrorKind,
    talpid_core::logging::windows::log_sink,
};

pub use mullvad_types::settings::*;

#[derive(Debug)]
pub struct SettingsPersister {
    settings: Settings,
}

impl SettingsPersister {
    pub fn load() -> Self {
        let settings = match Settings::load() {
            Ok(mut settings) => {
                // Force IPv6 to be enabled on Android
                if cfg!(target_os = "android") {
                    let _ = settings.set_enable_ipv6(true);
                }
                settings
            }
            #[cfg(windows)]
            Err(SettingsError::ReadError(ref _path, ref e)) if e.kind() == ErrorKind::NotFound => {
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
