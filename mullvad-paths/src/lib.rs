#![cfg(not(target_os = "android"))]

use std::io;

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::PRODUCT_NAME;

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use unix::PRODUCT_NAME;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create directory {0}")]
    CreateDirFailed(String, #[source] io::Error),

    #[error("Failed to remove directory {0}")]
    RemoveDir(String, #[source] io::Error),

    #[error("Failed to get directory permissions on {0}")]
    GetDirPermissionFailed(String, #[source] io::Error),

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

    #[cfg(windows)]
    #[error(transparent)]
    ContainsNul(#[from] widestring::error::ContainsNul<u16>),

    #[error("Device data directory has not been set")]
    NoDataDir,
}

#[derive(Clone, Copy)]
pub struct UserPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl UserPermissions {
    pub fn read_only() -> Self {
        UserPermissions {
            read: true,
            write: false,
            execute: false,
        }
    }
}

#[cfg(unix)]
use unix::create_dir;

#[cfg(windows)]
use windows::create_dir;

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
