use crate::Result;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{env, path::PathBuf};

/// Creates and returns the logging directory pointed to by `MULLVAD_LOG_DIR`, or the default
/// one if that variable is unset.
pub fn log_dir() -> Result<PathBuf> {
    #[cfg(unix)]
    let permissions = Some(PermissionsExt::from_mode(0o755));
    #[cfg(not(unix))]
    let permissions = Some(
        crate::windows::create_security_attributes_with_admin_full_access_user_read_only()?
    );
    crate::create_and_return(get_log_dir, permissions)
}

/// Get the logging directory, but don't try to create it.
pub fn get_log_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_LOG_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_log_dir(),
    }
}

pub fn get_default_log_dir() -> Result<PathBuf> {
    #[cfg(not(target_os = "android"))]
    {
        let dir;
        #[cfg(unix)]
        {
            dir = Ok(PathBuf::from("/var/log"));
        }
        #[cfg(windows)]
        {
            dir = crate::get_allusersprofile_dir();
        }
        dir.map(|dir| dir.join(crate::PRODUCT_NAME))
    }
    #[cfg(target_os = "android")]
    {
        Ok(PathBuf::from(crate::APP_PATH))
    }
}
