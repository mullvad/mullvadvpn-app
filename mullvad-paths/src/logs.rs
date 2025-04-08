use crate::{windows, Result};
use std::{env, path::PathBuf};

/// Creates and returns the logging directory pointed to by `MULLVAD_LOG_DIR`, or the default
/// one if that variable is unset.
pub fn log_dir() -> Result<PathBuf> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = Some(PermissionsExt::from_mode(0o755));
        crate::create_and_return(get_log_dir, permissions)
    }
    #[cfg(target_os = "windows")]
    {
        crate::create_and_return(get_log_dir, Some(windows::UserPermissions::read_only()))
    }
}

/// Get the logging directory, but don't try to create it.
pub fn get_log_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_LOG_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_log_dir(),
    }
}

#[cfg(unix)]
pub fn get_default_log_dir() -> Result<PathBuf> {
    let dir = PathBuf::from("/var/log").join(crate::PRODUCT_NAME);
    Ok(dir)
}

#[cfg(windows)]
pub fn get_default_log_dir() -> Result<PathBuf> {
    let dir = crate::get_allusersprofile_dir()?.join(crate::PRODUCT_NAME);
    Ok(dir)
}
