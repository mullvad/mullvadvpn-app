use {ErrorKind, Result, ResultExt};

use std::env;
use std::path::PathBuf;

pub fn get_log_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_LOG_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_log_dir(),
    }
}

#[cfg(unix)]
fn get_default_log_dir() -> Result<PathBuf> {
    let dir = PathBuf::from("/var/log/mullvad-daemon");
    ::std::fs::create_dir_all(&dir).chain_err(|| ErrorKind::CreateDirFailed)?;
    Ok(dir)
}

#[cfg(windows)]
fn get_default_log_dir() -> Result<PathBuf> {
    let program_data_dir = Path::new(
        ::std::env::var_os("ALLUSERSPROFILE").ok_or_else(|| ErrorKind::NoProgramDataDir)?,
    );
    Ok(program_data_dir.join(::PRODUCT_NAME))
}
