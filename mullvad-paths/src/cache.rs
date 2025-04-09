use crate::Result;
use std::{env, path::PathBuf};

/// Creates and returns the cache directory pointed to by `MULLVAD_CACHE_DIR`, or the default
/// one if that variable is unset.
pub fn cache_dir() -> Result<PathBuf> {
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let permissions = None;
    #[cfg(target_os = "macos")]
    let permissions = Some(std::os::unix::fs::PermissionsExt::from_mode(0o755));

    #[cfg(target_os = "windows")]
    let permissions = Some(crate::windows::UserPermissions {
        read: true,
        write: false,
        execute: true,
    });

    crate::create_and_return(get_cache_dir, permissions)
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
    let dir = crate::get_allusersprofile_dir()?
        .join(crate::PRODUCT_NAME)
        .join("cache");
    Ok(dir)
}

#[cfg(target_os = "macos")]
pub fn get_default_cache_dir() -> Result<PathBuf> {
    let dir = std::path::Path::new("/Library/Caches").join(crate::PRODUCT_NAME);
    Ok(dir)
}
