use {ErrorKind, Result, ResultExt};

use std::env;
use std::fs;
use std::path::PathBuf;

/// Creates and returns the logging directory pointed to by `MULLVAD_LOG_DIR`, or the default
/// one if that variable is unset.
pub fn log_dir() -> Result<PathBuf> {
    let dir = get_log_dir()?;
    fs::create_dir_all(&dir).chain_err(|| ErrorKind::CreateDirFailed(dir.clone()))?;
    Ok(dir)
}

/// Get the logging directory, but don't try to create it.
pub fn get_log_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_LOG_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_log_dir(),
    }
}

#[cfg(unix)]
fn get_default_log_dir() -> Result<PathBuf> {
    Ok(PathBuf::from("/var/log/mullvad-daemon"))
}

#[cfg(windows)]
fn get_default_log_dir() -> Result<PathBuf> {
    let program_data_dir =
        Path::new(env::var_os("ALLUSERSPROFILE").ok_or_else(|| ErrorKind::NoProgramDataDir)?);
    Ok(program_data_dir.join(::metadata::PRODUCT_NAME))
}
