use Result;

use std::env;
use std::path::PathBuf;

/// Creates and returns the logging directory pointed to by `MULLVAD_LOG_DIR`, or the default
/// one if that variable is unset.
pub fn log_dir() -> Result<PathBuf> {
    ::create_and_return(get_log_dir)
}

/// Get the logging directory, but don't try to create it.
pub fn get_log_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_LOG_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_log_dir().map(|dir| dir.join(::PRODUCT_NAME)),
    }
}

fn get_default_log_dir() -> Result<PathBuf> {
    #[cfg(unix)]
    {
        Ok(PathBuf::from("/var/log"))
    }
    #[cfg(windows)]
    {
        ::get_allusersprofile_dir()
    }
}
