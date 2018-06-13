use {ErrorKind, Result, ResultExt};

use std::env;
use std::fs;
use std::path::PathBuf;

/// Creates and returns the cache directory pointed to by `MULLVAD_CACHE_DIR`, or the default
/// one if that variable is unset.
pub fn cache_dir() -> Result<PathBuf> {
    let dir = get_cache_dir()?;
    fs::create_dir_all(&dir).chain_err(|| ErrorKind::CreateDirFailed(dir.clone()))?;
    Ok(dir)
}

fn get_cache_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_CACHE_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_cache_dir(),
    }
}

#[cfg(target_os = "linux")]
fn get_default_cache_dir() -> Result<PathBuf> {
    Ok(PathBuf::from("/var/cache/mullvad-daemon"))
}

#[cfg(any(target_os = "macos", windows))]
fn get_default_cache_dir() -> Result<PathBuf> {
    Ok(::app_dirs::get_app_root(
        ::app_dirs::AppDataType::UserCache,
        &::metadata::APP_INFO,
    )?)
}
