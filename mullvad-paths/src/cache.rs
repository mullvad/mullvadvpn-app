use {ErrorKind, Result, ResultExt};

use std::path::PathBuf;
use std::env;

pub fn get_cache_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_CACHE_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_cache_dir()
    }
}

#[cfg(target_os = "linux")]
fn get_default_cache_dir() -> Result<PathBuf> {
    use std::fs;

    let dir = PathBuf::from("/var/cache/mullvad-daemon");
    fs::create_dir_all(&dir).chain_err(|| ErrorKind::NoCacheDir)?;
    Ok(dir)
}

#[cfg(any(target_os = "macos", windows))]
fn get_default_cache_dir() -> Result<PathBuf> {
    ::app_dirs::app_root(::app_dirs::AppDataType::UserCache, &::APP_INFO)
        .chain_err(|| ErrorKind::NoCacheDir)
}
