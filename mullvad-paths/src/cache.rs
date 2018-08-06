use Result;

use std::env;
use std::path::PathBuf;

/// Creates and returns the cache directory pointed to by `MULLVAD_CACHE_DIR`, or the default
/// one if that variable is unset.
pub fn cache_dir() -> Result<PathBuf> {
    ::create_and_return(get_cache_dir)
}

fn get_cache_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_CACHE_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_cache_dir().map(|dir| dir.join(::PRODUCT_NAME)),
    }
}

fn get_default_cache_dir() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        Ok(PathBuf::from("/var/cache"))
    }
    #[cfg(any(target_os = "macos", windows))]
    {
        ::dirs::cache_dir().ok_or_else(|| ::ErrorKind::FindDirError.into())
    }
}
