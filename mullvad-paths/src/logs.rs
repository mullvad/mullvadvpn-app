use crate::Result;
use std::{env, path::PathBuf};

/// Creates and returns the logging directory pointed to by `MULLVAD_LOG_DIR`, or the default
/// one if that variable is unset.
pub fn log_dir() -> Result<PathBuf> {
    let permissions = Some(crate::UserPermissions {
        read: true,
        write: false,
        // Unix: Make directory contents readable
        execute: cfg!(unix),
    });

    crate::create_dir(get_log_dir()?, permissions)
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
    let dir = crate::windows::get_allusersprofile_dir()?.join(crate::PRODUCT_NAME);
    Ok(dir)
}

/// Returns the directory where any Mullvad-related frontend stores its logs.
/// Logging to this folder does not require root-permissions.
#[cfg(not(target_os = "android"))]
pub fn frontend_log_dir() -> Result<PathBuf> {
    use crate::Error;
    let mullvad_dir = cfg_select! {
        any(target_os = "linux", target_os = "windows") => { "Mullvad VPN/logs" }
        target_os = "macos" => { "Library/Logs/Mullvad VPN" }
    };
    local_log_dir()
        .map(|config_dir| config_dir.join(mullvad_dir))
        .map_err(Error::FindDirError)
}

fn local_log_dir() -> std::io::Result<PathBuf> {
    use std::io;
    // NOTE: These invocations to different `dirs` functions is due to Electron's app.getPath API:
    // https://www.electronjs.org/docs/latest/api/app#appgetpathname.
    cfg_select! {
        target_os = "linux" => { dirs::config_local_dir() }
        target_os = "macos" => { dirs::home_dir() }
        target_os = "windows" => { dirs::data_local_dir() }
    }
    .ok_or(io::Error::from(io::ErrorKind::NotFound))
}
