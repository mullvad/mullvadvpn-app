#[cfg(windows)]
use log::{error, warn};

use log::info;

#[cfg(windows)]
use mullvad_types::settings::Error as SettingsError;

pub use mullvad_types::settings::*;

#[cfg(windows)]
use std::io::ErrorKind;

#[cfg(windows)]
use std::{
    os::raw::{c_char, c_void},
    ptr,
};

pub fn load() -> Settings {
    match Settings::load() {
        Ok(settings) => settings,
        #[cfg(windows)]
        Err(SettingsError::ReadError(ref _path, ref e)) if e.kind() == ErrorKind::NotFound => {
            info!(
                "No settings file found. Attempting migration from Windows Update backup location"
            );
            if migrate_after_windows_update() {
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
    }
}

#[cfg(windows)]
fn migrate_after_windows_update() -> bool {
    match unsafe { ffi::WinUtil_MigrateAfterWindowsUpdate(Some(log_sink), ptr::null_mut()) } {
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

#[cfg(windows)]
extern "system" fn log_sink(msg: *const c_char, _ctx: *mut c_void) {
    use std::ffi::CStr;
    if msg.is_null() {
        error!("Log message from FFI boundary is NULL");
    } else {
        error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
    }
}

#[cfg(windows)]
mod ffi {
    use super::*;

    #[allow(dead_code)]
    #[repr(u32)]
    pub enum WinUtilMigrationStatus {
        Success = 0,
        Aborted = 1,
        NothingToMigrate = 2,
        Failed = 3,
        Dummy = 9001,
    }

    type ErrorSink = extern "system" fn(msg: *const c_char, ctx: *mut c_void);

    #[allow(non_snake_case)]
    extern "system" {
        #[link_name = "WinUtil_MigrateAfterWindowsUpdate"]
        pub fn WinUtil_MigrateAfterWindowsUpdate(
            sink: Option<ErrorSink>,
            sink_context: *mut c_void,
        ) -> WinUtilMigrationStatus;
    }
}
