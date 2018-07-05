#[cfg(any(windows, target_os = "macos"))]
extern crate app_dirs;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

use std::path::PathBuf;

error_chain! {
    errors {
        CreateDirFailed(path: PathBuf) {
            description("Failed to create directory")
            display("Failed to create directory {}", path.display())
        }
        #[cfg(windows)]
        NoProgramDataDir { description("Missing %ALLUSERSPROFILE% environment variable") }
    }
    foreign_links {
        AppDirs(app_dirs::AppDirsError) #[cfg(any(windows, target_os = "macos"))];
    }
}

#[cfg(any(windows, target_os = "macos"))]
mod metadata {
    use app_dirs::AppInfo;

    pub const PRODUCT_NAME: &str = "Mullvad VPN";

    pub const APP_INFO: AppInfo = AppInfo {
        name: PRODUCT_NAME,
        author: "Mullvad",
    };
}

#[cfg(windows)]
fn get_program_data_dir() -> Result<PathBuf> {
    use std::{env, path::Path};
    match env::var_os("ALLUSERSPROFILE") {
        Some(dir) => Ok(Path::new(&dir).join(::metadata::PRODUCT_NAME)),
        None => bail!(ErrorKind::NoProgramDataDir),
    }
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
