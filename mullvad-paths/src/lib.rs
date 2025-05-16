#![cfg(not(target_os = "android"))]

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::fs;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::path::Path;
use std::{io, path::PathBuf};

#[cfg(windows)]
use crate::windows::create_dir_recursive;

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

    #[error("Device data directory has not been set")]
    NoDataDir,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
const PRODUCT_NAME: &str = "mullvad-vpn";

#[cfg(windows)]
pub const PRODUCT_NAME: &str = "Mullvad VPN";

#[cfg(unix)]
#[derive(Clone, Copy, PartialEq)]
enum Permissions {
    /// Do not set any particular permissions. They will be inherited instead.
    Any,
    /// Only root should have write access. Other users will have
    /// read and execute permissions (0o755).
    ReadExecOnly,
}

#[cfg(unix)]
impl Permissions {
    fn fs_permissions(self) -> Option<fs::Permissions> {
        match self {
            Permissions::Any => None,
            Permissions::ReadExecOnly => Some(std::os::unix::fs::PermissionsExt::from_mode(0o755)),
        }
    }
}

#[cfg(windows)]
fn get_allusersprofile_dir() -> Result<PathBuf> {
    match std::env::var_os("ALLUSERSPROFILE") {
        Some(dir) => Ok(PathBuf::from(&dir)),
        None => Err(Error::NoProgramDataDir),
    }
}

#[cfg(unix)]
fn create_and_return(dir: PathBuf, permissions: Permissions) -> Result<PathBuf> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    let mut dir_builder = fs::DirBuilder::new();
    let fs_perms = permissions.fs_permissions();
    if let Some(fs_perms) = fs_perms.as_ref() {
        dir_builder.mode(fs_perms.mode());
    }
    match dir_builder.create(&dir) {
        Ok(()) => Ok(dir),
        // The directory already exists
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            // If the permissions are wrong, delete the directory
            if !dir_is_root_owned(&dir, fs_perms.as_ref())? {
                fs::remove_dir_all(&dir)
                    .or_else(|err| {
                        // ENOTDIR: If the path is not a directory, try to remove the file
                        if err.raw_os_error() == Some(20) {
                            fs::remove_file(&dir)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|e| Error::RemoveDir(dir.display().to_string(), e))?;
                // Try to create it again
                return create_and_return(dir, permissions);
            }
            // Correct permissions, so we're done
            Ok(dir)
        }
        // Fail on any other error
        Err(error) => Err(Error::CreateDirFailed(dir.display().to_string(), error)),
    }
}

#[cfg(unix)]
fn dir_is_root_owned(dir: &Path, perms: Option<&fs::Permissions>) -> Result<bool> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    const RELEVANT_BITS: u32 = 0o777;

    let meta = fs::symlink_metadata(&dir)
        .map_err(|e| Error::GetDirPermissionFailed(dir.display().to_string(), e))?;
    let matching_perms = perms
        .map(|perms| (perms.mode() & RELEVANT_BITS) == (meta.permissions().mode() & RELEVANT_BITS))
        .unwrap_or(true);
    Ok(matching_perms && meta.uid() == 0)
}

#[cfg(windows)]
fn create_and_return(dir: PathBuf, set_security_permissions: bool) -> Result<PathBuf> {
    create_dir_recursive(&dir, set_security_permissions)?;
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
