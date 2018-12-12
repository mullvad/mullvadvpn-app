#[macro_use]
extern crate error_chain;

use std::{fs, path::PathBuf};

error_chain! {
    errors {
        CreateDirFailed(path: PathBuf) {
            description("Failed to create directory")
            display("Failed to create directory {}", path.display())
        }
        SetDirPermissionFailed(path: PathBuf) {
            description("Failed to set directory permissions")
            display("Failed to set directory permissions on {}", path.display())
        }
        #[cfg(any(windows, target_os = "macos"))]
        FindDirError { description("Not able to find requested directory" )}
        #[cfg(windows)]
        NoProgramDataDir { description("Missing %ALLUSERSPROFILE% environment variable") }
    }
}

#[cfg(unix)]
const PRODUCT_NAME: &str = "mullvad-vpn";

#[cfg(windows)]
const PRODUCT_NAME: &str = "Mullvad VPN";


#[cfg(windows)]
fn get_allusersprofile_dir() -> Result<PathBuf> {
    match std::env::var_os("ALLUSERSPROFILE") {
        Some(dir) => Ok(PathBuf::from(&dir)),
        None => bail!(ErrorKind::NoProgramDataDir),
    }
}

fn create_and_return(
    dir_fn: fn() -> Result<PathBuf>,
    permissions: Option<fs::Permissions>,
) -> Result<PathBuf> {
    let dir = dir_fn()?;
    fs::create_dir_all(&dir).chain_err(|| ErrorKind::CreateDirFailed(dir.clone()))?;
    if let Some(permissions) = permissions {
        fs::set_permissions(&dir, permissions)
            .chain_err(|| ErrorKind::SetDirPermissionFailed(dir.clone()))?;
    }
    Ok(dir)
}

mod cache;
pub use crate::cache::{cache_dir, get_default_cache_dir};

mod logs;
pub use crate::logs::{get_default_log_dir, get_log_dir, log_dir};

pub mod resources;
pub use crate::resources::{get_default_resource_dir, get_resource_dir};

mod rpc_socket;
pub use crate::rpc_socket::{get_default_rpc_socket_path, get_rpc_socket_path};

mod settings;
pub use crate::settings::{get_default_settings_dir, settings_dir};
