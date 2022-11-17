#![deny(rust_2018_idioms)]

use std::{fs, io, path::PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to create directory {}", _0)]
    CreateDirFailed(String, #[error(source)] io::Error),

    #[error(display = "Failed to set directory permissions on {}", _0)]
    SetDirPermissionFailed(String, #[error(source)] io::Error),

    #[cfg(any(windows, target_os = "macos"))]
    #[error(display = "Not able to find requested directory")]
    FindDirError,

    #[cfg(windows)]
    #[error(display = "Missing %ALLUSERSPROFILE% environment variable")]
    NoProgramDataDir,

    #[cfg(all(windows, feature = "deduce-system-service"))]
    #[error(display = "Failed to deduce system service directory")]
    FailedToFindSystemServiceDir(#[error(source)] io::Error),
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
const PRODUCT_NAME: &str = "mullvad-vpn";

#[cfg(windows)]
pub const PRODUCT_NAME: &str = "Mullvad VPN";

#[cfg(target_os = "android")]
const APP_PATH: &str = "/data/data/net.mullvad.mullvadvpn";

#[cfg(windows)]
fn get_allusersprofile_dir() -> Result<PathBuf> {
    match std::env::var_os("ALLUSERSPROFILE") {
        Some(dir) => Ok(PathBuf::from(&dir)),
        None => Err(Error::NoProgramDataDir),
    }
}

fn create_and_return(
    dir_fn: fn() -> Result<PathBuf>,
    permissions: Option<fs::Permissions>,
) -> Result<PathBuf> {
    let dir = dir_fn()?;
    fs::create_dir_all(&dir).map_err(|e| Error::CreateDirFailed(dir.display().to_string(), e))?;
    if let Some(permissions) = permissions {
        fs::set_permissions(&dir, permissions)
            .map_err(|e| Error::SetDirPermissionFailed(dir.display().to_string(), e))?;
    }
    Ok(dir)
}

mod cache;
pub use crate::cache::{cache_dir, get_cache_dir, get_default_cache_dir};

mod logs;
pub use crate::logs::{get_default_log_dir, get_log_dir, log_dir};

pub mod resources;
pub use crate::resources::{get_default_resource_dir, get_resource_dir};

mod rpc_socket;
pub use crate::rpc_socket::{get_default_rpc_socket_path, get_rpc_socket_path};

mod settings;
pub use crate::settings::{get_default_settings_dir, settings_dir};

#[cfg(windows)]
mod windows;
