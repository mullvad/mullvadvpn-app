#![cfg(not(target_os = "android"))]

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::fs;
use std::{io, path::PathBuf};

#[cfg(windows)]
use crate::windows::create_dir_recursive_with_permissions;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create directory {0}")]
    CreateDirFailed(String, #[source] io::Error),

    #[error("Failed to set directory permissions on {0}")]
    SetDirPermissionFailed(String, #[source] io::Error),

    #[cfg(any(windows, target_os = "macos"))]
    #[error("Not able to find requested directory")]
    FindDirError,

    #[cfg(windows)]
    #[error("Missing %ALLUSERSPROFILE% environment variable")]
    NoProgramDataDir,

    #[cfg(windows)]
    #[error("Failed to create security attributes")]
    GetSecurityAttributes(#[source] io::Error),

    #[error("Device data directory has not been set")]
    NoDataDir,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
const PRODUCT_NAME: &str = "mullvad-vpn";

#[cfg(windows)]
pub const PRODUCT_NAME: &str = "Mullvad VPN";

#[cfg(windows)]
fn get_allusersprofile_dir() -> Result<PathBuf> {
    match std::env::var_os("ALLUSERSPROFILE") {
        Some(dir) => Ok(PathBuf::from(&dir)),
        None => Err(Error::NoProgramDataDir),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
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

/// Create a directory Windows. If `user_permissions` is `None`, the directory will be created
/// without modifying permissions, using [`std::fs::create_dir_all`]. If `user_permissions` is
/// `Some`, the directory will be created with the specified permissions.
#[cfg(windows)]
fn create_and_return(
    dir_fn: fn() -> Result<PathBuf>,
    user_permissions: Option<windows::UserPermissions>,
) -> Result<PathBuf> {
    let dir = dir_fn()?;
    if let Some(user_permissions) = user_permissions {
        create_dir_recursive_with_permissions(&dir, user_permissions)?;
    } else {
        std::fs::create_dir_all(&dir).map_err(|e| {
            Error::CreateDirFailed(
                format!("Could not create directory at {}", dir.display()),
                e,
            )
        })?;
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
pub mod windows;
