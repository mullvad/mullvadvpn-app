extern crate app_dirs;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

use app_dirs::AppInfo;

pub const PRODUCT_NAME: &str = "Mullvad VPN";

pub const APP_INFO: AppInfo = AppInfo {
    name: PRODUCT_NAME,
    author: "Mullvad",
};

error_chain! {
    errors {
        CreateDirFailed { description("Failed to create directory") }
    }
}

mod cache;
pub use cache::get_cache_dir;

mod resources;
pub use resources::get_resource_dir;

mod settings;
pub use settings::get_settings_dir;
