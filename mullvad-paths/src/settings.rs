use {ErrorKind, Result, ResultExt};

use std::env;
use std::fs;
use std::path::PathBuf;

/// Creates and returns the settings directory pointed to by `MULLVAD_SETTINGS_DIR`, or the default
/// one if that variable is unset.
pub fn settings_dir() -> Result<PathBuf> {
    let dir = get_settings_dir()?;
    fs::create_dir_all(&dir).chain_err(|| ErrorKind::CreateDirFailed)?;
    Ok(dir)
}

fn get_settings_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_SETTINGS_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_settings_dir(),
    }
}

#[cfg(unix)]
fn get_default_settings_dir() -> Result<PathBuf> {
    Ok(PathBuf::from("/etc/mullvad-daemon"))
}

#[cfg(windows)]
fn get_default_settings_dir() -> Result<PathBuf> {
    ::app_dirs::get_app_root(::app_dirs::AppDataType::UserConfig, &::metadataAPP_INFO)
        .chain_err(|| ErrorKind::CreateDirFailed)
}
