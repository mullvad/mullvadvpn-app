use {ErrorKind, Result, ResultExt};

use std::fs;
use std::path::PathBuf;

pub fn get_cache_dir() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let dir = PathBuf::from("/var/cache/mullvad-daemon");
        fs::create_dir_all(&dir).chain_err(|| ErrorKind::NoCacheDir)?;
        Ok(dir)
    }
    #[cfg(any(target_os = "macos", windows))]
    {
        use mullvad_metadata::APP_INFO;
        ::app_dirs::app_root(::app_dirs::AppDataType::UserCache, &APP_INFO)
            .chain_err(|| ErrorKind::NoCacheDir)
    }
}
