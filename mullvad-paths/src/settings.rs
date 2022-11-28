use crate::Result;
use std::{env, path::PathBuf};

/// Creates and returns the settings directory pointed to by `MULLVAD_SETTINGS_DIR`, or the default
/// one if that variable is unset.
pub fn settings_dir() -> Result<PathBuf> {
    crate::create_and_return(get_settings_dir, None)
}

fn get_settings_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_SETTINGS_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_settings_dir(),
    }
}

pub fn get_default_settings_dir() -> Result<PathBuf> {
    #[cfg(not(target_os = "android"))]
    {
        let dir;
        #[cfg(unix)]
        {
            dir = Ok(PathBuf::from("/etc"));
        }
        #[cfg(windows)]
        {
            dir = crate::windows::get_system_service_appdata().map_err(|error| {
                log::error!("Failed to obtain system app data path: {error}");
                crate::Error::FindDirError
            })
        }
        dir.map(|dir| dir.join(crate::PRODUCT_NAME))
    }
    #[cfg(target_os = "android")]
    {
        Ok(PathBuf::from(crate::APP_PATH))
    }
}
