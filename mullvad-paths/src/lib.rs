#[cfg(any(windows, target_os = "macos"))]
extern crate dirs;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

use std::fs;
use std::path::PathBuf;

error_chain! {
    errors {
        CreateDirFailed(path: PathBuf) {
            description("Failed to create directory")
            display("Failed to create directory {}", path.display())
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

fn create_and_return(dir_fn: fn() -> Result<PathBuf>) -> Result<PathBuf> {
    let dir = dir_fn()?;
    fs::create_dir_all(&dir).chain_err(|| ErrorKind::CreateDirFailed(dir.clone()))?;
    Ok(dir)
}

mod cache;
pub use cache::cache_dir;

mod logs;
pub use logs::{get_log_dir, log_dir};

pub mod resources;
pub use resources::get_resource_dir;

mod rpc_address;
pub use rpc_address::get_rpc_address_path;

mod settings;
pub use settings::settings_dir;
