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

#[cfg(windows)]
pub fn get_default_settings_dir() -> Result<PathBuf> {
    let dir = crate::windows::get_system_service_appdata()
        .map_err(|error| {
            log::error!("Failed to obtain system app data path: {error}");
            crate::Error::FindDirError
        })?
        .join(crate::PRODUCT_NAME);
    Ok(dir)
}

#[cfg(unix)]
pub fn get_default_settings_dir() -> Result<PathBuf> {
    let dir = PathBuf::from("/etc").join(crate::PRODUCT_NAME);
    Ok(dir)
}
