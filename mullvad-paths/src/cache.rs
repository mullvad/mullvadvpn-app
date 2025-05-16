use crate::Result;
use std::{env, path::PathBuf};

/// Creates and returns the cache directory pointed to by `MULLVAD_CACHE_DIR`, or the default
/// one if that variable is unset.
pub fn cache_dir() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    let permissions = crate::unix::Permissions::Any;
    #[cfg(target_os = "macos")]
    let permissions = crate::unix::Permissions::ReadExecOnly;
    #[cfg(target_os = "windows")]
    let permissions = true;
    crate::create_and_return(get_cache_dir()?, permissions)
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
