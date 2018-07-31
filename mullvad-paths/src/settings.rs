use Result;

use std::env;
use std::path::PathBuf;

/// Creates and returns the settings directory pointed to by `MULLVAD_SETTINGS_DIR`, or the default
/// one if that variable is unset.
pub fn settings_dir() -> Result<PathBuf> {
    ::create_and_return(get_settings_dir)
}

fn get_settings_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_SETTINGS_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_settings_dir().map(|dir| dir.join(::PRODUCT_NAME)),
    }
}

fn get_default_settings_dir() -> Result<PathBuf> {
    #[cfg(unix)]
    {
        Ok(PathBuf::from("/etc"))
    }
    #[cfg(windows)]
    {
        ::dirs::data_local_dir().ok_or_else(|| ::ErrorKind::FindDirError.into())
    }
}
