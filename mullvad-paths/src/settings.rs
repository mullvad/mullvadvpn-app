use {ErrorKind, Result, ResultExt};

use std::env;
use std::path::PathBuf;

pub fn get_settings_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_SETTINGS_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_settings_dir(),
    }
}

#[cfg(unix)]
fn get_default_settings_dir() -> Result<PathBuf> {
    let dir = PathBuf::from("/etc/mullvad-daemon");
    ::std::fs::create_dir_all(&dir).chain_err(|| ErrorKind::CreateDirFailed)?;
    Ok(dir)
}

#[cfg(windows)]
fn get_default_settings_dir() -> Result<PathBuf> {
    ::app_dirs::app_root(::app_dirs::AppDataType::UserConfig, &::APP_INFO)
        .chain_err(|| ErrorKind::CreateDirFailed)
}
