use crate::{Result, UserPermissions};
use std::{env, path::PathBuf};

/// Permissions of the cache directory.
const PERMISSIONS: UserPermissions = UserPermissions {
    read: true,
    write: false,
    execute: true,
};

/// Creates and returns the cache directory pointed to by `MULLVAD_CACHE_DIR`, or the default
/// one if that variable is unset.
#[cfg(not(windows))]
pub fn cache_dir() -> Result<PathBuf> {
    crate::create_dir(get_cache_dir()?, Some(PERMISSIONS))
}

/// Creates and returns the cache directory pointed to by `MULLVAD_CACHE_DIR`, or the default
/// one if that variable is unset.
#[cfg(windows)]
pub fn cache_dir() -> Result<PathBuf> {
    use std::fs;
    if let Some(path) = env::var_os("MULLVAD_CACHE_DIR") {
        let path = PathBuf::from(path);
        return crate::windows::create_dir(path, Some(PERMISSIONS));
    }

    let path = crate::windows::data_dir()?.join("cache");
    fs::create_dir(&path)
        .map_err(|e| crate::Error::CreateDirFailed(format!("Failed to create {path:?}"), e))?;
    Ok(path)
}

pub fn get_cache_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_CACHE_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_cache_dir(),
    }
}

#[cfg(target_os = "linux")]
pub fn get_default_cache_dir() -> Result<PathBuf> {
    let dir = PathBuf::from("/var/cache").join(crate::PRODUCT_NAME);
    Ok(dir)
}

#[cfg(windows)]
pub fn get_default_cache_dir() -> Result<PathBuf> {
    Ok(crate::windows::get_data_dir()?.join("cache"))
}

#[cfg(target_os = "macos")]
pub fn get_default_cache_dir() -> Result<PathBuf> {
    let dir = std::path::Path::new("/Library/Caches").join(crate::PRODUCT_NAME);
    Ok(dir)
}
